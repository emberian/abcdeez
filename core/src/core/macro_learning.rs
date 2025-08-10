use crate::core::learner::OperationType;
use crate::tasks::types::{Task, TaskType};
use crate::core::topology::Topology;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a discovered macro (sequence pattern) that can be reused
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub name: String,
    pub pattern: Vec<String>,
    pub frequency: usize,
    pub context: MacroContext,
    pub abstraction_level: AbstractionLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MacroContext {
    Sequential,          // Simple sequence (e.g., ABC, DEF)
    SkipPattern(usize),  // Skip pattern (e.g., ACE with skip=1)
    Boundary,            // Crosses chunk boundary
    Cyclic,              // Wraps around in cyclic topology
    Hierarchical(usize), // Nested macro at level N
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbstractionLevel {
    Concrete,   // Specific items (A, B, C)
    Relative,   // Relative positions (+1, +2)
    Structural, // Graph structure (successor, predecessor)
    Functional, // Functional role (vowel-consonant-vowel)
}

/// System for discovering and applying macros in navigation
pub struct MacroDiscoverySystem {
    topology: Topology,
    discovered_macros: HashMap<String, Macro>,
    macro_usage_history: Vec<(String, usize)>, // (macro_name, timestamp)
    pattern_frequency: HashMap<Vec<String>, usize>,
    abstraction_enabled: bool,
    rng: rand::rngs::StdRng,
}

impl MacroDiscoverySystem {
    pub fn new(topology: Topology) -> Self {
        Self::with_seed(topology, None)
    }

    pub fn with_seed(topology: Topology, seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_entropy(),
        };

        let mut system = MacroDiscoverySystem {
            topology,
            discovered_macros: HashMap::new(),
            macro_usage_history: Vec::new(),
            pattern_frequency: HashMap::new(),
            abstraction_enabled: true,
            rng,
        };

        // Initialize with common patterns
        system.initialize_common_macros();
        system
    }

    fn initialize_common_macros(&mut self) {
        // Common alphabet patterns
        if self.topology.nodes.len() == 26 {
            // ABC pattern
            self.discovered_macros.insert(
                "abc_forward".to_string(),
                Macro {
                    name: "abc_forward".to_string(),
                    pattern: vec!["A".to_string(), "B".to_string(), "C".to_string()],
                    frequency: 0,
                    context: MacroContext::Sequential,
                    abstraction_level: AbstractionLevel::Concrete,
                },
            );

            // Vowel sequence
            self.discovered_macros.insert(
                "vowels".to_string(),
                Macro {
                    name: "vowels".to_string(),
                    pattern: vec!["A", "E", "I", "O", "U"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    frequency: 0,
                    context: MacroContext::SkipPattern(3),
                    abstraction_level: AbstractionLevel::Functional,
                },
            );

            // Chunk endings (common boundary positions)
            self.discovered_macros.insert(
                "chunk_end".to_string(),
                Macro {
                    name: "chunk_end".to_string(),
                    pattern: vec!["F", "G", "H"].iter().map(|s| s.to_string()).collect(),
                    frequency: 0,
                    context: MacroContext::Boundary,
                    abstraction_level: AbstractionLevel::Structural,
                },
            );
        }
    }

    /// Discover macros from a sequence of navigation actions
    pub fn discover_macro_from_sequence(&mut self, sequence: &[String]) -> Option<Macro> {
        if sequence.len() < 3 {
            return None;
        }

        // Update frequency tracking
        *self.pattern_frequency.entry(sequence.to_vec()).or_insert(0) += 1;

        // Check if this pattern is frequent enough to become a macro
        let frequency = *self.pattern_frequency.get(sequence).unwrap();
        if frequency < 3 {
            return None; // Need at least 3 occurrences
        }

        // Analyze the pattern to determine its type
        let context = self.analyze_pattern_context(sequence);
        let abstraction = self.determine_abstraction_level(sequence);

        // Generate a name for the macro
        let name = self.generate_macro_name(sequence, &context);

        let macro_def = Macro {
            name: name.clone(),
            pattern: sequence.to_vec(),
            frequency,
            context,
            abstraction_level: abstraction,
        };

        self.discovered_macros
            .insert(name.clone(), macro_def.clone());
        Some(macro_def)
    }

    fn analyze_pattern_context(&self, sequence: &[String]) -> MacroContext {
        // Check if it's a simple sequential pattern
        let mut is_sequential = true;
        for i in 0..sequence.len() - 1 {
            if let (Some(curr), Some(next)) = (
                self.topology.get_node_by_label(&sequence[i]),
                self.topology.get_node_by_label(&sequence[i + 1]),
            ) {
                if self.topology.get_successor(&curr.id) != Some(next.id.clone()) {
                    is_sequential = false;
                    break;
                }
            }
        }

        if is_sequential {
            return MacroContext::Sequential;
        }

        // Check for skip patterns
        if let Some(skip_size) = self.detect_skip_pattern(sequence) {
            return MacroContext::SkipPattern(skip_size);
        }

        // Check if it crosses boundaries
        if self.crosses_chunk_boundary(sequence) {
            return MacroContext::Boundary;
        }

        // Check for cyclic wraparound
        if self.has_cyclic_wrap(sequence) {
            return MacroContext::Cyclic;
        }

        MacroContext::Sequential // Default
    }

    fn detect_skip_pattern(&self, sequence: &[String]) -> Option<usize> {
        if sequence.len() < 2 {
            return None;
        }

        let positions: Vec<_> = sequence
            .iter()
            .filter_map(|label| self.topology.get_node_by_label(label))
            .map(|node| node.position as usize)
            .collect();

        if positions.len() < 2 {
            return None;
        }

        let skip = positions[1] - positions[0];
        for i in 1..positions.len() - 1 {
            if positions[i + 1] - positions[i] != skip {
                return None;
            }
        }

        Some(skip - 1) // Return skip size (0 means consecutive)
    }

    fn crosses_chunk_boundary(&self, sequence: &[String]) -> bool {
        // Check if sequence crosses common chunk boundaries (e.g., F-G, M-N, S-T)
        let boundaries = vec![5, 12, 18]; // Common alphabet chunk boundaries

        let positions: Vec<_> = sequence
            .iter()
            .filter_map(|label| self.topology.get_node_by_label(label))
            .map(|node| node.position as usize)
            .collect();

        for i in 0..positions.len() - 1 {
            for &boundary in &boundaries {
                if positions[i] <= boundary && positions[i + 1] > boundary {
                    return true;
                }
            }
        }
        false
    }

    fn has_cyclic_wrap(&self, sequence: &[String]) -> bool {
        if !matches!(
            self.topology.topology_type,
            crate::core::topology::TopologyType::Cyclic
        ) {
            return false;
        }

        let positions: Vec<_> = sequence
            .iter()
            .filter_map(|label| self.topology.get_node_by_label(label))
            .map(|node| node.position as usize)
            .collect();

        for i in 0..positions.len() - 1 {
            // Check for wrap from end to beginning
            if positions[i] > positions[i + 1] + 10 {
                return true;
            }
        }
        false
    }

    fn determine_abstraction_level(&self, sequence: &[String]) -> AbstractionLevel {
        // If abstraction is disabled, always return Concrete
        if !self.abstraction_enabled {
            return AbstractionLevel::Concrete;
        }

        // Check if all items are vowels or consonants
        let vowels = vec!["A", "E", "I", "O", "U"];
        let all_vowels = sequence.iter().all(|s| vowels.contains(&s.as_str()));
        let all_consonants = sequence.iter().all(|s| !vowels.contains(&s.as_str()));

        if all_vowels || all_consonants {
            return AbstractionLevel::Functional;
        }

        // Check if it's a relative pattern (consistent jumps)
        if self.detect_skip_pattern(sequence).is_some() {
            return AbstractionLevel::Relative;
        }

        // Check for structural patterns
        if self.crosses_chunk_boundary(sequence) {
            return AbstractionLevel::Structural;
        }

        AbstractionLevel::Concrete
    }

    fn generate_macro_name(&self, sequence: &[String], context: &MacroContext) -> String {
        let prefix = match context {
            MacroContext::Sequential => "seq",
            MacroContext::SkipPattern(n) => &format!("skip{}", n),
            MacroContext::Boundary => "boundary",
            MacroContext::Cyclic => "cyclic",
            MacroContext::Hierarchical(n) => &format!("hier{}", n),
        };

        let content = if sequence.len() <= 3 {
            sequence.join("")
        } else {
            format!("{}..{}", sequence[0], sequence[sequence.len() - 1])
        };

        format!("{}_{}", prefix, content.to_lowercase())
    }

    /// Generate a task that teaches macro discovery
    pub fn generate_macro_discovery_task(&mut self) -> Task {
        // Select a random discovered macro or create a new pattern
        let macros: Vec<_> = self.discovered_macros.values().collect();

        let (pattern, macro_name) = if !macros.is_empty() && self.rng.gen::<bool>() {
            let macro_def = macros[self.rng.gen_range(0..macros.len())];
            (macro_def.pattern.clone(), macro_def.name.clone())
        } else {
            // Generate a new pattern for discovery
            let start_idx = self.rng.gen_range(0..(self.topology.nodes.len() - 4));
            let pattern_type = self.rng.gen_range(0..3);

            let pattern = match pattern_type {
                0 => {
                    // Sequential pattern
                    (0..4)
                        .map(|i| self.topology.nodes[start_idx + i].label.clone())
                        .collect()
                }
                1 => {
                    // Skip pattern
                    (0..3)
                        .map(|i| self.topology.nodes[start_idx + i * 2].label.clone())
                        .collect()
                }
                _ => {
                    // Boundary crossing pattern
                    let boundary = 6; // F-G boundary
                    let start = (boundary - 1).max(0);
                    (0..3)
                        .map(|i| self.topology.nodes[start + i].label.clone())
                        .collect()
                }
            };

            (pattern, "unknown".to_string())
        };

        let prompt = format!(
            "What macro or pattern does this sequence represent: {}?",
            pattern.join(", ")
        );

        let correct_answer = if macro_name != "unknown" {
            macro_name
        } else {
            self.identify_pattern_type(&pattern)
        };

        let mut options = vec![
            correct_answer.clone(),
            "Sequential forward".to_string(),
            "Skip pattern".to_string(),
            "Boundary crossing".to_string(),
            "Random sequence".to_string(),
        ];
        options.dedup();
        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::Segment {
                start: pattern[0].clone(),
                count: pattern.len(),
                reverse: false,
            },
            prompt,
            correct_answer,
            options,
            difficulty: 0.6 + (pattern.len() as f64 * 0.05),
            operation: OperationType::Segment(pattern.len(), false),
        }
    }

    fn identify_pattern_type(&self, sequence: &[String]) -> String {
        let context = self.analyze_pattern_context(sequence);
        match context {
            MacroContext::Sequential => "Sequential forward".to_string(),
            MacroContext::SkipPattern(n) => format!("Skip pattern (skip {})", n),
            MacroContext::Boundary => "Boundary crossing".to_string(),
            MacroContext::Cyclic => "Cyclic wraparound".to_string(),
            MacroContext::Hierarchical(n) => format!("Hierarchical level {}", n),
        }
    }

    /// Apply a discovered macro to navigate
    pub fn apply_macro(&mut self, macro_name: &str, start: &str) -> Option<Vec<String>> {
        // Clone the macro data to avoid borrow conflicts
        let (macro_pattern, abstraction_level, context) = {
            let m = self.discovered_macros.get(macro_name)?;
            (
                m.pattern.clone(),
                m.abstraction_level.clone(),
                m.context.clone(),
            )
        };

        // Record usage
        self.macro_usage_history.push((
            macro_name.to_string(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize,
        ));

        // Update frequency
        if let Some(m) = self.discovered_macros.get_mut(macro_name) {
            m.frequency += 1;
        }

        // Apply the macro pattern starting from the given position
        let start_node = self.topology.get_node_by_label(start)?;
        let mut result = vec![start.to_string()];

        match &abstraction_level {
            AbstractionLevel::Concrete => {
                // Direct application if starting point matches
                if macro_pattern[0] == start {
                    return Some(macro_pattern);
                }
            }
            AbstractionLevel::Relative => {
                // Apply relative jumps from start
                if let MacroContext::SkipPattern(skip) = context {
                    let mut current_idx = self.topology.node_map[&start_node.id];
                    for _ in 1..macro_pattern.len() {
                        current_idx += skip + 1;
                        if current_idx < self.topology.nodes.len() {
                            result.push(self.topology.nodes[current_idx].label.clone());
                        }
                    }
                    return Some(result);
                }
            }
            _ => {
                // For other abstraction levels, adapt the pattern
                // Need to reconstruct macro_def for adapt_macro_to_context
                let macro_def = Macro {
                    name: macro_name.to_string(),
                    pattern: macro_pattern,
                    abstraction_level,
                    context,
                    frequency: 0, // Frequency doesn't matter for adaptation
                };
                return self.adapt_macro_to_context(&macro_def, start);
            }
        }

        None
    }

    fn adapt_macro_to_context(&self, macro_def: &Macro, start: &str) -> Option<Vec<String>> {
        // Adapt abstract macros to specific starting positions
        match &macro_def.abstraction_level {
            AbstractionLevel::Structural => {
                // Apply structural pattern (e.g., chunk boundaries)
                let result = vec![start.to_string()];
                // Implementation depends on specific structural pattern
                Some(result)
            }
            AbstractionLevel::Functional => {
                // Apply functional pattern (e.g., vowel sequence)
                if macro_def.name == "vowels" {
                    return Some(
                        vec!["A", "E", "I", "O", "U"]
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                    );
                }
                None
            }
            _ => None,
        }
    }

    /// Generate a task that uses macro application
    pub fn generate_macro_application_task(&mut self) -> Task {
        let macros: Vec<_> = self.discovered_macros.values().collect();
        if macros.is_empty() {
            return self.generate_macro_discovery_task();
        }

        let macro_def = macros[self.rng.gen_range(0..macros.len())];
        let start_options: Vec<_> = self
            .topology
            .nodes
            .iter()
            .take(self.topology.nodes.len() - macro_def.pattern.len())
            .map(|n| n.label.clone())
            .collect();

        let start = &start_options[self.rng.gen_range(0..start_options.len())];

        let prompt = format!(
            "Apply the '{}' macro starting from '{}'). What is the resulting sequence?",
            macro_def.name, start
        );

        let correct_sequence = self.apply_macro_simulation(&macro_def, start);
        let correct_answer = correct_sequence.join(", ");

        // Generate distractors
        let mut options = vec![correct_answer.clone()];

        // Wrong direction
        let mut reversed = correct_sequence.clone();
        reversed.reverse();
        options.push(reversed.join(", "));

        // Off by one
        if correct_sequence.len() > 1 {
            let off_by_one = correct_sequence[1..].to_vec();
            options.push(off_by_one.join(", "));
        }

        // Random sequence
        let random_seq = self
            .topology
            .nodes
            .iter()
            .skip(self.rng.gen_range(0..10))
            .take(macro_def.pattern.len())
            .map(|n| n.label.clone())
            .collect::<Vec<_>>();
        options.push(random_seq.join(", "));

        options.dedup();
        options.shuffle(&mut self.rng);

        Task {
            task_type: TaskType::Segment {
                start: start.clone(),
                count: macro_def.pattern.len(),
                reverse: false,
            },
            prompt,
            correct_answer,
            options,
            difficulty: 0.7 + (macro_def.pattern.len() as f64 * 0.03),
            operation: OperationType::Segment(macro_def.pattern.len(), false),
        }
    }

    fn apply_macro_simulation(&self, macro_def: &Macro, start: &str) -> Vec<String> {
        // Simulate macro application for task generation
        match &macro_def.context {
            MacroContext::Sequential => {
                let start_node = self.topology.get_node_by_label(start).unwrap();
                let start_idx = self.topology.node_map[&start_node.id];
                (0..macro_def.pattern.len())
                    .map(|i| {
                        let idx = (start_idx + i) % self.topology.nodes.len();
                        self.topology.nodes[idx].label.clone()
                    })
                    .collect()
            }
            MacroContext::SkipPattern(skip) => {
                let start_node = self.topology.get_node_by_label(start).unwrap();
                let start_idx = self.topology.node_map[&start_node.id];
                (0..macro_def.pattern.len())
                    .map(|i| {
                        let idx = (start_idx + i * (skip + 1)) % self.topology.nodes.len();
                        self.topology.nodes[idx].label.clone()
                    })
                    .collect()
            }
            _ => {
                // For other patterns, use the original pattern as reference
                macro_def.pattern.clone()
            }
        }
    }
}
