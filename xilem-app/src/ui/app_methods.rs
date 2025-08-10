// AppData methods extracted from the original large implementation

use chrono::Utc;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use abcdeez_core::{
    hints::{HintLevel, InterventionAction, InterventionSystem, StruggleLevel},
    prelude::*,
    tasks::TaskResponse as CoreTaskResponse,
    AdaptiveScheduler, LearnerMetrics, TaskGenerator, TaskSession, Topology,
};

use crate::core::models::*;
use crate::core::services::*;
use crate::storage::{ConnectivityMonitor, OfflineStorage, SyncStatus};
use crate::ui::{AppData, Screen};
use crate::utils::easter_egg;

#[cfg(target_os = "ios")]
use crate::integrations::ios_auth;

impl AppData {
    pub fn login(&mut self) {
        // Set in-flight flag; the view::task in app_logic() will pick this up and perform async login.
        if !self.login_request_in_flight {
            self.login_request_in_flight = true;
            self.error_message = None;
            self.success_message = None;
        }
    }

    #[cfg(target_os = "ios")]
    pub fn init_ios_auth(&mut self) {
        if self.ios_auth_bridge.is_none() {
            self.ios_auth_bridge = Some(ios_auth::IOSAuthBridge::new(self.api_client.clone()));
        }
    }

    #[cfg(target_os = "ios")]
    pub fn apple_sign_in(&mut self) {
        self.init_ios_auth();

        if let Some(bridge) = &self.ios_auth_bridge {
            if !self.oauth_login_in_flight {
                self.oauth_login_in_flight = true;
                self.error_message = None;
                self.success_message = None;

                let bridge_clone = bridge.clone();
                let runtime = self.runtime.clone();

                // Start Apple Sign In flow
                runtime.spawn(async move {
                    let result = bridge_clone
                        .sign_in_with_apple(Box::new(|result| {
                            match result {
                                Ok(user) => {
                                    println!("Apple Sign In successful: {:?}", user);
                                    // In a real app, you'd update the app state here
                                }
                                Err(error) => {
                                    println!("Apple Sign In failed: {}", error);
                                }
                            }
                        }))
                        .await;

                    if let Err(e) = result {
                        println!("Failed to start Apple Sign In: {}", e);
                    }
                });
            }
        }
    }

    #[cfg(not(target_os = "ios"))]
    pub fn apple_sign_in(&mut self) {
        self.error_message = Some("Apple Sign In is only available on iOS".to_string());
    }

    pub fn create_learner(&mut self) {
        // Create topology based on selected domain
        let topology = self.create_topology_for_domain();

        // Create learner model using the core library
        let learner_id = Uuid::new_v4().to_string();
        let core_model = LearnerModel::new(learner_id.clone(), &topology);

        self.current_learner = Some(Learner {
            id: learner_id,
            user_id: self.current_user.as_ref().map(|u| u.id.clone()),
            display_name: self.current_user.as_ref().map(|u| u.username.clone()),
            created_at: Utc::now(),
            core_model,
            metadata: None,
        });

        self.topology = Some(topology);
        self.success_message = Some("Learner profile created!".to_string());
        self.error_message = None;
    }

    pub fn demo_showcase(&mut self) {
        self.demo_showcase_with_seed(None);
    }

