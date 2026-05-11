use crate::utils::encoding::base64_encode_standard;
use std::string::{FromUtf8Error, FromUtf16Error};
use tracing::{Level, event};
use uuid::Uuid;

/// Get a UTF8 string from provided bytes data. Invalid UTF8 is base64 encoded. Use `extract_uf8_string_lossy` if replacing bytes is acceptable
pub(crate) fn extract_utf8_string(data: &[u8]) -> String {
    let utf8_result = bytes_to_utf8_string(data);
    match utf8_result {
        Ok(result) => result,
        Err(err) => {
            event!(
                Level::WARN,
                "[strings-calf] Failed to get UTF8 string: {err:?}"
            );
            let max_size = 2097152;
            let issue = if data.len() < max_size {
                base64_encode_standard(data)
            } else {
                format!(
                    "[strings-calf] Binary data size larger than 2MB, size: {}",
                    data.len()
                )
            };
            format!("[strings-calf] Failed to get UTF8 string: {}", issue)
        }
    }
}

/// Get a UTF8 string from provided bytes data
fn bytes_to_utf8_string(data: &[u8]) -> Result<String, FromUtf8Error> {
    let result = String::from_utf8(data.to_vec())?;
    let value = result.trim_end_matches('\0').to_string();
    Ok(value)
}

/// Get a UTF16 string from provided bytes data. Will attempt to fix malformed UTF16. Such as UTF16 missing zeros
pub(crate) fn extract_utf16_string(data: &[u8]) -> String {
    let result = bytes_to_utf16_string(data, false);
    match result {
        Ok(result) => result.trim_start_matches('\u{1}').to_string(),
        Err(_err) => {
            // If we fail, try again with adjustment. Just incase it works.
            let result = bytes_to_utf16_string(data, true);
            match result {
                Ok(result) => result.trim_start_matches('\u{1}').to_string(),
                Err(err) => {
                    event!(
                        Level::WARN,
                        "[strings-calf] Failed to get UTF16 string: {err:?}"
                    );
                    base64_encode_standard(data)
                }
            }
        }
    }
}

/// Get a UTF16 string from provided bytes data
fn bytes_to_utf16_string(data: &[u8], adjust: bool) -> Result<String, FromUtf16Error> {
    let mut utf16_data: Vec<u16> = Vec::new();
    // Convert data to UTF16 (&[u16])
    let min_byte_size = 2;
    for wide_char in data.chunks(min_byte_size) {
        if wide_char == vec![0, 0] || wide_char.len() < min_byte_size {
            // Check for last character
            if !wide_char.is_empty() && !wide_char.contains(&0) {
                utf16_data.push(wide_char[0] as u16);
            }
            break;
        }

        // Sometimes we have to encode to UTF16 for some strings
        if !wide_char.contains(&0) && adjust {
            utf16_data.push(wide_char[0] as u16);
            utf16_data.push(wide_char[1] as u16);
            continue;
        }
        if wide_char[0] == 0 {
            utf16_data.push(u16::from_ne_bytes([wide_char[1], wide_char[0]]));
            continue;
        }

        utf16_data.push(u16::from_ne_bytes([wide_char[0], wide_char[1]]));
    }

    // Windows uses UTF16
    let utf16_result = String::from_utf16(&utf16_data)?;

    Ok(utf16_result)
}

#[derive(PartialEq)]
pub(crate) enum Endian {
    _Big,
    Little,
}

/// Extract GUID from bytes
pub(crate) fn extract_guid(data: &[u8], endian: Endian) -> String {
    let guid_size = 16;
    if data.len() != guid_size {
        event!(
            Level::WARN,
            "[strings-calf] Provided data does not meet GUID size of 16 bytes, got: {}",
            data.len()
        );
        return format!("Not a GUID/UUID: {data:?}");
    }

    let guid_data = data.try_into().unwrap_or_default();
    if endian == Endian::Little {
        return Uuid::from_bytes_le(guid_data).hyphenated().to_string();
    }
    Uuid::from_bytes(guid_data).hyphenated().to_string()
}
