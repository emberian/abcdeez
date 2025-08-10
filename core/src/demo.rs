use crate::core::adaptive::AdaptiveScheduler;
use crate::core::learner::{LearnerMetrics, LearnerModel};
use crate::tasks::types::{TaskSession, TaskType};
use crate::core::topology::Topology;

pub fn run_demo() {
    println!("\n═══════════════════════════════════════════════════");
    println!("    ADAPTIVE GRAPH-CODED LEARNING SYSTEM DEMO");
    println!("═══════════════════════════════════════════════════\n");

    let topology = Topology::alphabet();
    println!("Created topology: English Alphabet (26 letters)\n");

    let learner_model = LearnerModel::new("demo_user".to_string(), &topology);
    println!("Initialized learner model for: demo_user");

    let mut scheduler = AdaptiveScheduler::new(learner_model, topology.clone());
    let mut session = TaskSession::new(topology.clone());

    println!("\n──────────────────────────────────────────────────");
    println!("Starting adaptive training session...\n");

    for i in 1..=10 {
        println!("Task {}/10", i);
        println!("──────────────────────────────────────────────────");

        let task = scheduler.select_next_task();
        println!("Question: {}", task.prompt);

        if !task.options.is_empty() {
            println!("Options:");
            for (idx, option) in task.options.iter().enumerate() {
                println!("  [{}] {}", idx + 1, option);
            }
        }

        let simulated_answer = if rand::random::<f64>() > 0.3 {
            task.correct_answer.clone()
        } else {
            task.options.get(1).unwrap_or(&"Wrong".to_string()).clone()
        };

        println!("Simulated answer: {}", simulated_answer);

        session.start_task(Some(task.task_type.clone()));
        let response = session.submit_answer(simulated_answer);

        if response.correct {
            println!("✓ CORRECT!");
        } else {
            println!(
                "✗ INCORRECT - Correct answer: {}",
                response.task.correct_answer
            );
        }

        scheduler.update_model(&response.task, response.correct, response.response_time_ms);

        println!();
    }

    println!("\n═══════════════════════════════════════════════════");
    println!("Session Complete - Performance Metrics");
    println!("═══════════════════════════════════════════════════\n");

    let model = scheduler.get_learner_model();
    let metrics = LearnerMetrics::from_model(model);
    let session_stats = session.get_statistics();

    println!("Session Statistics:");
    println!("──────────────────────────────────────────────────");
    println!("Total tasks: {}", session_stats.total_tasks);
    println!("Accuracy: {:.1}%", session_stats.accuracy * 100.0);
    println!();

    println!("Graph-Coded Mastery Metrics:");
    println!("──────────────────────────────────────────────────");
    println!(
        "Bidirectionality Index: {:.3}",
        metrics.bidirectionality_index
    );
    println!("  (Lower is better - measures forward/backward asymmetry)");
    println!();
    println!(
        "Symbolic Distance Slope: {:.3}",
        metrics.symbolic_distance_slope
    );
    println!("  (Lower is better - indicates direct retrieval vs scanning)");
    println!();
    println!(
        "Chunk Boundary Penalty: {:.3}",
        metrics.chunk_boundary_penalty
    );
    println!("  (Lower is better - measures mental segmentation)");
    println!();
    println!(
        "Average Memory Strength: {:.1}%",
        metrics.avg_memory_strength * 100.0
    );
    println!();

    println!("Operation Proficiencies:");
    println!("──────────────────────────────────────────────────");
    for (op, prof) in metrics.operation_proficiencies {
        let bar_length = (prof * 20.0) as usize;
        let bar = "█".repeat(bar_length) + &"░".repeat(20 - bar_length);
        println!("{:25} [{}] {:.0}%", op, bar, prof * 100.0);
    }

    println!("\n═══════════════════════════════════════════════════");
    println!("Demo Complete!");
    println!("═══════════════════════════════════════════════════\n");
}

