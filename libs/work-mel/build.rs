fn main() {
    println!(
        "cargo:rustc-env=ARCH={}",
        std::env::var("CARGO_CFG_TARGET_ARCH").unwrap()
    );
}
