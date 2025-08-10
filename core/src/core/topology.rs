use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TopologyType {
    Linear,
    Cyclic,
    PartialOrder,
    GeneralGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub position: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub topology_type: TopologyType,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub node_map: HashMap<String, usize>,
}

impl Topology {
    pub fn new_linear(items: Vec<String>) -> Self {
        let n = items.len();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        for (i, label) in items.iter().enumerate() {
            let id = format!("node_{}", i);
            nodes.push(Node {
                id: id.clone(),
                label: label.clone(),
                position: i as f64,
            });
            node_map.insert(id.clone(), i);

            if i < n - 1 {
                edges.push(Edge {
                    from: id.clone(),
                    to: format!("node_{}", i + 1),
                    weight: 1.0,
                });
            }
        }

        Topology {
            topology_type: TopologyType::Linear,
            nodes,
            edges,
            node_map,
        }
    }

    pub fn new_cyclic(items: Vec<String>) -> Self {
        let n = items.len();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        for (i, label) in items.iter().enumerate() {
            let id = format!("node_{}", i);
            let angle = (2.0 * PI * i as f64) / n as f64;
            nodes.push(Node {
                id: id.clone(),
                label: label.clone(),
                position: angle,
            });
            node_map.insert(id.clone(), i);

            let next_idx = (i + 1) % n;
            edges.push(Edge {
                from: id.clone(),
                to: format!("node_{}", next_idx),
                weight: 1.0,
            });
        }

        Topology {
            topology_type: TopologyType::Cyclic,
            nodes,
            edges,
            node_map,
        }
    }

    pub fn alphabet() -> Self {
        let alphabet: Vec<String> = (b'A'..=b'Z').map(|c| (c as char).to_string()).collect();
        Self::new_linear(alphabet)
    }

    pub fn days_of_week() -> Self {
        let days = vec![
            "Monday".to_string(),
            "Tuesday".to_string(),
            "Wednesday".to_string(),
            "Thursday".to_string(),
            "Friday".to_string(),
            "Saturday".to_string(),
            "Sunday".to_string(),
        ];
        Self::new_cyclic(days)
    }

    pub fn get_successor(&self, node_id: &str) -> Option<String> {
        match self.topology_type {
            TopologyType::Cyclic => {
                let idx = *self.node_map.get(node_id)?;
                let next_idx = (idx + 1) % self.nodes.len();
                Some(format!("node_{}", next_idx))
            }
            _ => self
                .edges
                .iter()
                .find(|e| e.from == node_id)
                .map(|e| e.to.clone()),
        }
    }

    pub fn get_predecessor(&self, node_id: &str) -> Option<String> {
        match self.topology_type {
            TopologyType::Cyclic => {
                let idx = *self.node_map.get(node_id)?;
                let n = self.nodes.len();
                let prev_idx = (idx + n - 1) % n;
                Some(format!("node_{}", prev_idx))
            }
            _ => self
                .edges
                .iter()
                .find(|e| e.to == node_id)
                .map(|e| e.from.clone()),
        }
    }

    pub fn get_distance(&self, from: &str, to: &str) -> Option<usize> {
        // Accept either node IDs (e.g., "node_0") or labels (e.g., "Mon")
        let from_idx = if let Some(idx) = self.node_map.get(from) {
            *idx
        } else {
            self.get_node_by_label(from)
                .and_then(|n| self.node_map.get(&n.id).copied())?
        };
        let to_idx = if let Some(idx) = self.node_map.get(to) {
            *idx
        } else {
            self.get_node_by_label(to)
                .and_then(|n| self.node_map.get(&n.id).copied())?
        };

        match self.topology_type {
            TopologyType::Linear => Some((to_idx as i32 - from_idx as i32).abs() as usize),
            TopologyType::Cyclic => {
                let n = self.nodes.len();
                let forward = (to_idx + n - from_idx) % n;
                let backward = (from_idx + n - to_idx) % n;
                Some(forward.min(backward))
            }
            TopologyType::PartialOrder | TopologyType::GeneralGraph => {
                let path =
                    self.shortest_path(&self.nodes[from_idx].label, &self.nodes[to_idx].label)?;
                Some(path.len().saturating_sub(1))
            }
        }
    }

