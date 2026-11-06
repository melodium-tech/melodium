//! Digests of data, such as to check what was transferred or to name what it holds.
//!
//! Digests are given in lowercase hexadecimal.

use melodium_core::*;
use melodium_macro::{mel_function, mel_treatment};
use sha2::{Digest, Sha256};

/// Computes the SHA-256 digest of data.
///
/// The digest of the bytes received through `data` is given through `digest` once `data`
/// ends, in lowercase hexadecimal, such as to check a file that was transferred, or to name a
/// cache after the dependencies it holds.
///
/// ```mermaid
/// graph LR
///     T("sha256()")
///     B["… 🟦 🟦 🟦"] -->|data| T
///
///     T -->|digest| D["〈🟨〉"]
///
///     style B fill:#ffffff,stroke:#ffffff
///     style D fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input data Stream<byte>
    output digest Block<string>
)]
pub async fn sha256() {
    let mut hasher = Sha256::new();
    while let Ok(bytes) = data.recv_many_as::<u8>().await {
        hasher.update(&bytes);
    }
    let _ = digest.send_one_as(hex::encode(hasher.finalize())).await;
}

/// Gives the SHA-256 digest of `data`, in lowercase hexadecimal.
#[mel_function]
pub fn sha256(data: Vec<byte>) -> string {
    sha256_hex([data.as_slice()])
}

/// SHA-256 digest of `chunks` following each other, in lowercase hexadecimal.
fn sha256_hex<'a>(chunks: impl IntoIterator<Item = &'a [u8]>) -> String {
    let mut hasher = Sha256::new();
    for chunk in chunks {
        hasher.update(chunk);
    }
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FIPS 180-2, "abc" and the two-block message.
    const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    const TWO_BLOCKS: &[u8] = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
    const TWO_BLOCKS_DIGEST: &str =
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1";

    #[test]
    fn digests_match_fips_180_2() {
        assert_eq!(sha256_hex([&b"abc"[..]]), ABC);
        assert_eq!(sha256_hex([TWO_BLOCKS]), TWO_BLOCKS_DIGEST);
        assert_eq!(
            sha256_hex([]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn digests_do_not_depend_on_chunks() {
        for size in [1, 3, 7, 55, 56, 64, 100] {
            let chunks = TWO_BLOCKS.chunks(size).collect::<Vec<_>>();
            assert_eq!(sha256_hex(chunks), TWO_BLOCKS_DIGEST, "chunks of {size}");
        }
        assert_eq!(sha256_hex([&b""[..], b"a", b"", b"bc", b""]), ABC);
    }
}
