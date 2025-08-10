use crate::core::learner::LearnerModel;
use crate::tasks::types::{Task, TaskType};
use crate::core::topology::Topology;
// Define our own strategy types for the mixture model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MixtureStrategyType {
    Sequential,
    Chunking,
    Anchoring,
    Random,
    Pattern,
}
use rand::prelude::*;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use statrs::distribution::{Continuous, Normal};
use std::collections::HashMap;

/// Represents a cognitive strategy for solving tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveStrategy {
    pub name: String,
    pub strategy_type: MixtureStrategyType,
    pub parameters: StrategyParameters,
    pub activation_probability: f64,
    pub performance_history: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParameters {
    pub chunk_size: Option<usize>,
    pub anchor_points: Vec<String>,
    pub search_depth: usize,
    pub memory_span: usize,
    pub learning_rate: f64,
}

/// Mixture model for multiple strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMixtureModel {
    pub strategies: Vec<CognitiveStrategy>,
    pub mixture_weights: Vec<f64>,
    pub transition_matrix: Vec<Vec<f64>>,
    pub current_strategy_index: usize,
    pub evidence_accumulator: HashMap<String, Vec<f64>>,
    #[serde(skip)]
    #[serde(default = "default_rng")]
    rng: rand::rngs::StdRng,
}

fn default_rng() -> rand::rngs::StdRng {
    use rand::SeedableRng;
    rand::rngs::StdRng::from_entropy()
}

impl StrategyMixtureModel {
    pub fn new(topology: &Topology) -> Self {
        Self::with_seed(topology, None)
    }

    pub fn with_seed(topology: &Topology, seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };

        let strategies = Self::initialize_strategies(topology);
        let n = strategies.len();
        let mixture_weights = vec![1.0 / n as f64; n];
        let transition_matrix = vec![vec![1.0 / n as f64; n]; n];

