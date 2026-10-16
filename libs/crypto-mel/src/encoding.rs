//! Text encodings of bytes, as services give signatures and tokens.

use base64::Engine;

/// Encodings accepted by the elements of the package.
const ENCODINGS: &str = "'hex', 'base64' or 'base64url'";

/// Checks that `encoding` is supported: `hex`, `base64` or `base64url`.
pub(crate) fn check_encoding(encoding: &str) -> Result<(), String> {
    match encoding {
        "hex" | "base64" | "base64url" => Ok(()),
        _ => Err(format!(
            "'{encoding}' is not a supported encoding, expected {ENCODINGS}"
        )),
    }
}

/// Encodes `bytes` with `encoding`: `hex` in lowercase, `base64` with padding, or
/// `base64url` without padding, as JSON Web Tokens use it.
pub(crate) fn encode(bytes: &[u8], encoding: &str) -> String {
    match encoding {
        "base64" => base64::engine::general_purpose::STANDARD.encode(bytes),
        "base64url" => base64url(bytes),
        _ => hex::encode(bytes),
    }
}

/// Gives the bytes of `text` encoded as `encoding`, none if it is not valid.
///
/// `hex` is read in any case, and `base64url` with or without padding.
pub(crate) fn decode(text: &str, encoding: &str) -> Option<Vec<u8>> {
    match encoding {
        "base64" => base64::engine::general_purpose::STANDARD.decode(text).ok(),
        "base64url" => from_base64url(text),
        _ => hex::decode(text).ok(),
    }
}

/// Encodes `bytes` in base64 with the URL and file name safe alphabet, without padding.
pub(crate) fn base64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Gives the bytes of `text` in base64 with the URL and file name safe alphabet, with or
/// without padding, none if it is not valid.
pub(crate) fn from_base64url(text: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(text.trim_end_matches('='))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_are_encoded_and_decoded() {
        let bytes = [0x5b, 0xdc, 0xff, 0xfe];
        assert_eq!(encode(&bytes, "hex"), "5bdcfffe");
        assert_eq!(encode(&bytes, "base64"), "W9z//g==");
        assert_eq!(encode(&bytes, "base64url"), "W9z__g");
        assert_eq!(decode("5BDCFFFE", "hex"), Some(bytes.to_vec()));
        assert_eq!(decode("W9z//g==", "base64"), Some(bytes.to_vec()));
        assert_eq!(decode("W9z__g", "base64url"), Some(bytes.to_vec()));
        assert_eq!(decode("W9z__g==", "base64url"), Some(bytes.to_vec()));
        assert_eq!(decode("W9z//g", "base64url"), None);
        assert_eq!(decode("not hex", "hex"), None);
    }

    #[test]
    fn unsupported_encodings_are_refused() {
        assert!(check_encoding("base32").is_err());
        assert!(check_encoding("base64url").is_ok());
    }
}
