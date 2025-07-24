use crate::{
    error::CalfError,
    format::{
        header::{CalfHeader, Compression, Encryption, Header},
        level::{CalfLevel, Level},
    },
    reader::OsReader,
};
use std::io::BufReader;

pub struct CalfReader<T: std::io::Seek + std::io::Read> {
    pub fs: BufReader<T>,
}

pub struct QcowInfo {
    pub header: Header,
    pub level1_table: Vec<Level>,
}

pub trait CalfReaderAction<'qcow, 'reader, T: std::io::Seek + std::io::Read> {
    /// Return QCOW version
    fn version(&mut self) -> Result<u32, CalfError>;
    /// Return QCOW OS size
    fn size(&mut self) -> Result<u64, CalfError>;
    /// Check if QCOW is encrypted
    fn encryption(&mut self) -> Result<Encryption, CalfError>;
    /// Calculate the QCOW cluster size
    fn cluster_size(&mut self) -> Result<u64, CalfError>;
    /// Check if compression is enabled
    fn compression(&mut self) -> Result<Compression, CalfError>;
    /// Get number of QCOW snapshots
    fn snapshots_count(&mut self) -> Result<u32, CalfError>;
    /// Get cluster bits value for QCOW
    fn cluster_bits(&mut self) -> Result<u32, CalfError>;
    fn level1_entries(&mut self) -> Result<Vec<Level>, CalfError>;
    fn os_reader(
        &'reader mut self,
        info: &'qcow QcowInfo,
    ) -> Result<OsReader<'qcow, 'reader, T>, CalfError>;
}

impl<'qcow, 'reader, T: std::io::Seek + std::io::Read> CalfReaderAction<'qcow, 'reader, T>
    for CalfReader<T>
{
    fn version(&mut self) -> Result<u32, CalfError> {
        Ok(self.header()?.version)
    }

    fn size(&mut self) -> Result<u64, CalfError> {
        Ok(self.header()?.size)
    }

    fn encryption(&mut self) -> Result<Encryption, CalfError> {
        Ok(self.header()?.encryption_method)
    }

    fn cluster_size(&mut self) -> Result<u64, CalfError> {
        Ok(1 << self.header()?.cluster_block_bits_count)
    }

    fn compression(&mut self) -> Result<Compression, CalfError> {
        Ok(self.header()?.compression_method)
    }

    fn snapshots_count(&mut self) -> Result<u32, CalfError> {
        Ok(self.header()?.snapshots_count)
    }

    fn cluster_bits(&mut self) -> Result<u32, CalfError> {
        Ok(self.header()?.cluster_block_bits_count)
    }

    fn level1_entries(&mut self) -> Result<Vec<Level>, CalfError> {
        let header = self.header()?;
        self.levels(
            &header.level_one_table_offset,
            &(header.level_one_table_ref * 8),
        )
    }

    fn os_reader(
        &'reader mut self,
        info: &'qcow QcowInfo,
    ) -> Result<OsReader<'qcow, 'reader, T>, CalfError> {
        QcowInfo::setup_reader(info, &mut self.fs)
    }
}

#[cfg(test)]
mod tests {
    use super::CalfReader;
    use crate::calf::{CalfReaderAction, Compression, Encryption};
    use std::{fs::File, io::BufReader, path::PathBuf};

    #[test]
    fn test_calf() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/headers/header_version3.raw");
        let reader = File::open(test_location.to_str().unwrap()).unwrap();
        let buf = BufReader::new(reader);

        let mut calf = CalfReader { fs: buf };
        assert_eq!(calf.version().unwrap(), 3);
        assert_eq!(calf.compression().unwrap(), Compression::Zlib);
        assert_eq!(calf.encryption().unwrap(), Encryption::None);
        assert_eq!(calf.cluster_size().unwrap(), 65536);
        assert_eq!(calf.snapshots_count().unwrap(), 0);
        assert_eq!(calf.size().unwrap(), 85899345920);
    }
}
