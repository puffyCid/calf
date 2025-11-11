use crate::{calf::CalfReader, error::CalfError, utils::read::read_bytes};
use log::error;
use nom::number::complete::{be_u8, be_u32, be_u64};

/// Header info for QCOW file. Only Version 3 supported
/// Header docs: `https://github.com/qemu/qemu/blob/master/docs/interop/qcow2.txt`
#[derive(Debug)]
pub struct Header {
    pub sig: u32,
    pub version: u32,
    pub backing_filename_offset: u64,
    pub backing_filename_size: u32,
    /// Also called: `refcount bits`
    pub cluster_block_bits_count: u32,
    pub size: u64,
    /// Encryption not supported by calf
    pub encryption_method: Encryption,
    pub level_one_table_ref: u32,
    pub level_one_table_offset: u64,
    pub ref_table_offset_count: u64,
    pub ref_table_cluster_count: u32,
    pub snapshots_count: u32,
    pub snapshot_offset: u64,
    pub incompat_flags: Vec<IncompatFlags>,
    pub compat_flags: Vec<CompatFlags>,
    pub auto_clear_flags: Vec<AutoClear>,
    pub ref_count_order: u32,
    pub header_size: u32,
    /// Compression used in QCOW 3 format. Only found if the header size is 112 bytes
    pub compression_method: Compression,
}

