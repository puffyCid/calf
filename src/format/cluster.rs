use super::header::Compression;
use std::io::{self, BufReader, Read, Seek, SeekFrom};

pub(crate) fn read_cluster<T: std::io::Seek + std::io::Read>(
    reader: &mut BufReader<T>,
    offset: &u64,
    cluster_size: &u64,
    compression: &Compression,
    is_compressed: &bool,
) -> io::Result<Vec<u8>> {
    if *is_compressed {
        panic!("unsupported right now!");
    }
    if reader.seek(SeekFrom::Start(*offset)).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Seeked past the end of the qcow file when reading the current cluster",
        ));
    }
    let mut buf = vec![0; *cluster_size as usize];
    reader.read(&mut buf)?;
    Ok(buf)
}
