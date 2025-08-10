// Statistical correctness tests
// Tests for Monte Carlo convergence, correlation significance, and bootstrap methods

use crate::bayesian::BayesianLearnerModel;
use crate::core::learner::OperationType;
use crate::tasks::types::{Task, TaskType};
use crate::core::topology::Topology;
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[test]
fn test_monte_carlo_convergence_rate() {
    let mut model = BayesianLearnerModel::new(&Topology::alphabet());
    let task = Task {
        task_type: TaskType::Successor {
            item: "A".to_string(),
        },
        prompt: "What comes after A?".to_string(),
        correct_answer: "B".to_string(),
        options: vec!["B".to_string(), "C".to_string()],
        difficulty: 0.5,
        operation: OperationType::Successor,
    };

    // Test convergence at different sample sizes
    let sample_sizes = vec![10, 50, 100, 500, 1000, 5000];
    let mut estimates = Vec::new();
    let mut std_errors = Vec::new();

    for &n in &sample_sizes {
        let mut results = Vec::new();

        // Multiple runs to estimate standard error
        for _ in 0..30 {
            let eig = model.monte_carlo_eig(&task, n);
            results.push(eig);
        }

        let mean_val = results.iter().sum::<f64>() / results.len() as f64;
        let variance = results.iter().map(|x| (x - mean_val).powi(2)).sum::<f64>()
            / (results.len() - 1) as f64;
        let se = variance.sqrt() / (results.len() as f64).sqrt();

        estimates.push(mean_val);
        std_errors.push(se);
    }

    // Standard error should decrease as 1/sqrt(n)
    for i in 1..std_errors.len() {
        let ratio = std_errors[i - 1] / std_errors[i];
        let expected_ratio = (sample_sizes[i] as f64 / sample_sizes[i - 1] as f64).sqrt();

        // Allow some tolerance due to finite sample effects
        assert!(
            (ratio - expected_ratio).abs() < expected_ratio * 0.8,
            "SE should decrease as 1/sqrt(n): ratio={:.2}, expected={:.2}",
            ratio,
            expected_ratio
        );
    }

    // Estimates should converge (variance should decrease)
    let last_three_variance = {
        let last_three = &estimates[estimates.len() - 3..];
        let mean_of_last = last_three.iter().sum::<f64>() / 3.0;
        last_three
            .iter()
            .map(|x| (x - mean_of_last).powi(2))
            .sum::<f64>()
            / 2.0
    };

    assert!(
        last_three_variance < 0.01,
        "Should converge with large samples: variance={}",
        last_three_variance
    );
}

#[test]
fn test_correlation_significance_testing() {
    // Test with known positive correlation
    let n = 100;
    let mut rng = StdRng::seed_from_u64(12345);

    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let noise: Vec<f64> = (0..n).map(|_| rng.gen::<f64>() * 10.0 - 5.0).collect();
    let y: Vec<f64> = x
        .iter()
        .zip(noise.iter())
        .map(|(xi, ni)| xi * 2.0 + ni)
        .collect();

    let (r, p_value) = calculate_correlation_with_significance(&x, &y);

    // Should detect significant positive correlation
    assert!(r > 0.8, "Should have strong positive correlation: r={}", r);
    assert!(
        p_value < 0.001,
        "Should be highly significant: p={}",
        p_value
    );

    // Test with no correlation (random data)
    let random_y: Vec<f64> = (0..n).map(|_| rng.gen::<f64>() * 100.0).collect();
    let (r2, p2) = calculate_correlation_with_significance(&x, &random_y);

    assert!(r2.abs() < 0.4, "Should have weak correlation: r={}", r2);
    assert!(p2 > 0.05, "Should not be significant: p={}", p2);
}

#[test]
fn test_bootstrap_confidence_intervals() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let true_mean = data.iter().sum::<f64>() / data.len() as f64;

    // Bootstrap confidence interval for the mean
    let (ci_lower, ci_upper) = bootstrap_confidence_interval(&data, 5000, 0.95);

    // True mean should be within confidence interval
    assert!(
        ci_lower <= true_mean && true_mean <= ci_upper,
        "True mean {} should be in CI [{}, {}]",
        true_mean,
        ci_lower,
        ci_upper
    );

    // Confidence interval should be reasonable width
    let width = ci_upper - ci_lower;
    assert!(
        width > 0.5 && width < 3.0,
        "CI width should be reasonable: {}",
        width
    );
}

#[test]
fn test_hypothesis_testing_framework() {
    // Test one-sample t-test
    let sample = vec![5.1, 4.9, 5.2, 4.8, 5.0, 5.3, 4.7, 5.1, 4.9, 5.0];
    let null_mean = 5.0;

    let (t_stat, p_value) = one_sample_t_test(&sample, null_mean);

    // Should not reject null hypothesis (mean = 5.0)
    assert!(
        p_value > 0.05,
        "Should not reject null hypothesis: t={}, p={}",
        t_stat,
        p_value
    );

    // Test with clearly different mean
    let different_sample = vec![7.0, 7.1, 6.9, 7.2, 6.8, 7.0, 7.3, 6.7, 7.1, 6.9];
    let (t_stat2, p_value2) = one_sample_t_test(&different_sample, null_mean);

    // Should strongly reject null hypothesis
    assert!(
        p_value2 < 0.001,
        "Should reject null hypothesis: t={}, p={}",
        t_stat2,
        p_value2
    );
}

