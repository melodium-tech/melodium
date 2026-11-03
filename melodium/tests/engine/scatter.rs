//! `std/flow/scatter` makes one track per value of a stream, at most `parallel` at once, and
//! gathers how each ended.

use super::common;
use melodium_engine::debug::{Event, EventKind};
use std::collections::{HashMap, HashSet};

const VALUES: &str = include_str!("scripts/scatter_values.mel");
const NESTED: &str = include_str!("scripts/scatter_nested.mel");
const OUTSIDE: &str = include_str!("scripts/scatter_outside.mel");
const GATHERING: &str = include_str!("scripts/scatter_gathering.mel");

/// Messages of the logs labelled `label`.
fn messages(logs: &[melodium_common::executive::Log], label: &str) -> Vec<String> {
    logs.iter()
        .filter(|log| log.label == label)
        .map(|log| log.message.clone())
        .collect()
}

/// Number of child tracks of each parent, and most child tracks running at once.
fn children(events: &[Event]) -> (HashMap<usize, usize>, usize) {
    let mut count = HashMap::new();
    let mut children = HashSet::new();
    let mut running = 0;
    let mut most = 0;
    for event in events {
        match &event.kind {
            EventKind::TrackAdded { info, .. } => {
                if let Some(parent) = info.parent_id {
                    *count.entry(parent).or_insert(0) += 1;
                    children.insert(info.id);
                    running += 1;
                    most = most.max(running);
                }
            }
            EventKind::TrackFinished { info } if children.contains(&info.id) => running -= 1,
            _ => {}
        }
    }
    (count, most)
}

#[test]
fn values_are_scattered_and_gathered() {
    let (logs, events) = common::run(VALUES, HashMap::new());

    let mut items = messages(&logs, "item");
    items.sort();
    assert_eq!(
        items,
        ["fail-1", "ok-0", "ok-2", "ok-4", "ok-5", "silent-3"],
        "{logs:?}"
    );
    assert_eq!(messages(&logs, "shared"), vec!["shared"; 6]);

    // A failed track and one ending without telling fail the scatter, naming them.
    let errors = messages(&logs, "scatter");
    assert_eq!(errors.len(), 2, "{errors:?}");
    assert!(errors.contains(&"scattered track 1 failed: asked to fail".to_string()));
    assert!(errors.contains(&"scattered track 3 ended without telling how".to_string()));
    assert_eq!(common::sent(&events, "scatterFailed", "check").len(), 1);
    assert!(common::sent(&events, "scatterCompleted", "check").is_empty());
    assert_eq!(common::sent(&events, "scatterFinished", "check").len(), 1);

    // Six tracks of one parent, never more than two at once.
    let (count, most) = children(&events);
    assert_eq!(count.values().copied().collect::<Vec<_>>(), [6]);
    assert!(most <= 2, "{most}");

    // Values of another type than `scattered` takes make no track.
    let mismatched = messages(&logs, "mismatched");
    assert!(
        mismatched
            .iter()
            .any(|message| message.contains("no `scattered<string, void>`")),
        "{mismatched:?}"
    );
}

#[test]
fn values_of_tracks_are_gathered() {
    let (logs, events) = common::run(GATHERING, HashMap::new());

    // Track 2 gathers nothing, and still completes.
    let mut gathered = messages(&logs, "gathered");
    gathered.sort();
    assert_eq!(gathered, ["10", "20", "40"], "{logs:?}");
    assert!(messages(&logs, "scatter").is_empty(), "{logs:?}");
    assert_eq!(common::sent(&events, "scatterCompleted", "check").len(), 1);
    assert!(common::sent(&events, "scatterFailed", "check").is_empty());

    // Values of another type than the scatter gathers fail it, and are not given.
    assert_eq!(
        messages(&logs, "mismatched"),
        ["scattered track 0 gathered a value of type string, not u64"]
    );
    assert!(messages(&logs, "mismatched gathered").is_empty());
}

#[test]
fn scatters_nest() {
    let (logs, events) = common::run(NESTED, HashMap::new());

    let mut inner = messages(&logs, "inner");
    inner.sort();
    assert_eq!(inner, ["x", "x", "y", "y"], "{logs:?}");
    let mut outer = messages(&logs, "outer");
    outer.sort();
    assert_eq!(outer, ["a", "a", "b", "b"]);
    assert_eq!(common::sent(&events, "scattered", "check").len(), 1);

    // Two outer tracks of the main one, each with two inner tracks.
    let (count, _) = children(&events);
    let mut counts = count.values().copied().collect::<Vec<_>>();
    counts.sort();
    assert_eq!(counts, [2, 2, 2]);
}

#[test]
fn gathering_needs_a_scattered_track() {
    let errors = common::genesis_errors(OUTSIDE);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("Scattered") && error.contains("not available")),
        "{errors:?}"
    );
}
