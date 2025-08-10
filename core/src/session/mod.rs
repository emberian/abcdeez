//! # Session Management
//! 
//! This module handles learning session management, state persistence, and multi-session experiments:
//! 
//! - [`MultiSessionManager`]: Manage learning sessions across multiple days/weeks
//! - [`SessionPlan`]: Plan and schedule learning sessions
//! - [`SeedManager`]: Manage random seeds for reproducible experiments
//! - [`LongitudinalAnalysis`]: Analyze learning over extended periods
//! 
//! ## Multi-Session Learning
//! 
//! - **Session Scheduling**: Plan optimal session timing and spacing
//! - **State Persistence**: Save and restore learner state between sessions
//! - **Progress Tracking**: Monitor learning progress over time
//! - **Retention Analysis**: Measure knowledge retention between sessions
//! 
//! ## Longitudinal Experiments
//! 
//! - **Extended Studies**: Support for weeks/months-long experiments
//! - **Dropout Analysis**: Handle and analyze participant attrition
//! - **Developmental Tracking**: Monitor learning development over time
//! - **Long-term Retention**: Measure knowledge persistence
//! 
//! ## Reproducibility
//! 
//! - **Seed Management**: Consistent random number generation across sessions
//! - **Session Recording**: Complete audit trail of all session activities
//! - **State Snapshots**: Capture learner state at key time points
//! - **Replay Capability**: Recreate exact session sequences
//! 
//! ## Session Analytics
//! 
//! - **Learning Curves**: Model individual learning trajectories
//! - **Forgetting Curves**: Track knowledge decay between sessions
//! - **Optimal Spacing**: Determine ideal intervals between sessions
//! - **Individual Differences**: Analyze variation in learning patterns
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::session::{MultiSessionManager, SessionPlan};
//! 
//! // Create a multi-session learning plan
//! let mut manager = MultiSessionManager::new("learner_001");
//! 
//! let plan = SessionPlan::builder()
//!     .total_sessions(10)
//!     .session_duration_minutes(30)
//!     .spacing_days(vec![1, 2, 3, 5, 8]) // Fibonacci spacing
//!     .build()?;
//! 
//! manager.create_experiment("alphabet_learning", plan)?;
//! 
//! // Run a session
//! let session_result = manager.run_session(&session_config)?;
//! ```

pub mod multi_session;
pub mod seeds;

// Re-export main types
pub use multi_session::{
    MultiSessionExperiment, MultiSessionManager, SessionPlan, 
    LongitudinalAnalysis
};
pub use seeds::{
    SeedManager, ExperimentSeed, SessionSeed, 
    RandomizationEvent, ReproducibilityManifest
};