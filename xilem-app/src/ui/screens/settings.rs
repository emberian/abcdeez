use xilem::{
    view::{button, flex, label, prose, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::{api_client::ApiClientTrait, components::*, models::*, AppData, Screen};

// Enhanced Settings Screen with Demo Preferences
pub fn settings_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        label("⚙️ Settings")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        // Learning Preferences
        card(
            "🎓 Learning Preferences",
            flex((
                checkbox(
                    data.use_adaptive_scheduling,
                    "Adaptive Scheduling",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.use_adaptive_scheduling = checked;
                        if checked {
                            data.success_message = Some(
                                "Adaptive scheduling enabled - AI will optimize your learning path"
                                    .to_string(),
                            );
                        }
                    }),
                ),
                prose("Uses AI to select optimal tasks based on your learning state")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
                checkbox(
                    data.enable_hints,
                    "Enable Hints",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.enable_hints = checked;
                        if checked {
                            data.success_message = Some(
                                "Hints enabled - help will be available when needed".to_string(),
                            );
                        }
                    }),
                ),
                prose("Shows helpful hints when you're struggling with a task")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
                label(format!(
                    "Difficulty Level: {:.0}%",
                    data.difficulty_level * 100.0
                ))
                .alignment(TextAlignment::Start),
                flex((
                    button("Easier", |data: &mut AppData| {
                        data.difficulty_level = (data.difficulty_level - 0.1).max(0.0);
                        data.success_message = Some(format!(
                            "Difficulty set to {:.0}%",
                            data.difficulty_level * 100.0
                        ));
                    }),
                    button("Default", |data: &mut AppData| {
                        data.difficulty_level = 0.5;
                        data.success_message =
                            Some("Difficulty reset to default (50%)".to_string());
                    }),
                    button("Harder", |data: &mut AppData| {
                        data.difficulty_level = (data.difficulty_level + 0.1).min(1.0);
                        data.success_message = Some(format!(
                            "Difficulty set to {:.0}%",
                            data.difficulty_level * 100.0
                        ));
                    }),
                ))
                .direction(Axis::Horizontal),
                prose("Adjusts the baseline difficulty of tasks")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
            ))
            .direction(Axis::Vertical),
        ),
        // Demo Preferences
        card(
            "🎭 Demo Preferences",
            flex((
                label("Demo Mode Settings")
                    .brush(Color::from_rgb8(64, 64, 64))
                    .alignment(TextAlignment::Start),
                checkbox(
                    data.demo_auto_advance,
                    "Auto-advance Demo Steps",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.demo_auto_advance = checked;
                    }),
                ),
                prose("Automatically progress through demo steps after a delay")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
                checkbox(
                    data.demo_show_tooltips,
                    "Show Demo Tooltips",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.demo_show_tooltips = checked;
                    }),
                ),
                prose("Display helpful tooltips during guided demos")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
                checkbox(
                    data.demo_highlight_elements,
                    "Highlight UI Elements",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.demo_highlight_elements = checked;
                    }),
                ),
                prose("Visually highlight relevant UI elements during demos")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
                label(format!(
                    "Demo Speed: {}",
                    match data.demo_speed {
                        s if s < 0.8 => "Slow",
                        s if s < 1.2 => "Normal",
                        _ => "Fast",
                    }
                ))
                .alignment(TextAlignment::Start),
                flex((
                    button("Slower", |data: &mut AppData| {
                        data.demo_speed = (data.demo_speed * 0.8).max(0.5);
                        data.success_message = Some("Demo speed decreased".to_string());
                    }),
                    button("Normal", |data: &mut AppData| {
                        data.demo_speed = 1.0;
                        data.success_message = Some("Demo speed reset to normal".to_string());
                    }),
                    button("Faster", |data: &mut AppData| {
                        data.demo_speed = (data.demo_speed * 1.25).min(2.0);
                        data.success_message = Some("Demo speed increased".to_string());
                    }),
                ))
                .direction(Axis::Horizontal),
                prose("Controls the speed of automated demo sequences")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
                // Demo scenario preferences
                label("Preferred Demo Scenarios")
                    .brush(Color::from_rgb8(64, 64, 64))
                    .alignment(TextAlignment::Start),
                flex((
                    button("🎓 Quick Tour", |data: &mut AppData| {
                        data.preferred_demo = "quick_tour".to_string();
                        data.success_message = Some("Quick Tour set as preferred demo".to_string());
                    }),
                    button("🎯 Training Demo", |data: &mut AppData| {
                        data.preferred_demo = "training_demo".to_string();
                        data.success_message =
                            Some("Training Demo set as preferred demo".to_string());
                    }),
                    button("🚀 Advanced", |data: &mut AppData| {
                        data.preferred_demo = "advanced".to_string();
                        data.success_message = Some("Advanced demo coming soon!".to_string());
                    }),
                ))
                .direction(Axis::Horizontal),
            ))
            .direction(Axis::Vertical),
        ),
        // Account Information
        {
            let user_block = data.current_user.as_ref().map(|user| {
                flex((
                    metric_display(
                        "👤 Username",
                        user.username.clone(),
                        Color::from_rgb8(64, 64, 64),
                    ),
                    metric_display("📧 Email", user.email.clone(), Color::from_rgb8(64, 64, 64)),
                    metric_display(
                        "📅 Member Since",
                        user.created_at.format("%Y-%m-%d").to_string(),
                        Color::from_rgb8(64, 64, 64),
                    ),
                ))
                .direction(Axis::Vertical)
            });

            let guest_block = if data.current_user.is_none() {
                Some(
                    flex((
                        label("👤 Guest User").alignment(TextAlignment::Start),
                        prose("Sign up to save your progress across sessions")
                            .brush(Color::from_rgb8(100, 100, 100))
                            .alignment(TextAlignment::Start),
                    ))
                    .direction(Axis::Vertical),
                )
            } else {
                None
            };

            card(
                "Account",
                flex((user_block, guest_block)).direction(Axis::Vertical),
            )
        },
        // Display Preferences
        card(
            "🎨 Display Preferences",
            flex((
                checkbox(
                    data.show_advanced_metrics,
                    "Show Advanced Metrics",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.show_advanced_metrics = checked;
                    }),
                ),
                prose("Display detailed cognitive metrics during training")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
                checkbox(
                    data.show_response_times,
                    "Show Response Times",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.show_response_times = checked;
                    }),
                ),
                prose("Display how long you took to answer each question")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
                checkbox(
                    data.enable_animations,
                    "Enable Animations",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.enable_animations = checked;
                    }),
                ),
                prose("Enable smooth transitions and visual effects")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
            ))
            .direction(Axis::Vertical),
        ),
        // API Configuration
        card(
            "🌐 API Configuration",
            flex((
                flex((
                    button(
                        if data.show_api_settings {
                            "🔽 Hide API Settings"
                        } else {
                            "🔧 Show API Settings"
                        },
                        |data: &mut AppData| {
                            data.show_api_settings = !data.show_api_settings;
                        },
                    ),
                    prose("Configure backend server connection")
                        .brush(Color::from_rgb8(100, 100, 100))
                        .alignment(TextAlignment::Start),
                ))
                .direction(Axis::Horizontal),
                if data.show_api_settings {
                    let expanded_block = flex((
                        labeled_input(
                            "API Base URL",
                            data.api_url_input.clone(),
                            std::sync::Arc::new(|data: &mut AppData, value: String| {
                                data.api_url_input = value;
                            }),
                        ),
                        prose("Example: https://abcdeez.fg-goose.online/api/v1")
                            .brush(Color::from_rgb8(100, 100, 100))
                            .alignment(TextAlignment::Start),
                        labeled_input(
                            "Timeout (seconds)",
                            data.api_timeout_input.clone(),
                            std::sync::Arc::new(|data: &mut AppData, value: String| {
                                data.api_timeout_input = value;
                            }),
                        ),
                        checkbox(
                            data.api_fallback_enabled,
                            "Fallback to offline mode when API unavailable",
                            std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                                data.api_fallback_enabled = checked;
                            }),
                        ),
                        flex((
                            button("💾 Save API Settings", |data: &mut AppData| {
                                // Update configuration
                                match data.config_manager.update_config(|config| {
                                    config.api.base_url = data.api_url_input.clone();
                                    if let Ok(timeout) = data.api_timeout_input.parse::<u64>() {
                                        config.api.timeout_seconds = timeout;
                                    }
                                    config.api.fallback_to_mock = data.api_fallback_enabled;
                                }) {
                                    Ok(_) => {
                                        data.success_message =
                                            Some("API settings saved successfully!".to_string());
                                        // Recreate API client with new settings
                                        data.api_client = std::sync::Arc::new(
                                            crate::api_client::AdaptiveApiClient::new(
                                                data.config_manager.config(),
                                            ),
                                        );
                                    }
                                    Err(_) => {
                                        data.error_message =
                                            Some("Failed to save API settings".to_string());
                                    }
                                }
                            }),
                            button("🔍 Test Connection", |data: &mut AppData| {
                                data.api_connection_status = "Testing...".to_string();
                                // Test API connection asynchronously
                                let api_client = data.api_client.clone();
                                let runtime = data.runtime.clone();

                                // Spawn async task to test connection
                                runtime.spawn(async move {
                                    match api_client.as_ref().health_check().await {
                                        Ok(()) => {
                                            // Connection successful
                                            // In a real app we would update state via a channel; for now just log
                                        }
                                        Err(_) => {
                                            // Connection failed
                                        }
                                    }
                                });
                            }),
                        ))
                        .direction(Axis::Horizontal),
                        label(format!("Connection Status: {}", data.api_connection_status))
                            .brush(if data.api_connection_status.contains("Success") {
                                Color::from_rgb8(0, 128, 0)
                            } else if data.api_connection_status.contains("Failed") {
                                Color::from_rgb8(200, 0, 0)
                            } else {
                                Color::from_rgb8(128, 128, 128)
                            })
                            .alignment(TextAlignment::Start),
                    ))
                    .direction(Axis::Vertical);
                    expanded_block.boxed()
                } else {
                    // Create a boxed collapsed block to match the expanded arm type
                    let collapsed_block = flex((
                        label("API settings collapsed").alignment(TextAlignment::Middle),
                        label(""),
                        label(""),
                        label(""),
                        label(""),
                        label(""),
                    ))
                    .direction(Axis::Vertical);
                    collapsed_block.boxed()
                },
            ))
            .direction(Axis::Vertical),
        ),
        // Data Management
        card(
            "💾 Data Management",
            flex((
                label("Export Options")
                    .brush(Color::from_rgb8(64, 64, 64))
                    .alignment(TextAlignment::Start),
                prose("Export your learning data for analysis or backup")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
                flex((
                    button(
                        if data.export_data_in_flight {
                            "Exporting..."
                        } else {
                            "📄 Export JSON"
                        },
                        |data: &mut AppData| {
                            if !data.export_data_in_flight {
                                data.export_format = ExportFormat::Json;
                                data.export_current_data();
                            }
                        },
                    ),
                    button("📊 Export CSV", |data: &mut AppData| {
                        data.export_format = ExportFormat::Csv;
                        data.export_current_data();
                        data.success_message = Some("CSV export coming soon!".to_string());
                    }),
                    button("🔄 Export Replay", |data: &mut AppData| {
                        data.export_format = ExportFormat::Replay;
                        data.export_current_data();
                        data.success_message = Some("Replay export coming soon!".to_string());
                    }),
                ))
                .direction(Axis::Horizontal),
                label("Data Privacy")
                    .brush(Color::from_rgb8(64, 64, 64))
                    .alignment(TextAlignment::Start),
                checkbox(
                    data.anonymous_export,
                    "Anonymous Data Export",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.anonymous_export = checked;
                        if checked {
                            data.success_message = Some(
                                "Personal information will be removed from exports".to_string(),
                            );
                        }
                    }),
                ),
                prose("Remove personal identifiers when exporting data")
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::Start),
                button("🗑️ Clear Session Data", |data: &mut AppData| {
                    data.session_responses.clear();
                    data.current_metrics = PerformanceMetrics::default();
                    data.success_message = Some("Session data cleared".to_string());
                }),
            ))
            .direction(Axis::Vertical),
        ),
        // Navigation
        flex((
            button("📊 Dashboard", |data: &mut AppData| {
                data.current_screen = Screen::Dashboard;
            }),
            button("🏠 Home", |data: &mut AppData| {
                data.current_screen = Screen::Welcome;
            }),
            button("🎓 Training", |data: &mut AppData| {
                if data.current_session.is_some() {
                    data.current_screen = Screen::Training;
                } else {
                    data.current_screen = Screen::DomainSelection;
                }
            }),
        ))
        .direction(Axis::Horizontal),
    ))
    .direction(Axis::Vertical)
}
