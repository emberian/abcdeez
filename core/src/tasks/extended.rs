use crate::core::learner::OperationType;
use crate::tasks::types::{Task, TaskType};
use crate::core::topology::Topology;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Extended task types to complete paper specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtendedTaskType {
    // Missing from paper section 4.1
    BetweenQuery {
        a: String,
        b: String,
        c: String,
    },
    BoundaryBridging {
        start: String,
        count: usize,
        boundaries: Vec<usize>,
    },
    ReverseNTreadmill {
        start: String,
        n: usize,
        steps: usize,
    },

    // Missing from paper section 4.2
    DirectionalComparison {
        a: String,
        b: String,
        backward: bool,
    },

    // Missing from paper section 4.3
    InsertionAdaptation {
        item: String,
        after: String,
        before: String,
    },
    LinearExtensionGeneration {
        partial_order: Vec<(String, String)>,
    },

    // Missing from paper section 4.4
    NextStepPrediction {
        current: String,
        goal: String,
    },
    LandmarkNavigation {
        start: String,
        end: String,
        landmark: String,
    },
    MacroDiscovery {
        sequence: Vec<String>,
    },

    // Missing from paper section 4.5
    SemanticFilter {
        category: String,
        position: usize,
    },
    ProjectionSwitch {
        item: String,
        from_view: String,
        to_view: String,
    },
    IsomorphicTransfer {
        source_domain: String,
        target_domain: String,
        task: Box<Task>,
    },
}

pub struct ExtendedTaskGenerator {
    topology: Topology,
    semantic_attributes: HashMap<String, Vec<String>>,
    macros: HashMap<String, Vec<String>>,
    rng: rand::rngs::StdRng,
}

impl ExtendedTaskGenerator {
    pub fn new(topology: Topology) -> Self {
        Self::with_seed(topology, None)
    }

    pub fn with_seed(topology: Topology, seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };

        let mut semantic_attributes = HashMap::new();

        // For alphabet, add vowel/consonant categories
        if topology.nodes.len() == 26 {
            semantic_attributes.insert(
                "vowel".to_string(),
                vec!["A", "E", "I", "O", "U"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            );

            let consonants: Vec<String> = (b'A'..=b'Z')
                .map(|c| (c as char).to_string())
                .filter(|c| !["A", "E", "I", "O", "U"].contains(&c.as_str()))
                .collect();
            semantic_attributes.insert("consonant".to_string(), consonants);
        }

        // Common macros for navigation
        let mut macros = HashMap::new();
        macros.insert(
            "consecutive_forward".to_string(),
            vec!["A", "B", "C"].iter().map(|s| s.to_string()).collect(),
        );
        macros.insert(
            "skip_pattern".to_string(),
            vec!["A", "C", "E", "G"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );

        ExtendedTaskGenerator {
            topology,
            semantic_attributes,
            macros,
            rng,
        }
    }

    /// Generate a Reverse-N Treadmill Drill task
    /// The learner must recite N items backward, then continue for 'steps' iterations
    pub fn generate_reverse_n_treadmill(&mut self, start: String, n: usize, steps: usize) -> Task {
        let prompt = format!(
            "Starting from '{}', go back {} items, then continue backwards for {} more steps. What is the final item?",
            start, n, steps
        );

        let mut current = start.clone();
        let mut path = vec![current.clone()];

        // First go back N items
        for _ in 0..n {
            if let Some(pred) = self.topology.get_predecessor(&current) {
                if let Some(node) = self.topology.get_node_by_id(&pred) {
                    current = node.label.clone();
                    path.push(current.clone());
                }
            } else {
                break;
            }
        }

        // Then continue for 'steps' more iterations
        for _ in 0..steps {
            if let Some(pred) = self.topology.get_predecessor(&current) {
                if let Some(node) = self.topology.get_node_by_id(&pred) {
                    current = node.label.clone();
                    path.push(current.clone());
                }
            } else {
                // Handle wraparound for cyclic topologies
                if matches!(
                    self.topology.topology_type,
                    crate::core::topology::TopologyType::Cyclic
                ) {
                    // Wrap to the end
                    if let Some(last_node) = self.topology.nodes.last() {
                        current = last_node.label.clone();
                        path.push(current.clone());
                    }
                } else {
                    break;
                }
            }
        }

        let correct_answer = current.clone();

        // Generate distractors based on common errors
        let mut options = vec![correct_answer.clone()];

        // Error 1: Off by one (stopped one early)
        if path.len() > 1 {
            options.push(path[path.len() - 2].clone());
        }

        // Error 2: Went forward instead of backward
        let mut forward_current = start.clone();
        for _ in 0..(n + steps) {
            if let Some(succ) = self.topology.get_successor(&forward_current) {
                if let Some(node) = self.topology.get_node_by_id(&succ) {
                    forward_current = node.label.clone();
                }
            }
        }
        if !options.contains(&forward_current) {
            options.push(forward_current);
        }

        // Error 3: Confusion about total steps
        if path.len() > n && n > 0 {
            options.push(path[n].clone());
        }

        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::Segment {
                start: start.clone(),
                count: n + steps,
                reverse: true,
            },
            prompt,
            correct_answer,
            options,
            difficulty: 0.7 + (n as f64 + steps as f64) * 0.02, // Harder with more steps
            operation: OperationType::Segment(n + steps, true),
        }
    }

