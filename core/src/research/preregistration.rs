//! Pre-registration framework for scientific integrity
//!
//! This module enforces pre-registration of hypotheses, analysis plans, and decision criteria
//! before data collection, preventing p-hacking and HARKing (Hypothesizing After Results are Known).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Pre-registration document for an experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreRegistration {
    /// Unique identifier
    pub id: String,

    /// Timestamp when pre-registered
    pub registered_at: DateTime<Utc>,

    /// Cryptographic hash of the registration for verification
    #[serde(skip)]
    pub registration_hash: String,

    /// Study metadata
    pub study: StudyMetadata,

    /// Hypotheses to test
    pub hypotheses: Hypotheses,

    /// Analysis plan
    pub analysis_plan: AnalysisPlan,

    /// Data collection plan
    pub data_collection: DataCollectionPlan,

    /// Exclusion criteria
    pub exclusion_criteria: ExclusionCriteria,

    /// Decision rules for interpreting results
    pub decision_rules: DecisionRules,

    /// Status of the pre-registration
    pub status: RegistrationStatus,

    /// Deviations from the plan (tracked post-hoc)
    pub deviations: Vec<Deviation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyMetadata {
    pub title: String,
    pub description: String,
    pub researchers: Vec<String>,
    pub institution: String,
    pub ethical_approval: Option<String>,
    pub funding_source: Option<String>,
    pub conflicts_of_interest: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypotheses {
    /// Primary hypotheses (confirmatory)
    pub primary: Vec<Hypothesis>,

    /// Secondary hypotheses (exploratory)
    pub secondary: Vec<Hypothesis>,

    /// Directional predictions
    pub directional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub description: String,
    pub operationalization: String,
    pub predicted_effect: EffectPrediction,
    pub statistical_test: String,
    pub alpha_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectPrediction {
    GreaterThan { value: f64 },
    LessThan { value: f64 },
    Different { from: f64 },
    Range { min: f64, max: f64 },
    EffectSize { cohens_d: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPlan {
    /// Primary analyses (must be run exactly as specified)
    pub primary_analyses: Vec<PlannedAnalysis>,

    /// Secondary analyses (exploratory)
    pub secondary_analyses: Vec<PlannedAnalysis>,

    /// Multiple comparison correction method
    pub multiple_comparison_correction: Option<String>,

    /// Power analysis
    pub power_analysis: PowerAnalysisSpec,

    /// Robustness checks
    pub robustness_checks: Vec<RobustnessCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedAnalysis {
    pub name: String,
    pub description: String,
    pub dependent_variable: String,
    pub independent_variables: Vec<String>,
    pub covariates: Vec<String>,
    pub statistical_model: String,
    pub assumptions_to_check: Vec<String>,
    pub fallback_if_assumptions_violated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerAnalysisSpec {
    pub target_power: f64,
    pub alpha_level: f64,
    pub effect_size: f64,
    pub sample_size_calculation: String,
    pub achieved_sample_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustnessCheck {
    pub name: String,
    pub description: String,
    pub alternative_specification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCollectionPlan {
    pub target_sample_size: usize,
    pub sampling_method: String,
    pub inclusion_criteria: Vec<String>,
    pub randomization_procedure: Option<String>,
    pub blinding: BlindingLevel,
    pub stopping_rule: StoppingRule,
    pub data_quality_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlindingLevel {
    None,
    Single,
    Double,
    Triple,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoppingRule {
    FixedSampleSize {
        n: usize,
    },
    Sequential {
        max_n: usize,
        interim_analyses: Vec<usize>,
    },
    Adaptive {
        criteria: String,
    },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionCriteria {
    pub participant_level: Vec<String>,
    pub trial_level: Vec<String>,
    pub data_quality: Vec<String>,
    pub outlier_handling: OutlierStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlierStrategy {
    None,
    Remove { threshold: f64 },
    Winsorize { percentile: f64 },
    Transform { method: String },
    RobustMethods,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRules {
    pub success_criteria: Vec<String>,
    pub failure_criteria: Vec<String>,
    pub interpretation_guidelines: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistrationStatus {
    Draft,
    Registered,
    DataCollectionStarted,
    DataCollectionComplete,
    AnalysisComplete,
    Published,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deviation {
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub justification: String,
    pub impact_assessment: String,
}

impl PreRegistration {
    /// Create a new pre-registration
    pub fn new(title: String, description: String, researchers: Vec<String>) -> Self {
        let id = format!("prereg_{}", Utc::now().timestamp());

        PreRegistration {
            id: id.clone(),
            registered_at: Utc::now(),
            registration_hash: String::new(),
            study: StudyMetadata {
                title,
                description,
                researchers,
                institution: String::new(),
                ethical_approval: None,
                funding_source: None,
                conflicts_of_interest: Vec::new(),
            },
            hypotheses: Hypotheses {
                primary: Vec::new(),
                secondary: Vec::new(),
                directional: true,
            },
            analysis_plan: AnalysisPlan {
                primary_analyses: Vec::new(),
                secondary_analyses: Vec::new(),
                multiple_comparison_correction: Some("Benjamini-Hochberg".to_string()),
                power_analysis: PowerAnalysisSpec {
                    target_power: 0.80,
                    alpha_level: 0.05,
                    effect_size: 0.5,
                    sample_size_calculation: String::new(),
                    achieved_sample_size: None,
                },
                robustness_checks: Vec::new(),
            },
            data_collection: DataCollectionPlan {
                target_sample_size: 0,
                sampling_method: String::new(),
                inclusion_criteria: Vec::new(),
                randomization_procedure: None,
                blinding: BlindingLevel::None,
                stopping_rule: StoppingRule::None,
                data_quality_checks: Vec::new(),
            },
            exclusion_criteria: ExclusionCriteria {
                participant_level: Vec::new(),
                trial_level: Vec::new(),
                data_quality: Vec::new(),
                outlier_handling: OutlierStrategy::None,
            },
            decision_rules: DecisionRules {
                success_criteria: Vec::new(),
                failure_criteria: Vec::new(),
                interpretation_guidelines: HashMap::new(),
            },
            status: RegistrationStatus::Draft,
            deviations: Vec::new(),
        }
    }

    /// Add a primary hypothesis
    pub fn add_primary_hypothesis(&mut self, hypothesis: Hypothesis) {
        self.hypotheses.primary.push(hypothesis);
    }

    /// Add a secondary hypothesis
    pub fn add_secondary_hypothesis(&mut self, hypothesis: Hypothesis) {
        self.hypotheses.secondary.push(hypothesis);
    }

    /// Add a planned analysis
    pub fn add_primary_analysis(&mut self, analysis: PlannedAnalysis) {
        self.analysis_plan.primary_analyses.push(analysis);
    }

    /// Finalize and lock the pre-registration
    pub fn finalize(&mut self) -> Result<String, String> {
        if self.hypotheses.primary.is_empty() {
            return Err("At least one primary hypothesis required".to_string());
        }

        if self.analysis_plan.primary_analyses.is_empty() {
            return Err("At least one primary analysis required".to_string());
        }

        if self.data_collection.target_sample_size == 0 {
            return Err("Target sample size must be specified".to_string());
        }

        // Set timestamp and status first, then generate hash
        self.registered_at = Utc::now();
        self.status = RegistrationStatus::Registered;
        self.registration_hash = self.generate_hash();

        Ok(self.registration_hash.clone())
    }

    /// Generate SHA-256 hash of the registration content
    fn generate_hash(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify the integrity of the registration
    pub fn verify_integrity(&self) -> bool {
        if self.registration_hash.is_empty() {
            return false;
        }

        let current_hash = self.generate_hash();
        current_hash == self.registration_hash
    }

    /// Record a deviation from the pre-registered plan
    pub fn record_deviation(&mut self, description: String, justification: String, impact: String) {
        self.deviations.push(Deviation {
            timestamp: Utc::now(),
            description,
            justification,
            impact_assessment: impact,
        });
    }

    /// Check if an analysis is pre-registered
    pub fn is_analysis_preregistered(&self, analysis_name: &str) -> bool {
        self.analysis_plan
            .primary_analyses
            .iter()
            .any(|a| a.name == analysis_name)
    }

    /// Generate a transparency report
    pub fn generate_transparency_report(&self) -> TransparencyReport {
        TransparencyReport {
            registration_id: self.id.clone(),
            registered_at: self.registered_at,
            registration_hash: self.registration_hash.clone(),
            n_primary_hypotheses: self.hypotheses.primary.len(),
            n_secondary_hypotheses: self.hypotheses.secondary.len(),
            n_primary_analyses: self.analysis_plan.primary_analyses.len(),
            n_secondary_analyses: self.analysis_plan.secondary_analyses.len(),
            n_deviations: self.deviations.len(),
            deviation_descriptions: self
                .deviations
                .iter()
                .map(|d| d.description.clone())
                .collect(),
            status: self.status.clone(),
        }
    }
}

/// Transparency report for publication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparencyReport {
    pub registration_id: String,
    pub registered_at: DateTime<Utc>,
    pub registration_hash: String,
    pub n_primary_hypotheses: usize,
    pub n_secondary_hypotheses: usize,
    pub n_primary_analyses: usize,
    pub n_secondary_analyses: usize,
    pub n_deviations: usize,
    pub deviation_descriptions: Vec<String>,
    pub status: RegistrationStatus,
}

/// Analysis validator to ensure analyses follow pre-registration
pub struct AnalysisValidator {
    preregistration: PreRegistration,
}

impl AnalysisValidator {
    pub fn new(preregistration: PreRegistration) -> Self {
        AnalysisValidator { preregistration }
    }

    /// Validate that a planned analysis matches the pre-registration
    pub fn validate_analysis(
        &self,
        analysis_name: &str,
        actual_test: &str,
        actual_variables: &[String],
    ) -> ValidationResult {
        // Find the pre-registered analysis
        let planned = self
            .preregistration
            .analysis_plan
            .primary_analyses
            .iter()
            .find(|a| a.name == analysis_name);

        if let Some(planned_analysis) = planned {
            let test_matches = planned_analysis.statistical_model == actual_test;
            let vars_match = Self::check_variables_match(
                &planned_analysis.dependent_variable,
                &planned_analysis.independent_variables,
                actual_variables,
            );

            if test_matches && vars_match {
                ValidationResult::Valid
            } else {
                ValidationResult::Deviation {
                    reason: format!(
                        "Analysis differs from pre-registration. Planned: {} with {:?}, Actual: {} with {:?}",
                        planned_analysis.statistical_model,
                        planned_analysis.independent_variables,
                        actual_test,
                        actual_variables
                    ),
                }
            }
        } else {
            ValidationResult::NotPreregistered
        }
    }

    fn check_variables_match(dependent: &str, independent: &[String], actual: &[String]) -> bool {
        // Check if actual variables match planned ones
        let mut planned = vec![dependent.to_string()];
        planned.extend(independent.iter().cloned());

        planned.len() == actual.len() && planned.iter().all(|v| actual.contains(v))
    }

    /// Mark an analysis as exploratory (not pre-registered)
    pub fn mark_exploratory(&self, analysis_name: &str) -> ExploratoryMarker {
        ExploratoryMarker {
            analysis_name: analysis_name.to_string(),
            timestamp: Utc::now(),
            warning: "This analysis was not pre-registered and should be considered exploratory"
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    Valid,
    Deviation { reason: String },
    NotPreregistered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploratoryMarker {
    pub analysis_name: String,
    pub timestamp: DateTime<Utc>,
    pub warning: String,
}

/// Builder for creating pre-registrations with validation
pub struct PreRegistrationBuilder {
    registration: PreRegistration,
}

impl PreRegistrationBuilder {
    pub fn new(title: String, description: String, researchers: Vec<String>) -> Self {
        PreRegistrationBuilder {
            registration: PreRegistration::new(title, description, researchers),
        }
    }

    pub fn with_hypothesis(mut self, hypothesis: Hypothesis, primary: bool) -> Self {
        if primary {
            self.registration.add_primary_hypothesis(hypothesis);
        } else {
            self.registration.add_secondary_hypothesis(hypothesis);
        }
        self
    }

    pub fn with_sample_size(mut self, n: usize) -> Self {
        self.registration.data_collection.target_sample_size = n;
        self
    }

    pub fn with_power_analysis(mut self, power: f64, alpha: f64, effect_size: f64) -> Self {
        self.registration.analysis_plan.power_analysis = PowerAnalysisSpec {
            target_power: power,
            alpha_level: alpha,
            effect_size,
            sample_size_calculation: format!(
                "n = {} for power = {}, alpha = {}, d = {}",
                self.calculate_sample_size(power, alpha, effect_size),
                power,
                alpha,
                effect_size
            ),
            achieved_sample_size: None,
        };
        self
    }

    fn calculate_sample_size(&self, power: f64, alpha: f64, effect_size: f64) -> usize {
        // Simplified calculation for t-test
        use statrs::distribution::{ContinuousCDF, Normal};
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - alpha / 2.0);
        let z_beta = normal.inverse_cdf(power);
        let n = ((z_alpha + z_beta).powi(2) * 2.0) / effect_size.powi(2);
        n.ceil() as usize
    }

    pub fn with_analysis(mut self, analysis: PlannedAnalysis, primary: bool) -> Self {
        if primary {
            self.registration
                .analysis_plan
                .primary_analyses
                .push(analysis);
        } else {
            self.registration
                .analysis_plan
                .secondary_analyses
                .push(analysis);
        }
        self
    }

    pub fn with_exclusion_criteria(mut self, criteria: ExclusionCriteria) -> Self {
        self.registration.exclusion_criteria = criteria;
        self
    }

    pub fn with_blinding(mut self, level: BlindingLevel) -> Self {
        self.registration.data_collection.blinding = level;
        self
    }

    pub fn build(mut self) -> Result<PreRegistration, String> {
        self.registration.finalize()?;
        Ok(self.registration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preregistration_creation() {
        let mut prereg = PreRegistration::new(
            "Test Study".to_string(),
            "Testing pre-registration".to_string(),
            vec!["Researcher A".to_string()],
        );

        prereg.add_primary_hypothesis(Hypothesis {
            id: "H1".to_string(),
            description: "Treatment improves performance".to_string(),
            operationalization: "Performance measured by accuracy".to_string(),
            predicted_effect: EffectPrediction::GreaterThan { value: 0.0 },
            statistical_test: "t-test".to_string(),
            alpha_level: 0.05,
        });

        prereg.add_primary_analysis(PlannedAnalysis {
            name: "Primary Analysis".to_string(),
            description: "Compare treatment vs control".to_string(),
            dependent_variable: "accuracy".to_string(),
            independent_variables: vec!["group".to_string()],
            covariates: vec![],
            statistical_model: "t-test".to_string(),
            assumptions_to_check: vec!["normality".to_string(), "homogeneity".to_string()],
            fallback_if_assumptions_violated: Some("Mann-Whitney U".to_string()),
        });

        prereg.data_collection.target_sample_size = 100;

        let result = prereg.finalize();
        assert!(result.is_ok());
        assert!(!prereg.registration_hash.is_empty());
    }

    #[test]
    fn test_integrity_verification() {
        let mut prereg = PreRegistration::new(
            "Test".to_string(),
            "Test".to_string(),
            vec!["Test".to_string()],
        );

        prereg.add_primary_hypothesis(Hypothesis {
            id: "H1".to_string(),
            description: "Test hypothesis".to_string(),
            operationalization: "Test".to_string(),
            predicted_effect: EffectPrediction::Different { from: 0.0 },
            statistical_test: "t-test".to_string(),
            alpha_level: 0.05,
        });

        prereg.add_primary_analysis(PlannedAnalysis {
            name: "Test".to_string(),
            description: "Test".to_string(),
            dependent_variable: "y".to_string(),
            independent_variables: vec!["x".to_string()],
            covariates: vec![],
            statistical_model: "regression".to_string(),
            assumptions_to_check: vec![],
            fallback_if_assumptions_violated: None,
        });

        prereg.data_collection.target_sample_size = 50;
        prereg.finalize().unwrap();

        assert!(prereg.verify_integrity());

        // Tamper with the registration
        prereg.hypotheses.primary[0].alpha_level = 0.01;

        // Hash should no longer match
        assert!(!prereg.verify_integrity());
    }
}
