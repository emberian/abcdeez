use xilem::{
    view::{button, flex, label, prose, textbox, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::core::models::*;
use crate::ui::AppData;
use std::sync::Arc;

// Reusable card component
pub fn card<T: 'static, V>(title: &str, content: V) -> impl WidgetView<T> + 'static
where
    V: WidgetView<T> + 'static,
{
    flex((
        label(title)
            .brush(Color::from_rgb8(64, 64, 64))
            .alignment(TextAlignment::Start),
        content,
    ))
    .direction(Axis::Vertical)
}

// Progress bar component
pub fn progress_bar(progress: f64, label_text: String) -> impl WidgetView<AppData> {
    let width = 300.0;
    let height = 20.0;
    let filled_width = (width * progress.min(1.0).max(0.0)) as i32;

    flex((
        label(label_text).alignment(TextAlignment::Middle),
        // Simple text-based progress visualization
        label(format!(
            "[{}{}] {:.1}%",
            "=".repeat((filled_width / 10) as usize),
            " ".repeat(((width - filled_width as f64) / 10.0) as usize),
            progress * 100.0
        ))
        .brush(Color::from_rgb8(0, 128, 255))
        .alignment(TextAlignment::Middle),
    ))
    .direction(Axis::Vertical)
}

// Metric display component
pub fn metric_display(label_text: &str, value: String, color: Color) -> impl WidgetView<AppData> {
    flex((
        label(label_text)
            .brush(Color::from_rgb8(128, 128, 128))
            .alignment(TextAlignment::Start),
        label(value).brush(color).alignment(TextAlignment::End),
    ))
    .direction(Axis::Horizontal)
}

// Task card component for displaying questions (using UITask from models)
pub fn task_card(task: &UITask) -> impl WidgetView<AppData> {
    flex((
        label(task.display_prompt.clone()).alignment(TextAlignment::Middle),
        if let Some(hint) = &task.hint {
            label(hint.clone())
                .brush(Color::from_rgb8(128, 128, 128))
                .alignment(TextAlignment::Middle)
        } else {
            label("").alignment(TextAlignment::Middle)
        },
    ))
    .direction(Axis::Vertical)
}

// Answer options component
pub fn answer_options(
    options: Vec<String>,
    on_select: Arc<dyn Fn(&mut AppData, usize) + Send + Sync + 'static>,
) -> impl WidgetView<AppData> {
    let buttons = options
        .into_iter()
        .enumerate()
        .map(move |(index, option)| {
            let on_select = Arc::clone(&on_select);
            button(option, move |data: &mut AppData| {
                (on_select)(data, index);
            })
        })
        .collect::<Vec<_>>();

    flex(buttons).direction(Axis::Vertical)
}

// Performance chart component (simplified text-based)
pub fn performance_chart(metrics: &PerformanceMetrics) -> impl WidgetView<AppData> {
    flex((
        label("Performance Overview")
            .brush(Color::from_rgb8(64, 64, 64))
            .alignment(TextAlignment::Middle),
        metric_display(
            "Accuracy:",
            format!("{:.1}%", metrics.accuracy_rate * 100.0),
            Color::from_rgb8(0, 200, 0),
        ),
        metric_display(
            "Total Responses:",
            metrics.total_responses.to_string(),
            Color::from_rgb8(0, 128, 255),
        ),
        metric_display(
            "Correct:",
            metrics.correct_responses.to_string(),
            Color::from_rgb8(0, 200, 0),
        ),
        metric_display(
            "Avg Response Time:",
            format!("{:.0}ms", metrics.average_response_time_ms),
            Color::from_rgb8(255, 128, 0),
        ),
    ))
    .direction(Axis::Vertical)
}

/// Wrapper component that adds highlighting for demo mode
pub fn highlighted<V>(element_id: &str, content: V, app_data: &AppData) -> impl WidgetView<AppData>
where
    V: WidgetView<AppData> + 'static,
{
    let tooltip_text = if app_data
        .demo_controller
        .should_highlight(element_id)
        .is_some()
    {
        app_data
            .demo_controller
            .get_tooltip(element_id)
            .map(|t| format!("💡 {}", t))
            .unwrap_or_else(|| String::new())
    } else {
        String::new()
    };

    flex((
        content,
        label(tooltip_text).alignment(TextAlignment::Middle),
    ))
    .direction(Axis::Vertical)
}

/// Create a highlighted button for demo mode
pub fn demo_button(element_id: &str, text: &str, app_data: &AppData) -> impl WidgetView<AppData> {
    let tooltip_text = if app_data
        .demo_controller
        .should_highlight(element_id)
        .is_some()
    {
        app_data
            .demo_controller
            .get_tooltip(element_id)
            .map(|t| format!("💡 {}", t))
            .unwrap_or_else(|| String::new())
    } else {
        String::new()
    };

    flex((
        label(text).alignment(TextAlignment::Middle),
        label(tooltip_text).alignment(TextAlignment::Middle),
    ))
    .direction(Axis::Vertical)
}

/// Create a highlighted card for demo mode
pub fn demo_card<V>(
    element_id: &str,
    title: &str,
    content: V,
    app_data: &AppData,
) -> impl WidgetView<AppData>
where
    V: WidgetView<AppData> + 'static,
{
    highlighted(element_id, card(title, content), app_data)
}

/// Confirmation modal component
pub fn confirm_modal(
    title: &str,
    message: &str,
    on_confirm: std::sync::Arc<dyn Fn(&mut AppData) + Send + Sync + 'static>,
    on_cancel: std::sync::Arc<dyn Fn(&mut AppData) + Send + Sync + 'static>,
) -> impl WidgetView<AppData> {
    card(
        title,
        flex((
            prose(message).alignment(TextAlignment::Middle),
            flex((
                {
                    let cb = on_confirm.clone();
                    button("✓ Confirm", move |data: &mut AppData| (cb)(data))
                },
                {
                    let cb = on_cancel.clone();
                    button("✗ Cancel", move |data: &mut AppData| (cb)(data))
                },
            ))
            .direction(Axis::Horizontal),
        ))
        .direction(Axis::Vertical),
    )
}

/// Toast notification component
pub fn toast_notification(message: String, is_success: bool) -> impl WidgetView<AppData> {
    let icon = if is_success { "✓" } else { "⚠" };
    let color = if is_success { "#4CAF50" } else { "#f44336" };

    card(
        &format!("{} Notification", icon),
        prose(message).alignment(TextAlignment::Middle),
    )
}

/// Loading overlay component
pub fn loading_overlay(message: &str) -> impl WidgetView<AppData> {
    card(
        "Loading",
        flex((
            label("⏳").alignment(TextAlignment::Middle),
            prose(message).alignment(TextAlignment::Middle),
        ))
        .direction(Axis::Vertical),
    )
}

// Domain card component
pub fn domain_card(domain: &Domain, selected: bool) -> impl WidgetView<AppData> {
    let color = if selected {
        Color::from_rgb8(0, 128, 255)
    } else {
        Color::from_rgb8(128, 128, 128)
    };

    flex((
        label(domain.display_name())
            .brush(color)
            .alignment(TextAlignment::Middle),
        label(match domain {
            Domain::Alphabet => "Learn letter positions and sequences",
            Domain::DaysOfWeek => "Master the order of days in a week",
            Domain::Music => "Master intervals, scales, and theory",
            Domain::Mathematics => "Practice arithmetic and patterns",
            Domain::Custom(_) => "Custom learning domain",
        })
        .brush(Color::from_rgb8(160, 160, 160))
        .alignment(TextAlignment::Middle),
    ))
    .direction(Axis::Vertical)
}

// Session info component
pub fn session_info(session: &Session) -> impl WidgetView<AppData> {
    let duration = if let Some(end) = session.end_time {
        let diff = end - session.start_time;
        format!("{}m {}s", diff.num_minutes(), diff.num_seconds() % 60)
    } else {
        "In Progress".to_string()
    };

    flex((
        metric_display(
            "Session ID:",
            session.id[..8].to_string(),
            Color::from_rgb8(128, 128, 128),
        ),
        metric_display(
            "Status:",
            session.status.clone(),
            Color::from_rgb8(0, 200, 0),
        ),
        metric_display("Duration:", duration, Color::from_rgb8(0, 128, 255)),
        metric_display(
            "Domain:",
            session.topology_type.clone(),
            Color::from_rgb8(255, 128, 0),
        ),
    ))
    .direction(Axis::Vertical)
}

// Input field with label
pub fn labeled_input(
    label_text: &str,
    value: String,
    on_change: Arc<dyn Fn(&mut AppData, String) + Send + Sync + 'static>,
) -> impl WidgetView<AppData> {
    let on_change_cb = on_change.clone();
    flex((
        label(label_text).alignment(TextAlignment::Start),
        textbox(value, move |data: &mut AppData, text: String| {
            on_change_cb(data, text);
        }),
    ))
    .direction(Axis::Vertical)
}

// Navigation button bar
pub fn nav_bar(_current_screen: &str) -> impl WidgetView<AppData> {
    flex((
        button("Home", |data: &mut AppData| {
            data.current_screen = crate::Screen::Welcome;
        }),
        button("Domains", |data: &mut AppData| {
            data.current_screen = crate::Screen::DomainSelection;
        }),
        button("Training", |data: &mut AppData| {
            if data.current_session.is_some() {
                data.current_screen = crate::Screen::Training;
            }
        }),
        button("Dashboard", |data: &mut AppData| {
            data.current_screen = crate::Screen::Dashboard;
        }),
        button("Settings", |data: &mut AppData| {
            data.current_screen = crate::Screen::Settings;
        }),
    ))
    .direction(Axis::Horizontal)
}

// Error message display
pub fn error_message(message: Option<String>) -> impl WidgetView<AppData> {
    if let Some(msg) = message {
        label(msg)
            .brush(Color::from_rgb8(255, 0, 0))
            .alignment(TextAlignment::Middle)
    } else {
        label("").alignment(TextAlignment::Middle)
    }
}

// Success message display
pub fn success_message(message: Option<String>) -> impl WidgetView<AppData> {
    if let Some(msg) = message {
        label(msg)
            .brush(Color::from_rgb8(0, 200, 0))
            .alignment(TextAlignment::Middle)
    } else {
        label("").alignment(TextAlignment::Middle)
    }
}

// Checkbox component (using button as a workaround since xilem doesn't have native checkbox yet)
pub fn checkbox(
    checked: bool,
    label_text: &str,
    on_change: Arc<dyn Fn(&mut AppData, bool) + Send + Sync + 'static>,
) -> impl WidgetView<AppData> {
    let checkbox_display = if checked { "[✓]" } else { "[ ]" };

    flex((
        {
            let on_change = Arc::clone(&on_change);
            button(checkbox_display, move |data: &mut AppData| {
                (on_change)(data, !checked);
            })
        },
        label(label_text),
    ))
    .direction(Axis::Horizontal)
}

// Advanced statistics visualizations
pub fn response_time_histogram(response_times: &[u128]) -> impl WidgetView<AppData> {
    let histogram_bars = if response_times.is_empty() {
        vec![label("No data yet")]
    } else {
        // Create simple text-based histogram
        let min_rt = *response_times.iter().min().unwrap() as f64;
        let max_rt = *response_times.iter().max().unwrap() as f64;
        let range = (max_rt - min_rt).max(1.0);
        let num_bins = 10;
        let bin_width = range / num_bins as f64;

        let mut bins = vec![0; num_bins];
        for &rt in response_times {
            let bin = ((rt as f64 - min_rt) / bin_width).floor() as usize;
            let bin_idx = bin.min(num_bins - 1);
            bins[bin_idx] += 1;
        }

        let max_count = *bins.iter().max().unwrap_or(&1);
        bins.iter()
            .enumerate()
            .map(|(i, &count)| {
                let bin_start = min_rt + (i as f64 * bin_width);
                let bar_length = (count as f64 / max_count as f64 * 20.0) as usize;
                let bar = "█".repeat(bar_length);
                label(format!("{:>4.0}ms: {} ({})", bin_start, bar, count))
            })
            .collect::<Vec<_>>()
    };

    card(
        "Response Time Distribution",
        flex(histogram_bars).direction(Axis::Vertical),
    )
}

pub fn learning_curve_display(
    session_responses: &[abcdeez_core::tasks::TaskResponse],
) -> impl WidgetView<AppData> {
    // Calculate moving average accuracy over time or show placeholder
    let curve_points = if session_responses.is_empty() {
        vec![label("No data yet")]
    } else {
        let window_size = 10;
        let mut moving_averages = Vec::new();
        let mut running_correct = 0;

        for (i, response) in session_responses.iter().enumerate() {
            if response.correct {
                running_correct += 1;
            }

            if i + 1 >= window_size {
                let accuracy = running_correct as f64 / window_size as f64;
                moving_averages.push(accuracy);

                // Remove the oldest response from the window
                if session_responses[i + 1 - window_size].correct {
                    running_correct -= 1;
                }
            } else {
                let accuracy = if i + 1 > 0 {
                    running_correct as f64 / (i + 1) as f64
                } else {
                    0.0
                };
                moving_averages.push(accuracy);
            }
        }

        // Create simple text-based learning curve
        moving_averages
            .iter()
            .enumerate()
            .step_by(moving_averages.len().max(20) / 20) // Show ~20 points max
            .map(|(i, &accuracy)| {
                let visual_height = (accuracy * 10.0).clamp(0.0, 10.0) as usize;
                let bar = "▓".repeat(visual_height) + &"░".repeat(10 - visual_height);
                label(format!("T{:>3}: [{}] {:.1}%", i + 1, bar, accuracy * 100.0))
            })
            .collect::<Vec<_>>()
    };

    card(
        "Learning Curve (Moving Average)",
        flex(curve_points).direction(Axis::Vertical),
    )
}

pub fn error_analysis_display(
    session_responses: &[abcdeez_core::tasks::TaskResponse],
) -> impl WidgetView<AppData> {
    use std::collections::HashMap;

    let error_displays = if session_responses.is_empty() {
        vec![label("No data yet")]
    } else {
        // Analyze error patterns
        let mut task_type_errors: HashMap<String, (usize, usize)> = HashMap::new();
        let mut difficulty_errors: HashMap<String, Vec<bool>> = HashMap::new();

        for response in session_responses {
            let task_type = format!("{:?}", response.task.task_type);
            let entry = task_type_errors.entry(task_type.clone()).or_insert((0, 0));
            entry.1 += 1; // total
            if !response.correct {
                entry.0 += 1; // errors
            }

            let difficulty_bucket = format!("{:.1}", response.task.difficulty);
            difficulty_errors
                .entry(difficulty_bucket)
                .or_insert_with(Vec::new)
                .push(response.correct);
        }

        // Display error rates by task type
        let mut error_rates: Vec<(String, f64)> = task_type_errors
            .into_iter()
            .map(|(task_type, (errors, total))| {
                (
                    task_type,
                    if total > 0 {
                        errors as f64 / total as f64
                    } else {
                        0.0
                    },
                )
            })
            .collect();
        error_rates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        error_rates
            .into_iter()
            .take(5)
            .map(|(task_type, error_rate)| {
                let error_bar_length = (error_rate * 20.0) as usize;
                let error_bar = "█".repeat(error_bar_length) + &"░".repeat(20 - error_bar_length);
                label(format!(
                    "{}: [{}] {:.1}%",
                    task_type.chars().take(15).collect::<String>(),
                    error_bar,
                    error_rate * 100.0
                ))
            })
            .collect::<Vec<_>>()
    };

    card(
        "Error Patterns by Task Type",
        flex(error_displays).direction(Axis::Vertical),
    )
}

pub fn strategy_analysis_display(learner: &crate::models::Learner) -> impl WidgetView<AppData> {
    // Extract strategy indicators from the learner model
    let core_metrics = abcdeez_core::LearnerMetrics::from_model(&learner.core_model);
    let bidirectionality = core_metrics.bidirectionality_index;
    let distance_slope = core_metrics.symbolic_distance_slope;

    // Interpret strategy based on metrics
    let strategy_interpretation = if distance_slope > 100.0 {
        "Serial Scanning: High distance effect suggests step-by-step navigation"
    } else if distance_slope < 50.0 {
        "Direct Access: Low distance effect suggests direct memory retrieval"
    } else {
        "Mixed Strategy: Moderate distance effect suggests flexible approach"
    };

    let bidirectionality_interpretation = if bidirectionality > 0.8 {
        "Excellent bidirectional knowledge"
    } else if bidirectionality > 0.6 {
        "Good forward and backward navigation"
    } else if bidirectionality > 0.4 {
        "Moderate bidirectional ability"
    } else {
        "Forward-biased navigation pattern"
    };

    card(
        "Cognitive Strategy Analysis",
        flex((
            metric_display(
                "Strategy Type:",
                strategy_interpretation.to_string(),
                Color::from_rgb8(0, 128, 255),
            ),
            metric_display(
                "Distance Slope:",
                format!("{:.1}ms/step", distance_slope),
                Color::from_rgb8(128, 0, 255),
            ),
            metric_display(
                "Bidirectionality:",
                format!("{:.3}", bidirectionality),
                Color::from_rgb8(0, 128, 255),
            ),
            prose(bidirectionality_interpretation),
        ))
        .direction(Axis::Vertical),
    )
}
