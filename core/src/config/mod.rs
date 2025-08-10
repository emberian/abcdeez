//! # Configuration Management
//! 
//! This module provides comprehensive configuration systems for different populations,
//! domains, and experimental setups:
//! 
//! - [`LearnerConfig`]: Population-specific learning parameters
//! - [`DomainConfig`]: Domain-specific difficulty and boundary settings
//! - [`SystemConfig`]: Complete system configuration combining all aspects
//! - [`AdaptiveSchedulingConfig`]: Adaptive task selection parameters
//! 
//! ## Population Types
//! 
//! - **Adult**: Standard adult learner parameters
//! - **Child**: Higher learning rates, gentler error penalties
//! - **OlderAdult**: More conservative parameters, slower learning
//! - **Expert**: Lower uncertainty, extended ability ranges
//! - **LearningDisability**: Supportive parameters, no error penalties
//! 
//! ## Domain Configurations
//! 
//! - **Alphabet**: Letter sequence learning with vowel/consonant boundaries
//! - **Music**: Musical note sequences with octave boundaries  
//! - **Mathematics**: Number sequences with decade boundaries
//! 
//! ## Key Features
//! 
//! - **Population Adaptation**: Parameters tuned for different learner populations
//! - **Domain Specificity**: Task difficulties calibrated per domain
//! - **Hint Configuration**: Intervention thresholds adapted to population needs
//! - **Preset Combinations**: Ready-to-use configuration presets
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::config::{SystemConfig, PopulationType, DomainType, LearningGoal};
//! 
//! // Create a preset configuration
//! let config = SystemConfig::preset(
//!     PopulationType::Child,
//!     DomainType::Alphabet, 
//!     LearningGoal::Mastery
//! );
//! 
//! // Or customize individual components
//! let learner_config = LearnerConfig::child();
//! ```

pub mod learner;

// Re-export main types
pub use learner::{
    LearnerConfig, AdaptiveSchedulingConfig, HintInterventionConfig, 
    DomainConfig, SystemConfig, PopulationType, DomainType, LearningGoal,
    TaskDifficultyConfig, ScoringWeights
};