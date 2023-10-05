fn main() {
    println!("cargo:rustc-env=HOST={}", std::env::var("HOST").unwrap());
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    println!(
        "cargo:rustc-env=TARGET_FEATURE={}",
        std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default()
    );
}
