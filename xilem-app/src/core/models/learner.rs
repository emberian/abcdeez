use chrono::{DateTime, Utc};
use abcdeez_core::{LearnerMetrics as CoreLearnerMetrics, LearnerModel as CoreLearnerModel};
use serde::{Deserialize, Serialize};

// Wrapper for core LearnerModel with UI-specific fields
#[derive(Debug, Clone)]
pub struct Learner {
    pub id: String,
    pub user_id: Option<String>,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub core_model: CoreLearnerModel, // The actual learner model from the library
    pub metadata: Option<serde_json::Value>,
}

// Serialization helpers for Learner (since it contains non-serializable CoreLearnerModel)
impl Serialize for Learner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Learner", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.serialize_field("display_name", &self.display_name)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Learner {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // For now, we can't deserialize a Learner with a CoreLearnerModel
        // This would need to be handled by reconstructing from stored data
        unimplemented!("Learner deserialization requires topology information")
    }
}

// Performance metrics with UI-specific fields
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub core_metrics: Option<CoreLearnerMetrics>,
    pub total_responses: usize,
    pub correct_responses: usize,
    pub average_response_time_ms: f64,
    pub accuracy_rate: f64,
    pub recent_accuracy: f64, // Last 10 responses
    pub improvement_rate: f64,
    pub streak_count: usize,
    pub best_streak: usize,
}

impl PerformanceMetrics {
    pub fn update(&mut self, correct: bool, response_time_ms: i32) {
        self.total_responses += 1;
        if correct {
            self.correct_responses += 1;
            self.streak_count += 1;
            if self.streak_count > self.best_streak {
                self.best_streak = self.streak_count;
            }
        } else {
            self.streak_count = 0;
        }

        // Update average response time
        let old_avg = self.average_response_time_ms;
        self.average_response_time_ms = (old_avg * (self.total_responses - 1) as f64
            + response_time_ms as f64)
            / self.total_responses as f64;

        // Update accuracy rate
        self.accuracy_rate = self.correct_responses as f64 / self.total_responses as f64;
    }

    pub fn update_from_core_metrics(&mut self, metrics: &CoreLearnerMetrics) {
        self.core_metrics = Some(metrics.clone());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceData {
    pub overall_accuracy: f64,
    pub average_response_time: f64,
    pub total_sessions: u32,
    pub total_tasks: u32,
    pub learning_metrics: serde_json::Value,
}