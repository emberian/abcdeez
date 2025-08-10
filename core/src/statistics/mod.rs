//! # Statistical Analysis & Validation
//! 
//! This module provides comprehensive statistical analysis tools for learning research:
//! 
//! - [`StatisticalValidator`]: Validates statistical assumptions and performs tests
//! - [`MixedEffectsAnalyzer`]: Hierarchical/mixed-effects modeling  
//! - [`PowerAnalyzer`]: Statistical power analysis and effect size calculation
//! - [`ResponseTimeDistribution`]: Models for response time distributions
//! 
//! ## Statistical Tests
//! 
//! - **Assumption Checking**: Normality, homoscedasticity, independence
//! - **Hypothesis Testing**: t-tests, ANOVA, non-parametric alternatives
//! - **Effect Sizes**: Cohen's d, eta-squared, omega-squared
//! - **Model Comparison**: AIC, BIC, likelihood ratio tests
//! 
//! ## Mixed Effects Models
//! 
//! Supports hierarchical modeling with:
//! - Random intercepts and slopes for participants
//! - Crossed random effects (participants × items)
//! - Multiple levels of nesting
//! - Bayesian and frequentist estimation
//! 
//! ## Power Analysis
//! 
//! - A priori power analysis for study planning
//! - Post-hoc power analysis for completed studies  
//! - Effect size estimation with confidence intervals
//! - Sample size recommendations
//! 
//! ## Quick Start
//! 
//! ```rust
//! use abcdeez_core::statistics::{StatisticalValidator, PowerAnalyzer};
//! 
//! // Validate assumptions
//! let validator = StatisticalValidator::new(0.95);
//! let normality = validator.test_normality(&data);
//! 
//! // Power analysis
//! let power_analyzer = PowerAnalyzer::new();
//! let power = power_analyzer.power_ttest(0.5, 30, 30, 0.05)?;
//! ```

pub mod validation;
pub mod modeling;
pub mod mixed_effects;
pub mod power_analysis;

// Re-export main types
pub use validation::{
    StatisticalValidator, AssumptionChecks, NormalityTest, 
    HomoscedasticityTest, OutlierAnalysis
};
pub use modeling::{
    DetailedStatistics, ResponseTimeDistribution, ExGaussianModel,
    ExGaussianParameters, SessionAnalyzer, StrategyType, TestResult
};
pub use mixed_effects::{
    MixedEffectsAnalyzer, MixedEffectsModel, MixedEffectsResults,
    MixedEffectsData, RandomEffectSpec
};
pub use power_analysis::{
    PowerAnalyzer, PowerAnalysis, EffectSizeCalculator,
    RealTimeEffectMonitor, RealTimeMonitor
};