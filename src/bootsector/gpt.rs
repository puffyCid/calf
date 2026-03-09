use crate::utils::strings::{Endian, extract_guid, extract_utf16_string};
use nom::{
    bytes::complete::take,
    number::complete::{le_u32, le_u64},
};

/*
 * TODO:
 * 1. Determine attributes for `GptEntry` values (https://en.wikipedia.org/wiki/GUID_Partition_Table)
 * 2. Tests
 * 3. Figure out way to handle GUID lookups? What does velo or dissect do?
 *    - dissect just has a hard coded list. Seems reasonable
 *    - Python script to create? Or export table from wikipedia?
 * 4. Done?
 */

/// Parse the GPT partition data
pub(crate) fn parse_gpt(data: &[u8]) -> nom::IResult<&[u8], ()> {
    let boot_binary_code: u16 = 512;
    let (input, _binary) = take(boot_binary_code)(data)?;
    let (input, signature) = le_u64(input)?;

    // Should be "EFI PART"
    let sig = 6075990659671082565;
    if signature != sig {
        panic!("not good!");
    }

    let (input, _revision) = le_u32(input)?;
    let (input, header_size) = le_u32(input)?;
    let (input, _crc_hash) = le_u32(input)?;
    let (input, _reserved) = le_u32(input)?;

    // LBA - logical based address
    let (input, current_logical_based_address) = le_u64(input)?;
    let (input, backup_lba) = le_u64(input)?;
    let (input, first_usable_partition) = le_u64(input)?;
    let (input, secondary_partition_table) = le_u64(input)?;

    let guid_size: u8 = 16;
    let (input, guid_bytes) = take(guid_size)(input)?;
    let guid = extract_guid(guid_bytes, Endian::Little);

    let (input, start_lba_array_entries) = le_u64(input)?;
    let (input, number_partitions_in_array) = le_u32(input)?;
    let (input, single_partition_size) = le_u32(input)?;
    let (input, _crc_partitions_hash) = le_u32(input)?;
    // Remaining bytes are reserved. Should be all zeros

    println!("{guid}");
    println!(
        "array entries: {start_lba_array_entries}. Number partitions in array: {number_partitions_in_array}"
    );
    println!("partition size: {single_partition_size}");
    println!("first partition for data: {first_usable_partition}");
    println!("secondary table: {secondary_partition_table}");
    println!("current LBA: {current_logical_based_address}");

    Ok((input, ()))
}

#[derive(Default)]
struct GptEntry {
    partition_guid: String,
    guid: String,
    first_lba: u64,
    last_lba: u64,
    attributes: Vec<Attributes>,
    partition_name: String,
}

enum Attributes {
    PlatformRequired,
    EfiIgnore,
    LegacyBios,
    WindowsReadOnly,
    WindowsShadowCopy,
    WindowsHidden,
    WindowsNoDriveLetter,
    BootFlag,
    ChromeOsBoot,
    ChromeOsNotBootable,
}

fn parse_gpt_entry(data: &[u8]) -> nom::IResult<&[u8], GptEntry> {
    let guid_size: u8 = 16;
    let (input, guid_bytes) = take(guid_size)(data)?;
    let partion_guid = extract_guid(guid_bytes, Endian::Little);
    let (input, guid_bytes) = take(guid_size)(input)?;
    let guid = extract_guid(guid_bytes, Endian::Little);

    let (input, first_lba) = le_u64(input)?;
    let (input, last_lba) = le_u64(input)?;
    let (input, attribute_flags) = le_u64(input)?;

    let name_size: u8 = 72;
    let (input, name_bytes) = take(name_size)(input)?;
    let name = extract_utf16_string(name_bytes);
    println!("{name}");
    println!("part guid: {partion_guid}");
    println!("{guid}");

    Ok((input, GptEntry::default()))
}

#[cfg(test)]
mod tests {
    use crate::bootsector::gpt::{parse_gpt, parse_gpt_entry};
    use std::{fs::read, path::PathBuf};

    #[test]
    fn test_parse_gpt() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/gpt/lba1.raw");
        let bytes = read(test_location.to_str().unwrap()).unwrap();

        let (_, gpt) = parse_gpt(&bytes).unwrap();
    }

    #[test]
    fn test_parse_gpt_bios_entry() {
        let test = [
            72, 97, 104, 33, 73, 100, 111, 110, 116, 78, 101, 101, 100, 69, 70, 73, 78, 100, 243,
            223, 191, 211, 6, 70, 150, 50, 124, 147, 165, 176, 161, 30, 0, 8, 0, 0, 0, 0, 0, 0,
            255, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ];

        let (_, result) = parse_gpt_entry(&test).unwrap();
    }

    #[test]
    fn test_parse_gpt_linux_filesystem_entry() {
        let test = [
            175, 61, 198, 15, 131, 132, 114, 71, 142, 121, 61, 105, 216, 71, 125, 228, 162, 39,
            157, 128, 20, 7, 129, 73, 166, 99, 60, 28, 235, 140, 229, 23, 0, 16, 0, 0, 0, 0, 0, 0,
            255, 247, 127, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];

        let (_, result) = parse_gpt_entry(&test).unwrap();
    }
}
