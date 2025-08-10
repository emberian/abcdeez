use xilem::{
    view::{button, flex, label, prose, Axis, FlexExt},
    Color, TextAlignment, WidgetView,
};

use crate::{components::*, AppData, Screen};

// Training Session Screen with enhanced task rendering and modals
pub fn training_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    // Session progress bar with enhanced metrics
    let progress = if data.session_responses.len() > 0 {
        let correct = data.session_responses.iter().filter(|r| r.correct).count();
        correct as f64 / data.session_responses.len() as f64
    } else {
        0.0
    };

    let progress_display = card(
        "Session Progress",
        flex((
            flex((
                metric_display(
                    "Tasks:",
                    format!("{}", data.session_responses.len()),
                    Color::from_rgb8(0, 128, 255),
                ),
                metric_display(
                    "Accuracy:",
                    format!("{:.1}%", data.current_metrics.accuracy_rate * 100.0),
                    if data.current_metrics.accuracy_rate >= 0.7 {
                        Color::from_rgb8(0, 200, 0)
                    } else {
                        Color::from_rgb8(255, 128, 0)
                    },
                ),
                metric_display(
                    "Streak:",
                    format!("{}", data.current_metrics.streak_count),
                    Color::from_rgb8(255, 128, 0),
                ),
                metric_display(
                    "Best:",
                    format!("{}", data.current_metrics.best_streak),
                    Color::from_rgb8(128, 0, 255),
                ),
            ))
            .direction(Axis::Horizontal),
            progress_bar(progress, "Session Completion".to_string()),
            // Time elapsed display
            label(if let Some(session) = &data.current_session {
                let elapsed = chrono::Utc::now()
                    .signed_duration_since(session.start_time)
                    .num_seconds();
                format!("⏱ Time: {}:{:02}", elapsed / 60, elapsed % 60)
            } else {
                "⏱ Time: 0:00".to_string()
            })
            .brush(Color::from_rgb8(100, 100, 100))
            .alignment(TextAlignment::End),
        ))
        .direction(Axis::Vertical),
    );

    // Enhanced task display with beautiful rendering
    let task_display = {
        let prompt = data
            .current_task
            .as_ref()
            .map(|t| t.display_prompt.clone())
            .unwrap_or_else(|| "Loading next task...".to_string());

        // Task type indicator
        let task_type = data
            .current_task
            .as_ref()
            .map(|t| match t.core_task.operation {
                _ => "📝 Question",
            })
            .unwrap_or("⏳ Loading");

        let difficulty = data
            .current_task
            .as_ref()
            .map(|t| {
                let stars = match t.core_task.difficulty {
                    d if d < 0.3 => "⭐",
                    d if d < 0.5 => "⭐⭐",
                    d if d < 0.7 => "⭐⭐⭐",
                    d if d < 0.9 => "⭐⭐⭐⭐",
                    _ => "⭐⭐⭐⭐⭐",
                };
                format!("Difficulty: {} ({:.1})", stars, t.core_task.difficulty)
            })
            .unwrap_or_else(|| "Calculating difficulty...".to_string());

        // Enhanced answer options with better visual design
        let option_buttons = data
            .current_task
            .as_ref()
            .map(|t| {
                t.display_options
                    .iter()
                    .enumerate()
                    .map(|(index, option)| {
                        let option_text = format!(
                            "{}. {}",
                            ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'][index.min(7)],
                            option
                        );
                        let is_selected = data.selected_answer_index == Some(index);

                        button(
                            if is_selected {
                                format!("→ {} ←", option_text)
                            } else {
                                option_text
                            },
                            move |data: &mut AppData| {
                                if !data.show_feedback && !data.submit_response_in_flight {
                                    data.submit_answer(index);
                                }
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| Vec::new());

        let masked_buttons = if data.show_feedback || data.submit_response_in_flight {
            Vec::<_>::new()
        } else {
            option_buttons
        };

        // Beautiful answer grid (2 columns for better layout)
        let answer_grid = if masked_buttons.len() > 2 {
            let mid = (masked_buttons.len() + 1) / 2;
            let mut left = masked_buttons;
            let right = left.split_off(mid);
            flex((
                flex(left).direction(Axis::Vertical),
                flex(right).direction(Axis::Vertical),
            ))
            .direction(Axis::Horizontal)
        } else {
            let left = masked_buttons;
            let right: Vec<_> = Vec::new();
            flex((
                flex(left).direction(Axis::Vertical),
                flex(right).direction(Axis::Vertical),
            ))
            .direction(Axis::Horizontal)
        };

        // Enhanced feedback with emoji and colors
        let (feedback_text, feedback_emoji) = if data.submit_response_in_flight {
            ("Processing your answer...".to_string(), "⏳")
        } else if data.show_feedback {
            if data.last_response_correct {
                let encouragements = [
                    "Excellent!",
                    "Great job!",
                    "Perfect!",
                    "Outstanding!",
                    "Brilliant!",
                ];
                let idx = (data.session_responses.len() % encouragements.len())
                    .min(encouragements.len() - 1);
                (encouragements[idx].to_string(), "✅")
            } else {
                let correct_answer = data
                    .current_task
                    .as_ref()
                    .map(|t| t.core_task.correct_answer.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                (
                    format!("Not quite. The answer was: {}", correct_answer),
                    "❌",
                )
            }
        } else {
            ("".to_string(), "")
        };

        let feedback_color = if data.submit_response_in_flight {
            Color::from_rgb8(128, 128, 128)
        } else if data.show_feedback {
            if data.last_response_correct {
                Color::from_rgb8(0, 200, 0)
            } else {
                Color::from_rgb8(255, 100, 100)
            }
        } else {
            Color::from_rgb8(128, 128, 128)
        };

        let feedback_display = flex((
            label(feedback_emoji)
                .brush(feedback_color)
                .alignment(TextAlignment::Middle),
            label(feedback_text)
                .brush(feedback_color)
                .alignment(TextAlignment::Middle),
        ))
        .direction(Axis::Horizontal);

        // Continue button with smart text
        let continue_button = button(
            if data.submit_response_in_flight {
                "⏳ Processing..."
            } else if data.show_feedback {
                "Next Task →"
            } else {
                "Submit Answer"
            },
            |data: &mut AppData| {
                if data.show_feedback && !data.submit_response_in_flight {
                    data.continue_to_next_task();
                }
            },
        );

        // Enhanced hint system with progression levels
        let hint_section = if data.enable_hints && !data.show_feedback {
            let hint_button = button(
                match data.hint_level {
                    _ if data.current_hint.is_none() => "💡 Get Hint",
                    _ if data
                        .current_hint
                        .as_ref()
                        .map_or(false, |h| h.contains("Level 1")) =>
                    {
                        "💡 Get More Help"
                    }
                    _ if data
                        .current_hint
                        .as_ref()
                        .map_or(false, |h| h.contains("Level 2")) =>
                    {
                        "💡 Get Answer Help"
                    }
                    _ => "💡 Hint Available",
                },
                |data: &mut AppData| {
                    if !data.submit_response_in_flight {
                        data.request_hint();
                    }
                },
            );

            let (hint_title, hint_body) = if let Some(hint) = &data.current_hint {
                (
                    "💡 Hint",
                    flex((
                        prose(hint.as_str()).alignment(TextAlignment::Start),
                        label(format!("Hint Level: {:?}", data.hint_level))
                            .brush(Color::from_rgb8(100, 100, 100))
                            .alignment(TextAlignment::End),
                    ))
                    .direction(Axis::Vertical),
                )
            } else {
                (
                    "💡 Hints",
                    flex((
                        prose("Hints will appear here when requested")
                            .alignment(TextAlignment::Middle),
                        label(""),
                    ))
                    .direction(Axis::Vertical),
                )
            };
            let hint_display = card(hint_title, hint_body);

            Some(flex((hint_button, hint_display)).direction(Axis::Vertical))
        } else {
            None
        };

        // Task card with beautiful layout
        card(
            "Task",
            flex((
                // Task header
                flex((
                    label(task_type)
                        .brush(Color::from_rgb8(0, 128, 255))
                        .alignment(TextAlignment::Start),
                    label(difficulty)
                        .brush(Color::from_rgb8(128, 128, 128))
                        .alignment(TextAlignment::End),
                ))
                .direction(Axis::Horizontal),
                // Task prompt with larger, clearer text
                card(
                    "",
                    label(prompt)
                        .brush(Color::from_rgb8(0, 0, 0))
                        .alignment(TextAlignment::Middle),
                ),
                // Answer options
                answer_grid,
                // Feedback section
                feedback_display,
                // Action buttons
                continue_button,
                // Hint section (optional)
                hint_section,
            ))
            .direction(Axis::Vertical),
        )
    };

    // Advanced metrics display
    let metrics_card = card(
        "Learning Metrics",
        flex((
            if let Some(core_metrics) = &data.current_metrics.core_metrics {
                flex((
                    metric_display(
                        "Bidirectionality:",
                        format!("{:.3}", core_metrics.bidirectionality_index),
                        Color::from_rgb8(0, 128, 255),
                    ),
                    metric_display(
                        "Distance Effect:",
                        format!("{:.3}", core_metrics.symbolic_distance_slope),
                        Color::from_rgb8(128, 0, 255),
                    ),
                    metric_display(
                        "Chunk Penalty:",
                        format!("{:.3}", core_metrics.chunk_boundary_penalty),
                        Color::from_rgb8(255, 128, 0),
                    ),
                ))
                .direction(Axis::Horizontal)
                .into_any_flex()
            } else {
                flex((label("Metrics will appear after a few responses")
                    .alignment(TextAlignment::Middle),))
                .direction(Axis::Horizontal)
                .into_any_flex()
            },
            // Response time trend
            if data.session_responses.len() > 2 {
                let recent_times: Vec<f64> = data
                    .session_responses
                    .iter()
                    .rev()
                    .take(5)
                    .map(|r| r.response_time_ms as f64)
                    .collect();
                let avg_recent = recent_times.iter().sum::<f64>() / recent_times.len() as f64;
                label(format!("⚡ Avg Response: {:.0}ms", avg_recent))
                    .brush(Color::from_rgb8(100, 100, 100))
                    .alignment(TextAlignment::End)
            } else {
                label("").alignment(TextAlignment::End)
            },
        ))
        .direction(Axis::Vertical),
    );

    // Session controls with confirmation modal
    let session_controls = flex((
        button("⏸ Pause", |data: &mut AppData| {
            data.current_screen = Screen::Dashboard;
            data.success_message = Some("Session paused. Resume anytime!".to_string());
        }),
        button(
            if data.end_session_in_flight {
                "Ending..."
            } else {
                "🏁 End Session"
            },
            |data: &mut AppData| {
                if !data.end_session_in_flight {
                    // Show confirmation modal (simplified for now)
                    data.show_end_session_confirmation = true;
                }
            },
        ),
        button("📊 Dashboard", |data: &mut AppData| {
            data.current_screen = Screen::Dashboard;
        }),
    ))
    .direction(Axis::Horizontal);

    // Main layout
    let main_content = flex((
        label("Training Session")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        progress_display,
        task_display,
        metrics_card,
        session_controls,
    ))
    .direction(Axis::Vertical);

    // End session confirmation modal (overlay)
    if data.show_end_session_confirmation {
        flex((
            main_content,
            confirm_modal(
                "End Training Session?",
                &format!(
                    "You've completed {} tasks with {:.0}% accuracy. Are you sure you want to end this session?",
                    data.session_responses.len(),
                    data.current_metrics.accuracy_rate * 100.0
                ),
                std::sync::Arc::new(|data: &mut AppData| {
                    data.show_end_session_confirmation = false;
                    data.end_session();
                }),
                std::sync::Arc::new(|data: &mut AppData| {
                    data.show_end_session_confirmation = false;
                }),
            ),
        ))
        .direction(Axis::Vertical)
    } else {
        flex((
            main_content,
            confirm_modal(
                "",
                "",
                std::sync::Arc::new(|_data: &mut AppData| {}),
                std::sync::Arc::new(|_data: &mut AppData| {}),
            ),
        ))
        .direction(Axis::Vertical)
    }
}
