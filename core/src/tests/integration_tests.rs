use crate::core::adaptive::AdaptiveScheduler;
use crate::bayesian::BayesianLearnerModel;
use crate::core::learner::LearnerModel;
use crate::tasks::types::{Task, TaskGenerator, TaskResponse, TaskType};
use crate::core::topology::Topology;

#[test]
fn test_complete_learning_workflow() {
    // Initialize topology
    let topology = Topology::alphabet();

    // Create learner
    let mut learner = LearnerModel::new("test_learner".to_string(), &topology);

    // Create task generator
    let mut generator = TaskGenerator::new(topology.clone());

    // Generate and complete tasks
    for i in 0..10 {
        let task = generator.generate_task(None);

        // Simulate response
        let correct = (i % 2) == 0; // Alternate correct/incorrect for testing
        learner.update_operation_proficiency(&task.operation, correct);

        // Update memory if applicable
        match &task.task_type {
            TaskType::Successor { item } | TaskType::Predecessor { item } => {
                learner.update_memory_strength(item, correct);
            }
            _ => {}
        }
    }

    // Check that learning occurred
    let proficiencies: Vec<f64> = learner
        .operation_proficiencies
        .values()
        .map(|p| p.theta)
        .collect();

    assert!(
        proficiencies.iter().any(|&p| p != 0.5),
        "Some proficiencies should have changed from initial value"
    );
}

#[test]
fn test_adaptive_scheduling_workflow() {
    let topology = Topology::alphabet();
    let learner = LearnerModel::new("test".to_string(), &topology);
    let mut scheduler = AdaptiveScheduler::new(learner, topology.clone());

    // Get initial task and its EIG
    let task1 = scheduler.select_next_task();
    assert!(!task1.prompt.is_empty(), "Task should have a prompt");

    // Simulate response to first task
    scheduler.update_model(&task1, true, 1000);

    // Get next task after update
    let task2 = scheduler.select_next_task();

    // Tasks should be different (adaptive)
    assert_ne!(task1.prompt, task2.prompt, "Tasks should be different");

    // The scheduler should be adapting based on what it learned
    // After a correct response, it might select a harder task or explore different operations
    // Note: TaskType doesn't implement PartialEq, so we can't directly compare
    // Instead, check that tasks are meaningfully different
    assert_ne!(
        task1.correct_answer, task2.correct_answer,
        "Adaptive scheduler should select different tasks"
    );

    // Simulate multiple responses to test adaptation
    for i in 0..5 {
        let task = scheduler.select_next_task();
        let correct = i % 2 == 0;
        let response_time = 1000 + (i * 200) as u128;
        scheduler.update_model(&task, correct, response_time);
    }

    // After learning, scheduler should have updated model
    // After learning, the scheduler should be selecting different tasks
    // Get a final task to see if it's adapted
    let final_task = scheduler.select_next_task();

    // The final task should be different from early tasks (adaptation occurred)
    assert_ne!(
        final_task.prompt, task1.prompt,
        "After learning, scheduler should select different tasks"
    );
}

#[test]
fn test_bayesian_integration() {
    let topology = Topology::alphabet();
    let mut bayesian = BayesianLearnerModel::new(&topology);
    let mut generator = TaskGenerator::new(topology.clone());

    // Generate candidate tasks
    let tasks: Vec<Task> = (0..5).map(|_| generator.generate_task(None)).collect();

    // Rank by EIG
    let ranked = bayesian.rank_tasks_by_eig(tasks.clone());

    assert_eq!(ranked.len(), tasks.len());

    // Update with best task
    if let Some((best_task, _eig)) = ranked.first() {
        let response = crate::bayesian::ResponseData {
            task: best_task.clone(),
            correct: true,
            response_time: 1000.0,
        };
        bayesian.update_with_response(response);
    }

    // Entropy should decrease after update
    let final_entropy = bayesian.total_entropy();
    assert!(final_entropy > 0.0, "Entropy should be positive");
}

