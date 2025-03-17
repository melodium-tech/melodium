use std::process::{exit, Command};

fn main() {
    // Without TLS
    test_download(
        "http://testing.melodium.tech/no-tls.txt",
        "no-tls.txt",
        "no-tls.log",
    );
    // With TLS
    test_download(
        "https://testing.melodium.tech/tls.txt",
        "tls.txt",
        "tls.log",
    );
    test_download(
        "https://melodium.tech/",
        "melodium_withtls.html",
        "melodium_withtls.log",
    );
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
                match std::fs::read_to_string(log) {
                    Ok(content) if !content.is_empty() => {
                        eprintln!("Download log: {content}");
                        exit(1);
                    }
                    _ => {}
                }

                if metadata.len() == 0 {
                    eprintln!("File size for '{file}' is null");
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("Error retrieving metadata for '{file}': {err}");
                match std::fs::read_to_string(log) {
                    Ok(content) if !content.is_empty() => {
                        eprintln!("Download log: {content}");
                        exit(1);
                    }
                    _ => {}
                }
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
