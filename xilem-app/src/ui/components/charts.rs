// visualization_components.rs - Xilem wrapper components for Plotters visualizations

use xilem::{
    view::{flex, label, Axis},
    Color, WidgetView,
};

use crate::{models::*, visualizations::*, AppData};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

// Very simple in-memory buffer cache keyed by a short hash of inputs
static CHART_BUFFER_CACHE: Lazy<Mutex<std::collections::HashMap<String, Vec<u8>>>> =
    Lazy::new(|| Mutex::new(std::collections::HashMap::new()));

fn cache_get(key: &str) -> Option<Vec<u8>> {
    CHART_BUFFER_CACHE
        .lock()
        .ok()
        .and_then(|m| m.get(key).cloned())
}

fn cache_set(key: String, value: Vec<u8>) {
    if let Ok(mut m) = CHART_BUFFER_CACHE.lock() {
        // simple size cap
        if m.len() > 64 {
            m.clear();
        }
        m.insert(key, value);
    }
}

fn short_hash<T: serde::Serialize>(value: &T) -> String {
    let json = serde_json::to_string(value).unwrap_or_default();
    // FNV-like quick hash
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in json.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}
use abcdeez_core::tasks::TaskResponse;

/// Create a learning curve chart component
pub fn learning_curve_chart(
    responses: &[TaskResponse],
    width: u32,
    height: u32,
) -> impl WidgetView<AppData> {
    // Generate the chart as RGB buffer
    let cache_key = format!(
        "learning_curve:{}:{}:{}",
        short_hash(&responses),
        width,
        height
    );
    let _chart_data = if let Some(buf) = cache_get(&cache_key) {
        buf
    } else {
        let buf = match create_learning_curve(responses, width, height) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to create learning curve: {}", e);
                vec![255; (width * height * 3) as usize] // White fallback
            }
        };
        cache_set(cache_key, buf.clone());
        buf
    };

    // Convert to image view
    // Note: In a real implementation, we'd need a custom widget or image view
    // For now, we'll use a placeholder with the chart description
    flex((
        label("📈 Learning Progress Chart").brush(Color::from_rgb8(102, 126, 234)),
        label(format!("Showing {} responses", responses.len()))
            .brush(Color::from_rgb8(128, 128, 128)),
        label(if responses.is_empty() {
            "No data yet - complete some tasks to see your learning curve".to_string()
        } else {
            let accuracy = responses.iter().filter(|r| r.correct).count() as f64
                / responses.len() as f64
                * 100.0;
            format!("Overall Accuracy: {:.1}%", accuracy)
        })
        .brush(
            if responses.iter().filter(|r| r.correct).count() as f64 / responses.len().max(1) as f64
                >= 0.8
            {
                Color::from_rgb8(46, 213, 115)
            } else if responses.iter().filter(|r| r.correct).count() as f64
                / responses.len().max(1) as f64
                >= 0.6
            {
                Color::from_rgb8(255, 165, 2)
            } else {
                Color::from_rgb8(255, 71, 87)
            },
        ),
    ))
    .direction(Axis::Vertical)
}

/// Create a response time histogram component
pub fn response_time_histogram_chart(
    response_times: &[u128],
    width: u32,
    height: u32,
) -> impl WidgetView<AppData> {
    // Generate the histogram
    let cache_key = format!(
        "rt_hist:{}:{}:{}",
        short_hash(&response_times),
        width,
        height
    );
    let _chart_data = if let Some(buf) = cache_get(&cache_key) {
        buf
    } else {
        let buf = match create_response_time_histogram(response_times, width, height) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to create histogram: {}", e);
                vec![255; (width * height * 3) as usize]
            }
        };
        cache_set(cache_key, buf.clone());
        buf
    };

    // Create component
    flex((
        label("📊 Response Time Distribution").brush(Color::from_rgb8(102, 126, 234)),
        label(format!("{} responses analyzed", response_times.len()))
            .brush(Color::from_rgb8(128, 128, 128)),
        if !response_times.is_empty() {
            let avg = response_times.iter().sum::<u128>() as f64 / response_times.len() as f64;
            let min = *response_times.iter().min().unwrap_or(&0);
            let max = *response_times.iter().max().unwrap_or(&0);

            flex((
                label(format!("Avg: {:.0}ms", avg)).brush(Color::from_rgb8(0, 123, 255)),
                label(format!("Min: {}ms", min)).brush(Color::from_rgb8(46, 213, 115)),
                label(format!("Max: {}ms", max)).brush(Color::from_rgb8(255, 71, 87)),
            ))
            .direction(Axis::Horizontal)
        } else {
            flex((
                label("Complete tasks to see timing data").brush(Color::from_rgb8(128, 128, 128)),
                label(""),
                label(""),
            ))
            .direction(Axis::Horizontal)
        },
    ))
    .direction(Axis::Vertical)
}

