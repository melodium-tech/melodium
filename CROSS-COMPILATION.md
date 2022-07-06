
# Cross compilation

Mélodium can be compiled for a large range of platforms.
Here is explained how to cross compile Mélodium from a x86_64-unknown-linux-gnu arch (Ubuntu-like) distribution to:
- i686-unknown-linux-gnu
- x86_64-unknown-linux-gnu
- aarch64-unknown-linux-gnu
- i686-pc-windows-gnu
- x86_64-pc-windows-gnu
- x86_64-apple-darwin

## i686-unknown-linux-gnu

For i686-unknown-linux-gnu we need other GCC libs, pkg-config, and Alsa i386 dev to be installed, `apt install -y gcc-multilib pkg-config-i686-linux-gnu libasound2-dev:i386`.

```sh
rustup target add i686-unknown-linux-gnu
PKG_CONFIG=/usr/bin/i686-linux-gnu-pkg-config cargo build --release --target i686-unknown-linux-gnu
```

## x86_64-unknown-linux-gnu

As host is considered x86_64-unknown-linux-gnu, not much is to do here.
Install the prerequisites `apt install -y libasound2-dev`.

```sh
cargo build --release --target x86_64-unknown-linux-gnu
```

## aarch64-unknown-linux-gnu

> Not working at this point, will be studied later as ARM64 is not a main goal at this point of the project.
> Direct compilation _on_ ARM64 machine works.

For aarch64-unknown-linux-gnu we need other GCC libs and pkg-config for aarch64 to be installed, `apt install -y gcc-multilib pkg-config-aarch64-linux-gnu`.

```sh
rustup target add aarch64-unknown-linux-gnu
PKG_CONFIG=/usr/bin/aarch64-linux-gnu-pkg-config cargo build --release --target aarch64-unknown-linux-gnu
```

## i686-pc-windows-gnu

> Not working at this point, will be studied later as Windows 32bits is not a main goal at this point of the project.

A.k.a. Windows 32 bits, compiling for i686-pc-windows-gnu requires MinGW to be installed, `apt install -y mingw-w64`.

```sh
rustup target add i686-pc-windows-gnu
cargo build --release --target i686-pc-windows-gnu
```

## x86_64-pc-windows-gnu

A.k.a. Windows 64 bits, compiling for x86_64-pc-windows-gnu requires MinGW to be installed, `apt install -y mingw-w64`.

```sh
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## x86_64-apple-darwin

A.k.a. MacOS 64 bits, compiling for x86_64-apple-darwin requires a full setup explained at https://github.com/tpoechtrager/osxcross.

