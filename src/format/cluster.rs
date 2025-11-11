use super::header::Compression;
use log::warn;
use std::io::{self, BufReader, Read, Seek, SeekFrom};

/// Read bytes from the qcow cluster region
pub(crate) fn read_cluster<T: std::io::Seek + std::io::Read>(
    reader: &mut BufReader<T>,
    offset: u64,
    cluster_size: u64,
    compression: &Compression,
    is_compressed: &bool,
) -> io::Result<Vec<u8>> {
    if *is_compressed {
        warn!("[calf] Got compressed data? This is unsupported right now! Type: {compression:?}");
    }
    if reader.seek(SeekFrom::Start(offset)).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Seeked past the end of the qcow file when reading the current cluster",
        ));
    }
    let mut buf = vec![0; cluster_size as usize];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use crate::{
        calf::{CalfReader, CalfReaderAction, QcowInfo},
        format::{
            cluster::read_cluster,
            header::{CalfHeader, Compression},
        },
    };
    use std::{fs::File, io::BufReader, path::PathBuf};

    #[test]
    fn test_read_cluster() {
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
        let bytes = read_cluster(
            &mut os_reader.reader,
            327680,
            65536,
            &Compression::Zlib,
            &false,
        )
        .unwrap();

        assert_eq!(
            bytes[0..305],
            [
                235, 99, 144, 16, 142, 208, 188, 0, 176, 184, 0, 0, 142, 216, 142, 192, 251, 190,
                0, 124, 191, 0, 6, 185, 0, 2, 243, 164, 234, 33, 6, 0, 0, 190, 190, 7, 56, 4, 117,
                11, 131, 198, 16, 129, 254, 254, 7, 117, 243, 235, 22, 180, 2, 176, 1, 187, 0, 124,
                178, 128, 138, 116, 1, 139, 76, 2, 205, 19, 234, 0, 124, 0, 0, 235, 254, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 1, 0, 0, 0, 0, 0, 0, 0, 255, 250, 144,
                144, 246, 194, 128, 116, 5, 246, 194, 112, 116, 2, 178, 128, 234, 121, 124, 0, 0,
                49, 192, 142, 216, 142, 208, 188, 0, 32, 251, 160, 100, 124, 60, 255, 116, 2, 136,
                194, 82, 190, 128, 125, 232, 23, 1, 190, 5, 124, 180, 65, 187, 170, 85, 205, 19,
                90, 82, 114, 61, 129, 251, 85, 170, 117, 55, 131, 225, 1, 116, 50, 49, 192, 137,
                68, 4, 64, 136, 68, 255, 137, 68, 2, 199, 4, 16, 0, 102, 139, 30, 92, 124, 102,
                137, 92, 8, 102, 139, 30, 96, 124, 102, 137, 92, 12, 199, 68, 6, 0, 112, 180, 66,
                205, 19, 114, 5, 187, 0, 112, 235, 118, 180, 8, 205, 19, 115, 13, 90, 132, 210, 15,
                131, 216, 0, 190, 139, 125, 233, 130, 0, 102, 15, 182, 198, 136, 100, 255, 64, 102,
                137, 68, 4, 15, 182, 209, 193, 226, 2, 136, 232, 136, 244, 64, 137, 68, 8, 15, 182,
                194, 192, 232, 2, 102, 137, 4, 102, 161, 96, 124, 102, 9, 192, 117, 78, 102, 161,
                92, 124, 102, 49, 210, 102, 247, 52, 136, 209, 49, 210, 102, 247, 116, 4, 59, 68,
            ]
        );
    }
}
