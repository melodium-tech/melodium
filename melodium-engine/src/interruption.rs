//! Interruption of the process by signals.
//!
//! On Unix, SIGINT, SIGTERM and SIGHUP ask the engine to end. Running tracks get a grace
//! period to finish, and a second signal or the end of the grace period stops them.
//!
//! On Windows, as a best effort, the console control events do the same: Ctrl+C and
//! Ctrl+Break, as well as the closing of the console, logoff and shutdown. Windows ends
//! the process a few seconds after these last three, so running tracks get a shorter
//! grace period then.

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
    /// Signal that interrupted the process, or console control event on Windows.
    pub fn signal(&self) -> i32 {
        match self {
            Interruption::Ended(signal) | Interruption::Stopped(signal) => *signal,
        }
    }

    /// Conventional exit code for a process interrupted by this signal: 128 plus the
    /// signal number on Unix, `STATUS_CONTROL_C_EXIT` on Windows.
    pub fn exit_code(&self) -> i32 {
        #[cfg(windows)]
        {
            console::STATUS_CONTROL_C_EXIT
        }
        #[cfg(not(windows))]
        {
            128 + self.signal()
        }
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
    #[cfg(windows)]
    {
        if let Some(name) = console::event_name(signal) {
            return name.to_string();
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
/// Gives how the engine stopped if the process was interrupted. Signals are handled on
/// Unix, and console control events on Windows; elsewhere the engine just lives until it
/// ends.
pub async fn live_until_interrupted(
    engine: &Arc<dyn Engine>,
    grace: Duration,
) -> Option<Interruption> {
    #[cfg(any(unix, windows))]
    {
        let mut interruptions = match Interruptions::listen() {
            Some(interruptions) => interruptions,
            None => {
                engine.live().await;
                return None;
            }
        };
        let interruption = live_interruptible(engine, grace, &mut interruptions).await;
        interruptions.close();
        if interruption.is_some() {
            *LAST_INTERRUPTION.lock().unwrap() = interruption;
        }
        interruption
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = grace;
        engine.live().await;
        None
    }
}

#[cfg(any(unix, windows))]
async fn live_interruptible(
    engine: &Arc<dyn Engine>,
    grace: Duration,
    interruptions: &mut Interruptions,
) -> Option<Interruption> {
    use futures::{pin_mut, select, FutureExt};
    use melodium_common::executive::Level;

    let live = engine.live().fuse();
    pin_mut!(live);

    let signal = {
        let next = interruptions.next().fuse();
        pin_mut!(next);
        select! {
            _ = live => return None,
            signal = next => signal,
        }
    };
    let signal = match signal {
        Some(signal) => signal,
        None => {
            live.await;
            return None;
        }
    };

    let grace = Interruptions::grace(signal, grace);
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
    let second = interruptions.next().fuse();
    pin_mut!(second);
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
        second = second => {
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
}

#[cfg(unix)]
struct Interruptions {
    signals: futures::stream::Fuse<signal_hook_async_std::Signals>,
    handle: signal_hook_async_std::Handle,
}

#[cfg(unix)]
impl Interruptions {
    fn listen() -> Option<Self> {
        use futures::StreamExt;
        use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};

        let signals = signal_hook_async_std::Signals::new([SIGINT, SIGTERM, SIGHUP]).ok()?;
        let handle = signals.handle();
        Some(Self {
            signals: signals.fuse(),
            handle,
        })
    }

    async fn next(&mut self) -> Option<i32> {
        use futures::StreamExt;
        self.signals.next().await
    }

    fn grace(_signal: i32, grace: Duration) -> Duration {
        grace
    }

    fn close(self) {
        self.handle.close();
    }
}

#[cfg(windows)]
struct Interruptions {
    events: async_std::channel::Receiver<i32>,
}

#[cfg(windows)]
impl Interruptions {
    fn listen() -> Option<Self> {
        console::listen().map(|events| Self { events })
    }

    async fn next(&mut self) -> Option<i32> {
        self.events.recv().await.ok()
    }

    fn grace(signal: i32, grace: Duration) -> Duration {
        if console::terminates(signal) {
            grace.min(console::TERMINATION_GRACE)
        } else {
            grace
        }
    }

    fn close(self) {
        console::close();
    }
}

#[cfg(windows)]
mod console {
    //! Console control events, received through `SetConsoleCtrlHandler`.

    use async_std::channel::{unbounded, Receiver, Sender};
    use core::time::Duration;
    use std::sync::Mutex;
    use windows_sys::core::BOOL;
    use windows_sys::Win32::System::Console::{
        SetConsoleCtrlHandler, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_C_EVENT, CTRL_LOGOFF_EVENT,
        CTRL_SHUTDOWN_EVENT,
    };

    /// Exit code of a process ended by Ctrl+C, as given by Windows.
    pub const STATUS_CONTROL_C_EXIT: i32 = 0xC000013Au32 as i32;

    /// Grace period once the console is closed, or on logoff or shutdown: Windows ends the
    /// process about five seconds after, logs and reports also need some of this time.
    pub const TERMINATION_GRACE: Duration = Duration::from_secs(2);

    static EVENTS: Mutex<Option<Sender<i32>>> = Mutex::new(None);

    /// Whether Windows ends the process once the handler of `event` returns.
    pub fn terminates(event: i32) -> bool {
        matches!(
            event as u32,
            CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT
        )
    }

    pub fn event_name(event: i32) -> Option<&'static str> {
        match event as u32 {
            CTRL_C_EVENT => Some("Ctrl+C"),
            CTRL_BREAK_EVENT => Some("Ctrl+Break"),
            CTRL_CLOSE_EVENT => Some("console close"),
            CTRL_LOGOFF_EVENT => Some("logoff"),
            CTRL_SHUTDOWN_EVENT => Some("shutdown"),
            _ => None,
        }
    }

    unsafe extern "system" fn handler(event: u32) -> BOOL {
        let events = EVENTS.lock().ok().and_then(|events| events.clone());
        match events {
            Some(events) => {
                let _ = events.try_send(event as i32);
                if terminates(event as i32) {
                    // Windows ends the process as soon as this returns, so the handler
                    // waits for the process to exit by itself, or for Windows to end it.
                    loop {
                        std::thread::park();
                    }
                }
                1
            }
            None => 0,
        }
    }

    pub fn listen() -> Option<Receiver<i32>> {
        let (sender, receiver) = unbounded();
        *EVENTS.lock().ok()? = Some(sender);
        if unsafe { SetConsoleCtrlHandler(Some(handler), 1) } == 0 {
            close();
            return None;
        }
        Some(receiver)
    }

    pub fn close() {
        unsafe { SetConsoleCtrlHandler(Some(handler), 0) };
        if let Ok(mut events) = EVENTS.lock() {
            *events = None;
        }
    }
}
