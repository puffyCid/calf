use super::features::Features;
use crate::{calf::CalfReader, error::CalfError, utils::read::read_bytes};
use log::{error, warn};
use nom::{bytes::complete::take, number::complete::be_u32};

/// QCOW may have header extensions.
/// All are optional
#[derive(Debug)]
pub struct Extensions {
    pub features: Vec<Features>,
}

pub trait CalfExtensions<T: std::io::Seek + std::io::Read> {
    fn ext(&mut self) -> Result<Extensions, CalfError>;
}

impl<T: std::io::Seek + std::io::Read> CalfExtensions<T> for CalfReader<T> {
    /// Grab QCOW extensions
    fn ext(&mut self) -> Result<Extensions, CalfError> {
        let size = 512;
        let offset = 112;
        let bytes = read_bytes(offset, size, &mut self.fs)?;
        Extensions::grab_extensions(&bytes)
    }
}

impl Extensions {
    /// Grab option header extensions
    pub(crate) fn grab_extensions(data: &[u8]) -> Result<Extensions, CalfError> {
        let extenions = match Extensions::get_extensions(data) {
            Ok((_, result)) => result,
            Err(_err) => {
                error!("[calf] Could not parse the header extensions");
                return Err(CalfError::HeaderExtensions);
            }
        };

        Ok(extenions)
    }

    /// Parse each header extension
    fn get_extensions(data: &[u8]) -> nom::IResult<&[u8], Extensions> {
        let mut input = data;
        let mut ext = Extensions {
            features: Vec::new(),
        };
        let min_size = 9;
        while !input.len() >= min_size {
            let (remaining, sig) = be_u32(input)?;
            // Does not include the sig and size bytes
            let (remaining, size) = be_u32(remaining)?;

            let (remaining, feature_data) = take(size)(remaining)?;
            let padding_size = 8;
            let padding_value = size % padding_size;

            let padding = padding_size - padding_value;
            let (remaining, _padding_data) = take(padding)(remaining)?;
            input = remaining;

            match sig {
                0x0 => break,
                0xe2792aca => warn!("[calf] Have backing file extension"),
                0x6803f857 => {
                    ext.features = Features::grab_features(feature_data).unwrap_or_default();
                }
                0x23852875 => warn!("[calf] Have bitmaps extension"),
                0x0537be77 => warn!("[calf] Have encryption info"),
                0x44415441 => warn!("[calf] Have External data file name string"),
                _ => warn!("[calf] Unknown extension sig: {sig}"),
            }
        }

        Ok((input, ext))
    }
}

#[cfg(test)]
mod tests {
    use super::Extensions;

    #[test]
    #[should_panic(expected = "HeaderExtensions")]
    fn test_grab_extensions() {
        let test = [104, 3, 248, 87];

        let _ = Extensions::grab_extensions(&test).unwrap();
    }

    #[test]
    fn test_get_extensions() {
        let test = [
            104, 3, 248, 87, 0, 0, 1, 128, 0, 0, 100, 105, 114, 116, 121, 32, 98, 105, 116, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 1, 99, 111, 114, 114, 117, 112, 116, 32, 98, 105, 116, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 2, 101, 120, 116, 101, 114, 110, 97, 108, 32, 100, 97, 116, 97, 32, 102, 105,
            108, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 3, 99, 111, 109, 112, 114, 101, 115, 115, 105, 111, 110, 32, 116, 121, 112,
            101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 4, 101, 120, 116, 101, 110, 100, 101, 100, 32, 76, 50, 32, 101, 110, 116,
            114, 105, 101, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 108, 97, 122, 121, 32, 114, 101, 102, 99, 111, 117, 110, 116, 115,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 2, 0, 98, 105, 116, 109, 97, 112, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1,
            114, 97, 119, 32, 101, 120, 116, 101, 114, 110, 97, 108, 32, 100, 97, 116, 97, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ];

        let (_, extensions) = Extensions::get_extensions(&test).unwrap();
        assert_eq!(extensions.features.len(), 8);
    }
}
