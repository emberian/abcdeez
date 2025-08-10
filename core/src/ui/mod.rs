//! # User Interfaces
//! 
//! This module provides various user interface components for the learning system:
//! 
//! - [`ResearchDashboard`]: Comprehensive research analytics dashboard
//! - Terminal User Interface (TUI) for command-line interaction
//! - UI components for web and desktop applications
//! - Data visualization components
//! 
//! ## Research Dashboard
//! 
//! - **Real-time Analytics**: Live learning performance metrics
//! - **Experiment Monitoring**: Track ongoing experiments
//! - **Learner Profiles**: Individual learner progress visualization
//! - **Population Analytics**: Cross-learner comparisons and trends
//! 
//! ## Terminal Interface
//! 
//! - **Command-line Tools**: Interactive CLI for researchers
//! - **Configuration Management**: Set up experiments from terminal
//! - **Data Export**: Command-line data export utilities
//! - **System Monitoring**: Monitor system health and performance
//! 
//! ## Web Components
//! 
//! - **Interactive Widgets**: Learning task interfaces
//! - **Progress Visualization**: Learning curve displays
//! - **Real-time Feedback**: Immediate performance feedback
//! - **Responsive Design**: Works on desktop and mobile
//! 
//! ## Visualization Components
//! 
//! - **Learning Curves**: Individual and group progress plots
//! - **Heat Maps**: Performance across different task types
//! - **Network Graphs**: Topology and relationship visualizations
//! - **Statistical Plots**: Distribution plots and confidence intervals
//! 
//! ## Quick Start
//! 
//! ```rust
//! #[cfg(feature = "ui")]
//! use abcdeez_core::ui::ResearchDashboard;
//! 
//! #[cfg(feature = "ui")]
//! {
//!     // Create research dashboard
//!     let dashboard = ResearchDashboard::new();
//!     dashboard.add_experiment_monitor(&experiment)?;
//!     dashboard.start_server("localhost:8080")?;
//! }
//! ```

#[cfg(feature = "cli")]
pub mod tui;
pub mod components;
#[cfg(feature = "cli")]
pub mod dashboard;

// Re-export main types (conditionally compiled)
#[cfg(feature = "cli")]
pub use dashboard::ResearchDashboard;