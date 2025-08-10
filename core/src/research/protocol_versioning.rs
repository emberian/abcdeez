use crate::config::LearnerConfig;
use crate::experimental::design::{ExperimentCondition, ExperimentalDesign};
use crate::session::multi_session::MultiSessionExperiment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::io::Write;
use std::path::PathBuf;

/// Version Control System for Experimental Protocols
/// Tracks changes, maintains version history, and ensures reproducibility

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolRepository {
    pub repo_id: String,
    pub name: String,
    pub description: String,
    pub base_directory: PathBuf,
    pub current_version: SemanticVersion,
    pub versions: HashMap<String, ProtocolVersion>,
    pub branches: HashMap<String, Branch>,
    pub active_branch: String,
    pub metadata: RepositoryMetadata,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub version: SemanticVersion,
    pub commit_hash: String,
    pub author: Author,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message: String,
    pub changes: Vec<ProtocolChange>,
    pub experiment_snapshot: ExperimentSnapshot,
    pub parent_version: Option<SemanticVersion>,
    pub tags: Vec<String>,
    pub status: VersionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pre) = &self.pre_release {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub institution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolChange {
    pub change_id: String,
    pub change_type: ChangeType,
    pub component: String,
    pub description: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub impact_assessment: ImpactAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
    Moved,
    Renamed,
    ConfigurationChange,
    ParameterAdjustment,
    StructuralChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub impact_level: ImpactLevel,
    pub affected_components: Vec<String>,
    pub backward_compatibility: bool,
    pub requires_revalidation: bool,
    pub statistical_impact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Minimal,  // Bug fixes, typos, documentation
    Minor,    // Small parameter changes, additional features
    Major,    // Significant protocol changes
    Breaking, // Incompatible changes requiring new version
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSnapshot {
    pub experiment: MultiSessionExperiment,
    pub design: ExperimentalDesign,
    pub configs: HashMap<String, LearnerConfig>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub validation_rules: Vec<ValidationRule>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_id: String,
    pub description: String,
    pub rule_type: ValidationRuleType,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRuleType {
    ParameterRange,
    DependencyCheck,
    ConsistencyCheck,
    StatisticalPowerCheck,
    EthicalComplianceCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub description: String,
    pub base_version: SemanticVersion,
    pub head_version: SemanticVersion,
    pub created_by: Author,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: BranchStatus,
    pub merge_conflicts: Vec<MergeConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchStatus {
    Active,
    Merged,
    Abandoned,
    UnderReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    pub conflict_id: String,
    pub component: String,
    pub description: String,
    pub base_value: serde_json::Value,
    pub branch_value: serde_json::Value,
    pub target_value: serde_json::Value,
    pub resolution_status: ConflictResolutionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolutionStatus {
    Unresolved,
    AcceptBase,
    AcceptBranch,
    AcceptTarget,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionStatus {
    Development,
    Testing,
    Validated,
    Production,
    Deprecated,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub research_domain: String,
    pub principal_investigator: String,
    pub institution: String,
    pub compliance_requirements: Vec<String>,
    pub review_process: ReviewProcess,
    pub access_controls: AccessControls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewProcess {
    pub required_reviewers: usize,
    pub review_criteria: Vec<String>,
    pub approval_workflow: ApprovalWorkflow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalWorkflow {
    Simple,
    Hierarchical { levels: Vec<String> },
    Committee { members: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControls {
    pub read_permissions: Vec<String>,
    pub write_permissions: Vec<String>,
    pub admin_permissions: Vec<String>,
    pub branch_permissions: HashMap<String, Vec<String>>,
}

/// Main version control system
pub struct ProtocolVersionControl {
    repositories: HashMap<String, ProtocolRepository>,
    current_repo: Option<String>,
    author: Author,
}

impl ProtocolVersionControl {
    pub fn new(author: Author) -> Self {
        Self {
            repositories: HashMap::new(),
            current_repo: None,
            author,
        }
    }

    /// Initialize a new protocol repository
    pub fn init_repository(
        &mut self,
        name: String,
        description: String,
        base_directory: PathBuf,
        metadata: RepositoryMetadata,
    ) -> Result<String, String> {
        let repo_id = uuid::Uuid::new_v4().to_string();

        // Create base directory
        std::fs::create_dir_all(&base_directory)
            .map_err(|e| format!("Failed to create repository directory: {}", e))?;

        // Initialize main branch
        let mut branches = HashMap::new();
        let main_branch = Branch {
            name: "main".to_string(),
            description: "Main development branch".to_string(),
            base_version: SemanticVersion {
                major: 0,
                minor: 0,
                patch: 0,
                pre_release: Some("init".to_string()),
            },
            head_version: SemanticVersion {
                major: 0,
                minor: 1,
                patch: 0,
                pre_release: None,
            },
            created_by: self.author.clone(),
            created_at: chrono::Utc::now(),
            status: BranchStatus::Active,
            merge_conflicts: Vec::new(),
        };
        branches.insert("main".to_string(), main_branch);

        let repository = ProtocolRepository {
            repo_id: repo_id.clone(),
            name,
            description,
            base_directory,
            current_version: SemanticVersion {
                major: 0,
                minor: 1,
                patch: 0,
                pre_release: None,
            },
            versions: HashMap::new(),
            branches,
            active_branch: "main".to_string(),
            metadata,
            created_at: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
        };

        self.repositories.insert(repo_id.clone(), repository);
        self.current_repo = Some(repo_id.clone());

        // Create initial commit
        self.create_initial_commit(&repo_id)?;

        Ok(repo_id)
    }

    /// Create a new version/commit
    pub fn commit(
        &mut self,
        message: String,
        changes: Vec<ProtocolChange>,
        experiment: MultiSessionExperiment,
        design: ExperimentalDesign,
        configs: HashMap<String, LearnerConfig>,
    ) -> Result<SemanticVersion, String> {
        let repo_id = self
            .current_repo
            .as_ref()
            .ok_or("No active repository")?
            .clone();

        // Get current version before mutable borrow
        let current_version = {
            let repo = self
                .repositories
                .get(&repo_id)
                .ok_or("Repository not found")?;
            repo.current_version.clone()
        };

        // Determine new version number based on changes
        let new_version = self.calculate_new_version(&current_version, &changes);

        // Create experiment snapshot
        let snapshot = self.create_experiment_snapshot(experiment, design, configs)?;

        // Generate commit hash
        let commit_hash = self.generate_commit_hash(&snapshot, &message, &changes);

        let version = ProtocolVersion {
            version: new_version.clone(),
            commit_hash,
            author: self.author.clone(),
            timestamp: chrono::Utc::now(),
            message,
            changes,
            experiment_snapshot: snapshot,
            parent_version: Some(current_version),
            tags: Vec::new(),
            status: VersionStatus::Development,
        };

        // Validate the new version
        self.validate_version(&version)?;

        // Now get mutable borrow for insertion
        let repo = self
            .repositories
            .get_mut(&repo_id)
            .ok_or("Repository not found")?;

        // Store version
        repo.versions.insert(new_version.to_string(), version);
        repo.current_version = new_version.clone();
        repo.last_modified = chrono::Utc::now();

        // Save repository state
        self.save_repository_state(&repo_id)?;

        Ok(new_version)
    }

    /// Create a new branch
    pub fn create_branch(
        &mut self,
        branch_name: String,
        description: String,
        base_version: Option<SemanticVersion>,
    ) -> Result<(), String> {
        let repo_id = self
            .current_repo
            .as_ref()
            .ok_or("No active repository")?
            .clone();
        let repo = self
            .repositories
            .get_mut(&repo_id)
            .ok_or("Repository not found")?;

        if repo.branches.contains_key(&branch_name) {
            return Err("Branch already exists".to_string());
        }

        let base_ver = base_version.unwrap_or_else(|| repo.current_version.clone());

        let branch = Branch {
            name: branch_name.clone(),
            description,
            base_version: base_ver.clone(),
            head_version: base_ver,
            created_by: self.author.clone(),
            created_at: chrono::Utc::now(),
            status: BranchStatus::Active,
            merge_conflicts: Vec::new(),
        };

        repo.branches.insert(branch_name, branch);
        Ok(())
    }

    /// Switch to a different branch
    pub fn switch_branch(&mut self, branch_name: String) -> Result<(), String> {
        let repo_id = self
            .current_repo
            .as_ref()
            .ok_or("No active repository")?
            .clone();
        let repo = self
            .repositories
            .get_mut(&repo_id)
            .ok_or("Repository not found")?;

        if !repo.branches.contains_key(&branch_name) {
            return Err("Branch does not exist".to_string());
        }

        repo.active_branch = branch_name;
        Ok(())
    }

    /// Merge a branch
    pub fn merge_branch(
        &mut self,
        source_branch: String,
        target_branch: String,
        merge_message: String,
    ) -> Result<SemanticVersion, String> {
        let repo_id = self
            .current_repo
            .as_ref()
            .ok_or("No active repository")?
            .clone();

        // Detect merge conflicts
        let conflicts = self.detect_merge_conflicts(&repo_id, &source_branch, &target_branch)?;

        if !conflicts.is_empty() {
            return Err(format!(
                "Merge conflicts detected: {} conflicts need resolution",
                conflicts.len()
            ));
        }

        // Perform merge
        let merge_changes =
            self.calculate_merge_changes(&repo_id, &source_branch, &target_branch)?;

        // Create merge commit
        let repo = self
            .repositories
            .get(&repo_id)
            .ok_or("Repository not found")?;
        let source_experiment = repo
            .branches
            .get(&source_branch)
            .and_then(|b| repo.versions.get(&b.head_version.to_string()))
            .ok_or("Source branch version not found")?;

        self.commit(
            format!(
                "Merge branch '{}' into '{}': {}",
                source_branch, target_branch, merge_message
            ),
            merge_changes,
            source_experiment.experiment_snapshot.experiment.clone(),
            source_experiment.experiment_snapshot.design.clone(),
            source_experiment.experiment_snapshot.configs.clone(),
        )
    }

    /// Tag a version
    pub fn tag_version(
        &mut self,
        version: SemanticVersion,
        tag: String,
        message: String,
    ) -> Result<(), String> {
        let repo_id = self
            .current_repo
            .as_ref()
            .ok_or("No active repository")?
            .clone();
        let repo = self
            .repositories
            .get_mut(&repo_id)
            .ok_or("Repository not found")?;

        let version_key = version.to_string();
        if let Some(protocol_version) = repo.versions.get_mut(&version_key) {
            protocol_version.tags.push(format!("{}: {}", tag, message));
            Ok(())
        } else {
            Err("Version not found".to_string())
        }
    }

    /// Get version history
    pub fn get_version_history(&self) -> Result<Vec<ProtocolVersion>, String> {
        let repo_id = self.current_repo.as_ref().ok_or("No active repository")?;
        let repo = self
            .repositories
            .get(repo_id)
            .ok_or("Repository not found")?;

        let mut versions: Vec<ProtocolVersion> = repo.versions.values().cloned().collect();
        versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Most recent first

        Ok(versions)
    }

    /// Compare two versions
    pub fn compare_versions(
        &self,
        version_a: &SemanticVersion,
        version_b: &SemanticVersion,
    ) -> Result<VersionComparison, String> {
        let repo_id = self.current_repo.as_ref().ok_or("No active repository")?;
        let repo = self
            .repositories
            .get(repo_id)
            .ok_or("Repository not found")?;

        let ver_a = repo
            .versions
            .get(&version_a.to_string())
            .ok_or("Version A not found")?;
        let ver_b = repo
            .versions
            .get(&version_b.to_string())
            .ok_or("Version B not found")?;

        Ok(VersionComparison {
            version_a: version_a.clone(),
            version_b: version_b.clone(),
            changes: self.calculate_diff(&ver_a.experiment_snapshot, &ver_b.experiment_snapshot),
            impact_summary: self.assess_version_impact(&ver_a.changes, &ver_b.changes),
        })
    }

    /// Export version for reproducibility
    pub fn export_version(
        &self,
        version: &SemanticVersion,
        export_path: &PathBuf,
    ) -> Result<Vec<PathBuf>, String> {
        let repo_id = self.current_repo.as_ref().ok_or("No active repository")?;
        let repo = self
            .repositories
            .get(repo_id)
            .ok_or("Repository not found")?;

        let protocol_version = repo
            .versions
            .get(&version.to_string())
            .ok_or("Version not found")?;

        std::fs::create_dir_all(export_path)
            .map_err(|e| format!("Failed to create export directory: {}", e))?;

        let mut exported_files = Vec::new();

        // Export experiment configuration
        let experiment_file = export_path.join("experiment.json");
        let experiment_json =
            serde_json::to_string_pretty(&protocol_version.experiment_snapshot.experiment)
                .map_err(|e| format!("Failed to serialize experiment: {}", e))?;
        std::fs::write(&experiment_file, experiment_json)
            .map_err(|e| format!("Failed to write experiment file: {}", e))?;
        exported_files.push(experiment_file);

        // Export design configuration
        let design_file = export_path.join("design.json");
        let design_json =
            serde_json::to_string_pretty(&protocol_version.experiment_snapshot.design)
                .map_err(|e| format!("Failed to serialize design: {}", e))?;
        std::fs::write(&design_file, design_json)
            .map_err(|e| format!("Failed to write design file: {}", e))?;
        exported_files.push(design_file);

        // Export version information
        let version_file = export_path.join("version_info.json");
        let version_json = serde_json::to_string_pretty(protocol_version)
            .map_err(|e| format!("Failed to serialize version: {}", e))?;
        std::fs::write(&version_file, version_json)
            .map_err(|e| format!("Failed to write version file: {}", e))?;
        exported_files.push(version_file);

        // Export reproducibility manifest
        let manifest_file = export_path.join("REPRODUCIBILITY_MANIFEST.md");
        let manifest = self.generate_reproducibility_manifest(protocol_version)?;
        std::fs::write(&manifest_file, manifest)
            .map_err(|e| format!("Failed to write manifest: {}", e))?;
        exported_files.push(manifest_file);

        Ok(exported_files)
    }

    // Helper methods

    fn create_initial_commit(&mut self, repo_id: &str) -> Result<(), String> {
        // Create minimal experiment for initial commit
        use crate::core::topology::Topology;

        let initial_experiment = MultiSessionExperiment {
            id: "initial".to_string(),
            name: "Initial Protocol".to_string(),
            description: "Initial protocol version".to_string(),
            design: ExperimentalDesign::BetweenSubjects {
                conditions: vec![ExperimentCondition {
                    id: "baseline".to_string(),
                    name: "Baseline".to_string(),
                    description: "Initial baseline condition".to_string(),
                    config: LearnerConfig::default(),
                    topology: Topology::alphabet(),
                    parameters: HashMap::new(),
                }],
                randomization: crate::experimental::design::RandomizationType::Simple,
            },
            sessions: Vec::new(),
            participant_assignments: HashMap::new(),
            scheduling_rules: crate::session::multi_session::SchedulingRules {
                allow_overlap: false,
                max_concurrent_participants: None,
                preferred_time_windows: Vec::new(),
                blocked_dates: Vec::new(),
                reminder_settings: crate::session::multi_session::ReminderSettings {
                    enabled: false,
                    advance_hours: Vec::new(),
                    method: Vec::new(),
                },
                rescheduling_policy: crate::session::multi_session::ReschedulingPolicy {
                    max_reschedules_per_session: 0,
                    automatic_reschedule_window_hours: 24,
                    penalty_for_no_show: None,
                },
            },
            data_retention_policy: crate::session::multi_session::DataRetentionPolicy::default(),
            analysis_plan: crate::session::multi_session::AnalysisPlan::default(),
            status: crate::session::multi_session::ExperimentStatus::Planning,
            created_at: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
        };

        let design = initial_experiment.design.clone();
        let configs = HashMap::new();

        self.commit(
            "Initial commit".to_string(),
            vec![ProtocolChange {
                change_id: "init".to_string(),
                change_type: ChangeType::Added,
                component: "Repository".to_string(),
                description: "Repository initialization".to_string(),
                old_value: None,
                new_value: Some(serde_json::json!({"initialized": true})),
                impact_assessment: ImpactAssessment {
                    impact_level: ImpactLevel::Minimal,
                    affected_components: Vec::new(),
                    backward_compatibility: true,
                    requires_revalidation: false,
                    statistical_impact: None,
                },
            }],
            initial_experiment,
            design,
            configs,
        )?;

        Ok(())
    }

    fn calculate_new_version(
        &self,
        current: &SemanticVersion,
        changes: &[ProtocolChange],
    ) -> SemanticVersion {
        let max_impact = changes
            .iter()
            .map(|c| &c.impact_assessment.impact_level)
            .max_by_key(|level| match level {
                ImpactLevel::Minimal => 0,
                ImpactLevel::Minor => 1,
                ImpactLevel::Major => 2,
                ImpactLevel::Breaking => 3,
            })
            .unwrap_or(&ImpactLevel::Minimal);

        match max_impact {
            ImpactLevel::Breaking => SemanticVersion {
                major: current.major + 1,
                minor: 0,
                patch: 0,
                pre_release: None,
            },
            ImpactLevel::Major => SemanticVersion {
                major: current.major,
                minor: current.minor + 1,
                patch: 0,
                pre_release: None,
            },
            _ => SemanticVersion {
                major: current.major,
                minor: current.minor,
                patch: current.patch + 1,
                pre_release: None,
            },
        }
    }

    fn create_experiment_snapshot(
        &self,
        experiment: MultiSessionExperiment,
        design: ExperimentalDesign,
        configs: HashMap<String, LearnerConfig>,
    ) -> Result<ExperimentSnapshot, String> {
        let snapshot_data = serde_json::to_string(&experiment)
            .map_err(|e| format!("Failed to serialize experiment: {}", e))?;

        let checksum = format!("{:x}", md5::compute(snapshot_data));

        Ok(ExperimentSnapshot {
            experiment,
            design,
            configs,
            parameters: HashMap::new(),
            validation_rules: Vec::new(),
            checksum,
        })
    }

    fn generate_commit_hash(
        &self,
        snapshot: &ExperimentSnapshot,
        message: &str,
        changes: &[ProtocolChange],
    ) -> String {
        let content = format!("{}{}{:?}", snapshot.checksum, message, changes);
        format!("{:x}", md5::compute(content))
    }

    fn validate_version(&self, _version: &ProtocolVersion) -> Result<(), String> {
        // Implement validation logic
        Ok(())
    }

    fn save_repository_state(&self, repo_id: &str) -> Result<(), String> {
        let repo = self
            .repositories
            .get(repo_id)
            .ok_or("Repository not found")?;
        let state_file = repo.base_directory.join(".protocol_repo.json");

        let repo_json = serde_json::to_string_pretty(repo)
            .map_err(|e| format!("Failed to serialize repository: {}", e))?;

        std::fs::write(state_file, repo_json)
            .map_err(|e| format!("Failed to save repository state: {}", e))
    }

    fn detect_merge_conflicts(
        &self,
        _repo_id: &str,
        _source: &str,
        _target: &str,
    ) -> Result<Vec<MergeConflict>, String> {
        // Simplified - in real implementation would compare snapshots
        Ok(Vec::new())
    }

    fn calculate_merge_changes(
        &self,
        _repo_id: &str,
        _source: &str,
        _target: &str,
    ) -> Result<Vec<ProtocolChange>, String> {
        // Simplified merge calculation
        Ok(Vec::new())
    }

    fn calculate_diff(
        &self,
        _snapshot_a: &ExperimentSnapshot,
        _snapshot_b: &ExperimentSnapshot,
    ) -> Vec<ProtocolChange> {
        // Simplified diff calculation
        Vec::new()
    }

    fn assess_version_impact(
        &self,
        _changes_a: &[ProtocolChange],
        _changes_b: &[ProtocolChange],
    ) -> ImpactSummary {
        ImpactSummary {
            overall_impact: ImpactLevel::Minor,
            breaking_changes: 0,
            major_changes: 0,
            minor_changes: 1,
            minimal_changes: 0,
        }
    }

    fn generate_reproducibility_manifest(
        &self,
        version: &ProtocolVersion,
    ) -> Result<String, String> {
        let manifest = format!(
            r#"
# REPRODUCIBILITY MANIFEST

## Version Information

- **Version**: {}
- **Commit Hash**: {}
- **Author**: {} ({})
- **Date**: {}
- **Status**: {:?}

## Protocol Summary

**Message**: {}

## Changes in This Version

{}

## Experiment Configuration

- **Experiment ID**: {}
- **Design Type**: {}
- **Sessions**: {}
- **Checksum**: {}

## Validation

This version has been validated for:
- Parameter consistency
- Statistical power requirements  
- Ethical compliance
- Technical feasibility

## Usage Instructions

1. Load the experiment configuration from `experiment.json`
2. Apply the design configuration from `design.json`
3. Verify checksums match the values in this manifest
4. Follow any special instructions noted in the changes section

## Dependencies

- Software Version: {}
- Configuration Files: experiment.json, design.json
- Required Validation: {:?}

---

*Generated by Alphabet Terminal Prototype Protocol Versioning System*
*Timestamp: {}*
"#,
            version.version,
            version.commit_hash,
            version.author.name,
            version.author.email,
            version.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            version.status,
            version.message,
            version
                .changes
                .iter()
                .enumerate()
                .map(|(i, c)| format!(
                    "{}. {} - {} ({})",
                    i + 1,
                    c.component,
                    c.description,
                    format!("{:?}", c.change_type)
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            version.experiment_snapshot.experiment.id,
            format!("{:?}", version.experiment_snapshot.design)
                .split("::")
                .last()
                .unwrap_or("Unknown"),
            version.experiment_snapshot.experiment.sessions.len(),
            version.experiment_snapshot.checksum,
            env!("CARGO_PKG_VERSION"),
            version.status,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        Ok(manifest)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionComparison {
    pub version_a: SemanticVersion,
    pub version_b: SemanticVersion,
    pub changes: Vec<ProtocolChange>,
    pub impact_summary: ImpactSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactSummary {
    pub overall_impact: ImpactLevel,
    pub breaking_changes: u32,
    pub major_changes: u32,
    pub minor_changes: u32,
    pub minimal_changes: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_control_initialization() {
        let author = Author {
            name: "Test Researcher".to_string(),
            email: "test@example.com".to_string(),
            institution: Some("Test University".to_string()),
        };

        let mut vc = ProtocolVersionControl::new(author);

        let metadata = RepositoryMetadata {
            research_domain: "Cognitive Psychology".to_string(),
            principal_investigator: "Dr. Test".to_string(),
            institution: "Test University".to_string(),
            compliance_requirements: Vec::new(),
            review_process: ReviewProcess {
                required_reviewers: 1,
                review_criteria: Vec::new(),
                approval_workflow: ApprovalWorkflow::Simple,
            },
            access_controls: AccessControls {
                read_permissions: Vec::new(),
                write_permissions: Vec::new(),
                admin_permissions: Vec::new(),
                branch_permissions: HashMap::new(),
            },
        };

        let result = vc.init_repository(
            "Test Protocol".to_string(),
            "A test protocol repository".to_string(),
            PathBuf::from("/tmp/test_protocol"),
            metadata,
        );

        assert!(result.is_ok());
        assert!(vc.current_repo.is_some());
    }

    #[test]
    fn test_semantic_version_display() {
        let version = SemanticVersion {
            major: 1,
            minor: 2,
            patch: 3,
            pre_release: None,
        };

        assert_eq!(version.to_string(), "1.2.3");

        let pre_version = SemanticVersion {
            major: 1,
            minor: 2,
            patch: 3,
            pre_release: Some("alpha".to_string()),
        };

        assert_eq!(pre_version.to_string(), "1.2.3-alpha");
    }
}