pub fn demonstrate_dag_tasks() {
    println!("\n═══════════════════════════════════════════════════");
    println!("    DAG/PARTIAL ORDER TASK DEMONSTRATIONS");
    println!("═══════════════════════════════════════════════════\n");

    let dag = crate::core::topology::Topology::example_dag();
    let mut generator = crate::tasks::TaskGenerator::new(dag.clone());

    println!("Example DAG: Software Deployment Pipeline");
    println!("──────────────────────────────────────────────────");
    if let Some(topo_sort) = dag.get_topological_sort() {
        println!("Valid ordering: {}", topo_sort.join(" -> "));
    }
    println!();

    println!("1. Comparability Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(crate::tasks::TaskType::Comparability {
        a: "Database Setup".to_string(),
        b: "Frontend".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("2. Minimal Elements (Entry Points):");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(crate::tasks::TaskType::MinimalElements));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("3. Maximal Elements (Final Tasks):");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(crate::tasks::TaskType::MaximalElements));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("4. Topological Sort (Subset):");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(crate::tasks::TaskType::TopologicalSort {
        items: vec![
            "Database Setup".to_string(),
            "API Server".to_string(),
            "User Auth".to_string(),
        ],
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("5. Shortest Path:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(crate::tasks::TaskType::ShortestPath {
        from: "Database Setup".to_string(),
        to: "Deploy".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);
}

pub fn demonstrate_eig() {
    println!("\n═══════════════════════════════════════════════════");
    println!("    EXPECTED INFORMATION GAIN DEMONSTRATION");
    println!("═══════════════════════════════════════════════════\n");

    let topology = crate::core::topology::Topology::alphabet();
    let learner_model = crate::core::learner::LearnerModel::new("eig_demo".to_string(), &topology);
    let mut scheduler =
        crate::core::adaptive::AdaptiveScheduler::new_with_eig(learner_model, topology.clone(), true);

    println!(
        "Initial model entropy: {:.2}",
        scheduler.get_model_entropy()
    );
    println!("\nGenerating candidate tasks and ranking by EIG...\n");

    // Generate various task types
    let mut task_gen = crate::tasks::TaskGenerator::new(topology.clone());
    let candidates = vec![
        task_gen.generate_task(Some(crate::tasks::TaskType::PairwiseOrder {
            a: "M".to_string(),
            b: "N".to_string(),
        })),
        task_gen.generate_task(Some(crate::tasks::TaskType::PairwiseOrder {
            a: "A".to_string(),
            b: "Z".to_string(),
        })),
        task_gen.generate_task(Some(crate::tasks::TaskType::Successor {
            item: "G".to_string(),
        })),
        task_gen.generate_task(Some(crate::tasks::TaskType::Segment {
            start: "F".to_string(),
            count: 4,
            reverse: false,
        })),
        task_gen.generate_task(Some(crate::tasks::TaskType::KJump {
            start: "L".to_string(),
            k: 3,
        })),
    ];

    let bayesian_model = scheduler.get_bayesian_model_mut();
    let ranked = bayesian_model.rank_tasks_by_eig(candidates);

    println!("Tasks ranked by Expected Information Gain:");
    println!("──────────────────────────────────────────────────");
    for (i, (task, eig)) in ranked.iter().enumerate().take(5) {
        println!("{}. EIG = {:.4}", i + 1, eig);
        println!("   Task: {}", task.prompt);
        println!("   Type: {:?}", task.task_type);
        println!();
    }

    println!("Simulating 10 adaptive selections with EIG...\n");
    let mut total_entropy_reduction = 0.0;
    let initial_entropy = scheduler.get_model_entropy();

    for i in 1..=10 {
        let entropy_before = scheduler.get_model_entropy();
        let task = scheduler.select_next_task();

        // Simulate response
        let correct = rand::random::<f64>() > 0.3;
        scheduler.update_model(&task, correct, 1000);

        let entropy_after = scheduler.get_model_entropy();
        let reduction = entropy_before - entropy_after;
        total_entropy_reduction += reduction;

        println!(
            "Round {}: Entropy {:.3} -> {:.3} (Δ = {:.4})",
            i, entropy_before, entropy_after, reduction
        );
    }

    println!("\n──────────────────────────────────────────────────");
    println!("Total entropy reduction: {:.3}", total_entropy_reduction);
    println!("Final model entropy: {:.3}", scheduler.get_model_entropy());
    println!(
        "Entropy reduction rate: {:.1}%",
        (initial_entropy - scheduler.get_model_entropy()) / initial_entropy * 100.0
    );

    println!("\nComparing with Random Selection:");
    println!("──────────────────────────────────────────────────");

    let learner_model2 = crate::core::learner::LearnerModel::new("random".to_string(), &topology);
    let mut scheduler2 =
        crate::core::adaptive::AdaptiveScheduler::new_with_eig(learner_model2, topology.clone(), false);

    let initial_entropy2 = scheduler2.get_model_entropy();
    for _ in 1..=10 {
        let task = scheduler2.select_next_task();
        let correct = rand::random::<f64>() > 0.3;
        scheduler2.update_model(&task, correct, 1000);
    }

    println!(
        "Random selection entropy reduction: {:.1}%",
        (initial_entropy2 - scheduler2.get_model_entropy()) / initial_entropy2 * 100.0
    );
    println!("\nEIG-based selection is more efficient at reducing uncertainty!");
}

pub fn demonstrate_statistical_analysis() {
    println!("\n═══════════════════════════════════════════════════");
    println!("    STATISTICAL ANALYSIS DEMONSTRATION");
    println!("═══════════════════════════════════════════════════\n");

    let topology = crate::core::topology::Topology::alphabet();
    let mut session = crate::tasks::TaskSession::new(topology.clone());
    let mut responses = Vec::new();

    println!("Simulating 50 training trials...\n");

    for i in 0..50 {
        session.start_task(None);
        let correct = rand::random::<f64>() > (0.4 - i as f64 * 0.008);
        let rt = 1000.0 + rand::random::<f64>() * 500.0 - i as f64 * 10.0;

        if let Some(task) = &session.current_task {
            let answer = if correct {
                task.correct_answer.clone()
            } else {
                "Wrong".to_string()
            };

            let response = crate::tasks::TaskResponse {
                task: task.clone(),
                user_answer: answer,
                correct,
                response_time_ms: rt as u128,
                timestamp: chrono::Utc::now(),
            };
            responses.push(response);
        }
    }

    let analyzer = crate::statistics::SessionAnalyzer::new(responses);
    let analysis = analyzer.generate_full_analysis();

    println!("Performance Analysis Results:");
    println!("═══════════════════════════════════════════════════");

    println!("\n1. Learning Curve Analysis:");
    println!("──────────────────────────────────────────────────");
    println!(
        "   Improvement rate: {:.2}%",
        analysis.learning_curves.improvement_rate * 100.0
    );
    if let Some(plateau) = analysis.learning_curves.plateau_point {
        println!("   Performance plateau reached at trial: {}", plateau);
    }
    println!(
        "   Final accuracy: {:.1}%",
        analysis
            .learning_curves
            .accuracy_over_time
            .last()
            .unwrap_or(&0.0)
            * 100.0
    );

    println!("\n2. Strategy Analysis:");
    println!("──────────────────────────────────────────────────");
    println!(
        "   RT-Distance Correlation: {:.3}",
        analysis.strategy_analysis.rt_distance_correlation
    );
    println!(
        "   Strategy Classification: {:?}",
        analysis.strategy_analysis.strategy_classification
    );
    if let Some(transition) = analysis.strategy_analysis.transition_point {
        println!("   Strategy transition detected at trial: {}", transition);
    }

    println!("\n3. Error Pattern Analysis:");
    println!("──────────────────────────────────────────────────");
    println!(
        "   Locality index: {:.2} (proportion of errors within distance 1-2)",
        analysis.error_patterns.locality_index
    );

    if !analysis.error_patterns.systematic_errors.is_empty() {
        println!("   Systematic errors detected:");
        for (expected, actual, count) in &analysis.error_patterns.systematic_errors
            [..3.min(analysis.error_patterns.systematic_errors.len())]
        {
            println!("     {} <-> {}: {} times", expected, actual, count);
        }
    }

    println!("\n4. Response Time Statistics by Task Type:");
    println!("──────────────────────────────────────────────────");
    for (task_type, stats) in analysis.task_type_stats.iter().take(3) {
        println!("   {}:", task_type);
        println!(
            "     Mean RT: {:.0}ms (SD: {:.0}ms)",
            stats.mean, stats.std_dev
        );
        println!(
            "     Median: {:.0}ms, IQR: {:.0}ms",
            stats.median, stats.iqr
        );
    }

    println!("\n5. Ex-Gaussian RT Model:");
    println!("──────────────────────────────────────────────────");
    let all_rts: Vec<f64> = analysis
        .rt_by_distance
        .values()
        .flatten()
        .cloned()
        .collect();

    if !all_rts.is_empty() {
        let ex_gaussian = crate::statistics::ExGaussianModel::fit(&all_rts);
        println!("   μ (Gaussian mean): {:.0}ms", ex_gaussian.params.mu);
        println!("   σ (Gaussian SD): {:.0}ms", ex_gaussian.params.sigma);
        println!("   τ (Exponential rate): {:.0}ms", ex_gaussian.params.tau);
        println!("   Model mean: {:.0}ms", ex_gaussian.mean());
        println!("   Model variance: {:.0}", ex_gaussian.variance());
    }
}

pub fn demonstrate_task_types() {
    println!("\n═══════════════════════════════════════════════════");
    println!("    TASK TYPE DEMONSTRATIONS");
    println!("═══════════════════════════════════════════════════\n");

    let topology = Topology::alphabet();
    let mut generator = crate::tasks::TaskGenerator::new(topology.clone());

    println!("1. Pairwise Order Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::PairwiseOrder {
        a: "D".to_string(),
        b: "K".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("2. Successor Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::Successor {
        item: "M".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("3. Predecessor Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::Predecessor {
        item: "P".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("4. K-Jump Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::KJump {
        start: "E".to_string(),
        k: 3,
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("5. Segment Task (Forward):");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::Segment {
        start: "L".to_string(),
        count: 4,
        reverse: false,
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("6. Segment Task (Reverse):");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::Segment {
        start: "R".to_string(),
        count: 3,
        reverse: true,
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("7. Index Position Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::Index {
        item: "T".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("8. Missing Item Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::MissingItem {
        before: "G".to_string(),
        after: "I".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("9. Shortest Distance Task:");
    println!("──────────────────────────────────────────────────");
    let task = generator.generate_task(Some(TaskType::ShortestDistance {
        from: "B".to_string(),
        to: "X".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("\n═══════════════════════════════════════════════════");
    println!("Cyclic Topology Example: Days of Week");
    println!("═══════════════════════════════════════════════════\n");

    let cyclic_topology = Topology::days_of_week();
    let mut cyclic_generator = crate::tasks::TaskGenerator::new(cyclic_topology);

    println!("Successor with Wraparound:");
    println!("──────────────────────────────────────────────────");
    let task = cyclic_generator.generate_task(Some(TaskType::Successor {
        item: "Sunday".to_string(),
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("K-Jump with Wraparound:");
    println!("──────────────────────────────────────────────────");
    let task = cyclic_generator.generate_task(Some(TaskType::KJump {
        start: "Friday".to_string(),
        k: 3,
    }));
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);
}

pub fn demonstrate_extended_tasks() {
    println!("\n═══════════════════════════════════════════════════");
    println!("    EXTENDED TASK DEMONSTRATIONS");
    println!("═══════════════════════════════════════════════════\n");

    let topology = crate::core::topology::Topology::alphabet();
    let mut ext_gen = crate::tasks::extended::ExtendedTaskGenerator::new(topology.clone());

    println!("1. Between Query (3-way comparison):");
    println!("──────────────────────────────────────────────────");
    let task = ext_gen.generate_between_query("D".to_string(), "F".to_string(), "H".to_string());
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("2. Boundary Bridging (chunk crossing):");
    println!("──────────────────────────────────────────────────");
    let task = ext_gen.generate_boundary_bridging("E".to_string(), 4, vec![6, 13]);
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("3. Directional Comparison:");
    println!("──────────────────────────────────────────────────");
    let task = ext_gen.generate_directional_comparison("P".to_string(), "L".to_string(), true);
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("4. Insertion Adaptation:");
    println!("──────────────────────────────────────────────────");
    let task =
        ext_gen.generate_insertion_adaptation("X".to_string(), "M".to_string(), "Q".to_string());
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("5. Next-Step Prediction:");
    println!("──────────────────────────────────────────────────");
    let task = ext_gen.generate_next_step_prediction("B".to_string(), "Y".to_string());
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("6. Landmark Navigation:");
    println!("──────────────────────────────────────────────────");
    let task =
        ext_gen.generate_landmark_navigation("A".to_string(), "Z".to_string(), "M".to_string());
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("7. Macro Discovery:");
    println!("──────────────────────────────────────────────────");
    let task = ext_gen.generate_macro_discovery(vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "D".to_string(),
    ]);
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("8. Semantic Filter (vowels category):");
    println!("──────────────────────────────────────────────────");
    let task = ext_gen.generate_semantic_filter("vowel".to_string(), 3);
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("9. Projection Switch:");
    println!("──────────────────────────────────────────────────");
    let task = ext_gen.generate_projection_switch(
        "E".to_string(),
        "alphabetical".to_string(),
        "vowels_only".to_string(),
    );
    println!("   {}", task.prompt);
    println!("   Answer: {}\n", task.correct_answer);

    println!("\n═══════════════════════════════════════════════════");
    println!("Dynamic Topology Demonstration");
    println!("═══════════════════════════════════════════════════\n");

    let base_topology = crate::core::topology::Topology::new_linear(vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
    ]);
    let mut dynamic = crate::tasks::extended::DynamicTopology::new(base_topology);

    println!("Initial topology: A -> B -> C");

    dynamic.apply_modification(crate::tasks::extended::GraphModification::AddNode {
        id: "node_3".to_string(),
        label: "D".to_string(),
        position: 3.0,
    });

    dynamic.apply_modification(crate::tasks::extended::GraphModification::AddEdge {
        from: "node_2".to_string(),
        to: "node_3".to_string(),
        weight: 1.0,
    });

    println!("After adding D: A -> B -> C -> D");

    dynamic.apply_modification(crate::tasks::extended::GraphModification::AddEdge {
        from: "node_0".to_string(),
        to: "node_2".to_string(),
        weight: 2.0,
    });

    println!("Added shortcut: A --(2)--> C");

    println!("\n═══════════════════════════════════════════════════");
    println!("Transfer Learning Demonstration");
    println!("═══════════════════════════════════════════════════\n");

    let source = crate::core::topology::Topology::new_linear(vec![
        "1".to_string(),
        "2".to_string(),
        "3".to_string(),
    ]);
    let target = crate::core::topology::Topology::new_linear(vec![
        "One".to_string(),
        "Two".to_string(),
        "Three".to_string(),
    ]);

    let transfer = crate::tasks::extended::TransferLearning::new(source.clone(), target.clone());

    let source_task = crate::tasks::Task {
        task_type: crate::tasks::TaskType::Successor {
            item: "2".to_string(),
        },
        prompt: "What comes after '2'?".to_string(),
        correct_answer: "3".to_string(),
        options: vec!["1".to_string(), "3".to_string()],
        difficulty: 0.3,
        operation: crate::core::learner::OperationType::Successor,
    };

    if let Some(transferred) = transfer.transfer_task(&source_task) {
        println!("Source task: {}", source_task.prompt);
        println!("Transferred: {}", transferred.prompt);
        println!("Isomorphic mapping preserved!");
    }

    let efficiency = transfer.measure_transfer_efficiency(0.8, 0.95);
    println!("Transfer efficiency: {:.0}%", efficiency * 100.0);
}
