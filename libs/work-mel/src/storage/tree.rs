//! File trees of volumes, as run artifacts hold them.

use melodium_core::common::executive::RevealedScan;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// File of a tree.
pub(crate) struct TreeFile {
    /// Path within the tree, its parts separated by `/`.
    pub path: String,
    pub full: PathBuf,
    pub size: u64,
}

/// Lists the files of the tree at `root`: the files of a directory and of its
/// subdirectories, or a file alone, named as it is. Links are not followed, and are given
/// apart.
pub(crate) fn list(root: &Path) -> Result<(Vec<TreeFile>, Vec<String>), String> {
    let metadata = std::fs::symlink_metadata(root).map_err(|err| err.to_string())?;
    let mut files = Vec::new();
    let mut links = Vec::new();
    if metadata.is_file() {
        let name = root
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "its name is not valid UTF-8".to_string())?;
        files.push(TreeFile {
            path: name.to_string(),
            full: root.to_path_buf(),
            size: metadata.len(),
        });
    } else if metadata.is_dir() {
        walk(root, "", &mut files, &mut links)?;
    } else if metadata.is_symlink() {
        return Err("it is a link, artifacts hold files only".to_string());
    } else {
        return Err("it is neither a file nor a directory".to_string());
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((files, links))
}

fn walk(
    directory: &Path,
    prefix: &str,
    files: &mut Vec<TreeFile>,
    links: &mut Vec<String>,
) -> Result<(), String> {
    for entry in std::fs::read_dir(directory).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let name = entry.file_name();
        let name = name.to_str().ok_or_else(|| {
            format!(
                "'{}' has a name that is not valid UTF-8",
                entry.path().display()
            )
        })?;
        let path = format!("{prefix}{name}");
        let metadata = entry.metadata().map_err(|err| err.to_string())?;
        if metadata.is_symlink() {
            links.push(path);
        } else if metadata.is_dir() {
            walk(&entry.path(), &format!("{path}/"), files, links)?;
        } else if metadata.is_file() {
            files.push(TreeFile {
                path,
                full: entry.path(),
                size: metadata.len(),
            });
        }
    }
    Ok(())
}

/// Checks that `path` is a path within a tree: relative, its parts separated by `/`, with
/// no empty, `.` or `..` part.
pub(crate) fn check_path(path: &str) -> Result<(), String> {
    if !path.is_empty()
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != ".." && !part.contains('\\'))
    {
        Ok(())
    } else {
        Err(format!(
            "'{path}' is not a path within an artifact, relative, its parts separated by '/', without '.' or '..'"
        ))
    }
}

/// Reads `file` whole, giving its sha256, unless `scan` finds a value revealed by the
/// engine in it, whose secret is then given.
pub(crate) fn hash_and_scan(
    file: &TreeFile,
    scan: &mut RevealedScan,
) -> Result<Result<String, String>, String> {
    let mut reader = File::open(&file.full).map_err(|err| format!("'{}': {err}", file.path))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 20];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|err| format!("'{}': {err}", file.path))?;
        if read == 0 {
            break;
        }
        if let Some(name) = scan.scan(&buffer[..read]) {
            return Ok(Err(name));
        }
        hasher.update(&buffer[..read]);
    }
    Ok(Ok(hex::encode(hasher.finalize())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trees_list_files_and_links() {
        let root = std::env::temp_dir().join(format!("work-mel-tree-{}", std::process::id()));
        std::fs::create_dir_all(root.join("assets/img")).unwrap();
        std::fs::write(root.join("index.html"), b"<html>").unwrap();
        std::fs::write(root.join("assets/img/logo.svg"), b"<svg>").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("/etc/passwd", root.join("assets/passwd")).unwrap();

        let (files, links) = list(&root).unwrap();
        assert_eq!(
            files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>(),
            vec!["assets/img/logo.svg", "index.html"]
        );
        #[cfg(unix)]
        assert_eq!(links, vec!["assets/passwd".to_string()]);

        let (files, _) = list(&root.join("index.html")).unwrap();
        assert_eq!(files[0].path, "index.html");
        assert_eq!(files[0].size, 6);

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn paths_stay_within_trees() {
        for valid in ["index.html", "assets/img/logo.svg", ".hidden/file"] {
            assert!(check_path(valid).is_ok(), "{valid}");
        }
        for invalid in [
            "",
            "/etc/passwd",
            "../up",
            "a/../b",
            "a//b",
            "a/./b",
            "a\\b",
            "a/",
        ] {
            assert!(check_path(invalid).is_err(), "{invalid}");
        }
    }
}
