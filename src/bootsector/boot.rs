use crate::{bootsector::mbr::parse_mbr, error::CalfError, reader::OsReader};
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct BootInfo {
    pub boot_type: BootType,
    pub partitions: Vec<Partition>,
}

#[derive(Debug)]
pub enum BootType {
    MasterBootRecord,
    GuidPartitionTable,
}

#[derive(Debug)]
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

#[derive(PartialEq, Debug)]
pub enum PartitionType {
    Ntfs,
    Linux,
    Unknown,
    Fat16,
    Fat32,
    ExFat,
    Protective,
    Extended,
    None,
}

pub(crate) fn boot_info<'qcow, 'reader, T: std::io::Seek + std::io::Read>(
    reader: &mut OsReader<'qcow, 'reader, T>,
) -> Result<BootInfo, CalfError> {
    if let Err(err) = reader.seek(SeekFrom::Start(0)) {
        println!("[calf] Could not seek to start for boot info: {err:?}");
        return Err(CalfError::SeekFile);
    }

    // 512 seems to be the most common
    let sector_size = 512;
    let mut mbr_buff = vec![0; sector_size];
    if let Err(err) = reader.read_exact(&mut mbr_buff) {
        println!("[calf] Could not read MBR first {sector_size} bytes: {err:?}");
        return Err(CalfError::ReadFile);
    }

    let boot = match parse_mbr(&mbr_buff) {
        Ok((_, result)) => result,
        Err(err) => {
            println!("[calf] Could not parse MBR first {sector_size} bytes: {err:?}");
            return Err(CalfError::ParseMbr);
        }
    };

    Ok(boot)
}
