use crate::bayesian::*;
use crate::core::learner::OperationType;
use crate::tasks::types::{Task, TaskType};
use crate::core::topology::{Topology, TopologyType};

#[test]
fn test_bayesian_model_initialization() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    // Verify correct number of parameters
    assert_eq!(
        model.node_positions.len(),
        26,
        "Should have 26 node positions"
    );
    assert_eq!(
        model.operation_proficiencies.len(),
        6,
        "Should have 6 operation types"
    );

    // Verify initial posteriors match expected priors
    // Note: node_positions uses node IDs like "node_0", not sequential enumeration
    for (key, posterior) in &model.node_positions {
        // Extract node index from key (e.g., "node_0" -> 0)
        let node_index = key
            .strip_prefix("node_")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        // Initial mean should match node position from topology
        let expected_position = node_index as f64;
        assert!(
            (posterior.mean - expected_position).abs() < 2.0,
            "Node {} initial mean {} should be near position {}",
            key,
            posterior.mean,
            expected_position
        );

        // Initial variance should be 1.0 (moderate uncertainty)
        assert!(
            (posterior.variance - 1.0).abs() < 0.1,
            "Initial variance {} should be near 1.0",
            posterior.variance
        );

        // Confidence should be 1/(1+variance) ≈ 0.5 for variance=1
        assert!(
            (posterior.confidence - 0.5).abs() < 0.1,
            "Initial confidence {} should be near 0.5",
            posterior.confidence
        );
    }

    // Verify operation proficiencies start neutral
    for (_op, prof) in &model.operation_proficiencies {
        assert!(
            prof.mean.abs() < 0.1,
            "Initial proficiency mean should be near 0"
        );
        assert!(
            (prof.variance - 1.0).abs() < 0.1,
            "Initial proficiency variance should be near 1"
        );
    }

    // Verify chunk boundaries for linear topology
    if matches!(topo.topology_type, crate::core::topology::TopologyType::Linear) {
        assert_eq!(
            model.chunk_boundaries.len(),
            3,
            "Should have 3 chunk boundaries"
        );
        let expected_positions = vec![6, 13, 19];
        for (i, boundary) in model.chunk_boundaries.iter().enumerate() {
            assert_eq!(
                boundary.position, expected_positions[i],
                "Chunk boundary {} at wrong position",
                i
            );
        }
    }
}

#[test]
fn test_monte_carlo_eig_positive() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    let task = Task {
        task_type: TaskType::Successor {
            item: "C".to_string(),
        },
        prompt: "What comes after C?".to_string(),
        correct_answer: "D".to_string(),
        options: vec!["B", "C", "D", "E"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        difficulty: 0.3,
        operation: OperationType::Successor,
    };

    // Test with more samples for better convergence
    let eig_100 = model.monte_carlo_eig(&task, 100);
    let eig_1000 = model.monte_carlo_eig(&task, 1000);

    // EIG should be non-negative (information cannot be lost)
    assert!(
        eig_100 >= 0.0,
        "EIG with 100 samples was negative: {}",
        eig_100
    );
    assert!(
        eig_1000 >= 0.0,
        "EIG with 1000 samples was negative: {}",
        eig_1000
    );

    // EIG should be bounded by total entropy
    let total_entropy = model.total_entropy();
    assert!(
        eig_100 <= total_entropy,
        "EIG {} exceeds total entropy {}",
        eig_100,
        total_entropy
    );
    assert!(
        eig_1000 <= total_entropy,
        "EIG {} exceeds total entropy {}",
        eig_1000,
        total_entropy
    );

    // More samples should give more stable estimate (not necessarily larger)
    // But they should be in the same ballpark
    assert!(
        (eig_100 - eig_1000).abs() < eig_100 * 0.5,
        "EIG estimates should be similar: 100 samples={}, 1000 samples={}",
        eig_100,
        eig_1000
    );

    // EIG should be positive for an informative task
    assert!(
        eig_1000 > 0.001,
        "EIG should be positive for informative task, got {}",
        eig_1000
    );
}

