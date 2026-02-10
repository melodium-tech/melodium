use chrono::{DateTime, Utc};
use melodium_common::executive::Level;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Log {
    pub timestamp: DateTime<Utc>,
    pub level: Level,
    pub label: String,
    pub message: String,
    pub track_id: Option<usize>,
    pub run_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
}

impl Log {
    pub(crate) fn new_now(
        level: Level,
        label: String,
        message: String,
        track_id: Option<usize>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            label,
            message,
            track_id,
            run_id: Some(*crate::execution_run_id()),
            group_id: Some(*crate::execution_group_id()),
        }
    }
}
