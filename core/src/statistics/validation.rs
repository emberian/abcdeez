use serde::{Deserialize, Serialize};
use statrs::distribution::{ChiSquared, ContinuousCDF, FisherSnedecor, Normal, StudentsT};
use statrs::statistics::Statistics;

/// Comprehensive statistical validation framework for the learning system
pub struct StatisticalValidator {
    confidence_level: f64,
    min_sample_size: usize,
    multiple_comparison_correction: CorrectionMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrectionMethod {
    None,
    Bonferroni,
    HolmBonferroni,
    BenjaminiHochberg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisTestResult {
    pub test_name: String,
    pub statistic: f64,
    pub p_value: f64,
    pub significant: bool,
    pub effect_size: f64,
    pub confidence_interval: (f64, f64),
    pub power: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub tests_performed: Vec<HypothesisTestResult>,
    pub assumptions_checked: AssumptionChecks,
    pub model_fit_metrics: ModelFitMetrics,
    pub cross_validation_results: CrossValidationResults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumptionChecks {
    pub normality: NormalityTest,
    pub homoscedasticity: HomoscedasticityTest,
    pub independence: IndependenceTest,
    pub linearity: LinearityTest,
    pub outliers: OutlierAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalityTest {
    pub shapiro_wilk_statistic: f64,
    pub shapiro_wilk_p_value: f64,
    pub anderson_darling_statistic: f64,
    pub is_normal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomoscedasticityTest {
    pub levene_statistic: f64,
    pub levene_p_value: f64,
    pub bartlett_statistic: f64,
    pub bartlett_p_value: f64,
    pub equal_variance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndependenceTest {
    pub durbin_watson_statistic: f64,
    pub autocorrelation: f64,
    pub is_independent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearityTest {
    pub r_squared: f64,
    pub residual_pattern: String,
    pub is_linear: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierAnalysis {
    pub outlier_count: usize,
    pub outlier_percentage: f64,
    pub outlier_indices: Vec<usize>,
    pub max_z_score: f64,
    pub outliers_detected: bool,
    pub method_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFitMetrics {
    pub aic: f64, // Akaike Information Criterion
    pub bic: f64, // Bayesian Information Criterion
    pub log_likelihood: f64,
    pub rmse: f64, // Root Mean Square Error
    pub mae: f64,  // Mean Absolute Error
    pub r_squared: f64,
    pub adjusted_r_squared: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossValidationResults {
    pub k_folds: usize,
    pub mean_accuracy: f64,
    pub std_accuracy: f64,
    pub fold_results: Vec<f64>,
    pub confusion_matrix: Vec<Vec<usize>>,
}

impl StatisticalValidator {
    pub fn new(confidence_level: f64) -> Self {
        StatisticalValidator {
            confidence_level,
            min_sample_size: 30,
            multiple_comparison_correction: CorrectionMethod::BenjaminiHochberg,
        }
    }

    /// Perform t-test for comparing two groups
    pub fn t_test(&self, group1: &[f64], group2: &[f64], paired: bool) -> HypothesisTestResult {
        let n1 = group1.len();
        let n2 = group2.len();

        // Check for minimum sample size
        if n1 < self.min_sample_size || n2 < self.min_sample_size {
            return HypothesisTestResult {
                test_name: "t-test (insufficient sample size)".to_string(),
                statistic: 0.0,
                p_value: 1.0,
                significant: false,
                effect_size: 0.0,
                confidence_interval: (0.0, 0.0),
                power: 0.0,
            };
        }

        let mean1 = group1.mean();
        let mean2 = group2.mean();
        let var1 = group1.variance();
        let var2 = group2.variance();

        let (t_statistic, df) = if paired {
            // Paired t-test
            let differences: Vec<f64> = group1
                .iter()
                .zip(group2.iter())
                .map(|(a, b)| a - b)
                .collect();
            let mean_diff = (&differences).mean();
            let std_diff = (&differences).std_dev();
            let se_diff = std_diff / (n1 as f64).sqrt();

            (mean_diff / se_diff, (n1 - 1) as f64)
        } else {
            // Independent samples t-test (Welch's)
            let se = ((var1 / n1 as f64) + (var2 / n2 as f64)).sqrt();
            let t = (mean1 - mean2) / se;

            // Welch-Satterthwaite degrees of freedom
            let df = ((var1 / n1 as f64) + (var2 / n2 as f64)).powi(2)
                / ((var1 / n1 as f64).powi(2) / (n1 - 1) as f64
                    + (var2 / n2 as f64).powi(2) / (n2 - 1) as f64);

            (t, df)
        };

        // Calculate p-value using Student's t distribution
        let t_dist = StudentsT::new(0.0, 1.0, df).unwrap();
        let p_value = 2.0 * (1.0 - t_dist.cdf(t_statistic.abs()));

        // Calculate effect size (Cohen's d)
        let pooled_std = ((var1 + var2) / 2.0).sqrt();
        let effect_size = (mean1 - mean2).abs() / pooled_std;

        // Calculate confidence interval
        let critical_value = t_dist.inverse_cdf(1.0 - (1.0 - self.confidence_level) / 2.0);
        let margin_of_error = critical_value * ((var1 / n1 as f64) + (var2 / n2 as f64)).sqrt();
        let ci_lower = (mean1 - mean2) - margin_of_error;
        let ci_upper = (mean1 - mean2) + margin_of_error;

        // Calculate statistical power
        let power = self.calculate_power(effect_size, n1.min(n2), self.confidence_level);

        HypothesisTestResult {
            test_name: if paired {
                "Paired t-test"
            } else {
                "Welch's t-test"
            }
            .to_string(),
            statistic: t_statistic,
            p_value,
            significant: p_value < (1.0 - self.confidence_level),
            effect_size,
            confidence_interval: (ci_lower, ci_upper),
            power,
        }
    }

    /// ANOVA for comparing multiple groups
    pub fn anova(&self, groups: Vec<Vec<f64>>) -> HypothesisTestResult {
        let k = groups.len();
        if k < 2 {
            return HypothesisTestResult {
                test_name: "ANOVA".to_string(),
                statistic: 0.0,
                p_value: 1.0,
                significant: false,
                effect_size: 0.0,
                confidence_interval: (0.0, 0.0),
                power: 0.0,
            };
        }

        // Calculate group means and overall mean
        let group_means: Vec<f64> = groups.iter().map(|g| g.mean()).collect();
        let all_values: Vec<f64> = groups.iter().flatten().cloned().collect();
        let grand_mean = (&all_values).mean();
        let n_total = all_values.len();

        // Calculate sum of squares
        let ss_between: f64 = groups
            .iter()
            .zip(&group_means)
            .map(|(group, &mean)| group.len() as f64 * (mean - grand_mean).powi(2))
            .sum();

        let ss_within: f64 = groups
            .iter()
            .zip(&group_means)
            .map(|(group, &mean)| group.iter().map(|&x| (x - mean).powi(2)).sum::<f64>())
            .sum();

        let ss_total = ss_between + ss_within;

        // Degrees of freedom
        let df_between = (k - 1) as f64;
        let df_within = (n_total - k) as f64;

        // Mean squares
        let ms_between = ss_between / df_between;
        let ms_within = ss_within / df_within;

        // F-statistic
        let f_statistic = ms_between / ms_within;

        // P-value from F-distribution
        let f_dist = FisherSnedecor::new(df_between, df_within).unwrap();
        let p_value = 1.0 - f_dist.cdf(f_statistic);

        // Effect size (eta squared)
        let effect_size = ss_between / ss_total;

        // Power calculation
        let power = self.calculate_power(effect_size.sqrt(), n_total / k, self.confidence_level);

        HypothesisTestResult {
            test_name: "One-way ANOVA".to_string(),
            statistic: f_statistic,
            p_value,
            significant: p_value < (1.0 - self.confidence_level),
            effect_size,
            confidence_interval: (0.0, 0.0), // Not applicable for ANOVA
            power,
        }
    }

    /// Chi-squared test for independence
    pub fn chi_squared_test(&self, observed: Vec<Vec<f64>>) -> HypothesisTestResult {
        let rows = observed.len();
        let cols = if rows > 0 { observed[0].len() } else { 0 };

        if rows < 2 || cols < 2 {
            return HypothesisTestResult {
                test_name: "Chi-squared test".to_string(),
                statistic: 0.0,
                p_value: 1.0,
                significant: false,
                effect_size: 0.0,
                confidence_interval: (0.0, 0.0),
                power: 0.0,
            };
        }

        // Calculate row and column totals
        let row_totals: Vec<f64> = observed.iter().map(|row| row.iter().sum()).collect();

        let col_totals: Vec<f64> = (0..cols)
            .map(|j| observed.iter().map(|row| row[j]).sum())
            .collect();

        let grand_total: f64 = row_totals.iter().sum();

        // Calculate expected frequencies and check assumptions
        let mut chi_squared = 0.0;
        let mut min_expected = f64::INFINITY;
        let mut expected_frequencies = vec![vec![0.0; cols]; rows];

        for i in 0..rows {
            for j in 0..cols {
                let expected = (row_totals[i] * col_totals[j]) / grand_total;
                expected_frequencies[i][j] = expected;
                min_expected = min_expected.min(expected);
                if expected > 0.0 {
                    chi_squared += (observed[i][j] - expected).powi(2) / expected;
                }
            }
        }

        // Check assumptions: all expected frequencies should be >= 5
        if min_expected < 5.0 {
            eprintln!("Warning: Chi-square test assumption violated (expected < 5). Consider Fisher's exact test.");
            // For 2x2 tables, we could implement Fisher's exact test
            if rows == 2 && cols == 2 {
                // Fall back to Fisher's exact test for 2x2 tables
                return self.fishers_exact_test_2x2(&observed);
            }
        }

        // Check sample size
        if grand_total < 20.0 {
            eprintln!(
                "Warning: Small sample size ({}) for chi-square test",
                grand_total
            );
        }

        // Degrees of freedom
        let df = ((rows - 1) * (cols - 1)) as f64;

        // P-value
        let chi_dist = ChiSquared::new(df).unwrap();
        let p_value = 1.0 - chi_dist.cdf(chi_squared);

        // Effect size (Cramér's V)
        let n = grand_total;
        let min_dim = (rows - 1).min(cols - 1) as f64;
        let effect_size = (chi_squared / (n * min_dim)).sqrt();

        HypothesisTestResult {
            test_name: "Chi-squared test of independence".to_string(),
            statistic: chi_squared,
            p_value,
            significant: p_value < (1.0 - self.confidence_level),
            effect_size,
            confidence_interval: (0.0, 0.0),
            power: self.calculate_power(effect_size, n as usize, self.confidence_level),
        }
    }

    /// Check normality assumptions with comprehensive analysis
    pub fn check_normality(&self, data: &[f64]) -> NormalityTest {
        if data.len() < 3 {
            return NormalityTest {
                shapiro_wilk_statistic: 0.0,
                shapiro_wilk_p_value: 1.0,
                anderson_darling_statistic: 0.0,
                is_normal: false,
            };
        }

        // Use Shapiro-Wilk for small samples, Anderson-Darling for larger samples
        if data.len() <= 50 {
            self.shapiro_wilk_detailed(data)
        } else {
            self.anderson_darling_detailed(data)
        }
    }

    fn shapiro_wilk_detailed(&self, data: &[f64]) -> NormalityTest {
        let n = data.len() as f64;
        let sorted_data: Vec<f64> = {
            let mut d = data.to_vec();
            d.sort_by(|a, b| a.partial_cmp(b).unwrap());
            d
        };

        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);

        // Simplified W statistic calculation
        let w_stat = if variance > 0.0 {
            let numerator: f64 = sorted_data
                .iter()
                .enumerate()
                .map(|(i, &x)| {
                    let coeff = self.shapiro_wilk_coefficient(i, data.len());
                    coeff * x
                })
                .sum::<f64>()
                .powi(2);
            numerator / (variance * (n - 1.0))
        } else {
            1.0
        };

        let p_value = if w_stat > 0.95 {
            0.8
        } else if w_stat > 0.90 {
            0.2
        } else if w_stat > 0.85 {
            0.05
        } else {
            0.01
        };
        let ad_statistic = self.anderson_darling_statistic(data);

        NormalityTest {
            shapiro_wilk_statistic: w_stat,
            shapiro_wilk_p_value: p_value,
            anderson_darling_statistic: ad_statistic,
            is_normal: p_value > (1.0 - self.confidence_level),
        }
    }

    fn anderson_darling_detailed(&self, data: &[f64]) -> NormalityTest {
        let n = data.len() as f64;
        let sorted_data: Vec<f64> = {
            let mut d = data.to_vec();
            d.sort_by(|a, b| a.partial_cmp(b).unwrap());
            d
        };

        let mean = data.iter().sum::<f64>() / n;
        let std_dev = (data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();

        // Standardize data
        let normal = Normal::new(0.0, 1.0).unwrap();
        let mut ad_stat = 0.0;

        for (i, &x) in sorted_data.iter().enumerate() {
            let z = (x - mean) / std_dev;
            let phi = normal.cdf(z);
            let i_f64 = (i + 1) as f64;

            if phi > 0.0 && phi < 1.0 {
                ad_stat += (2.0 * i_f64 - 1.0)
                    * (phi.ln() + (1.0 - sorted_data[sorted_data.len() - i - 1]).ln());
            }
        }

        ad_stat = -n - ad_stat / n;
        let ad_stat_adj = ad_stat * (1.0 + 0.75 / n + 2.25 / (n * n));

        let p_value = self.anderson_darling_p_value(ad_stat_adj);

        NormalityTest {
            shapiro_wilk_statistic: 0.0, // Not computed for large samples
            shapiro_wilk_p_value: 0.0,
            anderson_darling_statistic: ad_stat_adj,
            is_normal: p_value > (1.0 - self.confidence_level),
        }
    }

    fn anderson_darling_p_value(&self, ad_stat: f64) -> f64 {
        if ad_stat < 0.2 {
            1.0 - (-1.2337 * ad_stat).exp()
        } else if ad_stat < 0.34 {
            1.0 - (1.0833 * ad_stat - 2.1962).exp()
        } else if ad_stat < 0.6 {
            1.0 - (-1.9003 * ad_stat - 1.0837).exp()
        } else {
            (ad_stat - 0.6) * (-0.37782 * ad_stat + 2.8123).exp()
        }
    }

    fn shapiro_wilk_coefficient(&self, i: usize, n: usize) -> f64 {
        // Simplified coefficients (would use table in production)
        let normal = Normal::new(0.0, 1.0).unwrap();
        normal.inverse_cdf((i as f64 + 0.375) / (n as f64 + 0.25))
    }

    fn anderson_darling_statistic(&self, data: &[f64]) -> f64 {
        let n = data.len();
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mean = data.mean();
        let std_dev = data.std_dev();
        let normal = Normal::new(mean, std_dev).unwrap();

        let mut sum = 0.0;
        for i in 0..n {
            let z_i = normal.cdf(sorted[i]);
            let z_ni = normal.cdf(sorted[n - 1 - i]);
            sum += (2 * i + 1) as f64 * (z_i.ln() + (1.0 - z_ni).ln());
        }

        -(n as f64) - sum / (n as f64)
    }

    /// Calculate statistical power
    fn calculate_power(&self, effect_size: f64, sample_size: usize, alpha: f64) -> f64 {
        // Simplified power calculation using normal approximation
        let z_alpha = Normal::new(0.0, 1.0)
            .unwrap()
            .inverse_cdf(1.0 - alpha / 2.0);

        let non_centrality = effect_size * (sample_size as f64).sqrt();
        let z_beta = z_alpha - non_centrality;

        let normal = Normal::new(0.0, 1.0).unwrap();
        1.0 - normal.cdf(z_beta)
    }

    /// Apply multiple comparison correction
    pub fn apply_correction(&self, p_values: Vec<f64>) -> Vec<f64> {
        match self.multiple_comparison_correction {
            CorrectionMethod::None => p_values,
            CorrectionMethod::Bonferroni => {
                let m = p_values.len() as f64;
                p_values.iter().map(|&p| (p * m).min(1.0)).collect()
            }
            CorrectionMethod::HolmBonferroni => {
                let mut indexed: Vec<_> = p_values.iter().enumerate().collect();
                indexed.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

                let mut corrected = vec![0.0; p_values.len()];
                for (rank, &(idx, &p)) in indexed.iter().enumerate() {
                    let m = p_values.len() - rank;
                    corrected[idx] = (p * m as f64).min(1.0);
                }
                corrected
            }
            CorrectionMethod::BenjaminiHochberg => {
                let mut indexed: Vec<_> = p_values.iter().enumerate().collect();
                indexed.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

                let m = p_values.len() as f64;
                let mut corrected = vec![0.0; p_values.len()];

                for (rank, &(idx, &p)) in indexed.iter().enumerate() {
                    let adjusted = (p * m / (rank + 1) as f64).min(1.0);
                    corrected[idx] = adjusted;
                }

                // Ensure monotonicity
                for i in (0..corrected.len() - 1).rev() {
                    if corrected[indexed[i].0] > corrected[indexed[i + 1].0] {
                        corrected[indexed[i].0] = corrected[indexed[i + 1].0];
                    }
                }

                corrected
            }
        }
    }

    /// Perform cross-validation
    pub fn cross_validate<F>(
        &self,
        data: &[f64],
        labels: &[bool],
        k: usize,
        predictor: F,
    ) -> CrossValidationResults
    where
        F: Fn(&[f64], &[bool]) -> Vec<bool>,
    {
        let n = data.len();
        let fold_size = n / k;
        let mut fold_results = Vec::new();
        let mut all_predictions = Vec::new();
        let mut all_labels = Vec::new();

        for fold in 0..k {
            let test_start = fold * fold_size;
            let test_end = if fold == k - 1 {
                n
            } else {
                (fold + 1) * fold_size
            };

            // Split data
            let mut train_data = Vec::new();
            let mut train_labels = Vec::new();
            let mut test_data = Vec::new();
            let mut test_labels = Vec::new();

            for i in 0..n {
                if i >= test_start && i < test_end {
                    test_data.push(data[i]);
                    test_labels.push(labels[i]);
                } else {
                    train_data.push(data[i]);
                    train_labels.push(labels[i]);
                }
            }

            // Make predictions
            let predictions = predictor(&train_data, &train_labels);

            // Calculate accuracy for this fold
            let correct = predictions
                .iter()
                .zip(&test_labels)
                .filter(|(&pred, &actual)| pred == actual)
                .count();
            let accuracy = correct as f64 / test_labels.len() as f64;
            fold_results.push(accuracy);

            all_predictions.extend(predictions);
            all_labels.extend(test_labels);
        }

        // Build confusion matrix
        let mut confusion_matrix = vec![vec![0; 2]; 2];
        for (&pred, &actual) in all_predictions.iter().zip(&all_labels) {
            confusion_matrix[actual as usize][pred as usize] += 1;
        }

        CrossValidationResults {
            k_folds: k,
            mean_accuracy: (&fold_results).mean(),
            std_accuracy: (&fold_results).std_dev(),
            fold_results,
            confusion_matrix,
        }
    }

    /// Fisher's exact test for 2x2 contingency tables
    fn fishers_exact_test_2x2(&self, observed: &Vec<Vec<f64>>) -> HypothesisTestResult {
        // For 2x2 table [[a, b], [c, d]]
        let a = observed[0][0] as i32;
        let b = observed[0][1] as i32;
        let c = observed[1][0] as i32;
        let d = observed[1][1] as i32;

        let _n = a + b + c + d;
        let row1_total = a + b;
        let row2_total = c + d;
        let col1_total = a + c;
        let _col2_total = b + d;

        // Calculate hypergeometric probability for observed table
        let p_observed = self.hypergeometric_prob(a, b, c, d);

        // Calculate two-tailed p-value
        let mut p_value = 0.0;

        // Sum probabilities of all tables as extreme or more extreme
        for a_i in 0..=row1_total.min(col1_total) {
            let b_i = row1_total - a_i;
            let c_i = col1_total - a_i;
            let d_i = row2_total - c_i;

            if b_i >= 0 && c_i >= 0 && d_i >= 0 {
                let p_i = self.hypergeometric_prob(a_i, b_i, c_i, d_i);
                if p_i <= p_observed + 1e-10 {
                    // Small epsilon for numerical stability
                    p_value += p_i;
                }
            }
        }

        // Odds ratio as effect size
        let odds_ratio = if b > 0 && c > 0 {
            (a as f64 * d as f64) / (b as f64 * c as f64)
        } else {
            f64::INFINITY
        };

        HypothesisTestResult {
            test_name: "Fisher's Exact Test".to_string(),
            statistic: odds_ratio,
            p_value,
            significant: p_value < self.confidence_level,
            effect_size: odds_ratio.ln(),    // Log odds ratio
            confidence_interval: (0.0, 0.0), // Would need more computation
            power: 0.0,                      // Not applicable for exact test
        }
    }

    /// Hypergeometric probability for Fisher's exact test
    fn hypergeometric_prob(&self, a: i32, b: i32, c: i32, d: i32) -> f64 {
        let n = a + b + c + d;

        // log(P) = log(C(a+b, a)) + log(C(c+d, c)) - log(C(n, a+c))
        let log_p = self.log_binomial_coefficient(a + b, a)
            + self.log_binomial_coefficient(c + d, c)
            - self.log_binomial_coefficient(n, a + c);

        log_p.exp()
    }

    /// Log binomial coefficient using Stirling's approximation for large values
    fn log_binomial_coefficient(&self, n: i32, k: i32) -> f64 {
        if k < 0 || k > n {
            return f64::NEG_INFINITY;
        }
        if k == 0 || k == n {
            return 0.0;
        }

        // Use logarithm of factorials
        self.log_factorial(n) - self.log_factorial(k) - self.log_factorial(n - k)
    }

    /// Log factorial using Stirling's approximation for large n
    fn log_factorial(&self, n: i32) -> f64 {
        if n <= 0 {
            return 0.0;
        }

        if n < 20 {
            // Direct computation for small n
            (1..=n).map(|i| (i as f64).ln()).sum()
        } else {
            // Stirling's approximation: ln(n!) ≈ n*ln(n) - n + 0.5*ln(2πn)
            let n_f = n as f64;
            n_f * n_f.ln() - n_f + 0.5 * (2.0 * std::f64::consts::PI * n_f).ln()
        }
    }

    /// Check for homogeneity of variance using Levene's test
    pub fn check_homogeneity(&self, groups: &[Vec<f64>]) -> HomoscedasticityTest {
        if groups.len() < 2 {
            return HomoscedasticityTest {
                levene_statistic: 0.0,
                levene_p_value: 1.0,
                bartlett_statistic: 0.0,
                bartlett_p_value: 1.0,
                equal_variance: false,
            };
        }

        let levene_result = self.levene_test(groups);
        let bartlett_result = self.bartlett_test(groups);

        HomoscedasticityTest {
            levene_statistic: levene_result.0,
            levene_p_value: levene_result.1,
            bartlett_statistic: bartlett_result.0,
            bartlett_p_value: bartlett_result.1,
            equal_variance: levene_result.1 > (1.0 - self.confidence_level)
                && bartlett_result.1 > (1.0 - self.confidence_level),
        }
    }

    fn levene_test(&self, groups: &[Vec<f64>]) -> (f64, f64) {
        let k = groups.len();
        let mut n_total = 0;
        let mut group_medians = Vec::new();
        let mut group_sizes = Vec::new();

        for group in groups {
            if group.is_empty() {
                continue;
            }

            // Calculate median (more robust than mean)
            let mut sorted_group = group.clone();
            sorted_group.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = if sorted_group.len() % 2 == 0 {
                (sorted_group[sorted_group.len() / 2 - 1] + sorted_group[sorted_group.len() / 2])
                    / 2.0
            } else {
                sorted_group[sorted_group.len() / 2]
            };

            group_medians.push(median);
            group_sizes.push(group.len());
            n_total += group.len();
        }

        if k < 2 || n_total < 3 {
            return (0.0, 1.0);
        }

        // Calculate absolute deviations from group medians
        let mut all_deviations = Vec::new();
        let mut group_deviation_means = Vec::new();

        for (i, group) in groups.iter().enumerate() {
            if group.is_empty() {
                continue;
            }

            let deviations: Vec<f64> = group
                .iter()
                .map(|&x| (x - group_medians[i]).abs())
                .collect();
            let deviation_mean = deviations.iter().sum::<f64>() / deviations.len() as f64;
            group_deviation_means.push(deviation_mean);
            all_deviations.extend(deviations);
        }

        // Calculate overall mean of deviations
        let overall_mean = all_deviations.iter().sum::<f64>() / all_deviations.len() as f64;

        // Calculate between-group and within-group sum of squares
        let mut ss_between = 0.0;
        for (i, &group_mean) in group_deviation_means.iter().enumerate() {
            ss_between += group_sizes[i] as f64 * (group_mean - overall_mean).powi(2);
        }

        let mut ss_within = 0.0;
        let mut current_index = 0;
        for (i, group) in groups.iter().enumerate() {
            if group.is_empty() {
                continue;
            }
            for j in 0..group.len() {
                ss_within += (all_deviations[current_index + j] - group_deviation_means[i]).powi(2);
            }
            current_index += group.len();
        }

        // Calculate F-statistic
        let df_between = (k - 1) as f64;
        let df_within = (n_total - k) as f64;
        let ms_between = ss_between / df_between;
        let ms_within = ss_within / df_within;
        let f_stat = if ms_within > 0.0 {
            ms_between / ms_within
        } else {
            0.0
        };

        // Calculate p-value
        let f_dist = FisherSnedecor::new(df_between, df_within).unwrap();
        let p_value = 1.0 - f_dist.cdf(f_stat);

        (f_stat, p_value)
    }

    fn bartlett_test(&self, groups: &[Vec<f64>]) -> (f64, f64) {
        let k = groups.len();
        if k < 2 {
            return (0.0, 1.0);
        }

        let mut variances = Vec::new();
        let mut sample_sizes = Vec::new();
        let mut total_n = 0;

        for group in groups {
            if group.len() > 1 {
                let variance = group.variance();
                variances.push(variance);
                sample_sizes.push(group.len());
                total_n += group.len();
            }
        }

        if variances.len() < 2 {
            return (0.0, 1.0);
        }

        // Calculate pooled variance
        let mut numerator = 0.0;
        for (i, &var) in variances.iter().enumerate() {
            numerator += (sample_sizes[i] - 1) as f64 * var;
        }
        let pooled_variance = numerator / (total_n - k) as f64;

        // Bartlett's test statistic
        let mut sum_log_vars = 0.0;
        let mut sum_weights = 0.0;
        for (i, &var) in variances.iter().enumerate() {
            let weight = (sample_sizes[i] - 1) as f64;
            sum_log_vars += weight * var.ln();
            sum_weights += weight;
        }

        let bartlett_stat = sum_weights * pooled_variance.ln() - sum_log_vars;

        // Apply correction factor (simplified)
        let correction = 1.0
            + (1.0 / (3.0 * (k - 1) as f64))
                * (variances
                    .iter()
                    .enumerate()
                    .map(|(i, _)| 1.0 / (sample_sizes[i] - 1) as f64)
                    .sum::<f64>()
                    - 1.0 / (total_n - k) as f64);

        let corrected_stat = bartlett_stat / correction;

        // Chi-squared distribution with k-1 degrees of freedom
        let chi_dist = ChiSquared::new((k - 1) as f64).unwrap();
        let p_value = 1.0 - chi_dist.cdf(corrected_stat);

        (corrected_stat, p_value)
    }

    /// Detect outliers using modified Z-score method
    pub fn check_outliers(&self, data: &[f64]) -> OutlierAnalysis {
        if data.len() < 4 {
            return OutlierAnalysis {
                outlier_count: 0,
                outlier_percentage: 0.0,
                outlier_indices: Vec::new(),
                max_z_score: 0.0,
                outliers_detected: false,
                method_used: "Insufficient data".to_string(),
            };
        }

        // Calculate median
        let median = {
            let mut sorted = data.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            if sorted.len() % 2 == 0 {
                (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
            } else {
                sorted[sorted.len() / 2]
            }
        };

        // Calculate median absolute deviation (MAD)
        let mut absolute_deviations: Vec<f64> = data.iter().map(|&x| (x - median).abs()).collect();
        absolute_deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mad = if absolute_deviations.len() % 2 == 0 {
            (absolute_deviations[absolute_deviations.len() / 2 - 1]
                + absolute_deviations[absolute_deviations.len() / 2])
                / 2.0
        } else {
            absolute_deviations[absolute_deviations.len() / 2]
        };

        let threshold = 3.5; // Common threshold for outlier detection
        let mut outlier_indices = Vec::new();
        let mut max_z_score = 0.0;

        for (i, &value) in data.iter().enumerate() {
            let modified_z = if mad > 0.0 {
                0.6745 * (value - median).abs() / mad
            } else {
                0.0
            };

            if modified_z > threshold {
                outlier_indices.push(i);
            }

            if modified_z > max_z_score {
                max_z_score = modified_z;
            }
        }

        let outlier_percentage = (outlier_indices.len() as f64 / data.len() as f64) * 100.0;

        OutlierAnalysis {
            outlier_count: outlier_indices.len(),
            outlier_percentage,
            outlier_indices: outlier_indices.clone(),
            max_z_score,
            outliers_detected: !outlier_indices.is_empty(),
            method_used: "Modified Z-Score".to_string(),
        }
    }

    /// Comprehensive assumption checking for analysis planning
    pub fn comprehensive_assumption_check(&self, data: &[Vec<f64>]) -> AssumptionChecks {
        let mut normality_tests = Vec::new();
        for group in data {
            if !group.is_empty() {
                normality_tests.push(self.check_normality(group));
            }
        }

        // Take the worst normality result
        let normality = normality_tests
            .into_iter()
            .min_by(|a, b| {
                a.shapiro_wilk_p_value
                    .partial_cmp(&b.shapiro_wilk_p_value)
                    .unwrap()
            })
            .unwrap_or(NormalityTest {
                shapiro_wilk_statistic: 0.0,
                shapiro_wilk_p_value: 1.0,
                anderson_darling_statistic: 0.0,
                is_normal: false,
            });

        let homoscedasticity = self.check_homogeneity(data);

        // Check outliers across all data
        let all_data: Vec<f64> = data.iter().flatten().cloned().collect();
        let outliers = self.check_outliers(&all_data);

        // Independence and linearity require more context
        let independence = IndependenceTest {
            durbin_watson_statistic: 2.0, // Placeholder
            autocorrelation: 0.0,
            is_independent: true, // Assumed unless evidence suggests otherwise
        };

        let linearity = LinearityTest {
            r_squared: 0.0, // Would need regression analysis
            residual_pattern: "Assessment required".to_string(),
            is_linear: true, // Assumed for basic tests
        };

        AssumptionChecks {
            normality,
            homoscedasticity,
            independence,
            linearity,
            outliers,
        }
    }
}
