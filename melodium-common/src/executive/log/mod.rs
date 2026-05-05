use chrono::{DateTime, Utc};
use core::fmt::{Debug, Display};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Error => write!(f, "error"),
            Level::Warning => write!(f, "warning"),
            Level::Info => write!(f, "info"),
            Level::Debug => write!(f, "debug"),
            Level::Trace => write!(f, "trace"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Log {
    pub timestamp: DateTime<Utc>,
    pub level: Level,
    pub label: String,
    pub message: String,
    pub track_id: Option<usize>,
    pub run_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
}
