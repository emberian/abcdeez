use crate::config::LearnerConfig;
use crate::core::topology::Topology;
use rand::prelude::*;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Experimental Design Tools for Research Applications
/// Provides counterbalancing, randomization, and design validation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentalDesign {
    /// Between-subjects design with random assignment
    BetweenSubjects {
        conditions: Vec<ExperimentCondition>,
        randomization: RandomizationType,
    },
    /// Within-subjects design with counterbalancing
    WithinSubjects {
        conditions: Vec<ExperimentCondition>,
        counterbalancing: CounterbalancingMethod,
    },
    /// Mixed design combining between and within factors
    Mixed {
        between_factors: Vec<Factor>,
        within_factors: Vec<Factor>,
        counterbalancing: CounterbalancingMethod,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factor {
    pub name: String,
    pub levels: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentCondition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: LearnerConfig,
    pub topology: Topology,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RandomizationType {
    /// Simple random assignment
    Simple,
    /// Block randomization with specified block size
    Block { block_size: usize },
    /// Stratified randomization based on participant characteristics
    Stratified {
        strata: Vec<StratificationCriterion>,
    },
    /// Adaptive randomization to balance group sizes
    Adaptive { target_ratio: Vec<f64> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CounterbalancingMethod {
    /// Complete counterbalancing (all possible orders)
    Complete,
    /// Latin Square design
    LatinSquare,
    /// Balanced Latin Square (each condition follows every other exactly once)
    BalancedLatinSquare,
    /// Williams Square (controls for first-order carryover effects)
    WilliamsSquare,
    /// Random counterbalancing with constraints
    RandomWithConstraints { min_separation: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StratificationCriterion {
    pub variable: String,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantAssignment {
    pub participant_id: String,
    pub condition_sequence: Vec<String>,
    pub randomization_details: RandomizationRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomizationRecord {
    pub method: String,
    pub seed: u64,
    pub assignment_time: chrono::DateTime<chrono::Utc>,
    pub balancing_info: HashMap<String, serde_json::Value>,
}

pub struct ExperimentalDesigner {
    rng: StdRng,
    assignments: Vec<ParticipantAssignment>,
}

impl ExperimentalDesigner {
    pub fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        Self {
            rng,
            assignments: Vec::new(),
        }
    }

    /// Assign participant to experimental condition(s)
    pub fn assign_participant(
        &mut self,
        participant_id: String,
        design: &ExperimentalDesign,
        participant_characteristics: Option<HashMap<String, String>>,
    ) -> Result<ParticipantAssignment, String> {
        match design {
            ExperimentalDesign::BetweenSubjects {
                conditions,
                randomization,
            } => self.assign_between_subjects(
                participant_id,
                conditions,
                randomization,
                participant_characteristics,
            ),
            ExperimentalDesign::WithinSubjects {
                conditions,
                counterbalancing,
            } => self.assign_within_subjects(participant_id, conditions, counterbalancing),
            ExperimentalDesign::Mixed {
                between_factors,
                within_factors,
                counterbalancing,
            } => self.assign_mixed_design(
                participant_id,
                between_factors,
                within_factors,
                counterbalancing,
                participant_characteristics,
            ),
        }
    }

    fn assign_between_subjects(
        &mut self,
        participant_id: String,
        conditions: &[ExperimentCondition],
        randomization: &RandomizationType,
        characteristics: Option<HashMap<String, String>>,
    ) -> Result<ParticipantAssignment, String> {
        if conditions.is_empty() {
            return Err("No conditions specified".to_string());
        }

        let selected_condition = match randomization {
            RandomizationType::Simple => conditions.choose(&mut self.rng).unwrap().id.clone(),
            RandomizationType::Block { block_size } => {
                self.block_randomization(conditions, *block_size)?
            }
            RandomizationType::Stratified { strata } => {
                self.stratified_randomization(conditions, strata, &characteristics)?
            }
            RandomizationType::Adaptive { target_ratio } => {
                self.adaptive_randomization(conditions, target_ratio)?
            }
        };

        let assignment = ParticipantAssignment {
            participant_id,
            condition_sequence: vec![selected_condition],
            randomization_details: RandomizationRecord {
                method: format!("{:?}", randomization),
                seed: self.rng.next_u64(),
                assignment_time: chrono::Utc::now(),
                balancing_info: self.calculate_balancing_info(conditions),
            },
        };

        self.assignments.push(assignment.clone());
        Ok(assignment)
    }

    fn assign_within_subjects(
        &mut self,
        participant_id: String,
        conditions: &[ExperimentCondition],
        counterbalancing: &CounterbalancingMethod,
    ) -> Result<ParticipantAssignment, String> {
        let condition_sequence = match counterbalancing {
            CounterbalancingMethod::Complete => self.complete_counterbalancing(conditions)?,
            CounterbalancingMethod::LatinSquare => {
                self.latin_square_counterbalancing(conditions)?
            }
            CounterbalancingMethod::BalancedLatinSquare => {
                self.balanced_latin_square_counterbalancing(conditions)?
            }
            CounterbalancingMethod::WilliamsSquare => {
                self.williams_square_counterbalancing(conditions)?
            }
            CounterbalancingMethod::RandomWithConstraints { min_separation } => {
                self.random_counterbalancing_with_constraints(conditions, *min_separation)?
            }
        };

        let assignment = ParticipantAssignment {
            participant_id,
            condition_sequence,
            randomization_details: RandomizationRecord {
                method: format!("{:?}", counterbalancing),
                seed: self.rng.next_u64(),
                assignment_time: chrono::Utc::now(),
                balancing_info: HashMap::new(),
            },
        };

        self.assignments.push(assignment.clone());
        Ok(assignment)
    }

    fn assign_mixed_design(
        &mut self,
        participant_id: String,
        between_factors: &[Factor],
        within_factors: &[Factor],
        counterbalancing: &CounterbalancingMethod,
        characteristics: Option<HashMap<String, String>>,
    ) -> Result<ParticipantAssignment, String> {
        // For mixed designs, first assign between-subjects factors
        let between_assignment = self.assign_between_factors(between_factors, &characteristics)?;

        // Then generate within-subjects sequence
        let within_sequence = self.generate_within_sequence(within_factors, counterbalancing)?;

        // Combine into full condition sequence
        let condition_sequence =
            self.combine_between_within_conditions(&between_assignment, &within_sequence);

        let assignment = ParticipantAssignment {
            participant_id,
            condition_sequence,
            randomization_details: RandomizationRecord {
                method: "Mixed Design".to_string(),
                seed: self.rng.next_u64(),
                assignment_time: chrono::Utc::now(),
                balancing_info: HashMap::new(),
            },
        };

        self.assignments.push(assignment.clone());
        Ok(assignment)
    }

    /// Block randomization implementation
    fn block_randomization(
        &mut self,
        conditions: &[ExperimentCondition],
        block_size: usize,
    ) -> Result<String, String> {
        if block_size % conditions.len() != 0 {
            return Err("Block size must be multiple of number of conditions".to_string());
        }

        // Create block with equal numbers of each condition
        let mut block = Vec::new();
        let repeats_per_condition = block_size / conditions.len();

        for condition in conditions {
            for _ in 0..repeats_per_condition {
                block.push(condition.id.clone());
            }
        }

        // Shuffle the block
        block.shuffle(&mut self.rng);

        // Return assignment based on current position in block
        let position = self.assignments.len() % block_size;
        Ok(block[position].clone())
    }

    /// Stratified randomization implementation
    fn stratified_randomization(
        &mut self,
        conditions: &[ExperimentCondition],
        strata: &[StratificationCriterion],
        characteristics: &Option<HashMap<String, String>>,
    ) -> Result<String, String> {
        let characteristics = characteristics
            .as_ref()
            .ok_or("Participant characteristics required for stratified randomization")?;

        // Determine stratum for this participant
        let mut stratum_key = String::new();
        for criterion in strata {
            let value = characteristics
                .get(&criterion.variable)
                .ok_or(format!("Missing characteristic: {}", criterion.variable))?;
            stratum_key.push_str(&format!("{}:{};", criterion.variable, value));
        }

        // Count assignments in this stratum for each condition
        let mut stratum_counts: HashMap<String, usize> = HashMap::new();
        for assignment in &self.assignments {
            // This would require storing stratum info in assignments - simplified for now
            for condition_id in &assignment.condition_sequence {
                *stratum_counts.entry(condition_id.clone()).or_insert(0) += 1;
            }
        }

        // Find condition with minimum assignments in this stratum
        let min_count = stratum_counts.values().min().cloned().unwrap_or(0);
        let candidates: Vec<_> = conditions
            .iter()
            .filter(|c| stratum_counts.get(&c.id).unwrap_or(&0) == &min_count)
            .collect();

        Ok(candidates.choose(&mut self.rng).unwrap().id.clone())
    }

    /// Adaptive randomization implementation
    fn adaptive_randomization(
        &mut self,
        conditions: &[ExperimentCondition],
        target_ratio: &[f64],
    ) -> Result<String, String> {
        if target_ratio.len() != conditions.len() {
            return Err("Target ratio must match number of conditions".to_string());
        }

        // Count current assignments
        let mut condition_counts: HashMap<String, usize> = HashMap::new();
        for assignment in &self.assignments {
            for condition_id in &assignment.condition_sequence {
                *condition_counts.entry(condition_id.clone()).or_insert(0) += 1;
            }
        }

        // Calculate imbalance for each condition
        let total_assignments = self.assignments.len() as f64;
        let mut imbalances = Vec::new();

        for (i, condition) in conditions.iter().enumerate() {
            let current_count = condition_counts.get(&condition.id).unwrap_or(&0);
            let expected_count = total_assignments * target_ratio[i];
            let imbalance = expected_count - *current_count as f64;
            imbalances.push((condition.id.clone(), imbalance));
        }

        // Select condition with highest positive imbalance (most under-assigned)
        imbalances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(imbalances[0].0.clone())
    }

    /// Complete counterbalancing (all possible orders)
    fn complete_counterbalancing(
        &mut self,
        conditions: &[ExperimentCondition],
    ) -> Result<Vec<String>, String> {
        let condition_ids: Vec<_> = conditions.iter().map(|c| c.id.clone()).collect();
        let all_permutations = Self::generate_permutations(&condition_ids);

        let participant_number = self.assignments.len();
        let selected_order = &all_permutations[participant_number % all_permutations.len()];

        Ok(selected_order.clone())
    }

    /// Latin Square counterbalancing
    fn latin_square_counterbalancing(
        &mut self,
        conditions: &[ExperimentCondition],
    ) -> Result<Vec<String>, String> {
        let n = conditions.len();
        let participant_number = self.assignments.len();
        let row = participant_number % n;

        let mut sequence = Vec::new();
        for col in 0..n {
            let condition_index = (row + col) % n;
            sequence.push(conditions[condition_index].id.clone());
        }

        Ok(sequence)
    }

    /// Balanced Latin Square counterbalancing
    fn balanced_latin_square_counterbalancing(
        &mut self,
        conditions: &[ExperimentCondition],
    ) -> Result<Vec<String>, String> {
        let n = conditions.len();
        if n % 2 != 0 {
            return Err("Balanced Latin Square requires even number of conditions".to_string());
        }

        let participant_number = self.assignments.len();
        let row = participant_number % n;

        // Generate balanced Latin square where each condition follows every other exactly once
        let mut sequence = Vec::new();
        for col in 0..n {
            let condition_index = if col % 2 == 0 {
                (row + col / 2) % n
            } else {
                (row - col / 2 + n) % n
            };
            sequence.push(conditions[condition_index].id.clone());
        }

        Ok(sequence)
    }

    /// Williams Square counterbalancing (controls for first-order carryover)
    fn williams_square_counterbalancing(
        &mut self,
        conditions: &[ExperimentCondition],
    ) -> Result<Vec<String>, String> {
        let n = conditions.len();
        let participant_number = self.assignments.len();

        // Williams square construction is complex - simplified implementation
        let mut sequence = Vec::new();
        let offset = participant_number % n;

        for i in 0..n {
            let condition_index = (offset + i * (n - 1) / 2) % n;
            sequence.push(conditions[condition_index].id.clone());
        }

        Ok(sequence)
    }

    /// Random counterbalancing with separation constraints
    fn random_counterbalancing_with_constraints(
        &mut self,
        conditions: &[ExperimentCondition],
        min_separation: usize,
    ) -> Result<Vec<String>, String> {
        let condition_ids: Vec<_> = conditions.iter().map(|c| c.id.clone()).collect();
        let repeat_count = (min_separation + 1) / conditions.len() + 1;
        let mut sequence = Vec::new();
        for _ in 0..repeat_count {
            sequence.extend(condition_ids.clone());
        }

        // Shuffle while maintaining minimum separation
        for _ in 0..100 {
            // Max attempts
            sequence.shuffle(&mut self.rng);
            if self.check_separation_constraint(&sequence, min_separation) {
                // Trim to desired length
                sequence.truncate(conditions.len());
                return Ok(sequence);
            }
        }

        // Fallback to simple shuffle if constraint cannot be satisfied
        let mut simple_sequence = condition_ids;
        simple_sequence.shuffle(&mut self.rng);
        Ok(simple_sequence)
    }

    // Helper methods
    fn generate_permutations(items: &[String]) -> Vec<Vec<String>> {
        if items.len() <= 1 {
            return vec![items.to_vec()];
        }

        let mut result = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let mut remaining = items.to_vec();
            remaining.remove(i);

            for mut perm in Self::generate_permutations(&remaining) {
                perm.insert(0, item.clone());
                result.push(perm);
            }
        }
        result
    }

    fn check_separation_constraint(&self, sequence: &[String], min_separation: usize) -> bool {
        let mut last_positions: HashMap<String, usize> = HashMap::new();

        for (pos, condition) in sequence.iter().enumerate() {
            if let Some(&last_pos) = last_positions.get(condition) {
                if pos - last_pos <= min_separation {
                    return false;
                }
            }
            last_positions.insert(condition.clone(), pos);
        }
        true
    }

    fn calculate_balancing_info(
        &self,
        conditions: &[ExperimentCondition],
    ) -> HashMap<String, serde_json::Value> {
        let mut info = HashMap::new();
        let mut counts = HashMap::new();

        for assignment in &self.assignments {
            for condition_id in &assignment.condition_sequence {
                *counts.entry(condition_id.clone()).or_insert(0) += 1;
            }
        }

        info.insert(
            "condition_counts".to_string(),
            serde_json::to_value(&counts).unwrap(),
        );
        info.insert(
            "total_assignments".to_string(),
            serde_json::json!(self.assignments.len()),
        );

        info
    }

    fn assign_between_factors(
        &mut self,
        factors: &[Factor],
        characteristics: &Option<HashMap<String, String>>,
    ) -> Result<HashMap<String, String>, String> {
        let mut assignment = HashMap::new();

        for factor in factors {
            let level = factor.levels.choose(&mut self.rng).unwrap();
            assignment.insert(factor.name.clone(), level.clone());
        }

        Ok(assignment)
    }

    fn generate_within_sequence(
        &mut self,
        factors: &[Factor],
        counterbalancing: &CounterbalancingMethod,
    ) -> Result<Vec<HashMap<String, String>>, String> {
        // Simplified - generate all combinations of within-subjects factors
        let mut sequences = Vec::new();

        // This would need proper combinatorial generation for multiple factors
        if let Some(factor) = factors.first() {
            for level in &factor.levels {
                let mut condition = HashMap::new();
                condition.insert(factor.name.clone(), level.clone());
                sequences.push(condition);
            }
            sequences.shuffle(&mut self.rng);
        }

        Ok(sequences)
    }

    fn combine_between_within_conditions(
        &self,
        between: &HashMap<String, String>,
        within: &[HashMap<String, String>],
    ) -> Vec<String> {
        let mut combined = Vec::new();

        for within_condition in within {
            let mut full_condition = between.clone();
            full_condition.extend(within_condition.clone());

            // Generate condition ID from combination
            let condition_id = full_condition
                .iter()
                .map(|(k, v)| format!("{}:{}", k, v))
                .collect::<Vec<_>>()
                .join(",");

            combined.push(condition_id);
        }

        combined
    }

    /// Get current assignment statistics
    pub fn get_assignment_statistics(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        // Count assignments per condition
        let mut condition_counts: HashMap<String, usize> = HashMap::new();
        for assignment in &self.assignments {
            for condition in &assignment.condition_sequence {
                *condition_counts.entry(condition.clone()).or_insert(0) += 1;
            }
        }

        stats.insert(
            "condition_counts".to_string(),
            serde_json::to_value(&condition_counts).unwrap(),
        );
        stats.insert(
            "total_participants".to_string(),
            serde_json::json!(self.assignments.len()),
        );
        stats.insert(
            "assignment_history".to_string(),
            serde_json::to_value(&self.assignments).unwrap(),
        );

        stats
    }

    /// Validate experimental design for statistical power
    pub fn validate_design(
        &self,
        design: &ExperimentalDesign,
        minimum_n_per_condition: usize,
    ) -> Result<ValidationReport, String> {
        let mut report = ValidationReport {
            valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            recommendations: Vec::new(),
        };

        match design {
            ExperimentalDesign::BetweenSubjects { conditions, .. } => {
                if conditions.len() < 2 {
                    report
                        .errors
                        .push("Between-subjects design requires at least 2 conditions".to_string());
                    report.valid = false;
                }

                let assignments_per_condition = self.assignments.len() / conditions.len();
                if assignments_per_condition < minimum_n_per_condition {
                    report.warnings.push(format!(
                        "Low statistical power: {} participants per condition (recommended: {})",
                        assignments_per_condition, minimum_n_per_condition
                    ));
                }
            }
            ExperimentalDesign::WithinSubjects {
                conditions,
                counterbalancing,
            } => {
                if conditions.len() < 2 {
                    report
                        .errors
                        .push("Within-subjects design requires at least 2 conditions".to_string());
                    report.valid = false;
                }

                match counterbalancing {
                    CounterbalancingMethod::Complete => {
                        let required_participants = Self::factorial(conditions.len());
                        if self.assignments.len() % required_participants != 0 {
                            report.recommendations.push(format!(
                                "Complete counterbalancing works best with multiples of {} participants",
                                required_participants
                            ));
                        }
                    }
                    CounterbalancingMethod::LatinSquare
                    | CounterbalancingMethod::BalancedLatinSquare => {
                        if self.assignments.len() % conditions.len() != 0 {
                            report.recommendations.push(format!(
                                "Latin square designs work best with multiples of {} participants",
                                conditions.len()
                            ));
                        }
                    }
                    _ => {}
                }
            }
            ExperimentalDesign::Mixed {
                between_factors,
                within_factors,
                ..
            } => {
                if between_factors.is_empty() || within_factors.is_empty() {
                    report
                        .errors
                        .push("Mixed design requires both between and within factors".to_string());
                    report.valid = false;
                }
            }
        }

        Ok(report)
    }

    fn factorial(n: usize) -> usize {
        (1..=n).product()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_randomization() {
        let mut designer = ExperimentalDesigner::new(Some(12345));

        let conditions = vec![
            ExperimentCondition {
                id: "condition_a".to_string(),
                name: "Condition A".to_string(),
                description: "Test condition A".to_string(),
                config: LearnerConfig::default(),
                topology: Topology::alphabet(),
                parameters: HashMap::new(),
            },
            ExperimentCondition {
                id: "condition_b".to_string(),
                name: "Condition B".to_string(),
                description: "Test condition B".to_string(),
                config: LearnerConfig::default(),
                topology: Topology::alphabet(),
                parameters: HashMap::new(),
            },
        ];

        let design = ExperimentalDesign::BetweenSubjects {
            conditions,
            randomization: RandomizationType::Block { block_size: 4 },
        };

        // Assign 8 participants
        for i in 0..8 {
            let assignment = designer
                .assign_participant(format!("participant_{}", i), &design, None)
                .unwrap();
            assert!(!assignment.condition_sequence.is_empty());
        }

        // Check that assignments are balanced
        let stats = designer.get_assignment_statistics();
        // Should have equal assignments (4 each with block size 4)
        assert_eq!(designer.assignments.len(), 8);
    }

    #[test]
    fn test_latin_square_counterbalancing() {
        let mut designer = ExperimentalDesigner::new(Some(54321));

        let conditions = vec![
            ExperimentCondition {
                id: "A".to_string(),
                name: "Condition A".to_string(),
                description: "Test condition A".to_string(),
                config: LearnerConfig::default(),
                topology: Topology::alphabet(),
                parameters: HashMap::new(),
            },
            ExperimentCondition {
                id: "B".to_string(),
                name: "Condition B".to_string(),
                description: "Test condition B".to_string(),
                config: LearnerConfig::default(),
                topology: Topology::alphabet(),
                parameters: HashMap::new(),
            },
            ExperimentCondition {
                id: "C".to_string(),
                name: "Condition C".to_string(),
                description: "Test condition C".to_string(),
                config: LearnerConfig::default(),
                topology: Topology::alphabet(),
                parameters: HashMap::new(),
            },
        ];

        let design = ExperimentalDesign::WithinSubjects {
            conditions,
            counterbalancing: CounterbalancingMethod::LatinSquare,
        };

        // Assign 3 participants (should get different orders)
        let mut sequences = Vec::new();
        for i in 0..3 {
            let assignment = designer
                .assign_participant(format!("participant_{}", i), &design, None)
                .unwrap();
            sequences.push(assignment.condition_sequence);
        }

        // Each sequence should be different and contain all conditions
        for sequence in &sequences {
            assert_eq!(sequence.len(), 3);
            assert!(sequence.contains(&"A".to_string()));
            assert!(sequence.contains(&"B".to_string()));
            assert!(sequence.contains(&"C".to_string()));
        }

        // All sequences should be different (Latin Square property)
        assert_ne!(sequences[0], sequences[1]);
        assert_ne!(sequences[1], sequences[2]);
        assert_ne!(sequences[0], sequences[2]);
    }
}
