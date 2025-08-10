//! # Real-Time Intervention System
//! 
//! This module provides intelligent real-time interventions to support struggling learners:
//! 
//! - [`StruggleDetector`]: Detects when learners are struggling
//! - [`HintGenerator`]: Generates contextual hints at multiple levels  
//! - [`InterventionSystem`]: Orchestrates real-time interventions
//! - [`DifficultyAdapter`]: Dynamically adjusts task difficulty
//! 
//! ## Struggle Detection
//! 
//! Detects struggle through multiple signals:
//! - **Response Time**: Unusually long response times
//! - **Error Patterns**: Consecutive errors or high error rates  
//! - **Adaptive Thresholds**: User-specific baselines
//! - **Multi-level Assessment**: None → Mild → Moderate → Severe
//! 
//! ## Hint Levels
//! 
//! - **Confirmation**: Gentle guidance without giving away the answer
//! - **Partial**: Provides partial information or first steps
//! - **Scaffold**: Step-by-step guidance through the problem
//! - **Worked**: Complete worked example with explanation
//! 
//! ## Intervention Actions
//! 
//! - **Contextual Hints**: Task-specific assistance
//! - **Difficulty Adjustment**: Make tasks easier or harder  
//! - **Break Suggestions**: Recommend breaks during extended struggle
//! - **Task Skipping**: Skip tasks that are too difficult
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::intervention::{InterventionSystem, StruggleLevel};
//! use abcdeez_core::core::Topology;
//! 
//! let topology = Topology::alphabet();
//! let mut system = InterventionSystem::new(topology);
//! 
//! // Process a learner response
//! system.process_response(&task, correct, response_time_ms);
//! 
//! // Check if intervention is needed
//! if let Some(action) = system.check_intervention_needed(&task, elapsed_ms) {
//!     match action {
//!         InterventionAction::ProvideHint(hint) => println!("Hint: {}", hint),
//!         InterventionAction::DecreaseDifficulty => /* adjust difficulty */,
//!         _ => /* handle other interventions */,
//!     }
//! }
//! ```

pub mod hints;

// Re-export main types  
pub use hints::{
    StruggleDetector, StruggleLevel, HintGenerator, HintLevel,
    DifficultyAdapter, InterventionSystem, InterventionAction,
    InterventionType, InterventionSummary
};