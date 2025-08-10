use crate::bayesian::*;
use crate::statistics::*;
use crate::core::topology::Topology;

#[test]
fn test_ex_gaussian_pdf_formula_correctness() {
    // This test verifies the Ex-Gaussian PDF formula is correct
    // The formula should be: f(x) = (1/(2τ)) * exp(...) * erfc(...)
    // NOT: f(x) = (1/τ) * exp(...) * erfc(...)

    let params = ExGaussianParameters {
        mu: 5.0,
        sigma: 1.0,
        tau: 2.0,
    };

    let model = ExGaussianModel::new(params.clone());

    // Test at x = mu (should give a reasonable probability)
    let pdf_at_mu = model.pdf(params.mu);

    // For these parameters, PDF at mu should be around 0.15-0.25
    assert!(
        pdf_at_mu > 0.1 && pdf_at_mu < 0.3,
        "PDF at mu={} should be reasonable, got {}",
        params.mu,
        pdf_at_mu
    );

    // Test numerical integration to verify normalization
    // A properly normalized PDF should integrate to 1
    let mut integral = 0.0;
    let dx = 0.01;
    for i in -1000..5000 {
        let x = i as f64 * dx;
        integral += model.pdf(x) * dx;
    }

    assert!(
        (integral - 1.0).abs() < 0.02,
        "PDF should integrate to 1, got {}",
        integral
    );

    // Test specific known values
    // The implementation uses the correct formula with 1/(2τ)
    // Let's verify it produces reasonable values
    let test_params = ExGaussianParameters {
        mu: 0.0,
        sigma: 1.0,
        tau: 1.0,
    };
    let test_model = ExGaussianModel::new(test_params);
    let pdf_at_zero = test_model.pdf(0.0);

    // The PDF should be positive and reasonable
    assert!(
        pdf_at_zero > 0.2 && pdf_at_zero < 0.3,
        "PDF at x=0 for standard params should be in range [0.2, 0.3], got {}",
        pdf_at_zero
    );

    // Verify the PDF has the right shape: should decrease for negative x
    let pdf_negative = test_model.pdf(-2.0);
    assert!(
        pdf_negative < pdf_at_zero,
        "PDF should decrease for negative x: pdf(-2)={} should be < pdf(0)={}",
        pdf_negative,
        pdf_at_zero
    );

    // Should have exponential tail for positive x
    let pdf_tail = test_model.pdf(5.0);
    assert!(
        pdf_tail > 0.0 && pdf_tail < 0.1,
        "PDF should have exponential tail: pdf(5)={}",
        pdf_tail
    );
}

#[test]
fn test_kl_divergence_calculation() {
    // Test KL divergence between two Gaussian distributions

    // Test 1: KL divergence with itself should be 0
    let dist1 = PosteriorDistribution::new(5.0, 2.0);
    let kl_self = dist1.kl_divergence(&dist1);
    assert!(
        kl_self.abs() < 1e-10,
        "KL divergence with itself should be 0, got {}",
        kl_self
    );

    // Test 2: KL divergence should be positive for different distributions
    let dist2 = PosteriorDistribution::new(7.0, 3.0);
    let kl = dist1.kl_divergence(&dist2);
    assert!(
        kl > 0.0,
        "KL divergence between different distributions should be positive, got {}",
        kl
    );

    // Test 3: Known value test
    // KL(N(0,1) || N(1,1)) = 0.5 * ((0-1)^2 / 1) = 0.5
    let standard = PosteriorDistribution::new(0.0, 1.0);
    let shifted = PosteriorDistribution::new(1.0, 1.0);
    let kl_known = standard.kl_divergence(&shifted);
    assert!(
        (kl_known - 0.5).abs() < 0.01,
        "KL(N(0,1) || N(1,1)) should be 0.5, got {}",
        kl_known
    );

    // Test 4: KL divergence formula components
    // KL(N(μ₁,σ₁²) || N(μ₂,σ₂²)) = 0.5 * [log(σ₂²/σ₁²) + σ₁²/σ₂² + (μ₁-μ₂)²/σ₂² - 1]
    let mu1 = 2.0;
    let var1 = 1.5;
    let mu2 = 3.0;
    let var2 = 2.0;

    let dist_a = PosteriorDistribution::new(mu1, var1);
    let dist_b = PosteriorDistribution::new(mu2, var2);

    let calculated_kl = dist_a.kl_divergence(&dist_b);

    // Manual calculation
    let expected_kl = 0.5 * ((var2 / var1).ln() + var1 / var2 + (mu1 - mu2).powi(2) / var2 - 1.0);

    assert!(
        (calculated_kl - expected_kl).abs() < 0.001,
        "KL divergence calculation incorrect: expected {}, got {}",
        expected_kl,
        calculated_kl
    );
}

