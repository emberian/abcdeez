//! # Core Learning Engine
//! 
//! This module contains the core adaptive learning algorithms including:
//! 
//! - [`LearnerModel`]: Individual learner state and proficiency tracking
//! - [`BayesianLearnerModel`]: Bayesian inference for learning parameters  
//! - [`AdaptiveScheduler`]: Task selection with Expected Information Gain
//! - [`Topology`]: Abstract representations of learning domains
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::core::{LearnerModel, Topology};
//! 
//! let topology = Topology::alphabet();
//! let learner = LearnerModel::new("learner_001", &topology);
//! ```

pub mod learner;
pub mod bayesian; 
pub mod adaptive;
pub mod topology;
pub mod hierarchical;
pub mod macro_learning;
pub mod strategy_mixture;

// Re-export main types for convenience
pub use learner::{LearnerModel, LearnerMetrics, OperationType, LatentNodeEmbedding, ChunkBoundary};
pub use bayesian::{BayesianLearnerModel, ResponseData, PosteriorDistribution};
pub use adaptive::AdaptiveScheduler;
pub use topology::{Topology, TopologyType, Node, Edge};