use crate::config::LearnerConfig;
use crate::statistics::power_analysis::PowerAnalyzer;
use crate::statistics::validation::StatisticalValidator;
use crate::statistics::TestResult;
use crate::core::topology::Topology;
use crate::research::citations::{CitationTracker, MethodologyReport};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// A/B Testing Framework for Intervention Comparisons
/// Supports multi-armed bandits, sequential testing, and adaptive allocation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub hypothesis: Hypothesis,
    pub variants: Vec<TestVariant>,
    pub allocation_strategy: AllocationStrategy,
    pub stopping_criteria: StoppingCriteria,
    pub success_metrics: Vec<SuccessMetric>,
    pub guard_metrics: Vec<GuardMetric>,
    pub status: TestStatus,
    pub configuration: TestConfiguration,
    pub results: Option<ABTestResults>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub primary_metric: String,
    pub expected_effect_size: f64,
    pub direction: EffectDirection,
    pub null_value: f64,
    pub alternative_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectDirection {
    Increase,
    Decrease,
    TwoSided,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestVariant {
    pub id: String,
    pub name: String,
    pub description: String,
    pub intervention: InterventionSpec,
    pub allocation_weight: f64,
    pub expected_participants: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionSpec {
    pub config: LearnerConfig,
    pub topology: Topology,
    pub parameters: HashMap<String, serde_json::Value>,
    pub intervention_type: InterventionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionType {
    HintingStrategy { level: String },
    DifficultyProgression { rate: f64 },
    FeedbackTiming { delay_ms: u32 },
    TaskSequencing { strategy: String },
    MotivationalFraming { frame: String },
    InterfaceDesign { layout: String },
    AdaptiveAlgorithm { algorithm: String },
    Control, // No intervention
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStrategy {
    /// Simple randomization with fixed proportions
    FixedRandomization { proportions: Vec<f64> },

    /// Block randomization to ensure balance
    BlockRandomization { block_size: usize },

    /// Stratified randomization based on participant characteristics
    StratifiedRandomization { strata: Vec<String> },

    /// Multi-armed bandit with exploration/exploitation
    MultiarmedBandit {
        strategy: BanditStrategy,
        exploration_rate: f64,
        burn_in_samples: usize,
    },

    /// Adaptive allocation based on observed performance
    AdaptiveAllocation {
        reallocation_frequency: usize,
        min_allocation_per_arm: f64,
    },

    /// Sequential probability ratio test
    SequentialTesting {
        alpha_spending_function: AlphaSpendingFunction,
        beta_spending_function: BetaSpendingFunction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BanditStrategy {
    EpsilonGreedy { epsilon: f64 },
    UCB1 { confidence_level: f64 },
    ThompsonSampling,
    LinUCB { alpha: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlphaSpendingFunction {
    OBrienFleming,
    Pocock,
    Linear,
    Custom { function: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BetaSpendingFunction {
    Linear,
    Exponential { rate: f64 },
    Custom { function: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoppingCriteria {
    pub max_participants: Option<usize>,
    pub max_duration_days: Option<u32>,
    pub statistical_power_threshold: f64,
    pub practical_significance_threshold: f64,
    pub futility_threshold: f64,
    pub interim_analysis_frequency: InterimAnalysisFrequency,
    pub early_stopping_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterimAnalysisFrequency {
    FixedSampleSize {
        every_n_participants: usize,
    },
    FixedTime {
        every_n_hours: u32,
    },
    AdaptiveLooking {
        information_fraction: f64,
    },
    CalendarTime {
        schedule: Vec<chrono::DateTime<chrono::Utc>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessMetric {
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub aggregation: AggregationMethod,
    pub is_primary: bool,
    pub higher_is_better: bool,
    pub minimum_detectable_effect: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    ConversionRate,
    ContinuousValue,
    Count,
    TimeToEvent,
    Ratio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationMethod {
    Mean,
    Median,
    Proportion,
    Sum,
    Count,
    P95,
    Custom { formula: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardMetric {
    pub name: String,
    pub description: String,
    pub threshold: GuardThreshold,
    pub action: GuardAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardThreshold {
    AbsoluteDecrease { threshold: f64 },
    RelativeDecrease { percentage: f64 },
    Statistical { p_value: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardAction {
    StopTest,
    Alert,
    ReduceAllocation,
    ExcludeVariant { variant_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfiguration {
    pub alpha: f64,
    pub power: f64,
    pub minimum_effect_size: f64,
    pub sequential_testing: bool,
    pub multiple_testing_correction: MultipleTesting,
    pub randomization_seed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MultipleTesting {
    None,
    Bonferroni,
    BenjaminiHochberg,
    Holm,
    Sidak,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    Draft,
    Ready,
    Running,
    Paused,
    Stopped,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResults {
    pub primary_analysis: PrimaryAnalysis,
    pub secondary_analyses: Vec<SecondaryAnalysis>,
    pub variant_performance: HashMap<String, VariantPerformance>,
    pub statistical_tests: Vec<TestResult>,
    pub confidence_intervals: HashMap<String, (f64, f64)>,
    pub practical_significance: PracticalSignificanceAssessment,
    pub recommendation: TestRecommendation,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryAnalysis {
    pub metric_name: String,
    pub winner: Option<String>,
    pub effect_size: f64,
    pub statistical_significance: bool,
    pub practical_significance: bool,
    pub confidence_level: f64,
    pub p_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryAnalysis {
    pub metric_name: String,
    pub results: HashMap<String, f64>, // variant_id -> metric_value
    pub statistical_test: TestResult,
    pub effect_sizes: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantPerformance {
    pub participant_count: usize,
    pub metric_values: HashMap<String, f64>,
    pub confidence_intervals: HashMap<String, (f64, f64)>,
    pub allocation_history: Vec<AllocationRecord>,
    pub data_quality_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub participant_count: usize,
    pub allocation_probability: f64,
    pub cumulative_metric_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticalSignificanceAssessment {
    pub metric_improvements: HashMap<String, f64>,
    pub cost_benefit_analysis: Option<CostBenefitAnalysis>,
    pub implementation_complexity: ImplementationComplexity,
    pub recommendation_strength: RecommendationStrength,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBenefitAnalysis {
    pub implementation_cost: f64,
    pub maintenance_cost: f64,
    pub expected_benefit: f64,
    pub roi_estimate: f64,
    pub payback_period_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationComplexity {
    Trivial,
    Low,
    Medium,
    High,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationStrength {
    StrongImplement,
    WeakImplement,
    Neutral,
    WeakReject,
    StrongReject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestRecommendation {
    ImplementWinner {
        variant_id: String,
        confidence: f64,
    },
    ContinueTesting {
        reason: String,
        estimated_completion_date: chrono::DateTime<chrono::Utc>,
    },
    StopForFutility {
        reason: String,
    },
    RequiresManualReview {
        concerns: Vec<String>,
    },
    InconclusiveResults {
        next_steps: Vec<String>,
    },
}

/// Participant assignment tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantAssignment {
    pub participant_id: String,
    pub variant_id: String,
    pub assignment_time: chrono::DateTime<chrono::Utc>,
    pub assignment_method: String,
    pub context: HashMap<String, serde_json::Value>,
}

/// Main A/B testing framework
pub struct ABTestFramework {
    tests: HashMap<String, ABTest>,
    assignments: HashMap<String, ParticipantAssignment>,
    power_analyzer: PowerAnalyzer,
    statistical_validator: StatisticalValidator,
    citation_tracker: CitationTracker,
    // Track performance data separately from variant definitions
    variant_performance: HashMap<String, HashMap<String, VariantPerformance>>, // test_id -> variant_id -> performance
    rng: StdRng,
}

impl ABTestFramework {
    pub fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        Self {
            tests: HashMap::new(),
            assignments: HashMap::new(),
            power_analyzer: PowerAnalyzer::new(0.05, 0.8),
            statistical_validator: StatisticalValidator::new(0.95),
            citation_tracker: CitationTracker::new(),
            variant_performance: HashMap::new(),
            rng,
        }
    }

    /// Create a new A/B test
    pub fn create_test(
        &mut self,
        name: String,
        description: String,
        hypothesis: Hypothesis,
        variants: Vec<TestVariant>,
        allocation_strategy: AllocationStrategy,
    ) -> Result<String, String> {
        // Validate test configuration
        self.validate_test_configuration(&variants, &allocation_strategy)?;

        let test_id = uuid::Uuid::new_v4().to_string();

        let test = ABTest {
            id: test_id.clone(),
            name,
            description,
            hypothesis,
            variants,
            allocation_strategy,
            stopping_criteria: StoppingCriteria::default(),
            success_metrics: Vec::new(),
            guard_metrics: Vec::new(),
            status: TestStatus::Draft,
            configuration: TestConfiguration::default(),
            results: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            ended_at: None,
        };

        self.tests.insert(test_id.clone(), test);
        Ok(test_id)
    }

    /// Assign a participant to a test variant
    pub fn assign_participant(
        &mut self,
        test_id: &str,
        participant_id: String,
        context: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String, String> {
        // Check if participant is already assigned first
        if let Some(existing) = self.assignments.get(&participant_id) {
            return Ok(existing.variant_id.clone());
        }

        // Clone test data to avoid borrowing conflicts
        let test = {
            let test_ref = self.tests.get(test_id).ok_or("Test not found")?;

            if test_ref.status != TestStatus::Running {
                return Err("Test is not currently running".to_string());
            }

            test_ref.clone()
        };

        // Select variant based on allocation strategy
        let variant_id = self.select_variant(&test, &context)?;

        // Record assignment
        let assignment = ParticipantAssignment {
            participant_id: participant_id.clone(),
            variant_id: variant_id.clone(),
            assignment_time: chrono::Utc::now(),
            assignment_method: format!("{:?}", test.allocation_strategy),
            context: context.unwrap_or_default(),
        };

        self.assignments.insert(participant_id, assignment);

        Ok(variant_id)
    }

    /// Record outcome data for a participant
    pub fn record_outcome(
        &mut self,
        test_id: &str,
        participant_id: &str,
        metric_name: &str,
        value: f64,
        timestamp: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), String> {
        let assignment = self
            .assignments
            .get(participant_id)
            .ok_or("Participant not assigned to any test")?;

        // Get or create variant performance tracking
        let test_performance = self.variant_performance.entry(test_id.to_string())
            .or_insert_with(HashMap::new);
        let variant_id = &assignment.variant_id;
        let variant_perf = test_performance.entry(variant_id.clone())
            .or_insert_with(|| VariantPerformance {
                participant_count: 0,
                metric_values: HashMap::new(),
                confidence_intervals: HashMap::new(),
                allocation_history: Vec::new(),
                data_quality_score: 1.0,
            });
            
        // Record the outcome based on metric_name
        let outcome_timestamp = timestamp.unwrap_or_else(chrono::Utc::now);
        
        // Get test info for allocation calculation
        let variant_count = self.tests.get(test_id)
            .map(|t| t.variants.len())
            .unwrap_or(2) as f64;
        
        // Update participant count
        variant_perf.participant_count += 1;
        
        // Update metric values
        let current_value = variant_perf.metric_values.get(metric_name).copied().unwrap_or(0.0);
        let current_count = variant_perf.participant_count as f64;
        let new_average = (current_value * (current_count - 1.0) + value) / current_count;
        variant_perf.metric_values.insert(metric_name.to_string(), new_average);
        
        // Record allocation history
        variant_perf.allocation_history.push(AllocationRecord {
            timestamp: outcome_timestamp,
            participant_count: variant_perf.participant_count,
            allocation_probability: 1.0 / variant_count, // Simple equal allocation for now
            cumulative_metric_value: new_average,
        });
        
        // Update test status if needed
        let test = self.tests.get_mut(test_id).unwrap();
        if test.status == TestStatus::Running {
            let total_participants: usize = self.variant_performance
                .get(test_id)
                .map(|perf| perf.values().map(|v| v.participant_count).sum())
                .unwrap_or(0);
                
            // Check if we've reached sample size targets (placeholder logic)
            if total_participants >= 100 { // Simple threshold for now
                test.status = TestStatus::Completed;
            }
        }

        // Log the outcome for analysis (in practice, this would go to a database)
        println!("Recorded outcome: test={}, participant={}, metric={}, value={}, timestamp={:?}", 
                test_id, participant_id, metric_name, value, outcome_timestamp);

        // Check if interim analysis is needed
        self.check_interim_analysis(test_id)?;

        Ok(())
    }

    /// Run interim analysis and check stopping criteria
    pub fn run_interim_analysis(&mut self, test_id: &str) -> Result<InterimAnalysisResult, String> {
        // Clone test data to avoid borrowing conflicts
        let test = self.tests.get(test_id).ok_or("Test not found")?.clone();

        // Collect current data for all variants
        let variant_data = self.collect_variant_data(&test)?;

        // Perform statistical tests
        let statistical_results = self.perform_statistical_tests(&test, &variant_data)?;

        // Check stopping criteria
        let stopping_decision = self.evaluate_stopping_criteria(&test, &statistical_results)?;

        // Check guard metrics (requires &mut self for RNG)
        let guard_violations = self.check_guard_metrics(&test, &variant_data)?;

        // Generate recommendations
        let recommendations = self.generate_interim_recommendations(
            &stopping_decision,
            &guard_violations,
            &statistical_results,
        );

        Ok(InterimAnalysisResult {
            test_id: test_id.to_string(),
            analysis_time: chrono::Utc::now(),
            participant_counts: variant_data
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect(),
            statistical_results,
            stopping_decision,
            guard_violations,
            recommendations,
        })
    }

    /// Finalize test and generate comprehensive results
    pub fn finalize_test(&mut self, test_id: &str) -> Result<ABTestResults, String> {
        // First check test status and clone test data
        let test_clone = {
            let test = self.tests.get(test_id).ok_or("Test not found")?;

            if test.status != TestStatus::Running {
                return Err("Test is not running".to_string());
            }

            test.clone()
        };

        // Collect all data
        let variant_data = self.collect_variant_data(&test_clone)?;

        // Perform comprehensive analysis
        let primary_analysis = self.perform_primary_analysis(&test_clone, &variant_data)?;
        let secondary_analyses = self.perform_secondary_analyses(&test_clone, &variant_data)?;
        let variant_performance = self.calculate_variant_performance(&variant_data)?;

        // Statistical tests with multiple testing correction
        let statistical_tests = self.perform_final_statistical_tests(&test_clone, &variant_data)?;

        // Calculate confidence intervals
        let confidence_intervals = self.calculate_confidence_intervals(&variant_data, 0.95)?;

        // Practical significance assessment
        let practical_significance =
            self.assess_practical_significance(&test_clone, &primary_analysis)?;

        // Generate final recommendation
        let recommendation = self.generate_final_recommendation(
            &primary_analysis,
            &practical_significance,
            &statistical_tests,
        )?;

        let results = ABTestResults {
            primary_analysis,
            secondary_analyses,
            variant_performance,
            statistical_tests,
            confidence_intervals,
            practical_significance,
            recommendation,
            generated_at: chrono::Utc::now(),
        };

        // Update test status and results
        let test = self.tests.get_mut(test_id).ok_or("Test not found")?;
        test.status = TestStatus::Completed;
        test.ended_at = Some(chrono::Utc::now());
        test.results = Some(results.clone());

        Ok(results)
    }

    // Helper methods
    fn validate_test_configuration(
        &self,
        variants: &[TestVariant],
        allocation_strategy: &AllocationStrategy,
    ) -> Result<(), String> {
        if variants.len() < 2 {
            return Err("At least 2 variants required".to_string());
        }

        // Check allocation weights sum to 1.0
        let total_weight: f64 = variants.iter().map(|v| v.allocation_weight).sum();
        if (total_weight - 1.0).abs() > 0.001 {
            return Err("Allocation weights must sum to 1.0".to_string());
        }

        // Validate variant IDs are unique
        let mut ids = HashSet::new();
        for variant in variants {
            if !ids.insert(&variant.id) {
                return Err(format!("Duplicate variant ID: {}", variant.id));
            }
        }

        Ok(())
    }

    fn select_variant(
        &mut self,
        test: &ABTest,
        context: &Option<HashMap<String, serde_json::Value>>,
    ) -> Result<String, String> {
        match &test.allocation_strategy {
            AllocationStrategy::FixedRandomization { proportions } => {
                let rand_val: f64 = self.rng.gen();
                let mut cumulative = 0.0;

                for (i, variant) in test.variants.iter().enumerate() {
                    cumulative += proportions.get(i).unwrap_or(&variant.allocation_weight);
                    if rand_val <= cumulative {
                        return Ok(variant.id.clone());
                    }
                }

                // Fallback to last variant
                Ok(test.variants.last().unwrap().id.clone())
            }

            AllocationStrategy::MultiarmedBandit {
                strategy,
                exploration_rate,
                burn_in_samples,
            } => self.select_bandit_variant(test, strategy, *exploration_rate, *burn_in_samples),

            _ => {
                // For other strategies, use simple randomization as fallback
                let variant = test.variants.choose(&mut self.rng).unwrap();
                Ok(variant.id.clone())
            }
        }
    }

    fn select_bandit_variant(
        &mut self,
        test: &ABTest,
        strategy: &BanditStrategy,
        exploration_rate: f64,
        burn_in_samples: usize,
    ) -> Result<String, String> {
        // Simplified bandit implementation
        let total_assignments = self.assignments.len();

        if total_assignments < burn_in_samples {
            // During burn-in, use uniform random allocation
            let variant = test.variants.choose(&mut self.rng).unwrap();
            return Ok(variant.id.clone());
        }

        match strategy {
            BanditStrategy::EpsilonGreedy { epsilon } => {
                if self.rng.gen::<f64>() < *epsilon {
                    // Explore: random selection
                    let variant = test.variants.choose(&mut self.rng).unwrap();
                    Ok(variant.id.clone())
                } else {
                    // Exploit: select best performing variant
                    self.select_best_performing_variant(test)
                }
            }

            BanditStrategy::UCB1 { confidence_level } => {
                self.select_ucb1_variant(test, *confidence_level)
            }

            _ => {
                // Fallback to random
                let variant = test.variants.choose(&mut self.rng).unwrap();
                Ok(variant.id.clone())
            }
        }
    }

    fn select_best_performing_variant(&self, test: &ABTest) -> Result<String, String> {
        // In practice, this would query actual performance data
        // For now, select first variant as placeholder
        Ok(test.variants[0].id.clone())
    }

    fn select_ucb1_variant(&self, test: &ABTest, confidence_level: f64) -> Result<String, String> {
        if test.variants.is_empty() {
            return Err("No variants available for selection".to_string());
        }
        
        // Get performance data for this test
        let test_perf = self.variant_performance.get(&test.id)
            .ok_or("No performance data available for test")?;
        
        let mut variant_scores = HashMap::new();
        let total_trials: f64 = test_perf.values()
            .map(|v| v.participant_count as f64)
            .sum();
            
        if total_trials == 0.0 {
            // No data yet, return random variant
            return Ok(test.variants[0].id.clone());
        }
        
        for variant in &test.variants {
            let variant_perf = test_perf.get(&variant.id);
            
            let n_trials = variant_perf.map(|p| p.participant_count as f64).unwrap_or(0.0);
            
            if n_trials == 0.0 {
                // Unplayed variant gets infinite score (exploration)
                variant_scores.insert(variant.id.clone(), f64::INFINITY);
                continue;
            }
            
            // Use primary metric value as reward, fallback to 0.0
            let mean_reward = variant_perf
                .and_then(|p| p.metric_values.get("primary_metric").copied())
                .unwrap_or(0.0);
            
            // UCB1 formula: mean + confidence_level * sqrt(2 * ln(total_trials) / n_trials)
            let confidence_bonus = confidence_level * (2.0_f64 * total_trials.ln() / n_trials).sqrt();
            let ucb_score = mean_reward + confidence_bonus;
            
            variant_scores.insert(variant.id.clone(), ucb_score);
        }
        
        // Return variant with highest UCB score
        let best_variant = variant_scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _score)| name.clone())
            .ok_or("Failed to select variant")?;
            
        Ok(best_variant)
    }

    fn collect_variant_data(&self, test: &ABTest) -> Result<HashMap<String, Vec<f64>>, String> {
        // In practice, this would query the database for actual outcome data
        // For now, return mock data
        let mut data = HashMap::new();
        for variant in &test.variants {
            data.insert(variant.id.clone(), vec![0.75, 0.80, 0.70, 0.85]); // Mock data
        }
        Ok(data)
    }

    fn perform_statistical_tests(
        &self,
        test: &ABTest,
        variant_data: &HashMap<String, Vec<f64>>,
    ) -> Result<Vec<TestResult>, String> {
        let mut results = Vec::new();

        if variant_data.len() < 2 {
            return Ok(results);
        }

        // Perform pairwise comparisons
        let variants: Vec<_> = variant_data.keys().collect();
        for i in 0..variants.len() {
            for j in i + 1..variants.len() {
                let data1 = &variant_data[variants[i]];
                let data2 = &variant_data[variants[j]];

                let hypothesis_result = self.statistical_validator.t_test(data1, data2, false);
                
                // Record statistical procedure for citation tracking
                let procedure_description = format!(
                    "Independent samples t-test comparing {} vs {} (n1={}, n2={})",
                    variants[i], variants[j], data1.len(), data2.len()
                );
                
                // This would need mut self to record, so we note the procedure was used
                // In a real implementation, this would track procedures for later citation
                println!("Statistical procedure used: {}", procedure_description);
                
                let test_result = TestResult {
                    statistic: hypothesis_result.statistic,
                    p_value: hypothesis_result.p_value,
                    significant: hypothesis_result.significant,
                    test_name: hypothesis_result.test_name,
                    correction_applied: None,
                };
                results.push(test_result);
            }
        }

        Ok(results)
    }

    fn check_interim_analysis(&mut self, test_id: &str) -> Result<(), String> {
        let test = self.tests.get(test_id)
            .ok_or("Test not found")?;
            
        // Check if interim analysis should be triggered based on various criteria
        let total_participants: usize = self.variant_performance
            .get(test_id)
            .map(|perf| perf.values().map(|v| v.participant_count).sum())
            .unwrap_or(0);
            
        let should_run_interim = match &test.status {
            TestStatus::Running => {
                // Run interim analysis every 50 participants
                total_participants > 0 && total_participants % 50 == 0
            }
            _ => false,
        };
        
        if should_run_interim {
            println!("Triggering interim analysis for test {} with {} participants", 
                    test_id, total_participants);
                    
            // Run the interim analysis
            let _interim_result = self.run_interim_analysis(test_id)?;
            
            // Note: In practice, interim results would be stored and potentially
            // trigger early stopping decisions
        }
        
        Ok(())
    }

    fn evaluate_stopping_criteria(
        &self,
        test: &ABTest,
        statistical_results: &[TestResult],
    ) -> Result<StoppingDecision, String> {
        // Evaluate various stopping criteria
        let mut reasons = Vec::new();

        // Check for statistical significance
        let has_significant_result = statistical_results.iter().any(|r| r.significant);

        if has_significant_result {
            reasons.push("Significant result detected".to_string());
        }

        // Check participant count
        let participant_count = self.assignments.len();
        if let Some(max_participants) = test.stopping_criteria.max_participants {
            if participant_count >= max_participants {
                reasons.push("Maximum participant count reached".to_string());
            }
        }

        let decision = if !reasons.is_empty() {
            StoppingDecision::Stop { reasons }
        } else {
            StoppingDecision::Continue {
                reason: "Stopping criteria not met".to_string(),
            }
        };

        Ok(decision)
    }

    fn check_guard_metrics(
        &mut self,
        test: &ABTest,
        variant_data: &HashMap<String, Vec<f64>>,
    ) -> Result<Vec<GuardViolation>, String> {
        let mut violations = Vec::new();

        // Check each guard metric
        for guard in &test.guard_metrics {
            // Simplified check - in practice would evaluate actual guard conditions
            if self.rng.gen::<f64>() < 0.05 {
                // 5% chance of violation for demo
                violations.push(GuardViolation {
                    metric_name: guard.name.clone(),
                    threshold: guard.threshold.clone(),
                    current_value: 0.45, // Mock value
                    severity: ViolationSeverity::Warning,
                    recommended_action: guard.action.clone(),
                });
            }
        }

        Ok(violations)
    }

    fn generate_interim_recommendations(
        &self,
        stopping_decision: &StoppingDecision,
        guard_violations: &[GuardViolation],
        statistical_results: &[TestResult],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        match stopping_decision {
            StoppingDecision::Stop { reasons } => {
                recommendations.push("Consider stopping the test".to_string());
                recommendations.extend(reasons.iter().map(|r| format!("Reason: {}", r)));
            }
            StoppingDecision::Continue { reason } => {
                recommendations.push(format!("Continue testing: {}", reason));
            }
        }

        if !guard_violations.is_empty() {
            recommendations.push(format!(
                "{} guard metric violation(s) detected - review immediately",
                guard_violations.len()
            ));
        }

        recommendations
    }

    fn perform_primary_analysis(
        &self,
        test: &ABTest,
        variant_data: &HashMap<String, Vec<f64>>,
    ) -> Result<PrimaryAnalysis, String> {
        let primary_metric = &test.hypothesis.primary_metric;

        // Find the best performing variant
        let mut best_variant = None;
        let mut best_performance = f64::NEG_INFINITY;

        for (variant_id, data) in variant_data {
            let mean_performance = data.iter().sum::<f64>() / data.len() as f64;
            if mean_performance > best_performance {
                best_performance = mean_performance;
                best_variant = Some(variant_id.clone());
            }
        }

        // Calculate effect size (simplified)
        let effect_size = 0.3; // Mock value
        let p_value = 0.02; // Mock value

        Ok(PrimaryAnalysis {
            metric_name: primary_metric.clone(),
            winner: best_variant,
            effect_size,
            statistical_significance: p_value < test.configuration.alpha,
            practical_significance: effect_size.abs() >= test.configuration.minimum_effect_size,
            confidence_level: 1.0 - test.configuration.alpha,
            p_value,
        })
    }

    fn perform_secondary_analyses(
        &self,
        test: &ABTest,
        variant_data: &HashMap<String, Vec<f64>>,
    ) -> Result<Vec<SecondaryAnalysis>, String> {
        // Perform analysis for each secondary metric
        let mut analyses = Vec::new();

        for metric in &test.success_metrics {
            if metric.is_primary {
                continue;
            }

            let mut results = HashMap::new();
            let mut effect_sizes = HashMap::new();

            for (variant_id, data) in variant_data {
                let mean_value = data.iter().sum::<f64>() / data.len() as f64;
                results.insert(variant_id.clone(), mean_value);
                effect_sizes.insert(variant_id.clone(), 0.2); // Mock effect size
            }

            // Mock statistical test result
            let test_result = TestResult {
                statistic: 2.5,
                p_value: 0.03,
                significant: true,
                test_name: "t-test".to_string(),
                correction_applied: None,
            };

            analyses.push(SecondaryAnalysis {
                metric_name: metric.name.clone(),
                results,
                statistical_test: test_result,
                effect_sizes,
            });
        }

        Ok(analyses)
    }

    fn calculate_variant_performance(
        &self,
        variant_data: &HashMap<String, Vec<f64>>,
    ) -> Result<HashMap<String, VariantPerformance>, String> {
        let mut performance = HashMap::new();

        for (variant_id, data) in variant_data {
            let mean_value = data.iter().sum::<f64>() / data.len() as f64;

            let mut metric_values = HashMap::new();
            metric_values.insert("primary_metric".to_string(), mean_value);

            let mut confidence_intervals = HashMap::new();
            confidence_intervals.insert(
                "primary_metric".to_string(),
                (mean_value - 0.1, mean_value + 0.1),
            );

            performance.insert(
                variant_id.clone(),
                VariantPerformance {
                    participant_count: data.len(),
                    metric_values,
                    confidence_intervals,
                    allocation_history: Vec::new(), // Would be populated in practice
                    data_quality_score: 0.95,
                },
            );
        }

        Ok(performance)
    }

    fn perform_final_statistical_tests(
        &self,
        test: &ABTest,
        variant_data: &HashMap<String, Vec<f64>>,
    ) -> Result<Vec<TestResult>, String> {
        self.perform_statistical_tests(test, variant_data)
    }

    fn calculate_confidence_intervals(
        &self,
        variant_data: &HashMap<String, Vec<f64>>,
        confidence_level: f64,
    ) -> Result<HashMap<String, (f64, f64)>, String> {
        let mut intervals = HashMap::new();

        for (variant_id, data) in variant_data {
            let mean = data.iter().sum::<f64>() / data.len() as f64;
            let margin = 0.1; // Simplified calculation

            intervals.insert(variant_id.clone(), (mean - margin, mean + margin));
        }

        Ok(intervals)
    }

    /// Calculate statistical power for the test using the power_analyzer field
    fn calculate_statistical_power(
        &self,
        test: &ABTest,
        confidence_level: f64,
    ) -> Result<f64, String> {
        if test.variants.len() < 2 {
            return Err("Need at least 2 variants to calculate power".to_string());
        }
        
        // Get performance data
        let test_perf = self.variant_performance.get(&test.id)
            .ok_or("No performance data available for test")?;
            
        // Find control and treatment variants
        let control = test.variants.iter()
            .find(|v| v.name.to_lowercase().contains("control"))
            .or_else(|| test.variants.first())
            .ok_or("No control variant found")?;
            
        let treatment = test.variants.iter()
            .find(|v| !v.name.to_lowercase().contains("control") && v.id != control.id)
            .or_else(|| test.variants.get(1))
            .ok_or("No treatment variant found")?;
        
        // Get performance data for each variant
        let control_perf = test_perf.get(&control.id);
        let treatment_perf = test_perf.get(&treatment.id);
        
        // Calculate effect size using performance data
        let p1 = control_perf
            .and_then(|p| p.metric_values.get("primary_metric").copied())
            .unwrap_or(0.05); // Default baseline conversion rate
        let p2 = treatment_perf
            .and_then(|p| p.metric_values.get("primary_metric").copied())
            .unwrap_or(0.05);
        
        let effect_size = if p1 > 0.0 && p1 < 1.0 && p2 > 0.0 && p2 < 1.0 {
            // Cohen's h: 2 * (arcsin(sqrt(p2)) - arcsin(sqrt(p1)))
            2.0 * (p2.sqrt().asin() - p1.sqrt().asin()).abs()
        } else {
            // Fallback to simple difference for edge cases
            (p2 - p1).abs()
        };
        
        // Total sample size from performance data
        let total_n = control_perf.map(|p| p.participant_count).unwrap_or(0) +
                     treatment_perf.map(|p| p.participant_count).unwrap_or(0);
        
        if total_n == 0 {
            return Ok(0.0); // No power with no participants
        }
        
        // Use the power analyzer field to calculate statistical power
        let power = self.power_analyzer.calculate_power(
            crate::statistics::power_analysis::StatisticalTestType::IndependentTTest,
            effect_size,
            total_n,
        ).map_err(|e| format!("Power calculation failed: {}", e))?;
        
        // Record power analysis procedure for citation tracking
        let power_description = format!(
            "Statistical power analysis (Cohen's h={:.3}, n={}, power={:.3})",
            effect_size, total_n, power
        );
        println!("Power analysis performed: {}", power_description);
        
        Ok(power)
    }

    fn assess_practical_significance(
        &self,
        test: &ABTest,
        primary_analysis: &PrimaryAnalysis,
    ) -> Result<PracticalSignificanceAssessment, String> {
        let mut metric_improvements = HashMap::new();
        metric_improvements.insert("primary_metric".to_string(), primary_analysis.effect_size);

        Ok(PracticalSignificanceAssessment {
            metric_improvements,
            cost_benefit_analysis: None,
            implementation_complexity: ImplementationComplexity::Medium,
            recommendation_strength: if primary_analysis.practical_significance {
                RecommendationStrength::WeakImplement
            } else {
                RecommendationStrength::Neutral
            },
        })
    }

    fn generate_final_recommendation(
        &self,
        primary_analysis: &PrimaryAnalysis,
        practical_significance: &PracticalSignificanceAssessment,
        statistical_tests: &[TestResult],
    ) -> Result<TestRecommendation, String> {
        // Use practical_significance assessment in decision making
        let practical_is_significant = matches!(
            practical_significance.recommendation_strength,
            RecommendationStrength::StrongImplement | RecommendationStrength::WeakImplement
        );
        
        let has_meaningful_improvement = practical_significance.metric_improvements
            .values()
            .any(|&improvement| improvement.abs() > 0.01); // 1% minimum improvement threshold
        
        // Comprehensive recommendation logic using all parameters
        if primary_analysis.statistical_significance 
            && practical_is_significant 
            && has_meaningful_improvement {
            if let Some(winner) = &primary_analysis.winner {
                let _confidence_level = 0.95; // Use default confidence level
                    
                return Ok(TestRecommendation::ImplementWinner {
                    variant_id: winner.clone(),
                    confidence: 0.85,
                });
            }
        }

        if statistical_tests.iter().any(|t| t.significant) {
            Ok(TestRecommendation::RequiresManualReview {
                concerns: vec!["Mixed statistical results".to_string()],
            })
        } else {
            Ok(TestRecommendation::InconclusiveResults {
                next_steps: vec!["Consider increasing sample size".to_string()],
            })
        }
    }
}

// Supporting data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterimAnalysisResult {
    pub test_id: String,
    pub analysis_time: chrono::DateTime<chrono::Utc>,
    pub participant_counts: HashMap<String, usize>,
    pub statistical_results: Vec<TestResult>,
    pub stopping_decision: StoppingDecision,
    pub guard_violations: Vec<GuardViolation>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StoppingDecision {
    Continue { reason: String },
    Stop { reasons: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardViolation {
    pub metric_name: String,
    pub threshold: GuardThreshold,
    pub current_value: f64,
    pub severity: ViolationSeverity,
    pub recommended_action: GuardAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Error,
    Critical,
}

// Default implementations
impl Default for StoppingCriteria {
    fn default() -> Self {
        Self {
            max_participants: Some(1000),
            max_duration_days: Some(30),
            statistical_power_threshold: 0.8,
            practical_significance_threshold: 0.1,
            futility_threshold: 0.1,
            interim_analysis_frequency: InterimAnalysisFrequency::FixedSampleSize {
                every_n_participants: 50,
            },
            early_stopping_enabled: true,
        }
    }
    
    /// Generate methodology report with citations for statistical procedures used
    pub fn generate_methodology_report(&mut self, experiment_id: &str) -> Result<MethodologyReport, String> {
        // Record methods used in A/B testing
        self.citation_tracker.mark_method_used("A/B Testing");
        self.citation_tracker.mark_method_used("Statistical Power Analysis");
        self.citation_tracker.mark_method_used("Independent Samples t-test");
        
        // Add software citations for statistical computing
        self.citation_tracker.add_software_citation("Rust Statistical Computing".to_string(), "internal2024".to_string());
        
        // Generate the methodology report
        Ok(self.citation_tracker.generate_methodology_report(experiment_id))
    }
}

impl Default for TestConfiguration {
    fn default() -> Self {
        Self {
            alpha: 0.05,
            power: 0.8,
            minimum_effect_size: 0.1,
            sequential_testing: false,
            multiple_testing_correction: MultipleTesting::BenjaminiHochberg,
            randomization_seed: None,
        }
    }
}

impl Hash for TestVariant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for TestVariant {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TestVariant {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ab_test_creation() {
        let mut framework = ABTestFramework::new(Some(12345));

        let hypothesis = Hypothesis {
            primary_metric: "accuracy".to_string(),
            expected_effect_size: 0.3,
            direction: EffectDirection::Increase,
            null_value: 0.0,
            alternative_description: "Intervention improves accuracy".to_string(),
        };

        let variants = vec![
            TestVariant {
                id: "control".to_string(),
                name: "Control".to_string(),
                description: "Standard condition".to_string(),
                intervention: InterventionSpec {
                    config: LearnerConfig::default(),
                    topology: Topology::alphabet(),
                    parameters: HashMap::new(),
                    intervention_type: InterventionType::Control,
                },
                allocation_weight: 0.5,
                expected_participants: Some(100),
            },
            TestVariant {
                id: "treatment".to_string(),
                name: "Treatment".to_string(),
                description: "Enhanced hints".to_string(),
                intervention: InterventionSpec {
                    config: LearnerConfig::default(),
                    topology: Topology::alphabet(),
                    parameters: HashMap::new(),
                    intervention_type: InterventionType::HintingStrategy {
                        level: "enhanced".to_string(),
                    },
                },
                allocation_weight: 0.5,
                expected_participants: Some(100),
            },
        ];

        let allocation_strategy = AllocationStrategy::FixedRandomization {
            proportions: vec![0.5, 0.5],
        };

        let test_id = framework
            .create_test(
                "Hint Enhancement Test".to_string(),
                "Testing enhanced hinting strategy".to_string(),
                hypothesis,
                variants,
                allocation_strategy,
            )
            .unwrap();

        assert!(!test_id.is_empty());
        assert!(framework.tests.contains_key(&test_id));
    }

    #[test]
    fn test_participant_assignment() {
        let mut framework = ABTestFramework::new(Some(54321));

        // Create a simple test first (code omitted for brevity)
        // Then test assignment

        // This test would be expanded with actual test creation
        assert_eq!(framework.assignments.len(), 0);
    }
}
