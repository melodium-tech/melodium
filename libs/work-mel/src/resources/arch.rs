use crate::api;
use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function};

/// CPU architecture for a worker request.
///
/// `ToString` and `Display` return the canonical architecture string (e.g. `"amd64"`, `"arm64"`).
///
/// Each architecture is built with a function, and most have two: one named as in the
/// [OCI image specification](https://github.com/opencontainers/image-spec/blob/main/config.md#properties)
/// (the names of Go's [`GOARCH`](https://go.dev/doc/install/source#environment) list, like `|amd64()`),
/// and one named as Rust names its [target architectures](https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch)
/// (like `|x86_64()`). Both give the same architecture, so use the names you are used to.
///
/// Wherever an architecture is given as text, in a request for example, both names are understood too.
/// Architectures are always displayed and written with the first name of the table.
///
/// | Function | Architecture | Information |
/// |---|---|---|
/// | `\|amd64()`, `\|x86_64()` | `amd64`, `x86_64` | 64-bit x86 (Intel, AMD). Most servers, cloud instances, PCs and Intel Macs. |
/// | `\|arm64()`, `\|aarch64()` | `arm64`, `aarch64` | 64-bit ARM. Apple Silicon Macs, AWS Graviton and other ARM cloud instances, Raspberry Pi with a 64-bit system. |
/// | `\|i386()`, `\|x86()`, `\|i686()` | `386`, `x86`, `i686` | 32-bit x86. Old PCs and legacy systems. |
/// | `\|arm()`, `\|armv7()` | `arm`, `armv7` | 32-bit ARM, hard float. Raspberry Pi with a 32-bit system, embedded boards. |
/// | `\|riscv64()`, `\|riscv64gc()` | `riscv64`, `riscv64gc` | 64-bit RISC-V. Development boards and emerging servers. |
/// | `\|ppc64le()`, `\|powerpc64le()` | `ppc64le`, `powerpc64le` | 64-bit PowerPC, little endian. IBM Power servers running Linux. |
/// | `\|ppc64()`, `\|powerpc64()` | `ppc64`, `powerpc64` | 64-bit PowerPC, big endian. Older IBM Power and legacy PowerPC systems. |
/// | `\|s390x()` | `s390x` | IBM Z mainframes, including LinuxONE. |
/// | `\|loong64()`, `\|loongarch64()` | `loong64`, `loongarch64` | 64-bit LoongArch. Recent Loongson processors. |
/// | `\|mips()` | `mips` | 32-bit MIPS, big endian. Legacy networking equipment. |
/// | `\|mipsle()`, `\|mipsel()` | `mipsle`, `mipsel` | 32-bit MIPS, little endian. Routers and embedded devices. |
/// | `\|mips64()` | `mips64` | 64-bit MIPS, big endian. Legacy networking equipment and servers. |
/// | `\|mips64le()`, `\|mips64el()` | `mips64le`, `mips64el` | 64-bit MIPS, little endian. Older Loongson processors and some network devices. |
/// | `\|wasm32()` | `wasm32`, `wasm` | 32-bit WebAssembly. Browsers, WASI runtimes and edge platforms. |
/// | `\|wasm64()` | `wasm64` | 64-bit WebAssembly. Experimental, few runtimes support it. |
///
/// `|local_arch` gives the architecture of the machine running the engine.
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

/// Return the `amd64` architecture, under the name Rust uses.
///
/// Same as `|amd64`.
#[mel_function]
pub fn x86_64() -> Arch {
    Arch(api::Arch::Amd64)
}

/// Return the `arm64` architecture, under the name Rust uses.
///
/// Same as `|arm64`.
#[mel_function]
pub fn aarch64() -> Arch {
    Arch(api::Arch::Arm64)
}

/// Return the 32-bit x86 (`386`) architecture, under the name Rust uses.
///
/// Same as `|i386`.
#[mel_function]
pub fn x86() -> Arch {
    Arch(api::Arch::I386)
}

/// Return the 32-bit x86 (`386`) architecture, under the name Rust uses.
///
/// Same as `|i386`.
#[mel_function]
pub fn i686() -> Arch {
    Arch(api::Arch::I386)
}

/// Return the 32-bit ARM, hard float (`arm`) architecture, under the name Rust uses.
///
/// Same as `|arm`.
#[mel_function]
pub fn armv7() -> Arch {
    Arch(api::Arch::Arm)
}

/// Return the 64-bit RISC-V (`riscv64`) architecture, under the name Rust uses.
///
/// Same as `|riscv64`.
#[mel_function]
pub fn riscv64gc() -> Arch {
    Arch(api::Arch::Riscv64)
}

/// Return the 64-bit little-endian PowerPC (`ppc64le`) architecture, under the name Rust uses.
///
/// Same as `|ppc64le`.
#[mel_function]
pub fn powerpc64le() -> Arch {
    Arch(api::Arch::Ppc64le)
}

/// Return the 64-bit big-endian PowerPC (`ppc64`) architecture, under the name Rust uses.
///
/// Same as `|ppc64`.
#[mel_function]
pub fn powerpc64() -> Arch {
    Arch(api::Arch::Ppc64)
}

/// Return the 64-bit LoongArch (`loong64`) architecture, under the name Rust uses.
///
/// Same as `|loong64`.
#[mel_function]
pub fn loongarch64() -> Arch {
    Arch(api::Arch::Loong64)
}

/// Return the 32-bit little-endian MIPS (`mipsle`) architecture, under the name Rust uses.
///
/// Same as `|mipsle`.
#[mel_function]
pub fn mipsel() -> Arch {
    Arch(api::Arch::Mipsle)
}

/// Return the 64-bit little-endian MIPS (`mips64le`) architecture, under the name Rust uses.
///
/// Same as `|mips64le`.
#[mel_function]
pub fn mips64el() -> Arch {
    Arch(api::Arch::Mips64le)
}

/// Return the architecture of the machine running this engine.
///
/// The value is concrete (e.g. `amd64` on an x86-64 machine, `arm64` on an Apple Silicon one).
/// Give it as the `arch` parameter to request a worker of the same architecture as the machine running the program.
#[mel_function]
pub fn local_arch() -> Arch {
    Arch(api::Arch::host())
}
