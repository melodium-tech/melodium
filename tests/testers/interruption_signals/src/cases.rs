use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Runs `interruption_signals_sleeper.mel`, running `sleep <duration>` as child process.
fn sleeper(duration: &str) -> Child {
    run(&["interruption_signals_sleeper.mel", "--duration", duration])
}

/// Number of `sleep <duration>` processes running.
fn sleeping_children(duration: &str) -> usize {
    let expected = format!("sleep {duration}");
    match std::fs::read_dir("/proc") {
        Ok(entries) => entries
            .filter_map(|entry| std::fs::read(entry.ok()?.path().join("cmdline")).ok())
            .filter(|cmdline| {
                String::from_utf8_lossy(cmdline)
                    .trim_end_matches('\0')
                    .replace('\0', " ")
                    == expected
            })
            .count(),
        Err(_) => {
            let output = Command::new("ps").args(["-eo", "args"]).output().unwrap();
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|line| line.trim() == expected)
                .count()
        }
    }
}

fn run(args: &[&str]) -> Child {
    let child = tester::melodium()
        .arg("run")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_secs(3));
    child
}

fn signal(child: &Child, signal: i32) {
    tester::signal(child.id(), signal).unwrap();
}

fn second_signal_stops_right_away() {
    let duration = "61.1";
    let child = sleeper(duration);
    assert_eq!(sleeping_children(duration), 1);

    let start = Instant::now();
    signal(&child, tester::SIGINT);
    std::thread::sleep(Duration::from_millis(500));
    signal(&child, tester::SIGINT);
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(130));
    assert!(start.elapsed() < Duration::from_secs(5));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("interrupted by SIGINT, ending within 10 seconds"),
        "{}",
        stdout
    );
    assert!(
        stdout.contains("interrupted again by SIGINT, stopping"),
        "{}",
        stdout
    );
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(sleeping_children(duration), 0);
}

fn running_tracks_are_stopped_after_the_grace_period() {
    let duration = "61.2";
    let child = sleeper(duration);

    let start = Instant::now();
    signal(&child, tester::SIGTERM);
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(143));
    assert!(start.elapsed() >= Duration::from_secs(10));
    assert!(start.elapsed() < Duration::from_secs(15));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("still running at the end of the grace period, stopping"),
        "{}",
        stdout
    );
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(sleeping_children(duration), 0);
}

fn models_end_within_the_grace_period() {
    let child = run(&["interruption_signals_server.mel"]);

    let start = Instant::now();
    signal(&child, tester::SIGHUP);
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(129));
    assert!(start.elapsed() < Duration::from_secs(5));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("interrupted by SIGHUP"), "{}", stdout);
    assert!(!stdout.contains("stopping"), "{}", stdout);
}

pub fn run_cases() -> ! {
    let mut cases: Vec<(&str, fn())> = Vec::new();
    cases.push((
        "second_signal_stops_right_away",
        second_signal_stops_right_away,
    ));
    cases.push((
        "running_tracks_are_stopped_after_the_grace_period",
        running_tracks_are_stopped_after_the_grace_period,
    ));
    cases.push((
        "models_end_within_the_grace_period",
        models_end_within_the_grace_period,
    ));
    tester::cases(&cases)
}
