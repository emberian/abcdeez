use crate::core::learner::{LearnerModel, OperationType};
use crate::tasks::types::{Task, TaskType};
use crate::core::topology::{Topology, TopologyType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an isomorphic mapping between two domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsomorphicMapping {
    pub source_domain: String,
    pub target_domain: String,
    pub node_mapping: HashMap<String, String>,
    pub structure_preserved: bool,
    pub transfer_strength: f64, // 0.0 to 1.0
}

/// Types of domain transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferType {
    Direct,       // One-to-one mapping (alphabet -> numbers)
    Structural,   // Same structure, different labels
    Functional,   // Same function, different representation
    Analogical,   // Partial similarity
    Hierarchical, // Transfer at different abstraction levels
}

/// System for managing transfer learning between domains
pub struct TransferLearningSystem {
    source_topology: Topology,
    target_topology: Topology,
    mapping: IsomorphicMapping,
    transfer_type: TransferType,
    source_model: Option<LearnerModel>,
    transfer_history: Vec<TransferEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEvent {
    pub task_type: String,
    pub source_performance: f64,
    pub target_performance: f64,
    pub transfer_efficiency: f64,
    pub timestamp: usize,
}

impl TransferLearningSystem {
    pub fn new(source: Topology, target: Topology, transfer_type: TransferType) -> Self {
        let mapping = Self::compute_isomorphic_mapping(&source, &target);

        TransferLearningSystem {
            source_topology: source,
            target_topology: target,
            mapping,
            transfer_type,
            source_model: None,
            transfer_history: Vec::new(),
        }
    }

    /// Compute the isomorphic mapping between two topologies
    fn compute_isomorphic_mapping(source: &Topology, target: &Topology) -> IsomorphicMapping {
        let mut node_mapping = HashMap::new();

        // Check if topologies have the same structure
        let structure_preserved = source.nodes.len() == target.nodes.len()
            && source.topology_type == target.topology_type;

        if structure_preserved {
            // Direct position-based mapping
            for i in 0..source.nodes.len().min(target.nodes.len()) {
                node_mapping.insert(source.nodes[i].label.clone(), target.nodes[i].label.clone());
            }
        } else {
            // Partial mapping based on relative positions
            let ratio = target.nodes.len() as f64 / source.nodes.len() as f64;
            for (i, source_node) in source.nodes.iter().enumerate() {
                let target_idx = ((i as f64 * ratio) as usize).min(target.nodes.len() - 1);
                node_mapping.insert(
                    source_node.label.clone(),
                    target.nodes[target_idx].label.clone(),
                );
            }
        }

        IsomorphicMapping {
            source_domain: Self::get_domain_name(source),
            target_domain: Self::get_domain_name(target),
            node_mapping,
            structure_preserved,
            transfer_strength: if structure_preserved { 0.9 } else { 0.5 },
        }
    }

    fn get_domain_name(topology: &Topology) -> String {
        if topology.nodes.len() == 26 {
            "alphabet".to_string()
        } else if topology.nodes.len() == 7 {
            "days_of_week".to_string()
        } else if topology.nodes.len() == 12 {
            "months".to_string()
        } else if topology.nodes.len() == 10 {
            "digits".to_string()
        } else {
            format!("domain_{}", topology.nodes.len())
        }
    }

    /// Set the source model for transfer
    pub fn set_source_model(&mut self, model: LearnerModel) {
        self.source_model = Some(model);
    }

    /// Transfer knowledge from source to target domain
    pub fn transfer_knowledge(&self) -> LearnerModel {
        let mut target_model =
            LearnerModel::new("transfer_target".to_string(), &self.target_topology);

        if let Some(source_model) = &self.source_model {
            match self.transfer_type {
                TransferType::Direct => self.direct_transfer(source_model, &mut target_model),
                TransferType::Structural => {
                    self.structural_transfer(source_model, &mut target_model)
                }
                TransferType::Functional => {
                    self.functional_transfer(source_model, &mut target_model)
                }
                TransferType::Analogical => {
                    self.analogical_transfer(source_model, &mut target_model)
                }
                TransferType::Hierarchical => {
                    self.hierarchical_transfer(source_model, &mut target_model)
                }
            }
        }

        target_model
    }

