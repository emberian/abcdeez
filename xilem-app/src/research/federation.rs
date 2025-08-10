use crate::research::ExperimentType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Federated Data Collection System
/// Enables secure multi-institutional research collaboration
/// while maintaining data sovereignty and privacy

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationNetwork {
    pub network_id: String,
    pub local_node: FederationNode,
    pub peer_nodes: HashMap<String, FederationNode>,
    pub shared_protocols: HashMap<String, SharedProtocol>,
    pub active_studies: HashMap<String, FederatedStudy>,
    pub data_sharing_agreements: Vec<DataSharingAgreement>,
    pub encryption_keys: HashMap<String, String>, // Node ID -> Public Key
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationNode {
    pub node_id: String,
    pub institution_name: String,
    pub contact_info: InstitutionContact,
    pub capabilities: NodeCapabilities,
    pub trust_level: TrustLevel,
    pub data_policies: DataGovernancePolicy,
    pub api_endpoint: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionContact {
    pub primary_investigator: String,
    pub email: String,
    pub institution: String,
    pub department: String,
    pub irb_contact: String,
    pub data_protection_officer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub supported_experiments: Vec<ExperimentType>,
    pub max_participants: usize,
    pub available_sensors: Vec<String>,
    pub data_formats: Vec<DataFormat>,
    pub analysis_capabilities: Vec<AnalysisCapability>,
    pub compliance_standards: Vec<ComplianceStandard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    Untrusted,
    Provisional,     // New node, limited access
    Verified,        // IRB verified, standard access
    HighTrust,       // Long-term partner, full access
    InternalNetwork, // Same institution network
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGovernancePolicy {
    pub data_retention_months: u32,
    pub sharing_restrictions: Vec<SharingRestriction>,
    pub anonymization_level: AnonymizationLevel,
    pub geographic_restrictions: Vec<String>, // Country codes
    pub irb_approval_required: bool,
    pub audit_requirements: AuditRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharingRestriction {
    NoRawData,                    // Only aggregated data
    InstitutionOnly,              // Within institution network
    IRBApprovalRequired,          // Requires specific IRB approval
    TimeDelayed(u32),             // Delay in days before sharing
    GeographicLimit(Vec<String>), // Restricted to regions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnonymizationLevel {
    Identified,    // Contains PII
    Pseudonymized, // Reversible anonymization
    Anonymized,    // Irreversible anonymization
    Aggregate,     // Only summary statistics
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRequirement {
    pub logging_level: AuditLevel,
    pub retention_years: u32,
    pub external_audit: bool,
    pub real_time_monitoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLevel {
    Basic,    // Access logs only
    Standard, // Access + data operations
    Detailed, // Full audit trail
    Forensic, // Complete system state logging
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Maintenance,
    Restricted, // Limited functionality
    Suspended,  // Temporary suspension
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedProtocol {
    pub protocol_id: String,
    pub name: String,
    pub version: String,
    pub created_by: String, // Node ID
    pub experiment_template: ExperimentTemplate,
    pub data_schema: DataSchema,
    pub quality_requirements: QualityRequirements,
    pub participant_criteria: ParticipantCriteria,
    pub consent_requirements: ConsentRequirements,
    pub participating_nodes: HashSet<String>,
    pub approval_status: HashMap<String, ApprovalStatus>,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentTemplate {
    pub experiment_type: ExperimentType,
    pub conditions: Vec<ExperimentCondition>,
    pub min_trials: u32,
    pub max_duration_minutes: u32,
    pub randomization_scheme: RandomizationScheme,
    pub required_sensors: Vec<String>,
    pub data_collection_points: Vec<DataCollectionPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RandomizationScheme {
    SimpleRandomization,
    BlockRandomization(u32),
    StratifiedRandomization(Vec<String>),
    MinimizationRandomization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCollectionPoint {
    pub name: String,
    pub data_type: DataType,
    pub required: bool,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    ResponseTime,
    Accuracy,
    AudioRecording,
    SensorData(String),
    Questionnaire,
    BiometricData,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Range(f64, f64),
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    RequiredFields(Vec<String>),
    CustomValidator(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSchema {
    pub schema_version: String,
    pub field_definitions: HashMap<String, FieldDefinition>,
    pub required_fields: HashSet<String>,
    pub anonymization_rules: HashMap<String, AnonymizationRule>,
    pub validation_schema: String, // JSON Schema
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub field_type: String,
    pub description: String,
    pub constraints: Vec<String>,
    pub privacy_level: PrivacyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Public,       // Can be shared freely
    Restricted,   // Requires agreement
    Sensitive,    // High protection required
    Confidential, // Maximum protection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnonymizationRule {
    Remove,               // Delete field
    Hash,                 // One-way hash
    Generalize(String),   // e.g., age to age range
    Noise(f64),           // Add statistical noise
    Pseudonymize(String), // Replace with pseudonym
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub min_response_time_ms: u128,
    pub max_response_time_ms: u128,
    pub min_accuracy_threshold: f64,
    pub required_completion_rate: f64,
    pub data_completeness_threshold: f64,
    pub technical_requirements: Vec<TechnicalRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalRequirement {
    pub requirement_type: String,
    pub specification: String,
    pub mandatory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantCriteria {
    pub inclusion_criteria: Vec<String>,
    pub exclusion_criteria: Vec<String>,
    pub min_age: u32,
    pub max_age: Option<u32>,
    pub required_capabilities: Vec<String>,
    pub screening_questions: Vec<ScreeningQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreeningQuestion {
    pub question: String,
    pub question_type: QuestionType,
    pub required_answer: Option<String>,
    pub exclusionary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionType {
    YesNo,
    MultipleChoice(Vec<String>),
    Numeric,
    Text,
    Scale(u32, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRequirements {
    pub consent_version: String,
    pub required_consents: Vec<ConsentType>,
    pub withdrawal_policy: WithdrawalPolicy,
    pub data_usage_permissions: Vec<DataUsagePermission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsentType {
    General,         // General research participation
    DataSharing,     // Cross-institutional sharing
    AudioRecording,  // Audio data collection
    SensorData,      // Physiological sensors
    LongTermStorage, // Extended data retention
    CommercialUse,   // Commercial research use
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawalPolicy {
    pub withdrawal_allowed: bool,
    pub data_deletion_timeframe_days: u32,
    pub partial_withdrawal_options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataUsagePermission {
    ResearchOnly,
    AcademicPublications,
    ConferencePresentation,
    GovernmentReporting,
    CommercialPartnership,
    OpenDataSharing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    ConditionalApproval(Vec<String>),
    Rejected(String),
    UnderReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedStudy {
    pub study_id: String,
    pub protocol_id: String,
    pub name: String,
    pub principal_investigator: String,
    pub coordinating_institution: String,
    pub participating_nodes: HashMap<String, ParticipationStatus>,
    pub target_participants: HashMap<String, u32>, // Node ID -> Target N
    pub current_enrollment: HashMap<String, u32>,  // Node ID -> Current N
    pub study_status: StudyStatus,
    pub start_date: DateTime<Utc>,
    pub planned_end_date: DateTime<Utc>,
    pub interim_analyses: Vec<InterimAnalysis>,
    pub data_monitoring_committee: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationStatus {
    pub status: ParticipationLevel,
    pub enrollment_start: DateTime<Utc>,
    pub enrollment_end: Option<DateTime<Utc>>,
    pub irb_approval_date: Option<DateTime<Utc>>,
    pub data_sharing_approval: bool,
    pub quality_metrics: QualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipationLevel {
    Observer,        // Can view protocols, no data sharing
    DataContributor, // Contributes data, receives summaries
    FullParticipant, // Full data sharing and analysis access
    Coordinator,     // Administrative access
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StudyStatus {
    Planning,
    IRBReview,
    Recruiting,
    DataCollection,
    DataAnalysis,
    WriteUp,
    Completed,
    Suspended,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterimAnalysis {
    pub analysis_id: String,
    pub scheduled_date: DateTime<Utc>,
    pub completed_date: Option<DateTime<Utc>>,
    pub findings_summary: Option<String>,
    pub recommendations: Vec<String>,
    pub study_continuation: ContinuationDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContinuationDecision {
    Continue,
    ModifyProtocol(Vec<String>),
    EarlyTermination(String),
    ExtendEnrollment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub data_completeness: f64,
    pub protocol_adherence: f64,
    pub participant_retention: f64,
    pub data_quality_score: f64,
    pub technical_issues: Vec<TechnicalIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIssue {
    pub issue_type: String,
    pub description: String,
    pub frequency: u32,
    pub severity: Severity,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSharingAgreement {
    pub agreement_id: String,
    pub parties: Vec<String>, // Node IDs
    pub study_ids: Vec<String>,
    pub data_types: Vec<DataType>,
    pub sharing_conditions: Vec<SharingCondition>,
    pub effective_date: DateTime<Utc>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub signed_by: HashMap<String, DateTime<Utc>>, // Node ID -> Signature date
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingCondition {
    pub condition_type: ConditionType,
    pub description: String,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    IRBApproval,
    AnonymizationRequired,
    AggregationOnly,
    TimeDelay(u32),
    PurposeLimitation(Vec<String>),
    GeographicRestriction(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    JSON,
    CSV,
    HDF5,
    Parquet,
    BIDS,    // Brain Imaging Data Structure
    HL7FHIR, // Healthcare interoperability
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisCapability {
    DescriptiveStatistics,
    InferentialStatistics,
    MachineLearning,
    BayesianAnalysis,
    MetaAnalysis,
    NetworkAnalysis,
    TimeSeriesAnalysis,
    SurvivalAnalysis,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComplianceStandard {
    GDPR,       // EU General Data Protection Regulation
    HIPAA,      // US Health Insurance Portability
    FERPA,      // US Family Educational Rights
    CommonRule, // US Federal Research Regulations
    ISO27001,   // Information Security Management
    SOC2,       // Security and Availability
    Custom(String),
}

impl FederationNetwork {
    pub fn new(institution_name: String, contact: InstitutionContact) -> Self {
        let node_id = Uuid::new_v4().to_string();
        let local_node = FederationNode {
            node_id: node_id.clone(),
            institution_name,
            contact_info: contact,
            capabilities: NodeCapabilities::default(),
            trust_level: TrustLevel::HighTrust,
            data_policies: DataGovernancePolicy::default(),
            api_endpoint: "http://localhost:8080".to_string(),
            last_sync: None,
            status: NodeStatus::Online,
        };

        Self {
            network_id: Uuid::new_v4().to_string(),
            local_node,
            peer_nodes: HashMap::new(),
            shared_protocols: HashMap::new(),
            active_studies: HashMap::new(),
            data_sharing_agreements: Vec::new(),
            encryption_keys: HashMap::new(),
        }
    }

    pub fn add_peer_node(&mut self, node: FederationNode) -> Result<(), String> {
        // Validate node before adding
        if node.node_id.is_empty() {
            return Err("Node ID cannot be empty".to_string());
        }

        if self.peer_nodes.contains_key(&node.node_id) {
            return Err("Node already exists in network".to_string());
        }

        // Check trust level and capabilities
        match node.trust_level {
            TrustLevel::Untrusted => {
                return Err("Cannot add untrusted nodes directly".to_string());
            }
            _ => {}
        }

        self.peer_nodes.insert(node.node_id.clone(), node);
        Ok(())
    }

    pub fn create_shared_protocol(
        &mut self,
        name: String,
        experiment_template: ExperimentTemplate,
        data_schema: DataSchema,
    ) -> Result<String, String> {
        let protocol_id = Uuid::new_v4().to_string();

        let protocol = SharedProtocol {
            protocol_id: protocol_id.clone(),
            name,
            version: "1.0".to_string(),
            created_by: self.local_node.node_id.clone(),
            experiment_template,
            data_schema,
            quality_requirements: QualityRequirements::default(),
            participant_criteria: ParticipantCriteria::default(),
            consent_requirements: ConsentRequirements::default(),
            participating_nodes: HashSet::new(),
            approval_status: HashMap::new(),
            created_at: Utc::now(),
            last_modified: Utc::now(),
        };

        self.shared_protocols.insert(protocol_id.clone(), protocol);
        Ok(protocol_id)
    }

    pub fn join_protocol(&mut self, protocol_id: &str) -> Result<(), String> {
        // First, validate the protocol (using immutable borrow)
        {
            let protocol = self
                .shared_protocols
                .get(protocol_id)
                .ok_or("Protocol not found")?;

            if protocol
                .participating_nodes
                .contains(&self.local_node.node_id)
            {
                return Err("Already participating in this protocol".to_string());
            }

            // Check capabilities against requirements
            self.validate_protocol_compatibility(&protocol)?;
        }

        // Now get mutable access to update the protocol
        let protocol = self
            .shared_protocols
            .get_mut(protocol_id)
            .ok_or("Protocol not found")?;

        protocol
            .participating_nodes
            .insert(self.local_node.node_id.clone());
        protocol
            .approval_status
            .insert(self.local_node.node_id.clone(), ApprovalStatus::Pending);

        Ok(())
    }

    pub fn initiate_federated_study(
        &mut self,
        protocol_id: String,
        name: String,
        target_participants: HashMap<String, u32>,
    ) -> Result<String, String> {
        if !self.shared_protocols.contains_key(&protocol_id) {
            return Err("Protocol not found".to_string());
        }

        let study_id = Uuid::new_v4().to_string();
        let study = FederatedStudy {
            study_id: study_id.clone(),
            protocol_id,
            name,
            principal_investigator: self.local_node.contact_info.primary_investigator.clone(),
            coordinating_institution: self.local_node.institution_name.clone(),
            participating_nodes: HashMap::new(),
            target_participants,
            current_enrollment: HashMap::new(),
            study_status: StudyStatus::Planning,
            start_date: Utc::now(),
            planned_end_date: Utc::now() + chrono::Duration::days(365),
            interim_analyses: Vec::new(),
            data_monitoring_committee: Vec::new(),
        };

        self.active_studies.insert(study_id.clone(), study);
        Ok(study_id)
    }

    pub fn get_federation_summary(&self) -> FederationSummary {
        FederationSummary {
            network_id: self.network_id.clone(),
            total_nodes: self.peer_nodes.len() + 1,
            online_nodes: self
                .peer_nodes
                .values()
                .filter(|node| matches!(node.status, NodeStatus::Online))
                .count()
                + 1,
            active_protocols: self.shared_protocols.len(),
            active_studies: self.active_studies.len(),
            total_participants: self.calculate_total_participants(),
            compliance_status: self.check_compliance_status(),
        }
    }

    fn validate_protocol_compatibility(&self, protocol: &SharedProtocol) -> Result<(), String> {
        let capabilities = &self.local_node.capabilities;

        // Check experiment type support
        if !capabilities
            .supported_experiments
            .contains(&protocol.experiment_template.experiment_type)
        {
            return Err("Experiment type not supported".to_string());
        }

        // Check sensor requirements
        for required_sensor in &protocol.experiment_template.required_sensors {
            if !capabilities.available_sensors.contains(required_sensor) {
                return Err(format!(
                    "Required sensor not available: {}",
                    required_sensor
                ));
            }
        }

        Ok(())
    }

    fn calculate_total_participants(&self) -> u32 {
        self.active_studies
            .values()
            .flat_map(|study| study.current_enrollment.values())
            .sum()
    }

    fn check_compliance_status(&self) -> ComplianceStatus {
        // Check if all required compliance standards are met
        let required_standards = vec![ComplianceStandard::CommonRule, ComplianceStandard::ISO27001];

        let has_all_standards = required_standards.iter().all(|standard| {
            self.local_node
                .capabilities
                .compliance_standards
                .contains(standard)
        });

        if has_all_standards {
            ComplianceStatus::Compliant
        } else {
            ComplianceStatus::NonCompliant
        }
    }
}

impl Default for NodeCapabilities {
    fn default() -> Self {
        Self {
            supported_experiments: vec![
                ExperimentType::LearningCurve,
                ExperimentType::RetentionTest,
            ],
            max_participants: 1000,
            available_sensors: vec!["audio".to_string(), "response_time".to_string()],
            data_formats: vec![DataFormat::JSON, DataFormat::CSV],
            analysis_capabilities: vec![
                AnalysisCapability::DescriptiveStatistics,
                AnalysisCapability::InferentialStatistics,
            ],
            compliance_standards: vec![ComplianceStandard::CommonRule],
        }
    }
}

impl Default for DataGovernancePolicy {
    fn default() -> Self {
        Self {
            data_retention_months: 84, // 7 years
            sharing_restrictions: vec![SharingRestriction::IRBApprovalRequired],
            anonymization_level: AnonymizationLevel::Pseudonymized,
            geographic_restrictions: Vec::new(),
            irb_approval_required: true,
            audit_requirements: AuditRequirement {
                logging_level: AuditLevel::Standard,
                retention_years: 7,
                external_audit: false,
                real_time_monitoring: true,
            },
        }
    }
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            min_response_time_ms: 100,
            max_response_time_ms: 10000,
            min_accuracy_threshold: 0.5,
            required_completion_rate: 0.8,
            data_completeness_threshold: 0.9,
            technical_requirements: Vec::new(),
        }
    }
}

impl Default for ParticipantCriteria {
    fn default() -> Self {
        Self {
            inclusion_criteria: vec!["Age 18 or older".to_string()],
            exclusion_criteria: vec!["Cognitive impairment".to_string()],
            min_age: 18,
            max_age: None,
            required_capabilities: vec!["Computer literacy".to_string()],
            screening_questions: Vec::new(),
        }
    }
}

impl Default for ConsentRequirements {
    fn default() -> Self {
        Self {
            consent_version: "1.0".to_string(),
            required_consents: vec![ConsentType::General, ConsentType::DataSharing],
            withdrawal_policy: WithdrawalPolicy {
                withdrawal_allowed: true,
                data_deletion_timeframe_days: 30,
                partial_withdrawal_options: vec!["Data sharing only".to_string()],
            },
            data_usage_permissions: vec![
                DataUsagePermission::ResearchOnly,
                DataUsagePermission::AcademicPublications,
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationSummary {
    pub network_id: String,
    pub total_nodes: usize,
    pub online_nodes: usize,
    pub active_protocols: usize,
    pub active_studies: usize,
    pub total_participants: u32,
    pub compliance_status: ComplianceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    UnderReview,
}
