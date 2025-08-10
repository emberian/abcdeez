use rand::prelude::*;
use rand_distr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bayesian Expected Information Gain implementation for adaptive task selection
/// Based on the paper's equation: EIG = E[KL(p(θ|D_t) || p(θ|D_t, Response to q))]
#[derive(Debug, Clone, Serialize)]
pub struct BayesianLearnerModel {
    /// Posterior distributions for node positions
    pub node_positions: HashMap<String, PosteriorDistribution>,
    /// Posterior distributions for operation proficiencies
    pub operation_proficiencies: HashMap<String, PosteriorDistribution>,
    /// Posterior for chunk boundaries
    pub chunk_boundaries: Vec<ChunkBoundaryPosterior>,
    /// Historical responses for updating posteriors
    pub response_history: Vec<ResponseData>,
    /// Topology reference for node mapping
    topology: crate::core::topology::Topology,
    /// Confusability posteriors
    pub confusability: HashMap<(String, String), PosteriorDistribution>,
    /// Memory strength posteriors
    pub memory_strengths: HashMap<String, PosteriorDistribution>,
    /// Seeded RNG for reproducible sampling
    #[serde(skip)]
    rng: rand::rngs::StdRng,
}

impl<'de> serde::Deserialize<'de> for BayesianLearnerModel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BayesianLearnerModelData {
            node_positions: HashMap<String, PosteriorDistribution>,
            operation_proficiencies: HashMap<String, PosteriorDistribution>,
            chunk_boundaries: Vec<ChunkBoundaryPosterior>,
            response_history: Vec<ResponseData>,
            topology: crate::core::topology::Topology,
            confusability: HashMap<(String, String), PosteriorDistribution>,
            memory_strengths: HashMap<String, PosteriorDistribution>,
        }

        let data = BayesianLearnerModelData::deserialize(deserializer)?;

        use rand::SeedableRng;
        Ok(BayesianLearnerModel {
            node_positions: data.node_positions,
            operation_proficiencies: data.operation_proficiencies,
            chunk_boundaries: data.chunk_boundaries,
            response_history: data.response_history,
            topology: data.topology,
            confusability: data.confusability,
            memory_strengths: data.memory_strengths,
            rng: rand::rngs::StdRng::from_entropy(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosteriorDistribution {
    pub mean: f64,
    pub variance: f64,
    pub confidence: f64, // 1 - entropy/max_entropy
}

impl PosteriorDistribution {
    pub fn new(mean: f64, variance: f64) -> Self {
        let confidence = 1.0 / (1.0 + variance);
        PosteriorDistribution {
            mean,
            variance,
            confidence,
        }
    }

    pub fn entropy(&self) -> f64 {
        // Differential entropy of a Gaussian: H = 0.5 * ln(2πeσ²) = 0.5 * ln(2πe) + ln(σ)
        // Simplified: H = 0.5 * ln(2π * e * σ²)
        // Use machine epsilon for numerical stability
        let epsilon_threshold = f64::EPSILON.sqrt();
        if self.variance <= epsilon_threshold {
            return 0.0;
        }
        0.5 * (2.0 * std::f64::consts::PI * std::f64::consts::E * self.variance).ln()
    }

    pub fn update(&mut self, observation: f64, observation_variance: f64) {
        // Guard against NaN/Inf and non-positive variances
        if !observation.is_finite() {
            return;
        }
        let obs_var = if observation_variance.is_finite() && observation_variance > f64::EPSILON {
            observation_variance
        } else {
            1e-6
        };
        let prior_var = if self.variance.is_finite() && self.variance > f64::EPSILON {
            self.variance
        } else {
            1e-3
        };

        // Bayesian update for Gaussian posterior in precision form
        let precision_prior = 1.0 / prior_var;
        let precision_obs = 1.0 / obs_var;

        let precision_post = precision_prior + precision_obs;
        self.variance = (1.0 / precision_post).max(1e-12);

        self.mean = (precision_prior * self.mean + precision_obs * observation) / precision_post;
        if !self.mean.is_finite() {
            self.mean = 0.0;
        }
        // Use information-theoretic confidence based on entropy reduction
        // Confidence = 1 - H(current) / H(prior) where H is entropy
        // For Gaussian: H = 0.5 * ln(2πe * σ²)
        const PRIOR_VARIANCE: f64 = 1.0; // Initial uncertainty
        let current_entropy =
            0.5 * (2.0 * std::f64::consts::PI * std::f64::consts::E * self.variance).ln();
        let prior_entropy =
            0.5 * (2.0 * std::f64::consts::PI * std::f64::consts::E * PRIOR_VARIANCE).ln();

        // Confidence as normalized entropy reduction
        self.confidence = if prior_entropy > 0.0 {
            (1.0 - (current_entropy / prior_entropy)).max(0.0).min(1.0)
        } else {
            0.0
        };
    }

    pub fn kl_divergence(&self, other: &PosteriorDistribution) -> f64 {
        // KL divergence between two Gaussians
        // KL(P||Q) = 0.5 * [log(σ²_Q/σ²_P) + σ²_P/σ²_Q + (μ_P - μ_Q)²/σ²_Q - 1]
        // Using log-domain calculations for numerical stability

        // Ensure minimum variance to prevent division by zero
        const MIN_VARIANCE: f64 = 1e-10;
        let var_p = self.variance.max(MIN_VARIANCE);
        let var_q = other.variance.max(MIN_VARIANCE);

        // Check for extreme variance ratios that could cause overflow
        let variance_ratio = var_p / var_q;
        if variance_ratio > 1e10 || variance_ratio < 1e-10 {
            // Return a large but finite value for extreme cases
            return 100.0;
        }

        // Use log-domain calculation for better stability
        let log_variance_ratio = var_q.ln() - var_p.ln();
        let mean_diff_squared = (self.mean - other.mean).powi(2);

        // KL divergence formula with improved numerical stability
        0.5 * (log_variance_ratio + variance_ratio + mean_diff_squared / var_q - 1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkBoundaryPosterior {
    pub position: usize,
    pub strength: PosteriorDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub task: crate::tasks::Task,
    pub correct: bool,
    pub response_time: f64,
}

/// A sampled model from the posterior for Monte Carlo simulation
struct SampledModel {
    _positions: HashMap<String, f64>,
    proficiencies: HashMap<String, f64>,
}

impl SampledModel {
    /// Predict success probability given sampled parameters
    fn predict_success_probability(&self, task: &crate::tasks::Task) -> f64 {
        // Get operation proficiency for this task
        let op_key = format!("{:?}", task.operation);
        let proficiency = self.proficiencies.get(&op_key).unwrap_or(&0.0);

        // Convert to probability using sigmoid
        1.0 / (1.0 + (-proficiency).exp())
    }
}

impl BayesianLearnerModel {
    pub fn new(topology: &crate::core::topology::Topology) -> Self {
        Self::with_seed(topology, None)
    }

    pub fn with_seed(topology: &crate::core::topology::Topology, seed: Option<u64>) -> Self {
        let mut node_positions = HashMap::new();
        let mut operation_proficiencies = HashMap::new();
        let mut memory_strengths = HashMap::new();

        // Initialize node position posteriors
        for node in &topology.nodes {
            node_positions.insert(
                node.id.clone(),
                PosteriorDistribution::new(node.position, 1.0),
            );

            // Initialize memory strength posteriors
            memory_strengths.insert(node.id.clone(), PosteriorDistribution::new(0.5, 0.25));
        }

        // Initialize operation proficiency posteriors
        let operations = vec![
            "Successor",
            "Predecessor",
            "PairwiseOrder",
            "KJump",
            "Segment",
            "Index",
        ];

        for op in operations {
            operation_proficiencies.insert(
                op.to_string(),
                PosteriorDistribution::new(0.0, 1.0), // Start with neutral prior
            );
        }

        // Initialize chunk boundaries (for linear sequences)
        let chunk_boundaries = match topology.topology_type {
            crate::core::topology::TopologyType::Linear => vec![6, 13, 19]
                .into_iter()
                .map(|pos| ChunkBoundaryPosterior {
                    position: pos,
                    strength: PosteriorDistribution::new(0.5, 0.25),
                })
                .collect(),
            _ => vec![],
        };

        use rand::SeedableRng;
        let rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };

        BayesianLearnerModel {
            node_positions,
            operation_proficiencies,
            chunk_boundaries,
            response_history: vec![],
            topology: topology.clone(),
            confusability: HashMap::new(),
            memory_strengths,
            rng,
        }
    }

    /// Calculate Expected Information Gain for a given task using Monte Carlo simulation
    /// EIG = E[KL(p(θ|D_t) || p(θ|D_t, Response to q))]
    pub fn calculate_eig(&mut self, task: &crate::tasks::Task) -> f64 {
        // Use adaptive sampling for better convergence
        let (eig, _samples_used) = self.adaptive_monte_carlo_eig(task);

        // Bound EIG by the entropy of the specific parameters being queried
        // Information gain for a single task cannot exceed the entropy of the parameters it informs about
        let task_entropy = self.calculate_task_specific_entropy(task);
        eig.min(task_entropy)
    }

    /// Monte Carlo simulation for Expected Information Gain
    /// Samples from the posterior predictive distribution
    pub fn monte_carlo_eig(&mut self, task: &crate::tasks::Task, n_samples: usize) -> f64 {
        use rand::prelude::*;

        // Use the seeded RNG instead of thread_rng
        let mut total_eig = 0.0;

        for _ in 0..n_samples {
            // Sample from current posterior beliefs
            let sampled_model = self.sample_from_posterior();

            // Simulate response given sampled parameters
            let response_prob = sampled_model.predict_success_probability(task);
            let simulated_correct = self.rng.gen::<f64>() < response_prob;

            // Calculate KL divergence for this simulated outcome
            let kl = if simulated_correct {
                self.calculate_kl_if_correct_monte_carlo(task, &sampled_model)
            } else {
                self.calculate_kl_if_incorrect_monte_carlo(task, &sampled_model)
            };

            total_eig += kl;
        }

        total_eig / n_samples as f64
    }

    /// Adaptive Monte Carlo EIG with convergence checking
    pub fn adaptive_monte_carlo_eig(&mut self, task: &crate::tasks::Task) -> (f64, usize) {
        const MIN_SAMPLES: usize = 100;
        const MAX_SAMPLES: usize = 10000;
        const RELATIVE_ERROR_THRESHOLD: f64 = 0.01; // 1% relative error

        use rand::prelude::*;
        // Use the seeded RNG instead of thread_rng

        let mut running_mean = 0.0;
        let mut running_var = 0.0;

        for i in 0..MAX_SAMPLES {
            // Sample from current posterior beliefs
            let sampled_model = self.sample_from_posterior();

            // Simulate response given sampled parameters
            let response_prob = sampled_model.predict_success_probability(task);
            let simulated_correct = self.rng.gen::<f64>() < response_prob;

            // Calculate KL divergence for this simulated outcome
            let kl = if simulated_correct {
                self.calculate_kl_if_correct_monte_carlo(task, &sampled_model)
            } else {
                self.calculate_kl_if_incorrect_monte_carlo(task, &sampled_model)
            };

            // Update running statistics (Welford's method)
            let delta = kl - running_mean;
            running_mean += delta / (i + 1) as f64;
            if i > 0 {
                running_var += delta * (kl - running_mean);
            }

            // Check convergence after minimum samples
            if i >= MIN_SAMPLES {
                let std_error = (running_var / (i * (i + 1)) as f64).sqrt();
                let relative_error = std_error / running_mean.abs().max(1e-10);

                if relative_error < RELATIVE_ERROR_THRESHOLD {
                    return (running_mean, i + 1);
                }
            }
        }

        eprintln!(
            "Warning: Monte Carlo EIG did not converge after {} samples",
            MAX_SAMPLES
        );
        (running_mean, MAX_SAMPLES)
    }

    /// Sample a model from the current posterior distributions
    fn sample_from_posterior(&mut self) -> SampledModel {
        use rand_distr::Normal;

        let mut sampled_positions = HashMap::new();
        for (key, posterior) in &self.node_positions {
            let dist = Normal::new(posterior.mean, posterior.variance.sqrt())
                .unwrap_or(Normal::new(0.0, 1.0).unwrap());
            sampled_positions.insert(key.clone(), dist.sample(&mut self.rng));
        }

        let mut sampled_proficiencies = HashMap::new();
        for (key, posterior) in &self.operation_proficiencies {
            let dist = Normal::new(posterior.mean, posterior.variance.sqrt())
                .unwrap_or(Normal::new(0.0, 1.0).unwrap());
            sampled_proficiencies.insert(key.clone(), dist.sample(&mut self.rng));
        }

        SampledModel {
            _positions: sampled_positions,
            proficiencies: sampled_proficiencies,
        }
    }

    /// Calculate KL divergence for correct response in Monte Carlo
    fn calculate_kl_if_correct_monte_carlo(
        &self,
        task: &crate::tasks::Task,
        _sampled: &SampledModel,
    ) -> f64 {
        // Create updated posterior given correct response
        let mut updated_model = self.clone();
        updated_model.update_with_response(ResponseData {
            task: task.clone(),
            correct: true,
            response_time: 1000.0, // Default for simulation
        });

        // Calculate KL divergence between current and updated posteriors
        self.kl_divergence_to(&updated_model)
    }

    /// Calculate KL divergence for incorrect response in Monte Carlo
    fn calculate_kl_if_incorrect_monte_carlo(
        &self,
        task: &crate::tasks::Task,
        _sampled: &SampledModel,
    ) -> f64 {
        // Create updated posterior given incorrect response
        let mut updated_model = self.clone();
        updated_model.update_with_response(ResponseData {
            task: task.clone(),
            correct: false,
            response_time: 2000.0, // Default for simulation
        });

        // Calculate KL divergence between current and updated posteriors
        self.kl_divergence_to(&updated_model)
    }

    /// Calculate total KL divergence to another model
    fn kl_divergence_to(&self, other: &BayesianLearnerModel) -> f64 {
        let mut total_kl = 0.0;

        // KL for node positions
        for (key, pos) in &self.node_positions {
            if let Some(other_pos) = other.node_positions.get(key) {
                total_kl += pos.kl_divergence(other_pos);
            }
        }

        // KL for operation proficiencies
        for (key, prof) in &self.operation_proficiencies {
            if let Some(other_prof) = other.operation_proficiencies.get(key) {
                total_kl += prof.kl_divergence(other_prof);
            }
        }

        total_kl
    }

    fn predict_accuracy(&self, task: &crate::tasks::Task) -> f64 {
        // Get operation proficiency
        let op_key = format!("{:?}", task.operation);
        let proficiency = self
            .operation_proficiencies
            .get(&op_key)
            .map(|p| p.mean)
            .unwrap_or(0.0);

        // Sigmoid function for probability
        let z = proficiency - task.difficulty;
        1.0 / (1.0 + (-z).exp())
    }

    fn calculate_kl_if_correct(&self, task: &crate::tasks::Task) -> f64 {
        let mut total_kl = 0.0;

        // Calculate KL for affected parameters
        match &task.task_type {
            crate::tasks::TaskType::PairwiseOrder { a, b } => {
                // This task would reduce uncertainty about relative positions
                if let (Some(pos_a), Some(pos_b)) =
                    (self.get_node_position(a), self.get_node_position(b))
                {
                    // Simulate posterior after correct response
                    let mut post_a = pos_a.clone();
                    let mut post_b = pos_b.clone();

                    // Correct response confirms order, reduce variance
                    post_a.variance *= 0.8;
                    post_b.variance *= 0.8;

                    total_kl += pos_a.kl_divergence(&post_a);
                    total_kl += pos_b.kl_divergence(&post_b);
                }
            }
            crate::tasks::TaskType::Successor { item }
            | crate::tasks::TaskType::Predecessor { item } => {
                if let Some(pos) = self.get_node_position(item) {
                    let mut post = pos.clone();
                    post.variance *= 0.7; // Greater reduction for local adjacency
                    total_kl += pos.kl_divergence(&post);
                }
            }
            crate::tasks::TaskType::Segment { start, count, .. } => {
                // Segment tasks affect multiple nodes and chunk boundaries
                for i in 0..*count {
                    let boundary_kl = self.calculate_boundary_kl(start, i);
                    total_kl += boundary_kl;
                }
            }
            _ => {}
        }

        // Add operation proficiency KL
        let op_key = format!("{:?}", task.operation);
        if let Some(prof) = self.operation_proficiencies.get(&op_key) {
            let mut post_prof = prof.clone();
            post_prof.update(1.0, 0.1); // Update with success
            total_kl += prof.kl_divergence(&post_prof);
        }

        total_kl
    }

    fn calculate_kl_if_incorrect(&self, task: &crate::tasks::Task) -> f64 {
        let mut total_kl = 0.0;

        // Incorrect responses often increase uncertainty
        match &task.task_type {
            crate::tasks::TaskType::PairwiseOrder { a, b } => {
                if let (Some(pos_a), Some(pos_b)) =
                    (self.get_node_position(a), self.get_node_position(b))
                {
                    let mut post_a = pos_a.clone();
                    let mut post_b = pos_b.clone();

                    // Incorrect response increases uncertainty
                    post_a.variance *= 1.2;
                    post_b.variance *= 1.2;

                    total_kl += pos_a.kl_divergence(&post_a);
                    total_kl += pos_b.kl_divergence(&post_b);
                }
            }
            _ => {}
        }

        // Update operation proficiency for failure
        let op_key = format!("{:?}", task.operation);
        if let Some(prof) = self.operation_proficiencies.get(&op_key) {
            let mut post_prof = prof.clone();
            post_prof.update(0.0, 0.1); // Update with failure
            total_kl += prof.kl_divergence(&post_prof);
        }

        total_kl
    }

    fn get_node_position(&self, label: &str) -> Option<&PosteriorDistribution> {
        // Properly map label to node_id using topology
        if let Some(node) = self.topology.get_node_by_label(label) {
            self.node_positions.get(&node.id)
        } else {
            None
        }
    }

    fn calculate_boundary_kl(&self, start: &str, offset: usize) -> f64 {
        // Check if this crosses a chunk boundary
        if let Some(node) = self.topology.get_node_by_label(start) {
            let start_idx = self.topology.node_map.get(&node.id).copied().unwrap_or(0);
            let position = start_idx + offset;

            for boundary in &self.chunk_boundaries {
                if position == boundary.position {
                    // Crossing boundary would update our belief about its strength
                    let mut post = boundary.strength.clone();
                    post.variance *= 0.9;
                    return boundary.strength.kl_divergence(&post);
                }
            }
        }
        0.0
    }

    pub fn update_with_response(&mut self, response: ResponseData) {
        self.response_history.push(response.clone());

        // Calculate observation variance before mutable borrows
        let obs_variance = self.calculate_observation_variance(&response);

        // Update relevant posteriors based on response
        let op_key = format!("{:?}", response.task.operation);
        if let Some(prof) = self.operation_proficiencies.get_mut(&op_key) {
            let observation = if response.correct { 1.0 } else { 0.0 };
            prof.update(observation, obs_variance);
        }

        // Update node position posteriors and other parameters
        match &response.task.task_type {
            crate::tasks::TaskType::PairwiseOrder { a, b } => {
                // Update beliefs about relative positions
                self.update_pairwise_positions(a, b, response.correct);
            }
            crate::tasks::TaskType::Successor { item }
            | crate::tasks::TaskType::Predecessor { item } => {
                self.update_adjacency_beliefs(item, &response.task.task_type, response.correct);
            }
            crate::tasks::TaskType::Segment { start, count, .. } => {
                self.update_segment_beliefs(
                    start,
                    *count,
                    response.correct,
                    response.response_time,
                );
            }
            crate::tasks::TaskType::KJump { start, k } => {
                self.update_kjump_beliefs(start, *k, response.correct);
            }
            _ => {}
        }
    }

    fn update_pairwise_positions(&mut self, a: &str, b: &str, correct: bool) {
        // Update position posteriors based on comparison result
        if let (Some(node_a), Some(node_b)) = (
            self.topology.get_node_by_label(a),
            self.topology.get_node_by_label(b),
        ) {
            // Avoid double mutable borrow by updating positions separately
            let node_a_id = node_a.id.clone();
            let node_b_id = node_b.id.clone();

            // Update position for node A
            if let Some(pos_a) = self.node_positions.get_mut(&node_a_id) {
                let obs_variance = 0.2;
                if correct {
                    pos_a.update(pos_a.mean, obs_variance * 0.8);
                } else {
                    pos_a.variance *= 1.1;
                }
            }

            // Update position for node B
            if let Some(pos_b) = self.node_positions.get_mut(&node_b_id) {
                let obs_variance = 0.2;
                if correct {
                    pos_b.update(pos_b.mean, obs_variance * 0.8);
                } else {
                    pos_b.variance *= 1.1;
                }
            }

            // Update confusability if incorrect
            if !correct {
                let key = if a < b {
                    (a.to_string(), b.to_string())
                } else {
                    (b.to_string(), a.to_string())
                };
                self.confusability
                    .entry(key)
                    .or_insert(PosteriorDistribution::new(0.0, 1.0))
                    .update(1.0, 0.2);
            }

            // Update memory strengths
            if let Some(mem_a) = self.memory_strengths.get_mut(&node_a_id) {
                mem_a.update(if correct { 0.8 } else { 0.3 }, 0.1);
            }
            if let Some(mem_b) = self.memory_strengths.get_mut(&node_b_id) {
                mem_b.update(if correct { 0.8 } else { 0.3 }, 0.1);
            }
        }
    }

    /// Calculate adaptive observation variance based on response characteristics
    fn calculate_observation_variance(&self, response: &ResponseData) -> f64 {
        // Base variance
        let mut variance = 0.1;

        // Adjust based on response time (faster responses = more confidence)
        let rt_factor = (response.response_time / 1000.0).min(3.0).max(0.5);
        variance *= rt_factor / 1.5;

        // Adjust based on task difficulty
        let difficulty_factor = 0.5 + response.task.difficulty;
        variance *= difficulty_factor;

        // Adjust based on response history (more consistent = lower variance)
        if self.response_history.len() > 10 {
            let recent_accuracy = self
                .response_history
                .iter()
                .rev()
                .take(10)
                .filter(|r| r.correct)
                .count() as f64
                / 10.0;
            variance *= 2.0 - recent_accuracy; // Higher accuracy = lower variance
        }

        variance.max(0.01).min(1.0)
    }

    /// Update beliefs for adjacency tasks (successor/predecessor)
    fn update_adjacency_beliefs(
        &mut self,
        item: &str,
        task_type: &crate::tasks::TaskType,
        correct: bool,
    ) {
        if let Some(node) = self.topology.get_node_by_label(item) {
            if let Some(pos) = self.node_positions.get_mut(&node.id) {
                let obs_variance = if correct { 0.05 } else { 0.15 };
                pos.update(pos.mean, obs_variance);
            }

            // Update memory strength
            if let Some(mem) = self.memory_strengths.get_mut(&node.id) {
                mem.update(if correct { 0.9 } else { 0.4 }, 0.1);
            }

            // Update adjacency-specific beliefs
            match task_type {
                crate::tasks::TaskType::Successor { .. } => {
                    if let Some(next_id) = self.topology.get_successor(&node.id) {
                        if let Some(next_mem) = self.memory_strengths.get_mut(&next_id) {
                            next_mem.update(if correct { 0.7 } else { 0.3 }, 0.15);
                        }
                    }
                }
                crate::tasks::TaskType::Predecessor { .. } => {
                    if let Some(prev_id) = self.topology.get_predecessor(&node.id) {
                        if let Some(prev_mem) = self.memory_strengths.get_mut(&prev_id) {
                            prev_mem.update(if correct { 0.7 } else { 0.3 }, 0.15);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Update beliefs for segment tasks
    fn update_segment_beliefs(
        &mut self,
        start: &str,
        count: usize,
        correct: bool,
        response_time: f64,
    ) {
        if let Some(start_node) = self.topology.get_node_by_label(start) {
            let start_idx = self
                .topology
                .node_map
                .get(&start_node.id)
                .copied()
                .unwrap_or(0);

            // Update memory for all nodes in segment
            for i in 0..count {
                let idx = start_idx + i;
                if idx < self.topology.nodes.len() {
                    let node_id = &self.topology.nodes[idx].id;
                    if let Some(mem) = self.memory_strengths.get_mut(node_id) {
                        // Decay factor for distance from start
                        let distance_factor = 1.0 - (i as f64 / count as f64) * 0.3;
                        mem.update(if correct { 0.8 * distance_factor } else { 0.3 }, 0.12);
                    }
                }
            }

            // Update chunk boundaries if response time suggests difficulty
            if response_time > 2000.0 {
                for i in 1..count {
                    let boundary_pos = start_idx + i;
                    for boundary in &mut self.chunk_boundaries {
                        if boundary.position == boundary_pos {
                            // Slow response suggests chunk boundary
                            boundary.strength.update(0.7, 0.15);
                        }
                    }
                }
            }
        }
    }

    /// Update beliefs for k-jump tasks
    fn update_kjump_beliefs(&mut self, start: &str, k: i32, correct: bool) {
        if let Some(start_node) = self.topology.get_node_by_label(start) {
            let start_idx = self
                .topology
                .node_map
                .get(&start_node.id)
                .copied()
                .unwrap_or(0) as i32;
            let target_idx = (start_idx + k).max(0) as usize;

            // Update memory for start and target
            if let Some(mem) = self.memory_strengths.get_mut(&start_node.id) {
                mem.update(if correct { 0.85 } else { 0.4 }, 0.1);
            }

            if target_idx < self.topology.nodes.len() {
                let target_id = &self.topology.nodes[target_idx].id;
                if let Some(mem) = self.memory_strengths.get_mut(target_id) {
                    mem.update(if correct { 0.75 } else { 0.35 }, 0.12);
                }

                // Update position beliefs for large jumps
                if k.abs() > 2 {
                    if let Some(pos) = self.node_positions.get_mut(target_id) {
                        let obs_variance = if correct { 0.08 } else { 0.2 };
                        pos.update(pos.mean, obs_variance);
                    }
                }
            }
        }
    }

    /// Get tasks ranked by Expected Information Gain
    pub fn rank_tasks_by_eig(
        &mut self,
        tasks: Vec<crate::tasks::Task>,
    ) -> Vec<(crate::tasks::Task, f64)> {
        let mut ranked: Vec<(crate::tasks::Task, f64)> = tasks
            .into_iter()
            .map(|task| {
                let eig = self.calculate_eig(&task);
                (task, eig)
            })
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    /// Get current entropy of the model (uncertainty)
    pub fn total_entropy(&self) -> f64 {
        let mut entropy = 0.0;

        for pos in self.node_positions.values() {
            entropy += pos.entropy();
        }

        for prof in self.operation_proficiencies.values() {
            entropy += prof.entropy();
        }

        for boundary in &self.chunk_boundaries {
            entropy += boundary.strength.entropy();
        }

        // Ensure entropy is non-negative for numerical robustness in tests
        entropy.max(0.0)
    }

    /// Calculate entropy for parameters relevant to a specific task
    fn calculate_task_specific_entropy(&self, task: &crate::tasks::Task) -> f64 {
        let mut entropy = 0.0;

        // Add entropy of operation proficiency for this task
        let op_key = format!("{:?}", task.operation);
        if let Some(prof) = self.operation_proficiencies.get(&op_key) {
            entropy += prof.entropy();
        }

        // Add entropy of relevant node positions based on task type
        match &task.task_type {
            crate::tasks::TaskType::PairwiseOrder { a, b } => {
                if let Some(pos_a) = self.get_node_position(a) {
                    entropy += pos_a.entropy();
                }
                if let Some(pos_b) = self.get_node_position(b) {
                    entropy += pos_b.entropy();
                }
            }
            crate::tasks::TaskType::Successor { item }
            | crate::tasks::TaskType::Predecessor { item } => {
                if let Some(pos) = self.get_node_position(item) {
                    entropy += pos.entropy();
                    // Also include adjacent node uncertainty
                    if let Some(node) = self.topology.get_node_by_label(item) {
                        if let Some(adj_id) = match &task.task_type {
                            crate::tasks::TaskType::Successor { .. } => {
                                self.topology.get_successor(&node.id)
                            }
                            crate::tasks::TaskType::Predecessor { .. } => {
                                self.topology.get_predecessor(&node.id)
                            }
                            _ => None,
                        } {
                            if let Some(adj_node) = self.topology.get_node_by_id(&adj_id) {
                                if let Some(adj_pos) = self.get_node_position(&adj_node.label) {
                                    entropy += adj_pos.entropy();
                                }
                            }
                        }
                    }
                }
            }
            crate::tasks::TaskType::KJump { start, k } => {
                if let Some(pos) = self.get_node_position(start) {
                    entropy += pos.entropy();
                }
                // Include destination uncertainty
                if let Some(dest) = self.topology.get_k_jump(start, *k as i32) {
                    if let Some(dest_pos) = self.get_node_position(&dest) {
                        entropy += dest_pos.entropy();
                    }
                }
            }
            _ => {
                // For other task types, use a conservative estimate
                // based on operation proficiency entropy only
                if let Some(prof) = self.operation_proficiencies.get(&op_key) {
                    entropy = prof.entropy();
                } else {
                    entropy = 1.0;
                }
            }
        }

        entropy.max(0.0)
    }
}

/// Monte Carlo estimation of Expected Information Gain
pub struct MonteCarloEIG {
    samples: usize,
    rng: rand::rngs::StdRng,
}

impl MonteCarloEIG {
    pub fn new(samples: usize) -> Self {
        use rand::SeedableRng;
        MonteCarloEIG {
            samples,
            rng: rand::rngs::StdRng::from_entropy(),
        }
    }

    pub fn estimate_eig(&mut self, model: &BayesianLearnerModel, task: &crate::tasks::Task) -> f64 {
        let mut total_gain = 0.0;

        for _ in 0..self.samples {
            // Sample from current posterior
            let sampled_params = self.sample_from_posterior(model);

            // Simulate response given sampled parameters
            let p_correct = self.simulate_response(&sampled_params, task);

            // Calculate information gain for this sample
            let gain = if rand::random::<f64>() < p_correct {
                model.calculate_kl_if_correct(task)
            } else {
                model.calculate_kl_if_incorrect(task)
            };

            total_gain += gain;
        }

        total_gain / self.samples as f64
    }

    fn sample_from_posterior(&mut self, model: &BayesianLearnerModel) -> SampledParameters {
        let mut sampled = SampledParameters::new();

        // Sample from each posterior distribution
        for (key, dist) in &model.node_positions {
            use rand_distr::{Distribution, Normal as RandNormal};
            let normal = RandNormal::new(dist.mean, dist.variance.sqrt()).unwrap();
            sampled
                .node_positions
                .insert(key.clone(), normal.sample(&mut self.rng));
        }

        for (key, dist) in &model.operation_proficiencies {
            use rand_distr::{Distribution, Normal as RandNormal};
            let normal = RandNormal::new(dist.mean, dist.variance.sqrt()).unwrap();
            sampled
                .operation_proficiencies
                .insert(key.clone(), normal.sample(&mut self.rng));
        }

        sampled
    }

    fn simulate_response(&self, params: &SampledParameters, task: &crate::tasks::Task) -> f64 {
        // Simulate response probability given sampled parameters
        let op_key = format!("{:?}", task.operation);
        let proficiency = params.operation_proficiencies.get(&op_key).unwrap_or(&0.0);

        let z = proficiency - task.difficulty;
        1.0 / (1.0 + (-z).exp())
    }
}

struct SampledParameters {
    node_positions: HashMap<String, f64>,
    operation_proficiencies: HashMap<String, f64>,
}

impl SampledParameters {
    fn new() -> Self {
        SampledParameters {
            node_positions: HashMap::new(),
            operation_proficiencies: HashMap::new(),
        }
    }
}

/// Model Comparison Metrics for Bayesian Model Selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparisonMetrics {
    pub log_likelihood: f64,
    pub n_parameters: usize,
    pub n_observations: usize,
}

impl ModelComparisonMetrics {
    pub fn new(log_likelihood: f64, n_parameters: usize, n_observations: usize) -> Self {
        Self {
            log_likelihood,
            n_parameters,
            n_observations,
        }
    }

    /// Akaike Information Criterion: AIC = 2k - 2ln(L)
    pub fn aic(&self) -> f64 {
        2.0 * self.n_parameters as f64 - 2.0 * self.log_likelihood
    }

    /// Corrected AIC for small samples: AICc = AIC + 2k(k+1)/(n-k-1)
    pub fn aicc(&self) -> f64 {
        let k = self.n_parameters as f64;
        let n = self.n_observations as f64;
        if n - k - 1.0 <= 0.0 {
            return f64::INFINITY;
        }
        self.aic() + (2.0 * k * (k + 1.0)) / (n - k - 1.0)
    }

    /// Bayesian Information Criterion: BIC = k*ln(n) - 2ln(L)
    pub fn bic(&self) -> f64 {
        let k = self.n_parameters as f64;
        let n = self.n_observations as f64;
        k * n.ln() - 2.0 * self.log_likelihood
    }

    /// Compare two models using AIC difference
    pub fn aic_weight(&self, other: &ModelComparisonMetrics) -> f64 {
        let a1 = self.aic();
        let a2 = other.aic();
        let m = a1.min(a2);
        let w1 = ((-0.5) * (a1 - m)).exp();
        let w2 = ((-0.5) * (a2 - m)).exp();
        let denom = w1 + w2;
        if denom.is_finite() && denom > 0.0 {
            w1 / denom
        } else {
            0.5
        }
    }

    /// Evidence ratio for model comparison
    pub fn evidence_ratio(&self, other: &ModelComparisonMetrics) -> f64 {
        let delta_aic = other.aic() - self.aic();
        (0.5 * delta_aic).exp()
    }
}

/// Deviance Information Criterion (DIC) for hierarchical Bayesian models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIC {
    pub mean_deviance: f64,
    pub deviance_at_mean: f64,
}

impl DIC {
    pub fn new(mean_deviance: f64, deviance_at_mean: f64) -> Self {
        Self {
            mean_deviance,
            deviance_at_mean,
        }
    }

    /// Effective number of parameters: pD = D̄ - D(θ̄)
    pub fn effective_parameters(&self) -> f64 {
        self.mean_deviance - self.deviance_at_mean
    }

    /// DIC = D̄ + pD = 2D̄ - D(θ̄)
    pub fn dic(&self) -> f64 {
        2.0 * self.mean_deviance - self.deviance_at_mean
    }
}

/// Watanabe-Akaike Information Criterion (WAIC) - fully Bayesian
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WAIC {
    pub lppd: f64,   // Log pointwise predictive density
    pub p_waic: f64, // Effective number of parameters
}

impl WAIC {
    pub fn new(lppd: f64, p_waic: f64) -> Self {
        Self { lppd, p_waic }
    }

    /// WAIC = -2(lppd - p_waic)
    pub fn waic(&self) -> f64 {
        -2.0 * (self.lppd - self.p_waic)
    }

    /// Standard error of WAIC
    pub fn se(&self, pointwise_variances: &[f64]) -> f64 {
        let var_sum: f64 = pointwise_variances.iter().sum();
        var_sum.sqrt()
    }
}

impl BayesianLearnerModel {
    /// Calculate log-likelihood for current model given data
    pub fn log_likelihood(&self) -> f64 {
        self.response_history
            .iter()
            .map(|response| {
                let predicted_prob = self.predict_accuracy(&response.task);
                if response.correct {
                    (predicted_prob as f64).ln()
                } else {
                    (1.0 - predicted_prob as f64).ln()
                }
            })
            .sum()
    }

    /// Calculate model comparison metrics
    pub fn model_comparison_metrics(&self) -> ModelComparisonMetrics {
        let n_params = self.count_parameters();
        let n_obs = self.response_history.len();
        let log_lik = self.log_likelihood();

        ModelComparisonMetrics::new(log_lik, n_params, n_obs)
    }

    /// Count effective number of parameters
    fn count_parameters(&self) -> usize {
        let node_params = self.node_positions.len();
        let op_params = self.operation_proficiencies.len();
        let boundary_params = self.chunk_boundaries.len() * 2; // position + strength
        let confuse_params = self.confusability.len();
        let memory_params = self.memory_strengths.len();

        node_params + op_params + boundary_params + confuse_params + memory_params
    }

    /// Calculate DIC using posterior samples
    pub fn calculate_dic(&mut self, n_samples: usize) -> DIC {
        // Use the seeded RNG instead of thread_rng
        let mut deviances = Vec::new();

        // Sample from posterior and calculate deviances
        for _ in 0..n_samples {
            let sampled_model = self.sample_from_posterior();
            let deviance = -2.0 * self.log_likelihood_with_params(&sampled_model);
            deviances.push(deviance);
        }

        // Mean deviance
        let mean_deviance = deviances.iter().sum::<f64>() / n_samples as f64;

        // Deviance at posterior mean
        let deviance_at_mean = -2.0 * self.log_likelihood();

        DIC::new(mean_deviance, deviance_at_mean)
    }

    /// Calculate WAIC using posterior samples
    pub fn calculate_waic(&mut self, n_samples: usize) -> WAIC {
        // Use the seeded RNG instead of thread_rng
        let mut log_likelihoods = vec![Vec::new(); self.response_history.len()];

        // Sample from posterior
        for _ in 0..n_samples {
            let sampled_model = self.sample_from_posterior();

            // Calculate log-likelihood for each observation
            for (i, response) in self.response_history.iter().enumerate() {
                let prob = self.predict_with_params(&sampled_model, &response.task);
                let log_lik = if response.correct {
                    prob.ln()
                } else {
                    (1.0 - prob).ln()
                };
                log_likelihoods[i].push(log_lik);
            }
        }

        // Calculate lppd (log pointwise predictive density)
        let lppd: f64 = log_likelihoods
            .iter()
            .map(|liks| {
                let max_lik = liks.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let sum_exp: f64 = liks.iter().map(|l| (l - max_lik).exp()).sum();
                max_lik + (sum_exp / n_samples as f64).ln()
            })
            .sum();

        // Calculate p_waic (effective number of parameters)
        let p_waic: f64 = log_likelihoods
            .iter()
            .map(|liks| {
                let mean = liks.iter().sum::<f64>() / liks.len() as f64;
                let variance =
                    liks.iter().map(|l| (l - mean).powi(2)).sum::<f64>() / liks.len() as f64;
                variance
            })
            .sum();

        WAIC::new(lppd, p_waic)
    }

    /// Helper: calculate log-likelihood with specific parameters
    fn log_likelihood_with_params(&self, params: &SampledModel) -> f64 {
        self.response_history
            .iter()
            .map(|response| {
                let prob = self.predict_with_params(params, &response.task);
                if response.correct {
                    prob.ln()
                } else {
                    (1.0 - prob).ln()
                }
            })
            .sum()
    }

    /// Helper: predict with specific sampled parameters
    fn predict_with_params(&self, params: &SampledModel, task: &crate::tasks::Task) -> f64 {
        params.predict_success_probability(task)
    }

    /// Posterior Predictive Check: Generate replicated data and compare with observed
    pub fn posterior_predictive_check(
        &mut self,
        n_replications: usize,
    ) -> PosteriorPredictiveCheck {
        // Use the seeded RNG instead of thread_rng
        let mut replicated_data = Vec::new();

        for _ in 0..n_replications {
            let sampled_model = self.sample_from_posterior();
            let mut replicated_responses = Vec::new();

            // Generate replicated responses for each observed task
            let response_history_copy = self.response_history.clone();
            for response in &response_history_copy {
                let prob = self.predict_with_params(&sampled_model, &response.task);
                let replicated_correct = self.rng.gen::<f64>() < prob;

                // Simulate response time using Ex-Gaussian
                let rt_mean = 1000.0 + response.task.difficulty * 500.0;
                let rt = self.sample_response_time(rt_mean);

                replicated_responses.push(ResponseData {
                    task: response.task.clone(),
                    correct: replicated_correct,
                    response_time: rt,
                });
            }

            replicated_data.push(replicated_responses);
        }

        // Calculate test statistics
        let observed_stats = self.calculate_test_statistics(&self.response_history);
        let replicated_stats: Vec<TestStatistics> = replicated_data
            .iter()
            .map(|data| self.calculate_test_statistics(data))
            .collect();

        PosteriorPredictiveCheck {
            observed_statistics: observed_stats,
            replicated_statistics: replicated_stats,
            n_replications,
        }
    }

    /// Sample response time from Ex-Gaussian distribution
    fn sample_response_time(&mut self, mean: f64) -> f64 {
        use rand_distr::{Exp, Normal};

        // Ex-Gaussian parameters
        let mu = mean * 0.8;
        let sigma = mean * 0.15;
        let tau = mean * 0.2;

        // Sample from normal and exponential
        let normal = Normal::new(mu, sigma).unwrap_or(Normal::new(1000.0, 150.0).unwrap());
        let exp = Exp::new(1.0 / tau).unwrap_or(Exp::new(0.001).unwrap());

        let normal_sample: f64 = normal.sample(&mut self.rng);
        let exp_sample: f64 = exp.sample(&mut self.rng);

        (normal_sample + exp_sample).max(100.0) // Minimum 100ms RT
    }

    /// Calculate test statistics for a set of responses
    fn calculate_test_statistics(&self, responses: &[ResponseData]) -> TestStatistics {
        let n = responses.len() as f64;
        let n_correct = responses.iter().filter(|r| r.correct).count() as f64;
        let accuracy = n_correct / n;

        let mean_rt = responses.iter().map(|r| r.response_time).sum::<f64>() / n;

        let rt_variance = responses
            .iter()
            .map(|r| (r.response_time - mean_rt).powi(2))
            .sum::<f64>()
            / n;

        // Calculate autocorrelation of correctness
        let autocorr = if responses.len() > 1 {
            let mut sum = 0.0;
            for i in 1..responses.len() {
                let prev = if responses[i - 1].correct { 1.0 } else { 0.0 };
                let curr = if responses[i].correct { 1.0 } else { 0.0 };
                sum += (prev - accuracy) * (curr - accuracy);
            }
            sum / (n - 1.0) / (accuracy * (1.0 - accuracy)).max(0.01)
        } else {
            0.0
        };

        TestStatistics {
            accuracy,
            mean_rt,
            rt_std: rt_variance.sqrt(),
            autocorrelation: autocorr,
        }
    }
}

/// Test statistics for posterior predictive checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStatistics {
    pub accuracy: f64,
    pub mean_rt: f64,
    pub rt_std: f64,
    pub autocorrelation: f64,
}

/// Result of posterior predictive check
#[derive(Debug, Clone)]
pub struct PosteriorPredictiveCheck {
    pub observed_statistics: TestStatistics,
    pub replicated_statistics: Vec<TestStatistics>,
    pub n_replications: usize,
}

impl PosteriorPredictiveCheck {
    /// Calculate p-values for each test statistic
    pub fn calculate_p_values(&self) -> PosteriorPredictivePValues {
        let p_accuracy = self.calculate_p_value(
            self.observed_statistics.accuracy,
            self.replicated_statistics.iter().map(|s| s.accuracy),
        );

        let p_mean_rt = self.calculate_p_value(
            self.observed_statistics.mean_rt,
            self.replicated_statistics.iter().map(|s| s.mean_rt),
        );

        let p_rt_std = self.calculate_p_value(
            self.observed_statistics.rt_std,
            self.replicated_statistics.iter().map(|s| s.rt_std),
        );

        let p_autocorr = self.calculate_p_value(
            self.observed_statistics.autocorrelation,
            self.replicated_statistics.iter().map(|s| s.autocorrelation),
        );

        PosteriorPredictivePValues {
            accuracy: p_accuracy,
            mean_rt: p_mean_rt,
            rt_std: p_rt_std,
            autocorrelation: p_autocorr,
        }
    }

    /// Calculate two-tailed p-value
    fn calculate_p_value<I>(&self, observed: f64, replicated: I) -> f64
    where
        I: Iterator<Item = f64>,
    {
        let replicated: Vec<f64> = replicated.collect();
        let n = replicated.len().max(1) as f64;
        let mean = replicated.iter().copied().sum::<f64>() / n;
        let more_extreme = replicated
            .iter()
            .filter(|&&r| (r - mean).abs() >= (observed - mean).abs())
            .count() as f64;
        more_extreme / n
    }

    /// Check if model adequately fits the data
    pub fn check_model_adequacy(&self, alpha: f64) -> ModelAdequacy {
        let p_values = self.calculate_p_values();

        ModelAdequacy {
            accuracy_adequate: p_values.accuracy > alpha,
            mean_rt_adequate: p_values.mean_rt > alpha,
            rt_std_adequate: p_values.rt_std > alpha,
            autocorr_adequate: p_values.autocorrelation > alpha,
            overall_adequate: p_values.accuracy > alpha
                && p_values.mean_rt > alpha
                && p_values.rt_std > alpha
                && p_values.autocorrelation > alpha,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosteriorPredictivePValues {
    pub accuracy: f64,
    pub mean_rt: f64,
    pub rt_std: f64,
    pub autocorrelation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAdequacy {
    pub accuracy_adequate: bool,
    pub mean_rt_adequate: bool,
    pub rt_std_adequate: bool,
    pub autocorr_adequate: bool,
    pub overall_adequate: bool,
}
