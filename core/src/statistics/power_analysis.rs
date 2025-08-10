use serde::{Deserialize, Serialize};

use statrs::distribution::{ContinuousCDF, Normal};

/// Power Analysis and Effect Size Monitoring for Research Design
/// Provides comprehensive statistical power calculations and real-time monitoring

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerAnalysis {
    pub test_type: StatisticalTestType,
    pub effect_size: EffectSize,
    pub power: f64,
    pub sample_size: usize,
    pub alpha: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatisticalTestType {
    OneSampleTTest,
    IndependentTTest,
    PairedTTest,
    OneWayANOVA,
    TwoWayANOVA,
    Correlation,
    Regression,
    ChiSquare,
    MannWhitneyU,
    KruskalWallis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectSize {
    pub measure: EffectSizeMeasure,
    pub value: f64,
    pub confidence_interval: (f64, f64),
    pub interpretation: EffectSizeInterpretation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectSizeMeasure {
    CohenD,     // Standardized mean difference
    HedgeG,     // Bias-corrected Cohen's d
    GlassD,     // Uses control group SD only
    EtaSquared, // Proportion of variance explained (ANOVA)
    PartialEtaSquared,
    OmegaSquared, // Less biased than eta squared
    CohensF,      // Effect size for ANOVA
    R,            // Correlation coefficient
    RSquared,     // Coefficient of determination
    CramersV,     // Effect size for chi-square
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectSizeInterpretation {
    Negligible,
    Small,
    Medium,
    Large,
    VeryLarge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeEffectMonitor {
    pub current_effect_size: EffectSize,
    pub power_trajectory: Vec<PowerPoint>,
    pub sequential_analysis: SequentialAnalysisResult,
    pub stopping_criteria: StoppingCriteria,
    pub recommendations: Vec<MonitoringRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerPoint {
    pub n: usize,
    pub power: f64,
    pub effect_size: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequentialAnalysisResult {
    pub continue_sampling: bool,
    pub reason: String,
    pub probability_of_significance: f64,
    pub estimated_final_n: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoppingCriteria {
    pub max_n: usize,
    pub min_power: f64,
    pub max_p_value: f64,
    pub min_effect_size: f64,
    pub futility_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringRecommendation {
    ContinueDataCollection,
    StopForEfficacy,
    StopForFutility,
    IncreaseAlpha,
    AdjustDesign { suggestion: String },
    ConsiderBayesianAnalysis,
}

pub struct PowerAnalyzer {
    alpha: f64,
    beta: f64, // Type II error rate (1 - power)
}

impl PowerAnalyzer {
    pub fn new(alpha: f64, power: f64) -> Self {
        Self {
            alpha,
            beta: 1.0 - power,
        }
    }

    /// Get the Type II error rate (beta)
    pub fn get_beta(&self) -> f64 {
        self.beta
    }

    /// Get the current power (1 - beta)
    pub fn get_power(&self) -> f64 {
        1.0 - self.beta
    }

    /// Update beta (Type II error rate) and recalculate power
    pub fn set_beta(&mut self, beta: f64) {
        self.beta = beta.clamp(0.0, 1.0);
    }

    /// Calculate required sample size for desired power
    pub fn calculate_sample_size(
        &self,
        test_type: StatisticalTestType,
        effect_size: f64,
        power: f64,
    ) -> Result<usize, String> {
        match test_type {
            StatisticalTestType::OneSampleTTest => self.one_sample_t_test_n(effect_size, power),
            StatisticalTestType::IndependentTTest => self.independent_t_test_n(effect_size, power),
            StatisticalTestType::PairedTTest => self.paired_t_test_n(effect_size, power),
            StatisticalTestType::OneWayANOVA => {
                self.one_way_anova_n(effect_size, power, 3) // Default 3 groups
            }
            StatisticalTestType::Correlation => self.correlation_n(effect_size, power),
            _ => Err(format!(
                "Power analysis not implemented for {:?}",
                test_type
            )),
        }
    }

    /// Calculate statistical power for given sample size
    pub fn calculate_power(
        &self,
        test_type: StatisticalTestType,
        effect_size: f64,
        sample_size: usize,
    ) -> Result<f64, String> {
        match test_type {
            StatisticalTestType::OneSampleTTest => {
                self.one_sample_t_test_power(effect_size, sample_size)
            }
            StatisticalTestType::IndependentTTest => {
                self.independent_t_test_power(effect_size, sample_size)
            }
            StatisticalTestType::PairedTTest => self.paired_t_test_power(effect_size, sample_size),
            StatisticalTestType::OneWayANOVA => {
                self.one_way_anova_power(effect_size, sample_size, 3)
            }
            StatisticalTestType::Correlation => self.correlation_power(effect_size, sample_size),
            _ => Err(format!(
                "Power calculation not implemented for {:?}",
                test_type
            )),
        }
    }

    /// One-sample t-test sample size calculation
    fn one_sample_t_test_n(&self, effect_size: f64, power: f64) -> Result<usize, String> {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - self.alpha / 2.0);
        let z_beta = normal.inverse_cdf(power);

        let n = ((z_alpha + z_beta) / effect_size).powi(2);
        Ok(n.ceil() as usize)
    }

    /// Independent t-test sample size calculation
    fn independent_t_test_n(&self, effect_size: f64, power: f64) -> Result<usize, String> {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - self.alpha / 2.0);
        let z_beta = normal.inverse_cdf(power);

        // For equal group sizes
        let n_per_group = 2.0 * ((z_alpha + z_beta) / effect_size).powi(2);
        Ok((n_per_group.ceil() as usize).max(2))
    }

    /// Paired t-test sample size calculation
    fn paired_t_test_n(&self, effect_size: f64, power: f64) -> Result<usize, String> {
        // Same as one-sample t-test for differences
        self.one_sample_t_test_n(effect_size, power)
    }

    /// One-way ANOVA sample size calculation
    fn one_way_anova_n(
        &self,
        effect_size: f64,
        power: f64,
        groups: usize,
    ) -> Result<usize, String> {
        // Simplified calculation using Cohen's f
        let f_squared = effect_size.powi(2);
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_beta = normal.inverse_cdf(power);

        // Approximate sample size per group
        let n_per_group = ((z_beta + 2.0).powi(2)) / f_squared + groups as f64;
        Ok((n_per_group.ceil() as usize * groups).max(groups * 3))
    }

    /// Correlation sample size calculation
    fn correlation_n(&self, effect_size: f64, power: f64) -> Result<usize, String> {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - self.alpha / 2.0);
        let z_beta = normal.inverse_cdf(power);

        // Fisher's Z transformation
        let z_r = 0.5 * ((1.0 + effect_size) / (1.0 - effect_size)).ln();
        let n = ((z_alpha + z_beta) / z_r).powi(2) + 3.0;

        Ok(n.ceil() as usize)
    }

    /// One-sample t-test power calculation
    fn one_sample_t_test_power(&self, effect_size: f64, n: usize) -> Result<f64, String> {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - self.alpha / 2.0);
        let ncp = effect_size * (n as f64).sqrt(); // Non-centrality parameter

        let power = 1.0 - normal.cdf(z_alpha - ncp) + normal.cdf(-z_alpha - ncp);
        Ok(power.min(1.0).max(0.0))
    }

    /// Independent t-test power calculation
    fn independent_t_test_power(&self, effect_size: f64, total_n: usize) -> Result<f64, String> {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - self.alpha / 2.0);
        // Correct formula for independent t-test: d * sqrt(n / 4) for equal group sizes
        let ncp = effect_size * ((total_n as f64) / 4.0).sqrt();

        let power = 1.0 - normal.cdf(z_alpha - ncp) + normal.cdf(-z_alpha - ncp);
        Ok(power.min(1.0).max(0.0))
    }

    /// Paired t-test power calculation
    fn paired_t_test_power(&self, effect_size: f64, n: usize) -> Result<f64, String> {
        self.one_sample_t_test_power(effect_size, n)
    }

    /// One-way ANOVA power calculation
    fn one_way_anova_power(
        &self,
        effect_size: f64,
        total_n: usize,
        groups: usize,
    ) -> Result<f64, String> {
        let _n_per_group = total_n / groups;
        let f_squared = effect_size.powi(2);
        let ncp = f_squared * total_n as f64;

        // Simplified approximation - would need F-distribution for exact calculation
        let normal = Normal::new(0.0, 1.0).unwrap();
        let critical_f = 2.5; // Approximate F-critical for α = 0.05
        let power = 1.0 - normal.cdf((critical_f - ncp.sqrt()) / (2.0 * ncp).sqrt());

        Ok(power.min(1.0).max(0.0))
    }

    /// Correlation power calculation
    fn correlation_power(&self, r: f64, n: usize) -> Result<f64, String> {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_alpha = normal.inverse_cdf(1.0 - self.alpha / 2.0);

        // Fisher's Z transformation
        let z_r = 0.5 * ((1.0 + r) / (1.0 - r)).ln();
        let se = 1.0 / ((n - 3) as f64).sqrt();
        let ncp = z_r / se;

        let power = 1.0 - normal.cdf(z_alpha - ncp) + normal.cdf(-z_alpha - ncp);
        Ok(power.min(1.0).max(0.0))
    }

    /// Generate comprehensive power analysis report
    pub fn analyze_design(
        &self,
        test_type: StatisticalTestType,
        expected_effect_size: f64,
        current_n: Option<usize>,
    ) -> PowerAnalysis {
        let mut recommendations = Vec::new();

        // Calculate power for different sample sizes
        let sample_sizes = vec![10, 20, 30, 50, 80, 100, 150, 200, 300, 500];
        let mut adequate_n = None;

        for n in sample_sizes {
            if let Ok(power) = self.calculate_power(test_type.clone(), expected_effect_size, n) {
                if power >= 0.8 && adequate_n.is_none() {
                    adequate_n = Some(n);
                    break;
                }
            }
        }

        let recommended_n = adequate_n.unwrap_or(200); // Fallback

        // Current power if sample size is provided
        let current_power = current_n
            .and_then(|n| {
                self.calculate_power(test_type.clone(), expected_effect_size, n)
                    .ok()
            })
            .unwrap_or(0.0);

        // Generate recommendations
        if current_power < 0.5 {
            recommendations.push("Very low statistical power detected. Consider increasing sample size significantly.".to_string());
        } else if current_power < 0.8 {
            recommendations.push(format!(
                "Power is below conventional threshold (0.8). Consider increasing to N = {}",
                recommended_n
            ));
        }

        if expected_effect_size < 0.2 {
            recommendations.push("Small effect size detected. Consider very large sample sizes or more sensitive measures.".to_string());
        } else if expected_effect_size > 0.8 {
            recommendations.push(
                "Large effect size detected. Smaller sample size may be adequate.".to_string(),
            );
        }

        let effect_size = EffectSize {
            measure: match test_type {
                StatisticalTestType::OneSampleTTest
                | StatisticalTestType::IndependentTTest
                | StatisticalTestType::PairedTTest => EffectSizeMeasure::CohenD,
                StatisticalTestType::OneWayANOVA | StatisticalTestType::TwoWayANOVA => {
                    EffectSizeMeasure::CohensF
                }
                StatisticalTestType::Correlation => EffectSizeMeasure::R,
                _ => EffectSizeMeasure::CohenD,
            },
            value: expected_effect_size,
            confidence_interval: (expected_effect_size * 0.7, expected_effect_size * 1.3), // Rough estimate
            interpretation: Self::interpret_effect_size(expected_effect_size),
        };

        PowerAnalysis {
            test_type,
            effect_size,
            power: current_power,
            sample_size: current_n.unwrap_or(recommended_n),
            alpha: self.alpha,
            recommendations,
        }
    }

    fn interpret_effect_size(effect_size: f64) -> EffectSizeInterpretation {
        let abs_effect = effect_size.abs();
        match abs_effect {
            x if x < 0.1 => EffectSizeInterpretation::Negligible,
            x if x < 0.3 => EffectSizeInterpretation::Small,
            x if x < 0.5 => EffectSizeInterpretation::Medium,
            x if x < 0.8 => EffectSizeInterpretation::Large,
            _ => EffectSizeInterpretation::VeryLarge,
        }
    }
}

pub struct EffectSizeCalculator;

impl EffectSizeCalculator {
    /// Calculate Cohen's d for independent groups
    pub fn cohens_d_independent(
        mean1: f64,
        mean2: f64,
        sd1: f64,
        sd2: f64,
        n1: usize,
        n2: usize,
    ) -> f64 {
        let pooled_sd = (((n1 - 1) as f64 * sd1.powi(2) + (n2 - 1) as f64 * sd2.powi(2))
            / ((n1 + n2 - 2) as f64))
            .sqrt();
        (mean1 - mean2) / pooled_sd
    }

    /// Calculate Hedge's g (bias-corrected Cohen's d)
    pub fn hedges_g(cohens_d: f64, n1: usize, n2: usize) -> f64 {
        let correction_factor = 1.0 - (3.0 / (4.0 * (n1 + n2 - 2) as f64 - 1.0));
        cohens_d * correction_factor
    }

    /// Calculate Cohen's d for paired samples
    pub fn cohens_d_paired(differences: &[f64]) -> f64 {
        let mean_diff = differences.iter().sum::<f64>() / differences.len() as f64;
        let sd_diff = {
            let variance = differences
                .iter()
                .map(|d| (d - mean_diff).powi(2))
                .sum::<f64>()
                / (differences.len() - 1) as f64;
            variance.sqrt()
        };
        mean_diff / sd_diff
    }

    /// Calculate eta squared for ANOVA
    pub fn eta_squared(ss_between: f64, ss_total: f64) -> f64 {
        ss_between / ss_total
    }

    /// Calculate partial eta squared
    pub fn partial_eta_squared(ss_effect: f64, ss_error: f64) -> f64 {
        ss_effect / (ss_effect + ss_error)
    }

    /// Calculate omega squared (less biased than eta squared)
    pub fn omega_squared(ss_between: f64, ss_within: f64, ms_between: f64, df_between: f64) -> f64 {
        (ss_between - df_between * ms_between) / (ss_between + ss_within + ms_between)
    }

    /// Calculate Cohen's f for ANOVA
    pub fn cohens_f(eta_squared: f64) -> f64 {
        (eta_squared / (1.0 - eta_squared)).sqrt()
    }

    /// Calculate confidence interval for Cohen's d
    pub fn cohens_d_confidence_interval(
        d: f64,
        n1: usize,
        n2: usize,
        confidence: f64,
    ) -> (f64, f64) {
        let normal = Normal::new(0.0, 1.0).unwrap();
        let z_critical = normal.inverse_cdf(1.0 - (1.0 - confidence) / 2.0);

        // Standard error of Cohen's d
        let se_d =
            ((n1 + n2) as f64 / (n1 * n2) as f64 + d.powi(2) / (2.0 * (n1 + n2) as f64)).sqrt();

        let margin = z_critical * se_d;
        (d - margin, d + margin)
    }

    /// Calculate Cramer's V for chi-square tests
    pub fn cramers_v(chi_square: f64, n: usize, min_dimension: usize) -> f64 {
        (chi_square / (n as f64 * (min_dimension - 1) as f64)).sqrt()
    }
}

pub struct RealTimeMonitor {
    data_points: Vec<f64>,
    sample_sizes: Vec<usize>,
    effect_sizes: Vec<f64>,
    p_values: Vec<f64>,
    power_analyzer: PowerAnalyzer,
    test_type: StatisticalTestType,
}

impl RealTimeMonitor {
    pub fn new(test_type: StatisticalTestType, alpha: f64, target_power: f64) -> Self {
        Self {
            data_points: Vec::new(),
            sample_sizes: Vec::new(),
            effect_sizes: Vec::new(),
            p_values: Vec::new(),
            power_analyzer: PowerAnalyzer::new(alpha, target_power),
            test_type,
        }
    }

    /// Add new data point and update monitoring
    pub fn add_data_point(&mut self, value: f64) -> RealTimeEffectMonitor {
        self.data_points.push(value);
        let current_n = self.data_points.len();
        self.sample_sizes.push(current_n);

        // Calculate current effect size (simplified for demonstration)
        let current_effect = self.calculate_current_effect_size();
        self.effect_sizes.push(current_effect);

        // Perform statistical test if enough data
        let current_p = if current_n >= 10 {
            self.calculate_current_p_value()
        } else {
            1.0
        };
        self.p_values.push(current_p);

        // Generate monitoring report
        self.generate_monitoring_report(current_effect, current_p, current_n)
    }

    fn calculate_current_effect_size(&self) -> f64 {
        if self.data_points.len() < 2 {
            return 0.0;
        }

        // Simplified - assume one-sample test against zero
        let mean = self.data_points.iter().sum::<f64>() / self.data_points.len() as f64;
        let variance = self
            .data_points
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / (self.data_points.len() - 1) as f64;
        let sd = variance.sqrt();

        if sd == 0.0 {
            0.0
        } else {
            mean / sd
        }
    }

    fn calculate_current_p_value(&self) -> f64 {
        if self.data_points.len() < 2 {
            return 1.0;
        }

        // Simplified one-sample t-test against zero
        let n = self.data_points.len() as f64;
        let mean = self.data_points.iter().sum::<f64>() / n;
        let variance = self
            .data_points
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / (n - 1.0);
        let se = (variance / n).sqrt();

        if se == 0.0 {
            return if mean == 0.0 { 1.0 } else { 0.0 };
        }

        let t_stat = mean / se;
        let df = n - 1.0;

        // Approximate p-value using normal approximation
        let normal = Normal::new(0.0, 1.0).unwrap();
        2.0 * (1.0 - normal.cdf(t_stat.abs() / (1.0 + 2.0 / df).sqrt()))
    }

    fn generate_monitoring_report(
        &self,
        effect_size: f64,
        p_value: f64,
        n: usize,
    ) -> RealTimeEffectMonitor {
        let current_power = self
            .power_analyzer
            .calculate_power(self.test_type.clone(), effect_size.abs(), n)
            .unwrap_or(0.0);

        let power_trajectory: Vec<PowerPoint> = self
            .sample_sizes
            .iter()
            .zip(self.effect_sizes.iter())
            .enumerate()
            .map(|(i, (&sample_n, &effect))| PowerPoint {
                n: sample_n,
                power: self
                    .power_analyzer
                    .calculate_power(self.test_type.clone(), effect.abs(), sample_n)
                    .unwrap_or(0.0),
                effect_size: effect,
                timestamp: chrono::Utc::now()
                    - chrono::Duration::minutes((self.sample_sizes.len() - i - 1) as i64),
            })
            .collect();

        let stopping_criteria = StoppingCriteria {
            max_n: 1000,
            min_power: 0.8,
            max_p_value: 0.05,
            min_effect_size: 0.2,
            futility_threshold: 0.1, // Stop if power unlikely to reach threshold
        };

        let sequential_analysis =
            self.analyze_sequential_stopping(&stopping_criteria, current_power, p_value);
        let recommendations =
            self.generate_recommendations(&sequential_analysis, current_power, p_value, n);

        RealTimeEffectMonitor {
            current_effect_size: EffectSize {
                measure: EffectSizeMeasure::CohenD,
                value: effect_size,
                confidence_interval: EffectSizeCalculator::cohens_d_confidence_interval(
                    effect_size,
                    n,
                    n,
                    0.95,
                ),
                interpretation: PowerAnalyzer::interpret_effect_size(effect_size),
            },
            power_trajectory,
            sequential_analysis,
            stopping_criteria,
            recommendations,
        }
    }

    fn analyze_sequential_stopping(
        &self,
        criteria: &StoppingCriteria,
        power: f64,
        p_value: f64,
    ) -> SequentialAnalysisResult {
        let continue_sampling = if self.data_points.len() >= criteria.max_n {
            false
        } else if p_value <= criteria.max_p_value && power >= criteria.min_power {
            false // Stop for efficacy
        } else if power < criteria.futility_threshold {
            false // Stop for futility
        } else {
            true
        };

        let reason = if !continue_sampling {
            if self.data_points.len() >= criteria.max_n {
                "Maximum sample size reached".to_string()
            } else if p_value <= criteria.max_p_value {
                "Significant result achieved with adequate power".to_string()
            } else {
                "Futility threshold reached - unlikely to achieve significance".to_string()
            }
        } else {
            "Continue data collection".to_string()
        };

        // Estimate final sample size needed
        let estimated_final_n = if continue_sampling {
            self.power_analyzer
                .calculate_sample_size(
                    self.test_type.clone(),
                    self.effect_sizes.last().unwrap_or(&0.3).abs(),
                    criteria.min_power,
                )
                .ok()
        } else {
            None
        };

        SequentialAnalysisResult {
            continue_sampling,
            reason,
            probability_of_significance: 1.0 - p_value, // Simplified
            estimated_final_n,
        }
    }

    fn generate_recommendations(
        &self,
        sequential: &SequentialAnalysisResult,
        power: f64,
        p_value: f64,
        n: usize,
    ) -> Vec<MonitoringRecommendation> {
        let mut recommendations = Vec::new();

        if sequential.continue_sampling {
            recommendations.push(MonitoringRecommendation::ContinueDataCollection);

            if let Some(estimated_n) = sequential.estimated_final_n {
                if estimated_n > 1000 {
                    recommendations.push(MonitoringRecommendation::AdjustDesign {
                        suggestion: "Consider increasing effect size through stronger manipulation or more sensitive measures".to_string(),
                    });
                }
            }

            if power < 0.2 && n > 50 {
                recommendations.push(MonitoringRecommendation::StopForFutility);
            }
        } else {
            if p_value <= 0.05 {
                recommendations.push(MonitoringRecommendation::StopForEfficacy);
            } else {
                recommendations.push(MonitoringRecommendation::StopForFutility);
            }
        }

        if p_value > 0.05 && p_value < 0.10 && power > 0.6 {
            recommendations.push(MonitoringRecommendation::ConsiderBayesianAnalysis);
        }

        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_calculation() {
        let analyzer = PowerAnalyzer::new(0.05, 0.8);

        let power = analyzer
            .calculate_power(
                StatisticalTestType::IndependentTTest,
                0.5, // Medium effect size
                64,  // Total sample size
            )
            .unwrap();

        assert!(power > 0.4); // Should have reasonable power for medium effect size, n=64
        assert!(power < 1.0);
    }

    #[test]
    fn test_sample_size_calculation() {
        let analyzer = PowerAnalyzer::new(0.05, 0.8);

        let n = analyzer
            .calculate_sample_size(
                StatisticalTestType::IndependentTTest,
                0.5, // Medium effect size
                0.8, // 80% power
            )
            .unwrap();

        assert!(n > 20); // Should require reasonable sample size
        assert!(n < 200); // But not excessive for medium effect
    }

    #[test]
    fn test_effect_size_calculation() {
        let d = EffectSizeCalculator::cohens_d_independent(10.0, 8.0, 2.0, 2.0, 30, 30);
        assert!((d - 1.0).abs() < 0.1); // Should be approximately 1.0

        let g = EffectSizeCalculator::hedges_g(d, 30, 30);
        assert!(g < d); // Hedge's g should be smaller (bias correction)
    }

    #[test]
    fn test_real_time_monitoring() {
        let mut monitor = RealTimeMonitor::new(StatisticalTestType::OneSampleTTest, 0.05, 0.8);

        // Add some data points with medium effect
        for _ in 0..20 {
            monitor.add_data_point(0.5);
        }

        let report = monitor.add_data_point(0.6);
        assert!(report.current_effect_size.value > 0.0);
        assert!(!report.power_trajectory.is_empty());
    }
}
