use crate::core::learner::OperationType;
use crate::tasks::types::{Task, TaskType};
use crate::core::topology::Topology;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationConstraint {
    AvoidNodes(Vec<String>),         // Don't pass through these
    RequireNodes(Vec<String>),       // Must pass through these
    MaxDistance(usize),              // Path length limit
    MinDistance(usize),              // Minimum path length
    RequireProperty(PropertyFilter), // Only nodes with property
    AvoidProperty(PropertyFilter),   // Avoid nodes with property
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyFilter {
    NodeType(String),         // e.g., "vowel", "consonant"
    Position(PositionFilter), // Position-based filtering
    Custom(String, String),   // Custom property key-value
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionFilter {
    Before(usize),
    After(usize),
    Between(usize, usize),
    Modulo(usize, usize), // position % n == m
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalNavigationTask {
    pub start: String,
    pub goal: String,
    pub constraints: Vec<NavigationConstraint>,
}

pub struct ConstrainedNavigator {
    topology: Topology,
    node_properties: HashMap<String, HashMap<String, String>>,
}

impl ConstrainedNavigator {
    pub fn new(topology: Topology) -> Self {
        let mut node_properties = HashMap::new();

        // Add default properties for alphabet
        if topology.nodes.len() == 26 {
            for node in &topology.nodes {
                let mut props = HashMap::new();
                let letter = &node.label;

                // Vowel/consonant classification
                if ["A", "E", "I", "O", "U"].contains(&letter.as_str()) {
                    props.insert("type".to_string(), "vowel".to_string());
                } else {
                    props.insert("type".to_string(), "consonant".to_string());
                }

                // Position properties
                props.insert("position".to_string(), node.position.to_string());
                props.insert(
                    "quadrant".to_string(),
                    (node.position as usize / 7).to_string(),
                );

                node_properties.insert(node.label.clone(), props);
            }
        }

        ConstrainedNavigator {
            topology,
            node_properties,
        }
    }

    pub fn find_constrained_path(&self, task: &ConditionalNavigationTask) -> Option<Vec<String>> {
        // Use A* with constraint checking
        let start_node = self.topology.get_node_by_label(&task.start)?;
        let goal_node = self.topology.get_node_by_label(&task.goal)?;

        let mut open_set = VecDeque::new();
        let mut came_from: HashMap<String, String> = HashMap::new();
        let mut g_score: HashMap<String, f64> = HashMap::new();
        let mut f_score: HashMap<String, f64> = HashMap::new();

        // Track which required nodes have been visited
        let required_nodes = self.get_required_nodes(&task.constraints);

        open_set.push_back((start_node.id.clone(), HashSet::new()));
        g_score.insert(start_node.id.clone(), 0.0);
        f_score.insert(
            start_node.id.clone(),
            self.heuristic(&start_node.id, &goal_node.id),
        );

        while let Some((current_id, visited_required)) = open_set.pop_front() {
            if current_id == goal_node.id {
                // Check if all required nodes were visited
                if visited_required.len() == required_nodes.len() {
                    return Some(self.reconstruct_path(&came_from, &current_id));
                }
            }

            let current_g = *g_score.get(&current_id).unwrap_or(&f64::INFINITY);

            // Check each neighbor
            for edge in &self.topology.edges {
                if edge.from != current_id {
                    continue;
                }

                let neighbor_id = &edge.to;
                let neighbor = self.topology.get_node_by_id(neighbor_id)?;

                // Check if neighbor violates constraints
                if !self.is_node_valid(neighbor, &task.constraints, current_g as usize + 1) {
                    continue;
                }

                let tentative_g = current_g + edge.weight;

                if tentative_g < *g_score.get(neighbor_id).unwrap_or(&f64::INFINITY) {
                    // Update visited required nodes
                    let mut new_visited = visited_required.clone();
                    if required_nodes.contains(&neighbor.label) {
                        new_visited.insert(neighbor.label.clone());
                    }

                    came_from.insert(neighbor_id.clone(), current_id.clone());
                    g_score.insert(neighbor_id.clone(), tentative_g);

                    let h = self.heuristic(neighbor_id, &goal_node.id);
                    f_score.insert(neighbor_id.clone(), tentative_g + h);

                    if !open_set.iter().any(|(id, _)| id == neighbor_id) {
                        open_set.push_back((neighbor_id.clone(), new_visited));
                    }
                }
            }

            // Sort by f_score
            open_set.make_contiguous().sort_by(|a, b| {
                let f_a = f_score.get(&a.0).unwrap_or(&f64::INFINITY);
                let f_b = f_score.get(&b.0).unwrap_or(&f64::INFINITY);
                f_a.partial_cmp(f_b).unwrap()
            });
        }

        None
    }

    fn get_required_nodes(&self, constraints: &[NavigationConstraint]) -> HashSet<String> {
        let mut required = HashSet::new();

        for constraint in constraints {
            if let NavigationConstraint::RequireNodes(nodes) = constraint {
                for node in nodes {
                    required.insert(node.clone());
                }
            }
        }

        required
    }

    fn is_node_valid(
        &self,
        node: &crate::core::topology::Node,
        constraints: &[NavigationConstraint],
        current_distance: usize,
    ) -> bool {
        for constraint in constraints {
            match constraint {
                NavigationConstraint::AvoidNodes(nodes) => {
                    if nodes.contains(&node.label) {
                        return false;
                    }
                }
                NavigationConstraint::MaxDistance(max) => {
                    if current_distance > *max {
                        return false;
                    }
                }
                NavigationConstraint::RequireProperty(filter) => {
                    if !self.node_matches_filter(&node.label, filter) {
                        return false;
                    }
                }
                NavigationConstraint::AvoidProperty(filter) => {
                    if self.node_matches_filter(&node.label, filter) {
                        return false;
                    }
                }
                _ => {}
            }
        }

        true
    }

    fn node_matches_filter(&self, node_label: &str, filter: &PropertyFilter) -> bool {
        match filter {
            PropertyFilter::NodeType(node_type) => self
                .node_properties
                .get(node_label)
                .and_then(|props| props.get("type"))
                .map(|t| t == node_type)
                .unwrap_or(false),
            PropertyFilter::Position(pos_filter) => {
                if let Some(node) = self.topology.get_node_by_label(node_label) {
                    let pos = node.position as usize;
                    match pos_filter {
                        PositionFilter::Before(n) => pos < *n,
                        PositionFilter::After(n) => pos > *n,
                        PositionFilter::Between(a, b) => pos >= *a && pos <= *b,
                        PositionFilter::Modulo(n, m) => pos % n == *m,
                    }
                } else {
                    false
                }
            }
            PropertyFilter::Custom(key, value) => self
                .node_properties
                .get(node_label)
                .and_then(|props| props.get(key))
                .map(|v| v == value)
                .unwrap_or(false),
        }
    }

    fn heuristic(&self, from: &str, to: &str) -> f64 {
        // Manhattan distance in graph space
        self.topology
            .get_distance(from, to)
            .map(|d| d as f64)
            .unwrap_or(f64::INFINITY)
    }

    fn reconstruct_path(&self, came_from: &HashMap<String, String>, current: &str) -> Vec<String> {
        let mut path = vec![self.topology.get_node_by_id(current).unwrap().label.clone()];
        let mut current_id = current.to_string();

        while let Some(prev) = came_from.get(&current_id) {
            if let Some(node) = self.topology.get_node_by_id(prev) {
                path.push(node.label.clone());
                current_id = prev.clone();
            } else {
                break;
            }
        }

        path.reverse();
        path
    }

    pub fn generate_conditional_task(&self, task: ConditionalNavigationTask) -> Task {
        let path = self.find_constrained_path(&task);

        let constraints_str = task
            .constraints
            .iter()
            .map(|c| match c {
                NavigationConstraint::AvoidNodes(nodes) => format!("avoiding {}", nodes.join(", ")),
                NavigationConstraint::RequireNodes(nodes) => {
                    format!("passing through {}", nodes.join(", "))
                }
                NavigationConstraint::MaxDistance(d) => format!("in at most {} steps", d),
                NavigationConstraint::MinDistance(d) => format!("in at least {} steps", d),
                NavigationConstraint::RequireProperty(PropertyFilter::NodeType(t)) => {
                    format!("only through {}s", t)
                }
                _ => String::new(),
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");

        let prompt = format!(
            "Navigate from '{}' to '{}' {}",
            task.start, task.goal, constraints_str
        );

        let correct_answer = path
            .map(|p| p.join(" → "))
            .unwrap_or_else(|| "No valid path".to_string());

        Task {
            task_type: TaskType::ShortestPath {
                from: task.start.clone(),
                to: task.goal.clone(),
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.7 + (task.constraints.len() as f64 * 0.1).min(0.3),
            operation: OperationType::PairwiseOrder,
        }
    }
}

// Multi-hop reasoning tasks
pub struct MultiHopNavigator {
    topology: Topology,
}

impl MultiHopNavigator {
    pub fn new(topology: Topology) -> Self {
        MultiHopNavigator { topology }
    }

    pub fn find_equidistant_nodes(&self, from_a: &str, from_b: &str) -> Vec<String> {
        let mut equidistant = Vec::new();

        for node in &self.topology.nodes {
            if let (Some(dist_a), Some(dist_b)) = (
                self.topology.get_distance(from_a, &node.label),
                self.topology.get_distance(from_b, &node.label),
            ) {
                if dist_a == dist_b {
                    equidistant.push(node.label.clone());
                }
            }
        }

        equidistant
    }

    pub fn find_nodes_at_distances(&self, constraints: Vec<(String, usize)>) -> Vec<String> {
        let mut valid_nodes = Vec::new();

        for node in &self.topology.nodes {
            let mut all_match = true;

            for (from, target_dist) in &constraints {
                if let Some(dist) = self.topology.get_distance(from, &node.label) {
                    if dist != *target_dist {
                        all_match = false;
                        break;
                    }
                } else {
                    all_match = false;
                    break;
                }
            }

            if all_match {
                valid_nodes.push(node.label.clone());
            }
        }

        valid_nodes
    }

    pub fn generate_multi_hop_task(&self) -> Task {
        let nodes = &self.topology.nodes;
        let n = nodes.len();

        // Pick random anchor points
        let a_idx = rand::random::<usize>() % n;
        let b_idx = (a_idx + n / 3) % n;

        let a = &nodes[a_idx].label;
        let b = &nodes[b_idx].label;

        let equidistant = self.find_equidistant_nodes(a, b);

        let prompt = format!(
            "Find all nodes that are equidistant from '{}' and '{}'",
            a, b
        );

        let correct_answer = if equidistant.is_empty() {
            "None".to_string()
        } else {
            equidistant.join(", ")
        };

        Task {
            task_type: TaskType::ShortestDistance {
                from: a.clone(),
                to: b.clone(),
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.8,
            operation: OperationType::PairwiseOrder,
        }
    }

    pub fn generate_distance_constraint_task(&self) -> Task {
        let nodes = &self.topology.nodes;

        // Create constraints
        let anchor = &nodes[rand::random::<usize>() % nodes.len()].label;
        let dist1 = (rand::random::<usize>() % 5) + 1;
        let _dist2 = (rand::random::<usize>() % 5) + 1;

        let valid = self.find_nodes_at_distances(vec![(anchor.clone(), dist1)]);

        let prompt = format!("What nodes are exactly {} steps from '{}'?", dist1, anchor);

        let correct_answer = if valid.is_empty() {
            "None".to_string()
        } else {
            valid.join(", ")
        };

        Task {
            task_type: TaskType::Index {
                item: anchor.clone(),
            },
            prompt,
            correct_answer,
            options: vec![],
            difficulty: 0.6,
            operation: OperationType::Index,
        }
    }
}
