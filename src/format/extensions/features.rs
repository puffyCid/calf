use crate::{error::CalfError, utils::strings::extract_utf8_string};
use log::{error, warn};
use nom::{bytes::complete::take, number::complete::be_u8};

#[derive(Debug)]
pub struct Features {
    pub feature_type: FeatureType,
    pub bit_number: u8,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub enum FeatureType {
    Incompatible,
    Compatible,
    Autoclear,
    Unknown,
}

impl Features {
    /// Grab any Features from header extension
    pub(crate) fn grab_features(data: &[u8]) -> Result<Vec<Features>, CalfError> {
        let features = match Features::get_features(data) {
            Ok((_, result)) => result,
            Err(_err) => {
                error!("[calf] Could not pare features extension");
                return Err(CalfError::HeaderExtensionFeatures);
            }
        };

        Ok(features)
    }

    /// Parse Features header extension
    fn get_features(data: &[u8]) -> nom::IResult<&[u8], Vec<Features>> {
        let mut input = data;
        let min_size = 48;

        let mut features = Vec::new();
        while input.len() >= min_size {
            let (remaining, feature_data) = be_u8(input)?;
            let (remaining, bit_number) = be_u8(remaining)?;

            let string_size: u8 = 46;
            let (remaining, string_data) = take(string_size)(remaining)?;
            input = remaining;

            let value = extract_utf8_string(string_data);

            let feature = Features {
                feature_type: Features::get_type(&feature_data),
                bit_number,
                value,
            };

            features.push(feature);
        }

        Ok((input, features))
    }

    /// Determine `FeatureType`
    fn get_type(data: &u8) -> FeatureType {
        match data {
            0 => FeatureType::Incompatible,
            1 => FeatureType::Compatible,
            2 => FeatureType::Autoclear,
            _ => {
                warn!("[calf] Unknown feature type {data}");
                FeatureType::Unknown
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Features;
    use crate::format::extensions::features::FeatureType;

    #[test]
    fn test_grab_features() {
        let test = [
            0, 0, 100, 105, 114, 116, 121, 32, 98, 105, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 99, 111,
            114, 114, 117, 112, 116, 32, 98, 105, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 101, 120, 116, 101,
            114, 110, 97, 108, 32, 100, 97, 116, 97, 32, 102, 105, 108, 101, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 99, 111, 109, 112,
            114, 101, 115, 115, 105, 111, 110, 32, 116, 121, 112, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 101, 120, 116,
            101, 110, 100, 101, 100, 32, 76, 50, 32, 101, 110, 116, 114, 105, 101, 115, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 108, 97,
            122, 121, 32, 114, 101, 102, 99, 111, 117, 110, 116, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 98, 105, 116,
            109, 97, 112, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 114, 97, 119, 32, 101, 120, 116,
            101, 114, 110, 97, 108, 32, 100, 97, 116, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let results = Features::grab_features(&test).unwrap();
        assert_eq!(results.len(), 8);

        assert_eq!(results[0].feature_type, FeatureType::Incompatible);
        assert_eq!(results[0].value, "dirty bit");

        assert_eq!(results[7].feature_type, FeatureType::Autoclear);
        assert_eq!(results[7].value, "raw external data");
    }

    #[test]
    fn test_get_features() {
        let test = [
            0, 0, 100, 105, 114, 116, 121, 32, 98, 105, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 99, 111,
            114, 114, 117, 112, 116, 32, 98, 105, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 101, 120, 116, 101,
            114, 110, 97, 108, 32, 100, 97, 116, 97, 32, 102, 105, 108, 101, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 99, 111, 109, 112,
            114, 101, 115, 115, 105, 111, 110, 32, 116, 121, 112, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 101, 120, 116,
            101, 110, 100, 101, 100, 32, 76, 50, 32, 101, 110, 116, 114, 105, 101, 115, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 108, 97,
            122, 121, 32, 114, 101, 102, 99, 111, 117, 110, 116, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 98, 105, 116,
            109, 97, 112, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 114, 97, 119, 32, 101, 120, 116,
            101, 114, 110, 97, 108, 32, 100, 97, 116, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let (_, results) = Features::get_features(&test).unwrap();

        assert_eq!(results.len(), 8);
        assert_eq!(results[2].feature_type, FeatureType::Incompatible);
        assert_eq!(results[2].value, "external data file");

        assert_eq!(results[5].feature_type, FeatureType::Compatible);
        assert_eq!(results[5].value, "lazy refcounts");
    }

    #[test]
    fn test_get_type() {
        let fake = 4;
        let value = Features::get_type(&fake);
        assert_eq!(value, FeatureType::Unknown);
    }
}
