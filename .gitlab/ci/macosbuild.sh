#!/usr/bin/env bash
set -e

git clone https://github.com/tpoechtrager/osxcross
cd osxcross
pwd
wget -nc https://s3.dockerproject.org/darwin/v2/MacOSX10.11.sdk.tar.xz
mv MacOSX10.11.sdk.tar.xz tarballs/
UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh
cd ..

mkdir ~/.cargo
echo '[target.x86_64-apple-darwin]' >> ~/.cargo/config.toml
echo 'linker = "x86_64-apple-darwin15-clang"' >> ~/.cargo/config.toml
echo 'ar = "x86_64-apple-darwin15-ar"' >> ~/.cargo/config.toml

rustc --version && cargo --version 
export PATH=/builds/melodium/melodium/osxcross/target/bin:$PATH
export CC=o64-clang
export CXX=o64-clang++
cargo build --locked --release --target x86_64-apple-darwin
