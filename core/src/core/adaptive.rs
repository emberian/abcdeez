use crate::core::bayesian::{BayesianLearnerModel, ResponseData};
use crate::core::learner::LearnerModel;
use crate::tasks::types::{Task, TaskGenerator, TaskType};
use crate::core::topology::Topology;
use rand::Rng;
use tracing::{debug, info, instrument, span, warn, Level};

#[derive(Debug)]
pub struct AdaptiveScheduler {
    learner_model: LearnerModel,
    bayesian_model: BayesianLearnerModel,
    topology: Topology,
    task_generator: TaskGenerator,
    epsilon: f64,
    use_eig: bool,
    trials_completed: usize,
    exploration_decay: f64,
    rng: rand::rngs::StdRng,
}

impl AdaptiveScheduler {
    #[instrument(level = "debug", fields(topology_size = topology.nodes.len()))]
    pub fn new(learner_model: LearnerModel, topology: Topology) -> Self {
        info!(
            topology_size = topology.nodes.len(),
            "Creating new adaptive scheduler"
        );

        let task_generator = TaskGenerator::new(topology.clone());
        let bayesian_model = BayesianLearnerModel::new(&topology);

        use rand::SeedableRng;
        let scheduler = AdaptiveScheduler {
            learner_model,
            bayesian_model,
            topology: topology.clone(),
            task_generator,
            epsilon: 0.15, // Start with higher exploration
            use_eig: true,
            trials_completed: 0,
            exploration_decay: 0.995, // Decay epsilon over time
            rng: rand::rngs::StdRng::from_entropy(),
        };

        debug!(
            epsilon = scheduler.epsilon,
            use_eig = scheduler.use_eig,
            "Adaptive scheduler initialized"
        );

        scheduler
    }

    pub fn new_with_eig(learner_model: LearnerModel, topology: Topology, use_eig: bool) -> Self {
        let task_generator = TaskGenerator::new(topology.clone());
        let bayesian_model = BayesianLearnerModel::new(&topology);

        use rand::SeedableRng;
        AdaptiveScheduler {
            learner_model,
            bayesian_model,
            topology: topology.clone(),
            task_generator,
            epsilon: 0.15, // Start with higher exploration
            use_eig,
            trials_completed: 0,
            exploration_decay: 0.995, // Decay epsilon over time
            rng: rand::rngs::StdRng::from_entropy(),
        }
    }

    #[instrument(level = "debug", fields(trials_completed = self.trials_completed))]
    pub fn select_next_task(&mut self) -> Task {
        let _span = span!(Level::DEBUG, "task_selection").entered();
        // Use task generator's RNG for consistent randomization

        // Adaptive epsilon-greedy: decay exploration over time
        let current_epsilon =
            self.epsilon * self.exploration_decay.powi(self.trials_completed as i32);
        let effective_epsilon = current_epsilon.max(0.01); // Minimum 1% exploration

        debug!(
            current_epsilon,
            effective_epsilon,
            trials_completed = self.trials_completed,
            "Computing exploration probability"
        );

        self.trials_completed += 1;

        let exploration_roll = self.rng.gen::<f64>();
        let is_exploration = exploration_roll < effective_epsilon;

        debug!(
            exploration_roll,
            is_exploration,
            use_eig = self.use_eig,
            "Task selection strategy determined"
        );

        let task = if is_exploration {
            // Exploration: random task
            debug!("Selecting random exploration task");
            self.task_generator.generate_task(None)
        } else {
            // Exploitation: select best task
            debug!(
                "Selecting exploitation task using {}",
                if self.use_eig { "EIG" } else { "standard" }
            );
            let candidates = self.generate_candidate_tasks();
            debug!(
                candidate_count = candidates.len(),
                "Generated task candidates"
            );

            let best_task = if self.use_eig {
                self.select_best_task_by_eig(candidates)
            } else {
                self.select_best_task(candidates)
            };
            best_task
        };

        info!(
            task_type = ?task.task_type,
            task_prompt = %task.prompt,
            is_exploration,
            trials_completed = self.trials_completed,
            "Task selected"
        );

        task
    }