    pub fn generate_between_query(&self, a: String, b: String, c: String) -> Task {
        let prompt = format!("Is '{}' between '{}' and '{}'?", b, a, c);

        let a_node = self.topology.get_node_by_label(&a);
        let b_node = self.topology.get_node_by_label(&b);
        let c_node = self.topology.get_node_by_label(&c);

        let correct_answer = if let (Some(an), Some(bn), Some(cn)) = (a_node, b_node, c_node) {
            match self.topology.topology_type {
                crate::core::topology::TopologyType::Linear => {
                    let between = (an.position < bn.position && bn.position < cn.position)
                        || (cn.position < bn.position && bn.position < an.position);
                    if between { "Yes" } else { "No" }.to_string()
                }
                crate::core::topology::TopologyType::Cyclic => {
                    // For cyclic, check both circular directions
                    let forward = self.check_cyclic_between(&an.id, &bn.id, &cn.id);
                    let backward = self.check_cyclic_between(&cn.id, &bn.id, &an.id);
                    if forward || backward { "Yes" } else { "No" }.to_string()
                }
                _ => "Not applicable".to_string(),
            }
        } else {
            "Invalid items".to_string()
        };

        Task {
            task_type: TaskType::PairwiseOrder {
                a: b.clone(),
                b: c.clone(),
            }, // Simplified mapping
            prompt,
            correct_answer: correct_answer.clone(),
            options: vec!["Yes".to_string(), "No".to_string()],
            difficulty: 0.5,
            operation: OperationType::PairwiseOrder,
        }
    }

    fn check_cyclic_between(&self, start: &str, middle: &str, end: &str) -> bool {
        let mut current = start.to_string();
        let mut found_middle = false;
        let max_steps = self.topology.nodes.len();

        for _ in 0..max_steps {
            if current == middle {
                found_middle = true;
            }
            if found_middle && current == end {
                return true;
            }

            if let Some(next) = self.topology.get_successor(&current) {
                current = next;
            } else {
                break;
            }
        }
        false
    }

