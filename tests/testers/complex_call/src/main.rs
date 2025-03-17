use std::process::{exit, Command};

const FILENAME: &str = "output_count";
const EXPECTED_SIZE: u64 = 16;
const CONTENTS: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x6b,
];

fn main() {
    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("complex_call.mel")
        .arg("--output")
        .arg(r#""./output_count""#)
        .spawn()
        .expect("failed to launch Mélodium executable");

    match melodium.wait() {
        Ok(status) if status.success() => match std::fs::metadata(FILENAME) {
            Ok(metadata) => {
                if metadata.len() != EXPECTED_SIZE {
                    eprintln!("File size is not {EXPECTED_SIZE} bytes");
                    exit(1);
                }

                match std::fs::read(FILENAME) {
                    Ok(contents) => {
                        if contents != CONTENTS {
                            eprintln!("Invalid result content");
                            exit(1);
                        }
                    }
                    Err(err) => {
                        eprintln!("Error reading file: {err}");
                        exit(1);
                    }
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
