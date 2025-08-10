use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

/// Protocol Version Control System for Experimental Design Reproducibility
/// Provides comprehensive versioning, change tracking, and collaboration features
/// for experimental protocols and research methodologies

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersionControl {
    pub repository_id: String,
    pub repository_name: String,
    pub protocols: HashMap<String, ProtocolHistory>,
    pub branches: HashMap<String, ProtocolBranch>,
    pub active_branch: String,
    pub collaboration_settings: CollaborationSettings,
    pub metadata: RepositoryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolHistory {
    pub protocol_id: String,
    pub protocol_name: String,
    pub versions: Vec<ProtocolVersion>,
    pub current_version: String,
    pub tags: HashMap<String, String>, // tag_name -> version_hash
    pub merge_history: Vec<MergeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub version_hash: String,
    pub version_number: String,
    pub parent_hash: Option<String>,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub commit_message: String,
    pub changes: Vec<ProtocolChange>,
    pub protocol_state: ProtocolSnapshot,
    pub validation_results: ValidationResults,
    pub diff_summary: DiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSnapshot {
    pub experimental_design: ExperimentalDesignSnapshot,
    pub randomization_settings: RandomizationSnapshot,
    pub data_collection_params: DataCollectionSnapshot,
    pub analysis_pipeline: AnalysisPipelineSnapshot,
    pub materials: Vec<MaterialSnapshot>,
    pub ethical_approvals: Vec<EthicalApprovalSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalDesignSnapshot {
    pub design_type: String,
    pub factors: Vec<FactorSnapshot>,
    pub conditions: Vec<ConditionSnapshot>,
    pub counterbalancing: CounterbalancingSnapshot,
    pub sample_size: SampleSizeSnapshot,
    pub inclusion_criteria: Vec<String>,
    pub exclusion_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorSnapshot {
    pub name: String,
    pub factor_type: String, // within, between, mixed
    pub levels: Vec<String>,
    pub manipulation_details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionSnapshot {
    pub condition_id: String,
    pub condition_name: String,
    pub factor_levels: HashMap<String, String>,
    pub expected_n: usize,
    pub materials: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterbalancingSnapshot {
    pub method: String, // latin_square, randomized, fixed
    pub constraints: Vec<String>,
    pub balancing_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleSizeSnapshot {
    pub target_n: usize,
    pub power_analysis: PowerAnalysisSnapshot,
    pub recruitment_plan: String,
    pub attrition_expectations: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerAnalysisSnapshot {
    pub effect_size: f64,
    pub alpha: f64,
    pub power: f64,
    pub test_type: String,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomizationSnapshot {
    pub randomization_type: String,
    pub seed: Option<u64>,
    pub block_size: Option<usize>,
    pub stratification_factors: Vec<String>,
    pub allocation_ratio: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCollectionSnapshot {
    pub collection_methods: Vec<String>,
    pub instruments: Vec<InstrumentSnapshot>,
    pub timing: TimingSnapshot,
    pub environment_controls: Vec<String>,
    pub quality_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentSnapshot {
    pub instrument_name: String,
    pub version: String,
    pub parameters: HashMap<String, String>,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingSnapshot {
    pub session_duration: String,
    pub break_intervals: Vec<String>,
    pub time_limits: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPipelineSnapshot {
    pub primary_analyses: Vec<AnalysisSnapshot>,
    pub secondary_analyses: Vec<AnalysisSnapshot>,
    pub exploratory_analyses: Vec<AnalysisSnapshot>,
    pub preprocessing_steps: Vec<String>,
    pub statistical_software: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSnapshot {
    pub analysis_name: String,
    pub method: String,
    pub dependent_variables: Vec<String>,
    pub independent_variables: Vec<String>,
    pub covariates: Vec<String>,
    pub assumptions: Vec<String>,
    pub interpretation_guidelines: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialSnapshot {
    pub material_id: String,
    pub material_type: String,
    pub content: String,
    pub version: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalApprovalSnapshot {
    pub approval_id: String,
    pub institution: String,
    pub approval_date: DateTime<Utc>,
    pub expiry_date: DateTime<Utc>,
    pub conditions: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolChange {
    pub change_type: ChangeType,
    pub section: String,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub justification: String,
    pub impact_assessment: ImpactAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Addition,
    Modification,
    Deletion,
    Restructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub validity_impact: ValidityImpact,
    pub sample_size_impact: SampleSizeImpact,
    pub timeline_impact: TimelineImpact,
    pub resource_impact: ResourceImpact,
    pub ethical_impact: EthicalImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidityImpact {
    None,
    Minimal,
    Moderate,
    Significant,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SampleSizeImpact {
    None,
    Increase(f64),
    Decrease(f64),
    Recalculation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimelineImpact {
    None,
    Delay(String),
    Acceleration(String),
    Restructure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceImpact {
    None,
    Additional(String),
    Reduced(String),
    Reallocation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EthicalImpact {
    None,
    Amendment,
    Reapproval,
    NewApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub completeness_score: f64,
    pub consistency_checks: ConsistencyResults,
    pub reproducibility_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub error_code: String,
    pub section: String,
    pub field: String,
    pub message: String,
    pub severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,
    Major,
    Minor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub warning_code: String,
    pub section: String,
    pub message: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyResults {
    pub internal_consistency: bool,
    pub cross_section_consistency: HashMap<String, bool>,
    pub temporal_consistency: bool,
    pub methodological_alignment: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub additions: usize,
    pub modifications: usize,
    pub deletions: usize,
    pub sections_changed: Vec<String>,
    pub critical_changes: Vec<String>,
    pub change_magnitude: ChangeMagnitude,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeMagnitude {
    Trivial,   // Minor parameter adjustments
    Minor,     // Single section modifications
    Moderate,  // Multiple section changes
    Major,     // Fundamental design changes
    Extensive, // Complete protocol overhaul
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolBranch {
    pub branch_name: String,
    pub base_version: String,
    pub head_version: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub description: String,
    pub status: BranchStatus,
    pub merge_conflicts: Vec<MergeConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchStatus {
    Active,
    ReadyForReview,
    UnderReview,
    Approved,
    Merged,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRecord {
    pub merge_id: String,
    pub source_branch: String,
    pub target_branch: String,
    pub merge_strategy: MergeStrategy,
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub conflicts_resolved: Vec<ConflictResolution>,
    pub validation_post_merge: ValidationResults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
    FastForward,
    ThreeWay,
    Squash,
    Cherry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    pub conflict_id: String,
    pub section: String,
    pub field: String,
    pub base_value: String,
    pub source_value: String,
    pub target_value: String,
    pub conflict_type: ConflictType,
    pub resolution_options: Vec<ResolutionOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    ValueConflict,
    StructuralConflict,
    TypeConflict,
    DependencyConflict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionOption {
    pub option_name: String,
    pub description: String,
    pub resulting_value: String,
    pub impact_assessment: ImpactAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub conflict_id: String,
    pub resolution_choice: String,
    pub justification: String,
    pub validator: String,
    pub validation_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSettings {
    pub collaborators: Vec<Collaborator>,
    pub access_control: AccessControl,
    pub review_requirements: ReviewRequirements,
    pub notification_settings: NotificationSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub role: CollaboratorRole,
    pub permissions: HashSet<Permission>,
    pub institution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaboratorRole {
    Owner,
    Principal,
    Researcher,
    Analyst,
    Reviewer,
    Observer,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Permission {
    Read,
    Write,
    CreateBranch,
    Merge,
    Delete,
    AdministerUsers,
    ReviewChanges,
    ApproveChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    pub require_approval: bool,
    pub protected_sections: Vec<String>,
    pub minimum_reviewers: usize,
    pub auto_approval_roles: Vec<CollaboratorRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequirements {
    pub mandatory_reviews: Vec<String>, // Section names
    pub review_criteria: Vec<ReviewCriterion>,
    pub sign_off_requirements: Vec<SignOffRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCriterion {
    pub criterion_name: String,
    pub description: String,
    pub required: bool,
    pub applicable_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignOffRequirement {
    pub requirement_name: String,
    pub required_role: CollaboratorRole,
    pub sections: Vec<String>,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub email_notifications: bool,
    pub change_notifications: Vec<String>,
    pub review_notifications: bool,
    pub merge_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub description: String,
    pub research_domain: String,
    pub study_type: String,
    pub institutions: Vec<String>,
    pub funding_sources: Vec<String>,
    pub license: String,
}

/// Main protocol version control manager
pub struct ProtocolVersionManager {
    repository: ProtocolVersionControl,
    validation_rules: ValidationRuleSet,
}

#[derive(Debug, Clone)]
pub struct ValidationRuleSet {
    pub required_sections: HashSet<String>,
    pub cross_validation_rules: Vec<CrossValidationRule>,
    pub format_validators: HashMap<String, FormatValidator>,
    pub business_rules: Vec<BusinessRule>,
}

#[derive(Debug, Clone)]
pub struct CrossValidationRule {
    pub rule_name: String,
    pub sections: Vec<String>,
    pub validation_function: String, // Would be function pointer in practice
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct FormatValidator {
    pub validator_name: String,
    pub pattern: String,
    pub error_message: String,
}

#[derive(Debug, Clone)]
pub struct BusinessRule {
    pub rule_name: String,
    pub condition: String,
    pub action: String,
    pub severity: ErrorSeverity,
}

impl ProtocolVersionManager {
    pub fn new(repository_name: String, created_by: String) -> Self {
        let repository_id = format!("repo_{}", Utc::now().timestamp());

        let mut branches = HashMap::new();
        branches.insert(
            "main".to_string(),
            ProtocolBranch {
                branch_name: "main".to_string(),
                base_version: "initial".to_string(),
                head_version: "initial".to_string(),
                author: created_by.clone(),
                created_at: Utc::now(),
                description: "Main development branch".to_string(),
                status: BranchStatus::Active,
                merge_conflicts: Vec::new(),
            },
        );

        let repository = ProtocolVersionControl {
            repository_id,
            repository_name,
            protocols: HashMap::new(),
            branches,
            active_branch: "main".to_string(),
            collaboration_settings: CollaborationSettings {
                collaborators: Vec::new(),
                access_control: AccessControl {
                    require_approval: true,
                    protected_sections: vec![
                        "experimental_design".to_string(),
                        "analysis_pipeline".to_string(),
                    ],
                    minimum_reviewers: 1,
                    auto_approval_roles: vec![CollaboratorRole::Owner],
                },
                review_requirements: ReviewRequirements {
                    mandatory_reviews: vec!["experimental_design".to_string()],
                    review_criteria: Vec::new(),
                    sign_off_requirements: Vec::new(),
                },
                notification_settings: NotificationSettings {
                    email_notifications: true,
                    change_notifications: vec!["all".to_string()],
                    review_notifications: true,
                    merge_notifications: true,
                },
            },
            metadata: RepositoryMetadata {
                created_at: Utc::now(),
                created_by,
                description: String::new(),
                research_domain: String::new(),
                study_type: String::new(),
                institutions: Vec::new(),
                funding_sources: Vec::new(),
                license: "Proprietary".to_string(),
            },
        };

        let validation_rules = ValidationRuleSet {
            required_sections: [
                "experimental_design",
                "randomization_settings",
                "data_collection_params",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            cross_validation_rules: Vec::new(),
            format_validators: HashMap::new(),
            business_rules: Vec::new(),
        };

        Self {
            repository,
            validation_rules,
        }
    }

    /// Create a new protocol version
    pub fn create_version(
        &mut self,
        protocol_id: String,
        author: String,
        commit_message: String,
        protocol_snapshot: ProtocolSnapshot,
    ) -> Result<String, String> {
        // Validate the protocol snapshot
        let validation_results = self.validate_protocol(&protocol_snapshot)?;

        if !validation_results.is_valid {
            return Err(format!(
                "Protocol validation failed: {:?}",
                validation_results.errors
            ));
        }

        // Generate version hash
        let version_hash = self.generate_version_hash(&protocol_snapshot, &author, &commit_message);
        let version_number = self.generate_version_number(&protocol_id);

        // Determine parent hash
        let parent_hash = self
            .repository
            .protocols
            .get(&protocol_id)
            .and_then(|history| history.versions.last())
            .map(|v| v.version_hash.clone());

        // Calculate changes from parent
        let changes = if let Some(ref parent) = parent_hash {
            self.calculate_changes(&protocol_id, parent, &protocol_snapshot)?
        } else {
            Vec::new() // First version has no changes
        };

        let diff_summary = self.calculate_diff_summary(&changes);

        let version = ProtocolVersion {
            version_hash: version_hash.clone(),
            version_number: version_number.clone(),
            parent_hash,
            author,
            timestamp: Utc::now(),
            commit_message,
            changes,
            protocol_state: protocol_snapshot,
            validation_results,
            diff_summary,
        };

        // Add to protocol history
        let protocol_history = self
            .repository
            .protocols
            .entry(protocol_id.clone())
            .or_insert_with(|| ProtocolHistory {
                protocol_id: protocol_id.clone(),
                protocol_name: format!("Protocol {}", protocol_id),
                versions: Vec::new(),
                current_version: String::new(),
                tags: HashMap::new(),
                merge_history: Vec::new(),
            });

        protocol_history.versions.push(version);
        protocol_history.current_version = version_hash.clone();

        Ok(version_hash)
    }

    /// Create a new branch
    pub fn create_branch(
        &mut self,
        branch_name: String,
        base_version: String,
        author: String,
        description: String,
    ) -> Result<(), String> {
        if self.repository.branches.contains_key(&branch_name) {
            return Err("Branch already exists".to_string());
        }

        let branch = ProtocolBranch {
            branch_name: branch_name.clone(),
            base_version: base_version.clone(),
            head_version: base_version,
            author,
            created_at: Utc::now(),
            description,
            status: BranchStatus::Active,
            merge_conflicts: Vec::new(),
        };

        self.repository.branches.insert(branch_name, branch);
        Ok(())
    }

    /// Switch to a different branch
    pub fn checkout_branch(&mut self, branch_name: String) -> Result<(), String> {
        if !self.repository.branches.contains_key(&branch_name) {
            return Err("Branch does not exist".to_string());
        }

        self.repository.active_branch = branch_name;
        Ok(())
    }

    /// Merge a branch
    pub fn merge_branch(
        &mut self,
        source_branch: String,
        target_branch: String,
        author: String,
        strategy: MergeStrategy,
    ) -> Result<String, String> {
        // Check if branches exist
        let source = self
            .repository
            .branches
            .get(&source_branch)
            .ok_or("Source branch does not exist")?;
        let target = self
            .repository
            .branches
            .get(&target_branch)
            .ok_or("Target branch does not exist")?;

        // Detect conflicts
        let conflicts = self.detect_merge_conflicts(&source_branch, &target_branch)?;

        if !conflicts.is_empty() {
            return Err(format!(
                "Merge conflicts detected: {} conflicts",
                conflicts.len()
            ));
        }

        // Perform merge
        let merge_id = format!("merge_{}", Utc::now().timestamp());
        let merge_record = MergeRecord {
            merge_id: merge_id.clone(),
            source_branch: source_branch.clone(),
            target_branch: target_branch.clone(),
            merge_strategy: strategy,
            timestamp: Utc::now(),
            author,
            conflicts_resolved: Vec::new(),
            validation_post_merge: ValidationResults {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                completeness_score: 1.0,
                consistency_checks: ConsistencyResults {
                    internal_consistency: true,
                    cross_section_consistency: HashMap::new(),
                    temporal_consistency: true,
                    methodological_alignment: true,
                },
                reproducibility_score: 1.0,
            },
        };

        // Update branch statuses
        if let Some(source_branch_mut) = self.repository.branches.get_mut(&source_branch) {
            source_branch_mut.status = BranchStatus::Merged;
        }

        // Add merge record to protocols
        for protocol_history in self.repository.protocols.values_mut() {
            protocol_history.merge_history.push(merge_record.clone());
        }

        Ok(merge_id)
    }

    /// Tag a version
    pub fn tag_version(
        &mut self,
        protocol_id: String,
        version_hash: String,
        tag_name: String,
    ) -> Result<(), String> {
        let protocol_history = self
            .repository
            .protocols
            .get_mut(&protocol_id)
            .ok_or("Protocol not found")?;

        // Verify version exists
        let version_exists = protocol_history
            .versions
            .iter()
            .any(|v| v.version_hash == version_hash);

        if !version_exists {
            return Err("Version not found".to_string());
        }

        protocol_history.tags.insert(tag_name, version_hash);
        Ok(())
    }

    /// Get protocol history
    pub fn get_protocol_history(&self, protocol_id: &str) -> Option<&ProtocolHistory> {
        self.repository.protocols.get(protocol_id)
    }

    /// Get version diff
    pub fn get_version_diff(
        &self,
        protocol_id: &str,
        version1: &str,
        version2: &str,
    ) -> Result<Vec<ProtocolChange>, String> {
        let protocol_history = self
            .repository
            .protocols
            .get(protocol_id)
            .ok_or("Protocol not found")?;

        let v1 = protocol_history
            .versions
            .iter()
            .find(|v| v.version_hash == version1)
            .ok_or("Version 1 not found")?;

        let v2 = protocol_history
            .versions
            .iter()
            .find(|v| v.version_hash == version2)
            .ok_or("Version 2 not found")?;

        // Calculate differences between snapshots
        self.calculate_snapshot_diff(&v1.protocol_state, &v2.protocol_state)
    }

    /// Add collaborator
    pub fn add_collaborator(
        &mut self,
        user_id: String,
        name: String,
        email: String,
        role: CollaboratorRole,
        institution: String,
    ) -> Result<(), String> {
        let permissions = match role {
            CollaboratorRole::Owner => [
                Permission::Read,
                Permission::Write,
                Permission::CreateBranch,
                Permission::Merge,
                Permission::Delete,
                Permission::AdministerUsers,
                Permission::ReviewChanges,
                Permission::ApproveChanges,
            ]
            .iter()
            .cloned()
            .collect(),
            CollaboratorRole::Principal => [
                Permission::Read,
                Permission::Write,
                Permission::CreateBranch,
                Permission::Merge,
                Permission::ReviewChanges,
                Permission::ApproveChanges,
            ]
            .iter()
            .cloned()
            .collect(),
            CollaboratorRole::Researcher => [
                Permission::Read,
                Permission::Write,
                Permission::CreateBranch,
            ]
            .iter()
            .cloned()
            .collect(),
            CollaboratorRole::Analyst => [Permission::Read, Permission::Write]
                .iter()
                .cloned()
                .collect(),
            CollaboratorRole::Reviewer => [Permission::Read, Permission::ReviewChanges]
                .iter()
                .cloned()
                .collect(),
            CollaboratorRole::Observer => [Permission::Read].iter().cloned().collect(),
        };

        let collaborator = Collaborator {
            user_id,
            name,
            email,
            role,
            permissions,
            institution,
        };

        self.repository
            .collaboration_settings
            .collaborators
            .push(collaborator);
        Ok(())
    }

    /// Export protocol version for external use
    pub fn export_protocol_version(
        &self,
        protocol_id: &str,
        version_hash: &str,
        format: ExportFormat,
    ) -> Result<String, String> {
        let protocol_history = self
            .repository
            .protocols
            .get(protocol_id)
            .ok_or("Protocol not found")?;

        let version = protocol_history
            .versions
            .iter()
            .find(|v| v.version_hash == version_hash)
            .ok_or("Version not found")?;

        match format {
            ExportFormat::JSON => serde_json::to_string_pretty(&version.protocol_state)
                .map_err(|e| format!("JSON export failed: {}", e)),
            ExportFormat::YAML => serde_yaml::to_string(&version.protocol_state)
                .map_err(|e| format!("YAML export failed: {}", e)),
            ExportFormat::PDF => {
                // Would generate PDF report
                Ok("PDF export not yet implemented".to_string())
            }
            ExportFormat::LaTeX => {
                // Would generate LaTeX document
                self.generate_latex_protocol(&version.protocol_state)
            }
        }
    }

    // Helper methods

    fn validate_protocol(&self, snapshot: &ProtocolSnapshot) -> Result<ValidationResults, String> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check required sections
        if snapshot.experimental_design.factors.is_empty() {
            errors.push(ValidationError {
                error_code: "ED001".to_string(),
                section: "experimental_design".to_string(),
                field: "factors".to_string(),
                message: "At least one factor must be defined".to_string(),
                severity: ErrorSeverity::Critical,
            });
        }

        if snapshot.experimental_design.conditions.is_empty() {
            errors.push(ValidationError {
                error_code: "ED002".to_string(),
                section: "experimental_design".to_string(),
                field: "conditions".to_string(),
                message: "At least one condition must be defined".to_string(),
                severity: ErrorSeverity::Critical,
            });
        }

        // Check sample size
        if snapshot.experimental_design.sample_size.target_n == 0 {
            errors.push(ValidationError {
                error_code: "SS001".to_string(),
                section: "experimental_design".to_string(),
                field: "sample_size".to_string(),
                message: "Target sample size must be greater than 0".to_string(),
                severity: ErrorSeverity::Critical,
            });
        }

        // Check power analysis
        let power = snapshot
            .experimental_design
            .sample_size
            .power_analysis
            .power;
        if power < 0.8 {
            warnings.push(ValidationWarning {
                warning_code: "PA001".to_string(),
                section: "experimental_design".to_string(),
                message: format!(
                    "Power is low ({:.2}), consider increasing sample size",
                    power
                ),
                recommendation: "Increase sample size to achieve power ≥ 0.8".to_string(),
            });
        }

        let is_valid = errors.is_empty();
        let completeness_score = self.calculate_completeness_score(snapshot);
        let reproducibility_score = self.calculate_reproducibility_score(snapshot);

        Ok(ValidationResults {
            is_valid,
            errors,
            warnings,
            completeness_score,
            consistency_checks: ConsistencyResults {
                internal_consistency: true,
                cross_section_consistency: HashMap::new(),
                temporal_consistency: true,
                methodological_alignment: true,
            },
            reproducibility_score,
        })
    }

    fn generate_version_hash(
        &self,
        snapshot: &ProtocolSnapshot,
        author: &str,
        message: &str,
    ) -> String {
        let mut hasher = Sha256::new();

        // Hash the protocol snapshot
        if let Ok(snapshot_json) = serde_json::to_string(snapshot) {
            hasher.update(snapshot_json.as_bytes());
        }

        hasher.update(author.as_bytes());
        hasher.update(message.as_bytes());
        hasher.update(Utc::now().timestamp().to_string().as_bytes());

        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    fn generate_version_number(&self, protocol_id: &str) -> String {
        let version_count = self
            .repository
            .protocols
            .get(protocol_id)
            .map(|h| h.versions.len())
            .unwrap_or(0);

        format!("v1.{}.0", version_count)
    }

    fn calculate_changes(
        &self,
        protocol_id: &str,
        parent_hash: &str,
        current_snapshot: &ProtocolSnapshot,
    ) -> Result<Vec<ProtocolChange>, String> {
        let protocol_history = self
            .repository
            .protocols
            .get(protocol_id)
            .ok_or("Protocol not found")?;

        let parent_version = protocol_history
            .versions
            .iter()
            .find(|v| v.version_hash == parent_hash)
            .ok_or("Parent version not found")?;

        self.calculate_snapshot_diff(&parent_version.protocol_state, current_snapshot)
    }

    fn calculate_snapshot_diff(
        &self,
        old_snapshot: &ProtocolSnapshot,
        new_snapshot: &ProtocolSnapshot,
    ) -> Result<Vec<ProtocolChange>, String> {
        let mut changes = Vec::new();

        // Compare experimental design factors
        if old_snapshot.experimental_design.factors.len()
            != new_snapshot.experimental_design.factors.len()
        {
            changes.push(ProtocolChange {
                change_type: ChangeType::Modification,
                section: "experimental_design".to_string(),
                field: "factors".to_string(),
                old_value: Some(format!(
                    "{} factors",
                    old_snapshot.experimental_design.factors.len()
                )),
                new_value: Some(format!(
                    "{} factors",
                    new_snapshot.experimental_design.factors.len()
                )),
                justification: "Factor structure modified".to_string(),
                impact_assessment: ImpactAssessment {
                    validity_impact: ValidityImpact::Moderate,
                    sample_size_impact: SampleSizeImpact::Recalculation,
                    timeline_impact: TimelineImpact::None,
                    resource_impact: ResourceImpact::None,
                    ethical_impact: EthicalImpact::Amendment,
                },
            });
        }

        // Compare sample sizes
        if old_snapshot.experimental_design.sample_size.target_n
            != new_snapshot.experimental_design.sample_size.target_n
        {
            changes.push(ProtocolChange {
                change_type: ChangeType::Modification,
                section: "experimental_design".to_string(),
                field: "sample_size".to_string(),
                old_value: Some(
                    old_snapshot
                        .experimental_design
                        .sample_size
                        .target_n
                        .to_string(),
                ),
                new_value: Some(
                    new_snapshot
                        .experimental_design
                        .sample_size
                        .target_n
                        .to_string(),
                ),
                justification: "Sample size adjusted based on power analysis".to_string(),
                impact_assessment: ImpactAssessment {
                    validity_impact: ValidityImpact::Minimal,
                    sample_size_impact: if new_snapshot.experimental_design.sample_size.target_n
                        > old_snapshot.experimental_design.sample_size.target_n
                    {
                        SampleSizeImpact::Increase(
                            (new_snapshot.experimental_design.sample_size.target_n as f64
                                / old_snapshot.experimental_design.sample_size.target_n as f64)
                                - 1.0,
                        )
                    } else {
                        SampleSizeImpact::Decrease(
                            1.0 - (new_snapshot.experimental_design.sample_size.target_n as f64
                                / old_snapshot.experimental_design.sample_size.target_n as f64),
                        )
                    },
                    timeline_impact: TimelineImpact::None,
                    resource_impact: ResourceImpact::Additional(
                        "Recruitment resources".to_string(),
                    ),
                    ethical_impact: EthicalImpact::Amendment,
                },
            });
        }

        Ok(changes)
    }

    fn calculate_diff_summary(&self, changes: &[ProtocolChange]) -> DiffSummary {
        let additions = changes
            .iter()
            .filter(|c| matches!(c.change_type, ChangeType::Addition))
            .count();
        let modifications = changes
            .iter()
            .filter(|c| matches!(c.change_type, ChangeType::Modification))
            .count();
        let deletions = changes
            .iter()
            .filter(|c| matches!(c.change_type, ChangeType::Deletion))
            .count();

        let sections_changed: HashSet<String> = changes.iter().map(|c| c.section.clone()).collect();
        let sections_changed: Vec<String> = sections_changed.into_iter().collect();

        let critical_changes: Vec<String> = changes
            .iter()
            .filter(|c| {
                matches!(
                    c.impact_assessment.validity_impact,
                    ValidityImpact::Critical | ValidityImpact::Significant
                )
            })
            .map(|c| format!("{}.{}", c.section, c.field))
            .collect();

        let total_changes = additions + modifications + deletions;
        let change_magnitude = if total_changes == 0 {
            ChangeMagnitude::Trivial
        } else if total_changes < 3 && critical_changes.is_empty() {
            ChangeMagnitude::Minor
        } else if total_changes < 10 {
            ChangeMagnitude::Moderate
        } else if total_changes < 20 {
            ChangeMagnitude::Major
        } else {
            ChangeMagnitude::Extensive
        };

        DiffSummary {
            additions,
            modifications,
            deletions,
            sections_changed,
            critical_changes,
            change_magnitude,
        }
    }

    fn detect_merge_conflicts(
        &self,
        _source_branch: &str,
        _target_branch: &str,
    ) -> Result<Vec<MergeConflict>, String> {
        // Simplified conflict detection
        // In practice, would compare protocol states between branches
        Ok(Vec::new())
    }

    fn calculate_completeness_score(&self, snapshot: &ProtocolSnapshot) -> f64 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Experimental design completeness
        max_score += 1.0;
        if !snapshot.experimental_design.factors.is_empty()
            && !snapshot.experimental_design.conditions.is_empty()
            && snapshot.experimental_design.sample_size.target_n > 0
        {
            score += 1.0;
        }

        // Randomization completeness
        max_score += 1.0;
        if !snapshot
            .randomization_settings
            .randomization_type
            .is_empty()
        {
            score += 1.0;
        }

        // Data collection completeness
        max_score += 1.0;
        if !snapshot
            .data_collection_params
            .collection_methods
            .is_empty()
        {
            score += 1.0;
        }

        // Analysis pipeline completeness
        max_score += 1.0;
        if !snapshot.analysis_pipeline.primary_analyses.is_empty() {
            score += 1.0;
        }

        if max_score > 0.0 {
            score / max_score
        } else {
            0.0
        }
    }

    fn calculate_reproducibility_score(&self, snapshot: &ProtocolSnapshot) -> f64 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Randomization seed present
        max_score += 1.0;
        if snapshot.randomization_settings.seed.is_some() {
            score += 1.0;
        }

        // Detailed experimental parameters
        max_score += 1.0;
        if !snapshot.data_collection_params.instruments.is_empty() {
            score += 1.0;
        }

        // Complete analysis specification
        max_score += 1.0;
        if !snapshot.analysis_pipeline.preprocessing_steps.is_empty() {
            score += 1.0;
        }

        if max_score > 0.0 {
            score / max_score
        } else {
            0.0
        }
    }

    fn generate_latex_protocol(&self, snapshot: &ProtocolSnapshot) -> Result<String, String> {
        let mut latex = String::new();

        latex.push_str("\\documentclass{article}\n");
        latex.push_str("\\usepackage[utf8]{inputenc}\n");
        latex.push_str("\\usepackage{booktabs}\n");
        latex.push_str("\\usepackage{geometry}\n");
        latex.push_str("\\title{Experimental Protocol}\n");
        latex.push_str("\\begin{document}\n");
        latex.push_str("\\maketitle\n");

        // Experimental design section
        latex.push_str("\\section{Experimental Design}\n");
        latex.push_str(&format!(
            "\\textbf{{Design Type:}} {}\\\\\n",
            snapshot.experimental_design.design_type
        ));
        latex.push_str(&format!(
            "\\textbf{{Target Sample Size:}} {}\\\\\n",
            snapshot.experimental_design.sample_size.target_n
        ));

        // Factors
        latex.push_str("\\subsection{Factors}\n");
        for factor in &snapshot.experimental_design.factors {
            latex.push_str(&format!(
                "\\textbf{{{}}}: {} ({})\\\\\n",
                factor.name,
                factor.levels.join(", "),
                factor.factor_type
            ));
        }

        // Conditions
        latex.push_str("\\subsection{Conditions}\n");
        latex.push_str("\\begin{tabular}{lll}\n");
        latex.push_str("\\toprule\n");
        latex.push_str("Condition & Levels & Expected N \\\\\n");
        latex.push_str("\\midrule\n");
        for condition in &snapshot.experimental_design.conditions {
            latex.push_str(&format!(
                "{} & {} & {} \\\\\n",
                condition.condition_name,
                condition
                    .factor_levels
                    .values()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", "),
                condition.expected_n
            ));
        }
        latex.push_str("\\bottomrule\n");
        latex.push_str("\\end{tabular}\n");

        latex.push_str("\\end{document}\n");

        Ok(latex)
    }
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    JSON,
    YAML,
    PDF,
    LaTeX,
}

impl Default for ProtocolVersionManager {
    fn default() -> Self {
        Self::new("Default Repository".to_string(), "System".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version_manager_creation() {
        let manager =
            ProtocolVersionManager::new("Test Repository".to_string(), "test_user".to_string());

        assert_eq!(manager.repository.repository_name, "Test Repository");
        assert_eq!(manager.repository.metadata.created_by, "test_user");
        assert_eq!(manager.repository.active_branch, "main");
        assert!(manager.repository.branches.contains_key("main"));
    }

    #[test]
    fn test_create_version() {
        let mut manager =
            ProtocolVersionManager::new("Test Repository".to_string(), "test_user".to_string());

        let snapshot = create_test_snapshot();
        let result = manager.create_version(
            "protocol_1".to_string(),
            "test_author".to_string(),
            "Initial version".to_string(),
            snapshot,
        );

        assert!(result.is_ok());
        let version_hash = result.unwrap();
        assert!(!version_hash.is_empty());

        let history = manager.get_protocol_history("protocol_1");
        assert!(history.is_some());
        assert_eq!(history.unwrap().versions.len(), 1);
    }

    #[test]
    fn test_branch_operations() {
        let mut manager =
            ProtocolVersionManager::new("Test Repository".to_string(), "test_user".to_string());

        // Create branch
        let result = manager.create_branch(
            "feature_branch".to_string(),
            "initial".to_string(),
            "test_user".to_string(),
            "Feature development branch".to_string(),
        );
        assert!(result.is_ok());

        // Switch to branch
        let result = manager.checkout_branch("feature_branch".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.repository.active_branch, "feature_branch");
    }

    #[test]
    fn test_collaborator_management() {
        let mut manager =
            ProtocolVersionManager::new("Test Repository".to_string(), "test_user".to_string());

        let result = manager.add_collaborator(
            "user123".to_string(),
            "John Doe".to_string(),
            "john@example.com".to_string(),
            CollaboratorRole::Researcher,
            "University".to_string(),
        );

        assert!(result.is_ok());
        assert_eq!(
            manager
                .repository
                .collaboration_settings
                .collaborators
                .len(),
            1
        );

        let collaborator = &manager.repository.collaboration_settings.collaborators[0];
        assert_eq!(collaborator.name, "John Doe");
        assert!(collaborator.permissions.contains(&Permission::Read));
        assert!(collaborator.permissions.contains(&Permission::Write));
    }

    #[test]
    fn test_version_validation() {
        let manager =
            ProtocolVersionManager::new("Test Repository".to_string(), "test_user".to_string());

        let snapshot = create_test_snapshot();
        let validation = manager.validate_protocol(&snapshot);

        assert!(validation.is_ok());
        let results = validation.unwrap();
        assert!(results.is_valid);
        assert!(results.completeness_score > 0.0);
        assert!(results.reproducibility_score > 0.0);
    }

    fn create_test_snapshot() -> ProtocolSnapshot {
        ProtocolSnapshot {
            experimental_design: ExperimentalDesignSnapshot {
                design_type: "Between-subjects".to_string(),
                factors: vec![FactorSnapshot {
                    name: "Condition".to_string(),
                    factor_type: "between".to_string(),
                    levels: vec!["Control".to_string(), "Treatment".to_string()],
                    manipulation_details: "Text manipulation".to_string(),
                }],
                conditions: vec![ConditionSnapshot {
                    condition_id: "control".to_string(),
                    condition_name: "Control".to_string(),
                    factor_levels: [("Condition".to_string(), "Control".to_string())]
                        .iter()
                        .cloned()
                        .collect(),
                    expected_n: 50,
                    materials: vec!["control_text.txt".to_string()],
                }],
                counterbalancing: CounterbalancingSnapshot {
                    method: "randomized".to_string(),
                    constraints: Vec::new(),
                    balancing_factors: Vec::new(),
                },
                sample_size: SampleSizeSnapshot {
                    target_n: 100,
                    power_analysis: PowerAnalysisSnapshot {
                        effect_size: 0.5,
                        alpha: 0.05,
                        power: 0.8,
                        test_type: "t-test".to_string(),
                        assumptions: vec!["normality".to_string()],
                    },
                    recruitment_plan: "Online recruitment".to_string(),
                    attrition_expectations: 0.1,
                },
                inclusion_criteria: vec!["Age 18-65".to_string()],
                exclusion_criteria: vec!["Non-native speakers".to_string()],
            },
            randomization_settings: RandomizationSnapshot {
                randomization_type: "simple".to_string(),
                seed: Some(12345),
                block_size: None,
                stratification_factors: Vec::new(),
                allocation_ratio: vec![0.5, 0.5],
            },
            data_collection_params: DataCollectionSnapshot {
                collection_methods: vec!["Online survey".to_string()],
                instruments: vec![InstrumentSnapshot {
                    instrument_name: "Questionnaire".to_string(),
                    version: "1.0".to_string(),
                    parameters: HashMap::new(),
                    validation_status: "Validated".to_string(),
                }],
                timing: TimingSnapshot {
                    session_duration: "30 minutes".to_string(),
                    break_intervals: Vec::new(),
                    time_limits: HashMap::new(),
                },
                environment_controls: vec!["Quiet environment".to_string()],
                quality_checks: vec!["Attention checks".to_string()],
            },
            analysis_pipeline: AnalysisPipelineSnapshot {
                primary_analyses: vec![AnalysisSnapshot {
                    analysis_name: "Main effect test".to_string(),
                    method: "Independent t-test".to_string(),
                    dependent_variables: vec!["Response".to_string()],
                    independent_variables: vec!["Condition".to_string()],
                    covariates: Vec::new(),
                    assumptions: vec!["normality", "homogeneity"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    interpretation_guidelines: "p < 0.05 for significance".to_string(),
                }],
                secondary_analyses: Vec::new(),
                exploratory_analyses: Vec::new(),
                preprocessing_steps: vec!["Remove outliers".to_string()],
                statistical_software: vec!["R".to_string()],
            },
            materials: Vec::new(),
            ethical_approvals: Vec::new(),
        }
    }
}