    pub fn generate_boundary_bridging(
        &self,
        start: String,
        count: usize,
        boundaries: Vec<usize>,
    ) -> Task {
        let prompt = format!(
            "List {} items starting from '{}', crossing chunk boundaries at positions {:?}",
            count, start, boundaries
        );

        let segment = self.topology.get_segment(&start, count, false);
        let correct_answer = segment.join(", ");

        // Calculate difficulty based on boundary crossings
        let start_node = self.topology.get_node_by_label(&start);
        let mut boundary_crossings = 0;
        if let Some(node) = start_node {
            let start_idx = self.topology.node_map[&node.id];
            for boundary in &boundaries {
                if *boundary >= start_idx && *boundary < start_idx + count {
                    boundary_crossings += 1;
                }
            }
        }

        let difficulty = 0.3 + (0.2 * boundary_crossings as f64).min(0.7);

        Task {
            task_type: TaskType::Segment {
                start,
                count,
                reverse: false,
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty,
            operation: OperationType::Segment(count, false),
        }
    }

    pub fn generate_directional_comparison(&self, a: String, b: String, backward: bool) -> Task {
        let direction = if backward { "backward" } else { "forward" };
        let prompt = format!("Moving {}, does '{}' come before '{}'?", direction, a, b);

        let correct_answer = match self.topology.topology_type {
            crate::core::topology::TopologyType::Cyclic => {
                // In cyclic, direction affects the path
                let path = if backward {
                    self.find_backward_path(&a, &b)
                } else {
                    self.find_forward_path(&a, &b)
                };
                if path { "Yes" } else { "No" }.to_string()
            }
            _ => {
                // Linear doesn't change with direction
                if let Some(before) = self.topology.is_before(&a, &b) {
                    if before { "Yes" } else { "No" }.to_string()
                } else {
                    "Not applicable".to_string()
                }
            }
        };

        Task {
            task_type: TaskType::PairwiseOrder { a, b },
            prompt,
            correct_answer: correct_answer.clone(),
            options: vec!["Yes".to_string(), "No".to_string()],
            difficulty: 0.5,
            operation: OperationType::PairwiseOrder,
        }
    }

    fn find_forward_path(&self, from: &str, to: &str) -> bool {
        let mut current = from.to_string();
        for _ in 0..self.topology.nodes.len() {
            if current == to {
                return true;
            }
            if let Some(next) = self.topology.get_successor(&current) {
                if let Some(node) = self.topology.get_node_by_id(&next) {
                    current = node.label.clone();
                }
            } else {
                break;
            }
        }
        false
    }

    fn find_backward_path(&self, from: &str, to: &str) -> bool {
        let mut current = from.to_string();
        for _ in 0..self.topology.nodes.len() {
            if current == to {
                return true;
            }
            if let Some(prev) = self.topology.get_predecessor(&current) {
                if let Some(node) = self.topology.get_node_by_id(&prev) {
                    current = node.label.clone();
                }
            } else {
                break;
            }
        }
        false
    }

    /// Generate all valid linear extensions (topological sorts) of a partial order
    pub fn generate_linear_extension_task(&mut self, partial_order: Vec<(String, String)>) -> Task {
        let prompt = format!(
            "Given the partial order constraints {:?}, which of the following is a valid linear extension?",
            partial_order
        );

        // Build adjacency list for topological sort
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut all_nodes: HashSet<String> = HashSet::new();

        for (before, after) in &partial_order {
            adj_list
                .entry(before.clone())
                .or_insert_with(Vec::new)
                .push(after.clone());
            *in_degree.entry(after.clone()).or_insert(0) += 1;
            in_degree.entry(before.clone()).or_insert(0);
            all_nodes.insert(before.clone());
            all_nodes.insert(after.clone());
        }

        // Generate one valid linear extension using Kahn's algorithm
        let valid_extension = self.kahns_topological_sort(&adj_list, &in_degree, &all_nodes);

        // Generate invalid options by violating constraints
        let mut options = vec![valid_extension.join(", ")];

        // Invalid option 1: Reverse a constraint
        if !partial_order.is_empty() {
            let (before, after) = &partial_order[0];
            let mut invalid = valid_extension.clone();
            if let (Some(pos1), Some(pos2)) = (
                invalid.iter().position(|x| x == before),
                invalid.iter().position(|x| x == after),
            ) {
                if pos1 < pos2 {
                    invalid.swap(pos1, pos2);
                    options.push(invalid.join(", "));
                }
            }
        }

        // Invalid option 2: Random permutation
        let mut random_perm: Vec<String> = all_nodes.iter().cloned().collect();
        random_perm.shuffle(&mut self.rng);
        options.push(random_perm.join(", "));

        // Invalid option 3: Reverse the valid extension
        let mut reversed = valid_extension.clone();
        reversed.reverse();
        options.push(reversed.join(", "));

        options.shuffle(&mut self.rng);
        let correct_answer = valid_extension.join(", ");

        Task {
            task_type: TaskType::MissingItem {
                before: partial_order
                    .first()
                    .map(|p| p.0.clone())
                    .unwrap_or_default(),
                after: partial_order
                    .last()
                    .map(|p| p.1.clone())
                    .unwrap_or_default(),
            },
            prompt,
            correct_answer,
            options,
            difficulty: 0.7 + partial_order.len() as f64 * 0.05,
            operation: OperationType::PairwiseOrder,
        }
    }

    fn kahns_topological_sort(
        &self,
        adj_list: &HashMap<String, Vec<String>>,
        in_degree: &HashMap<String, usize>,
        all_nodes: &HashSet<String>,
    ) -> Vec<String> {
        use std::collections::VecDeque;

        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        let mut in_degree_copy = in_degree.clone();

        // Find all nodes with in-degree 0
        for node in all_nodes {
            if *in_degree_copy.get(node).unwrap_or(&0) == 0 {
                queue.push_back(node.clone());
            }
        }

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            if let Some(neighbors) = adj_list.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree_copy.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        // If we couldn't sort all nodes, there's a cycle
        if result.len() != all_nodes.len() {
            // Return nodes in any order as fallback
            all_nodes.iter().cloned().collect()
        } else {
            result
        }
    }

    pub fn generate_insertion_adaptation(
        &mut self,
        item: String,
        after: String,
        before: String,
    ) -> Task {
        let prompt = format!(
            "If '{}' must come after '{}' but before '{}', where in the sequence should it be inserted?",
            item, after, before
        );

        let after_node = self.topology.get_node_by_label(&after);
        let before_node = self.topology.get_node_by_label(&before);

        let (correct_answer, options) = if let (Some(an), Some(bn)) = (after_node, before_node) {
            let after_idx = self.topology.node_map[&an.id];
            let before_idx = self.topology.node_map[&bn.id];

            if before_idx > after_idx {
                // Valid ordering constraint
                let mut valid_positions = Vec::new();

                // Collect all valid insertion positions
                for i in after_idx + 1..before_idx {
                    if i < self.topology.nodes.len() {
                        let position = if i == after_idx + 1 && i == before_idx - 1 {
                            format!("Between {} and {}", after, before)
                        } else {
                            format!("After {}", self.topology.nodes[i - 1].label)
                        };
                        valid_positions.push(position);
                    }
                }

                // Generate answer and options
                let answer = if valid_positions.len() == 1 {
                    valid_positions[0].clone()
                } else if valid_positions.is_empty() {
                    "No valid position".to_string()
                } else {
                    // For multiple valid positions, list the range
                    format!(
                        "Any position from {} to {}",
                        valid_positions.first().unwrap(),
                        valid_positions.last().unwrap()
                    )
                };

                // Create options including distractors
                let mut opts = vec![answer.clone()];

                // Add "before after" as a distractor
                if after_idx > 0 {
                    opts.push(format!("Before {}", after));
                }

                // Add "after before" as a distractor
                if before_idx < self.topology.nodes.len() - 1 {
                    opts.push(format!("After {}", before));
                }

                // Add "no valid position" if not already there
                if !opts.contains(&"No valid position".to_string()) {
                    opts.push("No valid position".to_string());
                }

                opts.shuffle(&mut self.rng);
                (answer, opts)
            } else {
                // Invalid ordering (before comes before after)
                let answer = "No valid position (constraints conflict)".to_string();
                let opts = vec![
                    answer.clone(),
                    format!("After {}", after),
                    format!("Before {}", before),
                    format!("Between {} and {}", after, before),
                ];
                (answer, opts)
            }
        } else {
            let answer = "Invalid constraints".to_string();
            let opts = vec![answer.clone()];
            (answer, opts)
        };

        Task {
            task_type: TaskType::MissingItem {
                before: after.clone(),
                after: before.clone(),
            },
            prompt,
            correct_answer,
            options,
            difficulty: 0.8,
            operation: OperationType::PairwiseOrder,
        }
    }

    pub fn generate_next_step_prediction(&mut self, current: String, goal: String) -> Task {
        let prompt = format!(
            "You are at '{}'. To reach '{}', what should be your next step?",
            current, goal
        );

        let path = self.topology.shortest_path(&current, &goal);
        let correct_answer = if let Some(p) = path {
            if p.len() > 1 {
                p[1].clone()
            } else {
                "Already at goal".to_string()
            }
        } else {
            "No path available".to_string()
        };

        // Generate plausible alternatives
        let mut options = vec![correct_answer.clone()];
        if let Some(node) = self.topology.get_node_by_label(&current) {
            // Add neighbors
            if let Some(succ) = self.topology.get_successor(&node.id) {
                if let Some(n) = self.topology.get_node_by_id(&succ) {
                    if !options.contains(&n.label) {
                        options.push(n.label.clone());
                    }
                }
            }
            if let Some(pred) = self.topology.get_predecessor(&node.id) {
                if let Some(n) = self.topology.get_node_by_id(&pred) {
                    if !options.contains(&n.label) {
                        options.push(n.label.clone());
                    }
                }
            }
        }

        // Add some random options
        for node in self.topology.nodes.iter().take(5) {
            if !options.contains(&node.label) && node.label != current {
                options.push(node.label.clone());
                if options.len() >= 4 {
                    break;
                }
            }
        }

        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::ShortestPath {
                from: current,
                to: goal,
            },
            prompt,
            correct_answer,
            options,
            difficulty: 0.6,
            operation: OperationType::PairwiseOrder,
        }
    }

