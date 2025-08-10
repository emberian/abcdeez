use crate::core::learner::LearnerModel;
use crate::statistics::validation::StatisticalValidator;
use crate::tasks::types::TaskGenerator;
use crate::core::topology::Topology;
use chrono::{DateTime, Utc};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/// Complete experimental framework for running and analyzing learning experiments
pub struct ExperimentFramework {
    pub experiments: Vec<Experiment>,
    pub current_experiment: Option<usize>,
    pub output_directory: String,
    pub random_seed: Option<u64>,
    rng: StdRng, // Properly managed RNG for reproducibility
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: ExperimentConfig,
    pub conditions: Vec<ExperimentCondition>,
    pub participants: Vec<Participant>,
    pub sessions: Vec<ExperimentSession>,
    pub results: Option<ExperimentResults>,
    pub metadata: ExperimentMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub topology_type: String,
    pub n_participants: usize,
    pub n_sessions_per_participant: usize,
    pub n_trials_per_session: usize,
    pub adaptive_scheduling: bool,
    pub use_bayesian_model: bool,
    pub use_strategy_mixture: bool,
    pub use_hierarchical_model: bool,
    pub use_transfer_learning: bool,
    pub use_macro_learning: bool,
    pub randomization: RandomizationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomizationConfig {
    pub randomize_conditions: bool,
    pub randomize_trials: bool,
    pub counterbalance: bool,
    pub block_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentCondition {
    pub name: String,
    pub parameters: HashMap<String, f64>,
    pub task_distribution: TaskDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDistribution {
    pub successor_prob: f64,
    pub predecessor_prob: f64,
    pub k_jump_prob: f64,
    pub segment_prob: f64,
    pub pairwise_prob: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub condition: String,
    pub demographics: Option<Demographics>,
    pub learner_model: Option<String>, // Serialized model
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demographics {
    pub age: Option<u32>,
    pub education_level: Option<String>,
    pub prior_experience: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub accuracy: f64,
    pub mean_response_time: f64,
    pub learning_curve: Vec<f64>,
    pub strategy_profile: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSession {
    pub participant_id: String,
    pub session_number: usize,
    pub condition: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub trials: Vec<Trial>,
    pub session_metrics: SessionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trial {
    pub trial_number: usize,
    pub task_type: String,
    pub stimulus: String,
    pub correct_response: String,
    pub participant_response: String,
    pub response_time: f64,
    pub correct: bool,
    pub eig_value: Option<f64>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub total_trials: usize,
    pub correct_trials: usize,
    pub mean_rt: f64,
    pub median_rt: f64,
    pub strategy_switches: usize,
    pub macro_usage_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    pub summary_statistics: SummaryStatistics,
    pub hypothesis_tests: Vec<HypothesisTest>,
    pub effect_sizes: HashMap<String, f64>,
    pub model_comparisons: Vec<ModelComparison>,
    pub visualizations: Vec<String>, // Paths to generated plots
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStatistics {
    pub overall_accuracy: f64,
    pub accuracy_by_condition: HashMap<String, f64>,
    pub mean_rt_overall: f64,
    pub mean_rt_by_condition: HashMap<String, f64>,
    pub learning_rate: f64,
    pub transfer_efficiency: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisTest {
    pub name: String,
    pub test_type: String,
    pub statistic: f64,
    pub p_value: f64,
    pub significant: bool,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparison {
    pub model1: String,
    pub model2: String,
    pub aic_difference: f64,
    pub bic_difference: f64,
    pub bayes_factor: Option<f64>,
    pub preferred_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
    pub researcher: String,
    pub notes: String,
}

impl ExperimentFramework {
    pub fn new(output_directory: String) -> Self {
        ExperimentFramework {
            experiments: Vec::new(),
            current_experiment: None,
            output_directory,
            random_seed: None,
            rng: StdRng::from_entropy(), // Initialize with system entropy
        }
    }

    /// Set seed for reproducibility (must be called before run_experiment)
    pub fn set_seed(&mut self, seed: u64) {
        self.random_seed = Some(seed);
        self.rng = StdRng::seed_from_u64(seed);
    }

    /// Create a new experiment
    pub fn create_experiment(&mut self, name: String, config: ExperimentConfig) -> String {
        let id = format!("exp_{}", Utc::now().timestamp());

        let experiment = Experiment {
            id: id.clone(),
            name,
            description: String::new(),
            config,
            conditions: Vec::new(),
            participants: Vec::new(),
            sessions: Vec::new(),
            results: None,
            metadata: ExperimentMetadata {
                created_at: Utc::now(),
                updated_at: Utc::now(),
                version: "1.0.0".to_string(),
                researcher: String::new(),
                notes: String::new(),
            },
        };

        self.experiments.push(experiment);
        self.current_experiment = Some(self.experiments.len() - 1);

        id
    }

    /// Add experimental condition
    pub fn add_condition(&mut self, condition: ExperimentCondition) {
        if let Some(idx) = self.current_experiment {
            self.experiments[idx].conditions.push(condition);
        }
    }

    /// Run the current experiment
    pub fn run_experiment(&mut self) -> Result<(), String> {
        let exp_idx = self.current_experiment.ok_or("No current experiment")?;

        // Ensure RNG is properly seeded for reproducibility
        // Seed should be set via set_seed() before calling this method

        // Get config values before mutable operations
        let (n_participants, n_sessions) = {
            let experiment = &self.experiments[exp_idx];
            (
                experiment.config.n_participants,
                experiment.config.n_sessions_per_participant,
            )
        };

        // Create participants
        self.create_participants(exp_idx)?;

        // Assign conditions
        self.assign_conditions(exp_idx)?;

        // Run sessions for each participant
        for participant_idx in 0..n_participants {
            for session_num in 0..n_sessions {
                self.run_session(exp_idx, participant_idx, session_num)?;
            }
        }

        // Analyze results
        self.analyze_experiment(exp_idx)?;

        // Save results
        self.save_experiment(exp_idx)?;

        Ok(())
    }

    fn create_participants(&mut self, exp_idx: usize) -> Result<(), String> {
        let experiment = &mut self.experiments[exp_idx];
        let n_participants = experiment.config.n_participants;

        for i in 0..n_participants {
            let participant = Participant {
                id: format!("P{:03}", i + 1),
                condition: String::new(), // Will be assigned later
                demographics: None,
                learner_model: None,
                performance_metrics: PerformanceMetrics {
                    accuracy: 0.0,
                    mean_response_time: 0.0,
                    learning_curve: Vec::new(),
                    strategy_profile: HashMap::new(),
                },
            };

            experiment.participants.push(participant);
        }

        Ok(())
    }

    fn assign_conditions(&mut self, exp_idx: usize) -> Result<(), String> {
        let experiment = &mut self.experiments[exp_idx];

        if experiment.conditions.is_empty() {
            return Err("No conditions defined".to_string());
        }

        let n_conditions = experiment.conditions.len();
        let _n_participants = experiment.participants.len();

        // Assign conditions (round-robin or randomized)
        if experiment.config.randomization.randomize_conditions {
            for participant in &mut experiment.participants {
                let condition_idx = self.rng.gen_range(0..n_conditions);
                participant.condition = experiment.conditions[condition_idx].name.clone();
            }
        } else {
            // Counterbalanced assignment
            for (i, participant) in experiment.participants.iter_mut().enumerate() {
                let condition_idx = i % n_conditions;
                participant.condition = experiment.conditions[condition_idx].name.clone();
            }
        }

        Ok(())
    }

    fn run_session(
        &mut self,
        exp_idx: usize,
        participant_idx: usize,
        session_num: usize,
    ) -> Result<(), String> {
        // Get necessary data before mutable operations
        let (participant_id, participant_condition) = {
            let experiment = &self.experiments[exp_idx];
            let participant = &experiment.participants[participant_idx];
            (participant.id.clone(), participant.condition.clone())
        };

        let condition = {
            let experiment = &self.experiments[exp_idx];
            experiment
                .conditions
                .iter()
                .find(|c| c.name == participant_condition)
                .ok_or("Condition not found")?
                .clone()
        };

        // Create topology
        let topology = Topology::alphabet();

        // Create learner model
        let mut learner_model = LearnerModel::new(participant_id.clone(), &topology);

        // Create task generator
        let mut task_generator = TaskGenerator::new(topology.clone());

        // Create session
        let mut session = ExperimentSession {
            participant_id: participant_id.clone(),
            session_number: session_num,
            condition: participant_condition.clone(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            trials: Vec::new(),
            session_metrics: SessionMetrics {
                total_trials: 0,
                correct_trials: 0,
                mean_rt: 0.0,
                median_rt: 0.0,
                strategy_switches: 0,
                macro_usage_count: 0,
            },
        };

        // Run trials
        let n_trials = self.experiments[exp_idx].config.n_trials_per_session;
        let mut response_times = Vec::new();

        for trial_num in 0..n_trials {
            // Generate task based on condition distribution
            let task = self.generate_task_for_condition(&mut task_generator, &topology, &condition);

            // Simulate participant response
            let (response, rt, correct) = self.simulate_response(&mut learner_model, &task);

            // Record trial
            let trial = Trial {
                trial_number: trial_num,
                task_type: format!("{:?}", task.task_type),
                stimulus: task.prompt.clone(),
                correct_response: task.correct_answer.clone(),
                participant_response: response,
                response_time: rt,
                correct,
                eig_value: None,
                confidence: None,
            };

            session.trials.push(trial);
            response_times.push(rt);

            if correct {
                session.session_metrics.correct_trials += 1;
            }

            // Update learner model
            learner_model.update_operation_proficiency(&task.operation, correct);
            if let crate::tasks::TaskType::Successor { ref item }
            | crate::tasks::TaskType::Predecessor { ref item } = task.task_type
            {
                learner_model.update_memory_strength(item, correct);
            }
        }

        // Update session metrics
        session.session_metrics.total_trials = n_trials;
        session.session_metrics.mean_rt = response_times.iter().sum::<f64>() / n_trials as f64;
        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        session.session_metrics.median_rt = response_times[n_trials / 2];

        session.end_time = Utc::now();

        // Update participant metrics
        let accuracy = session.session_metrics.correct_trials as f64 / n_trials as f64;

        // Store session and update participant
        let experiment = &mut self.experiments[exp_idx];
        experiment.sessions.push(session);
        experiment.participants[participant_idx]
            .performance_metrics
            .learning_curve
            .push(accuracy);

        Ok(())
    }

    fn generate_task_for_condition(
        &mut self,
        generator: &mut TaskGenerator,
        topology: &Topology,
        condition: &ExperimentCondition,
    ) -> crate::tasks::Task {
        let r: f64 = self.rng.gen();

        let dist = &condition.task_distribution;

        // Pick a random item from the topology
        let nodes = &topology.nodes;
        let random_item = nodes[self.rng.gen_range(0..nodes.len())].label.clone();

        let task_type = if r < dist.successor_prob {
            crate::tasks::TaskType::Successor { item: random_item }
        } else if r < dist.successor_prob + dist.predecessor_prob {
            crate::tasks::TaskType::Predecessor { item: random_item }
        } else if r < dist.successor_prob + dist.predecessor_prob + dist.k_jump_prob {
            crate::tasks::TaskType::KJump {
                start: random_item,
                k: 2,
            }
        } else if r < dist.successor_prob
            + dist.predecessor_prob
            + dist.k_jump_prob
            + dist.segment_prob
        {
            crate::tasks::TaskType::Segment {
                start: random_item,
                count: 3,
                reverse: false,
            }
        } else {
            let random_item2 = nodes[self.rng.gen_range(0..nodes.len())].label.clone();
            crate::tasks::TaskType::PairwiseOrder {
                a: random_item,
                b: random_item2,
            }
        };

        generator.generate_task(Some(task_type))
    }

    fn simulate_response(
        &mut self,
        model: &mut LearnerModel,
        task: &crate::tasks::Task,
    ) -> (String, f64, bool) {
        // Use model-based prediction instead of arbitrary simulation
        let op_key = format!("{:?}", task.operation);
        let proficiency = model
            .operation_proficiencies
            .get(&op_key)
            .map(|p| Self::sigmoid(p.theta))
            .unwrap_or(0.5);

        // Calculate probability of correct response based on proficiency and difficulty
        let success_prob = Self::sigmoid(proficiency - task.difficulty);

        // Generate response based on model prediction
        let correct = self.rng.gen::<f64>() < success_prob;

        let response = if correct {
            task.correct_answer.clone()
        } else {
            // Select incorrect answer weighted by confusability
            let incorrect_options: Vec<_> = task
                .options
                .iter()
                .filter(|&opt| opt != &task.correct_answer)
                .cloned()
                .collect();

            if incorrect_options.is_empty() {
                task.correct_answer.clone()
            } else {
                incorrect_options[self.rng.gen_range(0..incorrect_options.len())].clone()
            }
        };

        // Response time based on difficulty and proficiency
        // RT = base_rt * exp(difficulty - proficiency/2)
        let base_rt = 1.5; // seconds
        let rt_mean = base_rt * (task.difficulty - proficiency / 2.0).exp();
        let rt_noise = self.rng.gen::<f64>() * 0.5 - 0.25; // ±0.25s noise
        let rt = (rt_mean + rt_noise).max(0.3); // Minimum 300ms

        // Update model based on response
        model.update_operation_proficiency(&task.operation, correct);

        (response, rt, correct)
    }

    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    fn analyze_experiment(&mut self, exp_idx: usize) -> Result<(), String> {
        let experiment = &mut self.experiments[exp_idx];

        // Calculate summary statistics
        let mut total_correct = 0;
        let mut total_trials = 0;
        let mut all_rts = Vec::new();
        let mut accuracy_by_condition: HashMap<String, Vec<f64>> = HashMap::new();
        let mut rt_by_condition: HashMap<String, Vec<f64>> = HashMap::new();

        for session in &experiment.sessions {
            total_correct += session.session_metrics.correct_trials;
            total_trials += session.session_metrics.total_trials;

            for trial in &session.trials {
                all_rts.push(trial.response_time);

                accuracy_by_condition
                    .entry(session.condition.clone())
                    .or_insert_with(Vec::new)
                    .push(if trial.correct { 1.0 } else { 0.0 });

                rt_by_condition
                    .entry(session.condition.clone())
                    .or_insert_with(Vec::new)
                    .push(trial.response_time);
            }
        }

        let overall_accuracy = total_correct as f64 / total_trials as f64;
        let mean_rt_overall = all_rts.iter().sum::<f64>() / all_rts.len() as f64;

        let mut condition_accuracies = HashMap::new();
        let mut condition_rts = HashMap::new();

        for (condition, accuracies) in &accuracy_by_condition {
            let mean_acc = accuracies.iter().sum::<f64>() / accuracies.len() as f64;
            condition_accuracies.insert(condition.clone(), mean_acc);
        }

        for (condition, rts) in &rt_by_condition {
            let mean_rt = rts.iter().sum::<f64>() / rts.len() as f64;
            condition_rts.insert(condition.clone(), mean_rt);
        }

        // Perform hypothesis tests
        let validator = StatisticalValidator::new(0.95);
        let mut hypothesis_tests = Vec::new();

        // Test for condition differences
        if experiment.conditions.len() >= 2 {
            let groups: Vec<Vec<f64>> = accuracy_by_condition.values().cloned().collect();
            let anova_result = validator.anova(groups);

            hypothesis_tests.push(HypothesisTest {
                name: "Condition Effect on Accuracy".to_string(),
                test_type: "ANOVA".to_string(),
                statistic: anova_result.statistic,
                p_value: anova_result.p_value,
                significant: anova_result.significant,
                interpretation: if anova_result.significant {
                    "Significant differences between conditions".to_string()
                } else {
                    "No significant differences between conditions".to_string()
                },
            });
        }

        // Store results
        experiment.results = Some(ExperimentResults {
            summary_statistics: SummaryStatistics {
                overall_accuracy,
                accuracy_by_condition: condition_accuracies,
                mean_rt_overall,
                mean_rt_by_condition: condition_rts,
                learning_rate: 0.1, // Placeholder
                transfer_efficiency: None,
            },
            hypothesis_tests,
            effect_sizes: HashMap::new(),
            model_comparisons: Vec::new(),
            visualizations: Vec::new(),
        });

        Ok(())
    }

    fn save_experiment(&self, exp_idx: usize) -> Result<(), String> {
        let experiment = &self.experiments[exp_idx];

        // Create output file
        let filename = format!("{}/{}.json", self.output_directory, experiment.id);
        let mut file = File::create(filename).map_err(|e| e.to_string())?;

        // Serialize experiment
        let json = serde_json::to_string_pretty(experiment).map_err(|e| e.to_string())?;

        // Write to file
        file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Load experiment from file
    pub fn load_experiment(&mut self, filename: &str) -> Result<String, String> {
        let file = std::fs::File::open(filename).map_err(|e| e.to_string())?;
        let experiment: Experiment = serde_json::from_reader(file).map_err(|e| e.to_string())?;

        let id = experiment.id.clone();
        self.experiments.push(experiment);
        self.current_experiment = Some(self.experiments.len() - 1);

        Ok(id)
    }

    /// Generate experiment report
    pub fn generate_report(&self, exp_idx: usize) -> String {
        let experiment = &self.experiments[exp_idx];

        let mut report = String::new();
        report.push_str(&format!("# Experiment Report: {}\n\n", experiment.name));
        report.push_str(&format!("## Metadata\n"));
        report.push_str(&format!("- ID: {}\n", experiment.id));
        report.push_str(&format!("- Created: {}\n", experiment.metadata.created_at));
        report.push_str(&format!(
            "- Researcher: {}\n\n",
            experiment.metadata.researcher
        ));

        report.push_str(&format!("## Configuration\n"));
        report.push_str(&format!(
            "- Participants: {}\n",
            experiment.config.n_participants
        ));
        report.push_str(&format!(
            "- Sessions per participant: {}\n",
            experiment.config.n_sessions_per_participant
        ));
        report.push_str(&format!(
            "- Trials per session: {}\n\n",
            experiment.config.n_trials_per_session
        ));

        if let Some(results) = &experiment.results {
            report.push_str(&format!("## Results\n"));
            report.push_str(&format!("### Overall Performance\n"));
            report.push_str(&format!(
                "- Accuracy: {:.2}%\n",
                results.summary_statistics.overall_accuracy * 100.0
            ));
            report.push_str(&format!(
                "- Mean RT: {:.2}s\n\n",
                results.summary_statistics.mean_rt_overall
            ));

            report.push_str(&format!("### Performance by Condition\n"));
            for (condition, accuracy) in &results.summary_statistics.accuracy_by_condition {
                report.push_str(&format!("- {}: {:.2}%\n", condition, accuracy * 100.0));
            }

            report.push_str(&format!("\n### Statistical Tests\n"));
            for test in &results.hypothesis_tests {
                report.push_str(&format!(
                    "- {}: {} (p={:.4})\n",
                    test.name, test.interpretation, test.p_value
                ));
            }
        }

        report
    }
}

/// Prebuilt experiment templates
pub struct ExperimentTemplates;

impl ExperimentTemplates {
    pub fn basic_learning_experiment() -> ExperimentConfig {
        ExperimentConfig {
            topology_type: "alphabet".to_string(),
            n_participants: 30,
            n_sessions_per_participant: 5,
            n_trials_per_session: 50,
            adaptive_scheduling: false,
            use_bayesian_model: true,
            use_strategy_mixture: false,
            use_hierarchical_model: false,
            use_transfer_learning: false,
            use_macro_learning: false,
            randomization: RandomizationConfig {
                randomize_conditions: true,
                randomize_trials: true,
                counterbalance: true,
                block_size: Some(10),
            },
        }
    }

    pub fn transfer_learning_experiment() -> ExperimentConfig {
        ExperimentConfig {
            topology_type: "alphabet".to_string(),
            n_participants: 40,
            n_sessions_per_participant: 8,
            n_trials_per_session: 60,
            adaptive_scheduling: true,
            use_bayesian_model: true,
            use_strategy_mixture: true,
            use_hierarchical_model: false,
            use_transfer_learning: true,
            use_macro_learning: true,
            randomization: RandomizationConfig {
                randomize_conditions: true,
                randomize_trials: false,
                counterbalance: true,
                block_size: Some(15),
            },
        }
    }

    pub fn hierarchical_modeling_experiment() -> ExperimentConfig {
        ExperimentConfig {
            topology_type: "alphabet".to_string(),
            n_participants: 100,
            n_sessions_per_participant: 10,
            n_trials_per_session: 40,
            adaptive_scheduling: true,
            use_bayesian_model: true,
            use_strategy_mixture: true,
            use_hierarchical_model: true,
            use_transfer_learning: false,
            use_macro_learning: true,
            randomization: RandomizationConfig {
                randomize_conditions: false,
                randomize_trials: true,
                counterbalance: false,
                block_size: None,
            },
        }
    }
}
