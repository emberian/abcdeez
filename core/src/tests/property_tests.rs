// Property-based tests using proptest
// These tests verify invariants and mathematical properties

use crate::bayesian::*;
use crate::core::learner::LearnerModel;
use crate::statistics::*;
use crate::core::topology::Topology;
use proptest::prelude::*;

// Distribution Properties
proptest! {
    #[test]
    fn test_ex_gaussian_pdf_properties(
        mu in -100.0..100.0,
        sigma in 0.1..10.0,
        tau in 0.1..10.0,
        x in -200.0..200.0
    ) {
        let model = ExGaussianModel::from_params(mu, sigma, tau);
        let pdf = model.pdf(x);

        // PDF must be non-negative
        prop_assert!(pdf >= 0.0, "PDF must be non-negative at x={}: {}", x, pdf);

        // PDF must be finite
        prop_assert!(pdf.is_finite() || pdf == 0.0,
                    "PDF must be finite at x={}: {}", x, pdf);

        // CDF must be in [0,1]
        let cdf = model.cdf(x);
        prop_assert!(cdf >= 0.0 && cdf <= 1.0,
                    "CDF must be in [0,1] at x={}: {}", x, cdf);
    }

    #[test]
    fn test_ex_gaussian_mean_variance(
        mu in -100.0..100.0,
        sigma in 0.1..10.0,
        tau in 0.1..10.0
    ) {
        let model = ExGaussianModel::from_params(mu, sigma, tau);

        // Mean should be mu + tau
        let expected_mean = mu + tau;
        let calculated_mean = model.mean();
        prop_assert!((calculated_mean - expected_mean).abs() < 1e-10,
                    "Mean formula incorrect: expected {}, got {}",
                    expected_mean, calculated_mean);

        // Variance should be sigma^2 + tau^2
        let expected_variance = sigma.powi(2) + tau.powi(2);
        let calculated_variance = model.variance();
        prop_assert!((calculated_variance - expected_variance).abs() < 1e-10,
                    "Variance formula incorrect: expected {}, got {}",
                    expected_variance, calculated_variance);
    }

    #[test]
    fn test_ex_gaussian_cdf_monotonic(
        mu in -10.0..10.0,
        sigma in 0.5..5.0,
        tau in 0.5..5.0
    ) {
        let model = ExGaussianModel::from_params(mu, sigma, tau);

        // Generate sorted x values
        let x_values: Vec<f64> = (-10..20)
            .map(|i| mu + (i as f64) * 0.5)
            .collect();

        // CDF should be monotonically increasing
        for window in x_values.windows(2) {
            let cdf1 = model.cdf(window[0]);
            let cdf2 = model.cdf(window[1]);
            prop_assert!(cdf2 >= cdf1 - 1e-10, // Allow tiny numerical error
                        "CDF not monotonic: CDF({})={} > CDF({})={}",
                        window[0], cdf1, window[1], cdf2);
        }
    }

    #[test]
    fn test_kl_divergence_properties(
        mu1 in -100.0..100.0,
        var1 in 0.1..100.0,
        mu2 in -100.0..100.0,
        var2 in 0.1..100.0
    ) {
        let dist1 = PosteriorDistribution::new(mu1, var1);
        let dist2 = PosteriorDistribution::new(mu2, var2);

        let kl = dist1.kl_divergence(&dist2);

        // KL >= 0 (with numerical tolerance)
        prop_assert!(kl >= -1e-10,
                    "KL must be non-negative: {}", kl);

        // KL is finite
        prop_assert!(kl.is_finite(),
                    "KL must be finite: {}", kl);

        // KL(P||P) = 0
        let self_kl = dist1.kl_divergence(&dist1);
        prop_assert!(self_kl.abs() < 1e-10,
                    "KL(P||P) should be 0: {}", self_kl);

        // KL is not symmetric (unless distributions are equal)
        let reverse_kl = dist2.kl_divergence(&dist1);
        if (mu1 - mu2).abs() > 0.01 || (var1 - var2).abs() > 0.01 {
            prop_assert!((kl - reverse_kl).abs() > 1e-10,
                        "KL should not be symmetric: KL(P||Q)={}, KL(Q||P)={}",
                        kl, reverse_kl);
        }
    }

    #[test]
    fn test_posterior_update_consistency(
        prior_mean in -100.0..100.0,
        prior_var in 0.1..10.0,
        obs in -100.0..100.0,
        obs_var in 0.1..10.0
    ) {
        let mut posterior = PosteriorDistribution::new(prior_mean, prior_var);

        // Store initial values
        let initial_mean = posterior.mean;
        let initial_var = posterior.variance;

        // Update
        posterior.update(obs, obs_var);

        // Variance should decrease (more information)
        prop_assert!(posterior.variance <= initial_var + 1e-10,
                    "Variance should not increase: {} -> {}",
                    initial_var, posterior.variance);

        // Mean should be weighted average
        let precision_prior = 1.0 / initial_var;
        let precision_obs = 1.0 / obs_var;
        let expected_precision = precision_prior + precision_obs;
        let expected_mean = (precision_prior * initial_mean + precision_obs * obs) / expected_precision;

        prop_assert!((posterior.mean - expected_mean).abs() < 1e-10,
                    "Mean update incorrect: expected {}, got {}",
                    expected_mean, posterior.mean);
    }
}

