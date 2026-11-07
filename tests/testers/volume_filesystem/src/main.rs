//! Volumes of workers are reached through `work/resources::getFileSystem` and the steps of
//! `cicd`, every path staying within the volume: `..`, absolute paths and links leading out
//! of it are refused, for reading, writing, creating directories and scanning, while links
//! staying within it are followed.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tester::{melodium, temp_file, temp_path};

const SCRIPT: &str = include_str!("../../../volume_filesystem.mel");
const STEP_SCRIPT: &str = include_str!("../../../volume_filesystem_step.mel");

const OUTSIDE: &str = "volume-filesystem-outside-sentinel";
const REFUSED: &str = "a path led outside of the filesystem";

/// Directory holding the volume `volume`, with `inside/file` and a link `inner` to `inside`,
/// and `outside` beside it, holding `secret`, with links `out` and `abs` to it in the volume.
struct Fixture {
    base: PathBuf,
    volume: PathBuf,
    outside: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let base = temp_path("volume_filesystem", name);
        let _ = std::fs::remove_dir_all(&base);
        let volume = base.join("volume");
        let outside = base.join("outside");
        std::fs::create_dir_all(volume.join("inside")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(volume.join("inside/file"), "inside").unwrap();
        std::fs::write(outside.join("secret"), OUTSIDE).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("inside", volume.join("inner")).unwrap();
            std::os::unix::fs::symlink("../outside", volume.join("out")).unwrap();
            std::os::unix::fs::symlink(&outside, volume.join("abs")).unwrap();
        }
        Self {
            base,
            volume,
            outside,
        }
    }

    /// Files beside the volume.
    fn outside_files(&self) -> Vec<String> {
        let mut files: Vec<String> = std::fs::read_dir(&self.outside)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        files.sort();
        files
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

/// Runs `script` with the volume `data` at `volume`, as on a worker.
fn run(script: &Path, volume: &Path, args: &[&str]) -> Output {
    let mut command: Command = melodium();
    command
        .arg("run")
        .arg(script)
        .args(args)
        .env("MELODIUM_RUN_VOLUMES", "data")
        .env("MELODIUM_RUN_VOLUME_data", volume);
    command.output().unwrap()
}

fn assert_contains(text: &str, expected: &str) {
    assert!(
        text.contains(expected),
        "expected {:?} in:\n{}",
        expected,
        text
    );
}

fn files_within_the_volume_are_reached() {
    let script = temp_file("volume_filesystem", "script.mel", SCRIPT);
    let fixture = Fixture::new("within");
    let read = if cfg!(unix) {
        "inner/file"
    } else {
        "inside/file"
    };

    let output = run(
        &script,
        &fixture.volume,
        &[
            "--read_path",
            read,
            "--write_path",
            "new/deep/written.txt",
            "--dir_path",
            "made/dir",
            "--scan_path",
            "inside",
        ],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "read: inside");
    assert_contains(&stdout, "write: done");
    assert_contains(&stdout, "create: done");
    assert_contains(
        &stdout,
        &format!("entry: {}", fixture.volume.join("inside/file").display()),
    );
    assert!(!stdout.contains("error"), "{}", stdout);
    assert_eq!(
        std::fs::read_to_string(fixture.volume.join("new/deep/written.txt")).unwrap(),
        "written"
    );
    assert!(fixture.volume.join("made/dir").is_dir());
}

fn paths_leaving_the_volume_are_refused() {
    let script = temp_file("volume_filesystem", "script.mel", SCRIPT);
    let fixture = Fixture::new("leaving");
    let absolute = fixture.outside.to_string_lossy().to_string();

    let mut leaving = vec!["../outside", "inside/../../outside", absolute.as_str()];
    // Links within the volume leading out of it, as executors sharing it can make.
    if cfg!(unix) {
        leaving.extend(["out", "abs"]);
    }
    for directory in leaving {
        let output = run(
            &script,
            &fixture.volume,
            &[
                "--read_path",
                &format!("{directory}/secret"),
                "--write_path",
                &format!("{directory}/written.txt"),
                "--dir_path",
                &format!("{directory}/made"),
                "--scan_path",
                directory,
            ],
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        for refused in ["read-error", "write-error", "create-error", "scan-error"] {
            assert_contains(&stdout, &format!("{refused}: {REFUSED}"));
        }
        assert!(!stdout.contains(OUTSIDE), "{directory}: {stdout}");
        assert!(!stdout.contains("write: done"), "{directory}: {stdout}");
        assert!(!stdout.contains("create: done"), "{directory}: {stdout}");
        assert!(!stdout.contains("entry:"), "{directory}: {stdout}");
        assert_eq!(
            fixture.outside_files(),
            vec!["secret".to_string()],
            "{directory}"
        );
    }
}

fn steps_keep_their_files_within_the_volume() {
    let step = temp_file("volume_filesystem", "step.mel", STEP_SCRIPT);
    let fixture = Fixture::new("step");

    let output = run(
        &step,
        &fixture.volume,
        &["--in_file", "step/in.txt", "--out_file", "step/in.txt"],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{:?}", output);
    assert_contains(&stdout, "out: step-input");
    assert_eq!(
        std::fs::read_to_string(fixture.volume.join("step/in.txt")).unwrap(),
        "step-input"
    );

    let output = run(
        &step,
        &fixture.volume,
        &[
            "--in_file",
            "../outside/in.txt",
            "--out_file",
            "../outside/secret",
        ],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_contains(&stdout, &format!("step: {REFUSED}"));
    assert!(!stdout.contains(OUTSIDE), "{}", stdout);
    assert_eq!(fixture.outside_files(), vec!["secret".to_string()]);

    if cfg!(unix) {
        let output = run(
            &step,
            &fixture.volume,
            &["--in_file", "inside/in.txt", "--out_file", "out/secret"],
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_contains(&stdout, &format!("step: {REFUSED}"));
        assert!(!stdout.contains(OUTSIDE), "{}", stdout);
    }
}

pub fn run_cases() -> ! {
    tester::cases(&[
        (
            "files_within_the_volume_are_reached",
            files_within_the_volume_are_reached,
        ),
        (
            "paths_leaving_the_volume_are_refused",
            paths_leaving_the_volume_are_refused,
        ),
        (
            "steps_keep_their_files_within_the_volume",
            steps_keep_their_files_within_the_volume,
        ),
    ])
}

fn main() {
    run_cases();
}
