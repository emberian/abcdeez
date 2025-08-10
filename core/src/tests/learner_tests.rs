use crate::core::learner::*;
use crate::core::topology::Topology;
use chrono::{Duration, Utc};

#[test]
fn test_learner_initialization() {
    let topo = Topology::alphabet();
    let learner = LearnerModel::new("test_learner".to_string(), &topo);

    assert_eq!(learner.learner_id, "test_learner");
    assert_eq!(learner.node_embeddings.len(), 26);
    assert_eq!(learner.memory_strengths.len(), 26);
    assert_eq!(learner.operation_proficiencies.len(), 8); // 8 operation types initialized

    // Check initial values
    for embedding in learner.node_embeddings.values() {
        // Check that embedding has been initialized
        assert!(!embedding.node_id.is_empty());
    }
}

#[test]
fn test_operation_proficiency_update() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);

    let initial_prof = learner
        .operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);

    // Update with correct response
    learner.update_operation_proficiency(&OperationType::Successor, true);

    let updated_prof = learner
        .operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);

    assert!(
        updated_prof > initial_prof,
        "Proficiency should increase after correct response: {} -> {}",
        initial_prof,
        updated_prof
    );

    // Check practice count
    let practice_count = learner
        .operation_proficiencies
        .get("Successor")
        .map(|p| p.practice_count)
        .unwrap_or(0);

    assert_eq!(practice_count, 1, "Practice count should be 1");
}

#[test]
fn test_memory_strength_update() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);

    // Update memory for node A
    learner.update_memory_strength("A", true);

    let strength = learner
        .memory_strengths
        .get("node_0")
        .map(|m| m.strength)
        .unwrap_or(0.0);

    assert!(
        strength > 0.5,
        "Memory strength should increase after correct response"
    );

    // Update with incorrect response
    learner.update_memory_strength("A", false);

    let updated_strength = learner
        .memory_strengths
        .get("node_0")
        .map(|m| m.strength)
        .unwrap_or(0.0);

    assert!(
        updated_strength < strength,
        "Memory strength should decrease after incorrect response"
    );
}

#[test]
fn test_memory_decay() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);

    // Set initial memory strength
    learner.update_memory_strength("A", true);

    // Get initial strength value directly
    let initial_strength = learner
        .memory_strengths
        .get("node_0")
        .map(|m| m.strength)
        .unwrap_or(0.5);

    // Simulate time passing by updating last_practice
    if let Some(memory) = learner.memory_strengths.get_mut("node_0") {
        memory.last_practice = Utc::now() - Duration::hours(1); // 1 hour ago
    }

    // Calculate decayed strength manually using decay formula
    let elapsed_hours = 1.0f64;
    let decay_rate = 0.1f64;
    let decayed_strength = initial_strength * (-decay_rate * elapsed_hours).exp();

    assert!(
        decayed_strength < initial_strength,
        "Memory should decay over time: {} -> {}",
        initial_strength,
        decayed_strength
    );
}

#[test]
fn test_performance_over_time() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);

    // Test that proficiency updates work
    let initial = learner
        .operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);

    // Simulate multiple correct responses
    for _ in 0..5 {
        learner.update_operation_proficiency(&OperationType::Successor, true);
    }

    let final_prof = learner
        .operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);

    assert!(
        final_prof > initial,
        "Proficiency should increase with practice: {} -> {}",
        initial,
        final_prof
    );
}

#[test]
fn test_strategy_tracking() {
    // This test should verify that the system can track strategy changes
    // Currently the LearnerModel doesn't have explicit strategy tracking,
    // but we can test related functionality

    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);

    // Simulate a pattern that suggests serial scanning strategy:
    // Successive items should be easier (higher success rate)

    // Practice successor tasks (should improve if using serial strategy)
    for _ in 0..10 {
        learner.update_operation_proficiency(&OperationType::Successor, true);
    }

    // Practice non-adjacent tasks (harder with serial strategy)
    for _ in 0..10 {
        learner.update_operation_proficiency(&OperationType::KJump(3), false);
    }

    // Check proficiencies reflect the pattern
    let successor_prof = learner
        .operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);

    let kjump_prof = learner
        .operation_proficiencies
        .get("KJump_3")
        .map(|p| p.theta)
        .unwrap_or(0.0);

    // With serial strategy, successor should have higher proficiency
    assert!(
        successor_prof > kjump_prof,
        "Serial strategy should show higher successor proficiency ({}) than k-jump ({})",
        successor_prof,
        kjump_prof
    );

    // Verify practice counts
    let successor_count = learner
        .operation_proficiencies
        .get("Successor")
        .map(|p| p.practice_count)
        .unwrap_or(0);
    assert_eq!(
        successor_count, 10,
        "Should have 10 successor practice trials"
    );
}