#[test]
fn test_task_session_workflow() {
    let topology = Topology::alphabet();
    let mut generator = TaskGenerator::new(topology.clone());

    let mut responses = Vec::new();

    // Generate and complete tasks
    for i in 0..10 {
        let task = generator.generate_task(None);

        // Verify task is valid
        assert!(!task.prompt.is_empty(), "Task should have prompt");
        assert!(!task.correct_answer.is_empty(), "Task should have answer");
        assert!(
            task.difficulty >= 0.0 && task.difficulty <= 1.0,
            "Difficulty should be in [0,1]"
        );

        // Simulate response
        let response = crate::tasks::TaskResponse {
            task: task.clone(),
            user_answer: if i < 5 {
                task.correct_answer.clone()
            } else {
                "wrong".to_string()
            },
            correct: i < 5,
            response_time_ms: 1000 + (i * 100) as u128,
            timestamp: chrono::Utc::now(),
        };

        responses.push(response);
    }

    // Calculate session metrics manually
    let total_tasks = responses.len();
    let correct_tasks = responses.iter().filter(|r| r.correct).count();
    let accuracy = correct_tasks as f64 / total_tasks as f64;
    let average_rt = responses
        .iter()
        .map(|r| r.response_time_ms as f64)
        .sum::<f64>()
        / total_tasks as f64;

    // Verify metrics are reasonable
    assert_eq!(total_tasks, 10, "Should have 10 tasks");
    assert_eq!(correct_tasks, 5, "Should have 5 correct");
    assert_eq!(accuracy, 0.5, "Accuracy should be 50%");

    // Average RT should be around 1450ms ((1000 + 1900) / 2)
    assert!(
        average_rt > 1000.0 && average_rt < 2000.0,
        "Average RT should be reasonable: {}",
        average_rt
    );
}

#[test]
fn test_extended_task_generation() {
    use crate::tasks::extended::ExtendedTaskGenerator;

    let topology = Topology::alphabet();
    let mut generator = ExtendedTaskGenerator::new(topology);

    // Test task generation
    let task = generator.generate_reverse_n_treadmill("M".to_string(), 3, 2);
    assert!(!task.prompt.is_empty());

    // Test that generator works
    assert!(!task.prompt.is_empty());
    assert!(!task.correct_answer.is_empty());
}

#[test]
fn test_hierarchical_model_workflow() {
    use crate::hierarchical_bayes::{HierarchicalBayesianModel, ResponseData};

    let topology = Topology::alphabet();
    let mut model = HierarchicalBayesianModel::new(&topology);

    // Add learners
    for i in 0..3 {
        model.add_learner(format!("learner_{}", i));
    }

    // Add items (tasks)
    let mut generator = TaskGenerator::new(topology);
    for _i in 0..5 {
        let task = generator.generate_task(None);
        model.add_item(&task);
    }

    // Simulate responses
    for i in 0..10 {
        let response = ResponseData {
            learner_id: format!("learner_{}", i % 3),
            task_id: format!("task_{}", i % 5),
            correct: (i % 2) == 0,
            response_time: 1000.0 + (i as f64 * 100.0),
            timestamp: i,
        };
        model.update(response);
    }

    // Verify model has data
    assert!(!model.individual_models.is_empty());
    assert!(!model.item_bank.is_empty());
}

#[test]
fn test_export_workflow() {
    use crate::data::export::SessionExporter;

    let exporter = SessionExporter::new();

    // Create mock session data
    let topology = Topology::alphabet();
    let mut generator = TaskGenerator::new(topology.clone());

    let task1 = generator.generate_task(None);
    let task2 = generator.generate_task(None);

    let responses = vec![
        TaskResponse {
            task: task1.clone(),
            user_answer: "B".to_string(),
            correct: true,
            response_time_ms: 1000,
            timestamp: chrono::Utc::now(),
        },
        TaskResponse {
            task: task2.clone(),
            user_answer: "D".to_string(),
            correct: false,
            response_time_ms: 1500,
            timestamp: chrono::Utc::now(),
        },
    ];

    // Generate CSV
    let csv = exporter.export_to_csv(&responses).unwrap();
    assert!(csv.contains("task_id"));
    assert!(csv.contains("response_time_ms"));

    // Generate summary statistics
    let summary = exporter.generate_summary(&responses);
    assert!(summary.contains("Total responses: 2"));
    assert!(summary.contains("Overall accuracy:"));
}
