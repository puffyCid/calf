use std::io::{BufReader, Read, Seek, SeekFrom};

use crate::{
    calf::CalfReader,
    error::CalfError,
    utils::{
        nom_helper::{Endian, nom_unsigned_eight_bytes},
        read::read_bytes,
    },
};
use log::{error, warn};

#[derive(Debug, Clone)]
pub struct Level {
    /// Level 1 table offset is to Level 2 table.  
    /// Level 2 table offset is to cluster block
    pub offset: u64,
    pub is_copied: bool,
    pub is_compressed: bool,
}

pub trait CalfLevel<T: std::io::Seek + std::io::Read> {
    /// Return array of `Levels` at provided offset
    fn levels(&mut self, offset: &u64, size: &u32) -> Result<Vec<Level>, CalfError>;
}

impl<T: std::io::Seek + std::io::Read> CalfLevel<T> for CalfReader<T> {
    fn levels(&mut self, offset: &u64, level_entries: &u32) -> Result<Vec<Level>, CalfError> {
        let bytes = read_bytes(offset, &(*level_entries as u64), &mut self.fs)?;
        let value = match Level::get_levels(&bytes) {
            Ok((_, results)) => results,
            Err(_err) => {
                println!("{_err:?}");
                error!("[calf] Failed to parse level");
                return Err(CalfError::Level);
            }
        };

        Ok(value)
    }
}

pub(crate) fn read_level<T: std::io::Seek + std::io::Read>(
    reader: &mut BufReader<T>,
    cluster_bits: &u32,
    offset: &u64,
) -> Result<Vec<Level>, CalfError> {
    if reader.seek(SeekFrom::Start(*offset)).is_err() {
        error!("[calf] Could not seek to level offset");
        return Err(CalfError::SeekFile);
    }

    println!("cluster bits for read level: {cluster_bits}");
    let mut buf = vec![0; (1 << *cluster_bits) as usize];
    if let Ok(bytes) = reader.read(&mut buf) {
        let levels = match Level::get_levels(&buf) {
            Ok((_, results)) => results,
            Err(_err) => {
                println!("{_err:?}");
                error!("[calf] Failed to parse level");
                return Err(CalfError::Level);
            }
        };

        if bytes != buf.len() {
            warn!(
                "[calf] Bytes read does not equal expected cluster bits size {}",
                1 << *cluster_bits
            );
        }

        return Ok(levels);
    }

    error!("[calf] Could not read to level data");
    Err(CalfError::ReadFile)
}

impl Level {
    /// Parse the `Levels` data
    fn get_levels(data: &[u8]) -> nom::IResult<&[u8], Vec<Level>> {
        let mut input = data;
        let min_size = 8;
        let offset_check = 0xfffffffffffe00;

        let mut levels = Vec::new();
        // Last two bits will determine if the data is compressed
        let is_copied = 0x8000000000000000;
        let is_compressed = 0x4000000000000000;
        while input.len() >= min_size {
            let (remaining, value) = nom_unsigned_eight_bytes(input, Endian::Be)?;
            input = remaining;

            // Even if the offset is 0. Do not skip
            let offset = value & offset_check;
            let level = Level {
                offset,
                is_compressed: value & is_compressed != 0,
                is_copied: value & is_copied != 0,
            };

            levels.push(level);
        }

        Ok((input, levels))
    }
}

#[cfg(test)]
mod tests {
    use super::Level;
    use crate::{calf::CalfReader, format::level::CalfLevel};
    use std::{
        fs::{File, read},
        io::BufReader,
        path::PathBuf,
    };

    #[test]
    fn test_read_level1() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/levels/level1_version3.raw");
        let reader = File::open(test_location.to_str().unwrap()).unwrap();
        let buf = BufReader::new(reader);

        let mut calf = CalfReader { fs: buf };
        let results = calf.levels(&0, &1280).unwrap();
        assert_eq!(results.len(), 160);
        assert_eq!(results[0].offset, 196608);
        assert_eq!(results[0].is_compressed, false);
        assert_eq!(results[0].is_copied, true);
        assert_eq!(results[123].offset, 10878976);
        assert_eq!(results[159].offset, 393216);
        assert_eq!(results[1].offset, 1572864);
    }

    #[test]
    fn test_read_level2() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/levels/level2_version3.raw");
        let reader = File::open(test_location.to_str().unwrap()).unwrap();
        let buf = BufReader::new(reader);

        let mut calf = CalfReader { fs: buf };
        let results = calf.levels(&0, &65536).unwrap();
        assert_eq!(results.len(), 8192);
        assert_eq!(results[0].offset, 327680);
        assert_eq!(results[0].is_compressed, false);
        assert_eq!(results[0].is_copied, true);
        assert_eq!(results[123].offset, 39911424);
        assert_eq!(results[159].offset, 42532864);
        assert_eq!(results[1].offset, 0);
    }

    #[test]
    fn test_grab_level2() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/levels/level2_version3.raw");
        let test = read(test_location.to_str().unwrap()).unwrap();

        let (_, results) = Level::get_levels(&test).unwrap();
        assert_eq!(results.len(), 8192);
        assert_eq!(results[0].offset, 327680);
        assert_eq!(results[0].is_compressed, false);
        assert_eq!(results[0].is_copied, true);
        assert_eq!(results[123].offset, 39911424);
        assert_eq!(results[159].offset, 42532864);
        assert_eq!(results[1].offset, 0);
    }
}
