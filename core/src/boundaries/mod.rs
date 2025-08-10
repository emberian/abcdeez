//! # Boundary Detection & Crossing Training
//! 
//! This module handles cognitive boundaries and chunking in sequential learning:
//! 
//! - [`BoundaryTrainer`]: Generates tasks that specifically train boundary crossing
//! - [`BoundaryType`]: Different types of cognitive boundaries  
//! - [`HierarchicalChunker`]: Multi-level chunking for complex boundaries
//! - [`BoundaryBridgingTask`]: Tasks designed to cross specific boundaries
//! 
//! ## Boundary Types
//! 
//! - **ChunkBoundary**: Mental segmentation boundaries (e.g., F-G in alphabet)
//! - **CategoryBoundary**: Category transitions (vowel→consonant)
//! - **HierarchicalBoundary**: Multi-level boundaries with different strengths
//! - **OctaveBoundary**: Musical octave boundaries (B→C)
//! - **ModuleBoundary**: Programming/mathematical module boundaries
//! 
//! ## Key Concepts
//! 
//! Boundary crossing is a critical aspect of sequential learning where learners
//! must bridge between mental "chunks" or categories. This module provides
//! sophisticated tools for:
//! 
//! - Detecting natural boundaries in learning sequences
//! - Generating training tasks that specifically target boundary crossing
//! - Measuring the cognitive cost of crossing boundaries
//! - Hierarchical chunking at multiple levels of abstraction
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::boundaries::{BoundaryTrainer, BoundaryType};
//! use abcdeez_core::core::Topology;
//! 
//! let topology = Topology::alphabet();
//! let trainer = BoundaryTrainer::new(topology);
//! 
//! // Generate a task that crosses chunk boundaries
//! let task = trainer.generate_boundary_bridging_task(BoundaryType::ChunkBoundary);
//! 
//! // Generate a task that requires navigating between boundaries
//! let nav_task = trainer.generate_boundary_navigation_task();
//! ```

pub mod training;

// Re-export main types
pub use training::{
    BoundaryTrainer, BoundaryType, BoundaryBridgingTask, 
    HierarchicalChunker, ChunkLevel
};