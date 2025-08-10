use crate::core::learner::{ChunkBoundary, OperationType};
use crate::tasks::types::{Task, TaskType};
use crate::core::topology::Topology;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoundaryType {
    ChunkBoundary,    // Mental segmentation boundary (e.g., F-G in alphabet)
    OctaveBoundary,   // Musical octave boundary (B→C)
    ModuleBoundary,   // Programming module boundary
    CategoryBoundary, // Category transition (vowel→consonant)
    HierarchicalBoundary { level: usize }, // Multi-level boundaries
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryBridgingTask {
    pub boundary_type: BoundaryType,
    pub span_size: usize,
    pub boundary_positions: Vec<usize>,
    pub requires_integration: bool, // Whether task specifically trains boundary crossing
}

pub struct BoundaryTrainer {
    topology: Topology,
    boundaries: Vec<ChunkBoundary>,
    hierarchical_boundaries: HashMap<usize, Vec<usize>>, // level -> positions
}

impl BoundaryTrainer {
    pub fn new(topology: Topology) -> Self {
        let mut boundaries = Vec::new();
        let mut hierarchical_boundaries = HashMap::new();

        // Default boundaries for alphabet
        if topology.nodes.len() == 26 {
            // Level 1: Major chunks (every 6-7 letters)
            boundaries.push(ChunkBoundary {
                position: 6,
                strength: 0.8,
            }); // After F
            boundaries.push(ChunkBoundary {
                position: 13,
                strength: 0.8,
            }); // After M
            boundaries.push(ChunkBoundary {
                position: 19,
                strength: 0.8,
            }); // After S

            hierarchical_boundaries.insert(1, vec![6, 13, 19]);

            // Level 2: Minor chunks (every 3 letters)
            let level2 = vec![3, 6, 9, 12, 15, 18, 21, 24];
            hierarchical_boundaries.insert(2, level2.clone());

            for pos in level2 {
                if ![6, 13, 19].contains(&pos) {
                    // Don't duplicate major boundaries
                    boundaries.push(ChunkBoundary {
                        position: pos,
                        strength: 0.4,
                    });
                }
            }

            // Level 3: Vowel-consonant transitions
            let vowel_positions = vec![0, 4, 8, 14, 20]; // A, E, I, O, U positions
            for &pos in &vowel_positions {
                if pos > 0 {
                    boundaries.push(ChunkBoundary {
                        position: pos,
                        strength: 0.3,
                    });
                }
            }
            hierarchical_boundaries.insert(3, vowel_positions);
        }

        BoundaryTrainer {
            topology,
            boundaries,
            hierarchical_boundaries,
        }
    }

    pub fn generate_boundary_bridging_task(&self, boundary_type: BoundaryType) -> Task {
        let task = match boundary_type {
            BoundaryType::ChunkBoundary => self.generate_chunk_boundary_task(),
            BoundaryType::CategoryBoundary => self.generate_category_boundary_task(),
            BoundaryType::HierarchicalBoundary { level } => {
                self.generate_hierarchical_boundary_task(level)
            }
            _ => self.generate_chunk_boundary_task(), // Default
        };

        task
    }

    fn generate_chunk_boundary_task(&self) -> Task {
        // Select a boundary to cross
        let boundary = &self.boundaries[rand::random::<usize>() % self.boundaries.len()];

        // Generate a task that spans this boundary
        let start_idx = boundary.position.saturating_sub(2);
        let span_size = 5; // Ensures crossing the boundary

        let start_node = &self.topology.nodes[start_idx];
        let segment = self
            .topology
            .get_segment(&start_node.label, span_size, false);

        let prompt = format!(
            "List {} items starting from '{}' (crosses mental boundary at position {})",
            span_size, start_node.label, boundary.position
        );

        let correct_answer = segment.join(", ");

        // Calculate difficulty based on boundary strength
        let base_difficulty = 0.4;
        let boundary_penalty = boundary.strength * 0.3;
        let difficulty = (base_difficulty + boundary_penalty).min(0.9);

        Task {
            task_type: TaskType::Segment {
                start: start_node.label.clone(),
                count: span_size,
                reverse: false,
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty,
            operation: OperationType::Segment(span_size, false),
        }
    }

    fn generate_category_boundary_task(&self) -> Task {
        // Find vowel-consonant transitions
        let vowels = vec!["A", "E", "I", "O", "U"];
        let mut transitions = Vec::new();

        for i in 0..self.topology.nodes.len() - 1 {
            let curr = &self.topology.nodes[i].label;
            let next = &self.topology.nodes[i + 1].label;

            let curr_is_vowel = vowels.contains(&curr.as_str());
            let next_is_vowel = vowels.contains(&next.as_str());

            if curr_is_vowel != next_is_vowel {
                transitions.push(i);
            }
        }

        if transitions.is_empty() {
            return self.generate_chunk_boundary_task();
        }

        let transition_idx = transitions[rand::random::<usize>() % transitions.len()];
        let start_idx = transition_idx.saturating_sub(1);
        let span_size = 4;

        let start_node = &self.topology.nodes[start_idx];
        let segment = self
            .topology
            .get_segment(&start_node.label, span_size, false);

        let prompt = format!(
            "List {} items starting from '{}' (crosses vowel-consonant boundary)",
            span_size, start_node.label
        );

        let correct_answer = segment.join(", ");

        Task {
            task_type: TaskType::Segment {
                start: start_node.label.clone(),
                count: span_size,
                reverse: false,
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.6,
            operation: OperationType::Segment(span_size, false),
        }
    }

    /// Generate a boundary navigation task that requires explicit boundary awareness
    pub fn generate_boundary_navigation_task(&self) -> Task {
        // Select two boundaries to navigate between
        let boundary1 = &self.boundaries[rand::random::<usize>() % self.boundaries.len()];
        let boundary2 = &self.boundaries[rand::random::<usize>() % self.boundaries.len()];

        let start_pos = boundary1.position.min(boundary2.position);
        let end_pos = boundary1.position.max(boundary2.position);

        let start_node = &self.topology.nodes[start_pos];
        let end_node = &self.topology.nodes[end_pos.min(self.topology.nodes.len() - 1)];

        let prompt = format!(
            "Navigate from '{}' to '{}', explicitly noting each boundary you cross. \
             How many chunk boundaries are crossed?",
            start_node.label, end_node.label
        );

        // Count boundaries crossed
        let mut boundaries_crossed: usize = 0;
        for boundary in &self.boundaries {
            if boundary.position > start_pos && boundary.position <= end_pos {
                boundaries_crossed += 1;
            }
        }

        let correct_answer = boundaries_crossed.to_string();

        // Generate distractors
        let mut options = vec![
            correct_answer.clone(),
            (boundaries_crossed + 1).to_string(),
            boundaries_crossed.saturating_sub(1).to_string(),
            "0".to_string(),
        ];
        options.sort();
        options.dedup();

        Task {
            task_type: TaskType::Segment {
                start: start_node.label.clone(),
                count: (end_pos - start_pos),
                reverse: false,
            },
            prompt,
            correct_answer,
            options,
            difficulty: 0.6 + boundaries_crossed as f64 * 0.1,
            operation: OperationType::Segment(8, false),
        }
    }

    /// Generate a task that trains rapid boundary crossing
    pub fn generate_rapid_boundary_crossing_task(&self) -> Task {
        // Find the strongest boundary
        let strongest_boundary = self
            .boundaries
            .iter()
            .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap())
            .unwrap();

        // Create a task that requires crossing this boundary multiple times
        let start_idx = (strongest_boundary.position as i32 - 2).max(0) as usize;
        let oscillation_count = 3; // Number of times to cross back and forth

        let mut path = Vec::new();
        let mut current_idx = start_idx;

        for i in 0..oscillation_count * 2 {
            if i % 2 == 0 {
                // Forward across boundary
                for j in 0..4 {
                    if current_idx + j < self.topology.nodes.len() {
                        path.push(self.topology.nodes[current_idx + j].label.clone());
                    }
                }
                current_idx = (current_idx + 4).min(self.topology.nodes.len() - 1);
            } else {
                // Backward across boundary
                for j in 0..4 {
                    if current_idx >= j {
                        path.push(self.topology.nodes[current_idx - j].label.clone());
                    }
                }
                current_idx = current_idx.saturating_sub(4);
            }
        }

        let prompt = format!(
            "Starting from '{}', oscillate across the boundary at position {} three times. \
             What is the sequence of the first 8 items?",
            self.topology.nodes[start_idx].label, strongest_boundary.position
        );

        let correct_answer = path[..8.min(path.len())].join(", ");

        Task {
            task_type: TaskType::Segment {
                start: self.topology.nodes[start_idx].label.clone(),
                count: 8,
                reverse: false,
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.8 + strongest_boundary.strength * 0.1,
            operation: OperationType::Segment(8, false),
        }
    }

    fn generate_hierarchical_boundary_task(&self, level: usize) -> Task {
        let default_boundaries = vec![6, 13, 19];
        let boundaries = self
            .hierarchical_boundaries
            .get(&level)
            .unwrap_or(&default_boundaries);

        if boundaries.is_empty() {
            return self.generate_chunk_boundary_task();
        }

        let boundary_pos = boundaries[rand::random::<usize>() % boundaries.len()];
        let start_idx = boundary_pos.saturating_sub(2);
        let span_size = 5;

        let start_node = &self.topology.nodes[start_idx.min(self.topology.nodes.len() - span_size)];
        let segment = self
            .topology
            .get_segment(&start_node.label, span_size, false);

        let prompt = format!(
            "List {} items from '{}' (crosses level-{} hierarchical boundary)",
            span_size, start_node.label, level
        );

        let correct_answer = segment.join(", ");

        let difficulty = 0.3 + (level as f64 * 0.2).min(0.4);

        Task {
            task_type: TaskType::Segment {
                start: start_node.label.clone(),
                count: span_size,
                reverse: false,
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty,
            operation: OperationType::Segment(span_size, false),
        }
    }

    pub fn generate_integration_task(&self) -> Task {
        // Create a task that requires seamless integration across multiple boundaries
        let start_idx = rand::random::<usize>() % 10; // Start in first third
        let end_idx = 15 + (rand::random::<usize>() % 10); // End in last third

        let start = &self.topology.nodes[start_idx].label;
        let end = &self.topology.nodes[end_idx].label;

        // Count boundaries crossed
        let mut boundaries_crossed: usize = 0;
        for boundary in &self.boundaries {
            if boundary.position > start_idx && boundary.position < end_idx {
                boundaries_crossed += 1;
            }
        }

        let path = self
            .topology
            .shortest_path(start, end)
            .unwrap_or_else(|| vec!["No path".to_string()]);

        let prompt = format!(
            "Navigate from '{}' to '{}' (crosses {} chunk boundaries)",
            start, end, boundaries_crossed
        );

        let correct_answer = path.join(" → ");

        let difficulty = 0.5 + (boundaries_crossed as f64 * 0.15).min(0.4);

        Task {
            task_type: TaskType::ShortestPath {
                from: start.clone(),
                to: end.clone(),
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty,
            operation: OperationType::PairwiseOrder,
        }
    }

    pub fn measure_boundary_cost(&self, from: usize, to: usize) -> f64 {
        let mut cost = 0.0;

        let (start, end) = if from < to { (from, to) } else { (to, from) };

        for boundary in &self.boundaries {
            if boundary.position > start && boundary.position <= end {
                cost += boundary.strength;
            }
        }

        cost
    }

    pub fn generate_boundary_comparison_task(&self) -> Task {
        // Compare two paths: one crossing boundaries, one within chunk
        let boundary = &self.boundaries[rand::random::<usize>() % self.boundaries.len()];

        // Within-chunk path
        let within_start = boundary.position.saturating_sub(3);
        let within_end = boundary.position.saturating_sub(1);

        // Cross-boundary path
        let cross_start = boundary.position.saturating_sub(1);
        let cross_end = (boundary.position + 2).min(self.topology.nodes.len() - 1);

        let within_path = self.topology.get_segment(
            &self.topology.nodes[within_start].label,
            within_end - within_start + 1,
            false,
        );

        let cross_path = self.topology.get_segment(
            &self.topology.nodes[cross_start].label,
            cross_end - cross_start + 1,
            false,
        );

        let prompt = format!(
            "Which sequence is harder to recall: {} OR {}?",
            within_path.join("-"),
            cross_path.join("-")
        );

        let correct_answer = format!("{} (crosses boundary)", cross_path.join("-"));

        Task {
            task_type: TaskType::PairwiseOrder {
                a: within_path.join("-"),
                b: cross_path.join("-"),
            },
            prompt,
            correct_answer,
            options: vec![
                format!("{} (within chunk)", within_path.join("-")),
                format!("{} (crosses boundary)", cross_path.join("-")),
            ],
            difficulty: 0.5,
            operation: OperationType::PairwiseOrder,
        }
    }
}

// Hierarchical chunking for multi-level boundaries
pub struct HierarchicalChunker {
    topology: Topology,
    levels: Vec<ChunkLevel>,
}

#[derive(Debug, Clone)]
pub struct ChunkLevel {
    pub level: usize,
    pub chunk_size: usize,
    pub boundaries: Vec<usize>,
    pub labels: Vec<String>,
}

impl HierarchicalChunker {
    pub fn new(topology: Topology) -> Self {
        let mut levels = Vec::new();

        // Level 0: Individual items
        levels.push(ChunkLevel {
            level: 0,
            chunk_size: 1,
            boundaries: (0..topology.nodes.len()).collect(),
            labels: topology.nodes.iter().map(|n| n.label.clone()).collect(),
        });

        // Level 1: Triplets (ABC, DEF, etc.)
        let mut level1_boundaries = Vec::new();
        let mut level1_labels = Vec::new();
        for i in (0..topology.nodes.len()).step_by(3) {
            level1_boundaries.push(i);
            let chunk: Vec<String> = topology.nodes[i..std::cmp::min(i + 3, topology.nodes.len())]
                .iter()
                .map(|n| n.label.clone())
                .collect();
            level1_labels.push(chunk.join(""));
        }
        levels.push(ChunkLevel {
            level: 1,
            chunk_size: 3,
            boundaries: level1_boundaries,
            labels: level1_labels,
        });

        // Level 2: Sextets (ABCDEF, GHIJKL, etc.)
        let mut level2_boundaries = Vec::new();
        let mut level2_labels = Vec::new();
        for i in (0..topology.nodes.len()).step_by(6) {
            level2_boundaries.push(i);
            let chunk: Vec<String> = topology.nodes[i..std::cmp::min(i + 6, topology.nodes.len())]
                .iter()
                .map(|n| n.label.clone())
                .collect();
            level2_labels.push(chunk.join(""));
        }
        levels.push(ChunkLevel {
            level: 2,
            chunk_size: 6,
            boundaries: level2_boundaries,
            labels: level2_labels,
        });

        HierarchicalChunker { topology, levels }
    }

    pub fn generate_hierarchical_task(&self, from_level: usize, to_level: usize) -> Task {
        let from_chunks = &self.levels[from_level.min(self.levels.len() - 1)];
        let to_chunks = &self.levels[to_level.min(self.levels.len() - 1)];

        let chunk_idx = rand::random::<usize>() % from_chunks.labels.len();
        let chunk_label = &from_chunks.labels[chunk_idx];

        let prompt = if from_level < to_level {
            format!(
                "Break down '{}' into level-{} chunks",
                chunk_label, to_level
            )
        } else {
            format!(
                "Combine these into a level-{} chunk: {}",
                to_level, chunk_label
            )
        };

        let correct_answer = if from_level < to_level {
            // Breaking down
            let start_idx = from_chunks.boundaries[chunk_idx];
            let end_idx = if chunk_idx + 1 < from_chunks.boundaries.len() {
                from_chunks.boundaries[chunk_idx + 1]
            } else {
                self.topology.nodes.len()
            };

            let mut result = Vec::new();
            for &boundary in &to_chunks.boundaries {
                if boundary >= start_idx && boundary < end_idx {
                    let chunk_end = to_chunks
                        .boundaries
                        .iter()
                        .find(|&&b| b > boundary)
                        .copied()
                        .unwrap_or(end_idx)
                        .min(end_idx);

                    let sub_chunk: Vec<String> = self.topology.nodes[boundary..chunk_end]
                        .iter()
                        .map(|n| n.label.clone())
                        .collect();
                    result.push(sub_chunk.join(""));
                }
            }
            result.join(", ")
        } else {
            // Combining
            chunk_label.clone()
        };

        Task {
            task_type: TaskType::Segment {
                start: chunk_label.clone(),
                count: 1,
                reverse: false,
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.4 + (from_level as f64 - to_level as f64).abs() * 0.1,
            operation: OperationType::Segment(1, false),
        }
    }
}
