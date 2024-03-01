use std::process::{exit, Command};

const FILENAME: &str = "neverssl.html";

fn main() {
    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("http_client.mel")
        .arg("--url")
        .arg(r#""http://neverssl.com/""#)
        .arg("--file")
        .arg(&format!(r#""{FILENAME}""#))
        .spawn()
        .expect("failed to launch Mélodium executable");

    match melodium.wait() {
        Ok(status) if status.success() => match std::fs::metadata(FILENAME) {
            Ok(metadata) => {
                if metadata.len() == 0 {
                    eprintln!("File size is null");
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
