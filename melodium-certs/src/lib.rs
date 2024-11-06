#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

pub const ROOT_CERTIFICATE: &[u8; 2094] = include_bytes!("../melodium-ca.pem");