    pub fn generate_landmark_navigation(
        &self,
        start: String,
        end: String,
        landmark: String,
    ) -> Task {
        let prompt = format!(
            "To get from '{}' to '{}', is it efficient to go via '{}'?",
            start, end, landmark
        );

        let direct_path = self.topology.shortest_path(&start, &end);
        let via_landmark = if let (Some(p1), Some(p2)) = (
            self.topology.shortest_path(&start, &landmark),
            self.topology.shortest_path(&landmark, &end),
        ) {
            Some(p1.len() + p2.len() - 1) // -1 because landmark counted twice
        } else {
            None
        };

        let correct_answer = match (direct_path, via_landmark) {
            (Some(direct), Some(via)) => {
                if via <= direct.len() + 1 {
                    // Allow small detour
                    "Yes"
                } else {
                    "No"
                }
                .to_string()
            }
            _ => "Cannot determine".to_string(),
        };

        Task {
            task_type: TaskType::ShortestPath {
                from: start,
                to: end,
            },
            prompt,
            correct_answer: correct_answer.clone(),
            options: vec![
                "Yes".to_string(),
                "No".to_string(),
                "Cannot determine".to_string(),
            ],
            difficulty: 0.7,
            operation: OperationType::PairwiseOrder,
        }
    }

    pub fn generate_macro_discovery(&self, sequence: Vec<String>) -> Task {
        let prompt = format!(
            "What pattern or macro does this sequence represent: {:?}?",
            sequence
        );

        let pattern = self.identify_pattern(&sequence);

        // Check if sequence matches any predefined macros
        let mut options = vec![];
        for (macro_name, macro_sequence) in &self.macros {
            if macro_sequence == &sequence {
                options.push(format!("Macro: {}", macro_name));
            }
        }

        // Add standard pattern options
        options.extend(vec![
            "Consecutive forward".to_string(),
            "Consecutive backward".to_string(),
            "Skip pattern".to_string(),
            "Chunk boundary crossing".to_string(),
            "Random sequence".to_string(),
        ]);

        // Limit to 5 options
        options.truncate(5);

        Task {
            task_type: TaskType::Segment {
                start: sequence.first().unwrap_or(&"".to_string()).clone(),
                count: sequence.len(),
                reverse: false,
            },
            prompt,
            correct_answer: pattern.clone(),
            options,
            difficulty: 0.6,
            operation: OperationType::Segment(sequence.len(), false),
        }
    }

