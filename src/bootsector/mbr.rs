use crate::{
    bootsector::boot::{BootInfo, BootType, Partition, PartitionType},
    utils::nom_helper::{
        Endian, nom_unsigned_four_bytes, nom_unsigned_one_byte, nom_unsigned_two_bytes,
    },
};
use nom::bytes::complete::take;

pub(crate) fn parse_mbr(data: &[u8]) -> nom::IResult<&[u8], BootInfo> {
    let boot_binary_code: u16 = 440;
    let (input, _binary) = take(boot_binary_code)(data)?;

    let (input, _disk_id) = nom_unsigned_four_bytes(input, Endian::Le)?;
    let (mut input, _reserved) = nom_unsigned_two_bytes(input, Endian::Le)?;

    let mut info = BootInfo {
        boot_type: BootType::MasterBootRecord,
        partitions: Vec::new(),
    };

    let partition_size: u8 = 16;
    let max_partitions = 4;
    let mut count = 0;

    while count < max_partitions {
        let (remaining, partition) = take(partition_size)(input)?;
        input = remaining;

        let (_, (part, is_gpt)) = parse_partition(partition)?;
        println!("{part:?}");
        info.partitions.push(part);

        if is_gpt {
            info.boot_type = BootType::GuidPartitionTable;
        }
        count += 1;
    }
    let (input, _valid_bootsector) = nom_unsigned_two_bytes(input, Endian::Le)?;
    Ok((input, info))
}

fn parse_partition(data: &[u8]) -> nom::IResult<&[u8], (Partition, bool)> {
    let (input, bootable) = nom_unsigned_one_byte(data, Endian::Le)?;
    let sector_size: u8 = 3;
    let (input, sector_start) = take(sector_size)(input)?;
    let (input, partition_type) = nom_unsigned_one_byte(input, Endian::Le)?;
    let (input, sector_last) = take(sector_size)(input)?;

    let (input, first_logical_offset) = nom_unsigned_four_bytes(input, Endian::Le)?;
    let (input, sectors_in_partition) = nom_unsigned_four_bytes(input, Endian::Le)?;
    let last_sector_offset =
        ((sector_last[0] as u32) << 16) + ((sector_last[1] as u32) << 8) + sector_last[2] as u32;
    let first_sector_offset =
        ((sector_start[0] as u32) << 16) + ((sector_start[1] as u32) << 8) + sector_start[2] as u32;

    let mut is_gpt = false;
    let part = Partition {
        partition_type: get_partition_type(partition_type),
        partition_type_value: partition_type,
        first_sector_offset,
        last_sector_offset,
        first_logical_offset,
        offset_start: first_logical_offset as u64 * 512,
        sectors_in_partition,
        partition_size: (sectors_in_partition as u64 * 512),
        bootable: bootable == 0x80,
    };

    if part.partition_type == PartitionType::Protective {
        is_gpt = true;
    }

    Ok((input, (part, is_gpt)))
}

fn get_partition_type(part: u8) -> PartitionType {
    match part {
        0x0 => PartitionType::None,
        0x7 | 0x27 => PartitionType::Ntfs,
        0x83 => PartitionType::Linux,
        0xc => PartitionType::Fat32,
        0xee | 0xef => PartitionType::Protective,
        0x5 => PartitionType::Extended,
        _ => PartitionType::Unknown,
    }
}