#[test]
fn test_posterior_update_reduces_uncertainty() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    // Get initial variance for a node
    let initial_var = model.node_positions["node_0"].variance;

    // Provide correct response
    let response = ResponseData {
        task: Task {
            task_type: TaskType::Successor {
                item: "A".to_string(),
            },
            prompt: "What comes after A?".to_string(),
            correct_answer: "B".to_string(),
            options: vec!["A", "B", "C", "D"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            difficulty: 0.3,
            operation: OperationType::Successor,
        },
        correct: true,
        response_time: 1000.0,
    };

    model.update_with_response(response);

    // Variance should decrease (uncertainty reduced)
    let updated_var = model.node_positions["node_0"].variance;
    assert!(
        updated_var < initial_var,
        "Variance should decrease after update: {} -> {}",
        initial_var,
        updated_var
    );
}

#[test]
fn test_eig_high_for_uncertain_tasks() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    // Create two tasks - one for well-known nodes, one for uncertain
    let certain_task = Task {
        task_type: TaskType::Successor {
            item: "A".to_string(),
        },
        prompt: "".to_string(),
        correct_answer: "B".to_string(),
        options: vec![],
        difficulty: 0.1,
        operation: OperationType::Successor,
    };

    let uncertain_task = Task {
        task_type: TaskType::Successor {
            item: "X".to_string(),
        },
        prompt: "".to_string(),
        correct_answer: "Y".to_string(),
        options: vec![],
        difficulty: 0.9,
        operation: OperationType::Successor,
    };

    // Calculate initial EIG for both tasks (both uncertain)
    let initial_certain_eig = model.monte_carlo_eig(&certain_task, 500);
    let initial_uncertain_eig = model.monte_carlo_eig(&uncertain_task, 500);

    // Initially, both should have similar EIG (both uncertain)
    assert!(
        (initial_certain_eig - initial_uncertain_eig).abs() < initial_certain_eig * 0.5,
        "Initially, both tasks should have similar EIG: A={}, X={}",
        initial_certain_eig,
        initial_uncertain_eig
    );

    // Now reduce uncertainty about A through multiple observations
    for _ in 0..10 {
        let response = ResponseData {
            task: certain_task.clone(),
            correct: true,
            response_time: 1000.0,
        };
        model.update_with_response(response);
    }

    // Calculate EIG again after learning
    let certain_eig_after = model.monte_carlo_eig(&certain_task, 500);
    let uncertain_eig_after = model.monte_carlo_eig(&uncertain_task, 500);

    // After learning, certain task should have much lower EIG
    assert!(
        certain_eig_after < initial_certain_eig * 0.5,
        "EIG for learned task should decrease: {} -> {}",
        initial_certain_eig,
        certain_eig_after
    );

    // Uncertain task should have significantly higher EIG than certain task
    assert!(
        uncertain_eig_after > certain_eig_after * 2.0,
        "Uncertain task EIG {} should be much higher (>2x) than certain task EIG {}",
        uncertain_eig_after,
        certain_eig_after
    );
}

#[test]
fn test_task_ranking_by_eig() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    let tasks = vec![
        Task {
            task_type: TaskType::Successor {
                item: "A".to_string(),
            },
            prompt: "".to_string(),
            correct_answer: "B".to_string(),
            options: vec![],
            difficulty: 0.1,
            operation: OperationType::Successor,
        },
        Task {
            task_type: TaskType::PairwiseOrder {
                a: "M".to_string(),
                b: "N".to_string(),
            },
            prompt: "".to_string(),
            correct_answer: "M".to_string(),
            options: vec![],
            difficulty: 0.5,
            operation: OperationType::PairwiseOrder,
        },
        Task {
            task_type: TaskType::Segment {
                start: "X".to_string(),
                count: 3,
                reverse: false,
            },
            prompt: "".to_string(),
            correct_answer: "X, Y, Z".to_string(),
            options: vec![],
            difficulty: 0.8,
            operation: OperationType::Segment(3, false),
        },
    ];

    let ranked = model.rank_tasks_by_eig(tasks.clone());

    // Should return same number of tasks
    assert_eq!(ranked.len(), tasks.len());

    // Tasks should be sorted by EIG (descending)
    for i in 1..ranked.len() {
        assert!(
            ranked[i - 1].1 >= ranked[i].1,
            "Tasks not properly sorted by EIG: {} < {}",
            ranked[i - 1].1,
            ranked[i].1
        );
    }
}

#[test]
fn test_adaptive_observation_variance() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    // Fast, correct response should result in lower entropy after update
    let fast_correct = ResponseData {
        task: Task {
            task_type: TaskType::Successor {
                item: "A".to_string(),
            },
            prompt: "".to_string(),
            correct_answer: "B".to_string(),
            options: vec![],
            difficulty: 0.1,
            operation: OperationType::Successor,
        },
        correct: true,
        response_time: 500.0,
    };

    let initial_entropy = model.total_entropy();
    model.update_with_response(fast_correct);
    let entropy_after_fast = model.total_entropy();

    // Entropy should decrease
    assert!(
        entropy_after_fast < initial_entropy,
        "Entropy should decrease after fast correct response"
    );
}

