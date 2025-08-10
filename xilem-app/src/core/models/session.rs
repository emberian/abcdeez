use chrono::{DateTime, Utc};
use abcdeez_core::{tasks::TaskResponse as CoreTaskResponse, Topology};
use serde::{Deserialize, Serialize};

// Session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub learner_id: String,
    pub topology_type: String,
    pub topology: Option<Topology>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String,
    pub summary: Option<serde_json::Value>,
    pub responses: Vec<CoreTaskResponse>,
}

impl Session {
    pub fn new(id: String, learner_id: String) -> Self {
        Self {
            id,
            learner_id,
            topology_type: "linear".to_string(),
            topology: None,
            start_time: chrono::Utc::now(),
            end_time: None,
            status: "active".to_string(),
            summary: None,
            responses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Completed,
    Paused,
    Cancelled,
}