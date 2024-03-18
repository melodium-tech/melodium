use std::process::{exit, Command};

const FILENAME: &str = "output_number_regex";
const INPUT_STRING: &str = "reyzerytnvz_ruty,àrûthtyjyjjy$$ù        🟦0123456 % 🚀 ";
const EXPECTED_CONTENT: &str = "0123456";

fn main() {
    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("regex.mel")
        .arg("--file")
        .arg(&format!(r#""{FILENAME}""#))
        .arg("--text")
        .arg(&format!(r#""{INPUT_STRING}""#))
        .spawn()
        .expect("failed to launch Mélodium executable");

    match melodium.wait() {
        Ok(status) if status.success() => match std::fs::metadata(FILENAME) {
            Ok(_metadata) => match std::fs::read_to_string(FILENAME) {
                Ok(contents) => {
                    if contents != EXPECTED_CONTENT {
                        eprintln!("Invalid result content");
                        exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("Error reading file: {err}");
                    exit(1);
                }
            },
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
