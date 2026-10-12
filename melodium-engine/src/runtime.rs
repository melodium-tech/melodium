//! Threads running the tracks of programs.
//!
//! Tracks, and continuous tasks of models, run on a pool of threads owned by the engine.
//! It has one thread per core, unless the `MELODIUM_THREADS` environment variable gives
//! another number. The pool starts with the first task, and lives as long as the process.

use async_executor::{Executor, Task};
use std::future::Future;
use std::sync::Once;

/// Environment variable giving the number of threads running tracks.
pub const THREADS_VARIABLE: &str = "MELODIUM_THREADS";

static EXECUTOR: Executor<'static> = Executor::new();
static START: Once = Once::new();

/// Runs `future` on the threads of the engine.
///
/// The task is cancelled when the returned [`Task`] is dropped, unless it is detached.
pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> Task<T> {
    START.call_once(start);
    EXECUTOR.spawn(future)
}

/// Number of threads running tracks, from `MELODIUM_THREADS` or the number of cores.
pub fn threads() -> usize {
    std::env::var(THREADS_VARIABLE)
        .ok()
        .and_then(|threads| threads.trim().parse::<usize>().ok())
        .filter(|threads| *threads > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|cores| cores.get())
                .unwrap_or(1)
        })
}

fn start() {
    for number in 0..threads() {
        std::thread::Builder::new()
            .name(format!("melodium-{number}"))
            .spawn(|| loop {
                // A panicking task gives its panic to whoever awaits it, the thread
                // goes on running others.
                let _ = std::panic::catch_unwind(|| {
                    smol::block_on(EXECUTOR.run(std::future::pending::<()>()))
                });
            })
            .expect("cannot start the threads running tracks");
    }
}
