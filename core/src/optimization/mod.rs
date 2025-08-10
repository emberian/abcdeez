//! # Performance & Optimization
//! 
//! This module provides performance monitoring, prediction, and optimization tools:
//! 
//! - [`PerformancePredictor`]: Predict learner performance and outcomes
//! - [`ScheduleOptimizer`]: Optimize learning schedules and task sequences
//! - [`PerformanceTracker`]: Monitor system and learning performance
//! - [`AsyncPerformanceTracker`]: Async performance monitoring for web systems
//! 
//! ## Performance Prediction
//! 
//! - **Learning Curves**: Predict future learning performance
//! - **Completion Time**: Estimate time to mastery
//! - **Difficulty Modeling**: Predict task difficulty for individual learners
//! - **Success Probability**: Forecast likelihood of task success
//! 
//! ## Schedule Optimization
//! 
//! - **Curriculum Sequencing**: Optimize order of learning materials
//! - **Spaced Repetition**: Optimal timing for review sessions
//! - **Difficulty Progression**: Smooth difficulty curves for engagement
//! - **Multi-Objective**: Balance learning efficiency with engagement
//! 
//! ## Performance Monitoring
//! 
//! - **System Metrics**: CPU, memory, response times
//! - **Learning Metrics**: Progress rates, error patterns
//! - **Real-time Dashboards**: Live performance visualization
//! - **Alert Systems**: Notifications for performance issues
//! 
//! ## Optimization Algorithms
//! 
//! - **Genetic Algorithms**: Evolve optimal learning sequences
//! - **Simulated Annealing**: Fine-tune parameter configurations
//! - **Bayesian Optimization**: Efficient hyperparameter tuning
//! - **Reinforcement Learning**: Learn optimal teaching policies
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::optimization::{PerformancePredictor, ScheduleOptimizer};
//! 
//! // Predict learner performance
//! let predictor = PerformancePredictor::new(&learner_model);
//! let predicted_success = predictor.predict_task_success(&task)?;
//! 
//! // Optimize learning schedule
//! let optimizer = ScheduleOptimizer::new();
//! let optimal_sequence = optimizer.optimize_task_sequence(&tasks, &learner)?;
//! ```

pub mod prediction;
pub mod tracing;

// Re-export main types
pub use prediction::{PerformancePredictor, ScheduleOptimizer};
pub use tracing::{
    PerformanceTracker, AsyncPerformanceTracker, PerformanceMetrics,
    CriticalPathMonitor
};