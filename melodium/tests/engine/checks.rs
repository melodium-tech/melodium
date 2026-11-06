//! Genesis checks the tracks of every source: the contexts they need, and that connections
//! never loop back, each build being checked once whatever the number of paths to it.

use super::common;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const LOOP: &str = include_str!("scripts/check_loop.mel");
const CONTEXT: &str = include_str!("scripts/check_context.mel");

#[test]
fn connection_loops_are_refused() {
    let errors = common::genesis_errors(LOOP);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("infinite connection loop")),
        "{errors:?}"
    );
}

#[test]
fn missing_contexts_are_refused() {
    let errors = common::genesis_errors(CONTEXT);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("Context") && error.contains("not available")),
        "{errors:?}"
    );
}

/// Program of `levels` diamonds one after another, each output going to two treatments joined
/// again: there are 2 to the power of `levels` paths from `startup`.
fn diamonds(levels: usize) -> String {
    let mut script = String::from(
        "#!/usr/bin/env melodium\n#! name = check_diamonds\n#! version = 0.1.0\n#! require = std:0.11.0\n\n\
         use std/engine/util::startup\nuse std/flow::passBlock\nuse std/flow::one\n\n\
         treatment main()\n{\n    startup()\n",
    );
    let mut previous = "startup.trigger".to_string();
    for level in 0..levels {
        script.push_str(&format!(
            "    left{level}: passBlock<void>(cond=true)\n    right{level}: passBlock<void>(cond=true)\n    join{level}: one<void>()\n\
             \x20   {previous} -> left{level}.block,passed -> join{level}.a\n\
             \x20   {previous} -> right{level}.block,passed -> join{level}.b\n"
        ));
        previous = format!("join{level}.value");
    }
    script.push_str("}\n");
    script
}

#[test]
fn checks_take_time_proportional_to_programs() {
    let start = Instant::now();
    let (logs, _) = common::run(&diamonds(40), HashMap::new());
    assert!(logs.is_empty(), "{logs:?}");
    // Following every path, as checks once did, would never end.
    assert!(
        start.elapsed() < Duration::from_secs(20),
        "{:?}",
        start.elapsed()
    );
}
