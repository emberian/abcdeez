use xilem::{
    view::{button, flex, label, prose, Axis, FlexExt},
    Color, TextAlignment, WidgetView,
};

use crate::{components::*, visualization_components::*, AppData, Screen};

use abcdeez_core::learner::OperationType;
use abcdeez_core::tasks::{TaskResponse, TaskType};

pub fn visualizations_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    let has_data = !data.session_responses.is_empty();
    let enough_data = data.session_responses.len() >= 5;

    // Main visualization dashboard
    let main_dashboard = if enough_data {
        card(
            "📊 Interactive Visualizations",
            visualization_dashboard(data),
        )
        .into_any_flex()
    } else if has_data {
        card(
            "📊 Visualizations",
            flex((
                label(format!(
                    "You have {} data points",
                    data.session_responses.len()
                ))
                .alignment(TextAlignment::Middle),
                prose("Complete at least 5 tasks to unlock full visualizations")
                    .brush(Color::from_rgb8(128, 128, 128))
                    .alignment(TextAlignment::Middle),
                // Show what's available with limited data
                if data.session_responses.len() >= 2 {
                    Some(
                        flex((
                            label("Available with current data:")
                                .brush(Color::from_rgb8(102, 126, 234)),
                            sparkline(
                                &data
                                    .session_responses
                                    .iter()
                                    .map(|r| if r.correct { 1.0 } else { 0.0 })
                                    .collect::<Vec<_>>(),
                                200,
                                30,
                                Color::from_rgb8(46, 213, 115),
                            ),
                        ))
                        .direction(Axis::Vertical),
                    )
                } else {
                    None
                },
            ))
            .direction(Axis::Vertical),
        )
        .into_any_flex()
    } else {
        card(
            "📊 No Data Yet",
            flex((
                label("Start a training session to generate visualization data")
                    .alignment(TextAlignment::Middle),
                button("🚀 Start Training", |data: &mut AppData| {
                    data.current_screen = Screen::DomainSelection;
                }),
            ))
            .direction(Axis::Vertical),
        )
        .into_any_flex()
    };

    // Session comparisons if multiple sessions exist
    let session_comparison = if let Some(controller) = &data.research_controller {
        if controller.sessions.len() >= 2 {
            let session_data: Vec<Vec<abcdeez_core::tasks::TaskResponse>> = controller
                .sessions
                .iter()
                .map(|s| {
                    s.data_points
                        .iter()
                        .map(|dp| abcdeez_core::tasks::TaskResponse {
                            task: abcdeez_core::tasks::Task {
                                task_type: TaskType::Successor {
                                    item: dp.stimulus.clone(),
                                },
                                prompt: dp.stimulus.clone(),
                                correct_answer: dp.response.clone(),
                                options: vec![dp.response.clone()],
                                operation: OperationType::Successor,
                                difficulty: 0.5,
                            },
                            user_answer: dp.response.clone(),
                            correct: dp.correct,
                            response_time_ms: dp.response_time_ms,
                            timestamp: dp.timestamp,
                        })
                        .collect()
                })
                .collect();

            Some(card(
                "📈 Session Comparisons",
                session_comparison_chart(&session_data, 800, 400),
            ))
        } else {
            None
        }
    } else {
        None
    };

    // Real-time metrics with live updates
    let live_metrics = card(
        "⚡ Live Metrics",
        flex((
            flex((
                label("Current Streak:").alignment(TextAlignment::Start),
                label(format!("{}", data.current_metrics.streak_count))
                    .brush(if data.current_metrics.streak_count >= 5 {
                        Color::from_rgb8(46, 213, 115)
                    } else {
                        Color::from_rgb8(128, 128, 128)
                    })
                    .alignment(TextAlignment::End),
            ))
            .direction(Axis::Horizontal),
            flex((
                label("Last 10 Accuracy:").alignment(TextAlignment::Start),
                if data.session_responses.len() >= 10 {
                    let recent_accuracy = data
                        .session_responses
                        .iter()
                        .rev()
                        .take(10)
                        .filter(|r| r.correct)
                        .count() as f64
                        / 10.0;
                    Some(
                        label(format!("{:.0}%", recent_accuracy * 100.0))
                            .brush(get_metric_color(recent_accuracy))
                            .alignment(TextAlignment::End),
                    )
                } else {
                    Some(
                        label("N/A")
                            .brush(Color::from_rgb8(128, 128, 128))
                            .alignment(TextAlignment::End),
                    )
                },
            ))
            .direction(Axis::Horizontal),
            // Trend indicators
            if data.session_responses.len() >= 20 {
                let first_half = &data.session_responses[..data.session_responses.len() / 2];
                let second_half = &data.session_responses[data.session_responses.len() / 2..];
                let first_acc = first_half.iter().filter(|r| r.correct).count() as f64
                    / first_half.len() as f64;
                let second_acc = second_half.iter().filter(|r| r.correct).count() as f64
                    / second_half.len() as f64;
                let trend = second_acc - first_acc;

                Some(
                    flex((
                        label("Performance Trend:").alignment(TextAlignment::Start),
                        label(if trend > 0.1 {
                            "📈 Improving"
                        } else if trend < -0.1 {
                            "📉 Declining"
                        } else {
                            "➡️ Stable"
                        })
                        .brush(if trend > 0.1 {
                            Color::from_rgb8(46, 213, 115)
                        } else if trend < -0.1 {
                            Color::from_rgb8(255, 71, 87)
                        } else {
                            Color::from_rgb8(0, 123, 255)
                        })
                        .alignment(TextAlignment::End),
                    ))
                    .direction(Axis::Horizontal),
                )
            } else {
                None
            },
        ))
        .direction(Axis::Vertical),
    );

    // Advanced analytics
    let advanced_analytics = if enough_data {
        Some(card(
            "🔬 Advanced Analytics",
            flex((
                // Response time analysis
                flex((
                    label("Response Time Analysis:").alignment(TextAlignment::Start),
                    {
                        let times: Vec<f64> = data
                            .session_responses
                            .iter()
                            .map(|r| r.response_time_ms as f64)
                            .collect();
                        let avg = times.iter().sum::<f64>() / times.len() as f64;
                        let variance = times.iter().map(|t| (t - avg).powi(2)).sum::<f64>()
                            / times.len() as f64;
                        let std_dev = variance.sqrt();

                        flex((
                            label(format!("Mean: {:.0}ms", avg))
                                .brush(get_speed_color(avg))
                                .alignment(TextAlignment::Start),
                            label(format!("StdDev: {:.0}ms", std_dev))
                                .brush(Color::from_rgb8(128, 128, 128))
                                .alignment(TextAlignment::Start),
                        ))
                        .direction(Axis::Horizontal)
                    },
                ))
                .direction(Axis::Vertical),
                // Learning efficiency
                if data.session_responses.len() >= 10 {
                    let windows: Vec<f64> = data
                        .session_responses
                        .windows(5)
                        .map(|w| w.iter().filter(|r| r.correct).count() as f64 / 5.0)
                        .collect();

                    let improvement = if windows.len() >= 2 {
                        windows.last().unwrap() - windows.first().unwrap()
                    } else {
                        0.0
                    };

                    Some(
                        flex((
                            label("Learning Efficiency:").alignment(TextAlignment::Start),
                            label(format!("{:+.1}% improvement", improvement * 100.0))
                                .brush(if improvement > 0.0 {
                                    Color::from_rgb8(46, 213, 115)
                                } else if improvement < 0.0 {
                                    Color::from_rgb8(255, 71, 87)
                                } else {
                                    Color::from_rgb8(128, 128, 128)
                                })
                                .alignment(TextAlignment::End),
                        ))
                        .direction(Axis::Horizontal),
                    )
                } else {
                    None
                },
            ))
            .direction(Axis::Vertical),
        ))
    } else {
        None
    };

    // Export visualizations
    let export_section = card(
        "💾 Export Visualizations",
        flex((
            prose("Export visualization data for external analysis")
                .alignment(TextAlignment::Start),
            flex((
                button("📸 Screenshot", |data: &mut AppData| {
                    data.success_message = Some("Screenshot export coming soon!".to_string());
                }),
                button("📊 Export Data", |data: &mut AppData| {
                    data.export_current_data();
                }),
                button("🎬 Record Session", |data: &mut AppData| {
                    data.success_message = Some("Session recording coming soon!".to_string());
                }),
            ))
            .direction(Axis::Horizontal),
        ))
        .direction(Axis::Vertical),
    );

    // Navigation
    let navigation = card(
        "🎮 Navigation",
        flex((
            button("📊 Dashboard", |data: &mut AppData| {
                data.current_screen = Screen::Dashboard;
            }),
            button("🔬 Research", |data: &mut AppData| {
                data.current_screen = Screen::ResearchDashboard;
            }),
            button("🎯 Training", |data: &mut AppData| {
                data.current_screen = Screen::Training;
            }),
            button("⚙️ Settings", |data: &mut AppData| {
                data.current_screen = Screen::Settings;
            }),
        ))
        .direction(Axis::Horizontal),
    );

    // Main layout
    flex((
        label("🎨 Visualization Center")
            .brush(Color::from_rgb8(255, 0, 128))
            .alignment(TextAlignment::Middle),
        main_dashboard,
        session_comparison,
        live_metrics,
        advanced_analytics,
        export_section,
        navigation,
    ))
    .direction(Axis::Vertical)
}

// Helper function reused from visualization_components
fn get_metric_color(value: f64) -> Color {
    if value >= 0.8 {
        Color::from_rgb8(46, 213, 115)
    } else if value >= 0.6 {
        Color::from_rgb8(0, 123, 255)
    } else if value >= 0.4 {
        Color::from_rgb8(255, 165, 2)
    } else {
        Color::from_rgb8(255, 71, 87)
    }
}

fn get_speed_color(ms: f64) -> Color {
    if ms < 1000.0 {
        Color::from_rgb8(46, 213, 115)
    } else if ms < 2000.0 {
        Color::from_rgb8(0, 123, 255)
    } else if ms < 3000.0 {
        Color::from_rgb8(255, 165, 2)
    } else {
        Color::from_rgb8(255, 71, 87)
    }
}