        StrategyMixtureModel {
            strategies,
            mixture_weights,
            transition_matrix,
            current_strategy_index: 0,
            evidence_accumulator: HashMap::new(),
            rng,
        }
    }

    fn initialize_strategies(topology: &Topology) -> Vec<CognitiveStrategy> {
        vec![
            // Sequential strategy
            CognitiveStrategy {
                name: "Sequential".to_string(),
                strategy_type: MixtureStrategyType::Sequential,
                parameters: StrategyParameters {
                    chunk_size: None,
                    anchor_points: vec![],
                    search_depth: 1,
                    memory_span: 3,
                    learning_rate: 0.8,
                },
                activation_probability: 0.3,
                performance_history: Vec::new(),
            },
            // Chunking strategy (7±2 items)
            CognitiveStrategy {
                name: "Chunking".to_string(),
                strategy_type: MixtureStrategyType::Chunking,
                parameters: StrategyParameters {
                    chunk_size: Some(7),
                    anchor_points: vec![],
                    search_depth: 2,
                    memory_span: 7,
                    learning_rate: 0.6,
                },
                activation_probability: 0.25,
                performance_history: Vec::new(),
            },
            // Anchoring strategy (landmark-based)
            CognitiveStrategy {
                name: "Anchoring".to_string(),
                strategy_type: MixtureStrategyType::Anchoring,
                parameters: StrategyParameters {
                    chunk_size: None,
                    anchor_points: Self::identify_anchors(topology),
                    search_depth: 3,
                    memory_span: 5,
                    learning_rate: 0.7,
                },
                activation_probability: 0.2,
                performance_history: Vec::new(),
            },
            // Random search strategy
            CognitiveStrategy {
                name: "Random Search".to_string(),
                strategy_type: MixtureStrategyType::Random,
                parameters: StrategyParameters {
                    chunk_size: None,
                    anchor_points: vec![],
                    search_depth: 10,
                    memory_span: 1,
                    learning_rate: 0.3,
                },
                activation_probability: 0.15,
                performance_history: Vec::new(),
            },
            // Pattern-based strategy
            CognitiveStrategy {
                name: "Pattern Recognition".to_string(),
                strategy_type: MixtureStrategyType::Pattern,
                parameters: StrategyParameters {
                    chunk_size: Some(4),
                    anchor_points: vec![],
                    search_depth: 4,
                    memory_span: 8,
                    learning_rate: 0.9,
                },
                activation_probability: 0.1,
                performance_history: Vec::new(),
            },
        ]
    }

    fn identify_anchors(topology: &Topology) -> Vec<String> {
        // Identify landmark nodes (beginning, end, quarter points)
        let n = topology.nodes.len();
        let mut anchors = Vec::new();

        if n > 0 {
            anchors.push(topology.nodes[0].label.clone());
            if n > 1 {
                anchors.push(topology.nodes[n - 1].label.clone());
            }
            if n > 4 {
                anchors.push(topology.nodes[n / 4].label.clone());
                anchors.push(topology.nodes[n / 2].label.clone());
                anchors.push(topology.nodes[3 * n / 4].label.clone());
            }
        }

        anchors
    }

    /// Update mixture weights based on task performance
    pub fn update_weights(&mut self, task: &Task, success: bool, response_time: f64) {
        // Calculate likelihood of each strategy given the performance
        let mut likelihoods = Vec::new();

        for (_i, strategy) in self.strategies.iter().enumerate() {
            let likelihood =
                self.calculate_strategy_likelihood(strategy, task, success, response_time);
            likelihoods.push(likelihood);
        }

        // Update mixture weights using Bayesian update
        let total_evidence: f64 = self
            .mixture_weights
            .iter()
            .zip(&likelihoods)
            .map(|(w, l)| w * l)
            .sum();

        if total_evidence > 0.0 {
            for i in 0..self.mixture_weights.len() {
                self.mixture_weights[i] =
                    (self.mixture_weights[i] * likelihoods[i]) / total_evidence;
            }
        }

        // Add smoothing to prevent weights from going to zero
        let epsilon = 0.01;
        for weight in &mut self.mixture_weights {
            *weight = (*weight * (1.0 - epsilon)) + (epsilon / self.strategies.len() as f64);
        }

        // Normalize
        let sum: f64 = self.mixture_weights.iter().sum();
        for weight in &mut self.mixture_weights {
            *weight /= sum;
        }

        // Update performance history for current strategy
        let performance = if success { 1.0 } else { 0.0 };
        self.strategies[self.current_strategy_index]
            .performance_history
            .push(performance);
    }

    fn calculate_strategy_likelihood(
        &self,
        strategy: &CognitiveStrategy,
        task: &Task,
        success: bool,
        response_time: f64,
    ) -> f64 {
        // Model likelihood based on strategy characteristics
        let mut likelihood = 1.0;

        // Success rate component
        let expected_success = match strategy.strategy_type {
            MixtureStrategyType::Sequential => {
                if matches!(task.task_type, TaskType::Successor { .. }) {
                    0.9
                } else {
                    0.5
                }
            }
            MixtureStrategyType::Chunking => {
                if let TaskType::Segment { count, .. } = &task.task_type {
                    if *count <= strategy.parameters.chunk_size.unwrap_or(7) {
                        0.85
                    } else {
                        0.4
                    }
                } else {
                    0.6
                }
            }
            MixtureStrategyType::Anchoring => {
                if strategy
                    .parameters
                    .anchor_points
                    .contains(&task.correct_answer)
                {
                    0.95
                } else {
                    0.6
                }
            }
            MixtureStrategyType::Random => 0.25,
            MixtureStrategyType::Pattern => 0.7,
        };

        // Bernoulli likelihood for success
        likelihood *= if success {
            expected_success
        } else {
            1.0 - expected_success
        };

        // Response time component (log-normal distribution)
        let expected_rt: f64 = match strategy.strategy_type {
            MixtureStrategyType::Sequential => 1.5,
            MixtureStrategyType::Chunking => 2.0,
            MixtureStrategyType::Anchoring => 1.2,
            MixtureStrategyType::Random => 3.0,
            MixtureStrategyType::Pattern => 1.8,
        };

        let rt_std = 0.5f64;
        let normal = Normal::new(expected_rt.ln(), rt_std).unwrap();
        likelihood *= normal.pdf(response_time.ln()).exp();

        likelihood
    }

    /// Select next strategy based on mixture weights and transition probabilities
    pub fn select_strategy(&mut self) -> usize {
        // Combine mixture weights with transition probabilities
        let mut probabilities = Vec::new();
        for i in 0..self.strategies.len() {
            let mixture_prob = self.mixture_weights[i];
            let transition_prob = self.transition_matrix[self.current_strategy_index][i];
            probabilities.push(mixture_prob * 0.7 + transition_prob * 0.3);
        }

        // Normalize
        let sum: f64 = probabilities.iter().sum();
        for prob in &mut probabilities {
            *prob /= sum;
        }

        // Sample from categorical distribution
        let r: f64 = self.rng.gen();
        let mut cumsum = 0.0;
        for (i, &prob) in probabilities.iter().enumerate() {
            cumsum += prob;
            if r < cumsum {
                self.current_strategy_index = i;
                return i;
            }
        }

        self.current_strategy_index
    }

    /// Generate response based on current strategy
    pub fn generate_response(&mut self, task: &Task, model: &LearnerModel) -> (String, f64) {
        let strategy = &self.strategies[self.current_strategy_index];

        match strategy.strategy_type {
            MixtureStrategyType::Sequential => {
                // Use sequential navigation
                let confidence = model
                    .operation_proficiencies
                    .get("Successor")
                    .map(|p| p.theta)
                    .unwrap_or(0.5);

                if self.rng.gen::<f64>() < confidence {
                    (task.correct_answer.clone(), confidence)
                } else {
                    // Make sequential error
                    let options = &task.options;
                    let idx = self.rng.gen_range(0..options.len());
                    (options[idx].clone(), confidence * 0.5)
                }
            }

            MixtureStrategyType::Chunking => {
                // Use chunk-based retrieval
                let chunk_size = strategy.parameters.chunk_size.unwrap_or(7);
                let in_chunk = task.options.len() <= chunk_size;

                let confidence = if in_chunk { 0.85 } else { 0.5 };

                if self.rng.gen::<f64>() < confidence {
                    (task.correct_answer.clone(), confidence)
                } else {
                    let idx = self.rng.gen_range(0..task.options.len());
                    (task.options[idx].clone(), confidence * 0.5)
                }
            }

            MixtureStrategyType::Anchoring => {
                // Use landmark-based navigation
                let is_anchor = strategy
                    .parameters
                    .anchor_points
                    .contains(&task.correct_answer);

                let confidence = if is_anchor { 0.95 } else { 0.6 };

                if self.rng.gen::<f64>() < confidence {
                    (task.correct_answer.clone(), confidence)
                } else {
                    // Error biased toward anchors
                    if !strategy.parameters.anchor_points.is_empty() && self.rng.gen_bool(0.5) {
                        let idx = self
                            .rng
                            .gen_range(0..strategy.parameters.anchor_points.len());
                        (
                            strategy.parameters.anchor_points[idx].clone(),
                            confidence * 0.3,
                        )
                    } else {
                        let idx = self.rng.gen_range(0..task.options.len());
                        (task.options[idx].clone(), confidence * 0.5)
                    }
                }
            }

            MixtureStrategyType::Random => {
                // Random guessing
                let idx = self.rng.gen_range(0..task.options.len());
                (task.options[idx].clone(), 0.25)
            }

            MixtureStrategyType::Pattern => {
                // Pattern-based response
                let confidence = model
                    .operation_proficiencies
                    .values()
                    .map(|p| p.theta)
                    .sum::<f64>()
                    / model.operation_proficiencies.len() as f64;

                if self.rng.gen::<f64>() < confidence {
                    (task.correct_answer.clone(), confidence)
                } else {
                    let idx = self.rng.gen_range(0..task.options.len());
                    (task.options[idx].clone(), confidence * 0.5)
                }
            }
        }
    }

    /// Update transition matrix based on strategy switches
    pub fn update_transitions(&mut self, from_idx: usize, to_idx: usize, success: bool) {
        let learning_rate = 0.1;
        let boost = if success { 0.1 } else { -0.05 };

        // Update transition probability
        self.transition_matrix[from_idx][to_idx] += learning_rate * boost;

        // Ensure probabilities remain valid
        for row in &mut self.transition_matrix {
            // Clamp to [0.01, 1.0]
            for prob in row.iter_mut() {
                *prob = prob.max(0.01).min(1.0);
            }

            // Normalize row
            let sum: f64 = row.iter().sum();
            for prob in row.iter_mut() {
                *prob /= sum;
            }
        }
    }

    /// Estimate model parameters using EM algorithm
    pub fn estimate_parameters(&mut self, performance_data: &[(Task, bool, f64)]) {
        const MAX_ITERATIONS: usize = 100;
        const CONVERGENCE_THRESHOLD: f64 = 1e-6;

        for _ in 0..MAX_ITERATIONS {
            let old_weights = self.mixture_weights.clone();

            // E-step: Calculate responsibilities
            let mut responsibilities =
                vec![vec![0.0; self.strategies.len()]; performance_data.len()];

            for (i, (task, success, rt)) in performance_data.iter().enumerate() {
                let mut likelihoods = Vec::new();
                for strategy in &self.strategies {
                    likelihoods
                        .push(self.calculate_strategy_likelihood(strategy, task, *success, *rt));
                }

                let total: f64 = self
                    .mixture_weights
                    .iter()
                    .zip(&likelihoods)
                    .map(|(w, l)| w * l)
                    .sum();

                for j in 0..self.strategies.len() {
                    responsibilities[i][j] =
                        (self.mixture_weights[j] * likelihoods[j]) / total.max(1e-10);
                }
            }

            // M-step: Update mixture weights
            for j in 0..self.strategies.len() {
                self.mixture_weights[j] = responsibilities.iter().map(|r| r[j]).sum::<f64>()
                    / performance_data.len() as f64;
            }

            // Check convergence
            let diff: f64 = old_weights
                .iter()
                .zip(&self.mixture_weights)
                .map(|(old, new)| (old - new).abs())
                .sum();

            if diff < CONVERGENCE_THRESHOLD {
                break;
            }
        }
    }

    /// Get dominant strategy
    pub fn get_dominant_strategy(&self) -> &CognitiveStrategy {
        let max_idx = self
            .mixture_weights
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        &self.strategies[max_idx]
    }

    /// Calculate entropy of mixture distribution
    pub fn calculate_entropy(&self) -> f64 {
        -self
            .mixture_weights
            .iter()
            .filter(|&&w| w > 0.0)
            .map(|w| w * w.ln())
            .sum::<f64>()
    }

    /// Predict performance for a given task
    pub fn predict_performance(&self, task: &Task) -> (f64, f64) {
        let mut expected_success = 0.0;
        let mut expected_rt = 0.0;

        for (strategy, &weight) in self.strategies.iter().zip(&self.mixture_weights) {
            let (success_prob, rt) = self.predict_with_strategy(strategy, task);
            expected_success += weight * success_prob;
            expected_rt += weight * rt;
        }

        (expected_success, expected_rt)
    }

    fn predict_with_strategy(&self, strategy: &CognitiveStrategy, task: &Task) -> (f64, f64) {
        match strategy.strategy_type {
            MixtureStrategyType::Sequential => {
                let success = if matches!(task.task_type, TaskType::Successor { .. }) {
                    0.9
                } else {
                    0.5
                };
                (success, 1.5)
            }
            MixtureStrategyType::Chunking => {
                let success = if let TaskType::Segment { count, .. } = &task.task_type {
                    if *count <= 7 {
                        0.85
                    } else {
                        0.4
                    }
                } else {
                    0.6
                };
                (success, 2.0)
            }
            MixtureStrategyType::Anchoring => {
                let success = if strategy
                    .parameters
                    .anchor_points
                    .contains(&task.correct_answer)
                {
                    0.95
                } else {
                    0.6
                };
                (success, 1.2)
            }
            MixtureStrategyType::Random => (0.25, 3.0),
            MixtureStrategyType::Pattern => (0.7, 1.8),
        }
    }
}

