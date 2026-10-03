//! Interruption of the process by signals.
//!
//! On Unix, SIGINT, SIGTERM and SIGHUP ask the engine to end. Running tracks get a grace
//! period to finish, and a second signal or the end of the grace period stops them.

use crate::engine::Engine;
use core::time::Duration;
use std::sync::{Arc, Mutex};

static LAST_INTERRUPTION: Mutex<Option<Interruption>> = Mutex::new(None);

/// Gives the last interruption handled by `live_until_interrupted` in this process.
pub fn last_interruption() -> Option<Interruption> {
    *LAST_INTERRUPTION.lock().unwrap()
}

/// How the engine stopped after an interruption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interruption {
    /// Running tracks finished within the grace period.
    Ended(i32),
    /// Running tracks were stopped, by a second signal or at the end of the grace period.
    Stopped(i32),
}

impl Interruption {
    /// Signal that interrupted the process.
    pub fn signal(&self) -> i32 {
        match self {
            Interruption::Ended(signal) | Interruption::Stopped(signal) => *signal,
        }
    }

    /// Conventional exit code for a process interrupted by this signal.
    pub fn exit_code(&self) -> i32 {
        128 + self.signal()
    }
}

/// Gives the name of an interrupting signal.
pub fn signal_name(signal: i32) -> String {
    #[cfg(unix)]
    {
        use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
        match signal {
            SIGINT => return "SIGINT".to_string(),
            SIGTERM => return "SIGTERM".to_string(),
            SIGHUP => return "SIGHUP".to_string(),
            _ => {}
        }
    }
    format!("signal {signal}")
}

/// Lives `engine` until it ends, or until the process is interrupted.
///
/// On a first interruption, the engine is asked to end, and running tracks get `grace` to
/// finish. A second interruption, or the end of `grace`, stops them right away: their
/// futures are dropped, which kills the child processes they own (see process-mel).
///
/// Gives how the engine stopped if the process was interrupted. Signals are only handled
/// on Unix, elsewhere the engine just lives until it ends.
pub async fn live_until_interrupted(
    engine: &Arc<dyn Engine>,
    grace: Duration,
) -> Option<Interruption> {
    #[cfg(unix)]
    {
        use futures::{pin_mut, select, FutureExt, StreamExt};
        use melodium_common::executive::Level;
        use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};

        let signals = match signal_hook_async_std::Signals::new([SIGINT, SIGTERM, SIGHUP]) {
            Ok(signals) => signals,
            Err(_) => {
                engine.live().await;
                return None;
            }
        };
        let handle = signals.handle();
        let mut signals = signals.fuse();
        let live = engine.live().fuse();
        pin_mut!(live);

        let signal = select! {
            _ = live => None,
            signal = signals.next() => signal,
        };

        let interruption = if let Some(signal) = signal {
            engine
                .log(
                    Level::Warning,
                    "melodium".to_string(),
                    format!(
                        "interrupted by {}, ending within {} seconds",
                        signal_name(signal),
                        grace.as_secs()
                    ),
                )
                .await;
            engine.end().await;

            let grace_end = async_std::task::sleep(grace).fuse();
            pin_mut!(grace_end);
            select! {
                _ = live => Some(Interruption::Ended(signal)),
                _ = grace_end => {
                    engine
                        .log(
                            Level::Error,
                            "melodium".to_string(),
                            "still running at the end of the grace period, stopping".to_string(),
                        )
                        .await;
                    Some(Interruption::Stopped(signal))
                },
                second = signals.next() => {
                    engine
                        .log(
                            Level::Error,
                            "melodium".to_string(),
                            format!(
                                "interrupted again by {}, stopping",
                                second.map(signal_name).unwrap_or_default()
                            ),
                        )
                        .await;
                    Some(Interruption::Stopped(signal))
                },
            }
        } else {
            None
        };

        handle.close();
        if interruption.is_some() {
            *LAST_INTERRUPTION.lock().unwrap() = interruption;
        }
        interruption
    }
    #[cfg(not(unix))]
    {
        let _ = grace;
        engine.live().await;
        None
    }
}