    fn direct_transfer(&self, source: &LearnerModel, target: &mut LearnerModel) {
        // Transfer node embeddings with direct mapping
        for (source_label, target_label) in &self.mapping.node_mapping {
            if let (Some(source_node), Some(target_node)) = (
                self.source_topology
                    .nodes
                    .iter()
                    .find(|n| &n.label == source_label),
                self.target_topology
                    .nodes
                    .iter()
                    .find(|n| &n.label == target_label),
            ) {
                // Transfer embedding with transfer strength scaling
                if let Some(source_embedding) = source.node_embeddings.get(&source_node.id) {
                    if let Some(target_embedding) = target.node_embeddings.get_mut(&target_node.id)
                    {
                        target_embedding.position = source_embedding.position;
                        target_embedding.uncertainty =
                            source_embedding.uncertainty / self.mapping.transfer_strength;
                    }
                }
            }
        }

        // Transfer operation proficiencies with scaling
        for (op_key, source_prof) in &source.operation_proficiencies {
            if let Some(target_prof) = target.operation_proficiencies.get_mut(op_key) {
                target_prof.theta = source_prof.theta * self.mapping.transfer_strength;
                target_prof.practice_count =
                    (source_prof.practice_count as f64 * self.mapping.transfer_strength) as usize;
            }
        }
    }

    fn structural_transfer(&self, source: &LearnerModel, target: &mut LearnerModel) {
        // Transfer structural knowledge (relative positions, distances)
        if self.mapping.structure_preserved {
            // Transfer chunk boundaries
            target.chunk_boundaries = source
                .chunk_boundaries
                .iter()
                .map(|cb| crate::core::learner::ChunkBoundary {
                    position: cb.position,
                    strength: cb.strength * self.mapping.transfer_strength,
                })
                .collect();

            // Transfer relative position knowledge
            for (node_id, source_embedding) in &source.node_embeddings {
                if let Some(target_embedding) = target.node_embeddings.get_mut(node_id) {
                    target_embedding.position = source_embedding.position * 0.8;
                    target_embedding.uncertainty = source_embedding.uncertainty * 0.8;
                }
            }
        }
    }

    fn functional_transfer(&self, source: &LearnerModel, target: &mut LearnerModel) {
        // Transfer functional relationships (e.g., successor/predecessor patterns)

        // Transfer operation proficiencies with higher weight for structural operations
        for (op_key, source_prof) in &source.operation_proficiencies {
            if let Some(target_prof) = target.operation_proficiencies.get_mut(op_key) {
                let transfer_weight = match &source_prof.operation {
                    OperationType::Successor | OperationType::Predecessor => 0.9,
                    OperationType::KJump(_) => 0.8,
                    OperationType::PairwiseOrder => 0.7,
                    _ => 0.5,
                };

                target_prof.theta =
                    source_prof.theta * transfer_weight * self.mapping.transfer_strength;
            }
        }
    }

