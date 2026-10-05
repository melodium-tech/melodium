use std::process::{exit, Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

// A run interrupted by SIGTERM on Unix, or by Ctrl+Break on Windows, ends within its
// grace period, logs the interruption, and exits with the code of the interruption.
// `interruption.mel` runs a command lasting 60 seconds, much longer than the grace
// period (10 seconds) and the time given afterwards to write logs out (5 seconds).

const START_DELAY: Duration = Duration::from_secs(3);
const EXIT_TIMEOUT: Duration = Duration::from_secs(25);

#[cfg(unix)]
const EXPECTED_CODE: i32 = 143;
#[cfg(unix)]
const EXPECTED_NAME: &str = "SIGTERM";
#[cfg(unix)]
const COMMAND: (&str, &str) = ("sleep", r#"["60"]"#);

/// `STATUS_CONTROL_C_EXIT`
#[cfg(windows)]
const EXPECTED_CODE: i32 = 0xC000013Au32 as i32;
#[cfg(windows)]
const EXPECTED_NAME: &str = "Ctrl+Break";
#[cfg(windows)]
const COMMAND: (&str, &str) = (
    "powershell",
    r#"["-NoProfile", "-Command", "Start-Sleep -Seconds 60"]"#,
);

fn main() {
    let _ = std::fs::remove_file("interruption.logs");

    let mut melodium = spawn();
    sleep(START_DELAY);
    if let Err(error) = interrupt(&melodium) {
        eprintln!("Failure, unable to interrupt the program: {error}");
        let _ = melodium.kill();
        exit(1);
    }

    let start = Instant::now();
    let status = wait_with_timeout(&mut melodium, EXIT_TIMEOUT);
    let _ = melodium.kill();
    let logs = std::fs::read_to_string("interruption.logs").unwrap_or_default();

    match status {
        Some(status) if status.code() == Some(EXPECTED_CODE) => {}
        Some(status) => {
            eprintln!("Failure, program exited with unexpected status {status}\n{logs}");
            exit(1);
        }
        None => {
            eprintln!(
                "Failure, program still running {EXIT_TIMEOUT:?} after the interruption\n{logs}"
            );
            exit(1);
        }
    }
    if !logs.contains(&format!("interrupted by {EXPECTED_NAME}")) {
        eprintln!("Failure, interruption not logged\n{logs}");
        exit(1);
    }
    eprintln!(
        "Program ended {:?} after {EXPECTED_NAME}, with code {EXPECTED_CODE}, as expected",
        start.elapsed()
    );
}

fn spawn() -> Child {
    let mut melodium = Command::new("melodium");
    melodium
        .args([
            "run",
            "--parse-arguments",
            "--logs",
            "interruption.logs",
            "interruption.mel",
            "--command",
        ])
        .arg(format!("{:?}", COMMAND.0))
        .arg("--arguments")
        .arg(COMMAND.1);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // In its own process group, so that Ctrl+Break reaches only it and its children.
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        melodium.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }
    melodium
        .spawn()
        .expect("failed to launch Mélodium executable")
}

#[cfg(unix)]
fn interrupt(melodium: &Child) -> Result<(), String> {
    let status = Command::new("kill")
        .args(["-TERM", &melodium.id().to_string()])
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("kill exited with {status}"))
    }
}

#[cfg(windows)]
fn interrupt(melodium: &Child) -> Result<(), String> {
    #[link(name = "kernel32")]
    extern "system" {
        fn GenerateConsoleCtrlEvent(event: u32, group: u32) -> i32;
        fn FreeConsole() -> i32;
        fn AttachConsole(process: u32) -> i32;
    }
    const CTRL_BREAK_EVENT: u32 = 1;

    // The event only reaches processes sharing the console of the sender: without a
    // common console, the harness attaches to the one of the program.
    unsafe {
        if GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, melodium.id()) != 0 {
            return Ok(());
        }
        FreeConsole();
        if AttachConsole(melodium.id()) == 0 {
            return Err(format!(
                "cannot attach to its console: {}",
                std::io::Error::last_os_error()
            ));
        }
        if GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, melodium.id()) != 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error().to_string())
        }
    }
}

fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Option<std::process::ExitStatus> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    return None;
                }
                sleep(Duration::from_millis(100));
            }
            Err(_) => return None,
        }
    }
}
