//! The `fs` package removes and renames files and directories, within rooted file systems.

use super::common;
use melodium_common::executive::Value;
use std::collections::HashMap;

const OPERATIONS: &str = include_str!("scripts/fs_operations.mel");

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