    fn identify_pattern(&self, sequence: &[String]) -> String {
        if sequence.len() < 2 {
            return "Too short to identify".to_string();
        }

        // Check consecutive forward
        let mut consecutive_forward = true;
        for i in 0..sequence.len() - 1 {
            if let (Some(curr), Some(next)) = (
                self.topology.get_node_by_label(&sequence[i]),
                self.topology.get_node_by_label(&sequence[i + 1]),
            ) {
                if self.topology.get_successor(&curr.id) != Some(next.id.clone()) {
                    consecutive_forward = false;
                    break;
                }
            }
        }
        if consecutive_forward {
            return "Consecutive forward".to_string();
        }

        // Check consecutive backward
        let mut consecutive_backward = true;
        for i in 0..sequence.len() - 1 {
            if let (Some(curr), Some(next)) = (
                self.topology.get_node_by_label(&sequence[i]),
                self.topology.get_node_by_label(&sequence[i + 1]),
            ) {
                if self.topology.get_predecessor(&curr.id) != Some(next.id.clone()) {
                    consecutive_backward = false;
                    break;
                }
            }
        }
        if consecutive_backward {
            return "Consecutive backward".to_string();
        }

        // Check skip pattern
        let mut skip_distances = Vec::new();
        for i in 0..sequence.len() - 1 {
            if let Some(dist) = self.topology.get_distance(
                &self.topology.get_node_by_label(&sequence[i]).unwrap().id,
                &self
                    .topology
                    .get_node_by_label(&sequence[i + 1])
                    .unwrap()
                    .id,
            ) {
                skip_distances.push(dist);
            }
        }

        if !skip_distances.is_empty()
            && skip_distances
                .iter()
                .all(|&d| d == skip_distances[0] && d > 1)
        {
            return "Skip pattern".to_string();
        }

        // Check for chunk boundary crossing (simplified)
        if sequence.len() > 5 {
            return "Chunk boundary crossing".to_string();
        }

        "Random sequence".to_string()
    }

