#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

/// Root certificates trusted for the distribution protocol, in PEM format.
///
/// Every certificate in this file is trusted, so that a new root and the previous one
/// can both be trusted while nodes move from one to the other (see the README).
pub const ROOT_CERTIFICATE: &[u8] = include_bytes!("../melodium-ca.pem");