/// Hidden Markov Model for strategy dynamics
pub struct StrategyHMM {
    states: Vec<String>,
    initial_probs: Vec<f64>,
    transition_probs: Vec<Vec<f64>>,
    emission_probs: HashMap<String, Vec<f64>>,
}

impl StrategyHMM {
    pub fn new(strategies: &[CognitiveStrategy]) -> Self {
        let n = strategies.len();
        let states: Vec<String> = strategies.iter().map(|s| s.name.clone()).collect();
        let initial_probs = vec![1.0 / n as f64; n];
        let transition_probs = vec![vec![1.0 / n as f64; n]; n];
        let emission_probs = HashMap::new();

        StrategyHMM {
            states,
            initial_probs,
            transition_probs,
            emission_probs,
        }
    }

    /// Viterbi algorithm for finding most likely strategy sequence
    pub fn viterbi(&self, observations: &[String]) -> Vec<usize> {
        let n_states = self.states.len();
        let n_obs = observations.len();

        if n_obs == 0 {
            return Vec::new();
        }

        // Initialize
        let mut viterbi = vec![vec![0.0; n_states]; n_obs];
        let mut path = vec![vec![0; n_states]; n_obs];

        // First observation
        for i in 0..n_states {
            let emission = self
                .emission_probs
                .get(&observations[0])
                .and_then(|probs| probs.get(i))
                .unwrap_or(&0.1);
            viterbi[0][i] = self.initial_probs[i] * emission;
        }

        // Forward pass
        for t in 1..n_obs {
            for j in 0..n_states {
                let mut max_prob = 0.0;
                let mut max_state = 0;

                for i in 0..n_states {
                    let prob = viterbi[t - 1][i] * self.transition_probs[i][j];
                    if prob > max_prob {
                        max_prob = prob;
                        max_state = i;
                    }
                }

                let emission = self
                    .emission_probs
                    .get(&observations[t])
                    .and_then(|probs| probs.get(j))
                    .unwrap_or(&0.1);

                viterbi[t][j] = max_prob * emission;
                path[t][j] = max_state;
            }
        }

        // Backward pass
        let mut result = vec![0; n_obs];

        // Find most likely final state
        let mut max_prob = 0.0;
        let mut max_state = 0;
        for i in 0..n_states {
            if viterbi[n_obs - 1][i] > max_prob {
                max_prob = viterbi[n_obs - 1][i];
                max_state = i;
            }
        }
        result[n_obs - 1] = max_state;

        // Trace back
        for t in (0..n_obs - 1).rev() {
            result[t] = path[t + 1][result[t + 1]];
        }

        result
    }