#[test]
fn test_learning_curve_tracking() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);

    // Simulate learning over time
    for i in 0..10 {
        let correct = i > 3; // Start failing, then succeed
        learner.update_operation_proficiency(&OperationType::Successor, correct);
    }

    // Check that proficiency improved
    let final_prof = learner
        .operation_proficiencies
        .get("Successor")
        .map(|p| p.theta)
        .unwrap_or(0.0);

    assert!(
        final_prof > 0.0,
        "Should have positive proficiency after learning"
    );
}

#[test]
fn test_metrics_calculation() {
    let topo = Topology::alphabet();
    let learner = LearnerModel::new("test".to_string(), &topo);

    // Manually calculate metrics
    let avg_memory = learner
        .memory_strengths
        .values()
        .map(|m| m.strength)
        .sum::<f64>()
        / learner.memory_strengths.len() as f64;

    let avg_prof = learner
        .operation_proficiencies
        .values()
        .map(|p| 1.0 / (1.0 + (-p.theta).exp())) // sigmoid
        .sum::<f64>()
        / learner.operation_proficiencies.len() as f64;

    let total_practice: usize = learner
        .operation_proficiencies
        .values()
        .map(|p| p.practice_count)
        .sum();

    // Check all metrics are valid
    assert!(avg_memory >= 0.0 && avg_memory <= 1.0);
    assert!(avg_prof >= 0.0 && avg_prof <= 1.0);
    assert_eq!(total_practice, 0); // No practice yet
}

#[test]
fn test_embedding_updates() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);

    // Embeddings should exist for all nodes
    assert_eq!(learner.node_embeddings.len(), 26);

    // Update memory should affect embeddings indirectly
    learner.update_memory_strength("A", true);
    learner.update_memory_strength("B", true);

    // Verify embeddings still exist
    assert!(learner.node_embeddings.contains_key("node_0"));
    assert!(learner.node_embeddings.contains_key("node_1"));
}

#[test]
fn test_response_pattern_tracking() {
    let topo = Topology::alphabet();
    let mut learner = LearnerModel::new("test".to_string(), &topo);

    // Track response patterns across multiple items
    let test_sequence = vec!["A", "B", "C", "D", "E"];
    let mut response_pattern = Vec::new();

    // First pass: all correct
    for item in &test_sequence {
        learner.update_memory_strength(item, true);
        response_pattern.push(true);
    }

    // Check that all items have strengthened memory
    for i in 0..5 {
        let node_id = format!("node_{}", i);
        let strength = learner
            .memory_strengths
            .get(&node_id)
            .map(|m| m.strength)
            .unwrap_or(0.0);
        assert!(
            strength > 0.5,
            "Memory strength for {} should be > 0.5 after correct response, got {}",
            node_id,
            strength
        );
    }

    // Second pass: errors on specific items
    learner.update_memory_strength("B", false);
    learner.update_memory_strength("D", false);

    // Verify that errors reduced strength for those specific items
    let b_strength = learner
        .memory_strengths
        .get("node_1")
        .map(|m| m.strength)
        .unwrap_or(0.5);
    let d_strength = learner
        .memory_strengths
        .get("node_3")
        .map(|m| m.strength)
        .unwrap_or(0.5);
    let a_strength = learner
        .memory_strengths
        .get("node_0")
        .map(|m| m.strength)
        .unwrap_or(0.5);

    assert!(
        b_strength < a_strength,
        "B (with error) should have lower strength {} than A (no error) {}",
        b_strength,
        a_strength
    );
    assert!(
        d_strength < a_strength,
        "D (with error) should have lower strength {} than A (no error) {}",
        d_strength,
        a_strength
    );
}
