use std::process::{exit, Command, Stdio};

const FILENAME: &str = "fs_try.data";
const EXPECTED_SIZE: u64 = 128000 * 16;

fn main() {
    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("fs.mel")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to launch Mélodium executable");

    match melodium.wait() {
        Ok(status) if status.success() => match std::fs::metadata(FILENAME) {
            Ok(metadata) => {
                if metadata.len() != EXPECTED_SIZE {
                    eprintln!("File size is not {EXPECTED_SIZE} bytes");
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("Error retrieving metadata: {err}");
                exit(1);
            }
        },
        Ok(status) => {
            exit(status.code().unwrap_or(1));
        }
        Err(err) => {
            eprintln!("Execution error: {err}");
            exit(1);
        }
    }

    let _ = std::fs::remove_file(FILENAME);
}
