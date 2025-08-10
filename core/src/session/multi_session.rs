use crate::config::LearnerConfig;
use crate::experimental::design::{ExperimentalDesign, ParticipantAssignment};
use crate::statistics::power_analysis::StatisticalTestType;
use crate::core::topology::Topology;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Multi-Session Experiment Management System
/// Handles longitudinal studies, scheduling, and cross-session analysis

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSessionExperiment {
    pub id: String,
    pub name: String,
    pub description: String,
    pub design: ExperimentalDesign,
    pub sessions: Vec<SessionPlan>,
    pub participant_assignments: HashMap<String, ParticipantAssignment>,
    pub scheduling_rules: SchedulingRules,
    pub data_retention_policy: DataRetentionPolicy,
    pub analysis_plan: AnalysisPlan,
    pub status: ExperimentStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPlan {
    pub session_id: String,
    pub name: String,
    pub description: String,
    pub order: usize,
    pub duration_minutes: Option<u32>,
    pub minimum_interval_hours: Option<u32>, // Minimum time since previous session
    pub maximum_interval_hours: Option<u32>, // Maximum time before session expires
    pub config: LearnerConfig,
    pub topology: Topology,
    pub tasks: Vec<SessionTask>,
    pub pre_session_requirements: Vec<Requirement>,
    pub post_session_procedures: Vec<Procedure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTask {
    pub task_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub min_trials: usize,
    pub max_trials: Option<usize>,
    pub termination_criteria: Vec<TerminationCriterion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminationCriterion {
    MaxTrials { count: usize },
    MinAccuracy { threshold: f64, window: usize },
    MaxTime { minutes: u32 },
    ConsecutiveCorrect { count: usize },
    AdaptiveStopping { confidence: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingRules {
    pub allow_overlap: bool,
    pub max_concurrent_participants: Option<usize>,
    pub preferred_time_windows: Vec<TimeWindow>,
    pub blocked_dates: Vec<chrono::NaiveDate>,
    pub reminder_settings: ReminderSettings,
    pub rescheduling_policy: ReschedulingPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start_time: chrono::NaiveTime,
    pub end_time: chrono::NaiveTime,
    pub days_of_week: Vec<chrono::Weekday>,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderSettings {
    pub enabled: bool,
    pub advance_hours: Vec<u32>, // e.g., [24, 1] for 24h and 1h reminders
    pub method: Vec<ReminderMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReminderMethod {
    Email { address: String },
    SMS { phone: String },
    Push { device_token: String },
    InApp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReschedulingPolicy {
    pub max_reschedules_per_session: usize,
    pub automatic_reschedule_window_hours: u32,
    pub penalty_for_no_show: Option<ReschedulingPenalty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReschedulingPenalty {
    DelayNextSession { hours: u32 },
    RequireConfirmation,
    ExcludeFromStudy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionPolicy {
    pub retain_raw_data_days: u32,
    pub retain_aggregated_data_days: u32,
    pub anonymize_after_days: u32,
    pub export_before_deletion: bool,
    pub gdpr_compliance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPlan {
    pub primary_outcomes: Vec<OutcomeMeasure>,
    pub secondary_outcomes: Vec<OutcomeMeasure>,
    pub planned_comparisons: Vec<PlannedComparison>,
    pub interim_analyses: Vec<InterimAnalysis>,
    pub statistical_tests: Vec<StatisticalTestSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeMeasure {
    pub name: String,
    pub description: String,
    pub measure_type: MeasureType,
    pub calculation: String, // Formula or procedure description
    pub higher_is_better: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeasureType {
    AccuracyScore,
    ResponseTime,
    LearningRate,
    RetentionScore,
    TransferScore,
    EfficiencyIndex,
    Custom { formula: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedComparison {
    pub name: String,
    pub description: String,
    pub conditions: Vec<String>,
    pub outcome_measure: String,
    pub statistical_test: String,
    pub multiple_comparison_correction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterimAnalysis {
    pub scheduled_after_session: usize,
    pub outcomes_to_check: Vec<String>,
    pub stopping_rules: Vec<StoppingRule>,
    pub required_power: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoppingRule {
    Efficacy { alpha_spent: f64 },
    Futility { conditional_power_threshold: f64 },
    Safety { adverse_event_rate: f64 },
    Administrative { max_duration_days: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTestSpec {
    pub test_name: String,
    pub variables: Vec<String>,
    pub test_type: StatisticalTestType,
    pub alpha: f64,
    pub correction_method: Option<String>,
    pub assumptions_to_check: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Planning,
    ReadyToStart,
    Active,
    Paused,
    Completed,
    Terminated,
    AnalysisInProgress,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Requirement {
    MinimumRestTime { hours: u32 },
    CompletedPreviousSession,
    ConsentReaffirmation,
    EquipmentCheck { items: Vec<String> },
    EnvironmentalConditions { description: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Procedure {
    DataBackup,
    SessionSummary,
    ParticipantFeedback,
    EquipmentShutdown,
    DataQualityCheck,
    ScheduleNextSession,
}

/// Participant session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantProgress {
    pub participant_id: String,
    pub experiment_id: String,
    pub completed_sessions: Vec<CompletedSession>,
    pub scheduled_sessions: Vec<ScheduledSession>,
    pub status: ParticipantStatus,
    pub notes: Vec<ProgressNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedSession {
    pub session_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub performance_summary: HashMap<String, f64>,
    pub data_quality_score: f64,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledSession {
    pub session_id: String,
    pub scheduled_at: chrono::DateTime<chrono::Utc>,
    pub reminder_sent: Vec<chrono::DateTime<chrono::Utc>>,
    pub reschedule_count: usize,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantStatus {
    Active,
    Completed,
    Withdrawn,
    Excluded,
    NoShow,
    OnHold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressNote {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub author: String,
    pub content: String,
    pub category: NoteCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoteCategory {
    Performance,
    Technical,
    Behavioral,
    Administrative,
    DataQuality,
}

/// Main experiment management system
pub struct MultiSessionManager {
    experiments: HashMap<String, MultiSessionExperiment>,
    participant_progress: HashMap<String, ParticipantProgress>,
    data_directory: PathBuf,
    scheduler: ExperimentScheduler,
}

impl MultiSessionManager {
    pub fn new(data_directory: PathBuf) -> Self {
        Self {
            experiments: HashMap::new(),
            participant_progress: HashMap::new(),
            data_directory,
            scheduler: ExperimentScheduler::new(),
        }
    }

    /// Create a new multi-session experiment
    pub fn create_experiment(
        &mut self,
        name: String,
        description: String,
        design: ExperimentalDesign,
        sessions: Vec<SessionPlan>,
        scheduling_rules: SchedulingRules,
    ) -> Result<String, String> {
        let experiment_id = uuid::Uuid::new_v4().to_string();

        // Validate experiment design
        self.validate_experiment_design(&design, &sessions)?;

        let experiment = MultiSessionExperiment {
            id: experiment_id.clone(),
            name,
            description,
            design,
            sessions,
            participant_assignments: HashMap::new(),
            scheduling_rules,
            data_retention_policy: DataRetentionPolicy::default(),
            analysis_plan: AnalysisPlan::default(),
            status: ExperimentStatus::Planning,
            created_at: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
        };

        self.experiments.insert(experiment_id.clone(), experiment);
        Ok(experiment_id)
    }

    /// Register a participant for an experiment
    pub fn register_participant(
        &mut self,
        experiment_id: &str,
        participant_id: String,
        participant_characteristics: Option<HashMap<String, String>>,
    ) -> Result<ParticipantAssignment, String> {
        let experiment = self
            .experiments
            .get_mut(experiment_id)
            .ok_or("Experiment not found")?;

        // Use experimental design to assign participant
        use crate::experimental::design::ExperimentalDesigner;
        let mut designer = ExperimentalDesigner::new(None);
        let assignment = designer.assign_participant(
            participant_id.clone(),
            &experiment.design,
            participant_characteristics,
        )?;

        // Store assignment
        experiment
            .participant_assignments
            .insert(participant_id.clone(), assignment.clone());

        // Initialize participant progress tracking
        let progress = ParticipantProgress {
            participant_id: participant_id.clone(),
            experiment_id: experiment_id.to_string(),
            completed_sessions: Vec::new(),
            scheduled_sessions: Vec::new(),
            status: ParticipantStatus::Active,
            notes: Vec::new(),
        };

        self.participant_progress
            .insert(participant_id.clone(), progress);

        // Schedule first session
        self.schedule_next_session(&participant_id, experiment_id)?;

        Ok(assignment)
    }

    /// Schedule the next session for a participant
    pub fn schedule_next_session(
        &mut self,
        participant_id: &str,
        experiment_id: &str,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, String> {
        // Get experiment info first
        let (sessions_len, session_plan, scheduling_rules) = {
            let experiment = self
                .experiments
                .get(experiment_id)
                .ok_or("Experiment not found")?;
            let progress = self
                .participant_progress
                .get(participant_id)
                .ok_or("Participant not found")?;

            let next_session_index = progress.completed_sessions.len();
            if next_session_index >= experiment.sessions.len() {
                return Ok(None);
            }

            (
                experiment.sessions.len(),
                experiment.sessions[next_session_index].clone(),
                experiment.scheduling_rules.clone(),
            )
        };

        // Check requirements first (immutable borrow)
        let requirements_met = {
            let progress = self
                .participant_progress
                .get(participant_id)
                .ok_or("Participant not found")?;
            self.check_session_requirements(progress, &session_plan)?
        };

        if !requirements_met {
            return Err("Session requirements not met".to_string());
        }

        // Now get mutable progress
        let progress = self
            .participant_progress
            .get_mut(participant_id)
            .ok_or("Participant not found")?;

        let next_session_index = progress.completed_sessions.len();
        if next_session_index >= sessions_len {
            progress.status = ParticipantStatus::Completed;
            return Ok(None);
        }

        // Find available time slot
        let scheduled_time =
            self.scheduler
                .find_available_slot(&scheduling_rules, &session_plan, progress)?;

        // Create scheduled session
        let scheduled_session = ScheduledSession {
            session_id: session_plan.session_id.clone(),
            scheduled_at: scheduled_time,
            reminder_sent: Vec::new(),
            reschedule_count: 0,
            confirmed: false,
        };

        progress.scheduled_sessions.push(scheduled_session);

        // Schedule reminders
        self.scheduler.schedule_reminders(
            participant_id,
            &scheduled_time,
            &scheduling_rules.reminder_settings,
        )?;

        Ok(Some(scheduled_time))
    }

    /// Complete a session and update participant progress
    pub fn complete_session(
        &mut self,
        participant_id: &str,
        experiment_id: &str,
        session_id: &str,
        performance_data: HashMap<String, f64>,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), String> {
        // Calculate data quality score first (immutable borrow)
        let data_quality_score = self.calculate_data_quality_score(&performance_data);

        let progress = self
            .participant_progress
            .get_mut(participant_id)
            .ok_or("Participant not found")?;

        // Remove from scheduled sessions
        progress
            .scheduled_sessions
            .retain(|s| s.session_id != session_id);

        // Add to completed sessions
        let completed_session = CompletedSession {
            session_id: session_id.to_string(),
            started_at: start_time,
            completed_at: end_time,
            performance_summary: performance_data,
            data_quality_score,
            notes: String::new(),
        };

        let completed_sessions_count = progress.completed_sessions.len() + 1;
        progress.completed_sessions.push(completed_session);

        // Update experiment last modified
        if let Some(experiment) = self.experiments.get_mut(experiment_id) {
            experiment.last_modified = chrono::Utc::now();
        }

        // Check if interim analysis is needed
        self.check_interim_analysis(experiment_id, completed_sessions_count)?;

        // Schedule next session if available
        self.schedule_next_session(participant_id, experiment_id)?;

        Ok(())
    }

    /// Perform longitudinal analysis across sessions
    pub fn analyze_longitudinal_data(
        &self,
        experiment_id: &str,
        outcome_measure: &str,
    ) -> Result<LongitudinalAnalysis, String> {
        let experiment = self
            .experiments
            .get(experiment_id)
            .ok_or("Experiment not found")?;

        let mut participant_trajectories = HashMap::new();
        let mut session_means = Vec::new();

        // Collect data for each participant across sessions
        for (participant_id, assignment) in &experiment.participant_assignments {
            if let Some(progress) = self.participant_progress.get(participant_id) {
                let mut trajectory = Vec::new();

                for completed_session in &progress.completed_sessions {
                    if let Some(&value) = completed_session.performance_summary.get(outcome_measure)
                    {
                        trajectory.push(SessionDataPoint {
                            session_index: trajectory.len(),
                            value,
                            timestamp: completed_session.completed_at,
                            data_quality: completed_session.data_quality_score,
                        });
                    }
                }

                if !trajectory.is_empty() {
                    participant_trajectories.insert(participant_id.clone(), trajectory);
                }
            }
        }

        // Calculate session-wise statistics
        let max_sessions = participant_trajectories
            .values()
            .map(|t| t.len())
            .max()
            .unwrap_or(0);

        for session_idx in 0..max_sessions {
            let session_values: Vec<f64> = participant_trajectories
                .values()
                .filter_map(|trajectory| trajectory.get(session_idx).map(|dp| dp.value))
                .collect();

            if !session_values.is_empty() {
                let mean = session_values.iter().sum::<f64>() / session_values.len() as f64;
                let variance = session_values
                    .iter()
                    .map(|x| (x - mean).powi(2))
                    .sum::<f64>()
                    / (session_values.len() - 1) as f64;

                session_means.push(SessionStatistics {
                    session_index: session_idx,
                    n: session_values.len(),
                    mean,
                    std_dev: variance.sqrt(),
                    min: session_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                    max: session_values
                        .iter()
                        .fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                });
            }
        }

        // Fit learning curves and detect patterns
        let learning_curve = self.fit_learning_curve(&session_means);
        let change_points = self.detect_change_points(&session_means);

        Ok(LongitudinalAnalysis {
            outcome_measure: outcome_measure.to_string(),
            participant_trajectories,
            session_statistics: session_means,
            learning_curve,
            change_points,
            generated_at: chrono::Utc::now(),
        })
    }

    // Helper methods
    fn validate_experiment_design(
        &self,
        design: &ExperimentalDesign,
        sessions: &[SessionPlan],
    ) -> Result<(), String> {
        if sessions.is_empty() {
            return Err("At least one session is required".to_string());
        }

        // Check session ordering
        for (i, session) in sessions.iter().enumerate() {
            if session.order != i {
                return Err(format!(
                    "Session {} has incorrect order",
                    session.session_id
                ));
            }
        }

        // Validate session IDs are unique
        let mut session_ids = HashSet::new();
        for session in sessions {
            if !session_ids.insert(&session.session_id) {
                return Err(format!("Duplicate session ID: {}", session.session_id));
            }
        }

        Ok(())
    }

    fn check_session_requirements(
        &self,
        progress: &ParticipantProgress,
        session_plan: &SessionPlan,
    ) -> Result<bool, String> {
        for requirement in &session_plan.pre_session_requirements {
            match requirement {
                Requirement::MinimumRestTime { hours } => {
                    if let Some(last_session) = progress.completed_sessions.last() {
                        let time_since = chrono::Utc::now()
                            .signed_duration_since(last_session.completed_at)
                            .num_hours();

                        if time_since < *hours as i64 {
                            return Ok(false);
                        }
                    }
                }
                Requirement::CompletedPreviousSession => {
                    let expected_completed = session_plan.order;
                    if progress.completed_sessions.len() < expected_completed {
                        return Ok(false);
                    }
                }
                _ => {
                    // Other requirements would be checked based on implementation needs
                }
            }
        }

        Ok(true)
    }

    fn calculate_data_quality_score(&self, performance_data: &HashMap<String, f64>) -> f64 {
        // Simplified data quality assessment
        let mut score = 1.0;

        // Check for missing data
        if performance_data.is_empty() {
            score *= 0.1;
        }

        // Check for extreme outliers (simplified)
        for value in performance_data.values() {
            if value.is_nan() || value.is_infinite() {
                score *= 0.5;
            }
        }

        score
    }

    fn check_interim_analysis(
        &self,
        experiment_id: &str,
        completed_sessions: usize,
    ) -> Result<(), String> {
        let experiment = self
            .experiments
            .get(experiment_id)
            .ok_or("Experiment not found")?;

        for interim in &experiment.analysis_plan.interim_analyses {
            if interim.scheduled_after_session == completed_sessions {
                // TODO: Trigger interim analysis
                println!(
                    "Interim analysis triggered after session {}",
                    completed_sessions
                );
            }
        }

        Ok(())
    }

    fn fit_learning_curve(&self, session_stats: &[SessionStatistics]) -> LearningCurve {
        // Fit exponential learning curve: y = a * exp(b * x) + c
        // Simplified implementation - would use proper curve fitting

        if session_stats.len() < 3 {
            return LearningCurve::default();
        }

        let x_values: Vec<f64> = session_stats
            .iter()
            .map(|s| s.session_index as f64)
            .collect();
        let y_values: Vec<f64> = session_stats.iter().map(|s| s.mean).collect();

        // Simple linear regression on log-transformed data
        let n = x_values.len() as f64;
        let sum_x = x_values.iter().sum::<f64>();
        let sum_y = y_values.iter().sum::<f64>();
        let sum_xy = x_values
            .iter()
            .zip(y_values.iter())
            .map(|(x, y)| x * y)
            .sum::<f64>();
        let sum_x2 = x_values.iter().map(|x| x * x).sum::<f64>();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        LearningCurve {
            model_type: "linear".to_string(),
            parameters: vec![intercept, slope],
            r_squared: 0.0, // Would calculate properly
            predicted_values: x_values.iter().map(|x| intercept + slope * x).collect(),
        }
    }

    fn detect_change_points(&self, session_stats: &[SessionStatistics]) -> Vec<ChangePoint> {
        // Simplified change point detection
        let mut change_points = Vec::new();

        if session_stats.len() < 5 {
            return change_points;
        }

        // Look for significant changes in mean or variance
        for i in 2..session_stats.len() - 2 {
            let before: Vec<f64> = session_stats[0..i].iter().map(|s| s.mean).collect();
            let after: Vec<f64> = session_stats[i..].iter().map(|s| s.mean).collect();

            let mean_before = before.iter().sum::<f64>() / before.len() as f64;
            let mean_after = after.iter().sum::<f64>() / after.len() as f64;

            let change_magnitude = (mean_after - mean_before).abs();
            let pooled_std = (session_stats[i - 1].std_dev + session_stats[i].std_dev) / 2.0;

            // Simple threshold-based detection
            if change_magnitude > 2.0 * pooled_std && pooled_std > 0.0 {
                change_points.push(ChangePoint {
                    session_index: i,
                    change_type: if mean_after > mean_before {
                        ChangeType::Improvement
                    } else {
                        ChangeType::Decline
                    },
                    magnitude: change_magnitude,
                    confidence: 0.8, // Would calculate properly
                });
            }
        }

        change_points
    }
}

// Analysis structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongitudinalAnalysis {
    pub outcome_measure: String,
    pub participant_trajectories: HashMap<String, Vec<SessionDataPoint>>,
    pub session_statistics: Vec<SessionStatistics>,
    pub learning_curve: LearningCurve,
    pub change_points: Vec<ChangePoint>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDataPoint {
    pub session_index: usize,
    pub value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data_quality: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub session_index: usize,
    pub n: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningCurve {
    pub model_type: String,
    pub parameters: Vec<f64>,
    pub r_squared: f64,
    pub predicted_values: Vec<f64>,
}

impl Default for LearningCurve {
    fn default() -> Self {
        Self {
            model_type: "linear".to_string(),
            parameters: vec![0.0, 0.0],
            r_squared: 0.0,
            predicted_values: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePoint {
    pub session_index: usize,
    pub change_type: ChangeType,
    pub magnitude: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Improvement,
    Decline,
    PlateauStart,
    PlateauEnd,
}

// Default implementations for common configurations
impl Default for DataRetentionPolicy {
    fn default() -> Self {
        Self {
            retain_raw_data_days: 365,
            retain_aggregated_data_days: 1825, // 5 years
            anonymize_after_days: 90,
            export_before_deletion: true,
            gdpr_compliance: true,
        }
    }
}

impl Default for AnalysisPlan {
    fn default() -> Self {
        Self {
            primary_outcomes: vec![OutcomeMeasure {
                name: "accuracy".to_string(),
                description: "Overall accuracy score".to_string(),
                measure_type: MeasureType::AccuracyScore,
                calculation: "correct_responses / total_responses".to_string(),
                higher_is_better: true,
            }],
            secondary_outcomes: Vec::new(),
            planned_comparisons: Vec::new(),
            interim_analyses: Vec::new(),
            statistical_tests: Vec::new(),
        }
    }
}

/// Scheduling system for experiments
pub struct ExperimentScheduler {
    // Would implement calendar integration, conflict resolution, etc.
}

impl ExperimentScheduler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn find_available_slot(
        &self,
        rules: &SchedulingRules,
        session: &SessionPlan,
        progress: &ParticipantProgress,
    ) -> Result<chrono::DateTime<chrono::Utc>, String> {
        // Simplified implementation - would integrate with proper calendar system
        let base_time = chrono::Utc::now() + chrono::Duration::hours(24);

        // Apply minimum interval if required
        if let Some(min_hours) = session.minimum_interval_hours {
            if let Some(last_session) = progress.completed_sessions.last() {
                let min_time =
                    last_session.completed_at + chrono::Duration::hours(min_hours as i64);
                if base_time < min_time {
                    return Ok(min_time);
                }
            }
        }

        Ok(base_time)
    }

    pub fn schedule_reminders(
        &self,
        participant_id: &str,
        session_time: &chrono::DateTime<chrono::Utc>,
        reminder_settings: &ReminderSettings,
    ) -> Result<(), String> {
        if !reminder_settings.enabled {
            return Ok(());
        }

        // Would integrate with notification system
        for hours_before in &reminder_settings.advance_hours {
            let reminder_time = *session_time - chrono::Duration::hours(*hours_before as i64);
            println!(
                "Reminder scheduled for {} at {}",
                participant_id, reminder_time
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_multi_session_creation() {
        let mut manager = MultiSessionManager::new(PathBuf::from("./test_data"));

        let sessions = vec![
            SessionPlan {
                session_id: "session_1".to_string(),
                name: "Baseline".to_string(),
                description: "Initial assessment".to_string(),
                order: 0,
                duration_minutes: Some(30),
                minimum_interval_hours: None,
                maximum_interval_hours: None,
                config: LearnerConfig::default(),
                topology: Topology::alphabet(),
                tasks: Vec::new(),
                pre_session_requirements: Vec::new(),
                post_session_procedures: Vec::new(),
            },
            SessionPlan {
                session_id: "session_2".to_string(),
                name: "Training".to_string(),
                description: "Learning phase".to_string(),
                order: 1,
                duration_minutes: Some(45),
                minimum_interval_hours: Some(24),
                maximum_interval_hours: Some(168), // 1 week
                config: LearnerConfig::default(),
                topology: Topology::alphabet(),
                tasks: Vec::new(),
                pre_session_requirements: vec![Requirement::CompletedPreviousSession],
                post_session_procedures: Vec::new(),
            },
        ];

        let scheduling_rules = SchedulingRules {
            allow_overlap: false,
            max_concurrent_participants: Some(10),
            preferred_time_windows: Vec::new(),
            blocked_dates: Vec::new(),
            reminder_settings: ReminderSettings {
                enabled: true,
                advance_hours: vec![24, 1],
                method: vec![ReminderMethod::InApp],
            },
            rescheduling_policy: ReschedulingPolicy {
                max_reschedules_per_session: 2,
                automatic_reschedule_window_hours: 24,
                penalty_for_no_show: None,
            },
        };

        use crate::experimental::design::{
            ExperimentCondition, ExperimentalDesign, RandomizationType,
        };
        let design = ExperimentalDesign::BetweenSubjects {
            conditions: vec![ExperimentCondition {
                id: "control".to_string(),
                name: "Control".to_string(),
                description: "Standard condition".to_string(),
                config: LearnerConfig::default(),
                topology: Topology::alphabet(),
                parameters: HashMap::new(),
            }],
            randomization: RandomizationType::Simple,
        };

        let experiment_id = manager
            .create_experiment(
                "Test Study".to_string(),
                "Multi-session learning study".to_string(),
                design,
                sessions,
                scheduling_rules,
            )
            .unwrap();

        assert!(!experiment_id.is_empty());
        assert!(manager.experiments.contains_key(&experiment_id));
    }
}
