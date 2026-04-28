use crate::api;
use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function};

/// CPU architecture for a work container or worker request.
///
/// `ToString` and `Display` return the canonical architecture string (e.g. `"amd64"`, `"arm64"`).
#[mel_data(
    traits (PartialEquality Serialize Deserialize Display ToString)
)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Arch(pub api::Arch);

impl ToString for Arch {
    fn to_string(&self) -> String {
        format!("{}", self.0)
    }
}

impl Display for Arch {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

/// Return the `amd64` architecture.
///
/// Suitable for x86-64 processors; use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn amd64() -> Arch {
    Arch(api::Arch::Amd64)
}

/// Return the `arm64` architecture.
///
/// Suitable for ARM64/AArch64 processors; use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn arm64() -> Arch {
    Arch(api::Arch::Arm64)
}