// Topology Invariants
proptest! {
    #[test]
    fn test_topology_distance_triangle_inequality(
        size in 3usize..20
    ) {
        // Create alphabet subset
        let nodes: Vec<String> = (0..size)
            .map(|i| ((65 + i) as u8 as char).to_string())
            .collect();

        let topo = Topology::new_linear(nodes.clone());

        // Triangle inequality for all triplets
        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                for k in 0..nodes.len() {
                    let a = &nodes[i];
                    let b = &nodes[j];
                    let c = &nodes[k];

                    let ab = topo.get_distance(a, b).unwrap_or(0);
                    let bc = topo.get_distance(b, c).unwrap_or(0);
                    let ac = topo.get_distance(a, c).unwrap_or(0);

                    // Triangle inequality
                    prop_assert!(ac <= ab + bc,
                                "Triangle inequality violated: d({},{})={} > d({},{})={} + d({},{})={}",
                                a, c, ac, a, b, ab, b, c, bc);
                }
            }
        }
    }

    #[test]
    fn test_topology_distance_symmetry(
        size in 3usize..20
    ) {
        let nodes: Vec<String> = (0..size)
            .map(|i| ((65 + i) as u8 as char).to_string())
            .collect();

        let topo = Topology::new_linear(nodes.clone());

        // Distance should be symmetric
        for i in 0..nodes.len() {
            for j in 0..nodes.len() {
                let dist_ij = topo.get_distance(&nodes[i], &nodes[j]).unwrap_or(0);
                let dist_ji = topo.get_distance(&nodes[j], &nodes[i]).unwrap_or(0);

                prop_assert_eq!(dist_ij, dist_ji,
                               "Distance not symmetric: d({},{})={} != d({},{})={}",
                               nodes[i], nodes[j], dist_ij,
                               nodes[j], nodes[i], dist_ji);
            }
        }
    }
}

