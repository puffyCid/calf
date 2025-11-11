/// Heavily inspired by <https://github.com/panda-re/qcow-rs/blob/master/src/reader.rs> (MIT)
use crate::{
    bootsector::boot::{BootInfo, boot_info},
    calf::QcowInfo,
    error::CalfError,
    format::{
        cluster::read_cluster,
        level::{Level, read_level},
    },
};
use log::{debug, error};
use std::io::{self, BufReader, Read, Seek, SeekFrom};

pub struct OsReader<'qcow, 'reader, T>
where
    T: std::io::Seek + std::io::Read,
{
    qcow: &'qcow QcowInfo,
    pub(crate) reader: &'reader mut BufReader<T>,
    position: u64,
    cluster_bits: u32,
    cluster_size: u64,
    os_size: u64,
    level1_key: u64,
    level1_cache: &'qcow Level,
    level2_table_cache: Vec<Level>,
    level2_cache: Level,
    cluster_bytes: Vec<u8>,
    level2_key: u64,
}

impl QcowInfo {
    /// Create a reader that can read bytes from OS guest inside the QCOW file
    pub fn new<'qcow, 'reader, T: io::Seek + io::Read>(
        &'qcow self,
        reader: &'reader mut BufReader<T>,
    ) -> Result<OsReader<'qcow, 'reader, T>, CalfError> {
        let position = 0;
        let level1_key = 0;
        let level2_key = 0;

        let qcow = self;
        if let Some(level1_cache) = qcow.level1_table.get(level1_key as usize) {
            let cluster_size = 1 << qcow.header.cluster_block_bits_count;
            let cluster_bits = qcow.header.cluster_block_bits_count;
            let cluster_bytes: Vec<u8> = vec![0; cluster_size as usize];
            let level2_table_cache = read_level(reader, &cluster_bits, &level1_cache.offset)?;
            if let Some(level2_cache) = level2_table_cache.get(level2_key as usize) {
                return Ok(OsReader {
                    qcow,
                    reader,
                    position,
                    cluster_bits,
                    cluster_size,
                    os_size: qcow.header.size,
                    level1_key,
                    level1_cache,
                    level2_cache: level2_cache.clone(),
                    level2_table_cache,
                    cluster_bytes,
                    level2_key,
                });
            }
        }

        error!("[calf] Could not get level one table for key {level1_key}");
        Err(CalfError::Level)
    }
}

impl<'a, 'qcow, T: std::io::Seek + std::io::Read> OsReader<'a, 'qcow, T> {
    /// Determine OS boot information
    pub fn get_boot_info(&mut self) -> Result<BootInfo, CalfError> {
        boot_info(self)
    }

    fn refresh_level1_cache(&mut self) -> io::Result<()> {
        let size = 8;
        let level2_entries = self.cluster_size / size;

        let level1_key = (self.position / self.cluster_size) / level2_entries;
        if self.level1_key != level1_key {
            self.level1_key = level1_key;
            self.level1_cache =
                self.qcow
                    .level1_table
                    .get(level1_key as usize)
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "Read position past end of qcow file",
                        )
                    })?;

            self.level2_table_cache =
                read_level(self.reader, &self.cluster_bits, &self.level1_cache.offset)
                    .unwrap_or_default();
        }

        Ok(())
    }

    fn refresh_level2_cache(&mut self) -> io::Result<()> {
        let size = 8;
        let level2_entries = self.cluster_size / size;
        let level2_key = self.position / self.cluster_size;
        let level2_index = level2_key % level2_entries;

        if self.level2_key != level2_key {
            self.level2_key = level2_key;
            self.refresh_level1_cache()?;

            if self.level1_cache.offset != 0
                && let Some(value) = self.level2_table_cache.get(level2_index as usize)
            {
                self.level2_cache = value.clone();
                self.level2_key = level2_key;
            }
        }

        if self.level1_cache.offset == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "offset to level 2 is 0",
            ));
        }

        debug!(
            "[calf] level 2 cache: {:?}. Level 1 cache: {:?}",
            self.level2_cache, self.level1_cache
        );
        self.cluster_bytes = read_cluster(
            self.reader,
            self.level2_cache.offset,
            self.cluster_size,
            &self.qcow.header.compression_method,
            &self.level2_cache.is_compressed,
        )?;

        Ok(())
    }
}

impl<'a, 'qcow, T> Read for OsReader<'a, 'qcow, T>
where
    T: std::io::Seek + std::io::Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.refresh_level2_cache() {
            Ok(()) => {
                let position_in_cluster = self.position % self.cluster_size;
                let cluster_bytes_remaining = self.cluster_size - position_in_cluster;

                let read_len = u64::min(cluster_bytes_remaining, buf.len() as u64);
                let read_end = position_in_cluster + read_len;
                let pos_in_cluster = position_in_cluster as usize;
                // We set read_len to lowest value comparing cluster_bytes_remaining and buf.len()
                buf[..read_len as usize]
                    .copy_from_slice(&self.cluster_bytes[pos_in_cluster..read_end as usize]);

                self.position += read_len;
                let _ = self.refresh_level2_cache();

                Ok(read_len as usize)
            }
            Err(err) => (move || {
                self.reader.seek(SeekFrom::Start(self.position)).ok()?;
                let bytes_read = self.reader.read(buf).ok()?;

                self.position += bytes_read as u64;

                Some(bytes_read)
            })()
            .ok_or(err),
        }
    }
}

impl<'a, 'qcow, T> Seek for OsReader<'a, 'qcow, T>
where
    T: std::io::Seek + std::io::Read,
{
    fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> {
        match position {
            std::io::SeekFrom::Start(start_position) => self.position = start_position,
            std::io::SeekFrom::End(end_position) => {
                self.position =
                    (end_position + self.os_size as i64)
                        .try_into()
                        .map_err(|_err| {
                            io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "seek is out of range of 64-bit position",
                            )
                        })?;
            }
            std::io::SeekFrom::Current(relative_position) => {
                self.position = self
                    .position
                    .try_into()
                    .map_or_else(
                        |_| self.position as i64 + relative_position,
                        |pos: i64| pos + relative_position,
                    )
                    .try_into()
                    .map_err(|_err| {
                        io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "seek is out of range of 64-bit position",
                        )
                    })?;
            }
        }

        Ok(self.position)
    }
}
