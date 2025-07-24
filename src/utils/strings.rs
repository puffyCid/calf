use crate::utils::encoding::base64_encode_standard;
use log::warn;
use std::string::FromUtf8Error;

/// Get a UTF8 string from provided bytes data. Invalid UTF8 is base64 encoded. Use `extract_uf8_string_lossy` if replacing bytes is acceptable
pub(crate) fn extract_utf8_string(data: &[u8]) -> String {
    let utf8_result = bytes_to_utf8_string(data);
    match utf8_result {
        Ok(result) => result,
        Err(err) => {
            warn!("[strings] Failed to get UTF8 string: {err:?}");
            let max_size = 2097152;
            let issue = if data.len() < max_size {
                base64_encode_standard(data)
            } else {
                format!(
                    "[strings] Binary data size larger than 2MB, size: {}",
                    data.len()
                )
            };
            format!("[strings] Failed to get UTF8 string: {}", issue)
        }
    }
}

/// Get a UTF8 string from provided bytes data
fn bytes_to_utf8_string(data: &[u8]) -> Result<String, FromUtf8Error> {
    let result = String::from_utf8(data.to_vec())?;
    let value = result.trim_end_matches('\0').to_string();
    Ok(value)
}
