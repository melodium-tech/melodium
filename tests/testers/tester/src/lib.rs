//! Helpers of the testers: running `melodium`, running cases, temporary files, shell
//! commands, interruptions and signals.
//!
//! A tester runs from the `tests` directory, with the `melodium` executable to test in its
//! `PATH`. It exits with a non-zero code if any of its cases fails.

use std::path::PathBuf;
use std::process::{exit, Child, Command};

/// Command running the `melodium` executable under test.
pub fn melodium() -> Command {
    Command::new("melodium")
}

/// Runs `cases` one after the other, printing the outcome of each, and exits with a
/// non-zero code if any of them panics, as a failed assertion does.
pub fn cases(cases: &[(&str, fn())]) -> ! {
    let mut failed = Vec::new();
    for (name, case) in cases {
        println!("case {name} ...");
        match std::panic::catch_unwind(case) {
            Ok(()) => println!("case {name} ok"),
            Err(_) => {
                println!("case {name} FAILED");
                failed.push(*name);
            }
        }
    }
    if failed.is_empty() {
        exit(0)
    } else {
        eprintln!("Failure, failed cases: {}", failed.join(", "));
        exit(1)
    }
}

/// Exits successfully, telling why the tester does not run on this system.
pub fn skip(reason: &str) -> ! {
    println!("skipped: {reason}");
    exit(0)
}

/// Path in the temporary directory, unique to the tester `tester` and to this run.
pub fn temp_path(tester: &str, name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("melodium_{tester}_{}_{name}", std::process::id()))
}

/// Writes `content` to a file in the temporary directory, see `temp_path`.
pub fn temp_file(tester: &str, name: &str, content: &str) -> PathBuf {
    let path = temp_path(tester, name);
    std::fs::write(&path, content).unwrap();
    path
}

#[cfg(unix)]
pub const SIGHUP: i32 = 1;
#[cfg(unix)]
pub const SIGINT: i32 = 2;
#[cfg(unix)]
pub const SIGTERM: i32 = 15;

/// Sends `signal` to the process `pid`.
///
/// Through the system call, as the `kill` command is not installed in every test image.
#[cfg(unix)]
pub fn signal(pid: u32, signal: i32) -> Result<(), String> {
    extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    if unsafe { kill(pid as i32, signal) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

/// Arguments making a program run its shell commands with PowerShell, giving `commands`,
/// on Windows, and nothing elsewhere.
///
/// Programs running shell commands take them as `commands`, run as `<shell> -c <commands>`,
/// with `sh` as default `shell` and POSIX shell commands as default `commands`.
pub fn powershell(commands: &str) -> Vec<&str> {
    if cfg!(windows) {
        vec!["--shell", "powershell", "--commands", commands]
    } else {
        Vec::new()
    }
}

/// Makes `command` run in its own process group on Windows, so that `interrupt` reaches
/// only it and its children.
pub fn interruptible(command: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }
    command
}

/// Interrupts `child`, as a user does from the terminal: SIGINT on Unix, Ctrl+Break on
/// Windows, `child` being spawned from a command made `interruptible`.
pub fn interrupt(child: &Child) -> Result<(), String> {
    #[cfg(unix)]
    {
        signal(child.id(), SIGINT)
    }
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn GenerateConsoleCtrlEvent(event: u32, group: u32) -> i32;
            fn FreeConsole() -> i32;
            fn AttachConsole(process: u32) -> i32;
        }
        const CTRL_BREAK_EVENT: u32 = 1;

        // The event only reaches processes sharing the console of the sender: without a
        // common console, the tester attaches to the one of the child.
        unsafe {
            if GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, child.id()) != 0 {
                return Ok(());
            }
            FreeConsole();
            if AttachConsole(child.id()) == 0 {
                return Err(format!(
                    "cannot attach to its console: {}",
                    std::io::Error::last_os_error()
                ));
            }
            if GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, child.id()) != 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error().to_string())
            }
        }
    }
}
