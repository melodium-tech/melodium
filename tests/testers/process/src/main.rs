use std::process::{exit, Command, Stdio};

const FILENAME: &str = "replaced.txt";
const EXPECTED_CONTENT: &str = "This is bar!";
const FILENAME_FAILURE: &str = "failure.txt";
const FILENAME_ERROR: &str = "error.txt";

fn main() {
    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("process.mel")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to launch Mélodium executable");

    let output = melodium.wait_with_output();
    println!(
        "{}",
        String::from_utf8_lossy(&output.as_ref().unwrap().stdout)
    );
    println!(
        "{}",
        String::from_utf8_lossy(&output.as_ref().unwrap().stderr)
    );
    match output.map(|o| o.status) {
        Ok(status) if status.success() => {
            match std::fs::metadata(FILENAME_ERROR) {
                Ok(metadata) if metadata.len() != 0 => {
                    match std::fs::read_to_string(FILENAME_ERROR) {
                        Ok(content) => {
                            eprintln!("Data has been received on stderr: {content}");
                        }
                        Err(err) => {
                            eprintln!("Data has been received on stderr but cannot be read: {err}");
                        }
                    }
                }
                _ => {}
            }

            match std::fs::metadata(FILENAME) {
                Ok(metadata) => {
                    if metadata.len() as usize != EXPECTED_CONTENT.len() {
                        eprintln!("File size is not {} bytes", EXPECTED_CONTENT.len());
                        exit(1);
                    }

                    match std::fs::read_to_string(FILENAME) {
                        Ok(content) if content.as_str() != EXPECTED_CONTENT => {
                            eprintln!("File don't contain expected content");
                            exit(1);
                        }
                        Err(err) => {
                            eprintln!("Error reading file: {err}");
                            exit(1);
                        }
                        Ok(_) => {}
                    }
                }
                Err(err) => {
                    eprintln!("Error retrieving metadata: {err}");
                    exit(1);
                }
            }
        }
        Ok(status) => {
            match std::fs::read_to_string(FILENAME_FAILURE) {
                Ok(content) => {
                    eprintln!("Failure: {content}");
                }
                Err(err) => {
                    eprintln!("Error reading failure file: {err}");
                }
            }
            exit(status.code().unwrap_or(1));
        }
        Err(err) => {
            eprintln!("Execution error: {err}");
            exit(1);
        }
    }

    let _ = std::fs::remove_file(FILENAME);
    let _ = std::fs::remove_file(FILENAME_FAILURE);
    let _ = std::fs::remove_file(FILENAME_ERROR);
}
