//! Programs created by `melodium new` from the CI/CD template pass `melodium check`
//! for each of their entrypoints.

use std::{path::PathBuf, process::Command};

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "melodium_new_templates_{}_{name}",
        std::process::id()
    ))
}

#[test]
fn cicd_template_entrypoints_check() {
    let directory = temp_path("cicd");
    let _ = std::fs::remove_dir_all(&directory);

    let output = Command::new(env!("CARGO_BIN_EXE_melodium"))
        .arg("new")
        .arg("--template")
        .arg("cicd")
        .arg("--path")
        .arg(&directory)
        .arg("new_cicd")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "melodium new failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    for entrypoint in ["main", "advanced"] {
        let output = Command::new(env!("CARGO_BIN_EXE_melodium"))
            .arg("check")
            .arg(directory.join("Compo.toml"))
            .arg(entrypoint)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "entrypoint {entrypoint} does not check: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let _ = std::fs::remove_dir_all(&directory);
}
