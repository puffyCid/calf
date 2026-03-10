use crate::{
    bootsector::{
        gpt::{GuidNames, parse_gpt},
        mbr::{parse_extended, parse_mbr},
    },
    error::CalfError,
    reader::OsReader,
};
use log::error;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct BootInfo {
    pub boot_type: BootType,
    pub partitions: Vec<Partition>,
    pub gpt_partitions: Option<Vec<GptPartition>>,
}

#[derive(Debug, PartialEq)]
pub enum BootType {
    MasterBootRecord,
    GuidPartitionTable,
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub partition_type: PartitionType,
    pub partition_type_value: u8,
    pub first_sector_offset: u32,
    pub last_sector_offset: u32,
    pub first_logical_offset: u32,
    /**Offset to partition data */
    pub offset_start: u64,
    pub sectors_in_partition: u32,
    pub partition_size: u64,
    pub bootable: bool,
}

#[derive(PartialEq, Debug, Clone)]
pub enum PartitionType {
    Ntfs,
    Linux,
    Unknown,
    Fat16,
    Fat32,
    ExFat,
    Protective,
    Extended,
    LinuxSwap,
    LinuxLvm,
    None,
}

#[derive(Debug, Clone, Default)]
pub struct GptPartition {
    pub partition_guid: String,
    pub guid: String,
    pub platform: GuidNames,
    pub first_lba: u64,
    pub last_lba: u64,
    pub attributes: u64,
    pub partition_name: String,
}

/// Get the bootsector info from the QCOW file
pub(crate) fn boot_info<'qcow, 'reader, T: std::io::Seek + std::io::Read>(
    reader: &mut OsReader<'qcow, 'reader, T>,
) -> Result<BootInfo, CalfError> {
    if let Err(err) = reader.seek(SeekFrom::Start(0)) {
        error!("[calf] Could not seek to start for boot info: {err:?}");
        return Err(CalfError::SeekFile);
    }

    // 512 seems to be the most common
    let sector_size = 512;
    let mut mbr_buff = vec![0; sector_size];
    if let Err(err) = reader.read(&mut mbr_buff) {
        error!("[calf] Could not read MBR first {sector_size} bytes: {err:?}");
        return Err(CalfError::ReadFile);
    }

    let mut boot = match parse_mbr(&mbr_buff) {
        Ok((_, result)) => result,
        Err(err) => {
            error!("[calf] Could not parse MBR first {sector_size} bytes: {err:?}");
            return Err(CalfError::ParseMbr);
        }
    };

    if boot.boot_type == BootType::GuidPartitionTable {
        boot.gpt_partitions = Some(gpt_info(reader)?);
        return Ok(boot);
    }

    let mut extra_parts = Vec::new();
    let mut root_offset;
    // Second partition should be the extended type. There is only one
    for part in &boot.partitions {
        if part.partition_type != PartitionType::Extended {
            continue;
        }
        // All additional extended partitions are relative from the first extended partition
        root_offset = part.offset_start;

        if let Err(err) = reader.seek(SeekFrom::Start(part.offset_start)) {
            error!("[calf] Could not seek to extended partition: {err:?}");
            return Err(CalfError::ExtendedPartition);
        }
        let mut mbr_buff = vec![0; sector_size];
        if let Err(err) = reader.read(&mut mbr_buff) {
            error!("[calf] Could not read extended partition {sector_size} bytes: {err:?}");
            return Err(CalfError::ReadFile);
        }

        // We pass the root_offset to ensure any additional extended partition entries are properly setup to point to the absolute offset (root_offset + extended partition relative offset)
        let (mut ext_boot, mut has_extended) =
            match parse_extended(&mbr_buff, root_offset, part.offset_start) {
                Ok((_, result)) => result,
                Err(err) => {
                    error!(
                        "[calf] Could not parse extended partition {sector_size} bytes: {err:?}"
                    );
                    return Err(CalfError::ExtendedPartition);
                }
            };
        extra_parts.append(&mut ext_boot.clone());

        // Extended partitions may have a list that points to more extended partitions. These additional "partitions" are not real partitions they are just extensions of the first extended partition (linked list)
        while has_extended {
            has_extended = false;
            let mut next_ext = Vec::new();
            for entry in &ext_boot {
                if entry.partition_type != PartitionType::Extended {
                    continue;
                }

                // Our next extended partition is always relative from the first extended partition found in the Master Boot Record (MBR)
                if let Err(err) = reader.seek(SeekFrom::Start(entry.offset_start)) {
                    error!("[calf] Could not seek to next extended partition: {err:?}");
                    return Err(CalfError::ExtendedPartition);
                }
                let mut mbr_buff = vec![0; sector_size];
                if let Err(err) = reader.read(&mut mbr_buff) {
                    error!("[calf] Could not read next extended {sector_size} bytes: {err:?}");
                    return Err(CalfError::ReadFile);
                }

                // We pass the root_offset to ensure any additional extended partition entries are properly setup to point to the absolute offset (root_offset + extended partition relative offset)
                // We also need the current absolute offset of our current extended partition to ensure we can properly calculate the offsets for any non-extended partition types
                let (mut ext_boot, more_extended) =
                    match parse_extended(&mbr_buff, root_offset, entry.offset_start) {
                        Ok((_, result)) => result,
                        Err(err) => {
                            error!("[calf] Could not parse MBR first {sector_size} bytes: {err:?}");
                            return Err(CalfError::ExtendedPartition);
                        }
                    };
                has_extended = more_extended;

                if has_extended {
                    next_ext.append(&mut ext_boot.clone());
                }

                extra_parts.append(&mut ext_boot);
            }

            ext_boot = next_ext;
        }
    }
    boot.partitions.append(&mut extra_parts);

    Ok(boot)
}

