//! An engine's world must be released once the engine is dropped, including the
//! models it built (they used to hold a strong reference back to the world).

use melodium::{load_raw, LoadingConfig};
use melodium_common::executive::Level;
use melodium_engine::debug::DebugLevel;
use std::{collections::HashMap, sync::Arc};

const SCRIPT: &str = include_str!("scripts/world_release.mel");

#[test]
fn world_is_released_once_engine_is_dropped() {
    let (pkg, collection) = load_raw(
        Arc::new(SCRIPT.as_bytes().to_vec()),
        "main",
        LoadingConfig {
            core_packages: Vec::new(),
            search_locations: Vec::new(),
            raw_elements: Vec::new(),
        },
    )
    .into_result()
    .expect("script loads");
    let entrypoint = pkg.entrypoints().get("main").cloned().unwrap();

    let engine = melodium_engine::new_engine(collection, Level::Info, DebugLevel::None);
    assert!(engine.genesis(&entrypoint, HashMap::new()).is_success());
    async_std::task::block_on(async {
        engine.live().await;
        engine.end().await;
    });

    let world = Arc::downgrade(&engine);
    drop(engine);
    assert!(
        world.upgrade().is_none(),
        "world still alive after its engine was dropped"
    );
}
