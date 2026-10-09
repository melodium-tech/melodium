//! The `crypto` package signs data with HMAC.

use super::common;
use std::collections::HashMap;

const SIGN: &str = include_str!("scripts/crypto_sign.mel");

#[test]
fn data_is_signed_with_hmac() {
    std::env::set_var("MELODIUM_SECRET_CRYPTO_TEST_KEY", "Jefe");
    let (logs, _) = common::run(SIGN, HashMap::new());
    let message = |label: &str| common::message(&logs, label).to_string();

    // RFC 4231, test case 2.
    assert_eq!(
        message("hex"),
        "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
    );
    assert_eq!(
        message("base64"),
        "W9zBRr9gdU5qBCQmCJV1x1oAPwidJzmDnexYuWTsOEM="
    );
    assert_eq!(
        message("sha512"),
        "164b7a7bfcf819e2e395fbe73b56e0a387bd64222e831fd610270cd7ea2505549758bf75c05a994a6d034f65f8f0e6fdcaeab1a34d4a6b4b636e070a38bce737"
    );
    assert!(
        message("unknown").contains("'md5' is not a supported algorithm"),
        "{}",
        message("unknown")
    );
}
