use std::process::{exit, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    if std::env::var("CI").is_ok() && cfg!(target_env = "msvc") {
        // On CI for Windows MSVC, for now, we skip tests involving TLS because schannel is not able to register custom certificates.
        exit(0);
    }

    let mut melodium_distrib = Command::new("melodium")
        .arg("dist")
        .arg("--localhost")
        .arg("--port")
        .arg("28014")
        .arg("--recv-key")
        .arg("d0bf1006-a851-50eb-b32b-5f443d642ce6")
        .arg("--send-key")
        .arg("9a1bed00-1051-565e-b418-f3b32462620d")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to launch Mélodium executable");
    let mut melodium = Command::new("melodium")
        .arg("run")
        .arg("http_distributed.mel")
        .arg("--distrib_port")
        .arg("28014")
        .arg("--remote_key")
        .arg("\"d0bf1006-a851-50eb-b32b-5f443d642ce6\"")
        .arg("--self_key")
        .arg("\"9a1bed00-1051-565e-b418-f3b32462620d\"")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to launch Mélodium executable");

    let mut response = None;

    for trial in 0..3 {
        sleep(Duration::from_millis(500));
        match ureq::post("http://localhost:8080/hello")
            .config()
            .timeout_global(Some(Duration::from_secs(1)))
            .build()
            .header("Content-Type", "text/plain")
            .send(r#""Pingouin""#.as_bytes())
        {
            Ok(resp) => response = Some(resp),
            Err(err) => eprintln!("Trial {trial} failed: {err}"),
        }
    }

    let _ = melodium.kill();
    let _ = melodium_distrib.kill();

    eprintln!(
        "failure.log: {}",
        std::fs::read_to_string("failure.log").unwrap_or("Rien".to_string())
    );

    if let Some(mut resp) = response {
        if resp.status() == 200 {
            match resp.body_mut().read_to_string() {
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
