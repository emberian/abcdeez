use crate::core::learner::OperationType;
use crate::core::topology::Topology;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    PairwiseOrder {
        a: String,
        b: String,
    },
    Successor {
        item: String,
    },
    Predecessor {
        item: String,
    },
    KJump {
        start: String,
        k: i32,
    },
    Segment {
        start: String,
        count: usize,
        reverse: bool,
    },
    Index {
        item: String,
    },
    MissingItem {
        before: String,
        after: String,
    },
    ShortestDistance {
        from: String,
        to: String,
    },
    Comparability {
        a: String,
        b: String,
    },
    TopologicalSort {
        items: Vec<String>,
    },
    ShortestPath {
        from: String,
        to: String,
    },
    MinimalElements,
    MaximalElements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_type: TaskType,
    pub prompt: String,
    pub correct_answer: String,
    pub options: Vec<String>,
    pub difficulty: f64,
    pub operation: OperationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task: Task,
    pub user_answer: String,
    pub correct: bool,
    pub response_time_ms: u128,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct TaskGenerator {
    pub topology: Topology,
    rng: rand::rngs::StdRng,
}

impl std::fmt::Debug for TaskGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TaskGenerator")
            .field("topology", &self.topology)
            .field("rng", &"StdRng")
            .finish()
    }
}

impl TaskGenerator {
    pub fn new(topology: Topology) -> Self {
        Self::with_seed(topology, None)
    }

