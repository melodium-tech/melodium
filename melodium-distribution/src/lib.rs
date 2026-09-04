#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod error;
mod framing;
mod listen;
mod messages;
mod protocol;

pub use error::{DistributionError, DistributionResult};
pub use framing::{max_batch_chunk_bytes, max_frame_bytes};
pub use listen::{
    launch_listen, launch_listen_localcert, launch_listen_unsecure, max_concurrent_messages,
};
pub use messages::*;
pub use protocol::{Error, Protocol};

use melodium_common::descriptor::Version;

// `InputData`/`OutputData` now carry a `melodium_share::TransmissionValue` batch instead
// of a flat `Vec<RawValue>` (ticket #116 phase F) — a wire-shape change, hence the bump,
// same precedent as #114's framing change.
pub static VERSION: Version = Version::new(0, 3, 0);
