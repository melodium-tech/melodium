mod error;
mod listen;
mod messages;
mod protocol;

pub use error::{DistributionError, DistributionResult};
pub use listen::{launch_listen, launch_listen_localcert, launch_listen_unsecure};
pub use messages::*;
pub use protocol::{Error, Protocol};

use melodium_common::descriptor::Version;

pub static VERSION: Version = Version::new(0, 1, 0);
pub const ROOT_CERTIFICATE: &[u8; 2094] = include_bytes!("../melodium-ca.pem");