    pub fn get_k_jump(&self, start: &str, k: i32) -> Option<String> {
        let start_idx = self.node_map.get(start)?;
        let n = self.nodes.len();

        let target_idx = match self.topology_type {
            TopologyType::Linear => {
                let new_idx = *start_idx as i32 + k;
                if new_idx < 0 || new_idx >= n as i32 {
                    return None;
                }
                new_idx as usize
            }
            TopologyType::Cyclic => {
                let new_idx = (*start_idx as i32 + k).rem_euclid(n as i32);
                new_idx as usize
            }
            TopologyType::PartialOrder | TopologyType::GeneralGraph => {
                return None;
            }
        };

        Some(format!("node_{}", target_idx))
    }

    pub fn get_node_by_label(&self, label: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.label == label)
    }

    pub fn get_node_by_id(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn is_before(&self, a: &str, b: &str) -> Option<bool> {
        let a_node = self.get_node_by_label(a)?;
        let b_node = self.get_node_by_label(b)?;

        match self.topology_type {
            TopologyType::Linear => Some(a_node.position < b_node.position),
            TopologyType::Cyclic => None,
            TopologyType::PartialOrder => self.has_path(&a_node.id, &b_node.id),
            TopologyType::GeneralGraph => None,
        }
    }

    pub fn new_dag(nodes_labels: Vec<String>, dependencies: Vec<(String, String)>) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        for (i, label) in nodes_labels.iter().enumerate() {
            let id = format!("node_{}", i);
            nodes.push(Node {
                id: id.clone(),
                label: label.clone(),
                position: i as f64,
            });
            node_map.insert(id.clone(), i);
        }

        for (from_label, to_label) in dependencies {
            if let (Some(from_node), Some(to_node)) = (
                nodes.iter().find(|n| n.label == from_label),
                nodes.iter().find(|n| n.label == to_label),
            ) {
                edges.push(Edge {
                    from: from_node.id.clone(),
                    to: to_node.id.clone(),
                    weight: 1.0,
                });
            }
        }

        Topology {
            topology_type: TopologyType::PartialOrder,
            nodes,
            edges,
            node_map,
        }
    }

    pub fn new_graph(nodes_labels: Vec<String>, connections: Vec<(String, String, f64)>) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        for (i, label) in nodes_labels.iter().enumerate() {
            let id = format!("node_{}", i);
            nodes.push(Node {
                id: id.clone(),
                label: label.clone(),
                position: i as f64,
            });
            node_map.insert(id.clone(), i);
        }

        for (from_label, to_label, weight) in connections {
            if let (Some(from_node), Some(to_node)) = (
                nodes.iter().find(|n| n.label == from_label),
                nodes.iter().find(|n| n.label == to_label),
            ) {
                edges.push(Edge {
                    from: from_node.id.clone(),
                    to: to_node.id.clone(),
                    weight,
                });
            }
        }

        Topology {
            topology_type: TopologyType::GeneralGraph,
            nodes,
            edges,
            node_map,
        }
    }

    pub fn example_dag() -> Self {
        let nodes = vec![
            "Database Setup".to_string(),
            "Create Tables".to_string(),
            "User Auth".to_string(),
            "API Server".to_string(),
            "Frontend".to_string(),
            "Deploy".to_string(),
            "Config Files".to_string(),
        ];

        let dependencies = vec![
            ("Database Setup".to_string(), "Create Tables".to_string()),
            ("Database Setup".to_string(), "User Auth".to_string()),
            ("Create Tables".to_string(), "API Server".to_string()),
            ("User Auth".to_string(), "API Server".to_string()),
            ("Config Files".to_string(), "API Server".to_string()),
            ("API Server".to_string(), "Frontend".to_string()),
            ("Frontend".to_string(), "Deploy".to_string()),
            ("API Server".to_string(), "Deploy".to_string()),
        ];

        Self::new_dag(nodes, dependencies)
    }

    pub fn has_path(&self, from: &str, to: &str) -> Option<bool> {
        if from == to {
            return Some(true);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(from.to_string());

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            for edge in &self.edges {
                if edge.from == current {
                    if edge.to == to {
                        return Some(true);
                    }
                    queue.push_back(edge.to.clone());
                }
            }
        }

        Some(false)
    }

    pub fn are_comparable(&self, a: &str, b: &str) -> Option<bool> {
        let a_node = self.get_node_by_label(a)?;
        let b_node = self.get_node_by_label(b)?;

        match self.topology_type {
            TopologyType::PartialOrder => {
                let has_ab = self.has_path(&a_node.id, &b_node.id)?;
                let has_ba = self.has_path(&b_node.id, &a_node.id)?;
                Some(has_ab || has_ba)
            }
            _ => Some(true),
        }
    }

    pub fn get_topological_sort(&self) -> Option<Vec<String>> {
        if !matches!(self.topology_type, TopologyType::PartialOrder) {
            return None;
        }

        let mut in_degree = HashMap::new();
        for node in &self.nodes {
            in_degree.insert(node.id.clone(), 0);
        }

        for edge in &self.edges {
            *in_degree.get_mut(&edge.to).unwrap() += 1;
        }

        let mut queue = VecDeque::new();
        for (node_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        let mut result = Vec::new();
        while let Some(current) = queue.pop_front() {
            if let Some(node) = self.get_node_by_id(&current) {
                result.push(node.label.clone());
            }

            for edge in &self.edges {
                if edge.from == current {
                    let count = in_degree.get_mut(&edge.to).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        queue.push_back(edge.to.clone());
                    }
                }
            }
        }

        if result.len() == self.nodes.len() {
            Some(result)
        } else {
            None
        }
    }

    pub fn shortest_path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let from_node = self.get_node_by_label(from)?;
        let to_node = self.get_node_by_label(to)?;

        let mut distances = HashMap::new();
        let mut previous = HashMap::new();
        let mut unvisited = HashSet::new();

        for node in &self.nodes {
            distances.insert(node.id.clone(), f64::INFINITY);
            unvisited.insert(node.id.clone());
        }
        distances.insert(from_node.id.clone(), 0.0);

        while !unvisited.is_empty() {
            let current = unvisited
                .iter()
                .min_by(|a, b| {
                    distances[*a]
                        .partial_cmp(&distances[*b])
                        .unwrap_or(std::cmp::Ordering::Equal)
                })?
                .clone();

            if current == to_node.id {
                break;
            }

            unvisited.remove(&current);

            for edge in &self.edges {
                if edge.from == current && unvisited.contains(&edge.to) {
                    let alt = distances[&current] + edge.weight;
                    if alt < distances[&edge.to] {
                        distances.insert(edge.to.clone(), alt);
                        previous.insert(edge.to.clone(), current.clone());
                    }
                }
            }
        }

        if !previous.contains_key(&to_node.id) && from_node.id != to_node.id {
            return None;
        }

        let mut path = Vec::new();
        let mut current = to_node.id.clone();

        while current != from_node.id {
            if let Some(node) = self.get_node_by_id(&current) {
                path.push(node.label.clone());
            }
            current = previous.get(&current)?.clone();
        }

        path.push(from_node.label.clone());
        path.reverse();

        Some(path)
    }

    pub fn get_segment(&self, start: &str, count: usize, reverse: bool) -> Vec<String> {
        let mut result = Vec::new();
        let start_node = match self.get_node_by_label(start) {
            Some(n) => n,
            None => return result,
        };

        let mut current_id = start_node.id.clone();
        result.push(start_node.label.clone());

        for _ in 1..count {
            let next_id = if reverse {
                self.get_predecessor(&current_id)
            } else {
                self.get_successor(&current_id)
            };

            match next_id {
                Some(id) => {
                    if let Some(node) = self.get_node_by_id(&id) {
                        result.push(node.label.clone());
                        current_id = id;
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_topology() {
        let topo = Topology::alphabet();
        assert_eq!(topo.nodes.len(), 26);
        assert_eq!(topo.get_node_by_label("A").unwrap().position, 0.0);
        assert_eq!(topo.get_node_by_label("Z").unwrap().position, 25.0);
        assert!(topo.is_before("A", "B").unwrap());
        assert!(!topo.is_before("Z", "A").unwrap());
    }

    #[test]
    fn test_cyclic_topology() {
        let topo = Topology::days_of_week();
        assert_eq!(topo.nodes.len(), 7);
        assert_eq!(topo.get_distance("node_0", "node_3"), Some(3));
        assert_eq!(topo.get_distance("node_6", "node_1"), Some(2));
    }

    #[test]
    fn test_segment() {
        let topo = Topology::alphabet();
        let segment = topo.get_segment("D", 3, false);
        assert_eq!(segment, vec!["D", "E", "F"]);

        let reverse_segment = topo.get_segment("D", 3, true);
        assert_eq!(reverse_segment, vec!["D", "C", "B"]);
    }
}
