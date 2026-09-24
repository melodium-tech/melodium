//! Local, isolated reproduction/regression-check for the report_logs memory growth fixed
//! in this ticket, with no dependency on GitLab tokens, the real melodium API, or any real
//! worker dispatch.
//!
//! Feeds `work_mel::reporting::report_logs` a firehose of `Log` entries (simulating
//! `cargo build/test --verbose` volume across a few parallel CI legs) while pointing its
//! S3 upload target at an address nothing listens on, so every `send_logs_to_s3` attempt
//! fails at the transport level — exactly the condition under which memory used to grow
//! without bound (RSS 3.4 MB -> 8.5 GB in under a minute, upstream channel backlog to ~20M
//! pending `Log` entries, before this fix). Watch `[repro][rss]`: it should now plateau
//! (the reporting channel is bounded, and `report_logs` itself drops rather than queuing
//! indefinitely once too many batches are stuck failing to upload).
//!
//! Run with: cargo run --release -p work-mel --example reporting_leak_repro --features real,net-mel/real,std-mel/real,fs-mel/real,process-mel/real,async-std/tokio1

use async_std::channel::Sender;
use async_std::task;
use melodium_core::common::executive::{Level, Log};
use std::collections::HashMap;
use std::time::Duration;
use work_mel::reporting::{report_logs, PushSpecs};

fn rss_kb() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix("VmRSS:")
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|value| value.parse::<u64>().ok())
    })
}

async fn producer(tx: Sender<Log>, id: u32) {
    let mut i: u64 = 0;
    loop {
        let log = Log {
            timestamp: chrono::Utc::now(),
            level: Level::Debug,
            label: format!("leg-{id}"),
            message: format!(
                "simulated verbose build/test output line {i} from leg {id}, padding: {}",
                "x".repeat(200)
            ),
            track_id: None,
            run_id: None,
            group_id: None,
        };
        if tx.send(log).await.is_err() {
            break;
        }
        i += 1;
        // Yield occasionally so this stays a fast-but-cooperative producer rather than
        // starving the executor; still far faster than a 5s/5000-item batched uploader
        // could ever drain once uploads are failing.
        if i % 5000 == 0 {
            task::sleep(Duration::from_millis(1)).await;
        }
    }
}

fn main() {
    task::block_on(async {
        // Matches the bound `melodium/src/lib.rs`'s `api_report` now puts on its
        // logs/debug reporting channels, so this repro exercises the same end-to-end
        // shape as production instead of an artificially-unbounded upstream.
        let (tx, rx) = async_std::channel::bounded::<Log>(200_000);

        // Four parallel producers, mirroring .cadence-ci's four concurrent legs
        // (unit tests + 3 target architectures).
        for id in 0..4 {
            task::spawn(producer(tx.clone(), id));
        }

        let watchdog_tx = tx.clone();
        task::spawn(async move {
            loop {
                eprintln!(
                    "[repro][rss] rss_kb={:?} channel_backlog={}",
                    rss_kb(),
                    watchdog_tx.len()
                );
                task::sleep(Duration::from_secs(2)).await;
            }
        });
        drop(tx);

        eprintln!("[repro] starting report_logs against an unreachable S3 target; ^C to stop");

        let specs = PushSpecs::PresignedPostS3 {
            uri: "http://127.0.0.1:1/upload".to_string(),
            fields: HashMap::new(),
            path: "repro".to_string(),
        };

        report_logs(specs, rx).await;
    });
}
