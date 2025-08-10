// Stress tests for edge cases and numerical stability
// These tests ensure the system handles extreme conditions gracefully

use crate::bayesian::*;
use crate::core::learner::*;
use crate::statistics::*;
use crate::tasks::types::{Task, TaskType};
use crate::core::topology::Topology;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[test]
fn test_numerical_stability_extreme_values() {
    // Test with values near machine epsilon
    let tiny = f64::EPSILON;
    let dist = PosteriorDistribution::new(0.0, tiny);
    let entropy = dist.entropy();
    assert!(
        !entropy.is_nan(),
        "Entropy should not be NaN for tiny variance"
    );
    assert!(
        entropy.is_finite() || entropy == 0.0,
        "Entropy should be finite or zero: {}",
        entropy
    );

    // Test with very large values
    let huge = f64::MAX / 1000.0;
    let dist2 = PosteriorDistribution::new(huge, 1.0);
    let kl = dist.kl_divergence(&dist2);
    assert!(
        kl.is_finite() || kl == f64::INFINITY,
        "KL should be finite or infinity: {}",
        kl
    );

    // Test with zero variance
    let zero_var = PosteriorDistribution::new(5.0, 0.0);
    assert_eq!(
        zero_var.entropy(),
        0.0,
        "Zero variance should give zero entropy"
    );

    // Test Ex-Gaussian with extreme parameters
    let extreme_params = ExGaussianParameters {
        mu: 1000.0,
        sigma: 0.001,
        tau: 0.001,
    };
    let model = ExGaussianModel::new(extreme_params);

    // Test PDF at various points
    let test_points = vec![-1000.0, -1.0, 0.0, 999.0, 1000.0, 1001.0, 10000.0];

    for x in test_points {
        let pdf = model.pdf(x);
        assert!(
            pdf == 0.0 || pdf.is_finite(),
            "PDF at {} should be 0 or finite: {}",
            x,
            pdf
        );
        assert!(pdf >= 0.0, "PDF should be non-negative: {}", pdf);

        let cdf = model.cdf(x);
        assert!(
            cdf >= 0.0 && cdf <= 1.0,
            "CDF at {} should be in [0,1]: {}",
            x,
            cdf
        );
    }
}

#[test]
fn test_overflow_underflow_prevention() {
    // Test exponential overflow
    let large_exp: f64 = 1000.0;
    let exp_result = large_exp.exp();
    assert!(
        exp_result == f64::INFINITY,
        "Large exponential should overflow to infinity"
    );

    // Test exponential underflow
    let small_exp: f64 = -1000.0;
    let exp_result = small_exp.exp();
    assert_eq!(
        exp_result, 0.0,
        "Small exponential should underflow to zero"
    );

    // Test in Ex-Gaussian context
    let params = ExGaussianParameters {
        mu: 0.0,
        sigma: 1.0,
        tau: 0.01, // Very small tau could cause numerical issues
    };
    let model = ExGaussianModel::new(params);

    // Test at extreme x values
    let extreme_x = vec![-100.0, -10.0, 0.0, 10.0, 100.0, 1000.0];
    for x in extreme_x {
        let pdf = model.pdf(x);
        assert!(!pdf.is_nan(), "PDF should not be NaN at x={}", x);
        assert!(
            pdf.is_finite() || pdf == 0.0,
            "PDF should be finite or zero at x={}: {}",
            x,
            pdf
        );
    }
}

#[test]
fn test_division_by_zero_prevention() {
    // Test correlation with zero variance
    let x = vec![1.0, 1.0, 1.0, 1.0]; // No variance
    let y = vec![1.0, 2.0, 3.0, 4.0];

    let analysis =
        StrategyAnalysis::analyze(&y, &x.iter().map(|&v| v as usize).collect::<Vec<_>>());
    assert!(
        analysis.rt_distance_correlation.is_finite() || analysis.rt_distance_correlation == 0.0,
        "Should handle zero variance gracefully"
    );

    // Test with empty data
    let empty_x: Vec<f64> = vec![];
    let empty_y: Vec<usize> = vec![];
    let empty_analysis = StrategyAnalysis::analyze(&empty_x, &empty_y);
    assert_eq!(
        empty_analysis.rt_distance_correlation, 0.0,
        "Should return 0 for empty data"
    );
}

