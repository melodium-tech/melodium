//! Regression test: `compose::compose`'s initial `podman`/`docker` detection calls
//! (checking `<executor> version` and looking up the remote socket) used to have no
//! timeout at all. If the container engine's daemon is unresponsive (stuck service,
//! a pull already hung from another process, etc.), those very first subprocess
//! calls - made before anything else in `compose()` runs - would block forever, and
//! nothing in Mélodium would ever notice or give up. This reproduces that with a
//! fake `podman` script that just hangs, put first on `PATH`, and asserts
//! `compose()` still returns (with an error) within a bounded time instead of
//! hanging, using `MELODIUM_COMPOSE_DETECTION_TIMEOUT_SECS` to keep the test fast.

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;
use work_mel::api::{ModeRequest, Request};
use work_mel::compose::compose;

#[async_std::test]
async fn detection_does_not_hang_when_executor_is_unresponsive() {
    std::env::set_var("MELODIUM_COMPOSE_DETECTION_TIMEOUT_SECS", "2");

    let fake_bin_dir =
        std::env::temp_dir().join(format!("melodium-compose-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&fake_bin_dir).expect("failed to create fake bin dir");
    let fake_podman = fake_bin_dir.join("podman");
    {
        let mut file = fs::File::create(&fake_podman).expect("failed to create fake podman");
        // Hangs forever instead of responding, simulating an unresponsive daemon.
        file.write_all(b"#!/bin/sh\nsleep 3600\n")
            .expect("failed to write fake podman");
    }
    fs::set_permissions(&fake_podman, fs::Permissions::from_mode(0o755))
        .expect("failed to chmod fake podman");

    let original_path = std::env::var("PATH").unwrap_or_default();
    std::env::set_var(
        "PATH",
        format!("{}:{}", fake_bin_dir.display(), original_path),
    );

    let request = Request {
        config: None,
        id: None,
        organization_id: None,
        edition: None,
        version: "0.10.2".to_string(),
        mode: ModeRequest::Direct,
        max_duration: Some(60),
        memory: Some(100),
        cpu: Some(100),
        storage: Some(100),
        arch: None,
        volumes: vec![],
        containers: vec![],
        service_containers: vec![],
        tags: vec![],
        group_id: None,
        parent_id: None,
        local_exec: false,
    };

    let result = async_std::future::timeout(Duration::from_secs(10), compose(request)).await;

    std::env::set_var("PATH", original_path);
    let _ = fs::remove_dir_all(&fake_bin_dir);

    match result {
        Ok(Err(_errors)) => {
            // Expected: compose() itself gave up on the unresponsive executor
            // within its own (shortened) detection timeout, returning an error
            // instead of hanging.
        }
        Ok(Ok(_)) => {
            panic!("compose() unexpectedly succeeded against a fake, non-functional podman");
        }
        Err(_) => {
            panic!(
                "compose() did not return within 10s (regression: no timeout on the initial \
                 podman/docker detection calls leaves it hanging forever when the container \
                 engine is unresponsive)"
            );
        }
    }
}