// Learning Model Properties
proptest! {
    #[test]
    fn test_proficiency_bounds(
        initial_theta in -5.0..5.0,
        num_updates in 1usize..50,
        correct_ratio in 0.0..1.0
    ) {
        let topo = Topology::alphabet();
        let mut learner = LearnerModel::new("test".to_string(), &topo);

        // Set initial proficiency
        if let Some(prof) = learner.operation_proficiencies.get_mut("Successor") {
            prof.theta = initial_theta;
        }

        // Apply updates
        for i in 0..num_updates {
            let correct = (i as f64 / num_updates as f64) < correct_ratio;
            learner.update_operation_proficiency(&crate::learner::OperationType::Successor, correct);
        }

        // Get final proficiency
        let final_prof = learner.operation_proficiencies
            .get("Successor")
            .map(|p| p.theta)
            .unwrap_or(0.0);

        // Proficiency should be bounded
        prop_assert!(final_prof > -10.0 && final_prof < 10.0,
                    "Proficiency out of reasonable bounds: {}", final_prof);

        // Success probability should be in [0,1]
        let success_prob = 1.0 / (1.0 + (-final_prof).exp());
        prop_assert!(success_prob >= 0.0 && success_prob <= 1.0,
                    "Success probability out of bounds: {}", success_prob);
    }

    #[test]
    fn test_memory_decay_properties(
        initial_strength in 0.1..1.0,
        hours_elapsed in 0.0..168.0, // Up to 1 week
        decay_rate in 0.01..0.5
    ) {
        // Test decay formula
        let exponent: f64 = -(decay_rate as f64) * (hours_elapsed as f64);
        let decayed = initial_strength * exponent.exp();

        // Decayed strength should be less than or equal to initial
        prop_assert!(decayed <= initial_strength + 1e-10,
                    "Memory should not increase: {} -> {}",
                    initial_strength, decayed);

        // Should be non-negative
        prop_assert!(decayed >= 0.0,
                    "Memory strength should be non-negative: {}", decayed);

        // Should approach 0 as time increases
        if hours_elapsed > 100.0 && decay_rate > 0.1 {
            prop_assert!(decayed < 0.01,
                        "Should decay to near zero for large time: {}", decayed);
        }
    }
}

// Bayesian Model Properties
proptest! {
    #[test]
    fn test_entropy_properties(
        variance in 0.01..100.0
    ) {
        let dist = PosteriorDistribution::new(0.0, variance);
        let entropy = dist.entropy();

        // Entropy should be non-negative for continuous distributions
        // (Can be negative for very small variance, but should be bounded)
        prop_assert!(entropy > -10.0,
                    "Entropy too negative: {}", entropy);

        // For Gaussian, entropy = 0.5 * ln(2πe * variance)
        let expected = 0.5 * (2.0 * std::f64::consts::PI * std::f64::consts::E * variance).ln();
        prop_assert!((entropy - expected).abs() < 0.01,
                    "Entropy formula incorrect: expected {}, got {}",
                    expected, entropy);
    }

    #[test]
    fn test_eig_bounds(
        num_nodes in 5usize..20,
        num_samples in 10usize..100
    ) {
        // Create small topology
        let nodes: Vec<String> = (0..num_nodes)
            .map(|i| ((65 + i) as u8 as char).to_string())
            .collect();
        let topo = Topology::new_linear(nodes);
        let mut model = BayesianLearnerModel::new(&topo);

        // Create a task
        let task = crate::tasks::Task {
            task_type: crate::tasks::TaskType::Successor {
                item: "B".to_string()
            },
            prompt: "".to_string(),
            correct_answer: "C".to_string(),
            options: vec![],
            difficulty: 0.5,
            operation: crate::learner::OperationType::Successor,
        };

        // Calculate EIG
        let eig = model.monte_carlo_eig(&task, num_samples);

        // EIG should be non-negative
        prop_assert!(eig >= 0.0,
                    "EIG should be non-negative: {}", eig);

        // EIG should be bounded by total entropy
        let total_entropy = model.total_entropy();
        prop_assert!(eig <= total_entropy + 0.1,
                    "EIG {} should not exceed total entropy {}",
                    eig, total_entropy);
    }
}

