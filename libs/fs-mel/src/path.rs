use async_std::path::PathBuf;
use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Gives compition of paths.
///
/// For each streamed `path`, the outputs are filled with values:
/// - `extension` contains the part of the name after last `.`, else is empty;
/// - `file_name` contains the full name of the file (`bin` for `/usr/bin/`, `bar.txt` for `foo/bar.txt`, `etc` for `etc/./`, empty for `/`);
/// - `file_stem` contains the part of the name before last `.`, or full name if starting with `.` with none in, or not containing any `.` at all;
/// - `parent` contains the path up to the parent of the designated component, else is empty (`/usr` for `/usr/bin`, empty for `/`).
#[mel_treatment(
    input path Stream<string>
    output extension Stream<string>
    output file_name Stream<string>
    output file_stem Stream<string>
    output parent Stream<string>
)]
pub async fn composition() {
    while let Ok(paths) = path.recv_string().await {
        let mut extensions = Vec::with_capacity(paths.len());
        let mut file_names = Vec::with_capacity(paths.len());
        let mut file_stems = Vec::with_capacity(paths.len());
        let mut parents = Vec::with_capacity(paths.len());
        for path in paths {
            let path = PathBuf::from(path);
            extensions.push(
                path.extension()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
            );
            file_names.push(
                path.file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
            );
            file_stems.push(
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
            );
            parents.push(
                path.parent()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default(),
            );
        }
        if let (Err(_), Err(_), Err(_), Err(_)) = futures::join!(
            extension.send_string(extensions),
            file_name.send_string(file_names),
            file_stem.send_string(file_stems),
            parent.send_string(parents)
        ) {
            break;
        }
    }
}

/// Tells if path exists.
///
/// For every streamed `path`, `exists` tells if it exists and is accessible on filesystem.
/// This treatment will traverse symbolic links to query information about the destination file. In case of broken symbolic links this will give `false`.
/// If access to the directory containing the file is forbidden, this will give `false`.
#[mel_treatment(
    input path Stream<string>
    output exists Stream<bool>
)]
pub async fn exists() {
    while let Ok(paths) = path.recv_string().await {
        let mut results = Vec::with_capacity(paths.len());
        for path in paths {
            results.push(PathBuf::from(path).exists().await);
        }
        check!(exists.send_bool(results).await);
    }
}

/// Gives metadata of path.
///
/// For every streamed `path`, tells if it is a directory, a file, and the length of the designated element.
///
#[mel_treatment(
    input path Stream<string>
    output is_dir Stream<bool>
    output is_file Stream<bool>
    output length Stream<u64>
)]
pub async fn meta() {
    while let Ok(paths) = path.recv_string().await {
        let mut are_dirs = Vec::with_capacity(paths.len());
        let mut are_files = Vec::with_capacity(paths.len());
        let mut lengths = Vec::with_capacity(paths.len());
        for path in paths {
            if let Ok(metadata) = async_std::fs::metadata(path).await {
                are_dirs.push(metadata.is_dir());
                are_files.push(metadata.is_file());
                lengths.push(metadata.len());
            } else {
                are_dirs.push(false);
                are_files.push(false);
                lengths.push(0);
            }
        }
        if let (Err(_), Err(_), Err(_)) = futures::join!(
            is_dir.send_bool(are_dirs),
            is_file.send_bool(are_files),
            length.send_u64(lengths)
        ) {
            break;
        }
    }
}

/// Gives file path extension.
///
/// Return the part of the name after last `.`, or empty string.
#[mel_function]
pub fn extension(path: string) -> string {
    PathBuf::from(path)
        .extension()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Gives file name.
///
/// Return the full name of the file (`bin` for `/usr/bin/`, `bar.txt` for `foo/bar.txt`, `etc` for `etc/./`, empty for `/`).
#[mel_function]
pub fn file_name(path: string) -> string {
    PathBuf::from(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Gives file stem.
///
/// Return the part of the name before last `.`, or full name if starting with `.` with none in, or not containing any `.` at all.
#[mel_function]
pub fn file_stem(path: string) -> string {
    PathBuf::from(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Gives parent path.
///
/// Return the path up to the parent of the designated component, or emtpy string if none (`/usr` for `/usr/bin`, empty for `/`).
#[mel_function]
pub fn parent(path: string) -> string {
    PathBuf::from(path)
        .parent()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}
