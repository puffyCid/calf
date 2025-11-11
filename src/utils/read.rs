use crate::error::CalfError;
use log::{error, warn};
use std::io::{BufReader, Read, Seek, SeekFrom};

/// Read bytes from the QCOW file. This is *not* used to read OS bytes within the QCOW file! See instead read_cluster
pub(crate) fn read_bytes<T: std::io::Read + std::io::Seek>(
    offset: u64,
    bytes: u64,
    fs: &mut BufReader<T>,
) -> Result<Vec<u8>, CalfError> {
    if fs.seek(SeekFrom::Start(offset)).is_err() {
        error!("[calf] Could not seek to offset {offset}");
        return Err(CalfError::SeekFile);
    }
    let mut buff_size = vec![0u8; bytes as usize];
    let bytes_read = match fs.read(&mut buff_size) {
        Ok(result) => result,
        Err(err) => {
            error!("[calf] Could not read bytes: {err:?}");
            return Err(CalfError::ReadFile);
        }
    };

    if bytes_read != buff_size.len() {
        warn!("[calf] Did not read expected number of bytes. Wanted {bytes} got {bytes_read}",);
    }

    Ok(buff_size)
}