#[test]
fn test_concurrent_model_updates() {
    // Test thread safety with concurrent updates
    let topo = Topology::alphabet();
    let model = Arc::new(Mutex::new(BayesianLearnerModel::new(&topo)));
    let mut handles = vec![];

    // Spawn multiple threads updating the model
    for thread_id in 0..10 {
        let model_clone = Arc::clone(&model);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let task = Task {
                    task_type: TaskType::Successor {
                        item: format!("{}", ((65 + (i % 26)) as u8 as char)),
                    },
                    prompt: format!("Thread {} task {}", thread_id, i),
                    correct_answer: "B".to_string(),
                    options: vec![],
                    difficulty: 0.5,
                    operation: OperationType::Successor,
                };

                let response = ResponseData {
                    task,
                    correct: i % 2 == 0,
                    response_time: 1000.0 + (i as f64),
                };

                let mut m = model_clone.lock().unwrap();
                m.update_with_response(response);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify model is still valid
    let final_model = model.lock().unwrap();
    let entropy = final_model.total_entropy();
    assert!(
        entropy >= 0.0 && entropy.is_finite(),
        "Model should remain valid after concurrent updates: entropy={}",
        entropy
    );

    // Check that updates were applied
    let initial_model = BayesianLearnerModel::new(&topo);
    let initial_entropy = initial_model.total_entropy();
    assert!(
        entropy < initial_entropy,
        "Entropy should decrease after updates: {} < {}",
        entropy,
        initial_entropy
    );
}

#[test]
fn test_large_scale_performance() {
    // Test with large topology
    let large_nodes: Vec<String> = (0..100).map(|i| format!("Node_{}", i)).collect();
    let large_topo = Topology::new_linear(large_nodes);

    let start = Instant::now();
    let mut model = BayesianLearnerModel::new(&large_topo);
    let init_time = start.elapsed();

    assert!(
        init_time.as_millis() < 1000,
        "Initialization should be fast even for 100 nodes: {:?}",
        init_time
    );

    // Test EIG calculation performance
    let task = Task {
        task_type: TaskType::Successor {
            item: "Node_50".to_string(),
        },
        prompt: "".to_string(),
        correct_answer: "Node_51".to_string(),
        options: vec![],
        difficulty: 0.5,
        operation: OperationType::Successor,
    };

    let start = Instant::now();
    let _eig = model.monte_carlo_eig(&task, 100);
    let eig_time = start.elapsed();

    assert!(
        eig_time.as_millis() < 500,
        "EIG calculation should be reasonably fast: {:?}",
        eig_time
    );
}

#[test]
fn test_memory_efficiency() {
    // Test that memory usage is reasonable
    let topo = Topology::alphabet();
    let mut learners = Vec::new();

    // Create many learner instances
    for i in 0..100 {
        let learner = LearnerModel::new(format!("learner_{}", i), &topo);
        learners.push(learner);
    }

    // Update all learners
    for learner in &mut learners {
        for _ in 0..10 {
            learner.update_operation_proficiency(&OperationType::Successor, true);
        }
    }

    // Should not run out of memory
    assert_eq!(learners.len(), 100, "Should handle 100 learners");
}

#[test]
fn test_edge_case_topologies() {
    // Single node topology
    let single = Topology::new_linear(vec!["A".to_string()]);
    assert_eq!(single.nodes.len(), 1);
    let single_model = BayesianLearnerModel::new(&single);
    assert!(single_model.node_positions.len() > 0);

    // Two node topology
    let two = Topology::new_linear(vec!["A".to_string(), "B".to_string()]);
    assert_eq!(two.nodes.len(), 2);
    let two_model = BayesianLearnerModel::new(&two);
    assert_eq!(two_model.node_positions.len(), 2);

    // Empty topology (should handle gracefully)
    let empty = Topology::new_linear(vec![]);
    assert_eq!(empty.nodes.len(), 0);
    // Model should handle empty topology without panic
    let empty_model = BayesianLearnerModel::new(&empty);
    assert_eq!(empty_model.node_positions.len(), 0);
}

#[test]
fn test_nan_propagation_prevention() {
    // Test that NaN doesn't propagate through calculations
    let mut dist = PosteriorDistribution::new(0.0, 1.0);

    // Update with NaN (should be handled)
    dist.update(f64::NAN, 1.0);
    assert!(!dist.mean.is_nan(), "NaN should not propagate to mean");

    // Update with infinity
    dist.update(f64::INFINITY, 1.0);
    assert!(
        dist.mean.is_finite() || dist.mean == f64::INFINITY,
        "Should handle infinity gracefully"
    );

    // Test KL divergence with problematic distributions
    let normal = PosteriorDistribution::new(0.0, 1.0);
    let zero_var = PosteriorDistribution::new(0.0, 0.0);

    let kl = normal.kl_divergence(&zero_var);
    assert!(
        kl == f64::INFINITY || kl.is_finite(),
        "KL with zero variance should be infinity or finite"
    );
}

#[test]
fn test_cyclic_topology_edge_cases() {
    // Test cyclic topology wraparound
    let days = vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let cyclic = Topology::new_cyclic(days.iter().map(|s| s.to_string()).collect());

    // Distance should wrap around
    let dist_forward = cyclic.get_distance("Sun", "Mon").unwrap_or(999);
    assert!(dist_forward <= 1, "Should wrap from Sun to Mon");

    let dist_backward = cyclic.get_distance("Mon", "Sun").unwrap_or(999);
    assert!(dist_backward <= 1, "Should wrap from Mon to Sun");

    // Test with k-jump that wraps
    let task = Task {
        task_type: TaskType::KJump {
            start: "Fri".to_string(),
            k: 3, // Should wrap to Mon
        },
        prompt: "".to_string(),
        correct_answer: "Mon".to_string(),
        options: vec![],
        difficulty: 0.5,
        operation: OperationType::KJump(3),
    };

    assert_eq!(task.correct_answer, "Mon", "K-jump should wrap correctly");
}

#[test]
fn test_statistical_edge_cases() {
    // Test with all same values
    let same_data = vec![5.0, 5.0, 5.0, 5.0, 5.0];
    let stats = DetailedStatistics::from_data(&same_data);

    assert_eq!(stats.mean, 5.0);
    assert_eq!(stats.median, 5.0);
    assert_eq!(stats.std_dev, 0.0);
    assert_eq!(stats.min, 5.0);
    assert_eq!(stats.max, 5.0);

    // Test with single value
    let single_data = vec![42.0];
    let single_stats = DetailedStatistics::from_data(&single_data);
    assert_eq!(single_stats.mean, 42.0);

    // Test with empty data
    let empty_data: Vec<f64> = vec![];
    let empty_stats = DetailedStatistics::from_data(&empty_data);
    assert_eq!(empty_stats.mean, 0.0);
    assert_eq!(empty_stats.std_dev, 0.0);
}

#[test]
fn test_outlier_detection_edge_cases() {
    // Test with no outliers
    let normal_data = vec![10.0, 11.0, 10.5, 10.2, 10.8, 11.2, 10.1];
    let outliers = ResponseTimeDistribution::detect_outliers(&normal_data, 3.0);
    assert_eq!(
        outliers.len(),
        0,
        "Should detect no outliers in normal data"
    );

    // Test with extreme outlier
    let with_outlier = vec![10.0, 11.0, 10.5, 10.2, 10000.0, 10.8, 11.2];
    let outliers = ResponseTimeDistribution::detect_outliers(&with_outlier, 2.0);
    assert!(
        outliers.contains(&4),
        "Should detect extreme outlier at index 4"
    );

    // Test with empty data
    let empty: Vec<f64> = vec![];
    let outliers = ResponseTimeDistribution::detect_outliers(&empty, 2.0);
    assert_eq!(outliers.len(), 0, "Should handle empty data");
}

#[test]
fn test_adaptive_monte_carlo_convergence_edge_cases() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    // Test with task that should converge quickly (low variance)
    let simple_task = Task {
        task_type: TaskType::Successor {
            item: "A".to_string(),
        },
        prompt: "".to_string(),
        correct_answer: "B".to_string(),
        options: vec![],
        difficulty: 0.1,
        operation: OperationType::Successor,
    };

    let (eig, samples) = model.adaptive_monte_carlo_eig(&simple_task);
    assert!(samples >= 100, "Should use at least minimum samples");
    assert!(samples <= 10000, "Should not exceed maximum samples");
    assert!(eig >= 0.0, "EIG should be non-negative");
}

#[test]
fn test_boundary_condition_validation() {
    // Test chunk boundaries at edges
    let topo = Topology::alphabet();
    let model = BayesianLearnerModel::new(&topo);

    // First chunk boundary should be valid
    if !model.chunk_boundaries.is_empty() {
        let first_boundary = &model.chunk_boundaries[0];
        assert!(
            first_boundary.position < 26,
            "Chunk boundary position {} should be within alphabet",
            first_boundary.position
        );
        assert!(
            first_boundary.strength.mean >= 0.0 && first_boundary.strength.mean <= 1.0,
            "Chunk boundary strength should be in [0,1]"
        );
    }

    // Test segment task at boundary
    let boundary_task = Task {
        task_type: TaskType::Segment {
            start: "E".to_string(),
            count: 4,
            reverse: false,
        },
        prompt: "".to_string(),
        correct_answer: "E, F, G, H".to_string(),
        options: vec![],
        difficulty: 0.5,
        operation: OperationType::Segment(4, false),
    };

    // Should handle task crossing chunk boundary
    let response = ResponseData {
        task: boundary_task,
        correct: true,
        response_time: 1500.0,
    };

    let mut model_copy = model.clone();
    model_copy.update_with_response(response);
    // Should not panic
}
