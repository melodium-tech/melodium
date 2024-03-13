use std::env;
use std::process::{exit, Command};

const CONN_ERROR_FILENAME: &str = "connection_error";
const EXEC_ERROR_FILENAME: &str = "execution_error";
const SUCCESS_FILENAME: &str = "success_affected";

fn main() {
    if env::var("CI").is_ok() && (cfg!(target_os = "windows") || cfg!(target_os = "apple")) {
        // On CI for Windows and Mac, for now, we skip SQL tests as we have no acceptable solution
        // to get a working and available DBMS to make tests with (see tests.yml file).
        exit(0);
    }

    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("sql.mel")
        .arg("--server_url")
        .arg(&format!(
            r#""postgres://{user}:{password}@{host}:5432/{database}""#,
            user = env::var("POSTGRES_USER").unwrap(),
            password = env::var("POSTGRES_PASSWORD").unwrap(),
            host = env::var("POSTGRES_HOST").unwrap(),
            database = env::var("POSTGRES_DB").unwrap(),
        ))
        .arg("--conn_error_file")
        .arg(&format!(r#""{CONN_ERROR_FILENAME}""#))
        .arg("--exec_error_file")
        .arg(&format!(r#""{EXEC_ERROR_FILENAME}""#))
        .arg("--success_file")
        .arg(&format!(r#""{SUCCESS_FILENAME}""#))
        .spawn()
        .expect("failed to launch Mélodium executable");

    match melodium.wait() {
        Ok(status) if status.success() => {
            if let Ok(error_contents) = std::fs::read_to_string(CONN_ERROR_FILENAME) {
                if !error_contents.is_empty() {
                    eprintln!("SQL connection error: {error_contents}");
                }
            }

            if let Ok(error_contents) = std::fs::read_to_string(EXEC_ERROR_FILENAME) {
                if !error_contents.is_empty() {
                    eprintln!("SQL execution error: {error_contents}");
                }
            }

            match std::fs::metadata(SUCCESS_FILENAME) {
                Ok(_metadata) => match std::fs::read_to_string(SUCCESS_FILENAME) {
                    Ok(contents) => {
                        if contents != "8" {
                            eprintln!("Invalid result content: {contents}");
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
            }
        }
        Ok(status) => {
            exit(status.code().unwrap_or(1));
        }
        Err(err) => {
            eprintln!("Execution error: {err}");
            exit(1);
        }
    }

    let _ = std::fs::remove_file(CONN_ERROR_FILENAME);
    let _ = std::fs::remove_file(EXEC_ERROR_FILENAME);
    let _ = std::fs::remove_file(SUCCESS_FILENAME);
}