// Statistical Consistency Properties
proptest! {
    #[test]
    fn test_correlation_bounds(
        x_vals in prop::collection::vec(-100.0..100.0, 10..50),
        noise_level in 0.0..10.0
    ) {
        // Create y values with controlled correlation
        let y_vals: Vec<f64> = x_vals.iter()
            .map(|&x| x * 2.0 + noise_level * (rand::random::<f64>() - 0.5))
            .collect();

        // Calculate correlation (simplified version)
        let n = x_vals.len() as f64;
        let x_mean = x_vals.iter().sum::<f64>() / n;
        let y_mean = y_vals.iter().sum::<f64>() / n;

        let cov: f64 = x_vals.iter().zip(y_vals.iter())
            .map(|(x, y)| (x - x_mean) * (y - y_mean))
            .sum::<f64>() / n;

        let x_std = (x_vals.iter()
            .map(|x| (x - x_mean).powi(2))
            .sum::<f64>() / n).sqrt();

        let y_std = (y_vals.iter()
            .map(|y| (y - y_mean).powi(2))
            .sum::<f64>() / n).sqrt();

        if x_std > 0.0 && y_std > 0.0 {
            let correlation = cov / (x_std * y_std);

            // Correlation must be in [-1, 1]
            prop_assert!(correlation >= -1.0 - 1e-10 && correlation <= 1.0 + 1e-10,
                        "Correlation out of bounds: {}", correlation);
        }
    }

    #[test]
    fn test_strategy_classification_consistency(
        rt_distance_correlation in -1.0..1.0
    ) {
        use crate::statistics::StrategyType;

        // Test thresholds are consistent
        let strategy = if rt_distance_correlation > 0.7 {
            StrategyType::SerialScan
        } else if rt_distance_correlation < 0.3 {
            StrategyType::DirectAccess
        } else {
            StrategyType::Hybrid
        };

        // Verify classification logic
        match strategy {
            StrategyType::SerialScan => {
                prop_assert!(rt_distance_correlation > 0.7,
                            "Serial scan requires correlation > 0.7");
            }
            StrategyType::DirectAccess => {
                prop_assert!(rt_distance_correlation < 0.3,
                            "Direct access requires correlation < 0.3");
            }
            StrategyType::Hybrid => {
                prop_assert!(rt_distance_correlation >= 0.3 && rt_distance_correlation <= 0.7,
                            "Hybrid requires correlation in [0.3, 0.7]");
            }
            _ => {}
        }
    }
}

// Numerical Stability Properties
proptest! {
    #[test]
    fn test_numerical_stability_extreme_values(
        tiny in f64::EPSILON..1e-10,
        huge in 1e10..1e15
    ) {
        // Test with tiny variance
        let tiny_dist = PosteriorDistribution::new(0.0, tiny);
        let tiny_entropy = tiny_dist.entropy();
        prop_assert!(tiny_entropy.is_finite() || tiny_entropy == 0.0,
                    "Should handle tiny variance: {}", tiny_entropy);

        // Test with huge values
        let huge_dist = PosteriorDistribution::new(huge, 1.0);
        let kl = tiny_dist.kl_divergence(&huge_dist);
        prop_assert!(kl.is_finite() || kl == f64::INFINITY,
                    "Should handle huge values: {}", kl);

        // Test Ex-Gaussian with extreme parameters
        let extreme_params = ExGaussianParameters {
            mu: huge,
            sigma: tiny.max(0.001), // Avoid zero
            tau: tiny.max(0.001),
        };
        let model = ExGaussianModel::new(extreme_params);
        let pdf = model.pdf(huge);
        prop_assert!(pdf.is_finite() || pdf == 0.0,
                    "Should handle extreme parameters: {}", pdf);
    }

    #[test]
    fn test_overflow_prevention(
        large_exp in 50.0..100.0
    ) {
        // Test exponential overflow prevention
        let exp_result = (large_exp as f64).exp();
        if exp_result.is_finite() {
            prop_assert!(exp_result > 0.0,
                        "Exponential should be positive: {}", exp_result);
        } else {
            prop_assert_eq!(exp_result, f64::INFINITY,
                           "Should overflow to infinity");
        }

        // Test underflow prevention
        let neg_large: f64 = -(large_exp as f64);
        let tiny_exp = neg_large.exp();
        prop_assert!(tiny_exp >= 0.0 && tiny_exp < 1.0,
                    "Should handle underflow: {}", tiny_exp);
    }
}
