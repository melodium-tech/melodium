//! SIGINT, SIGTERM and SIGHUP end a run: running tracks get a grace period, and are
//! stopped by a second signal or at its end, killing the child processes they own.

#[cfg(unix)]
mod cases;

fn main() {
    #[cfg(unix)]
    cases::run_cases();
    #[cfg(not(unix))]
    tester::skip("sends SIGINT, SIGTERM and SIGHUP, Unix signals");
}
