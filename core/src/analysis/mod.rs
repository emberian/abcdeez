//! # Advanced Analysis Methods
//! 
//! This module provides sophisticated analysis methods for learning research:
//! 
//! - [`TransferLearningAnalyzer`]: Analyze knowledge transfer between domains
//! - Longitudinal analysis for learning curves and development
//! - Individual differences analysis and clustering
//! - Population-level comparisons and meta-analysis
//! 
//! ## Transfer Learning Analysis
//! 
//! - **Cross-Domain Transfer**: Measure transfer between different learning domains
//! - **Near/Far Transfer**: Distinguish between close and distant transfer
//! - **Positive/Negative Transfer**: Identify when prior knowledge helps or hurts
//! - **Transfer Metrics**: Quantitative measures of transfer effectiveness
//! 
//! ## Longitudinal Analysis
//! 
//! - **Learning Curves**: Model individual and group learning trajectories
//! - **Developmental Changes**: Track changes over extended time periods
//! - **Retention Analysis**: Measure long-term knowledge retention
//! - **Forgetting Curves**: Model knowledge decay patterns
//! 
//! ## Individual Differences
//! 
//! - **Learner Profiling**: Identify distinct learner types and strategies
//! - **Clustering Analysis**: Group learners by learning patterns
//! - **Predictor Analysis**: Identify factors that predict learning success
//! - **Adaptation Strategies**: Recommend personalized learning approaches
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::analysis::TransferLearningAnalyzer;
//! 
//! let analyzer = TransferLearningAnalyzer::new();
//! 
//! // Analyze transfer between domains
//! let transfer_metrics = analyzer.analyze_cross_domain_transfer(
//!     &source_performance, 
//!     &target_performance
//! )?;
//! 
//! println!("Transfer efficiency: {:.2}", transfer_metrics.transfer_efficiency);
//! ```

pub mod transfer;

// Re-export main types
pub use transfer::TransferLearningSystem;