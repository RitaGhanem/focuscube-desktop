pub mod platform;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkEvent {
    AppFocused { name: String, pid: u32 },
    FileSaved { path: PathBuf },
}

impl WorkEvent {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[async_trait::async_trait]
pub trait EventSource: Send + Sync {
    async fn next_event(&mut self) -> anyhow::Result<WorkEvent>;
}