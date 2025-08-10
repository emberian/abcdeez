use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Learner, Session, PerformanceMetrics};

// Export data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub learner: Learner,
    pub sessions: Vec<Session>,
    pub metrics: PerformanceMetrics,
    pub export_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo_seed: Option<u64>,
}

// Export format enum for data export functionality
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Replay,
}

impl Default for ExportFormat {
    fn default() -> Self {
        ExportFormat::Json
    }
}