/// Create a performance heatmap component
pub fn performance_heatmap_chart(
    responses: &[TaskResponse],
    width: u32,
    height: u32,
) -> impl WidgetView<AppData> {
    // Generate the heatmap
    let cache_key = format!("heatmap:{}:{}:{}", short_hash(&responses), width, height);
    let _chart_data = if let Some(buf) = cache_get(&cache_key) {
        buf
    } else {
        let buf = match create_performance_heatmap(responses, width, height) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to create heatmap: {}", e);
                vec![255; (width * height * 3) as usize]
            }
        };
        cache_set(cache_key, buf.clone());
        buf
    };

    flex((
        label("🗓️ Performance Heatmap").brush(Color::from_rgb8(102, 126, 234)),
        label("Performance by time of day and day of week").brush(Color::from_rgb8(128, 128, 128)),
        if responses.len() >= 20 {
            label("Darker colors indicate better performance").brush(Color::from_rgb8(46, 213, 115))
        } else {
            label(format!(
                "Need {} more responses for meaningful heatmap",
                20 - responses.len()
            ))
            .brush(Color::from_rgb8(255, 165, 2))
        },
    ))
    .direction(Axis::Vertical)
}

/// Create a metrics radar chart component
pub fn metrics_radar_chart(
    metrics: &PerformanceMetrics,
    width: u32,
    height: u32,
) -> impl WidgetView<AppData> {
    // Generate the radar chart
    let cache_key = format!("radar:{}:{:.0}:{:.0}", short_hash(metrics), width, height);
    let _chart_data = if let Some(buf) = cache_get(&cache_key) {
        buf
    } else {
        let buf = match create_metrics_radar_chart(metrics, width, height) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to create radar chart: {}", e);
                vec![255; (width * height * 3) as usize]
            }
        };
        cache_set(cache_key, buf.clone());
        buf
    };

    flex((
        label("🎯 Performance Radar").brush(Color::from_rgb8(102, 126, 234)),
        label("Multi-dimensional performance analysis").brush(Color::from_rgb8(128, 128, 128)),
        flex((
            label(format!("Accuracy: {:.1}%", metrics.accuracy_rate * 100.0))
                .brush(get_metric_color(metrics.accuracy_rate)),
            label(format!("Speed: {:.0}ms", metrics.average_response_time_ms))
                .brush(get_speed_color(metrics.average_response_time_ms)),
            label(format!("Streak: {}", metrics.best_streak)).brush(Color::from_rgb8(255, 165, 2)),
        ))
        .direction(Axis::Horizontal),
    ))
    .direction(Axis::Vertical)
}

/// Create a progress ring component
pub fn progress_ring_chart(
    percentage: f64,
    label_text: &str,
    size: u32,
) -> impl WidgetView<AppData> {
    // Generate the progress ring
    let cache_key = format!(
        "ring:{:.2}:{:.0}:{:.0}:{}",
        percentage, size, size, label_text
    );
    let _chart_data = if let Some(buf) = cache_get(&cache_key) {
        buf
    } else {
        let buf = match create_progress_ring(percentage, size, size, label_text) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to create progress ring: {}", e);
                vec![255; (size * size * 3) as usize]
            }
        };
        cache_set(cache_key, buf.clone());
        buf
    };

    flex((
        label(format!("{:.0}%", percentage)).brush(get_metric_color(percentage / 100.0)),
        label(label_text).brush(Color::from_rgb8(128, 128, 128)),
    ))
    .direction(Axis::Vertical)
}

/// Create a scatter plot component
pub fn scatter_plot_chart(
    data: &[(f64, f64)],
    _width: u32,
    _height: u32,
    _x_label: &str,
    _y_label: &str,
    title: &str,
) -> impl WidgetView<AppData> {
    // Generate the scatter plot
    // Note: scatter plot not implemented in visualizations.rs yet; show placeholder for now
    let _chart_data = match (|| -> Result<Vec<u8>, String> { Err("not implemented".into()) })() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to create scatter plot: {}", e);
            vec![255; (400 * 300 * 3) as usize] // Default fallback size
        }
    };

    flex((
        label(title).brush(Color::from_rgb8(102, 126, 234)),
        label(format!("{} data points", data.len())).brush(Color::from_rgb8(128, 128, 128)),
        if data.len() >= 2 {
            // Calculate correlation coefficient
            let n = data.len() as f64;
            let sum_x: f64 = data.iter().map(|(x, _)| x).sum();
            let sum_y: f64 = data.iter().map(|(_, y)| y).sum();
            let sum_xx: f64 = data.iter().map(|(x, _)| x * x).sum();
            let sum_yy: f64 = data.iter().map(|(_, y)| y * y).sum();
            let sum_xy: f64 = data.iter().map(|(x, y)| x * y).sum();

            let correlation = (n * sum_xy - sum_x * sum_y)
                / ((n * sum_xx - sum_x * sum_x) * (n * sum_yy - sum_y * sum_y)).sqrt();

            label(format!("Correlation: {:.3}", correlation)).brush(if correlation.abs() > 0.7 {
                Color::from_rgb8(46, 213, 115)
            } else if correlation.abs() > 0.4 {
                Color::from_rgb8(255, 165, 2)
            } else {
                Color::from_rgb8(128, 128, 128)
            })
        } else {
            label("Need more data points").brush(Color::from_rgb8(128, 128, 128))
        },
    ))
    .direction(Axis::Vertical)
}