    pub fn generate_semantic_filter(&mut self, category: String, position: usize) -> Task {
        let prompt = format!(
            "What is the {}th item in the '{}' category?",
            position, category
        );

        let correct_answer = if let Some(items) = self.semantic_attributes.get(&category) {
            if position > 0 && position <= items.len() {
                items[position - 1].clone()
            } else {
                "Position out of range".to_string()
            }
        } else {
            "Unknown category".to_string()
        };

        let mut options = vec![correct_answer.clone()];

        // Add some other items from the category
        if let Some(items) = self.semantic_attributes.get(&category) {
            for item in items.iter().take(5) {
                if !options.contains(item) {
                    options.push(item.clone());
                    if options.len() >= 4 {
                        break;
                    }
                }
            }
        }

        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::Index {
                item: correct_answer.clone(),
            },
            prompt,
            correct_answer,
            options,
            difficulty: 0.6,
            operation: OperationType::Index,
        }
    }

    pub fn generate_projection_switch(
        &self,
        item: String,
        from_view: String,
        to_view: String,
    ) -> Task {
        let prompt = format!(
            "If '{}' is at position X in '{}' view, what position in '{}' view?",
            item, from_view, to_view
        );

        let correct_answer = match (from_view.as_str(), to_view.as_str()) {
            ("alphabetical", "reverse") => {
                if let Some(node) = self.topology.get_node_by_label(&item) {
                    let pos = self.topology.node_map[&node.id];
                    let reverse_pos = self.topology.nodes.len() - pos;
                    reverse_pos.to_string()
                } else {
                    "Unknown".to_string()
                }
            }
            ("alphabetical", "vowels_only") => {
                if let Some(vowels) = self.semantic_attributes.get("vowel") {
                    if let Some(idx) = vowels.iter().position(|v| v == &item) {
                        (idx + 1).to_string()
                    } else {
                        "Not in this view".to_string()
                    }
                } else {
                    "Unknown".to_string()
                }
            }
            _ => "Same position".to_string(),
        };

        Task {
            task_type: TaskType::Index { item },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.5,
            operation: OperationType::Index,
        }
    }

    pub fn generate_isomorphic_transfer(
        &self,
        source_domain: String,
        target_domain: String,
    ) -> Task {
        let prompt = format!(
            "You learned pattern X in '{}'. Apply the same pattern in '{}'.",
            source_domain, target_domain
        );

        // Simplified: just test if they can do successor in a new domain
        let correct_answer = "Transfer successful".to_string();

        Task {
            task_type: TaskType::Successor {
                item: "A".to_string(),
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.7,
            operation: OperationType::Successor,
        }
    }
}

/// Dynamic graph adaptation support
pub struct DynamicTopology {
    base_topology: Topology,
    modifications: Vec<GraphModification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphModification {
    AddNode {
        id: String,
        label: String,
        position: f64,
    },
    RemoveNode {
        id: String,
    },
    AddEdge {
        from: String,
        to: String,
        weight: f64,
    },
    RemoveEdge {
        from: String,
        to: String,
    },
    UpdateNodePosition {
        id: String,
        new_position: f64,
    },
}

impl DynamicTopology {
    pub fn new(base: Topology) -> Self {
        DynamicTopology {
            base_topology: base,
            modifications: Vec::new(),
        }
    }

