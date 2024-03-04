use std::process::{exit, Command};

fn main() {
    // Without TLS
    test_download("http://neverssl.com/", "neverssl.html", "neverssl.log");
    // With TLS
    test_download("https://melodium.tech/", "melodium_withtls.html", "tls.log");
}

fn test_download(url: &str, file: &str, log: &str) {
    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("http_client.mel")
        .arg("--url")
        .arg(&format!(r#""{url}""#))
        .arg("--file")
        .arg(&format!(r#""{file}""#))
        .arg("--log")
        .arg(&format!(r#""{log}""#))
        .spawn()
        .expect("failed to launch Mélodium executable");

    match melodium.wait() {
        Ok(status) if status.success() => match std::fs::metadata(file) {
            Ok(metadata) => {
                let failure = std::fs::read_to_string(log).unwrap();
                if !failure.is_empty() {
                    eprintln!("Download failure: {failure}");
                    exit(1);
                }

                if metadata.len() == 0 {
                    eprintln!("File size for '{file}' is null");
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("Error retrieving metadata for '{file}': {err}");
                exit(1);
            }
        },
        Ok(status) => {
            exit(status.code().unwrap_or(1));
        }
        Err(err) => {
            eprintln!("Execution error for '{url}' > '{file}': {err}");
            exit(1);
        }
    }

    let _ = std::fs::remove_file(file);
}
