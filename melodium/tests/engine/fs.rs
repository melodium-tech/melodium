//! The `fs` package removes and renames files and directories, lists the files of trees, and
//! archives and extracts directories, within rooted file systems.

use super::common;
use melodium_common::executive::Value;
use std::collections::HashMap;

const OPERATIONS: &str = include_str!("scripts/fs_operations.mel");
const FILES: &str = include_str!("scripts/fs_files.mel");

#[test]
fn files_and_directories_are_renamed_and_removed() {
    let root = std::env::temp_dir().join(format!("melodium-fs-operations-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for dir in ["staging", "old/nested", "kept"] {
        std::fs::create_dir_all(root.join(dir)).unwrap();
    }
    std::fs::write(root.join("staging/restored"), b"restored").unwrap();
    std::fs::write(root.join("old/nested/stale"), b"stale").unwrap();
    std::fs::write(root.join("file"), b"file").unwrap();

    let (logs, events) = common::run(
        OPERATIONS,
        HashMap::from([(
            "root".to_string(),
            Value::String(root.to_string_lossy().to_string()),
        )]),
    );

    assert_eq!(common::sent(&events, "rename", "completed").len(), 1);
    assert_eq!(common::sent(&events, "removeDir", "completed").len(), 1);
    assert_eq!(common::sent(&events, "removeFile", "completed").len(), 1);
    assert_eq!(
        std::fs::read(root.join("target/restored")).unwrap(),
        b"restored"
    );
    assert!(!root.join("staging").exists());
    assert!(!root.join("old").exists());
    assert!(!root.join("file").exists());
    assert!(root.join("kept").is_dir());
    assert!(
        common::message(&logs, "not a file").contains("is a directory"),
        "{logs:?}"
    );
    std::fs::remove_dir_all(&root).unwrap();
}

const ARCHIVE: &str = include_str!("scripts/fs_archive.mel");

#[test]
fn directories_are_archived_and_extracted() {
    let root = std::env::temp_dir().join(format!("melodium-fs-archive-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("source/nested")).unwrap();
    std::fs::write(root.join("source/nested/file"), vec![3u8; 3_000_000]).unwrap();
    std::fs::write(root.join("source/small"), b"small").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink("nested/file", root.join("source/link")).unwrap();

    let (logs, events) = common::run(
        ARCHIVE,
        HashMap::from([(
            "root".to_string(),
            Value::String(root.to_string_lossy().to_string()),
        )]),
    );

    assert!(logs.is_empty(), "{logs:?}");
    assert_eq!(common::sent(&events, "tar", "completed").len(), 1);
    assert_eq!(common::sent(&events, "untar", "completed").len(), 1);
    assert_eq!(
        std::fs::read(root.join("copy/nested/file")).unwrap(),
        vec![3u8; 3_000_000]
    );
    assert_eq!(std::fs::read(root.join("copy/small")).unwrap(), b"small");
    #[cfg(unix)]
    assert_eq!(
        std::fs::read_link(root.join("copy/link")).unwrap(),
        std::path::Path::new("nested/file")
    );
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
#[cfg(unix)]
fn files_of_trees_are_listed() {
    let root = std::env::temp_dir().join(format!("melodium-fs-files-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("tree/site/css")).unwrap();
    std::fs::write(root.join("tree/index.html"), b"index").unwrap();
    std::fs::write(root.join("tree/site/css/main.css"), b"css").unwrap();
    std::os::unix::fs::symlink("index.html", root.join("tree/home.html")).unwrap();

    let (logs, events) = common::run(
        FILES,
        HashMap::from([(
            "root".to_string(),
            Value::String(root.to_string_lossy().to_string()),
        )]),
    );

    let files = logs
        .iter()
        .filter(|log| log.label == "file")
        .map(|log| log.message.clone())
        .collect::<Vec<_>>();
    assert_eq!(files, ["index.html", "site/css/main.css"], "{logs:?}");
    assert_eq!(common::sent(&events, "listed", "check").len(), 1);
    std::fs::remove_dir_all(&root).unwrap();
}