    fn analogical_transfer(&self, source: &LearnerModel, target: &mut LearnerModel) {
        // Transfer based on analogical reasoning (partial similarities)

        // Find analogous patterns
        let source_patterns = self.extract_patterns(source);
        let target_patterns = self.map_patterns_to_target(&source_patterns);

        // Apply patterns with reduced strength
        for (pattern_type, strength) in target_patterns {
            match pattern_type.as_str() {
                "sequential" => {
                    // Boost sequential operation proficiencies
                    for (_op_key, prof) in &mut target.operation_proficiencies {
                        if matches!(prof.operation, OperationType::Successor) {
                            prof.theta += strength * 0.5;
                        }
                    }
                }
                "cyclic" => {
                    // Transfer cyclic knowledge if applicable
                    if matches!(self.target_topology.topology_type, TopologyType::Cyclic) {
                        for embedding in target.node_embeddings.values_mut() {
                            embedding.position *= 1.0 + strength * 0.2;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn hierarchical_transfer(&self, source: &LearnerModel, target: &mut LearnerModel) {
        // Transfer at different levels of abstraction

        // Level 1: Transfer general proficiencies
        let avg_proficiency: f64 = source
            .operation_proficiencies
            .values()
            .map(|p| p.theta)
            .sum::<f64>()
            / source.operation_proficiencies.len() as f64;

        for prof in target.operation_proficiencies.values_mut() {
            prof.theta = avg_proficiency * 0.6;
        }

        // Level 2: Transfer chunking patterns
        if source.chunk_boundaries.len() > 0 {
            let chunk_ratio =
                target.node_embeddings.len() as f64 / source.node_embeddings.len() as f64;
            target.chunk_boundaries = source
                .chunk_boundaries
                .iter()
                .map(|cb| crate::core::learner::ChunkBoundary {
                    position: (cb.position as f64 * chunk_ratio) as usize,
                    strength: cb.strength * 0.7,
                })
                .collect();
        }

        // Level 3: Transfer meta-learning parameters
        // Note: learning_rate and exploration_rate are not fields in LearnerModel
        // We would need to track these separately or add them to the model
    }

    fn extract_patterns(&self, model: &LearnerModel) -> HashMap<String, f64> {
        let mut patterns = HashMap::new();

        // Check for sequential mastery
        if let Some(prof) = model.operation_proficiencies.get("Successor") {
            patterns.insert("sequential".to_string(), prof.theta);
        }

        // Check for cyclic patterns
        if matches!(self.source_topology.topology_type, TopologyType::Cyclic) {
            patterns.insert("cyclic".to_string(), 0.8);
        }

        // Check for chunking patterns
        if !model.chunk_boundaries.is_empty() {
            let avg_strength = model
                .chunk_boundaries
                .iter()
                .map(|cb| cb.strength)
                .sum::<f64>()
                / model.chunk_boundaries.len() as f64;
            patterns.insert("chunked".to_string(), avg_strength);
        }

        patterns
    }

    fn map_patterns_to_target(
        &self,
        source_patterns: &HashMap<String, f64>,
    ) -> HashMap<String, f64> {
        let mut target_patterns = HashMap::new();

        for (pattern, &strength) in source_patterns {
            // Apply transfer function based on domain similarity
            let transferred_strength = strength * self.mapping.transfer_strength;
            target_patterns.insert(pattern.clone(), transferred_strength);
        }

        target_patterns
    }

    /// Generate a transfer learning task
    pub fn generate_transfer_task(&mut self) -> Task {
        // Select a task that was learned in source domain
        let task_types = vec![
            ("successor", OperationType::Successor),
            ("predecessor", OperationType::Predecessor),
            ("pairwise", OperationType::PairwiseOrder),
            ("segment", OperationType::Segment(3, false)),
        ];

        let (task_name, operation) = &task_types[rand::random::<usize>() % task_types.len()];

        // Map a source task to target domain
        let source_item =
            &self.source_topology.nodes[rand::random::<usize>() % self.source_topology.nodes.len()];
        let target_item = self
            .mapping
            .node_mapping
            .get(&source_item.label)
            .unwrap_or(&self.target_topology.nodes[0].label);

        let prompt = format!(
            "In the {} domain, if '{}' in {} corresponds to '{}', what comes after '{}'?",
            self.mapping.target_domain,
            source_item.label,
            self.mapping.source_domain,
            target_item,
            target_item
        );

        let correct_answer =
            if let Some(target_node) = self.target_topology.get_node_by_label(target_item) {
                if let Some(succ_id) = self.target_topology.get_successor(&target_node.id) {
                    if let Some(succ_node) = self.target_topology.get_node_by_id(&succ_id) {
                        succ_node.label.clone()
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "End of sequence".to_string()
                }
            } else {
                "Invalid mapping".to_string()
            };

        // Generate options including the correct answer and distractors
        let mut options = vec![correct_answer.clone()];

        // Add neighboring items as distractors
        for node in self.target_topology.nodes.iter().take(5) {
            if node.label != correct_answer && !options.contains(&node.label) {
                options.push(node.label.clone());
                if options.len() >= 4 {
                    break;
                }
            }
        }

        // Record transfer event
        self.transfer_history.push(TransferEvent {
            task_type: task_name.to_string(),
            source_performance: 0.8, // Assumed source mastery
            target_performance: 0.0, // To be updated after response
            transfer_efficiency: self.mapping.transfer_strength,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize,
        });

        Task {
            task_type: TaskType::Successor {
                item: target_item.clone(),
            },
            prompt,
            correct_answer,
            options,
            difficulty: 0.5 / self.mapping.transfer_strength, // Easier with better transfer
            operation: operation.clone(),
        }
    }

    /// Measure transfer effectiveness
    pub fn measure_transfer_effectiveness(
        &self,
        source_performance: &HashMap<OperationType, f64>,
        target_performance: &HashMap<OperationType, f64>,
    ) -> f64 {
        let mut total_transfer = 0.0;
        let mut count = 0;

        for (op, &source_perf) in source_performance {
            if let Some(&target_perf) = target_performance.get(op) {
                // Calculate transfer ratio
                let expected_perf = source_perf * self.mapping.transfer_strength;
                let actual_transfer = target_perf / expected_perf.max(0.1);
                total_transfer += actual_transfer.min(1.0);
                count += 1;
            }
        }

        if count > 0 {
            total_transfer / count as f64
        } else {
            0.0
        }
    }
}
