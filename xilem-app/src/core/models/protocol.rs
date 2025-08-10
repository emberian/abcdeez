use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Protocol Versioning Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub id: String,
    pub experiment_id: String,
    pub version: String,
    pub commit_hash: String,
    pub author: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub protocol_content: ProtocolContent,
    pub is_current: bool,
    pub is_draft: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolContent {
    pub name: String,
    pub description: String,
    pub experiment_design: ExperimentDesignConfig,
    pub data_collection_plan: DataCollectionPlan,
    pub analysis_plan: AnalysisPlan,
    pub compliance_requirements: ComplianceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentDesignConfig {
    pub design_type: String,
    pub conditions: Vec<ExperimentCondition>,
    pub randomization: RandomizationConfig,
    pub sample_size: SampleSizeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentCondition {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub is_control: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomizationConfig {
    pub method: String,
    pub seed: Option<u64>,
    pub stratification: Vec<String>,
    pub block_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleSizeConfig {
    pub target_n: usize,
    pub power: f64,
    pub effect_size: f64,
    pub alpha: f64,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCollectionPlan {
    pub duration_weeks: u32,
    pub sessions_per_participant: u32,
    pub data_types: Vec<String>,
    pub quality_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPlan {
    pub primary_analyses: Vec<String>,
    pub secondary_analyses: Vec<String>,
    pub statistical_tests: Vec<String>,
    pub multiple_comparison_correction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirements {
    pub irb_required: bool,
    pub consent_required: bool,
    pub data_retention_years: u32,
    pub privacy_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersionDiff {
    pub from_version: String,
    pub to_version: String,
    pub changes: Vec<ProtocolChange>,
    pub summary: String,
    pub compatibility: CompatibilityStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolChange {
    pub field: String,
    pub change_type: ChangeType,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub description: String,
    pub impact_level: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Major, // Breaking changes
    Minor, // New features, non-breaking
    Patch, // Bug fixes, clarifications
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilityStatus {
    Compatible,
    MinorIncompatibility,
    MajorIncompatibility,
    RequiresReview,
}