    pub fn with_seed(topology: Topology, seed: Option<u64>) -> Self {
        use rand::SeedableRng;
        let rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };
        TaskGenerator { topology, rng }
    }

    pub fn generate_task(&mut self, task_type: Option<TaskType>) -> Task {
        let task_type = task_type.unwrap_or_else(|| self.random_task_type());

        match task_type {
            TaskType::PairwiseOrder { a, b } => self.generate_pairwise_order(a, b),
            TaskType::Successor { item } => self.generate_successor(item),
            TaskType::Predecessor { item } => self.generate_predecessor(item),
            TaskType::KJump { start, k } => self.generate_k_jump(start, k),
            TaskType::Segment {
                start,
                count,
                reverse,
            } => self.generate_segment(start, count, reverse),
            TaskType::Index { item } => self.generate_index(item),
            TaskType::MissingItem { before, after } => self.generate_missing_item(before, after),
            TaskType::ShortestDistance { from, to } => self.generate_shortest_distance(from, to),
            TaskType::Comparability { a, b } => self.generate_comparability(a, b),
            TaskType::TopologicalSort { items } => self.generate_topological_sort(items),
            TaskType::ShortestPath { from, to } => self.generate_shortest_path(from, to),
            TaskType::MinimalElements => self.generate_minimal_elements(),
            TaskType::MaximalElements => self.generate_maximal_elements(),
        }
    }

    fn random_task_type(&mut self) -> TaskType {
        let nodes = &self.topology.nodes;
        let n = nodes.len();

        let task_types = vec![
            TaskType::PairwiseOrder {
                a: nodes[self.rng.gen_range(0..n)].label.clone(),
                b: nodes[self.rng.gen_range(0..n)].label.clone(),
            },
            TaskType::Successor {
                item: nodes[self.rng.gen_range(0..n - 1)].label.clone(),
            },
            TaskType::Predecessor {
                item: nodes[self.rng.gen_range(1..n)].label.clone(),
            },
            TaskType::KJump {
                start: nodes[self.rng.gen_range(0..n)].label.clone(),
                k: self.rng.gen_range(1..4),
            },
            TaskType::Segment {
                start: nodes[self.rng.gen_range(0..n)].label.clone(),
                count: self.rng.gen_range(2..5),
                reverse: self.rng.gen_bool(0.5),
            },
            TaskType::Index {
                item: nodes[self.rng.gen_range(0..n)].label.clone(),
            },
        ];

        task_types.into_iter().choose(&mut self.rng).unwrap()
    }

    fn generate_pairwise_order(&self, a: String, b: String) -> Task {
        let prompt = format!("Does '{}' come before '{}'?", a, b);
        let correct_answer = match self.topology.is_before(&a, &b) {
            Some(true) => "Yes".to_string(),
            Some(false) => "No".to_string(),
            None => "Not applicable for cyclic".to_string(),
        };

        let distance = self
            .topology
            .get_node_by_label(&a)
            .and_then(|na| {
                self.topology
                    .get_node_by_label(&b)
                    .and_then(|nb| self.topology.get_distance(&na.id, &nb.id))
            })
            .unwrap_or(0);

        Task {
            task_type: TaskType::PairwiseOrder { a, b },
            prompt,
            correct_answer: correct_answer.clone(),
            options: vec!["Yes".to_string(), "No".to_string()],
            difficulty: (distance as f64 / 10.0).min(1.0),
            operation: OperationType::PairwiseOrder,
        }
    }

    fn generate_successor(&mut self, item: String) -> Task {
        let prompt = format!("What comes after '{}'?", item);
        let node = self.topology.get_node_by_label(&item).unwrap();
        let successor_id = self.topology.get_successor(&node.id);

        let correct_answer = successor_id
            .and_then(|id| self.topology.get_node_by_id(&id))
            .map(|n| n.label.clone())
            .unwrap_or("None".to_string());

        let mut options = self.generate_options(&correct_answer, 4);
        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::Successor { item },
            prompt,
            correct_answer: correct_answer.clone(),
            options,
            difficulty: 0.3,
            operation: OperationType::Successor,
        }
    }

    fn generate_predecessor(&mut self, item: String) -> Task {
        let prompt = format!("What comes before '{}'?", item);
        let node = self.topology.get_node_by_label(&item).unwrap();
        let predecessor_id = self.topology.get_predecessor(&node.id);

        let correct_answer = predecessor_id
            .and_then(|id| self.topology.get_node_by_id(&id))
            .map(|n| n.label.clone())
            .unwrap_or("None".to_string());

        let mut options = self.generate_options(&correct_answer, 4);
        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::Predecessor { item },
            prompt,
            correct_answer: correct_answer.clone(),
            options,
            difficulty: 0.4,
            operation: OperationType::Predecessor,
        }
    }

    fn generate_k_jump(&mut self, start: String, k: i32) -> Task {
        let direction = if k > 0 { "after" } else { "before" };
        let prompt = format!("What is {} positions {} '{}'?", k.abs(), direction, start);

        let node = self.topology.get_node_by_label(&start).unwrap();
        let target_id = self.topology.get_k_jump(&node.id, k);

        let correct_answer = target_id
            .and_then(|id| self.topology.get_node_by_id(&id))
            .map(|n| n.label.clone())
            .unwrap_or("Out of bounds".to_string());

        let mut options = self.generate_options(&correct_answer, 4);
        if !options.contains(&"Out of bounds".to_string()) {
            options.push("Out of bounds".to_string());
        }
        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::KJump { start, k },
            prompt,
            correct_answer: correct_answer.clone(),
            options,
            difficulty: 0.3 + (k.abs() as f64 * 0.1).min(0.7),
            operation: OperationType::KJump(k),
        }
    }

    fn generate_segment(&mut self, start: String, count: usize, reverse: bool) -> Task {
        // Enhanced segment recital task based on PAPER.md
        // Can ask for items before, after, or from a starting point
        // Use the internal RNG instead of thread_rng
        let recital_type = self.rng.gen_range(0..3);

        let (prompt, correct_answer, difficulty_bonus) = match recital_type {
            0 => {
                // Standard: from a starting point
                let direction = if reverse { "reverse" } else { "forward" };
                let prompt = format!(
                    "List {} items starting from '{}' in {} order:",
                    count, start, direction
                );
                let segment = self.topology.get_segment(&start, count, reverse);
                (prompt, segment.join(", "), 0.0)
            }
            1 if count <= 4 => {
                // Preceding items (like "4 letters preceding P in reverse")
                let prompt = if reverse {
                    format!(
                        "Recite the {} items preceding '{}' in reverse order:",
                        count, start
                    )
                } else {
                    format!("List the {} items that come before '{}':", count, start)
                };
                // Get items before start
                if let Some(node) = self.topology.get_node_by_label(&start) {
                    let idx = self.topology.node_map[&node.id];
                    let start_idx = idx.saturating_sub(count);
                    let mut items = Vec::new();
                    for i in start_idx..idx {
                        if i < self.topology.nodes.len() {
                            items.push(self.topology.nodes[i].label.clone());
                        }
                    }
                    if reverse {
                        items.reverse();
                    }
                    (prompt, items.join(", "), 0.1)
                } else {
                    let prompt = format!(
                        "List {} items starting from '{}' in forward order:",
                        count, start
                    );
                    let segment = self.topology.get_segment(&start, count, false);
                    (prompt, segment.join(", "), 0.0)
                }
            }
            _ => {
                // Standard fallback for larger segments
                let direction = if reverse { "reverse" } else { "forward" };
                let prompt = format!(
                    "List {} items starting from '{}' in {} order:",
                    count, start, direction
                );
                let segment = self.topology.get_segment(&start, count, reverse);
                (prompt, segment.join(", "), 0.0)
            }
        };

        Task {
            task_type: TaskType::Segment {
                start,
                count,
                reverse,
            },
            prompt,
            correct_answer: correct_answer.clone(),
            options: vec![],
            difficulty: 0.3
                + (count as f64 * 0.1)
                + if reverse { 0.2 } else { 0.0 }
                + difficulty_bonus,
            operation: OperationType::Segment(count, reverse),
        }
    }

    fn generate_index(&mut self, item: String) -> Task {
        let prompt = format!("What position (1-based) is '{}'?", item);
        let node = self.topology.get_node_by_label(&item).unwrap();
        let position = self.topology.node_map[&node.id] + 1;
        let correct_answer = position.to_string();

        let mut options: Vec<String> = vec![
            correct_answer.clone(),
            (position.saturating_sub(1)).to_string(),
            (position + 1).to_string(),
            (position + 2).to_string(),
        ];
        options.dedup();
        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::Index { item },
            prompt,
            correct_answer: correct_answer.clone(),
            options,
            difficulty: 0.4,
            operation: OperationType::Index,
        }
    }

    fn generate_missing_item(&mut self, before: String, after: String) -> Task {
        let prompt = format!("What comes between '{}' and '{}'?", before, after);

        let before_node = self.topology.get_node_by_label(&before);
        let after_node = self.topology.get_node_by_label(&after);

        let correct_answer = if let (Some(bn), Some(an)) = (before_node, after_node) {
            let before_idx = self.topology.node_map[&bn.id];
            let after_idx = self.topology.node_map[&an.id];

            if after_idx == before_idx + 2 {
                self.topology.nodes[before_idx + 1].label.clone()
            } else {
                "Not adjacent".to_string()
            }
        } else {
            "Invalid items".to_string()
        };

        let mut options = self.generate_options(&correct_answer, 3);
        options.push("Not adjacent".to_string());
        options.dedup();
        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::MissingItem { before, after },
            prompt,
            correct_answer: correct_answer.clone(),
            options,
            difficulty: 0.4,
            operation: OperationType::Successor,
        }
    }

    fn generate_shortest_distance(&mut self, from: String, to: String) -> Task {
        let prompt = format!("What is the shortest distance from '{}' to '{}'?", from, to);

        let from_node = self.topology.get_node_by_label(&from);
        let to_node = self.topology.get_node_by_label(&to);

        let distance = if let (Some(fn_), Some(tn)) = (from_node, to_node) {
            self.topology.get_distance(&fn_.id, &tn.id).unwrap_or(0)
        } else {
            0
        };

        let correct_answer = distance.to_string();

        let mut options = vec![
            correct_answer.clone(),
            (distance.saturating_sub(1)).to_string(),
            (distance + 1).to_string(),
            (distance + 2).to_string(),
        ];
        options.dedup();
        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::ShortestDistance { from, to },
            prompt,
            correct_answer,
            options,
            difficulty: (distance as f64 / 10.0).min(1.0),
            operation: OperationType::PairwiseOrder,
        }
    }

    fn generate_comparability(&self, a: String, b: String) -> Task {
        let prompt = format!(
            "Are '{}' and '{}' comparable (one must come before the other)?",
            a, b
        );

        let comparable = self.topology.are_comparable(&a, &b).unwrap_or(true);
        let correct_answer = if comparable { "Yes" } else { "No" }.to_string();

        Task {
            task_type: TaskType::Comparability { a, b },
            prompt,
            correct_answer: correct_answer.clone(),
            options: vec!["Yes".to_string(), "No".to_string()],
            difficulty: 0.6,
            operation: OperationType::PairwiseOrder,
        }
    }

    fn generate_topological_sort(&self, items: Vec<String>) -> Task {
        let prompt = format!("Arrange these items in a valid order: {:?}", items);

        let correct_answer = if let Some(sorted) = self.topology.get_topological_sort() {
            let filtered: Vec<String> = sorted.into_iter().filter(|s| items.contains(s)).collect();
            filtered.join(", ")
        } else {
            "Not applicable".to_string()
        };

        Task {
            task_type: TaskType::TopologicalSort { items },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.8,
            operation: OperationType::PairwiseOrder,
        }
    }

    fn generate_shortest_path(&self, from: String, to: String) -> Task {
        let prompt = format!("What is the shortest path from '{}' to '{}'?", from, to);

        let path = self.topology.shortest_path(&from, &to);
        let correct_answer = path
            .map(|p| p.join(" -> "))
            .unwrap_or_else(|| "No path".to_string());

        Task {
            task_type: TaskType::ShortestPath { from, to },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.7,
            operation: OperationType::PairwiseOrder,
        }
    }

    fn generate_minimal_elements(&self) -> Task {
        let prompt = "Which elements have no prerequisites?".to_string();

        let mut minimal = Vec::new();
        for node in &self.topology.nodes {
            let has_incoming = self.topology.edges.iter().any(|e| e.to == node.id);
            if !has_incoming {
                minimal.push(node.label.clone());
            }
        }

        let correct_answer = if minimal.is_empty() {
            "None".to_string()
        } else {
            minimal.join(", ")
        };

        Task {
            task_type: TaskType::MinimalElements,
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.5,
            operation: OperationType::PairwiseOrder,
        }
    }

    fn generate_maximal_elements(&self) -> Task {
        let prompt = "Which elements have no dependent tasks?".to_string();

        let mut maximal = Vec::new();
        for node in &self.topology.nodes {
            let has_outgoing = self.topology.edges.iter().any(|e| e.from == node.id);
            if !has_outgoing {
                maximal.push(node.label.clone());
            }
        }

        let correct_answer = if maximal.is_empty() {
            "None".to_string()
        } else {
            maximal.join(", ")
        };

        Task {
            task_type: TaskType::MaximalElements,
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.5,
            operation: OperationType::PairwiseOrder,
        }
    }

    fn generate_options(&mut self, correct: &str, count: usize) -> Vec<String> {
        let mut options = vec![correct.to_string()];
        let all_labels: Vec<String> = self
            .topology
            .nodes
            .iter()
            .map(|n| n.label.clone())
            .filter(|l| l != correct)
            .collect();

        // Use the internal RNG instead of thread_rng
        for _ in 1..count {
            if let Some(label) = all_labels.choose(&mut self.rng) {
                if !options.contains(label) {
                    options.push(label.clone());
                }
            }
        }

        options
    }
}