/// Create a comprehensive dashboard visualization panel
pub fn visualization_dashboard(data: &AppData) -> impl WidgetView<AppData> {
    flex((
        // Row 1: Learning curve and response time
        flex((
            learning_curve_chart(&data.session_responses, 400, 300),
            response_time_histogram_chart(
                &data
                    .session_responses
                    .iter()
                    .map(|r| r.response_time_ms as u128)
                    .collect::<Vec<_>>(),
                400,
                300,
            ),
        ))
        .direction(Axis::Horizontal),
        // Row 2: Heatmap and radar chart
        flex((
            performance_heatmap_chart(&data.session_responses, 400, 300),
            metrics_radar_chart(&data.current_metrics, 400, 300),
        ))
        .direction(Axis::Horizontal),
        // Row 3: Progress rings
        flex((
            progress_ring_chart(data.current_metrics.accuracy_rate * 100.0, "Accuracy", 150),
            progress_ring_chart(
                (data.current_metrics.streak_count as f64 / 10.0 * 100.0).min(100.0),
                "Streak",
                150,
            ),
            progress_ring_chart(
                data.current_metrics.improvement_rate * 100.0,
                "Improvement",
                150,
            ),
        ))
        .direction(Axis::Horizontal),
    ))
    .direction(Axis::Vertical)
}

// Helper function to get color based on metric value
fn get_metric_color(value: f64) -> Color {
    if value >= 0.8 {
        Color::from_rgb8(46, 213, 115) // Success green
    } else if value >= 0.6 {
        Color::from_rgb8(0, 123, 255) // Info blue
    } else if value >= 0.4 {
        Color::from_rgb8(255, 165, 2) // Warning orange
    } else {
        Color::from_rgb8(255, 71, 87) // Error red
    }
}

// Helper function to get color based on speed
fn get_speed_color(ms: f64) -> Color {
    if ms < 1000.0 {
        Color::from_rgb8(46, 213, 115) // Fast - green
    } else if ms < 2000.0 {
        Color::from_rgb8(0, 123, 255) // Good - blue
    } else if ms < 3000.0 {
        Color::from_rgb8(255, 165, 2) // Slow - orange
    } else {
        Color::from_rgb8(255, 71, 87) // Very slow - red
    }
}

/// Create a mini sparkline chart for inline display
pub fn sparkline(
    values: &[f64],
    width: u32,
    height: u32,
    color: Color,
) -> impl WidgetView<AppData> {
    if values.is_empty() {
        return label("—").brush(Color::from_rgb8(128, 128, 128));
    }

    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = max - min;

    // Create a simple ASCII sparkline for now
    let sparkline_chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let sparkline: String = values
        .iter()
        .map(|&v| {
            let normalized = if range > 0.0 { (v - min) / range } else { 0.5 };
            let index = (normalized * 7.0).round() as usize;
            sparkline_chars[index.min(7)]
        })
        .collect();

    label(sparkline).brush(color)
}

/// Create a comparison chart for multiple sessions
pub fn session_comparison_chart(
    sessions: &[Vec<TaskResponse>],
    _width: u32,
    _height: u32,
) -> impl WidgetView<AppData> {
    flex((
        label("📊 Session Comparison").brush(Color::from_rgb8(102, 126, 234)),
        if sessions.len() >= 2 {
            let rows: Vec<_> = sessions
                .iter()
                .enumerate()
                .map(|(i, session)| {
                    let accuracy = session.iter().filter(|r| r.correct).count() as f64
                        / session.len().max(1) as f64
                        * 100.0;
                    flex((
                        label(format!("Session {}", i + 1)).brush(Color::from_rgb8(128, 128, 128)),
                        label(format!("{:.1}%", accuracy))
                            .brush(get_metric_color(accuracy / 100.0)),
                    ))
                    .direction(Axis::Horizontal)
                })
                .collect();
            flex(rows).direction(Axis::Vertical)
        } else {
            let rows: Vec<_> = vec![flex((
                label("Complete more sessions to see comparisons")
                    .brush(Color::from_rgb8(128, 128, 128)),
                label(""),
            ))
            .direction(Axis::Horizontal)];
            flex(rows).direction(Axis::Vertical)
        },
    ))
    .direction(Axis::Vertical)
}