    /// Baum-Welch algorithm for learning HMM parameters
    pub fn baum_welch(&mut self, observations: Vec<Vec<String>>) {
        const MAX_ITERATIONS: usize = 100;
        const CONVERGENCE_THRESHOLD: f64 = 1e-6;

        for _ in 0..MAX_ITERATIONS {
            let old_transitions = self.transition_probs.clone();

            // Forward-backward algorithm for each sequence
            for obs_seq in &observations {
                if obs_seq.is_empty() {
                    continue;
                }

                let (alpha, beta, gamma, xi) = self.forward_backward(obs_seq);

                // Update parameters
                self.update_parameters(&alpha, &beta, &gamma, &xi, obs_seq);
            }

            // Check convergence
            let diff: f64 = old_transitions
                .iter()
                .zip(&self.transition_probs)
                .map(|(old_row, new_row)| {
                    old_row
                        .iter()
                        .zip(new_row)
                        .map(|(old, new)| (old - new).abs())
                        .sum::<f64>()
                })
                .sum();

            if diff < CONVERGENCE_THRESHOLD {
                break;
            }
        }
    }

    fn forward_backward(
        &self,
        observations: &[String],
    ) -> (
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        Vec<Vec<f64>>,
        Vec<Vec<Vec<f64>>>,
    ) {
        let n_states = self.states.len();
        let n_obs = observations.len();

        // Forward pass (alpha)
        let mut alpha = vec![vec![0.0; n_states]; n_obs];
        for i in 0..n_states {
            let emission = self
                .emission_probs
                .get(&observations[0])
                .and_then(|probs| probs.get(i))
                .unwrap_or(&0.1);
            alpha[0][i] = self.initial_probs[i] * emission;
        }

        for t in 1..n_obs {
            for j in 0..n_states {
                alpha[t][j] = 0.0;
                for i in 0..n_states {
                    alpha[t][j] += alpha[t - 1][i] * self.transition_probs[i][j];
                }
                let emission = self
                    .emission_probs
                    .get(&observations[t])
                    .and_then(|probs| probs.get(j))
                    .unwrap_or(&0.1);
                alpha[t][j] *= emission;
            }
        }

        // Backward pass (beta)
        let mut beta = vec![vec![0.0; n_states]; n_obs];
        for i in 0..n_states {
            beta[n_obs - 1][i] = 1.0;
        }

        for t in (0..n_obs - 1).rev() {
            for i in 0..n_states {
                beta[t][i] = 0.0;
                for j in 0..n_states {
                    let emission = self
                        .emission_probs
                        .get(&observations[t + 1])
                        .and_then(|probs| probs.get(j))
                        .unwrap_or(&0.1);
                    beta[t][i] += self.transition_probs[i][j] * emission * beta[t + 1][j];
                }
            }
        }

        // Gamma (state probabilities)
        let mut gamma = vec![vec![0.0; n_states]; n_obs];
        for t in 0..n_obs {
            let sum: f64 = (0..n_states).map(|i| alpha[t][i] * beta[t][i]).sum();
            for i in 0..n_states {
                gamma[t][i] = (alpha[t][i] * beta[t][i]) / sum.max(1e-10);
            }
        }

        // Xi (transition probabilities)
        let mut xi = vec![vec![vec![0.0; n_states]; n_states]; n_obs - 1];
        for t in 0..n_obs - 1 {
            let mut sum = 0.0;
            for i in 0..n_states {
                for j in 0..n_states {
                    let emission = self
                        .emission_probs
                        .get(&observations[t + 1])
                        .and_then(|probs| probs.get(j))
                        .unwrap_or(&0.1);
                    xi[t][i][j] =
                        alpha[t][i] * self.transition_probs[i][j] * emission * beta[t + 1][j];
                    sum += xi[t][i][j];
                }
            }

            if sum > 0.0 {
                for i in 0..n_states {
                    for j in 0..n_states {
                        xi[t][i][j] /= sum;
                    }
                }
            }
        }

        (alpha, beta, gamma, xi)
    }