#[test]
fn test_sampled_model_usage() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    // Test that we can rank tasks (which internally uses sampling)
    let tasks = vec![Task {
        task_type: TaskType::PairwiseOrder {
            a: "A".to_string(),
            b: "B".to_string(),
        },
        prompt: "".to_string(),
        correct_answer: "A".to_string(),
        options: vec![],
        difficulty: 0.3,
        operation: OperationType::PairwiseOrder,
    }];

    let ranked = model.rank_tasks_by_eig(tasks);

    // Should return valid EIG values
    assert_eq!(ranked.len(), 1);
    let (_task, eig) = &ranked[0];
    assert!(
        *eig >= 0.0 && *eig < 20.0,
        "EIG should be reasonable: {}",
        eig
    );
}

#[test]
fn test_chunk_boundary_updates() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    // Segment task crossing chunk boundary
    let response = ResponseData {
        task: Task {
            task_type: TaskType::Segment {
                start: "E".to_string(),
                count: 4,
                reverse: false,
            },
            prompt: "".to_string(),
            correct_answer: "E, F, G, H".to_string(),
            options: vec![],
            difficulty: 0.4,
            operation: OperationType::Segment(4, false),
        },
        correct: true,
        response_time: 2500.0, // Slow response suggests chunk boundary
    };

    model.update_with_response(response);

    // Check that chunk boundaries were considered
    assert!(
        !model.chunk_boundaries.is_empty() || topo.topology_type != TopologyType::Linear,
        "Chunk boundaries should exist for linear topology"
    );
}

#[test]
fn test_entropy_calculation() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    let initial_entropy = model.total_entropy();

    // Entropy should be positive for uncertain model
    assert!(
        initial_entropy > 0.0,
        "Initial entropy should be positive: {}",
        initial_entropy
    );

    // For 26 nodes + 6 operations + 3 chunk boundaries, each with variance ~1
    // Differential entropy of Gaussian with variance 1 is 0.5 * ln(2πe) ≈ 1.42
    // So total should be around 35 * 1.42 ≈ 50
    let expected_entropy = 35.0 * 1.42;
    assert!(
        initial_entropy > expected_entropy * 0.5 && initial_entropy < expected_entropy * 2.0,
        "Initial entropy {} should be near expected {}",
        initial_entropy,
        expected_entropy
    );

    // Provide information to reduce entropy
    let task = Task {
        task_type: TaskType::Successor {
            item: "A".to_string(),
        },
        prompt: "".to_string(),
        correct_answer: "B".to_string(),
        options: vec![],
        difficulty: 0.3,
        operation: OperationType::Successor,
    };

    for _ in 0..5 {
        model.update_with_response(ResponseData {
            task: task.clone(),
            correct: true,
            response_time: 1000.0,
        });
    }

    let entropy_after = model.total_entropy();

    // Entropy should decrease with information
    assert!(
        entropy_after < initial_entropy,
        "Entropy should decrease with information: {} -> {}",
        initial_entropy,
        entropy_after
    );

    // But should still be positive (not all uncertainty removed)
    assert!(
        entropy_after > 0.0,
        "Entropy should remain positive: {}",
        entropy_after
    );
}

#[test]
fn test_model_comparison_metrics() {
    use crate::bayesian::ModelComparisonMetrics;

    // Model 1: Better fit, more complex
    let model1 = ModelComparisonMetrics::new(-50.0, 10, 100);

    // Model 2: Worse fit, simpler
    let model2 = ModelComparisonMetrics::new(-60.0, 5, 100);

    // AIC should penalize complexity
    let aic1 = model1.aic();
    let aic2 = model2.aic();
    assert!(
        aic1 < aic2 + 5.0,
        "Model 1 has better likelihood, should have competitive AIC"
    );

    // BIC penalizes complexity more heavily
    let bic1 = model1.bic();
    let bic2 = model2.bic();
    assert!(
        bic1 > aic1 - aic2 + bic2,
        "BIC penalty should be stronger than AIC"
    );

    // AICc for small samples
    let model_small = ModelComparisonMetrics::new(-20.0, 8, 20);
    let aicc = model_small.aicc();
    let aic = model_small.aic();
    assert!(aicc > aic, "AICc should add correction for small samples");

    // Evidence ratio
    let evidence = model1.evidence_ratio(&model2);
    assert!(evidence > 1.0, "Model 1 should have higher evidence");
}

