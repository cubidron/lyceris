use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;

pub fn decode_base64(encoded: &str) -> crate::Result<Vec<u8>> {
    let mut base64 = encoded.replace('-', "+").replace('_', "/");
    let padding = 4 - (base64.len() % 4);
    if padding < 4 {
        base64.push_str(&"=".repeat(padding));
    }

    let decoded = BASE64_URL_SAFE.decode(&base64)?;
    Ok(decoded)
}
