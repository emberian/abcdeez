use crate::research::citations::CitationManager;
use crate::session::multi_session::MultiSessionExperiment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// IRB Compliance Documentation Generation System
/// Generates ethics approval documentation and compliance reports

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRBComplianceGenerator {
    pub institution_info: InstitutionInfo,
    pub principal_investigator: PrincipalInvestigator,
    pub compliance_settings: ComplianceSettings,
    pub risk_assessments: HashMap<String, RiskAssessment>,
    pub consent_templates: HashMap<String, ConsentTemplate>,
    pub data_protection_measures: DataProtectionPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionInfo {
    pub name: String,
    pub address: String,
    pub irb_contact: String,
    pub irb_phone: String,
    pub irb_email: String,
    pub federal_wide_assurance: Option<String>, // FWA number
    pub jurisdiction: Jurisdiction,
    pub policies: Vec<InstitutionalPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrincipalInvestigator {
    pub name: String,
    pub title: String,
    pub department: String,
    pub institution: String,
    pub email: String,
    pub phone: String,
    pub qualifications: Vec<String>,
    pub human_subjects_training: TrainingRecord,
    pub previous_irb_approvals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingRecord {
    pub program_name: String,
    pub completion_date: chrono::NaiveDate,
    pub expiration_date: Option<chrono::NaiveDate>,
    pub certificate_number: Option<String>,
    pub refresher_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Jurisdiction {
    US,     // United States - 45 CFR 46
    EU,     // European Union - GDPR
    Canada, // TCPS2
    UK,     // GCP-ICH
    Other {
        name: String,
        regulations: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalPolicy {
    pub policy_name: String,
    pub policy_number: String,
    pub effective_date: chrono::NaiveDate,
    pub applies_to_research: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSettings {
    pub requires_irb_approval: bool,
    pub exempt_categories: Vec<ExemptCategory>,
    pub expedited_categories: Vec<ExpeditedCategory>,
    pub vulnerable_populations: Vec<VulnerablePopulation>,
    pub international_sites: bool,
    pub data_sharing_planned: bool,
    pub long_term_storage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExemptCategory {
    EducationalSettings,       // 45 CFR 46.101(b)(1)
    EducationalTests,          // 45 CFR 46.101(b)(2)
    SurveyInterview,           // 45 CFR 46.101(b)(3)
    PublicBehaviorObservation, // 45 CFR 46.101(b)(4)
    PublicBenefitPrograms,     // 45 CFR 46.101(b)(5)
    FoodTasting,               // 45 CFR 46.101(b)(6)
    SecondaryResearch,         // 45 CFR 46.101(b)(7)
    BroadConsent,              // 45 CFR 46.101(b)(8)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpeditedCategory {
    MinimalRiskBehavioral, // Category 7
    DataRecords,           // Category 5
    VoiceRecordings,       // Category 7
    ModerateExercise,      // Category 4
    MaterialsCollection,   // Category 2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerablePopulation {
    Children,      // Subpart D
    Prisoners,     // Subpart C
    PregnantWomen, // Subpart B (historical)
    CognitiveImpairment,
    Economically,
    Educationally,
    Students,
    Employees,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub assessment_id: String,
    pub study_component: String,
    pub risk_category: RiskCategory,
    pub risk_level: RiskLevel,
    pub potential_harms: Vec<PotentialHarm>,
    pub mitigation_measures: Vec<MitigationMeasure>,
    pub monitoring_plan: MonitoringPlan,
    pub assessed_by: String,
    pub assessment_date: chrono::NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskCategory {
    Physical,
    Psychological,
    Social,
    Economic,
    Legal,
    Privacy,
    Confidentiality,
    Dignity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    NoMoreThanMinimal,
    MinimalRisk,
    GreaterThanMinimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotentialHarm {
    pub harm_type: String,
    pub description: String,
    pub probability: Probability,
    pub severity: Severity,
    pub affected_groups: Vec<String>,
    pub reversibility: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Probability {
    Unlikely, // < 10%
    Possible, // 10-50%
    Likely,   // 50-90%
    Certain,  // > 90%
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Negligible,   // No lasting impact
    Minor,        // Temporary discomfort
    Moderate,     // Lasting but manageable impact
    Major,        // Serious lasting impact
    Catastrophic, // Life-threatening or permanently disabling
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationMeasure {
    pub measure_type: MitigationType,
    pub description: String,
    pub implementation_timeline: String,
    pub responsible_person: String,
    pub effectiveness_rating: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MitigationType {
    Prevention,    // Prevent risk occurrence
    Detection,     // Early detection of problems
    Response,      // Immediate response procedures
    Recovery,      // Post-incident recovery
    Communication, // Communication protocols
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringPlan {
    pub monitoring_frequency: MonitoringFrequency,
    pub monitoring_methods: Vec<MonitoringMethod>,
    pub reporting_procedures: Vec<ReportingProcedure>,
    pub stopping_rules: Vec<StoppingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringFrequency {
    Continuous,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    AnnuallyOrLess,
    EventTriggered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringMethod {
    pub method_name: String,
    pub description: String,
    pub automated: bool,
    pub responsible_party: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingProcedure {
    pub event_type: String,
    pub reporting_timeline: String,
    pub recipients: Vec<String>,
    pub documentation_required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoppingRule {
    pub rule_name: String,
    pub trigger_condition: String,
    pub action_required: String,
    pub decision_authority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentTemplate {
    pub template_id: String,
    pub template_name: String,
    pub population_type: PopulationType,
    pub consent_elements: Vec<ConsentElement>,
    pub additional_elements: Vec<AdditionalElement>,
    pub language_versions: HashMap<String, String>, // language code -> file path
    pub reading_level: ReadingLevel,
    pub last_updated: chrono::NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PopulationType {
    Adults,
    Children,
    AdolescentsWithAssent,
    CognitivelyImpaired,
    NonEnglishSpeaking,
    Prisoners,
    Students,
    Employees,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentElement {
    pub element_type: ConsentElementType,
    pub required: bool,
    pub content: String,
    pub regulatory_basis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsentElementType {
    StudyPurpose,          // 45 CFR 46.116(b)(1)
    Procedures,            // 45 CFR 46.116(b)(1)
    Duration,              // 45 CFR 46.116(b)(1)
    RisksDiscomforts,      // 45 CFR 46.116(b)(2)
    Benefits,              // 45 CFR 46.116(b)(3)
    AlternativeProcedures, // 45 CFR 46.116(b)(4)
    Confidentiality,       // 45 CFR 46.116(b)(5)
    Compensation,          // 45 CFR 46.116(b)(6)
    MedicalTreatment,      // 45 CFR 46.116(b)(6)
    ContactInformation,    // 45 CFR 46.116(b)(7)
    VoluntaryNature,       // 45 CFR 46.116(b)(8)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalElement {
    pub element_name: String,
    pub content: String,
    pub justification: String,
    pub regulatory_basis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReadingLevel {
    EighthGrade,
    HighSchool,
    College,
    Graduate,
    Professional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProtectionPlan {
    pub data_classification: DataClassification,
    pub collection_methods: Vec<CollectionMethod>,
    pub storage_plan: StoragePlan,
    pub access_controls: AccessControls,
    pub sharing_plan: DataSharingPlan,
    pub retention_schedule: RetentionSchedule,
    pub destruction_procedures: DestructionProcedures,
    pub incident_response: IncidentResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMethod {
    pub method_name: String,
    pub data_types: Vec<String>,
    pub security_measures: Vec<String>,
    pub personnel_authorized: Vec<String>,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePlan {
    pub primary_storage: StorageLocation,
    pub backup_storage: Option<StorageLocation>,
    pub encryption_required: bool,
    pub encryption_standard: Option<String>,
    pub access_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLocation {
    pub location_type: LocationType,
    pub geographic_location: String,
    pub security_certifications: Vec<String>,
    pub provider_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocationType {
    InstitutionalServers,
    CloudProvider,
    LocalComputers,
    PortableDevices,
    PaperRecords,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControls {
    pub authentication_required: bool,
    pub authorization_levels: Vec<AuthorizationLevel>,
    pub audit_logging: bool,
    pub access_review_frequency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationLevel {
    pub role_name: String,
    pub permissions: Vec<Permission>,
    pub personnel_assigned: Vec<String>,
    pub training_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Delete,
    Export,
    Share,
    Administer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSharingPlan {
    pub sharing_permitted: bool,
    pub sharing_scope: SharingScope,
    pub sharing_conditions: Vec<String>,
    pub data_use_agreements_required: bool,
    pub embargo_period: Option<u32>, // months
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharingScope {
    NoSharing,
    InstitutionalOnly,
    CollaboratorsOnly,
    QualifiedResearchers,
    PublicRepository,
    OpenAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionSchedule {
    pub raw_data_years: u32,
    pub analyzed_data_years: u32,
    pub consent_forms_years: u32,
    pub regulatory_basis: String,
    pub review_triggers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestructionProcedures {
    pub electronic_data_method: String,
    pub physical_records_method: String,
    pub verification_required: bool,
    pub documentation_retained: bool,
    pub responsible_party: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentResponse {
    pub response_team: Vec<String>,
    pub notification_procedures: Vec<NotificationProcedure>,
    pub investigation_procedures: String,
    pub remediation_procedures: String,
    pub documentation_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationProcedure {
    pub incident_type: String,
    pub notification_timeline: String,
    pub recipients: Vec<String>,
    pub content_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRBApplication {
    pub application_id: String,
    pub study_title: String,
    pub principal_investigator: PrincipalInvestigator,
    pub study_summary: StudySummary,
    pub risk_benefit_analysis: RiskBenefitAnalysis,
    pub subject_selection: SubjectSelection,
    pub consent_process: ConsentProcess,
    pub data_management: DataManagement,
    pub monitoring_plan: StudyMonitoringPlan,
    pub supporting_documents: Vec<SupportingDocument>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudySummary {
    pub background_rationale: String,
    pub research_objectives: Vec<String>,
    pub study_design: String,
    pub methodology: String,
    pub statistical_analysis_plan: String,
    pub expected_duration: String,
    pub study_locations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskBenefitAnalysis {
    pub identified_risks: Vec<IdentifiedRisk>,
    pub risk_minimization: Vec<String>,
    pub anticipated_benefits: Vec<AnticipatedBenefit>,
    pub risk_benefit_ratio: RiskBenefitRatio,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedRisk {
    pub risk_type: String,
    pub description: String,
    pub probability: String,
    pub severity: String,
    pub affected_population: String,
    pub mitigation_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnticipatedBenefit {
    pub benefit_type: BenefitType,
    pub description: String,
    pub beneficiary: Beneficiary,
    pub likelihood: String,
    pub magnitude: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenefitType {
    DirectBenefit,     // To individual participants
    SocietalBenefit,   // To society/knowledge
    ScientificBenefit, // To scientific knowledge
    FutureBenefit,     // To future participants
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Beneficiary {
    IndividualParticipants,
    ParticipantPopulation,
    Society,
    Science,
    FuturePatients,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskBenefitRatio {
    Favorable,   // Benefits outweigh risks
    Acceptable,  // Risks acceptable given benefits
    Unfavorable, // Risks outweigh benefits
    Uncertain,   // Insufficient information
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectSelection {
    pub inclusion_criteria: Vec<String>,
    pub exclusion_criteria: Vec<String>,
    pub target_enrollment: u32,
    pub recruitment_methods: Vec<RecruitmentMethod>,
    pub vulnerable_populations: Vec<VulnerablePopulation>,
    pub population_justification: String,
    pub equitable_selection: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecruitmentMethod {
    pub method_name: String,
    pub description: String,
    pub materials: Vec<String>,
    pub personnel_involved: Vec<String>,
    pub coercion_safeguards: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentProcess {
    pub consent_required: bool,
    pub consent_type: ConsentType,
    pub consent_timing: String,
    pub consent_location: String,
    pub consent_personnel: Vec<String>,
    pub language_provisions: Vec<String>,
    pub capacity_assessment: bool,
    pub witness_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsentType {
    Written,
    Verbal,
    Electronic,
    Implied,
    BroadConsent,
    Waived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataManagement {
    pub data_collection_plan: String,
    pub data_security_measures: Vec<String>,
    pub data_sharing_plan: String,
    pub data_retention_plan: String,
    pub quality_assurance: Vec<String>,
    pub personnel_training: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyMonitoringPlan {
    pub monitoring_approach: MonitoringApproach,
    pub monitoring_personnel: Vec<String>,
    pub reporting_schedule: String,
    pub safety_monitoring: Vec<String>,
    pub data_monitoring_board: Option<DataMonitoringBoard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringApproach {
    SelfMonitoring,
    IndependentMonitoring,
    CentralizedMonitoring,
    RiskBasedMonitoring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMonitoringBoard {
    pub members: Vec<BoardMember>,
    pub charter: String,
    pub meeting_schedule: String,
    pub reporting_procedures: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardMember {
    pub name: String,
    pub qualifications: String,
    pub role: String,
    pub conflicts_of_interest: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportingDocument {
    pub document_type: DocumentType,
    pub document_name: String,
    pub version: String,
    pub date_created: chrono::NaiveDate,
    pub file_path: Option<PathBuf>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    ConsentForm,
    RecruitmentMaterials,
    Questionnaires,
    InterviewGuides,
    DataCollectionTools,
    ProtocolDeviationProcedures,
    AdverseEventReporting,
    QualificationsCurriculum,
    DataUseAgreement,
    CollaborationAgreement,
    RegulatoryCorrespondence,
}

impl IRBComplianceGenerator {
    pub fn new(institution: InstitutionInfo, pi: PrincipalInvestigator) -> Self {
        Self {
            institution_info: institution,
            principal_investigator: pi,
            compliance_settings: ComplianceSettings::default(),
            risk_assessments: HashMap::new(),
            consent_templates: HashMap::new(),
            data_protection_measures: DataProtectionPlan::default(),
        }
    }

    pub fn add_risk_assessment(&mut self, assessment: RiskAssessment) {
        self.risk_assessments
            .insert(assessment.assessment_id.clone(), assessment);
    }

    pub fn add_consent_template(&mut self, template: ConsentTemplate) {
        self.consent_templates
            .insert(template.template_id.clone(), template);
    }

    pub fn generate_irb_application(
        &self,
        experiment: &MultiSessionExperiment,
        citation_manager: &CitationManager,
    ) -> Result<IRBApplication, String> {
        let study_summary = self.generate_study_summary(experiment, citation_manager)?;
        let risk_benefit_analysis = self.generate_risk_benefit_analysis(experiment)?;
        let subject_selection = self.generate_subject_selection(experiment)?;
        let consent_process = self.generate_consent_process(experiment)?;
        let data_management = self.generate_data_management_section()?;
        let monitoring_plan = self.generate_monitoring_plan(experiment)?;
        let supporting_documents = self.identify_supporting_documents(experiment)?;

        Ok(IRBApplication {
            application_id: format!("IRB-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S")),
            study_title: experiment.name.clone(),
            principal_investigator: self.principal_investigator.clone(),
            study_summary,
            risk_benefit_analysis,
            subject_selection,
            consent_process,
            data_management,
            monitoring_plan,
            supporting_documents,
            generated_at: chrono::Utc::now(),
        })
    }

    pub fn export_irb_package(
        &self,
        application: &IRBApplication,
        output_directory: &PathBuf,
    ) -> std::io::Result<Vec<PathBuf>> {
        std::fs::create_dir_all(output_directory)?;
        let mut generated_files = Vec::new();

        // Generate main application document
        let application_path = output_directory.join("IRB_Application.md");
        let application_content = self.format_irb_application(application);
        let mut file = File::create(&application_path)?;
        file.write_all(application_content.as_bytes())?;
        generated_files.push(application_path);

        // Generate consent forms
        for (template_id, template) in &self.consent_templates {
            let consent_path = output_directory.join(format!("Consent_Form_{}.md", template_id));
            let consent_content = self.generate_consent_form(template, application);
            let mut file = File::create(&consent_path)?;
            file.write_all(consent_content.as_bytes())?;
            generated_files.push(consent_path);
        }

        // Generate risk assessment report
        let risk_path = output_directory.join("Risk_Assessment_Report.md");
        let risk_content = self.generate_risk_assessment_report(application);
        let mut file = File::create(&risk_path)?;
        file.write_all(risk_content.as_bytes())?;
        generated_files.push(risk_path);

        // Generate data management plan
        let dmp_path = output_directory.join("Data_Management_Plan.md");
        let dmp_content = self.generate_data_management_plan();
        let mut file = File::create(&dmp_path)?;
        file.write_all(dmp_content.as_bytes())?;
        generated_files.push(dmp_path);

        // Generate monitoring plan
        let monitoring_path = output_directory.join("Monitoring_Plan.md");
        let monitoring_content = self.format_monitoring_plan(&application.monitoring_plan);
        let mut file = File::create(&monitoring_path)?;
        file.write_all(monitoring_content.as_bytes())?;
        generated_files.push(monitoring_path);

        Ok(generated_files)
    }

    pub fn assess_compliance_status(
        &self,
        experiment: &MultiSessionExperiment,
    ) -> ComplianceStatus {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Check for required elements
        if experiment.participant_assignments.is_empty() {
            issues.push("No participant assignments defined".to_string());
        }

        if experiment.sessions.is_empty() {
            issues.push("No study sessions defined".to_string());
        }

        // Check risk assessments
        if self.risk_assessments.is_empty() {
            issues.push("No risk assessments completed".to_string());
            recommendations.push("Complete comprehensive risk assessment".to_string());
        }

        // Check consent templates
        if self.consent_templates.is_empty() {
            issues.push("No consent templates prepared".to_string());
            recommendations.push("Prepare appropriate consent forms".to_string());
        }

        // Check PI qualifications
        if self
            .principal_investigator
            .human_subjects_training
            .completion_date
            < chrono::Utc::now().naive_utc().date() - chrono::Duration::days(365)
        {
            issues.push("PI human subjects training may be expired".to_string());
            recommendations.push("Refresh human subjects protection training".to_string());
        }

        let overall_status = if issues.is_empty() {
            ComplianceLevel::Compliant
        } else if issues.len() < 3 {
            ComplianceLevel::MinorIssues
        } else {
            ComplianceLevel::MajorIssues
        };

        ComplianceStatus {
            overall_status,
            issues,
            recommendations,
            assessment_date: chrono::Utc::now().naive_utc().date(),
            next_review_date: chrono::Utc::now().naive_utc().date() + chrono::Duration::days(90),
        }
    }

    // Private helper methods
    fn generate_study_summary(
        &self,
        experiment: &MultiSessionExperiment,
        citation_manager: &CitationManager,
    ) -> Result<StudySummary, String> {
        let methodology_report = citation_manager.generate_methodology_report(&experiment.id);

        Ok(StudySummary {
            background_rationale: format!(
                "This study investigates learning processes using adaptive experimental methods. {}",
                experiment.description
            ),
            research_objectives: vec![
                "Measure learning progression over multiple sessions".to_string(),
                "Evaluate effectiveness of adaptive vs. fixed scheduling".to_string(),
                "Assess individual differences in learning strategies".to_string(),
            ],
            study_design: format!(
                "Multi-session experimental study with {} sessions using {:?} design",
                experiment.sessions.len(),
                experiment.design
            ),
            methodology: methodology_report.methods_used.iter()
                .map(|m| format!("{}: {}", m.method_name, m.description))
                .collect::<Vec<_>>()
                .join(". "),
            statistical_analysis_plan: "Mixed-effects modeling will be used to analyze repeated measures data, accounting for individual differences and session effects.".to_string(),
            expected_duration: format!("{} sessions over {} weeks", experiment.sessions.len(), experiment.sessions.len() / 7 + 1),
            study_locations: vec![self.institution_info.name.clone()],
        })
    }

    fn generate_risk_benefit_analysis(
        &self,
        experiment: &MultiSessionExperiment,
    ) -> Result<RiskBenefitAnalysis, String> {
        let mut identified_risks = Vec::new();

        // Analyze experiment-specific risks based on design
        self.analyze_experiment_risks(experiment, &mut identified_risks);

        // Analyze risks from assessments
        for assessment in self.risk_assessments.values() {
            for harm in &assessment.potential_harms {
                identified_risks.push(IdentifiedRisk {
                    risk_type: format!("{:?}", assessment.risk_category),
                    description: harm.description.clone(),
                    probability: format!("{:?}", harm.probability),
                    severity: format!("{:?}", harm.severity),
                    affected_population: harm.affected_groups.join(", "),
                    mitigation_strategy: assessment
                        .mitigation_measures
                        .iter()
                        .map(|m| m.description.clone())
                        .collect::<Vec<_>>()
                        .join("; "),
                });
            }
        }

        let anticipated_benefits = vec![
            AnticipatedBenefit {
                benefit_type: BenefitType::ScientificBenefit,
                description: "Advancement of understanding of human learning processes".to_string(),
                beneficiary: Beneficiary::Science,
                likelihood: "High".to_string(),
                magnitude: "Significant".to_string(),
            },
            AnticipatedBenefit {
                benefit_type: BenefitType::SocietalBenefit,
                description: "Improved educational methods and adaptive learning systems"
                    .to_string(),
                beneficiary: Beneficiary::Society,
                likelihood: "Moderate".to_string(),
                magnitude: "Moderate".to_string(),
            },
        ];

        Ok(RiskBenefitAnalysis {
            identified_risks,
            risk_minimization: vec![
                "Voluntary participation with right to withdraw".to_string(),
                "Confidential data handling procedures".to_string(),
                "Minimal psychological discomfort from learning tasks".to_string(),
            ],
            anticipated_benefits,
            risk_benefit_ratio: RiskBenefitRatio::Favorable,
            justification: "The study poses no more than minimal risk to participants while contributing valuable knowledge to learning science.".to_string(),
        })
    }

    /// Analyze experiment-specific risks based on the study design
    fn analyze_experiment_risks(
        &self,
        experiment: &MultiSessionExperiment,
        identified_risks: &mut Vec<IdentifiedRisk>,
    ) {
        // Analyze session duration risks
        for session in &experiment.sessions {
            if let Some(duration) = session.duration_minutes {
                if duration > 120 {
                    identified_risks.push(IdentifiedRisk {
                        risk_type: "Participant Fatigue".to_string(),
                        description: format!(
                            "Session {} duration of {} minutes may cause fatigue",
                            session.name, duration
                        ),
                        probability: "Low (30%)".to_string(),
                        severity: "Minimal".to_string(),
                        affected_population: "Participants in extended sessions".to_string(),
                        mitigation_strategy: "Include mandatory breaks and monitor participant well-being".to_string(),
                    });
                }
            }
        }

        // Analyze multi-session retention risks
        if experiment.sessions.len() > 3 {
            identified_risks.push(IdentifiedRisk {
                risk_type: "Participant Dropout".to_string(),
                description: format!(
                    "Multi-session study with {} sessions has elevated dropout risk",
                    experiment.sessions.len()
                ),
                probability: "Moderate (40%)".to_string(),
                severity: "Minimal".to_string(),
                affected_population: "Multi-session study participants".to_string(),
                mitigation_strategy: "Provide participant incentives, send reminder communications, and offer flexible scheduling".to_string(),
            });
        }

        // Analyze data retention risks
        if experiment.data_retention_policy.retain_raw_data_days > 2555 { // ~7 years
            identified_risks.push(IdentifiedRisk {
                risk_type: "Data Security".to_string(),
                description: "Long-term data retention increases security risks".to_string(),
                probability: "Low (20%)".to_string(),
                severity: "Moderate".to_string(),
                affected_population: "All study participants".to_string(),
                mitigation_strategy: "Regular security audits, encrypted storage, and access controls".to_string(),
            });
        }

        // Analyze algorithmic bias risks for adaptive systems
        // Check if any sessions have adaptive characteristics (non-zero learning rate)
        if experiment.sessions.iter().any(|s| s.config.learning_rate_base > 0.0) {
            identified_risks.push(IdentifiedRisk {
                risk_type: "Algorithmic Bias".to_string(),
                description: "Adaptive learning algorithms may introduce unintended bias".to_string(),
                probability: "Low (30%)".to_string(),
                severity: "Moderate".to_string(),
                affected_population: "Participants using adaptive learning features".to_string(),
                mitigation_strategy: "Regular bias testing, diverse training data, and fairness metrics monitoring".to_string(),
            });
        }
    }

    fn generate_subject_selection(
        &self,
        experiment: &MultiSessionExperiment,
    ) -> Result<SubjectSelection, String> {
        let mut inclusion_criteria = vec![
            "Age 18 or older".to_string(),
            "Able to provide informed consent".to_string(),
            "Proficient in English".to_string(),
        ];
        
        // Add experiment-specific criteria based on number of sessions
        if experiment.sessions.len() > 1 {
            inclusion_criteria.push(format!(
                "Available for {} study sessions over {} weeks",
                experiment.sessions.len(),
                experiment.sessions.len()
            ));
        }
        
        // Add criteria based on experiment complexity (multiple sessions suggest complexity)
        if experiment.sessions.len() > 2 {
            inclusion_criteria.push("Comfortable with extended learning sessions".to_string());
        }
        
        Ok(SubjectSelection {
            inclusion_criteria,
            exclusion_criteria: vec![
                "Cognitive impairment affecting ability to consent".to_string(),
                "Previous participation in similar learning studies".to_string(),
                "Inability to use computer interfaces".to_string(),
            ],
            target_enrollment: 100, // Default estimate
            recruitment_methods: vec![RecruitmentMethod {
                method_name: "Online recruitment".to_string(),
                description: "Recruitment through university subject pool".to_string(),
                materials: vec![
                    "Recruitment flyer".to_string(),
                    "Email announcement".to_string(),
                ],
                personnel_involved: vec![self.principal_investigator.name.clone()],
                coercion_safeguards: vec!["No coercion or undue inducement".to_string()],
            }],
            vulnerable_populations: Vec::new(),
            population_justification: "Adult volunteers are appropriate for this learning research"
                .to_string(),
            equitable_selection:
                "Recruitment is open to all eligible individuals regardless of demographics"
                    .to_string(),
        })
    }

    fn generate_consent_process(
        &self,
        experiment: &MultiSessionExperiment,
    ) -> Result<ConsentProcess, String> {
        // Default to electronic consent for learning studies
        let consent_type = ConsentType::Electronic;
        
        // Determine if witness is required based on risk level
        let witness_required = experiment.sessions.len() > 5; // Multi-session studies may need witness
        
        Ok(ConsentProcess {
            consent_required: true,
            consent_type,
            consent_timing: format!(
                "Before beginning {} study session{}",
                if experiment.sessions.len() == 1 { "the" } else { "the first" },
                if experiment.sessions.len() == 1 { "" } else { "s" }
            ),
            consent_location: "Online platform or research laboratory".to_string(),
            consent_personnel: vec![self.principal_investigator.name.clone()],
            language_provisions: vec!["English language consent form".to_string()],
            capacity_assessment: false,
            witness_required,
        })
    }

    fn generate_data_management_section(&self) -> Result<DataManagement, String> {
        Ok(DataManagement {
            data_collection_plan:
                "Data collected through secure online platform with automated backup".to_string(),
            data_security_measures: vec![
                "Encryption of data in transit and at rest".to_string(),
                "Access controls with user authentication".to_string(),
                "Regular security audits and updates".to_string(),
                "De-identification procedures for analysis data".to_string(),
            ],
            data_sharing_plan: format!(
                "{:?}: {}",
                self.data_protection_measures.sharing_plan.sharing_scope,
                self.data_protection_measures
                    .sharing_plan
                    .sharing_conditions
                    .join(", ")
            ),
            data_retention_plan: format!(
                "Raw data retained for {} years, analyzed data for {} years",
                self.data_protection_measures
                    .retention_schedule
                    .raw_data_years,
                self.data_protection_measures
                    .retention_schedule
                    .analyzed_data_years
            ),
            quality_assurance: vec![
                "Automated data validation checks".to_string(),
                "Regular data quality reviews".to_string(),
                "Audit trails for all data modifications".to_string(),
            ],
            personnel_training: vec![
                "Human subjects protection training".to_string(),
                "Data security and privacy training".to_string(),
                "Study-specific protocol training".to_string(),
            ],
        })
    }

    fn generate_monitoring_plan(
        &self,
        experiment: &MultiSessionExperiment,
    ) -> Result<StudyMonitoringPlan, String> {
        // Determine monitoring frequency based on experiment duration
        let reporting_schedule = if experiment.sessions.len() > 10 {
            "Monthly progress reports to IRB".to_string()
        } else if experiment.sessions.len() > 3 {
            "Quarterly progress reports to IRB".to_string()
        } else {
            "Final report to IRB upon completion".to_string()
        };
        
        let mut safety_monitoring = vec![
            "Continuous monitoring for adverse events".to_string(),
            "Regular review of participant feedback".to_string(),
            "Assessment of data quality and completeness".to_string(),
        ];
        
        // Add session-specific monitoring for multi-session studies
        if experiment.sessions.len() > 1 {
            safety_monitoring.push(format!(
                "Monitor participant retention across {} sessions",
                experiment.sessions.len()
            ));
        }
        
        Ok(StudyMonitoringPlan {
            monitoring_approach: MonitoringApproach::SelfMonitoring,
            monitoring_personnel: vec![self.principal_investigator.name.clone()],
            reporting_schedule,
            safety_monitoring,
            data_monitoring_board: None, // Not typically required for minimal risk behavioral studies
        })
    }

    fn identify_supporting_documents(
        &self,
        experiment: &MultiSessionExperiment,
    ) -> Result<Vec<SupportingDocument>, String> {
        let mut documents = Vec::new();

        // Standard consent form - customize based on experiment type
        let consent_description = if experiment.sessions.len() > 1 {
            format!("Electronic consent form for multi-session study ({} sessions)", 
                   experiment.sessions.len())
        } else {
            "Electronic consent form for single-session study".to_string()
        };

        documents.push(SupportingDocument {
            document_type: DocumentType::ConsentForm,
            document_name: format!("Informed Consent Form - {}", experiment.name),
            version: "1.0".to_string(),
            date_created: chrono::Utc::now().naive_utc().date(),
            file_path: None,
            description: consent_description,
        });

        documents.push(SupportingDocument {
            document_type: DocumentType::QualificationsCurriculum,
            document_name: "Principal Investigator CV".to_string(),
            version: "Current".to_string(),
            date_created: chrono::Utc::now().naive_utc().date(),
            file_path: None,
            description: "Curriculum vitae demonstrating PI qualifications".to_string(),
        });

        documents.push(SupportingDocument {
            document_type: DocumentType::DataCollectionTools,
            document_name: "Learning Task Battery".to_string(),
            version: "1.0".to_string(),
            date_created: chrono::Utc::now().naive_utc().date(),
            file_path: None,
            description: "Description of learning tasks and data collection procedures".to_string(),
        });

        Ok(documents)
    }

    fn format_irb_application(&self, application: &IRBApplication) -> String {
        format!(
            r#"
# IRB APPLICATION

**Application ID:** {}
**Study Title:** {}
**Principal Investigator:** {}
**Date:** {}

## STUDY SUMMARY

### Background and Rationale
{}

### Research Objectives
{}

### Study Design
{}

### Methodology
{}

### Statistical Analysis Plan
{}

## RISK-BENEFIT ANALYSIS

### Identified Risks
{}

### Risk Minimization Measures
{}

### Anticipated Benefits
{}

### Risk-Benefit Assessment
**Ratio:** {:?}
**Justification:** {}

## SUBJECT SELECTION

### Inclusion Criteria
{}

### Exclusion Criteria
{}

### Target Enrollment
{} participants

### Recruitment Methods
{}

## CONSENT PROCESS

**Consent Type:** {:?}
**Consent Timing:** {}
**Personnel:** {}

## DATA MANAGEMENT

### Data Collection
{}

### Security Measures
{}

### Data Sharing
{}

### Retention Plan
{}

## MONITORING PLAN

**Approach:** {:?}
**Personnel:** {}
**Reporting Schedule:** {}

## SUPPORTING DOCUMENTS

{}

---
*Generated by IRB Compliance System on {}*
"#,
            application.application_id,
            application.study_title,
            application.principal_investigator.name,
            application.generated_at.format("%Y-%m-%d"),
            application.study_summary.background_rationale,
            application
                .study_summary
                .research_objectives
                .iter()
                .enumerate()
                .map(|(i, obj)| format!("{}. {}", i + 1, obj))
                .collect::<Vec<_>>()
                .join("\n"),
            application.study_summary.study_design,
            application.study_summary.methodology,
            application.study_summary.statistical_analysis_plan,
            application
                .risk_benefit_analysis
                .identified_risks
                .iter()
                .map(|risk| format!(
                    "- **{}**: {} (Probability: {}, Severity: {})",
                    risk.risk_type, risk.description, risk.probability, risk.severity
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            application
                .risk_benefit_analysis
                .risk_minimization
                .iter()
                .map(|measure| format!("- {}", measure))
                .collect::<Vec<_>>()
                .join("\n"),
            application
                .risk_benefit_analysis
                .anticipated_benefits
                .iter()
                .map(|benefit| format!(
                    "- **{:?}**: {} (Likelihood: {}, Magnitude: {})",
                    benefit.benefit_type,
                    benefit.description,
                    benefit.likelihood,
                    benefit.magnitude
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            application.risk_benefit_analysis.risk_benefit_ratio,
            application.risk_benefit_analysis.justification,
            application
                .subject_selection
                .inclusion_criteria
                .iter()
                .map(|criterion| format!("- {}", criterion))
                .collect::<Vec<_>>()
                .join("\n"),
            application
                .subject_selection
                .exclusion_criteria
                .iter()
                .map(|criterion| format!("- {}", criterion))
                .collect::<Vec<_>>()
                .join("\n"),
            application.subject_selection.target_enrollment,
            application
                .subject_selection
                .recruitment_methods
                .iter()
                .map(|method| format!("**{}**: {}", method.method_name, method.description))
                .collect::<Vec<_>>()
                .join("\n\n"),
            application.consent_process.consent_type,
            application.consent_process.consent_timing,
            application.consent_process.consent_personnel.join(", "),
            application.data_management.data_collection_plan,
            application
                .data_management
                .data_security_measures
                .iter()
                .map(|measure| format!("- {}", measure))
                .collect::<Vec<_>>()
                .join("\n"),
            application.data_management.data_sharing_plan,
            application.data_management.data_retention_plan,
            application.monitoring_plan.monitoring_approach,
            application.monitoring_plan.monitoring_personnel.join(", "),
            application.monitoring_plan.reporting_schedule,
            application
                .supporting_documents
                .iter()
                .map(|doc| format!(
                    "- **{:?}**: {} (v{})",
                    doc.document_type, doc.document_name, doc.version
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    fn generate_consent_form(
        &self,
        template: &ConsentTemplate,
        application: &IRBApplication,
    ) -> String {
        let mut consent = format!(
            r#"
# INFORMED CONSENT TO PARTICIPATE IN RESEARCH

**Study Title:** {}
**Principal Investigator:** {}
**Institution:** {}

You are being invited to participate in a research study. Before you agree to participate, please read this consent form carefully and ask any questions you may have.

"#,
            application.study_title,
            application.principal_investigator.name,
            self.institution_info.name
        );

        for element in &template.consent_elements {
            consent.push_str(&self.format_consent_element(element, application));
            consent.push_str("\n\n");
        }

        consent.push_str(
            r#"
## CONSENT

By clicking "I agree" below, you indicate that:
- You have read and understand the information provided above
- You have had the opportunity to ask questions
- You voluntarily agree to participate in this research study
- You understand that you may withdraw from the study at any time

**Contact Information:**
If you have questions about this study, contact:
"#,
        );

        consent.push_str(&format!(
            "**Principal Investigator:** {} ({})\n",
            application.principal_investigator.name, application.principal_investigator.email
        ));

        consent.push_str(&format!(
            "**IRB Contact:** {} ({})\n",
            self.institution_info.irb_contact, self.institution_info.irb_email
        ));

        consent
    }

    fn format_consent_element(
        &self,
        element: &ConsentElement,
        application: &IRBApplication,
    ) -> String {
        match element.element_type {
            ConsentElementType::StudyPurpose => {
                format!("## PURPOSE OF THE STUDY\n\n{}", application.study_summary.background_rationale)
            },
            ConsentElementType::Procedures => {
                format!("## WHAT WILL HAPPEN\n\n{}", application.study_summary.methodology)
            },
            ConsentElementType::Duration => {
                format!("## TIME INVOLVEMENT\n\n{}", application.study_summary.expected_duration)
            },
            ConsentElementType::RisksDiscomforts => {
                let risks = application.risk_benefit_analysis.identified_risks.iter()
                    .map(|risk| format!("- {}", risk.description))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("## RISKS AND DISCOMFORTS\n\n{}", risks)
            },
            ConsentElementType::Benefits => {
                let benefits = application.risk_benefit_analysis.anticipated_benefits.iter()
                    .map(|benefit| format!("- {}", benefit.description))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("## BENEFITS\n\n{}", benefits)
            },
            ConsentElementType::Confidentiality => {
                format!("## CONFIDENTIALITY\n\n{}", application.data_management.data_security_measures.join(" "))
            },
            ConsentElementType::VoluntaryNature => {
                "## VOLUNTARY PARTICIPATION\n\nYour participation in this study is entirely voluntary. You may withdraw from the study at any time without penalty or loss of benefits to which you are otherwise entitled.".to_string()
            },
            _ => format!("## {:?}\n\n{}", element.element_type, element.content),
        }
    }

    fn generate_risk_assessment_report(&self, application: &IRBApplication) -> String {
        let mut report = format!(
            r#"
# RISK ASSESSMENT REPORT

**Study:** {}
**Date:** {}

## SUMMARY

This report provides a comprehensive risk assessment for the proposed research study.

## IDENTIFIED RISKS

"#,
            application.study_title,
            chrono::Utc::now().format("%Y-%m-%d")
        );

        for risk in &application.risk_benefit_analysis.identified_risks {
            report.push_str(&format!(
                r#"
### {} Risk

**Description:** {}
**Probability:** {}
**Severity:** {}
**Affected Population:** {}
**Mitigation Strategy:** {}

"#,
                risk.risk_type,
                risk.description,
                risk.probability,
                risk.severity,
                risk.affected_population,
                risk.mitigation_strategy
            ));
        }

        report.push_str(
            r#"
## RISK MINIMIZATION MEASURES

"#,
        );

        for measure in &application.risk_benefit_analysis.risk_minimization {
            report.push_str(&format!("- {}\n", measure));
        }

        report.push_str(
            r#"
## OVERALL RISK ASSESSMENT

"#,
        );
        report.push_str(&format!(
            "**Risk-Benefit Ratio:** {:?}\n",
            application.risk_benefit_analysis.risk_benefit_ratio
        ));
        report.push_str(&format!(
            "**Justification:** {}\n",
            application.risk_benefit_analysis.justification
        ));

        report
    }

    fn generate_data_management_plan(&self) -> String {
        format!(
            r#"
# DATA MANAGEMENT PLAN

## DATA COLLECTION

{}

## DATA STORAGE

**Primary Storage:** {:?} - {}
**Backup Storage:** {}
**Encryption:** {}

## ACCESS CONTROLS

**Authentication Required:** {}
**Audit Logging:** {}

## DATA SHARING

**Sharing Scope:** {:?}
**Conditions:** {}

## RETENTION SCHEDULE

- Raw Data: {} years
- Analyzed Data: {} years
- Consent Forms: {} years

## DATA DESTRUCTION

**Electronic Data Method:** {}
**Physical Records Method:** {}
**Verification Required:** {}

## INCIDENT RESPONSE

**Response Team:** {}
**Investigation Procedures:** {}

---
*Generated on {}*
"#,
            self.data_protection_measures
                .collection_methods
                .iter()
                .map(|method| format!(
                    "- **{}**: {}",
                    method.method_name,
                    method.data_types.join(", ")
                ))
                .collect::<Vec<_>>()
                .join("\n"),
            self.data_protection_measures
                .storage_plan
                .primary_storage
                .location_type,
            self.data_protection_measures
                .storage_plan
                .primary_storage
                .geographic_location,
            if let Some(backup) = &self.data_protection_measures.storage_plan.backup_storage {
                format!(
                    "{:?} - {}",
                    backup.location_type, backup.geographic_location
                )
            } else {
                "No backup storage configured".to_string()
            },
            if self
                .data_protection_measures
                .storage_plan
                .encryption_required
            {
                format!(
                    "Required ({})",
                    self.data_protection_measures
                        .storage_plan
                        .encryption_standard
                        .as_ref()
                        .unwrap_or(&"AES-256".to_string())
                )
            } else {
                "Not required".to_string()
            },
            self.data_protection_measures
                .access_controls
                .authentication_required,
            self.data_protection_measures.access_controls.audit_logging,
            self.data_protection_measures.sharing_plan.sharing_scope,
            self.data_protection_measures
                .sharing_plan
                .sharing_conditions
                .join(", "),
            self.data_protection_measures
                .retention_schedule
                .raw_data_years,
            self.data_protection_measures
                .retention_schedule
                .analyzed_data_years,
            self.data_protection_measures
                .retention_schedule
                .consent_forms_years,
            self.data_protection_measures
                .destruction_procedures
                .electronic_data_method,
            self.data_protection_measures
                .destruction_procedures
                .physical_records_method,
            self.data_protection_measures
                .destruction_procedures
                .verification_required,
            self.data_protection_measures
                .incident_response
                .response_team
                .join(", "),
            self.data_protection_measures
                .incident_response
                .investigation_procedures,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    fn format_monitoring_plan(&self, plan: &StudyMonitoringPlan) -> String {
        format!(
            r#"
# STUDY MONITORING PLAN

## MONITORING APPROACH

**Type:** {:?}
**Personnel:** {}
**Reporting Schedule:** {}

## SAFETY MONITORING

{}

## DATA MONITORING BOARD

{}

---
*Generated on {}*
"#,
            plan.monitoring_approach,
            plan.monitoring_personnel.join(", "),
            plan.reporting_schedule,
            plan.safety_monitoring
                .iter()
                .map(|item| format!("- {}", item))
                .collect::<Vec<_>>()
                .join("\n"),
            if let Some(dmb) = &plan.data_monitoring_board {
                format!(
                    "**Charter:** {}\n**Meeting Schedule:** {}",
                    dmb.charter, dmb.meeting_schedule
                )
            } else {
                "No Data Monitoring Board required for this study".to_string()
            },
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

impl Default for IRBComplianceGenerator {
    fn default() -> Self {
        Self::new(
            InstitutionInfo {
                name: "Default Institution".to_string(),
                address: "123 Default St, Default City, Default State 12345".to_string(),
                irb_contact: "Default Contact".to_string(),
                irb_phone: "555-0100".to_string(),
                irb_email: "irb@default.edu".to_string(),
                federal_wide_assurance: Some("FWA00000000".to_string()),
                jurisdiction: Jurisdiction::US,
                policies: Vec::new(),
            },
            PrincipalInvestigator {
                name: "Default PI".to_string(),
                title: "Principal Investigator".to_string(),
                department: "Research Department".to_string(),
                institution: "Default Institution".to_string(),
                email: "pi@default.edu".to_string(),
                phone: "555-0101".to_string(),
                qualifications: vec![
                    "PhD in Research".to_string(),
                    "Human Subjects Training".to_string(),
                ],
                human_subjects_training: TrainingRecord {
                    program_name: "CITI Program".to_string(),
                    completion_date: chrono::Utc::now().naive_utc().date(),
                    expiration_date: Some(
                        chrono::Utc::now().naive_utc().date() + chrono::Duration::days(365),
                    ),
                    certificate_number: Some("CITI-DEFAULT-123456".to_string()),
                    refresher_required: false,
                },
                previous_irb_approvals: Vec::new(),
            },
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub overall_status: ComplianceLevel,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub assessment_date: chrono::NaiveDate,
    pub next_review_date: chrono::NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceLevel {
    Compliant,
    MinorIssues,
    MajorIssues,
    NonCompliant,
}

// Default implementations
impl Default for ComplianceSettings {
    fn default() -> Self {
        Self {
            requires_irb_approval: true,
            exempt_categories: vec![ExemptCategory::EducationalTests],
            expedited_categories: vec![ExpeditedCategory::MinimalRiskBehavioral],
            vulnerable_populations: Vec::new(),
            international_sites: false,
            data_sharing_planned: false,
            long_term_storage: false,
        }
    }
}

impl Default for DataProtectionPlan {
    fn default() -> Self {
        Self {
            data_classification: DataClassification::Confidential,
            collection_methods: vec![CollectionMethod {
                method_name: "Online data collection".to_string(),
                data_types: vec!["Response data".to_string(), "Timing data".to_string()],
                security_measures: vec![
                    "HTTPS encryption".to_string(),
                    "Authentication".to_string(),
                ],
                personnel_authorized: vec!["Principal Investigator".to_string()],
                location: "Secure research platform".to_string(),
            }],
            storage_plan: StoragePlan {
                primary_storage: StorageLocation {
                    location_type: LocationType::InstitutionalServers,
                    geographic_location: "United States".to_string(),
                    security_certifications: vec!["SOC 2".to_string()],
                    provider_name: None,
                },
                backup_storage: None,
                encryption_required: true,
                encryption_standard: Some("AES-256".to_string()),
                access_logging: true,
            },
            access_controls: AccessControls {
                authentication_required: true,
                authorization_levels: vec![AuthorizationLevel {
                    role_name: "Principal Investigator".to_string(),
                    permissions: vec![Permission::Read, Permission::Write, Permission::Export],
                    personnel_assigned: vec!["PI".to_string()],
                    training_required: true,
                }],
                audit_logging: true,
                access_review_frequency: "Quarterly".to_string(),
            },
            sharing_plan: DataSharingPlan {
                sharing_permitted: false,
                sharing_scope: SharingScope::NoSharing,
                sharing_conditions: Vec::new(),
                data_use_agreements_required: false,
                embargo_period: None,
            },
            retention_schedule: RetentionSchedule {
                raw_data_years: 7,
                analyzed_data_years: 10,
                consent_forms_years: 7,
                regulatory_basis: "45 CFR 46 and institutional policy".to_string(),
                review_triggers: vec!["Study completion".to_string(), "Annual review".to_string()],
            },
            destruction_procedures: DestructionProcedures {
                electronic_data_method: "Secure deletion with overwriting".to_string(),
                physical_records_method: "Shredding and incineration".to_string(),
                verification_required: true,
                documentation_retained: true,
                responsible_party: "Principal Investigator".to_string(),
            },
            incident_response: IncidentResponse {
                response_team: vec![
                    "Principal Investigator".to_string(),
                    "IT Security".to_string(),
                ],
                notification_procedures: vec![NotificationProcedure {
                    incident_type: "Data breach".to_string(),
                    notification_timeline: "Within 24 hours".to_string(),
                    recipients: vec!["IRB".to_string(), "IT Security".to_string()],
                    content_requirements: vec![
                        "Incident description".to_string(),
                        "Affected data".to_string(),
                    ],
                }],
                investigation_procedures:
                    "Immediate containment, forensic analysis, and remediation".to_string(),
                remediation_procedures:
                    "System patching, access review, and participant notification if required"
                        .to_string(),
                documentation_requirements: vec![
                    "Incident report".to_string(),
                    "Timeline of events".to_string(),
                ],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_irb_generator_creation() {
        let institution = InstitutionInfo {
            name: "Test University".to_string(),
            address: "123 University Ave".to_string(),
            irb_contact: "IRB Chair".to_string(),
            irb_phone: "555-0123".to_string(),
            irb_email: "irb@test.edu".to_string(),
            federal_wide_assurance: Some("FWA00001234".to_string()),
            jurisdiction: Jurisdiction::US,
            policies: Vec::new(),
        };

        let pi = PrincipalInvestigator {
            name: "Dr. Jane Smith".to_string(),
            title: "Associate Professor".to_string(),
            department: "Psychology".to_string(),
            institution: "Test University".to_string(),
            email: "jsmith@test.edu".to_string(),
            phone: "555-0456".to_string(),
            qualifications: vec!["PhD in Psychology".to_string()],
            human_subjects_training: TrainingRecord {
                program_name: "CITI Training".to_string(),
                completion_date: chrono::Utc::now().naive_utc().date(),
                expiration_date: Some(
                    chrono::Utc::now().naive_utc().date() + chrono::Duration::days(1095),
                ),
                certificate_number: Some("12345".to_string()),
                refresher_required: false,
            },
            previous_irb_approvals: Vec::new(),
        };

        let generator = IRBComplianceGenerator::new(institution, pi);
        assert_eq!(generator.institution_info.name, "Test University");
        assert_eq!(generator.principal_investigator.name, "Dr. Jane Smith");
    }

    #[test]
    fn test_risk_assessment() {
        let assessment = RiskAssessment {
            assessment_id: "risk_001".to_string(),
            study_component: "Learning tasks".to_string(),
            risk_category: RiskCategory::Psychological,
            risk_level: RiskLevel::MinimalRisk,
            potential_harms: vec![PotentialHarm {
                harm_type: "Mild frustration".to_string(),
                description: "Participants may experience mild frustration with difficult tasks"
                    .to_string(),
                probability: Probability::Possible,
                severity: Severity::Minor,
                affected_groups: vec!["All participants".to_string()],
                reversibility: true,
            }],
            mitigation_measures: vec![MitigationMeasure {
                measure_type: MitigationType::Prevention,
                description: "Clear instructions and practice trials".to_string(),
                implementation_timeline: "Before study start".to_string(),
                responsible_person: "PI".to_string(),
                effectiveness_rating: 0.8,
            }],
            monitoring_plan: MonitoringPlan {
                monitoring_frequency: MonitoringFrequency::Continuous,
                monitoring_methods: Vec::new(),
                reporting_procedures: Vec::new(),
                stopping_rules: Vec::new(),
            },
            assessed_by: "PI".to_string(),
            assessment_date: chrono::Utc::now().naive_utc().date(),
        };

        assert_eq!(assessment.risk_level, RiskLevel::MinimalRisk);
        assert_eq!(assessment.potential_harms.len(), 1);
    }
}
