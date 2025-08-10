use crate::tasks::types::{Task, TaskType};
use crate::core::topology::Topology;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StruggleDetector {
    pub rt_threshold_ms: u64,          // Time before considering struggle
    pub error_streak_threshold: usize, // Consecutive errors before intervention
    pub hint_delay_ms: u64,            // Time before offering hint
    pub adaptive_threshold: bool,      // Adjust thresholds based on user
    recent_rts: VecDeque<u64>,
    recent_errors: VecDeque<bool>,
    baseline_rt: Option<f64>,
}

impl StruggleDetector {
    pub fn new() -> Self {
        StruggleDetector {
            rt_threshold_ms: 5000,
            error_streak_threshold: 3,
            hint_delay_ms: 3000,
            adaptive_threshold: true,
            recent_rts: VecDeque::with_capacity(20),
            recent_errors: VecDeque::with_capacity(10),
            baseline_rt: None,
        }
    }

    pub fn update(&mut self, response_time_ms: u64, correct: bool) {
        // Update recent history
        self.recent_rts.push_back(response_time_ms);
        if self.recent_rts.len() > 20 {
            self.recent_rts.pop_front();
        }

        self.recent_errors.push_back(!correct);
        if self.recent_errors.len() > 10 {
            self.recent_errors.pop_front();
        }

        // Update baseline if adaptive
        if self.adaptive_threshold && self.recent_rts.len() >= 5 {
            let sum: u64 = self.recent_rts.iter().sum();
            self.baseline_rt = Some(sum as f64 / self.recent_rts.len() as f64);

            // Adjust threshold to 2x baseline
            if let Some(baseline) = self.baseline_rt {
                self.rt_threshold_ms = (baseline * 2.0) as u64;
            }
        }
    }

    pub fn is_struggling(&self, current_rt_ms: u64) -> bool {
        // Check response time
        let rt_struggle = current_rt_ms > self.rt_threshold_ms;

        // Check error streak
        let consecutive_errors = self.recent_errors.iter().rev().take_while(|&&e| e).count();
        let error_struggle = consecutive_errors >= self.error_streak_threshold;

        rt_struggle || error_struggle
    }

    pub fn should_provide_hint(&self, elapsed_ms: u64) -> bool {
        elapsed_ms > self.hint_delay_ms && self.is_struggling(elapsed_ms)
    }

    pub fn should_reduce_difficulty(&self) -> bool {
        // Check if error rate is too high
        let error_count = self.recent_errors.iter().filter(|&&e| e).count();
        let error_rate = error_count as f64 / self.recent_errors.len().max(1) as f64;

        error_rate > 0.5 // More than 50% errors
    }

    pub fn should_increase_difficulty(&self) -> bool {
        // Check if performance is too good
        let error_count = self.recent_errors.iter().filter(|&&e| e).count();
        let error_rate = error_count as f64 / self.recent_errors.len().max(1) as f64;

        // Also check if RTs are very fast
        let avg_rt = if !self.recent_rts.is_empty() {
            self.recent_rts.iter().sum::<u64>() as f64 / self.recent_rts.len() as f64
        } else {
            f64::MAX
        };

        error_rate < 0.1 && avg_rt < 2000.0 // Less than 10% errors and fast responses
    }