pub struct TaskSession {
    pub generator: TaskGenerator,
    pub current_task: Option<Task>,
    pub task_start_time: Option<Instant>,
    pub history: Vec<TaskResponse>,
}

impl TaskSession {
    pub fn new(topology: Topology) -> Self {
        TaskSession {
            generator: TaskGenerator::new(topology),
            current_task: None,
            task_start_time: None,
            history: Vec::new(),
        }
    }

    pub fn start_task(&mut self, task_type: Option<TaskType>) -> &Task {
        let task = self.generator.generate_task(task_type);
        self.current_task = Some(task);
        self.task_start_time = Some(Instant::now());
        self.current_task.as_ref().unwrap()
    }

    pub fn submit_answer(&mut self, answer: String) -> TaskResponse {
        let task = self.current_task.take().expect("No active task");
        let start_time = self.task_start_time.take().expect("No start time");

        let response_time_ms = start_time.elapsed().as_millis();
        let correct = answer.trim().eq_ignore_ascii_case(&task.correct_answer);

        let response = TaskResponse {
            task: task.clone(),
            user_answer: answer,
            correct,
            response_time_ms,
            timestamp: chrono::Utc::now(),
        };

        self.history.push(response.clone());
        response
    }

    pub fn get_statistics(&self) -> SessionStatistics {
        let total_tasks = self.history.len();
        let correct_tasks = self.history.iter().filter(|r| r.correct).count();
        let avg_response_time = if total_tasks > 0 {
            self.history
                .iter()
                .map(|r| r.response_time_ms)
                .sum::<u128>()
                / total_tasks as u128
        } else {
            0
        };

        let mut task_type_stats = std::collections::HashMap::new();
        for response in &self.history {
            let key = format!("{:?}", response.task.operation);
            let entry = task_type_stats.entry(key).or_insert((0, 0));
            entry.0 += 1;
            if response.correct {
                entry.1 += 1;
            }
        }

        SessionStatistics {
            total_tasks,
            correct_tasks,
            accuracy: if total_tasks > 0 {
                correct_tasks as f64 / total_tasks as f64
            } else {
                0.0
            },
            avg_response_time_ms: avg_response_time,
            task_type_performance: task_type_stats,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionStatistics {
    pub total_tasks: usize,
    pub correct_tasks: usize,
    pub accuracy: f64,
    pub avg_response_time_ms: u128,
    pub task_type_performance: std::collections::HashMap<String, (usize, usize)>,
}
