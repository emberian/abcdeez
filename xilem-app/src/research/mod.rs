// Research-specific functionality module

pub mod audio;
pub mod federation;

// Re-export research controller items
use crate::api::models::SubmitResponseRequest;
use crate::core::models::{User, Session, PerformanceData, ProtocolVersion, ProtocolContent, ProtocolVersionDiff};
use crate::core::models::{AppleUserInfo};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

// Re-exports from the old research.rs file (originally at line 1-50)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSession {
    pub session_id: String,
    pub participant_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
    pub transcription: Option<String>,
    pub quality_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioRecordingState {
    Idle,
    Recording {
        start_time: DateTime<Utc>,
        duration: Duration,
    },
    Paused {
        total_duration: Duration,
    },
    Processing,
    Completed {
        file_path: String,
    },
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub sensor_type: SensorType,
    pub enabled: bool,
    pub sample_rate: u32,
    pub status: SensorStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SensorType {
    EEG,
    GSR,
    EyeTracker,
    HeartRate,
    Microphone,
    Camera,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SensorStatus {
    Connected,
    Disconnected,
    Error(String),
    Calibrating,
}

// Research controller and experiment types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExperimentType {
    MemoryStudy,
    LearningEfficiency,
    CognitiveLoad,
    UserInterface,
    LearningCurve,
    RetentionTest,
}

pub struct ResearchController {
    pub active_session: Option<String>,
}

impl ResearchController {
    pub fn new() -> Self {
        Self {
            active_session: None,
        }
    }

    pub fn record_data_point(&mut self, _data_point: DataPoint) -> Result<(), String> {
        // Placeholder implementation
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub trial_number: u32,
    pub stimulus: String,
    pub response: String,
    pub correct: bool,
    pub response_time_ms: u128,
    pub confidence: Option<f64>,
    pub eye_tracking: Option<serde_json::Value>,
    pub physiological: Option<serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
}