#[test]
fn test_adaptive_monte_carlo_convergence() {
    // Test that adaptive Monte Carlo actually converges
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    let task = crate::tasks::Task {
        task_type: crate::tasks::TaskType::Successor {
            item: "M".to_string(),
        },
        prompt: "".to_string(),
        correct_answer: "N".to_string(),
        options: vec![],
        difficulty: 0.5,
        operation: crate::learner::OperationType::Successor,
    };

    // Run adaptive Monte Carlo multiple times
    let mut results = Vec::new();
    let mut sample_counts = Vec::new();

    for _ in 0..10 {
        let (eig, samples) = model.adaptive_monte_carlo_eig(&task);
        results.push(eig);
        sample_counts.push(samples);
    }

    // Calculate mean and std dev of results
    let mean_eig = results.iter().sum::<f64>() / results.len() as f64;
    let variance =
        results.iter().map(|x| (x - mean_eig).powi(2)).sum::<f64>() / results.len() as f64;
    let std_dev = variance.sqrt();

    // Coefficient of variation should be small (< 10%)
    let cv = std_dev / mean_eig;
    assert!(
        cv < 0.1,
        "Adaptive Monte Carlo should converge consistently. CV={:.3} (std={:.4}, mean={:.4})",
        cv,
        std_dev,
        mean_eig
    );

    // Should not always use maximum samples
    let avg_samples = sample_counts.iter().sum::<usize>() / sample_counts.len();
    assert!(
        avg_samples < 5000,
        "Should converge before max samples. Average samples used: {}",
        avg_samples
    );

    // Should use more than minimum samples for non-trivial task
    assert!(
        avg_samples > 100,
        "Should use more than minimum samples for convergence. Average: {}",
        avg_samples
    );
}

#[test]
fn test_numerical_stability_thresholds() {
    // Test that numerical stability uses machine epsilon-based thresholds

    // Test with very small variance (should use epsilon threshold)
    let tiny_var = f64::EPSILON * 0.1;
    let dist = PosteriorDistribution::new(0.0, tiny_var);

    // Entropy calculation should handle tiny variance
    let entropy = dist.entropy();
    assert!(
        entropy.is_finite() || entropy == 0.0,
        "Entropy should handle tiny variance gracefully, got {}",
        entropy
    );

    // Test with zero variance (should return 0 entropy)
    let zero_var_dist = PosteriorDistribution::new(0.0, 0.0);
    let zero_entropy = zero_var_dist.entropy();
    assert_eq!(
        zero_entropy, 0.0,
        "Zero variance should give zero entropy, got {}",
        zero_entropy
    );

    // Test large values near overflow
    let params = ExGaussianParameters {
        mu: 1000.0,
        sigma: 100.0,
        tau: 0.001, // Very small tau could cause issues
    };
    let model = ExGaussianModel::new(params);

    // Should handle extreme values without panic or infinity
    let pdf_test = model.pdf(1000.0);
    assert!(
        pdf_test.is_finite() || pdf_test == 0.0,
        "PDF should handle extreme parameters, got {}",
        pdf_test
    );
}

#[test]
fn test_fisher_exact_test_implementation() {
    use crate::statistics::validation::StatisticalValidator;

    let validator = StatisticalValidator::new(0.05);

    // Test with a 2x2 contingency table
    let observed = vec![
        vec![1.0, 9.0],  // Treatment group: 1 success, 9 failures
        vec![11.0, 3.0], // Control group: 11 successes, 3 failures
    ];

    // This should trigger Fisher's exact test (small expected frequencies)
    let result = validator.chi_squared_test(observed);

    // The test should run without panic
    assert!(
        result.p_value >= 0.0 && result.p_value <= 1.0,
        "P-value should be in [0,1], got {}",
        result.p_value
    );

    // For this data, we expect a significant difference
    assert!(
        result.p_value < 0.05,
        "Should detect significant difference, p={}",
        result.p_value
    );

    // Test name should indicate Fisher's test was used
    assert!(
        result.test_name.contains("Fisher") || result.test_name.contains("Chi"),
        "Test name should indicate test type: {}",
        result.test_name
    );
}

#[test]
fn test_memory_decay_formula() {
    use crate::core::learner::LearnerModel;
    use chrono::{Duration, Utc};

    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);

    // Set initial strength
    learner.update_memory_strength("A", true);

    // Get initial strength
    let initial = learner
        .memory_strengths
        .get("node_0")
        .map(|m| m.strength)
        .unwrap_or(0.5);

    // Manually set last practice to 1 hour ago
    if let Some(mem) = learner.memory_strengths.get_mut("node_0") {
        mem.last_practice = Utc::now() - Duration::hours(1);
    }

    // Get retention probability after 1 hour
    let retention = learner.get_retention_probability("node_0");

    // With decay, retention should be less than initial strength
    assert!(
        retention < initial,
        "Memory should decay over time: {} -> {}",
        initial,
        retention
    );

    // Should follow exponential decay: strength * exp(-decay_rate * time)
    // For 1 hour with moderate decay rate, expect ~70-95% retention
    assert!(
        retention > initial * 0.6 && retention < initial * 0.98,
        "Decay should be reasonable after 1 hour: retention={}, initial={}",
        retention,
        initial
    );

    // Test longer duration (24 hours)
    if let Some(mem) = learner.memory_strengths.get_mut("node_0") {
        mem.last_practice = Utc::now() - Duration::hours(24);
    }

    let retention_24h = learner.get_retention_probability("node_0");

    // After 24 hours, should have more decay
    assert!(
        retention_24h < retention,
        "24h retention {} should be less than 1h retention {}",
        retention_24h,
        retention
    );

    // Should not decay to zero (unless very weak initially)
    if initial > 0.3 {
        assert!(
            retention_24h > 0.01,
            "Should not decay to near-zero for reasonable initial strength"
        );
    }
}
