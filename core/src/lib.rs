//! # AbcDeez Core Learning Library
//! 
//! A sophisticated adaptive learning research platform with Bayesian inference,
//! Expected Information Gain task selection, and comprehensive experimental design.
//!
//! ## Quick Start
//!
//! ```rust
//! use abcdeez_core::prelude::*;
//!
//! // Create a learning topology
//! let topology = Topology::alphabet();
//!
//! // Create a learner model
//! let learner = LearnerModel::new("learner_001", &topology);
//!
//! // Create an adaptive scheduler
//! let scheduler = AdaptiveScheduler::new(learner, topology);
//!
//! // Generate and run learning tasks
//! let task = scheduler.select_next_task();
//! ```

// Core learning engine
pub mod core;

// Task generation & management
pub mod tasks;

// Experimental design & execution
pub mod experimental;

// Boundary detection & training
pub mod boundaries;

// Statistical analysis & validation
pub mod statistics;

// Real-time interventions
pub mod intervention;

// Configuration management
pub mod config;

// Data persistence & export
pub mod data;

// Research infrastructure
pub mod research;

// Advanced analytics
pub mod analysis;

// Domain implementations
pub mod domains;

// Interaction tracking
pub mod interaction;

// Performance optimization
pub mod optimization;

// Session management
pub mod session;

// A/B testing
pub mod testing;

// User interfaces
pub mod ui;

// Global types
pub mod error;

// Remaining standalone modules (to be organized later)
pub mod backend;
pub mod demo;

// Test modules
#[cfg(test)]
mod tests;

// Re-export commonly used types at the root level
pub use core::{
    AdaptiveScheduler, LearnerModel, LearnerMetrics, BayesianLearnerModel, 
    Topology, TopologyType, OperationType
};
pub use tasks::{Task, TaskGenerator, TaskType, TaskSession};
pub use experimental::{
    ExperimentFramework, ExperimentalDesigner, CounterbalancingMethod, 
    RandomizationType, ExperimentalDesign
};
pub use config::{LearnerConfig, SystemConfig, PopulationType, DomainType, LearningGoal};
pub use intervention::{
    InterventionSystem, StruggleDetector, HintGenerator, HintLevel, 
    StruggleLevel, InterventionAction
};
pub use statistics::{
    StatisticalValidator, PowerAnalyzer, MixedEffectsAnalyzer, 
    ResponseTimeDistribution
};
pub use data::{LearnerDataExport, PopulationAnalyzer};
pub use error::{Error, Result};

// Version info
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        AdaptiveScheduler, LearnerModel, LearnerMetrics, BayesianLearnerModel,
        Task, TaskGenerator, TaskType, Topology, TopologyType,
        ExperimentFramework, SystemConfig, InterventionSystem,
        Error, Result
    };
}