    pub fn get_struggle_level(&self) -> StruggleLevel {
        let consecutive_errors = self.recent_errors.iter().rev().take_while(|&&e| e).count();

        let avg_rt = if !self.recent_rts.is_empty() {
            self.recent_rts.iter().sum::<u64>() as f64 / self.recent_rts.len() as f64
        } else {
            0.0
        };

        let rt_ratio = if let Some(baseline) = self.baseline_rt {
            avg_rt / baseline
        } else {
            avg_rt / 3000.0 // Default baseline
        };

        if consecutive_errors >= 5 || rt_ratio > 3.0 {
            StruggleLevel::Severe
        } else if consecutive_errors >= 3 || rt_ratio > 2.0 {
            StruggleLevel::Moderate
        } else if consecutive_errors >= 1 || rt_ratio > 1.5 {
            StruggleLevel::Mild
        } else {
            StruggleLevel::None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StruggleLevel {
    None,
    Mild,
    Moderate,
    Severe,
}

#[derive(Debug, Clone)]
pub struct HintGenerator {
    topology: Topology,
}

impl HintGenerator {
    pub fn new(topology: Topology) -> Self {
        HintGenerator { topology }
    }

    pub fn generate_hint(&self, task: &Task, level: HintLevel) -> String {
        match level {
            HintLevel::Confirmation => self.generate_confirmation_hint(task),
            HintLevel::Partial => self.generate_partial_hint(task),
            HintLevel::Scaffold => self.generate_scaffold_hint(task),
            HintLevel::Worked => self.generate_worked_example(task),
        }
    }

    fn generate_confirmation_hint(&self, task: &Task) -> String {
        match &task.task_type {
            TaskType::Successor { item } => {
                format!("Think about what comes directly after '{}'", item)
            }
            TaskType::Predecessor { item } => {
                format!("Think about what comes directly before '{}'", item)
            }
            TaskType::PairwiseOrder { a, b } => {
                format!("Consider the positions of '{}' and '{}'", a, b)
            }
            TaskType::Segment {
                start,
                count,
                reverse,
            } => {
                if *reverse {
                    format!("Start from '{}' and go backward {} steps", start, count)
                } else {
                    format!("Start from '{}' and go forward {} steps", start, count)
                }
            }
            TaskType::KJump { start, k } => {
                if *k > 0 {
                    format!("Count {} positions forward from '{}'", k, start)
                } else {
                    format!("Count {} positions backward from '{}'", k.abs(), start)
                }
            }
            _ => "Take your time and think through the structure".to_string(),
        }
    }

    fn generate_partial_hint(&self, task: &Task) -> String {
        match &task.task_type {
            TaskType::Successor { item } => {
                if let Some(node) = self.topology.get_node_by_label(item) {
                    let position = node.position as usize;
                    format!(
                        "'{}' is at position {}. What's at position {}?",
                        item,
                        position + 1,
                        position + 2
                    )
                } else {
                    self.generate_confirmation_hint(task)
                }
            }
            TaskType::PairwiseOrder { a, b } => {
                if let (Some(node_a), Some(node_b)) = (
                    self.topology.get_node_by_label(a),
                    self.topology.get_node_by_label(b),
                ) {
                    let hint = if node_a.position < node_b.position {
                        format!("'{}' appears earlier in the sequence", a)
                    } else {
                        format!("'{}' appears earlier in the sequence", b)
                    };
                    hint
                } else {
                    self.generate_confirmation_hint(task)
                }
            }
            TaskType::Segment { start, count, .. } => {
                let partial = self.topology.get_segment(start, (*count).min(2), false);
                format!("The sequence starts with: {}...", partial.join(", "))
            }
            _ => self.generate_confirmation_hint(task),
        }
    }

    fn generate_scaffold_hint(&self, task: &Task) -> String {
        match &task.task_type {
            TaskType::Successor { item } => {
                let mut hint = format!("To find what comes after '{}':\n", item);
                hint.push_str("1. Locate '{}' in the sequence\n");
                hint.push_str("2. Move one position forward\n");
                hint.push_str("3. Identify the item at that position");
                hint
            }
            TaskType::PairwiseOrder { a, b } => {
                let mut hint = "To determine order:\n".to_string();
                hint.push_str(&format!("1. Find the position of '{}'\n", a));
                hint.push_str(&format!("2. Find the position of '{}'\n", b));
                hint.push_str("3. Compare which position is smaller");
                hint
            }
            TaskType::Segment {
                start,
                count,
                reverse,
            } => {
                let mut hint = format!("To list {} items from '{}':\n", count, start);
                hint.push_str(&format!("1. Start at '{}'\n", start));
                if *reverse {
                    hint.push_str("2. Move backward one step at a time\n");
                } else {
                    hint.push_str("2. Move forward one step at a time\n");
                }
                hint.push_str(&format!("3. Collect {} items total", count));
                hint
            }
            _ => self.generate_partial_hint(task),
        }
    }

    fn generate_worked_example(&self, task: &Task) -> String {
        let answer = &task.correct_answer;

        match &task.task_type {
            TaskType::Successor { item } => {
                format!(
                    "Let me work through this:\n\
                    Looking for what comes after '{}'.\n\
                    In the alphabet, '{}' is followed by '{}'.\n\
                    Answer: {}",
                    item, item, answer, answer
                )
            }
            TaskType::PairwiseOrder { a, b } => {
                let (first, second) = if answer.contains("before") || answer == "Yes" {
                    (a, b)
                } else {
                    (b, a)
                };
                format!(
                    "Let me work through this:\n\
                    Comparing '{}' and '{}'.\n\
                    '{}' comes before '{}' in the sequence.\n\
                    Answer: {}",
                    a, b, first, second, answer
                )
            }
            TaskType::Segment {
                start,
                count,
                reverse,
            } => {
                let direction = if *reverse { "backward" } else { "forward" };
                format!(
                    "Let me work through this:\n\
                    Starting from '{}', going {} for {} items:\n\
                    {}",
                    start, direction, count, answer
                )
            }
            _ => format!("The answer is: {}", answer),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HintLevel {
    Confirmation, // Confirm/deny approach
    Partial,      // Partial information
    Scaffold,     // Step-by-step guidance
    Worked,       // Full worked example
}

// Adaptive difficulty adjustment
pub struct DifficultyAdapter {
    current_difficulty: f64,
    min_difficulty: f64,
    max_difficulty: f64,
    adjustment_rate: f64,
    performance_window: VecDeque<bool>,
    target_success_rate: f64,
}

impl DifficultyAdapter {
    pub fn new() -> Self {
        DifficultyAdapter {
            current_difficulty: 0.5,
            min_difficulty: 0.1,
            max_difficulty: 0.9,
            adjustment_rate: 0.05,
            performance_window: VecDeque::with_capacity(10),
            target_success_rate: 0.75, // 75% success rate
        }
    }

    pub fn update(&mut self, correct: bool) {
        self.performance_window.push_back(correct);
        if self.performance_window.len() > 10 {
            self.performance_window.pop_front();
        }

        // Adjust difficulty based on recent performance
        if self.performance_window.len() >= 5 {
            let success_count = self.performance_window.iter().filter(|&&c| c).count();
            let success_rate = success_count as f64 / self.performance_window.len() as f64;

            if success_rate > self.target_success_rate + 0.1 {
                // Too easy, increase difficulty
                self.increase_difficulty();
            } else if success_rate < self.target_success_rate - 0.1 {
                // Too hard, decrease difficulty
                self.decrease_difficulty();
            }
        }
    }

    pub fn increase_difficulty(&mut self) {
        self.current_difficulty =
            (self.current_difficulty + self.adjustment_rate).min(self.max_difficulty);
    }

    pub fn decrease_difficulty(&mut self) {
        self.current_difficulty =
            (self.current_difficulty - self.adjustment_rate).max(self.min_difficulty);
    }

    pub fn get_current_difficulty(&self) -> f64 {
        self.current_difficulty
    }

    pub fn get_performance_summary(&self) -> PerformanceSummary {
        let success_count = self.performance_window.iter().filter(|&&c| c).count();
        let total = self.performance_window.len();
        let success_rate = if total > 0 {
            success_count as f64 / total as f64
        } else {
            0.0
        };

        PerformanceSummary {
            recent_success_rate: success_rate,
            current_difficulty: self.current_difficulty,
            adjustment_needed: (success_rate - self.target_success_rate).abs() > 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub recent_success_rate: f64,
    pub current_difficulty: f64,
    pub adjustment_needed: bool,
}

// Real-time intervention system
pub struct InterventionSystem {
    struggle_detector: StruggleDetector,
    hint_generator: HintGenerator,
    difficulty_adapter: DifficultyAdapter,
    intervention_history: Vec<Intervention>,
    session_start: Instant,
}

#[derive(Debug, Clone)]
pub struct Intervention {
    pub timestamp: Duration,
    pub intervention_type: InterventionType,
    pub task_id: String,
    pub effectiveness: Option<bool>, // Did it help?
}

#[derive(Debug, Clone)]
pub enum InterventionType {
    HintProvided(HintLevel),
    DifficultyReduced,
    DifficultyIncreased,
    BreakSuggested,
    TaskSkipped,
}

impl InterventionSystem {
    pub fn new(topology: Topology) -> Self {
        InterventionSystem {
            struggle_detector: StruggleDetector::new(),
            hint_generator: HintGenerator::new(topology),
            difficulty_adapter: DifficultyAdapter::new(),
            intervention_history: Vec::new(),
            session_start: Instant::now(),
        }
    }

    pub fn process_response(&mut self, _task: &Task, correct: bool, rt_ms: u64) {
        self.struggle_detector.update(rt_ms, correct);
        self.difficulty_adapter.update(correct);

        // Check if previous intervention was effective
        if let Some(last_intervention) = self.intervention_history.last_mut() {
            if last_intervention.effectiveness.is_none() {
                last_intervention.effectiveness = Some(correct);
            }
        }
    }

    pub fn check_intervention_needed(
        &mut self,
        task: &Task,
        elapsed_ms: u64,
    ) -> Option<InterventionAction> {
        let struggle_level = self.struggle_detector.get_struggle_level();

        match struggle_level {
            StruggleLevel::None => {
                // Check if we should increase difficulty
                if self
                    .difficulty_adapter
                    .get_performance_summary()
                    .recent_success_rate
                    > 0.9
                {
                    self.record_intervention(InterventionType::DifficultyIncreased, &task.prompt);
                    Some(InterventionAction::IncreaseDifficulty)
                } else {
                    None
                }
            }
            StruggleLevel::Mild => {
                if self.struggle_detector.should_provide_hint(elapsed_ms) {
                    let hint = self
                        .hint_generator
                        .generate_hint(task, HintLevel::Confirmation);
                    self.record_intervention(
                        InterventionType::HintProvided(HintLevel::Confirmation),
                        &task.prompt,
                    );
                    Some(InterventionAction::ProvideHint(hint))
                } else {
                    None
                }
            }
            StruggleLevel::Moderate => {
                let hint = self.hint_generator.generate_hint(task, HintLevel::Partial);
                self.record_intervention(
                    InterventionType::HintProvided(HintLevel::Partial),
                    &task.prompt,
                );
                Some(InterventionAction::ProvideHint(hint))
            }
            StruggleLevel::Severe => {
                // Multiple interventions for severe struggle
                if self.intervention_history.len() % 10 == 0 && self.intervention_history.len() > 0
                {
                    // Suggest break every 10 interventions
                    self.record_intervention(InterventionType::BreakSuggested, &task.prompt);
                    Some(InterventionAction::SuggestBreak)
                } else {
                    // Provide worked example and reduce difficulty
                    let hint = self.hint_generator.generate_hint(task, HintLevel::Worked);
                    self.record_intervention(
                        InterventionType::HintProvided(HintLevel::Worked),
                        &task.prompt,
                    );
                    self.difficulty_adapter.decrease_difficulty();
                    Some(InterventionAction::ProvideWorkedExample(hint))
                }
            }
        }
    }

    fn record_intervention(&mut self, intervention_type: InterventionType, task_id: &str) {
        self.intervention_history.push(Intervention {
            timestamp: self.session_start.elapsed(),
            intervention_type,
            task_id: task_id.to_string(),
            effectiveness: None,
        });
    }

    pub fn get_intervention_summary(&self) -> InterventionSummary {
        let total = self.intervention_history.len();
        let effective = self
            .intervention_history
            .iter()
            .filter(|i| i.effectiveness == Some(true))
            .count();

        let hints_given = self
            .intervention_history
            .iter()
            .filter(|i| matches!(i.intervention_type, InterventionType::HintProvided(_)))
            .count();

        InterventionSummary {
            total_interventions: total,
            effective_interventions: effective,
            hints_provided: hints_given,
            current_difficulty: self.difficulty_adapter.get_current_difficulty(),
            session_duration: self.session_start.elapsed(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterventionAction {
    ProvideHint(String),
    ProvideWorkedExample(String),
    IncreaseDifficulty,
    DecreaseDifficulty,
    SuggestBreak,
    SkipTask,
}

#[derive(Debug, Clone)]
pub struct InterventionSummary {
    pub total_interventions: usize,
    pub effective_interventions: usize,
    pub hints_provided: usize,
    pub current_difficulty: f64,
    pub session_duration: Duration,
}
