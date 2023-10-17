use std::process::{exit, Command};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("http_javascript.mel")
        .spawn()
        .expect("failed to launch Mélodium executable");

    let mut response = None;

    for trial in 0..3 {
        sleep(Duration::from_millis(200));
        match ureq::post("http://localhost:62333/")
            .set("Content-Type", "text/plain")
            .send_bytes(r#""Pingouin""#.as_bytes())
        {
            Ok(resp) => response = Some(resp),
            Err(err) => eprintln!("Trial {trial} failed: {err}"),
        }
    }

    let _ = melodium.kill();

    if let Some(resp) = response {
        if resp.status() == 200 {
            match resp.into_string() {
                Ok(response) => {
                    let expected =
                        r#"{"ps":"Thanks for contacting me :D","response":"Hello Pingouin!"}"#;
                    if response != expected {
                        eprintln!("Failure, response is:\n{response}");
                        exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("Failure, {err:?}");
                    exit(1);
                }
            }
        } else {
            eprintln!("Failure, code is {}", resp.status());
            exit(1);
        }
    } else {
        eprintln!("Failure, unable to get HTTP response");
        exit(1);
    }
}