#[derive(Debug, PartialEq)]
pub enum Encryption {
    None,
    Aes,
    Luks,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum IncompatFlags {
    Dirty,
    Corrupt,
    DataFile,
    Compression,
    ExtendedL2,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum Compression {
    Zlib,
    Zstd,
    None,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum AutoClear {
    Bitmaps,
    DataFileRaw,
}

#[derive(Debug, PartialEq)]
pub enum CompatFlags {
    LazyRefCounts,
}

pub trait CalfHeader<T: std::io::Seek + std::io::Read> {
    fn header(&mut self) -> Result<Header, CalfError>;
}

impl<T: std::io::Seek + std::io::Read> CalfHeader<T> for CalfReader<T> {
    /// Grab QCOW header info
    fn header(&mut self) -> Result<Header, CalfError> {
        let size = 112;
        let bytes = read_bytes(0, size, &mut self.fs)?;
        let header = match Header::get_header(&bytes) {
            Ok((_, results)) => results,
            Err(err) => {
                error!("[calf] Could not parse the QCOW header: {err:?}");
                return Err(CalfError::Header);
            }
        };

        Ok(header)
    }
}
impl Header {
    /// Parse the QCOW header data
    fn get_header(data: &[u8]) -> nom::IResult<&[u8], Header> {
        let (remaining, sig) = be_u32(data)?;
        let (remaining, version) = be_u32(remaining)?;
        let (remaining, backing_filename_offset) = be_u64(remaining)?;
        let (remaining, backing_filename_size) = be_u32(remaining)?;

        let (remaining, cluster_block_bits_count) = be_u32(remaining)?;
        let (remaining, size) = be_u64(remaining)?;
        let (remaining, encrypt_method) = be_u32(remaining)?;

        let (remaining, level_one_table_ref) = be_u32(remaining)?;
        let (remaining, level_one_table_offset) = be_u64(remaining)?;
        let (remaining, ref_table_offset_count) = be_u64(remaining)?;
        let (remaining, ref_table_cluster_count) = be_u32(remaining)?;

        let (remaining, snapshots_count) = be_u32(remaining)?;
        let (remaining, snapshot_offset) = be_u64(remaining)?;

        let (remaining, incompat_flags) = be_u64(remaining)?;
        let (remaining, compat_flags) = be_u64(remaining)?;
        let (remaining, auto_clear_flags) = be_u64(remaining)?;

        let (remaining, ref_count_order) = be_u32(remaining)?;
        let (remaining, header_size) = be_u32(remaining)?;

        let is_compressed = 112;

        let (remaining, compression_method) = if header_size == is_compressed {
            let (remaining, compress_data) = be_u8(remaining)?;
            if compress_data == 0 {
                (remaining, Compression::Zlib)
            } else if compress_data == 1 {
                (remaining, Compression::Zstd)
            } else {
                error!("[calf] Unknown compression type {compress_data}");
                (remaining, Compression::Unknown)
            }
        } else {
            (remaining, Compression::None)
        };

        let head = Header {
            sig,
            version,
            backing_filename_offset,
            backing_filename_size,
            cluster_block_bits_count,
            size,
            encryption_method: Header::get_encrypt(&encrypt_method),
            level_one_table_ref: level_one_table_ref * 8,
            level_one_table_offset,
            ref_table_offset_count,
            ref_table_cluster_count,
            snapshots_count,
            snapshot_offset,
            incompat_flags: Header::get_incompat_flags(&incompat_flags),
            compat_flags: Header::get_compat_flags(&compat_flags),
            auto_clear_flags: Header::get_auto_clear_flags(&auto_clear_flags),
            ref_count_order,
            header_size,
            compression_method,
        };

        Ok((remaining, head))
    }

    /// Determine encryption type if any
    fn get_encrypt(input: &u32) -> Encryption {
        match input {
            0 => Encryption::None,
            1 => Encryption::Aes,
            2 => Encryption::Luks,
            _ => Encryption::Unknown,
        }
    }

    /// Get any incompatible flags
    fn get_incompat_flags(input: &u64) -> Vec<IncompatFlags> {
        let mut flags = Vec::new();
        if (input & 1) == 1 {
            flags.push(IncompatFlags::Dirty);
        }
        if (input & 2) == 2 {
            flags.push(IncompatFlags::Corrupt);
        }
        if (input & 4) == 4 {
            flags.push(IncompatFlags::DataFile);
        }
        if (input & 8) == 8 {
            flags.push(IncompatFlags::Compression);
        }
        if (input & 16) == 16 {
            flags.push(IncompatFlags::ExtendedL2);
        }

        flags
    }

    /// Get any compatible flags
    fn get_compat_flags(input: &u64) -> Vec<CompatFlags> {
        let mut flags = Vec::new();
        if (input & 1) == 1 {
            flags.push(CompatFlags::LazyRefCounts);
        }

        flags
    }

    /// Get any auto clear flags
    fn get_auto_clear_flags(input: &u64) -> Vec<AutoClear> {
        let mut flags = Vec::new();
        if (input & 1) == 1 {
            flags.push(AutoClear::Bitmaps);
        }
        if (input & 2) == 2 {
            flags.push(AutoClear::DataFileRaw);
        }

        flags
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        calf::CalfReader,
        format::header::{CalfHeader, Compression, Encryption, Header},
    };
    use std::{fs::File, io::BufReader, path::PathBuf};

    #[test]
    fn test_grab_header() {
        let mut test_location = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_location.push("tests/test_data/headers/header_version3.raw");
        let reader = File::open(test_location.to_str().unwrap()).unwrap();
        let buf = BufReader::new(reader);

        let mut calf = CalfReader { fs: buf };
        let result = calf.header().unwrap();

        assert_eq!(result.size, 85899345920);
        assert_eq!(result.cluster_block_bits_count, 16);
        assert_eq!(result.level_one_table_ref, 1280);
        assert_eq!(result.level_one_table_offset, 262144);
        assert_eq!(result.sig, 1363560955);
        assert_eq!(result.compression_method, Compression::Zlib);
    }

    #[test]
    fn test_get_header() {
        let test = [
            81, 70, 73, 251, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0,
            20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 160, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 112, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let (_, result) = Header::get_header(&test).unwrap();

        assert_eq!(result.backing_filename_offset, 0);
        assert_eq!(result.backing_filename_size, 0);
        assert_eq!(result.version, 3);
        assert_eq!(result.snapshot_offset, 0);
        assert_eq!(result.snapshots_count, 0);
    }

    #[test]
    fn test_get_encrypt() {
        let test = [0, 1, 2];
        for entry in test {
            let result = Header::get_encrypt(&entry);
            assert_ne!(result, Encryption::Unknown);
        }
    }

    #[test]
    fn test_get_incompat_flags() {
        let test = [1, 2, 4, 8, 16];
        for entry in test {
            assert!(!Header::get_incompat_flags(&entry).is_empty())
        }
    }

    #[test]
    fn test_get_compat_flags() {
        let test = [1];
        for entry in test {
            assert!(!Header::get_compat_flags(&entry).is_empty())
        }
    }

    #[test]
    fn test_get_auto_clear_flags() {
        let test = [1, 2];
        for entry in test {
            assert!(!Header::get_auto_clear_flags(&entry).is_empty())
        }
    }
}
