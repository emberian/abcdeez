//! # Experimental Design & Execution
//! 
//! This module provides comprehensive experimental design patterns and execution framework:
//! 
//! - [`ExperimentalDesigner`]: Creates counterbalanced experimental designs
//! - [`ExperimentFramework`]: Runs complete experiments with participants
//! - [`CounterbalancingMethod`]: Various counterbalancing strategies
//! - [`RandomizationType`]: Randomization approaches for participant assignment
//! 
//! ## Design Types
//! 
//! - **Between-Subjects**: Random assignment to conditions
//! - **Within-Subjects**: All participants experience all conditions  
//! - **Mixed Design**: Combination of between and within factors
//! 
//! ## Counterbalancing Methods
//! 
//! - **Latin Square**: Each condition appears in each position once
//! - **Williams Square**: Controls for first-order carryover effects
//! - **Balanced Latin Square**: Each condition follows every other exactly once
//! - **Random with Constraints**: Randomized with minimum separation
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::experimental::{ExperimentalDesigner, ExperimentalDesign, CounterbalancingMethod};
//! 
//! let mut designer = ExperimentalDesigner::new(Some(42)); // Seeded for reproducibility
//! 
//! let design = ExperimentalDesign::WithinSubjects {
//!     conditions: vec![/* your conditions */],
//!     counterbalancing: CounterbalancingMethod::LatinSquare,
//! };
//! 
//! let assignment = designer.assign_participant("P001".to_string(), &design, None)?;
//! ```

pub mod design;
pub mod framework;

// Re-export main types
pub use design::{
    ExperimentalDesign, ExperimentalDesigner, CounterbalancingMethod, RandomizationType,
    ExperimentCondition, ParticipantAssignment, ValidationReport
};
pub use framework::{
    ExperimentFramework, Experiment, ExperimentConfig, ExperimentResults, 
    ExperimentTemplates
};