fn gpt_info<'qcow, 'reader, T: std::io::Seek + std::io::Read>(
    reader: &mut OsReader<'qcow, 'reader, T>,
) -> Result<Vec<GptPartition>, CalfError> {
    if let Err(err) = reader.seek(SeekFrom::Start(0)) {
        error!("[calf] Could not seek to start for GPT boot info: {err:?}");
        return Err(CalfError::SeekFile);
    }

    // 512 seems to be the most common
    let sector_size = 512;
    let gpt_size = 128;
    // GPT most often has limit of 128 partitions each 128 bytes in size
    // Header is most often 512 bytes (one sector)
    // Boot code is also 512 bytes (one sector)
    let size = gpt_size * gpt_size + sector_size + sector_size;
    let mut gpt_buff = vec![0; size];
    if let Err(err) = reader.read(&mut gpt_buff) {
        error!("[calf] Could not read GPT first {size} bytes: {err:?}");
        return Err(CalfError::ReadFile);
    }

    let boot = match parse_gpt(&gpt_buff) {
        Ok((_, result)) => result,
        Err(err) => {
            error!("[calf] Could not parse GPT {sector_size} bytes: {err:?}");
            return Err(CalfError::ParseGpt);
        }
    };

    Ok(boot)
}

#[cfg(test)]
mod tests {
    use crate::{
        bootsector::boot::boot_info,
        calf::{CalfReader, CalfReaderAction, QcowInfo},
        format::header::CalfHeader,
    };
    use std::{fs::File, io::BufReader, path::PathBuf};

    #[test]
    fn test_boot_info() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/qcow/debian13.qcow");

        let reader = File::open(test_location.to_str().unwrap()).unwrap();
        let buf = BufReader::new(reader);

        let mut calf = CalfReader { fs: buf };
        let info = QcowInfo {
            header: calf.header().unwrap(),
            level1_table: calf.level1_entries().unwrap(),
        };
        let mut os_reader = calf.os_reader(&info).unwrap();
        let results = boot_info(&mut os_reader).unwrap();

        assert_eq!(results.partitions.len(), 12);
    }

    #[test]
    #[should_panic(expected = "Level")]
    fn test_boot_info_no_levels() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/qcow/debian13.qcow");

        let reader = File::open(test_location.to_str().unwrap()).unwrap();
        let buf = BufReader::new(reader);

        let mut calf = CalfReader { fs: buf };
        let info = QcowInfo {
            header: calf.header().unwrap(),
            level1_table: Vec::new(),
        };
        let _os_reader = calf.os_reader(&info).unwrap();
    }
}
