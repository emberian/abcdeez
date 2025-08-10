//! # Task Generation & Management
//! 
//! This module handles the generation and management of learning tasks:
//! 
//! - [`Task`]: Individual learning task representation
//! - [`TaskGenerator`]: Generates tasks based on topology and difficulty
//! - [`TaskType`]: Different types of learning tasks (successor, segment, etc.)
//! - [`TaskSession`]: Session management for task sequences
//! 
//! ## Task Types
//! 
//! - **Successor/Predecessor**: Next/previous item in sequence
//! - **K-Jump**: Skip k positions forward/backward  
//! - **Segment**: Generate sequence of consecutive items
//! - **PairwiseOrder**: Compare relative positions of two items
//! - **ShortestPath**: Navigate between distant items
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::tasks::{TaskGenerator, TaskType};
//! use abcdeez_core::core::Topology;
//! 
//! let topology = Topology::alphabet();
//! let mut generator = TaskGenerator::new(topology);
//! let task = generator.generate_task(Some(TaskType::Successor { 
//!     item: "M".to_string() 
//! }));
//! ```

pub mod types;
pub mod extended;

// Re-export from types for convenience  
pub use types::{Task, TaskGenerator, TaskType, TaskSession, TaskResponse};