    pub fn demo_showcase_with_seed(&mut self, seed: Option<u64>) {
        // Initialize RNG with seed if provided (for deterministic demos)
        let actual_seed = seed.unwrap_or_else(|| {
            // Use current timestamp as seed if not provided
            chrono::Utc::now().timestamp() as u64
        });
        self.demo_seed = Some(actual_seed);
        self.demo_rng = Some(rand::rngs::StdRng::seed_from_u64(actual_seed));

        // Ensure a demo user exists
        if self.current_user.is_none() {
            self.current_user = Some(User {
                id: Uuid::new_v4().to_string(),
                username: "DemoUser".to_string(),
                email: "demo@example.com".to_string(),
                password_hash: String::new(),
                apple_user_id: None,
                github_user_id: None,
                oauth_provider_id: None,
                auth_provider: "demo".to_string(),
                is_private_email: Some(false),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        // Create the learner model and topology
        self.create_learner();

        // Start a training session (sets up task generator/scheduler/intervention system)
        self.start_session_sync();

        // Perform a short sequence of interactions to demonstrate functionality.
        for _i in 0..5 {
            self.generate_next_task();

            if let Some(rng) = &mut self.demo_rng {
                let simulated_ms = rng.gen_range(500..2000);
                if let Some(start_time) = &mut self.task_start_time {
                    *start_time = start_time
                        .checked_sub(std::time::Duration::from_millis(simulated_ms))
                        .unwrap_or(*start_time);
                }
            }

            if let Some(task) = &self.current_task {
                let answer_index = if let Some(rng) = &mut self.demo_rng {
                    if rng.gen_bool(0.7) {
                        task.display_options
                            .iter()
                            .position(|opt| opt == &task.core_task.correct_answer)
                            .unwrap_or(0)
                    } else {
                        let incorrect_indices: Vec<usize> = task
                            .display_options
                            .iter()
                            .enumerate()
                            .filter(|(_, opt)| *opt != &task.core_task.correct_answer)
                            .map(|(idx, _)| idx)
                            .collect();
                        if !incorrect_indices.is_empty() {
                            incorrect_indices[rng.gen_range(0..incorrect_indices.len())]
                        } else {
                            0
                        }
                    }
                } else {
                    0
                };

                self.submit_answer_internal(answer_index, true);
                self.request_hint();
                self.check_struggle_and_provide_help();
            }
        }

        self.end_session_sync();
        self.demo_start();
        self.success_message =
            Some("Demo showcase complete — check the Dashboard for results.".to_string());
        self.error_message = None;
    }

    pub fn demo_start(&mut self) {
        if let Err(e) = self.demo_controller.start_scenario("quick_tour") {
            self.error_message = Some(format!("Failed to start demo: {}", e));
        }
    }

    pub fn demo_start_training(&mut self) {
        if let Err(e) = self.demo_controller.start_scenario("training_demo") {
            self.error_message = Some(format!("Failed to start training demo: {}", e));
        }
    }

    pub fn demo_next_step(&mut self) {
        if !self.demo_controller.is_active {
            return;
        }

        // Execute any action for the current step
        let should_execute = self.demo_controller.get_current_step().is_some();
        if should_execute {
            let action = self
                .demo_controller
                .get_current_step()
                .and_then(|s| s.action.clone());
            if let Some(action) = action {
                match action {
                    DemoAction::SubmitAnswer(answer) => {
                        self.selected_answer = Some(answer);
                        self.submit_response_in_flight = true;
                    }
                    DemoAction::RequestHint => {
                        self.request_hint();
                    }
                    DemoAction::NavigateTo(screen) => {
                        self.current_screen = screen;
                    }
                    DemoAction::SelectDomain(domain) => {
                        println!("Demo: Selecting domain {}", domain);
                    }
                    DemoAction::RunMiniDemo => {
                        for i in 0..3 {
                            self.generate_next_task();
                            self.selected_answer = Some(format!("Answer{}", i));
                            self.submit_response_in_flight = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        if !self.demo_controller.next_step() {
            self.success_message = Some("Demo completed successfully!".to_string());
        }

        if let Some(step) = self.demo_controller.get_current_step() {
            if let Some(screen) = &step.navigation {
                self.current_screen = screen.clone();
            }
        }
    }

    pub fn demo_end(&mut self) {
        self.demo_controller.end_demo();
        self.success_message = Some("Exited guided demo.".to_string());
    }

    fn create_topology_for_domain(&self) -> Topology {
        match self.selected_domain {
            Domain::Alphabet => {
                let letters: Vec<String> = ('A'..='Z').map(|c| c.to_string()).collect();
                Topology::new_linear(letters)
            }
            Domain::DaysOfWeek => {
                let days = vec![
                    "Monday".to_string(),
                    "Tuesday".to_string(),
                    "Wednesday".to_string(),
                    "Thursday".to_string(),
                    "Friday".to_string(),
                    "Saturday".to_string(),
                    "Sunday".to_string(),
                ];
                Topology::new_cyclic(days)
            }
            Domain::Music => {
                let notes = vec![
                    "C".to_string(),
                    "D".to_string(),
                    "E".to_string(),
                    "F".to_string(),
                    "G".to_string(),
                    "A".to_string(),
                    "B".to_string(),
                ];
                Topology::new_linear(notes)
            }
            Domain::Mathematics => {
                let numbers: Vec<String> = (0..20).map(|n| n.to_string()).collect();
                Topology::new_linear(numbers)
            }
            Domain::Custom(ref name) => {
                let items: Vec<String> = (1..10).map(|i| format!("{}-{}", name, i)).collect();
                Topology::new_linear(items)
            }
        }
    }

    pub fn start_session_sync(&mut self) {
        if let Some(learner) = &mut self.current_learner {
            if let Some(topology) = &self.topology {
                let task_generator = TaskGenerator::new(topology.clone());

                if self.use_adaptive_scheduling {
                    let scheduler = AdaptiveScheduler::new_with_eig(
                        learner.core_model.clone(),
                        topology.clone(),
                        true,
                    );
                    self.adaptive_scheduler = Some(scheduler);
                }

                let task_session = TaskSession::new(topology.clone());
                let intervention_system = InterventionSystem::new(topology.clone());

                self.task_generator = Some(task_generator);
                self.task_session = Some(task_session);
                self.intervention_system = Some(intervention_system);

                let session = Session {
                    id: Uuid::new_v4().to_string(),
                    learner_id: learner.id.clone(),
                    topology_type: self.selected_domain.as_str().to_string(),
                    topology: Some(topology.clone()),
                    start_time: Utc::now(),
                    end_time: None,
                    status: "active".to_string(),
                    summary: None,
                    responses: Vec::new(),
                };

                self.current_session = Some(session);
                self.session_responses.clear();
                self.current_metrics = PerformanceMetrics::default();
                self.generate_next_task();
                self.success_message = Some("Training session started!".to_string());
                self.error_message = None;
            }
        } else {
            self.error_message = Some("Please create a learner profile first".to_string());
        }
    }

    pub fn generate_next_task(&mut self) {
        let task = if self.use_adaptive_scheduling {
            if let Some(scheduler) = &mut self.adaptive_scheduler {
                scheduler.select_next_task()
            } else if let Some(generator) = &mut self.task_generator {
                generator.generate_task(None)
            } else {
                return;
            }
        } else if let Some(generator) = &mut self.task_generator {
            generator.generate_task(None)
        } else {
            return;
        };

        let ui_task = UITask::from_core_task(task);
        self.current_task = Some(ui_task);
        self.task_start_time = Some(Instant::now());
        self.selected_answer = None;
        self.selected_answer_index = None;
        self.show_feedback = false;
    }

    pub fn submit_answer(&mut self, answer_index: usize) {
        self.submit_answer_internal(answer_index, false);
    }

    fn submit_answer_internal(&mut self, answer_index: usize, is_demo: bool) {
        if !is_demo && (self.submit_response_in_flight || self.show_feedback) {
            return;
        }

        if let Some(ui_task) = &self.current_task {
            if let Some(start_time) = self.task_start_time {
                let response_time_ms = start_time.elapsed().as_millis() as u128;

                let answer = ui_task
                    .display_options
                    .get(answer_index)
                    .cloned()
                    .unwrap_or_default();

                let correct = answer == ui_task.core_task.correct_answer;

                if let Some(intervention_system) = &mut self.intervention_system {
                    intervention_system.process_response(
                        &ui_task.core_task,
                        correct,
                        response_time_ms as u64,
                    );
                }

                let response = CoreTaskResponse {
                    task: ui_task.core_task.clone(),
                    user_answer: answer.clone(),
                    correct,
                    response_time_ms,
                    timestamp: Utc::now(),
                };

                if let Some(scheduler) = &mut self.adaptive_scheduler {
                    scheduler.update_model(&ui_task.core_task, correct, response_time_ms);
                    if let Some(learner) = &mut self.current_learner {
                        learner.core_model = scheduler.get_learner_model().clone();
                    }
                } else if let Some(learner) = &mut self.current_learner {
                    learner
                        .core_model
                        .update_operation_proficiency(&ui_task.core_task.operation, correct);
                }

                self.current_metrics
                    .update(correct, response_time_ms as i32);

                if let Some(learner) = &self.current_learner {
                    let core_metrics = LearnerMetrics::from_model(&learner.core_model);
                    self.current_metrics.update_from_core_metrics(&core_metrics);
                }

                self.session_responses.push(response);

                if let Some(session) = &mut self.current_session {
                    session.responses = self.session_responses.clone();
                }

                self.last_response_correct = correct;
                self.selected_answer = Some(answer);
                self.selected_answer_index = Some(answer_index);
                self.current_hint = None;

                if is_demo {
                    self.show_feedback = true;
                } else {
                    self.submit_response_in_flight = true;
                }
            }
        }
    }

    pub fn request_hint(&mut self) {
        if let (Some(ui_task), Some(intervention_system)) =
            (&self.current_task, &mut self.intervention_system)
        {
            if self.enable_hints {
                let elapsed_ms = self
                    .task_start_time
                    .map(|start| start.elapsed().as_millis() as u64)
                    .unwrap_or(0);

                if let Some(action) =
                    intervention_system.check_intervention_needed(&ui_task.core_task, elapsed_ms)
                {
                    match action {
                        InterventionAction::ProvideHint(hint)
                        | InterventionAction::ProvideWorkedExample(hint) => {
                            self.current_hint = Some(hint);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn check_struggle_and_provide_help(&mut self) {
        if let (Some(ui_task), Some(intervention_system)) =
            (&self.current_task, &mut self.intervention_system)
        {
            let elapsed_ms = self
                .task_start_time
                .map(|start| start.elapsed().as_millis() as u64)
                .unwrap_or(0);

            if let Some(action) =
                intervention_system.check_intervention_needed(&ui_task.core_task, elapsed_ms)
            {
                match action {
                    InterventionAction::ProvideHint(hint)
                    | InterventionAction::ProvideWorkedExample(hint) => {
                        self.current_hint = Some(hint);
                    }
                    InterventionAction::SuggestBreak => {
                        self.success_message =
                            Some("Consider taking a short break to refresh your mind.".to_string());
                    }
                    InterventionAction::IncreaseDifficulty => {
                        self.difficulty_level = (self.difficulty_level + 0.1).min(1.0);
                        self.success_message =
                            Some("Great progress! Increasing difficulty.".to_string());
                    }
                    InterventionAction::DecreaseDifficulty => {
                        self.difficulty_level = (self.difficulty_level - 0.1).max(0.1);
                        self.success_message =
                            Some("Adjusting difficulty to help you learn better.".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    fn end_session_sync(&mut self) {
        if let Some(session) = &mut self.current_session {
            session.end_time = Some(Utc::now());
            session.status = "completed".to_string();

            let total_responses = self.session_responses.len();
            let correct_responses = self.session_responses.iter().filter(|r| r.correct).count();
            let accuracy = if total_responses > 0 {
                correct_responses as f64 / total_responses as f64
            } else {
                0.0
            };

            let learner_metrics = self
                .current_learner
                .as_ref()
                .map(|l| LearnerMetrics::from_model(&l.core_model));

            session.summary = Some(serde_json::json!({
                "total_responses": total_responses,
                "correct_responses": correct_responses,
                "accuracy": accuracy,
                "metrics": self.current_metrics,
                "learner_metrics": learner_metrics,
            }));

            self.success_message = Some("Session completed!".to_string());
        }
    }

    pub fn try_crab_triple_click(&mut self) {
        let now = std::time::Instant::now();

        if let Some(last_click) = self.last_click_time {
            if now.duration_since(last_click) > std::time::Duration::from_secs(2) {
                self.crab_trigger_clicks = 0;
            }
        }

        self.crab_trigger_clicks += 1;
        self.last_click_time = Some(now);

        if self.crab_trigger_clicks >= 3 {
            if let Some(crab) = &mut self.little_crab {
                if crab.try_discover(easter_egg::CrabTrigger::TripleClick) {
                    self.success_message = Some("🦀 You found the secret crab! 🦀".to_string());
                }
            }
            self.current_screen = Screen::WidgetGallery;
            self.crab_trigger_clicks = 0;
        }
    }

    pub fn check_crab_triggers(&mut self) {
        if let Some(crab) = &mut self.little_crab {
            if self.current_metrics.streak_count >= 10 {
                crab.try_discover(easter_egg::CrabTrigger::PerfectStreak(
                    self.current_metrics.streak_count,
                ));
            }

            if self.username_input.to_lowercase().contains("crab") {
                if crab.try_discover(easter_egg::CrabTrigger::SecretWord(
                    self.username_input.clone(),
                )) {
                    self.success_message = Some("🦀 The crab heard you call! 🦀".to_string());
                }
            }
        }
    }

    pub fn trigger_sync(&mut self) {
        if let Some(_storage) = &self.offline_storage {
            self.success_message = Some("Sync started...".to_string());
        } else {
            self.error_message = Some("Offline storage not initialized".to_string());
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn init_pwa_integration(&mut self) -> Result<(), String> {
        // PWA integration placeholder
        Ok(())
    }
}