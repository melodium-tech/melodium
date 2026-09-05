# Supported Platforms

Mélodium supports multiple platforms.
The term “Platform” refers to a set of operating system, machine architecture, and compilation method; also known as “target” by Rust developers.

Directly supported platforms are platforms for which Mélodium binaries are released.

| Platform | Notes |
|----------|-------|
| `aarch64-apple-darwin`       | ARM64 macOS (11.0+, Big Sur+) |
| `aarch64-pc-windows-msvc`    | ARM64 Windows MSVC |
| `aarch64-unknown-linux-gnu`  | ARM64 Linux (kernel 4.1, glibc 2.17+) |
| `aarch64-unknown-linux-musl` | ARM64 Linux with MUSL |
| `i686-pc-windows-gnu`        | 32-bit MinGW (Windows 7+) |
| `i686-pc-windows-msvc`       | 32-bit MSVC (Windows 7+) |
| `i686-unknown-linux-gnu`     | 32-bit Linux (kernel 3.2+, glibc 2.17+) |
| `i686-unknown-linux-musl`    | 32-bit Linux with MUSL |
| `x86_64-apple-darwin`        | 64-bit macOS (10.12+, Sierra+) |
| `x86_64-pc-windows-gnu`      | 64-bit MinGW (Windows 7+) |
| `x86_64-pc-windows-msvc`     | 64-bit MSVC (Windows 7+) |
| `x86_64-unknown-linux-gnu`   | 64-bit Linux (kernel 3.2+, glibc 2.17+) |
| `x86_64-unknown-linux-musl`  | 64-bit Linux with MUSL |

## Other platforms support

Mélodium may work on platforms that are not listed as _Directly supported platforms_.
For those platforms, it is needed to build and install Mélodium through the Rust `cargo` command.

```sh
cargo install melodium
```

These platforms are notably Linux/BSD-like ones, as well as less common machine architecture for operating systems already supported.  
For a full list of possible target, please refer:
1. to the Mélodium project repository [CI checks file](https://gitlab.com/melodium/melodium/-/blob/master/.gitlab/ci/checks.yml);
1. to the Rust [platform support](https://doc.rust-lang.org/nightly/rustc/platform-support.html) list, where any `std`-compatible target should hypothetically work.

If you have specific needs for a given platform (either getting prebuild binaries or making the compilation possible), please open a ticket on the [project repository](https://gitlab.com/melodium/melodium).


## Specificities

Some platforms may have specificities.
The aim is not to be exhaustive but to explain the reasons of these differences.

### Linux GNU vs. MUSL

Without deeping dive into the details, `*-gnu` for Linux platforms means Mélodium rely on glibc implementation embedded by the host distribution, and `*-musl` means Mélodium executable is _statically linked_ with the musl libc and so is fully autonomous in and by itself. 

While both are good choice, `*-gnu` may not fit in some situations, such as distributions that don't ship with glibc (most notably Alpine Linux), when `*-musl` should work anywhere, at the cost of some extra kilobytes embedded within the executable itself.

From user perspective, no difference should be noticed in any case.

### Windows GNU vs. MSVC

On Windows, `*-gnu` means Mélodium is built using the GNU MinGW toolchain, while `*-msvc` means it is built using the Microsoft Visual Studio toolchain.

Both versions are totally equivalent in terms of usage and compatibility on Windows platforms from a user and developer point of view.
Difference is mainly important for low-level software developers who might want to use one over the other in some very specific situations.

Again, from user perspective, no difference should be noticed in any case. If nothing explicitly restrain the choice between both, any of them can be picked indifferently.
