// Main application logic - extracted from the large lib.rs file

use xilem::{
    view::{button, flex, label, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::ui::{AppData, Screen};
use crate::ui::components::*;
use crate::ui::screens::*;
use crate::utils::easter_egg;

// Main app logic
pub fn app_logic(data: &mut AppData) -> impl WidgetView<AppData> {
    // Setup PWA hooks (wasm only) on first render
    #[cfg(target_arch = "wasm32")]
    {
        if !data.pwa_initialized {
            if let Err(e) = data.init_pwa_integration() {
                data.error_message = Some(format!("PWA init failed: {}", e));
            } else {
                data.pwa_initialized = true;
            }
        }
    }
    // Update the little crab behavior
    if data.little_crab.is_some() {
        if let Some(mut crab) = data.little_crab.take() {
            crab.update(data);
            data.little_crab = Some(crab);
        }
        // Check for crab discovery triggers
        data.check_crab_triggers();
    }

    // Clear old messages after some time (in a real app, use a timer)
    if data.error_message.is_some() || data.success_message.is_some() {
        // Messages will auto-clear after being displayed
    }

    // Handle async operations synchronously for now (Xilem task view not available)
    // In production, these would be proper async tasks
    handle_async_operations(data);

    // Sync status indicator
    let sync_indicator = create_sync_indicator(data);

    // Main content - use conditional rendering to avoid impl trait issues
    let content = flex((
        nav_bar(&format!("{:?}", data.current_screen)),
        sync_indicator,
        error_message(data.error_message.clone()),
        success_message(data.success_message.clone()),
        flex((
            (data.current_screen == Screen::Welcome).then(|| welcome_screen(data)),
            (data.current_screen == Screen::DomainSelection).then(|| domain_selection_screen(data)),
            (data.current_screen == Screen::Training).then(|| training_screen(data)),
            (data.current_screen == Screen::Dashboard).then(|| dashboard_screen(data)),
            (data.current_screen == Screen::Settings).then(|| settings_screen(data)),
            (data.current_screen == Screen::ResearchDashboard)
                .then(|| research_dashboard_screen(data)),
            (data.current_screen == Screen::Visualizations).then(|| visualizations_screen(data)),
            (data.current_screen == Screen::WidgetGallery)
                .then(|| widget_gallery_screen(data)),
        ))
        .direction(Axis::Vertical),
    ))
    .direction(Axis::Vertical);

    // Enhanced guided demo overlay
    let overlay = create_demo_overlay(data);

    // The little crab overlay (appears in corner of screen)
    let crab_overlay = data
        .little_crab
        .as_ref()
        .and_then(|crab| easter_egg::render_crab_overlay(crab));

    // Compose content + overlay + crab (crab last so it appears on top)
    flex((content, overlay, crab_overlay)).direction(Axis::Vertical)
}

fn handle_async_operations(data: &mut AppData) {
    use uuid::Uuid;
    use chrono::Utc;

    if data.login_request_in_flight {
        // Simulate login completion
        let username = data.username_input.clone();
        data.login_request_in_flight = false;
        data.current_user = Some(crate::core::models::User {
            id: Uuid::new_v4().to_string(),
            username: username.clone(),
            email: format!("{}@example.com", username),
            password_hash: String::new(),
            apple_user_id: None,
            github_user_id: None,
            oauth_provider_id: None,
            auth_provider: "local".to_string(),
            is_private_email: Some(false),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        data.current_screen = Screen::DomainSelection;
        data.success_message = Some("Login successful!".to_string());
        data.error_message = None;
        data.create_learner();
    }

    if data.create_session_in_flight {
        data.create_session_in_flight = false;
        if let Some(learner) = &data.current_learner {
            let session = crate::core::models::Session {
                id: Uuid::new_v4().to_string(),
                learner_id: learner.id.clone(),
                topology_type: data.selected_domain.as_str().to_string(),
                topology: data.topology.clone(),
                start_time: Utc::now(),
                end_time: None,
                status: "active".to_string(),
                summary: None,
                responses: Vec::new(),
            };
            data.current_session = Some(session);
            data.current_screen = Screen::Training;
            data.session_responses.clear();
            data.current_metrics = crate::core::models::PerformanceMetrics::default();
            data.generate_next_task();
            data.success_message = Some("Training session started!".to_string());
            data.error_message = None;
        }
    }

    if data.submit_response_in_flight {
        data.submit_response_in_flight = false;
        data.show_feedback = true;
    }

    if data.end_session_in_flight {
        data.end_session_in_flight = false;
        if let Some(session) = &mut data.current_session {
            session.end_time = Some(Utc::now());
            session.status = "completed".to_string();

            let total_responses = data.session_responses.len();
            let correct_responses = data.session_responses.iter().filter(|r| r.correct).count();
            let accuracy = if total_responses > 0 {
                correct_responses as f64 / total_responses as f64
            } else {
                0.0
            };

            session.summary = Some(serde_json::json!({
                "total_responses": total_responses,
                "correct_responses": correct_responses,
                "accuracy": accuracy,
                "metrics": data.current_metrics,
            }));
        }
        data.current_screen = Screen::Dashboard;
        data.success_message = Some("Session completed!".to_string());
    }

    if data.export_data_in_flight {
        data.export_data_in_flight = false;
        data.success_message = Some("Data exported successfully!".to_string());
    }
}

fn create_sync_indicator(data: &AppData) -> Option<impl WidgetView<AppData>> {
    if !data.sync_status.online || data.sync_status.pending_count > 0 {
        Some(
            flex((
                {
                    let text = if !data.sync_status.online {
                        "🔴 Offline Mode".to_string()
                    } else if data.sync_status.pending_count > 0 {
                        format!("🔄 Syncing {} items...", data.sync_status.pending_count)
                    } else {
                        "✅ Synced".to_string()
                    };
                    label(text)
                }
                .brush(if !data.sync_status.online {
                    Color::from_rgb8(255, 100, 100)
                } else if data.sync_status.pending_count > 0 {
                    Color::from_rgb8(255, 165, 0)
                } else {
                    Color::from_rgb8(0, 200, 0)
                })
                .alignment(TextAlignment::End),
                {
                    let btn_label = if data.sync_status.pending_count > 0 {
                        "Sync Now"
                    } else {
                        ""
                    };
                    button(btn_label, |data: &mut AppData| {
                        if data.sync_status.pending_count > 0 {
                            data.trigger_sync();
                        }
                    })
                },
            ))
            .direction(Axis::Horizontal),
        )
    } else {
        None
    }
}

fn create_demo_overlay(data: &AppData) -> Option<impl WidgetView<AppData>> {
    if data.demo_controller.is_active {
        let (current, total) = data.demo_controller.get_progress();
        let step = data.demo_controller.get_current_step();

        let step_content = if let Some(step) = step {
            let title = format!("📚 {} ({}/{})", step.title, current, total);
            let description = step.description.clone();

            // Progress bar
            let progress_text = format!(
                "[{}{}] {}/{}",
                "=".repeat(current),
                "-".repeat(total.saturating_sub(current)),
                current,
                total
            );

            flex((
                label(title).alignment(TextAlignment::Middle),
                label(progress_text).alignment(TextAlignment::Middle),
                label(description).alignment(TextAlignment::Start),
            ))
            .direction(Axis::Vertical)
        } else {
            flex((
                label("Loading demo...").alignment(TextAlignment::Middle),
                label("").alignment(TextAlignment::Middle),
                label("").alignment(TextAlignment::Start),
            ))
            .direction(Axis::Vertical)
        };

        let controls = flex((
            button("◀ Back", |d: &mut AppData| {
                d.demo_controller.previous_step();
            }),
            button(
                if data.demo_controller.is_paused {
                    "▶ Resume"
                } else {
                    "⏸ Pause"
                },
                |d: &mut AppData| {
                    if d.demo_controller.is_paused {
                        d.demo_controller.resume();
                    } else {
                        d.demo_controller.pause();
                    }
                },
            ),
            button("Next ▶", |d: &mut AppData| {
                d.demo_next_step();
            }),
            button("✖ End", |d: &mut AppData| {
                d.demo_end();
            }),
        ))
        .direction(Axis::Horizontal);

        Some(card::<AppData, _>(
            "Guided Demo",
            flex((step_content, controls)).direction(Axis::Vertical),
        ))
    } else {
        None
    }
}