    /// Update the scheduler after receiving a response
    #[instrument(level = "debug", fields(
        task_type = ?task.task_type,
        correct = correct,
        response_time = response_time
    ))]
    pub fn update_after_response(&mut self, task: &Task, correct: bool, response_time: f64) {
        debug!(
            task_prompt = %task.prompt,
            correct,
            response_time,
            "Updating scheduler with response"
        );

        // Update Bayesian model
        self.bayesian_model.update_with_response(ResponseData {
            task: task.clone(),
            correct,
            response_time,
        });

        // Update learner model
        self.learner_model
            .update_operation_proficiency(&task.operation, correct);

        info!(
            operation = %task.operation,
            correct,
            trials_completed = self.trials_completed,
            "Scheduler updated with response data"
        );
    }

    fn select_best_task_by_eig(&mut self, candidates: Vec<Task>) -> Task {
        let ranked = self.bayesian_model.rank_tasks_by_eig(candidates);

        // Filter by difficulty zone (70-80% success rate)
        let p_correct_target = 0.75;
        let tolerance = 0.15;

        let mut best_task = None;
        for (task, _eig) in &ranked {
            let p_correct = self
                .learner_model
                .get_probability_correct(&task.operation, task.difficulty);
            if (p_correct - p_correct_target).abs() < tolerance {
                best_task = Some(task.clone());
                break;
            }
        }

        // If no task in target difficulty, return highest EIG
        best_task.unwrap_or_else(|| {
            ranked
                .into_iter()
                .next()
                .map(|(task, _)| task)
                .unwrap_or_else(|| Task {
                    task_type: TaskType::Successor {
                        item: "A".to_string(),
                    },
                    prompt: "What comes after 'A'?".to_string(),
                    correct_answer: "B".to_string(),
                    options: vec!["B".to_string(), "C".to_string()],
                    difficulty: 0.3,
                    operation: crate::core::learner::OperationType::Successor,
                })
        })
    }

    fn generate_candidate_tasks(&mut self) -> Vec<Task> {
        let mut candidates = Vec::new();

        let nodes = self.topology.nodes.clone();
        let n = nodes.len();
        // Use task generator's RNG for consistent randomization

        for _ in 0..5 {
            let idx1 = self.rng.gen_range(0..n);
            let idx2 = self.rng.gen_range(0..n);
            candidates.push(
                self.task_generator
                    .generate_task(Some(TaskType::PairwiseOrder {
                        a: nodes[idx1].label.clone(),
                        b: nodes[idx2].label.clone(),
                    })),
            );
        }

        for _ in 0..3 {
            let idx = self.rng.gen_range(0..n - 1);
            candidates.push(self.task_generator.generate_task(Some(TaskType::Successor {
                item: nodes[idx].label.clone(),
            })));
        }

        for _ in 0..3 {
            let idx = self.rng.gen_range(1..n);
            candidates.push(
                self.task_generator
                    .generate_task(Some(TaskType::Predecessor {
                        item: nodes[idx].label.clone(),
                    })),
            );
        }

        for _ in 0..3 {
            let idx = self.rng.gen_range(0..n);
            let k = self.rng.gen_range(1..4);
            candidates.push(self.task_generator.generate_task(Some(TaskType::KJump {
                start: nodes[idx].label.clone(),
                k: if self.rng.gen_bool(0.5) { k } else { -k },
            })));
        }

        for _ in 0..2 {
            let idx = self.rng.gen_range(0..n.saturating_sub(3));
            let count = self.rng.gen_range(2..5);
            let reverse = self.rng.gen_bool(0.5);
            candidates.push(self.task_generator.generate_task(Some(TaskType::Segment {
                start: nodes[idx].label.clone(),
                count,
                reverse,
            })));
        }

        candidates
    }

    fn select_best_task(&self, candidates: Vec<Task>) -> Task {
        let mut best_task = candidates[0].clone();
        let mut best_score = 0.0;

        for task in candidates {
            let score = self.calculate_task_score(&task);
            if score > best_score {
                best_score = score;
                best_task = task;
            }
        }

        best_task
    }

    fn calculate_task_score(&self, task: &Task) -> f64 {
        let p_correct = self
            .learner_model
            .get_probability_correct(&task.operation, task.difficulty);

        let target_difficulty = 0.75;
        let difficulty_score = 1.0 - (p_correct - target_difficulty).abs();

        let uncertainty_score = self.calculate_uncertainty_reduction(task);

        let practice_score = self.calculate_practice_need(task);

        let weak_link_score = self.calculate_weak_link_score(task);

        0.3 * difficulty_score
            + 0.3 * uncertainty_score
            + 0.2 * practice_score
            + 0.2 * weak_link_score
    }

    fn calculate_uncertainty_reduction(&self, task: &Task) -> f64 {
        match &task.task_type {
            TaskType::PairwiseOrder { a, b } => {
                let node_a = self.topology.get_node_by_label(a);
                let node_b = self.topology.get_node_by_label(b);

                if let (Some(na), Some(nb)) = (node_a, node_b) {
                    let embedding_a = self.learner_model.node_embeddings.get(&na.id);
                    let embedding_b = self.learner_model.node_embeddings.get(&nb.id);

                    if let (Some(ea), Some(eb)) = (embedding_a, embedding_b) {
                        return (ea.uncertainty + eb.uncertainty) / 2.0;
                    }
                }
                0.5
            }
            TaskType::Successor { item } | TaskType::Predecessor { item } => {
                if let Some(node) = self.topology.get_node_by_label(item) {
                    if let Some(embedding) = self.learner_model.node_embeddings.get(&node.id) {
                        return embedding.uncertainty;
                    }
                }
                0.5
            }
            _ => 0.5,
        }
    }

    fn calculate_practice_need(&self, task: &Task) -> f64 {
        let op_key = format!("{:?}", task.operation);

        if let Some(prof) = self.learner_model.operation_proficiencies.get(&op_key) {
            let practice_factor = 1.0 / (1.0 + prof.practice_count as f64 / 10.0);
            let proficiency_factor = 1.0 - self.sigmoid(prof.theta);
            (practice_factor + proficiency_factor) / 2.0
        } else {
            1.0
        }
    }

    fn calculate_weak_link_score(&self, task: &Task) -> f64 {
        match &task.task_type {
            TaskType::Segment { start, count, .. } => {
                if let Some(node) = self.topology.get_node_by_label(start) {
                    let start_idx = self.topology.node_map[&node.id];
                    let end_idx = (start_idx + count).min(self.topology.nodes.len());

                    let boundary_crossings = self
                        .learner_model
                        .chunk_boundaries
                        .iter()
                        .filter(|b| b.position >= start_idx && b.position < end_idx)
                        .count();

                    return (boundary_crossings as f64 / 3.0).min(1.0);
                }
                0.0
            }
            TaskType::KJump { start, k } => {
                if let Some(node) = self.topology.get_node_by_label(start) {
                    let start_idx = self.topology.node_map[&node.id];
                    let end_idx = (start_idx as i32 + k).max(0) as usize;

                    let boundary_crossings = self
                        .learner_model
                        .chunk_boundaries
                        .iter()
                        .filter(|b| {
                            let min = start_idx.min(end_idx);
                            let max = start_idx.max(end_idx);
                            b.position >= min && b.position <= max
                        })
                        .count();

                    return (boundary_crossings as f64 / 2.0).min(1.0);
                }
                0.0
            }
            _ => 0.0,
        }
    }

    fn sigmoid(&self, x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    pub fn update_model(&mut self, task: &Task, correct: bool, response_time_ms: u128) {
        // Update Bayesian model with response
        let response_data = ResponseData {
            task: task.clone(),
            correct,
            response_time: response_time_ms as f64,
        };
        self.bayesian_model.update_with_response(response_data);

        // Continue with existing updates
        self.learner_model
            .update_operation_proficiency(&task.operation, correct);

        match &task.task_type {
            TaskType::PairwiseOrder { a, b } => {
                if let (Some(na), Some(nb)) = (
                    self.topology.get_node_by_label(a),
                    self.topology.get_node_by_label(b),
                ) {
                    if correct {
                        self.learner_model
                            .update_node_embedding(&na.id, na.position, 0.1);
                        self.learner_model
                            .update_node_embedding(&nb.id, nb.position, 0.1);
                    } else {
                        self.learner_model
                            .update_confusability(&na.id, &nb.id, true);
                    }
                    self.learner_model.update_memory_strength(&na.id, correct);
                    self.learner_model.update_memory_strength(&nb.id, correct);
                }
            }
            TaskType::Successor { item } | TaskType::Predecessor { item } => {
                if let Some(node) = self.topology.get_node_by_label(item) {
                    self.learner_model.update_node_embedding(
                        &node.id,
                        node.position,
                        if correct { 0.1 } else { 0.05 },
                    );
                    self.learner_model.update_memory_strength(&node.id, correct);
                }
            }
            TaskType::Segment { start, count, .. } => {
                let segment = self.topology.get_segment(start, *count, false);
                for label in segment {
                    if let Some(node) = self.topology.get_node_by_label(&label) {
                        self.learner_model.update_memory_strength(&node.id, correct);
                    }
                }

                if response_time_ms > 5000 {
                    if let Some(node) = self.topology.get_node_by_label(start) {
                        let start_idx = self.topology.node_map[&node.id];
                        for boundary in &mut self.learner_model.chunk_boundaries {
                            if boundary.position > start_idx
                                && boundary.position < start_idx + count
                            {
                                boundary.strength *= if correct { 0.95 } else { 1.05 };
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn get_learner_model(&self) -> &LearnerModel {
        &self.learner_model
    }

    pub fn get_learner_model_mut(&mut self) -> &mut LearnerModel {
        &mut self.learner_model
    }

    pub fn get_bayesian_model(&self) -> &BayesianLearnerModel {
        &self.bayesian_model
    }

    pub fn get_bayesian_model_mut(&mut self) -> &mut BayesianLearnerModel {
        &mut self.bayesian_model
    }

    pub fn get_model_entropy(&self) -> f64 {
        self.bayesian_model.total_entropy()
    }
}

#[allow(dead_code)]
pub struct InformationGain {
    entropy_before: f64,
    entropy_after: f64,
    gain: f64,
}

impl InformationGain {
    pub fn calculate(learner_model: &LearnerModel, _task: &Task) -> Self {
        let entropy_before = Self::calculate_entropy(learner_model);

        let entropy_after = entropy_before * 0.95;

        InformationGain {
            entropy_before,
            entropy_after,
            gain: entropy_before - entropy_after,
        }
    }

    fn calculate_entropy(model: &LearnerModel) -> f64 {
        let mut total_entropy = 0.0;

        for embedding in model.node_embeddings.values() {
            total_entropy += embedding.uncertainty * embedding.uncertainty.ln();
        }

        for prof in model.operation_proficiencies.values() {
            let p = 1.0 / (1.0 + (-prof.theta).exp());
            if p > 0.0 && p < 1.0 {
                total_entropy -= p * p.ln() + (1.0 - p) * (1.0 - p).ln();
            }
        }

        total_entropy.abs()
    }
}
