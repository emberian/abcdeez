use crate::config::LearnerConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentNodeEmbedding {
    pub node_id: String,
    pub position: f64,
    pub uncertainty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OperationType {
    Successor,
    Predecessor,
    PairwiseOrder,
    KJump(i32),
    Segment(usize, bool),
    Index,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Successor => write!(f, "Successor"),
            OperationType::Predecessor => write!(f, "Predecessor"),
            OperationType::PairwiseOrder => write!(f, "PairwiseOrder"),
            OperationType::KJump(k) => write!(f, "KJump({})", k),
            OperationType::Segment(size, ordered) => write!(f, "Segment({}, {})", size, ordered),
            OperationType::Index => write!(f, "Index"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProficiency {
    pub operation: OperationType,
    pub theta: f64,
    pub practice_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStrength {
    pub node_id: String,
    pub strength: f64,
    pub last_practice: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkBoundary {
    pub position: usize,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerModel {
    pub learner_id: String,
    pub node_embeddings: HashMap<String, LatentNodeEmbedding>,
    pub operation_proficiencies: HashMap<String, OperationProficiency>,
    pub memory_strengths: HashMap<String, MemoryStrength>,
    pub confusability_matrix: HashMap<(String, String), f64>,
    pub chunk_boundaries: Vec<ChunkBoundary>,
    pub total_practice_time: std::time::Duration,
    pub session_count: usize,
    pub config: LearnerConfig,
}

impl LearnerModel {
    pub fn new(learner_id: String, topology: &crate::core::topology::Topology) -> Self {
        Self::new_with_config(learner_id, topology, LearnerConfig::adult())
    }

    pub fn new_with_config(
        learner_id: String,
        topology: &crate::core::topology::Topology,
        config: LearnerConfig,
    ) -> Self {
        let mut node_embeddings = HashMap::new();
        let mut memory_strengths = HashMap::new();

        for node in &topology.nodes {
            node_embeddings.insert(
                node.id.clone(),
                LatentNodeEmbedding {
                    node_id: node.id.clone(),
                    position: node.position + rand::random::<f64>() * 0.5 - 0.25,
                    uncertainty: config.initial_uncertainty,
                },
            );

            memory_strengths.insert(
                node.id.clone(),
                MemoryStrength {
                    node_id: node.id.clone(),
                    strength: config.initial_memory_strength, // Start at neutral strength
                    last_practice: chrono::Utc::now(),
                },
            );
        }

        let mut operation_proficiencies = HashMap::new();
        let operations = vec![
            OperationType::Successor,
            OperationType::Predecessor,
            OperationType::PairwiseOrder,
            OperationType::KJump(2),
            OperationType::KJump(3),
            OperationType::Segment(3, false),
            OperationType::Segment(3, true),
            OperationType::Index,
        ];

        for op in operations {
            let key = format!("{:?}", op);
            operation_proficiencies.insert(
                key,
                OperationProficiency {
                    operation: op,
                    theta: config.initial_proficiency,
                    practice_count: 0,
                },
            );
        }

        let chunk_boundaries = match topology.topology_type {
            crate::core::topology::TopologyType::Linear => {
                vec![
                    ChunkBoundary {
                        position: 6,
                        strength: 0.8,
                    },
                    ChunkBoundary {
                        position: 13,
                        strength: 0.8,
                    },
                    ChunkBoundary {
                        position: 19,
                        strength: 0.8,
                    },
                ]
            }
            crate::core::topology::TopologyType::Cyclic => vec![],
            crate::core::topology::TopologyType::PartialOrder => vec![],
            crate::core::topology::TopologyType::GeneralGraph => vec![],
        };

        LearnerModel {
            learner_id,
            node_embeddings,
            operation_proficiencies,
            memory_strengths,
            confusability_matrix: HashMap::new(),
            chunk_boundaries,
            total_practice_time: std::time::Duration::new(0, 0),
            session_count: 0,
            config,
        }
    }

    pub fn update_node_embedding(
        &mut self,
        node_id: &str,
        new_position: f64,
        reduce_uncertainty: f64,
    ) {
        if let Some(embedding) = self.node_embeddings.get_mut(node_id) {
            let old_weight = self.config.position_update_weight;
            let new_weight = 1.0 - old_weight;
            embedding.position = old_weight * embedding.position + new_weight * new_position;
            embedding.uncertainty *= (1.0 - reduce_uncertainty).max(self.config.min_uncertainty);
        }
    }

    pub fn update_operation_proficiency(&mut self, operation: &OperationType, success: bool) {
        let key = format!("{:?}", operation);
        if let Some(prof) = self.operation_proficiencies.get_mut(&key) {
            prof.practice_count += 1;

            // Adaptive learning rate based on:
            // 1. Current proficiency (learn faster when less proficient)
            // 2. Practice count (decrease learning rate over time)
            // 3. Recent performance (adjust based on consistency)

            // Base learning rate decreases with practice (power law)
            let base_rate = self.config.learning_rate_base
                / (1.0 + prof.practice_count as f64).powf(self.config.learning_rate_decay);

            // Adjust based on current proficiency level
            // Learn faster in the middle range, slower at extremes
            let theta_range = self.config.theta_bounds.1 - self.config.theta_bounds.0;
            let proficiency_factor = 1.0 - (prof.theta.abs() / (theta_range / 2.0)).min(1.0);

            // Calculate adaptive learning rate
            let learning_rate = (base_rate * (0.5 + proficiency_factor)).max(0.01).min(0.5);

            if success {
                // Update with diminishing returns as proficiency increases
                prof.theta += learning_rate * (1.0 - sigmoid(prof.theta));
            } else {
                // Larger penalty for errors at high proficiency
                let error_weight = if prof.theta > 1.0 { 1.5 } else { 1.0 };
                prof.theta -= learning_rate * sigmoid(prof.theta) * error_weight;
            }

            // Ensure theta stays within reasonable bounds
            prof.theta = prof
                .theta
                .max(self.config.theta_bounds.0)
                .min(self.config.theta_bounds.1);
        }
    }

    pub fn update_memory_strength(&mut self, node_id: &str, correct: bool) {
        // Handle both "A" and "node_0" formats
        let key = if node_id.starts_with("node_") {
            node_id.to_string()
        } else {
            // Convert letter to node_X format
            let idx = (node_id.chars().next().unwrap_or('A') as usize) - ('A' as usize);
            format!("node_{}", idx)
        };

        if let Some(mem) = self.memory_strengths.get_mut(&key) {
            let now = chrono::Utc::now();
            let time_since = now.signed_duration_since(mem.last_practice);
            let hours_since = time_since.num_hours() as f64;

            let current_strength = mem.strength;
            let decay_rate = Self::calculate_decay_rate_static(current_strength);
            let decayed_strength =
                Self::apply_forgetting_curve_static(current_strength, hours_since, decay_rate);

            if correct {
                mem.strength = (decayed_strength + self.config.memory_update_correct).min(1.0);
            } else {
                mem.strength = (decayed_strength + self.config.memory_update_incorrect).max(0.0);
            }

            mem.last_practice = now;
        }
    }

    fn calculate_decay_rate_static(current_strength: f64) -> f64 {
        0.05 * (2.0 - current_strength)
    }

    fn apply_forgetting_curve_static(strength: f64, hours_elapsed: f64, decay_rate: f64) -> f64 {
        strength * (-decay_rate * hours_elapsed).exp()
    }

    fn calculate_decay_rate(&self, current_strength: f64) -> f64 {
        self.config.memory_decay_rate * (2.0 - current_strength)
    }

    fn apply_forgetting_curve(&self, strength: f64, hours_elapsed: f64, decay_rate: f64) -> f64 {
        strength * (-decay_rate * hours_elapsed).exp()
    }

    pub fn get_retention_probability(&self, node_id: &str) -> f64 {
        if let Some(mem) = self.memory_strengths.get(node_id) {
            let now = chrono::Utc::now();
            let time_since = now.signed_duration_since(mem.last_practice);
            // Use sub-hour resolution to make short-interval tests meaningful
            let hours_since = (time_since.num_seconds() as f64) / 3600.0;

            let decay_rate = self.calculate_decay_rate(mem.strength);
            let mut retention = self.apply_forgetting_curve(mem.strength, hours_since, decay_rate);

            // Small primacy/recency boost to reflect empirically observed edge benefits
            if let Some(embed) = self.node_embeddings.get(node_id) {
                let n_nodes = self.node_embeddings.len().max(1) as f64;
                // Normalize position to [0,1]
                let pos_norm = if n_nodes > 1.0 {
                    embed.position / (n_nodes - 1.0)
                } else {
                    0.0
                };
                // Cosine-based edge emphasis: peaks at edges, lowest in middle
                let edge_emphasis = (std::f64::consts::PI * pos_norm).cos().abs();
                // Keep boost very small so it doesn't break decay expectations
                let boost = 1.0 + self.config.edge_emphasis * edge_emphasis;
                retention = (retention * boost).min(1.0);
            }

            retention
        } else {
            0.0
        }
    }

    pub fn get_optimal_review_time(
        &self,
        node_id: &str,
        target_retention: f64,
    ) -> Option<chrono::DateTime<chrono::Utc>> {
        let mem = self.memory_strengths.get(node_id)?;

        if mem.strength <= target_retention {
            return Some(chrono::Utc::now());
        }

        let decay_rate = self.calculate_decay_rate(mem.strength);
        let hours_until_review = -(target_retention / mem.strength).ln() / decay_rate;

        Some(mem.last_practice + chrono::Duration::hours(hours_until_review as i64))
    }

    pub fn get_items_needing_review(&self, threshold: f64) -> Vec<String> {
        let mut items = Vec::new();

        for (node_id, _) in &self.memory_strengths {
            if self.get_retention_probability(node_id) < threshold {
                if let Some(node) = self.node_embeddings.get(node_id) {
                    items.push(node.node_id.clone());
                }
            }
        }

        items
    }

    pub fn update_confusability(&mut self, node_a: &str, node_b: &str, confused: bool) {
        let key = if node_a < node_b {
            (node_a.to_string(), node_b.to_string())
        } else {
            (node_b.to_string(), node_a.to_string())
        };

        let current = self.confusability_matrix.get(&key).unwrap_or(&0.0);
        let new_value = if confused {
            (current + 0.1).min(1.0)
        } else {
            (current * 0.9).max(0.0)
        };

        self.confusability_matrix.insert(key, new_value);
    }

    pub fn get_probability_correct(&self, operation: &OperationType, difficulty: f64) -> f64 {
        let key = format!("{:?}", operation);
        let prof = self.operation_proficiencies.get(&key);

        let theta = prof.map(|p| p.theta).unwrap_or(-1.0);

        sigmoid(theta - difficulty)
    }

    pub fn predict_response_time(&self, operation: &OperationType, distance: usize) -> f64 {
        let key = format!("{:?}", operation);
        let prof = self.operation_proficiencies.get(&key);

        let base_rt = 1000.0;
        let distance_penalty = 100.0 * distance as f64;

        let proficiency_bonus = prof.map(|p| 200.0 * sigmoid(p.theta)).unwrap_or(0.0);

        let boundary_penalty = self
            .chunk_boundaries
            .iter()
            .filter(|b| b.position < distance)
            .map(|b| 200.0 * b.strength)
            .sum::<f64>();

        (base_rt + distance_penalty + boundary_penalty - proficiency_bonus).max(300.0)
    }

    pub fn get_bidirectionality_index(&self) -> f64 {
        let forward = self
            .operation_proficiencies
            .get(&format!("{:?}", OperationType::Successor));
        let backward = self
            .operation_proficiencies
            .get(&format!("{:?}", OperationType::Predecessor));

        match (forward, backward) {
            (Some(f), Some(b)) => (f.theta - b.theta).abs(),
            _ => 1.0,
        }
    }

    pub fn get_symbolic_distance_slope(&self) -> f64 {
        let distances = vec![1, 2, 3, 4, 5];
        let mut rts = Vec::new();

        for d in &distances {
            let rt = self.predict_response_time(&OperationType::PairwiseOrder, *d);
            rts.push(rt);
        }

        let n = distances.len() as f64;
        let sum_x: f64 = distances.iter().sum::<usize>() as f64;
        let sum_y: f64 = rts.iter().sum();
        let sum_xy: f64 = distances
            .iter()
            .zip(rts.iter())
            .map(|(x, y)| *x as f64 * y)
            .sum();
        let sum_x2: f64 = distances.iter().map(|x| (*x * *x) as f64).sum();

        (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
    }

    pub fn get_chunk_boundary_penalty(&self) -> f64 {
        self.chunk_boundaries
            .iter()
            .map(|b| b.strength)
            .sum::<f64>()
            / self.chunk_boundaries.len().max(1) as f64
    }

    pub fn update_chunk_boundaries(&mut self, crossed_boundary: Option<usize>) {
        if let Some(pos) = crossed_boundary {
            if let Some(boundary) = self.chunk_boundaries.iter_mut().find(|b| b.position == pos) {
                boundary.strength = (boundary.strength + 0.1).min(1.0);
            } else {
                self.chunk_boundaries.push(ChunkBoundary {
                    position: pos,
                    strength: 0.1,
                });
            }
        }
    }

    /// Apply identifiability constraints to prevent gauge freedom in embeddings
    /// This fixes the first and last nodes' positions and centers the embeddings
    pub fn apply_identifiability_constraints(&mut self) {
        // Fix gauge freedom by anchoring first and last nodes
        if let Some(first_node) = self
            .node_embeddings
            .values()
            .min_by_key(|n| n.position as i64)
        {
            let first_id = first_node.node_id.clone();
            if let Some(first) = self.node_embeddings.get_mut(&first_id) {
                first.position = 0.0;
                first.uncertainty = 0.01; // Very certain about anchor
            }
        }

        if let Some(last_node) = self
            .node_embeddings
            .values()
            .max_by_key(|n| n.position as i64)
        {
            let last_id = last_node.node_id.clone();
            let n_nodes = self.node_embeddings.len() as f64;
            if let Some(last) = self.node_embeddings.get_mut(&last_id) {
                last.position = n_nodes - 1.0;
                last.uncertainty = 0.01; // Very certain about anchor
            }
        }

        // Center the embeddings to prevent drift
        let mean_position: f64 = self
            .node_embeddings
            .values()
            .map(|n| n.position)
            .sum::<f64>()
            / self.node_embeddings.len() as f64;
        let target_mean = (self.node_embeddings.len() as f64 - 1.0) / 2.0;
        let shift = target_mean - mean_position;

        for embedding in self.node_embeddings.values_mut() {
            // Don't shift the anchored nodes
            if embedding.uncertainty > 0.01 {
                embedding.position += shift;
            }
        }

        // Normalize uncertainties to prevent explosion
        let max_uncertainty = self
            .node_embeddings
            .values()
            .map(|n| n.uncertainty)
            .fold(0.0, f64::max);

        if max_uncertainty > 10.0 {
            for embedding in self.node_embeddings.values_mut() {
                embedding.uncertainty = embedding.uncertainty / max_uncertainty * 10.0;
            }
        }
    }

    /// Regularize embeddings to maintain proper ordering and spacing
    pub fn regularize_embeddings(&mut self, lambda: f64) {
        // Sort nodes by position
        let mut sorted_nodes: Vec<_> = self.node_embeddings.values().cloned().collect();
        sorted_nodes.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());

        // Apply regularization to maintain minimum spacing
        let min_spacing = 0.1;
        for i in 1..sorted_nodes.len() {
            let prev_pos = sorted_nodes[i - 1].position;
            let curr_pos = sorted_nodes[i].position;

            if curr_pos - prev_pos < min_spacing {
                // Push current node forward
                if let Some(node) = self.node_embeddings.get_mut(&sorted_nodes[i].node_id) {
                    node.position = prev_pos + min_spacing;
                }
            }
        }

        // Apply L2 regularization to prevent extreme positions
        for embedding in self.node_embeddings.values_mut() {
            let expected_pos = embedding
                .node_id
                .chars()
                .next()
                .and_then(|c| {
                    if c.is_ascii_uppercase() {
                        Some((c as u8 - b'A') as f64)
                    } else {
                        None
                    }
                })
                .unwrap_or(embedding.position);

            // Pull towards expected position with strength lambda
            embedding.position = (1.0 - lambda) * embedding.position + lambda * expected_pos;
        }
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerMetrics {
    pub bidirectionality_index: f64,
    pub symbolic_distance_slope: f64,
    pub chunk_boundary_penalty: f64,
    pub avg_memory_strength: f64,
    pub operation_proficiencies: HashMap<String, f64>,
}

impl LearnerMetrics {
    pub fn from_model(model: &LearnerModel) -> Self {
        let avg_memory_strength = model
            .memory_strengths
            .values()
            .map(|m| m.strength)
            .sum::<f64>()
            / model.memory_strengths.len().max(1) as f64;

        let mut operation_proficiencies = HashMap::new();
        for (key, prof) in &model.operation_proficiencies {
            operation_proficiencies.insert(key.clone(), sigmoid(prof.theta));
        }

        LearnerMetrics {
            bidirectionality_index: model.get_bidirectionality_index(),
            symbolic_distance_slope: model.get_symbolic_distance_slope(),
            chunk_boundary_penalty: model.get_chunk_boundary_penalty(),
            avg_memory_strength,
            operation_proficiencies,
        }
    }
}