#[test]
fn test_dic_calculation() {
    use crate::bayesian::DIC;

    let dic = DIC::new(100.0, 90.0);

    // Effective parameters
    let p_eff = dic.effective_parameters();
    assert_eq!(p_eff, 10.0, "Effective parameters = mean_dev - dev_at_mean");

    // DIC calculation
    let dic_value = dic.dic();
    assert_eq!(dic_value, 110.0, "DIC = 2*mean_dev - dev_at_mean");
}

#[test]
fn test_waic_calculation() {
    use crate::bayesian::WAIC;

    let waic = WAIC::new(-50.0, 5.0);

    // WAIC value
    let waic_value = waic.waic();
    assert_eq!(waic_value, -2.0 * (-50.0 - 5.0), "WAIC = -2(lppd - p_waic)");
    assert_eq!(waic_value, 110.0);

    // Standard error
    let variances = vec![0.1, 0.2, 0.15, 0.1];
    let se = waic.se(&variances);
    assert!(se > 0.0, "Standard error should be positive");
}

#[test]
fn test_aic_weights() {
    use crate::bayesian::ModelComparisonMetrics;

    // Two competing models
    let model1 = ModelComparisonMetrics::new(-50.0, 5, 100);
    let model2 = ModelComparisonMetrics::new(-52.0, 5, 100);

    // Model 1 is better (higher likelihood)
    let weight1 = model1.aic_weight(&model2);
    assert!(weight1 > 0.5, "Better model should have weight > 0.5");

    // Weights should sum to 1
    let weight2 = model2.aic_weight(&model1);
    assert!(
        (weight1 + weight2 - 1.0).abs() < 1e-10,
        "Weights should sum to 1"
    );
}

#[test]
fn test_posterior_predictive_check() {
    let topo = Topology::alphabet();
    let mut model = BayesianLearnerModel::new(&topo);

    // Add some response data
    for i in 0..20 {
        let task = Task {
            task_type: TaskType::Successor {
                item: char::from(b'A' + (i % 26) as u8).to_string(),
            },
            prompt: "test".to_string(),
            correct_answer: "test".to_string(),
            options: vec![],
            difficulty: 0.3,
            operation: OperationType::Successor,
        };

        model.update_with_response(ResponseData {
            task,
            correct: i % 3 != 0, // ~67% accuracy
            response_time: 1000.0 + (i as f64 * 50.0),
        });
    }

    // Run posterior predictive check
    let ppc = model.posterior_predictive_check(100);

    // Check that we have replications
    assert_eq!(ppc.n_replications, 100);
    assert_eq!(ppc.replicated_statistics.len(), 100);

    // Calculate p-values
    let p_values = ppc.calculate_p_values();

    // P-values should be between 0 and 1
    assert!(p_values.accuracy >= 0.0 && p_values.accuracy <= 1.0);
    assert!(p_values.mean_rt >= 0.0 && p_values.mean_rt <= 1.0);
    assert!(p_values.rt_std >= 0.0 && p_values.rt_std <= 1.0);
    assert!(p_values.autocorrelation >= 0.0 && p_values.autocorrelation <= 1.0);

    // Check model adequacy
    let adequacy = ppc.check_model_adequacy(0.05);

    // At least some statistics should be adequate (not all may be due to randomness)
    let adequate_count = [
        adequacy.accuracy_adequate,
        adequacy.mean_rt_adequate,
        adequacy.rt_std_adequate,
        adequacy.autocorr_adequate,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    assert!(
        adequate_count >= 2,
        "At least half of statistics should be adequate"
    );
}

#[test]
fn test_posterior_predictive_statistics() {
    use crate::bayesian::TestStatistics;

    // Create known test statistics
    let stats = TestStatistics {
        accuracy: 0.75,
        mean_rt: 1200.0,
        rt_std: 300.0,
        autocorrelation: 0.1,
    };

    // Verify all fields are accessible
    assert_eq!(stats.accuracy, 0.75);
    assert_eq!(stats.mean_rt, 1200.0);
    assert_eq!(stats.rt_std, 300.0);
    assert_eq!(stats.autocorrelation, 0.1);
}
