use std::process::{exit, Command};

const FILENAME: &str = "replaced.txt";
const EXPECTED_CONTENT: &str = "This is bar!";

fn main() {
    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("process.mel")
        .spawn()
        .expect("failed to launch Mélodium executable");

    match melodium.wait() {
        Ok(status) if status.success() => match std::fs::metadata(FILENAME) {
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