    pub fn apply_modification(&mut self, modification: GraphModification) {
        self.modifications.push(modification.clone());

        match modification {
            GraphModification::AddNode {
                id,
                label,
                position,
            } => {
                self.base_topology.nodes.push(crate::core::topology::Node {
                    id: id.clone(),
                    label,
                    position,
                });
                self.base_topology
                    .node_map
                    .insert(id, self.base_topology.nodes.len() - 1);
            }
            GraphModification::RemoveNode { id } => {
                self.base_topology.nodes.retain(|n| n.id != id);
                self.base_topology
                    .edges
                    .retain(|e| e.from != id && e.to != id);
                self.rebuild_node_map();
            }
            GraphModification::AddEdge { from, to, weight } => {
                self.base_topology
                    .edges
                    .push(crate::core::topology::Edge { from, to, weight });
            }
            GraphModification::RemoveEdge { from, to } => {
                self.base_topology
                    .edges
                    .retain(|e| !(e.from == from && e.to == to));
            }
            GraphModification::UpdateNodePosition { id, new_position } => {
                if let Some(node) = self.base_topology.nodes.iter_mut().find(|n| n.id == id) {
                    node.position = new_position;
                }
            }
        }
    }

    fn rebuild_node_map(&mut self) {
        self.base_topology.node_map.clear();
        for (i, node) in self.base_topology.nodes.iter().enumerate() {
            self.base_topology.node_map.insert(node.id.clone(), i);
        }
    }

    pub fn get_topology(&self) -> &Topology {
        &self.base_topology
    }

    pub fn rollback(&mut self, steps: usize) {
        for _ in 0..steps.min(self.modifications.len()) {
            self.modifications.pop();
        }
        // Rebuild topology from scratch with remaining modifications
        self.rebuild_from_modifications();
    }

    fn rebuild_from_modifications(&mut self) {
        // This would recreate the topology from base + modifications
        // Simplified for now
    }
}

/// Transfer learning framework
pub struct TransferLearning {
    source_domain: Topology,
    target_domain: Topology,
    mapping: HashMap<String, String>,
}

impl TransferLearning {
    pub fn new(source: Topology, target: Topology) -> Self {
        let mut mapping = HashMap::new();

        // Create isomorphic mapping if possible
        if source.nodes.len() == target.nodes.len() {
            for (s, t) in source.nodes.iter().zip(target.nodes.iter()) {
                mapping.insert(s.label.clone(), t.label.clone());
            }
        }

        TransferLearning {
            source_domain: source,
            target_domain: target,
            mapping,
        }
    }

    pub fn transfer_task(&self, source_task: &Task) -> Option<Task> {
        // Validate task is from source domain
        let is_valid_source = match &source_task.task_type {
            TaskType::Successor { item } | TaskType::Predecessor { item } => {
                self.source_domain.nodes.iter().any(|n| &n.label == item)
            }
            _ => true,
        };

        if !is_valid_source {
            return None;
        }

        // Map task from source to target domain
        match &source_task.task_type {
            TaskType::Successor { item } => {
                if let Some(target_item) = self.mapping.get(item) {
                    // Verify target item exists in target domain
                    if !self
                        .target_domain
                        .nodes
                        .iter()
                        .any(|n| &n.label == target_item)
                    {
                        return None;
                    }
                    Some(Task {
                        task_type: TaskType::Successor {
                            item: target_item.clone(),
                        },
                        prompt: source_task.prompt.replace(item, target_item),
                        correct_answer: self
                            .mapping
                            .get(&source_task.correct_answer)
                            .unwrap_or(&source_task.correct_answer)
                            .clone(),
                        options: source_task
                            .options
                            .iter()
                            .map(|o| self.mapping.get(o).unwrap_or(o).clone())
                            .collect(),
                        difficulty: source_task.difficulty,
                        operation: source_task.operation.clone(),
                    })
                } else {
                    None
                }
            }
            _ => None, // Simplified for now
        }
    }

    pub fn measure_transfer_efficiency(
        &self,
        source_performance: f64,
        target_performance: f64,
    ) -> f64 {
        // Calculate transfer efficiency metric
        if source_performance > 0.0 {
            target_performance / source_performance
        } else {
            0.0
        }
    }
}
