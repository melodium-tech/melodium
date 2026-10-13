use crate::api;
use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function};

/// CPU architecture for a work container or worker request.
///
/// `ToString` and `Display` return the canonical architecture string (e.g. `"amd64"`, `"arm64"`).
///
/// Architectures are named as in the OCI image specification (the `GOARCH` values), with `wasm32` and `wasm64` for WebAssembly,
/// which differs from the names Rust uses for its targets:
///
/// | Function | Architecture | Rust target triples (examples) |
/// |---|---|---|
/// | `\|amd64` | `amd64`, x86-64 | `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-apple-darwin`, `x86_64-pc-windows-gnu`, `x86_64-pc-windows-msvc` |
/// | `\|arm64` | `arm64`, 64-bit ARM | `aarch64-unknown-linux-gnu`, `aarch64-unknown-linux-musl`, `aarch64-apple-darwin`, `aarch64-pc-windows-msvc` |
/// | `\|i386` | `386`, x86 32-bit | `i686-unknown-linux-gnu`, `i686-pc-windows-gnu`, `i686-pc-windows-msvc` |
/// | `\|arm` | `arm`, ARM 32-bit hard float | `armv7-unknown-linux-gnueabihf`, `arm-unknown-linux-gnueabihf` |
/// | `\|riscv64` | `riscv64` | `riscv64gc-unknown-linux-gnu` |
/// | `\|ppc64le` | `ppc64le`, PowerPC 64-bit little endian | `powerpc64le-unknown-linux-gnu` |
/// | `\|ppc64` | `ppc64`, PowerPC 64-bit big endian | `powerpc64-unknown-linux-gnu` |
/// | `\|s390x` | `s390x`, IBM Z | `s390x-unknown-linux-gnu` |
/// | `\|loong64` | `loong64`, LoongArch 64-bit | `loongarch64-unknown-linux-gnu` |
/// | `\|mips`, `\|mipsle` | `mips`, `mipsle`, 32-bit MIPS big and little endian | `mips-unknown-linux-gnu`, `mipsel-unknown-linux-gnu` |
/// | `\|mips64`, `\|mips64le` | `mips64`, `mips64le`, 64-bit MIPS big and little endian | `mips64-unknown-linux-gnuabi64`, `mips64el-unknown-linux-gnuabi64` |
/// | `\|wasm32` | `wasm32`, 32-bit WebAssembly (OCI `wasm`) | `wasm32-unknown-unknown`, `wasm32-wasip1`, `wasm32-wasip2` |
/// | `\|wasm64` | `wasm64`, 64-bit WebAssembly | `wasm64-unknown-unknown` |
///
/// An architecture no engine is built for, or that no provider offers, can still be declared: the request is then refused with a message naming the need.
/// The local `compose` location only runs the architecture of the host.
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

/// Return the 32-bit x86 (`386`) architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn i386() -> Arch {
    Arch(api::Arch::I386)
}

/// Return the 32-bit ARM, hard float (`arm`) architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn arm() -> Arch {
    Arch(api::Arch::Arm)
}

/// Return the 64-bit RISC-V architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn riscv64() -> Arch {
    Arch(api::Arch::Riscv64)
}

/// Return the 64-bit little-endian PowerPC architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn ppc64le() -> Arch {
    Arch(api::Arch::Ppc64le)
}

/// Return the 64-bit big-endian PowerPC architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn ppc64() -> Arch {
    Arch(api::Arch::Ppc64)
}

/// Return the IBM Z (`s390x`) architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn s390x() -> Arch {
    Arch(api::Arch::S390x)
}

/// Return the 64-bit LoongArch architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn loong64() -> Arch {
    Arch(api::Arch::Loong64)
}

/// Return the 32-bit big-endian MIPS architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn mips() -> Arch {
    Arch(api::Arch::Mips)
}

/// Return the 32-bit little-endian MIPS architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn mipsle() -> Arch {
    Arch(api::Arch::Mipsle)
}

/// Return the 64-bit big-endian MIPS architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn mips64() -> Arch {
    Arch(api::Arch::Mips64)
}

/// Return the 64-bit little-endian MIPS architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn mips64le() -> Arch {
    Arch(api::Arch::Mips64le)
}

/// Return the 32-bit WebAssembly architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn wasm32() -> Arch {
    Arch(api::Arch::Wasm32)
}

/// Return the 64-bit WebAssembly architecture.
///
/// Use as the `arch` parameter when requesting workers or containers.
#[mel_function]
pub fn wasm64() -> Arch {
    Arch(api::Arch::Wasm64)
}

/// Return the architecture of the machine running this engine.
///
/// The value is concrete (e.g. `amd64` on an x86-64 machine, `arm64` on an Apple Silicon one), and is the one of the engine running the program,
/// not necessarily the one of the worker it requests. To get a container running on the same architecture as its worker,
/// whatever it is, give no architecture (`_`) to the container instead.
#[mel_function]
pub fn local_arch() -> Arch {
    Arch(api::Arch::host())
}