    fn update_parameters(
        &mut self,
        _alpha: &[Vec<f64>],
        _beta: &[Vec<f64>],
        gamma: &[Vec<f64>],
        xi: &[Vec<Vec<f64>>],
        observations: &[String],
    ) {
        let n_states = self.states.len();
        let n_obs = observations.len();

        // Update initial probabilities
        for i in 0..n_states {
            self.initial_probs[i] = gamma[0][i];
        }

        // Update transition probabilities
        for i in 0..n_states {
            let denominator: f64 = (0..n_obs - 1).map(|t| gamma[t][i]).sum();

            for j in 0..n_states {
                let numerator: f64 = (0..n_obs - 1).map(|t| xi[t][i][j]).sum();
                self.transition_probs[i][j] = numerator / denominator.max(1e-10);
            }
        }

        // Update emission probabilities
        for obs in observations {
            if !self.emission_probs.contains_key(obs) {
                self.emission_probs.insert(obs.clone(), vec![0.0; n_states]);
            }
        }

        for i in 0..n_states {
            let denominator: f64 = gamma.iter().map(|g| g[i]).sum();

            for (t, obs) in observations.iter().enumerate() {
                if let Some(emission_vec) = self.emission_probs.get_mut(obs) {
                    emission_vec[i] += gamma[t][i] / denominator.max(1e-10);
                }
            }
        }
    }
}
