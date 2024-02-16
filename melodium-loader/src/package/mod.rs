pub mod core;
#[cfg(feature = "filesystem")]
pub mod filesystem;
#[cfg(feature = "jeu")]
pub mod jeu;
pub mod package;
pub mod raw;

pub use self::core::CorePackage;
#[cfg(feature = "filesystem")]
pub use filesystem::FsPackage;
#[cfg(feature = "jeu")]
pub use jeu::JeuPackage;
pub use package::{Package, PackageInfo, PackageTrait};
pub use raw::RawPackage;

/// Builds a `.jeu` file from a given package on filesystem.
#[cfg_attr(docsrs, doc(cfg(all(feature = "filesystem", feature = "jeu"))))]
#[cfg(all(feature = "filesystem", feature = "jeu"))]
pub fn build_jeu(
    input: &std::path::Path,
    output: &std::path::Path,
) -> melodium_common::descriptor::LoadingResult<()> {
    use melodium_common::descriptor::{LoadingError, LoadingResult};

    FsPackage::new(input).and_then(|pkg| match std::fs::File::create(output) {
        Ok(jeu_file) => match { JeuPackage::build(&pkg, std::io::BufWriter::new(jeu_file)) } {
            Ok(_) => LoadingResult::new_success(()),
            Err(err) => LoadingResult::new_failure(LoadingError::unreachable_file(
                239,
                output.to_path_buf(),
                err.to_string(),
            )),
        },
        Err(err) => LoadingResult::new_failure(LoadingError::unreachable_file(
            240,
            output.to_path_buf(),
            err.to_string(),
        )),
    })
}

/// Extracts a `.jeu` file in given location.
#[cfg_attr(docsrs, doc(cfg(all(feature = "filesystem", feature = "jeu"))))]
#[cfg(all(feature = "filesystem", feature = "jeu"))]
pub fn extract_jeu(input: &std::path::Path, output: &std::path::Path) -> std::io::Result<()> {
    let output_full_path = {
        let mut path = output.to_path_buf();
        path.push(input.file_stem().unwrap_or_default());
        path
    };

    std::fs::create_dir_all(&output_full_path)?;
    let input_file = std::fs::File::open(input)?;

    JeuPackage::extract(input_file, &output_full_path)
}
