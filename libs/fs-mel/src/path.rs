use async_std::path::PathBuf;
use melodium_core::*;
use melodium_macro::{check, mel_function, mel_treatment};

/// Gives compition of paths.
///
/// For each streamed `path`, the outputs are filled with values:
/// - `extension` contains the part of the name after last `.`, else is none;
/// - `file_name` contains the full name of the file (`bin` for `/usr/bin/`, `bar.txt` for `foo/bar.txt`, `etc` for `etc/./`, empty for `/`);
/// - `file_stem` contains the part of the name before last `.`, or full name if starting with `.` with none in, or not containing any `.` at all;
/// - `parent` contains the path up to the parent of the designated component, else is empty (`/usr` for `/usr/bin`, empty for `/`).
#[mel_treatment(
    input path Stream<string>
    output extension Stream<Option<string>>
    output file_name Stream<Option<string>>
    output file_stem Stream<Option<string>>
    output parent Stream<Option<string>>
)]
pub async fn composition() {
    while let Ok(paths) = path
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        let mut extensions = VecDeque::with_capacity(paths.len());
        let mut file_names = VecDeque::with_capacity(paths.len());
        let mut file_stems = VecDeque::with_capacity(paths.len());
        let mut parents = VecDeque::with_capacity(paths.len());
        for path in paths {
            let path = PathBuf::from(path);
            extensions.push_back(Value::Option(
                path.extension()
                    .map(|s| Box::new(Value::from(s.to_string_lossy().to_string()))),
            ));
            file_names.push_back(Value::Option(
                path.file_name()
                    .map(|s| Box::new(Value::from(s.to_string_lossy().to_string()))),
            ));
            file_stems.push_back(Value::Option(
                path.file_stem()
                    .map(|s| Box::new(Value::from(s.to_string_lossy().to_string()))),
            ));
            parents.push_back(Value::Option(
                path.parent()
                    .map(|s| Box::new(Value::from(s.to_string_lossy().to_string()))),
            ));
        }
        if let (Err(_), Err(_), Err(_), Err(_)) = futures::join!(
            extension.send_many(TransmissionValue::Other(extensions)),
            file_name.send_many(TransmissionValue::Other(file_names)),
            file_stem.send_many(TransmissionValue::Other(file_stems)),
            parent.send_many(TransmissionValue::Other(parents))
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
    #[cfg(feature = "real")]
    while let Ok(paths) = path
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        let mut results = Vec::with_capacity(paths.len());
        for path in paths {
            results.push(PathBuf::from(path).exists().await);
        }
        check!(exists.send_many(results.into()).await);
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
    #[cfg(feature = "real")]
    while let Ok(paths) = path
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
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
            is_dir.send_many(are_dirs.into()),
            is_file.send_many(are_files.into()),
            length.send_many(lengths.into())
        ) {
            break;
        }
    }
}

/// Gives file path extension.
///
/// Return the part of the name after last `.`, or none if there is no extension.
///
/// The extension is:
///
/// - none, if there is no file name
/// - none, if there is no embedded `.`
/// - none, if the file name begins with `.` and has no other `.`s within
/// - otherwise, the portion of the file name after the final `.`
#[mel_function]
pub fn extension(path: string) -> Option<string> {
    PathBuf::from(path)
        .extension()
        .map(|s| s.to_string_lossy().to_string())
}

/// Gives file name.
///
/// Return the full name of the file (`bin` for `/usr/bin/`, `bar.txt` for `foo/bar.txt`, `etc` for `etc/./`, empty for `/`).
///
/// If the path is a normal file, this is the file name. If it's the path of a directory, this is the directory name.
///
/// Returns none if the path terminates in `..`.
#[mel_function]
pub fn file_name(path: string) -> Option<string> {
    PathBuf::from(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
}

/// Gives file stem.
///
/// Return the part of the name before last `.`, or full name if starting with `.` with none in, or not containing any `.` at all.
///
/// The stem is:
///
/// - none, if there is no file name
/// - the entire file name if there is no embedded `.`
/// - the entire file name if the file name begins with `.` and has no other `.`s within
/// - otherwise, the portion of the file name before the final `.`
#[mel_function]
pub fn file_stem(path: string) -> Option<string> {
    PathBuf::from(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
}

/// Gives parent path.
///
/// Return the path up to the parent of the designated component (`/usr` for `/usr/bin`, none for `/`).
///
/// Returns none if the path terminates in a root or prefix.
#[mel_function]
pub fn parent(path: string) -> Option<string> {
    PathBuf::from(path)
        .parent()
        .map(|s| s.to_string_lossy().to_string())
}