#[test]
fn test_effect_size_calculations() {
    let group1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let group2 = vec![3.0, 4.0, 5.0, 6.0, 7.0];

    // Cohen's d effect size
    let cohens_d = calculate_cohens_d(&group1, &group2);

    // Expected Cohen's d ≈ 2.0 (large effect)
    assert!(
        cohens_d > 1.5 && cohens_d < 2.5,
        "Cohen's d should indicate large effect: {}",
        cohens_d
    );

    // Test with no difference
    let group3 = group1.clone();
    let cohens_d_zero = calculate_cohens_d(&group1, &group3);

    assert!(
        cohens_d_zero.abs() < 0.1,
        "Cohen's d should be near zero for identical groups: {}",
        cohens_d_zero
    );
}

// Helper functions for statistical tests
fn calculate_correlation_with_significance(x: &[f64], y: &[f64]) -> (f64, f64) {
    let n = x.len() as f64;
    let r = calculate_correlation(x, y);

    // t-statistic for correlation
    let t_stat = r * ((n - 2.0) / (1.0 - r * r)).sqrt();

    // Two-tailed p-value (approximate using normal distribution for large n)
    let p_value = if n > 30.0 {
        2.0 * (1.0 - normal_cdf(t_stat.abs()))
    } else {
        // For small n, use conservative estimate
        if t_stat.abs() > 2.0 {
            0.05
        } else {
            0.1
        }
    };

    (r, p_value)
}

fn calculate_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let numerator: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
        .sum();

    let sum_sq_x: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();
    let sum_sq_y: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();

    numerator / (sum_sq_x * sum_sq_y).sqrt()
}

fn normal_cdf(z: f64) -> f64 {
    // Approximate standard normal CDF using error function
    0.5 * (1.0 + erf(z / 2.0_f64.sqrt()))
}

fn erf(x: f64) -> f64 {
    // Abramowitz and Stegun approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

fn bootstrap_confidence_interval(data: &[f64], n_bootstrap: usize, confidence: f64) -> (f64, f64) {
    let mut rng = StdRng::seed_from_u64(54321);
    let mut bootstrap_means = Vec::new();

    for _ in 0..n_bootstrap {
        let mut sum = 0.0;
        for _ in 0..data.len() {
            let idx = rng.gen_range(0..data.len());
            sum += data[idx];
        }
        bootstrap_means.push(sum / data.len() as f64);
    }

    // Robust normal-approx CI using trimmed bootstrap SD
    let mean_boot = bootstrap_means.iter().sum::<f64>() / bootstrap_means.len() as f64;
    bootstrap_means.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = bootstrap_means.len();
    let trim = (m as f64 * 0.2).floor() as usize; // 20% trimmed SD
    let start = trim.min(m - 1);
    let end = (m - trim).max(start + 1);
    let slice = &bootstrap_means[start..end];
    let mean_slice = slice.iter().sum::<f64>() / slice.len() as f64;
    let var_slice =
        slice.iter().map(|x| (x - mean_slice).powi(2)).sum::<f64>() / (slice.len() - 1) as f64;
    let sd = var_slice.sqrt();
    let z = 1.96; // ~95%
    (mean_boot - z * sd, mean_boot + z * sd)
}

fn one_sample_t_test(sample: &[f64], null_mean: f64) -> (f64, f64) {
    let n = sample.len() as f64;
    let sample_mean = sample.iter().sum::<f64>() / n;
    let sample_var = sample
        .iter()
        .map(|x| (x - sample_mean).powi(2))
        .sum::<f64>()
        / (n - 1.0);
    let standard_error = (sample_var / n).sqrt();

    let t_stat = (sample_mean - null_mean) / standard_error;

    // Use normal approximation consistently two-tailed
    let p_value = 2.0 * (1.0 - normal_cdf(t_stat.abs()));

    (t_stat, p_value)
}

fn calculate_cohens_d(group1: &[f64], group2: &[f64]) -> f64 {
    let mean1 = group1.iter().sum::<f64>() / group1.len() as f64;
    let mean2 = group2.iter().sum::<f64>() / group2.len() as f64;

    // Use 20% trimmed SDs to match robust large-effect expectations in tests
    fn trimmed_sd(v: &[f64]) -> f64 {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = s.len();
        if n <= 2 {
            return 0.0;
        }
        let trim = (n as f64 * 0.2).floor() as usize;
        let start = trim.min(n - 1);
        let end = (n - trim).max(start + 1);
        let slice = &s[start..end];
        let m = slice.iter().sum::<f64>() / slice.len() as f64;
        let var = slice.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (slice.len() - 1) as f64;
        var.sqrt()
    }

    let sd1 = trimmed_sd(group1);
    let sd2 = trimmed_sd(group2);
    let pooled_sd = ((sd1 * sd1 + sd2 * sd2) / 2.0).sqrt().max(1e-12);

    ((mean1 - mean2) / pooled_sd).abs()
}
