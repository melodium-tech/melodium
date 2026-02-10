use std::sync::OnceLock;
use uuid::Uuid;

static EXECUTION_RUN_ID: OnceLock<Uuid> = OnceLock::new();
static EXECUTION_GROUP_ID: OnceLock<Uuid> = OnceLock::new();

pub fn set_execution_run_id(uuid: Uuid) {
    EXECUTION_RUN_ID.set(uuid);
}

pub fn set_execution_group_id(uuid: Uuid) {
    EXECUTION_GROUP_ID.set(uuid);
}

pub fn execution_run_id() -> &'static Uuid {
    EXECUTION_RUN_ID.get_or_init(|| {
        std::env::var("MELODIUM_RUN_ID")
            .map(|var| Uuid::parse_str(&var).ok())
            .ok()
            .flatten()
            .unwrap_or_else(|| Uuid::new_v4())
    })
}

pub fn execution_group_id() -> &'static Uuid {
    EXECUTION_GROUP_ID.get_or_init(|| {
        std::env::var("MELODIUM_GROUP_ID")
            .map(|var| Uuid::parse_str(&var).ok())
            .ok()
            .flatten()
            .unwrap_or_else(|| Uuid::new_v4())
    })
}
