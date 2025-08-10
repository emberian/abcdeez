//! # A/B Testing Framework
//! 
//! This module provides comprehensive A/B testing capabilities for learning experiments:
//! 
//! - [`ABTestFramework`]: Comprehensive A/B testing infrastructure
//! - [`TestVariant`]: Define and manage test variants
//! - [`ABTestResults`]: Statistical analysis of A/B test results
//! - Bayesian and frequentist analysis methods
//! 
//! ## A/B Testing Features
//! 
//! - **Multi-Armed Bandits**: Dynamic allocation based on performance
//! - **Bayesian Analysis**: Continuous monitoring with credible intervals
//! - **Sequential Testing**: Stop tests early when significance is reached
//! - **Stratified Randomization**: Balance covariates across arms
//! 
//! ## Test Variants
//! 
//! - **Learning Algorithms**: Compare different adaptive algorithms
//! - **UI Designs**: Test different interface approaches
//! - **Curriculum Orders**: Compare different task sequences
//! - **Intervention Strategies**: Test different hint/help systems
//! 
//! ## Statistical Methods
//! 
//! - **Frequentist**: Traditional hypothesis testing with p-values
//! - **Bayesian**: Posterior probability of superiority
//! - **Bootstrap**: Non-parametric confidence intervals
//! - **Permutation Tests**: Exact p-values for small samples
//! 
//! ## Ethical Considerations
//! 
//! - **Equipoise**: Only test when genuine uncertainty exists
//! - **Harm Monitoring**: Stop tests if one arm shows clear harm
//! - **Informed Consent**: Participants aware of experimental nature
//! - **Fair Allocation**: Ensure all participants get reasonable experience
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::testing::{ABTestFramework, TestVariant};
//! 
//! // Create A/B test
//! let mut framework = ABTestFramework::new("learning_algorithm_test");
//! 
//! // Define test variants
//! let variant_a = TestVariant::new("standard_eig", "Standard EIG algorithm");
//! let variant_b = TestVariant::new("enhanced_eig", "Enhanced EIG with priors");
//! 
//! framework.add_variant(variant_a)?;
//! framework.add_variant(variant_b)?;
//! 
//! // Assign participant to variant
//! let assignment = framework.assign_participant("P001")?;
//! 
//! // Record outcome
//! framework.record_outcome("P001", "learning_efficiency", 0.85)?;
//! 
//! // Analyze results
//! let results = framework.analyze_results()?;
//! ```

pub mod framework;

// Re-export main types
pub use framework::{ABTest, ABTestFramework, ABTestResults, TestVariant};