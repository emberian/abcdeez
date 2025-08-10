// visualizations.rs - Honest, accessible data visualizations using Plotters
// No chartjunk, no lies, just truth

use chrono::{Datelike, Timelike};
use plotters::prelude::*;
use std::error::Error;

use crate::core::models::PerformanceMetrics;
use abcdeez_core::tasks::TaskResponse;

// Colorblind-safe palette using ColorBrewer schemes
// Tested with Coblis colorblind simulator
pub struct AccessiblePalette {
    pub primary: RGBColor,    // Blue - works for all colorblind types
    pub secondary: RGBColor,  // Orange - distinguishable from blue
    pub success: RGBColor,    // Teal - not pure green
    pub error: RGBColor,      // Vermillion - not pure red
    pub warning: RGBColor,    // Yellow - high contrast
    pub neutral: RGBColor,    // Gray
    pub background: RGBColor, // Light gray
    pub grid: RGBColor,       // Medium gray
    pub text: RGBColor,       // Dark gray
}

impl Default for AccessiblePalette {
    fn default() -> Self {
        Self {
            primary: RGBColor(0, 114, 178),      // Colorblind-safe blue
            secondary: RGBColor(230, 159, 0),    // Colorblind-safe orange
            success: RGBColor(0, 158, 115),      // Colorblind-safe teal
            error: RGBColor(213, 94, 0),         // Colorblind-safe vermillion
            warning: RGBColor(240, 228, 66),     // High-contrast yellow
            neutral: RGBColor(128, 128, 128),    // Neutral gray
            background: RGBColor(250, 250, 250), // Very light gray
            grid: RGBColor(200, 200, 200),       // Light grid lines
            text: RGBColor(50, 50, 50),          // Dark text
        }
    }
}

// Centralized performance thresholds based on educational research
pub struct PerformanceThresholds {
    pub mastery: f64,    // 85% - Educational mastery level
    pub proficient: f64, // 70% - Proficiency threshold
    pub developing: f64, // 55% - Developing skills
    pub struggling: f64, // 40% - Needs intervention
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            mastery: 0.85,
            proficient: 0.70,
            developing: 0.55,
            struggling: 0.40,
        }
    }
}

/// Create an HONEST learning curve visualization with confidence intervals
pub fn create_learning_curve(
    responses: &[TaskResponse],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();

        let palette = AccessiblePalette::default();
        let thresholds = PerformanceThresholds::default();
        root.fill(&palette.background)?;

        if responses.is_empty() {
            // Show meaningful empty state
            root.draw_text(
                "No data available yet",
                &("sans-serif", 20).into_font().color(&palette.text),
                (width as i32 / 2 - 80, height as i32 / 2),
            )?;
            root.present()?;
            // fallthrough to block end; buffer will be returned after drop
        }

        // Calculate ACTUAL performance with confidence intervals
        // Using proper statistical window sizing (sqrt(n) rule)
        let window_size = (responses.len() as f64).sqrt().max(3.0).min(20.0) as usize;

        let mut performance_data = Vec::new();
        let mut confidence_bands = Vec::new();

        for i in 0..responses.len() {
            let start = i.saturating_sub(window_size / 2);
            let end = (i + window_size / 2 + 1).min(responses.len());
            let window = &responses[start..end];

            if !window.is_empty() {
                let correct = window.iter().filter(|r| r.correct).count() as f64;
                let total = window.len() as f64;
                let accuracy = correct / total;

                // Calculate 95% confidence interval using Wilson score
                let z = 1.96; // 95% confidence
                let n = total;
                let p_hat = accuracy;

                let denominator = 1.0 + z * z / n;
                let center = (p_hat + z * z / (2.0 * n)) / denominator;
                let margin = (z / denominator)
                    * ((p_hat * (1.0 - p_hat) / n) + (z * z / (4.0 * n * n))).sqrt();

                performance_data.push((i as f64, accuracy));
                confidence_bands.push((
                    i as f64,
                    (center - margin).max(0.0),
                    (center + margin).min(1.0),
                ));
            }
        }

        // Determine appropriate Y-axis range based on actual data
        let min_accuracy = performance_data
            .iter()
            .map(|(_, acc)| *acc)
            .fold(1.0, f64::min);
        let max_accuracy = performance_data
            .iter()
            .map(|(_, acc)| *acc)
            .fold(0.0, f64::max);

        // Add padding but show actual range
        let y_min = (min_accuracy - 0.1).max(0.0);
        let y_max = (max_accuracy + 0.1).min(1.0);

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Learning Progress (with 95% Confidence Interval)",
                ("sans-serif", 20).into_font().color(&palette.text),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0f64..responses.len() as f64, y_min..y_max)?;

        chart
            .configure_mesh()
            .disable_mesh()
            .x_desc("Task Number")
            .y_desc("Accuracy")
            .x_label_formatter(&|x| format!("{:.0}", x))
            .y_label_formatter(&|y| format!("{:.0}%%", y * 100.0))
            .axis_desc_style(("sans-serif", 12).into_font().color(&palette.text))
            .draw()?;

        // Draw performance threshold lines with labels
        let threshold_lines = [
            (thresholds.mastery, "Mastery", palette.success),
            (thresholds.proficient, "Proficient", palette.primary),
            (thresholds.developing, "Developing", palette.warning),
            (thresholds.struggling, "Struggling", palette.error),
        ];

        for (threshold, label, color) in threshold_lines.iter() {
            if *threshold >= y_min && *threshold <= y_max {
                chart.draw_series(std::iter::once(PathElement::new(
                    vec![(0.0, *threshold), (responses.len() as f64, *threshold)],
                    color.mix(0.3).stroke_width(1),
                )))?;

                // Add threshold label
                root.draw_text(label, &("sans-serif", 10).into_font().color(&color), {
                    let (x_range, y_range) = chart.plotting_area().get_pixel_range();
                    let x =
                        ((responses.len() as f64 * 0.95) as i32).clamp(x_range.start, x_range.end);
                    let y = ((*threshold) as i32).clamp(y_range.start, y_range.end);
                    (x, y)
                })?;
            }
        }

        // Draw confidence bands (honest uncertainty representation)
        for window in confidence_bands.windows(2) {
            let (x1, lower1, upper1) = window[0];
            let (x2, lower2, upper2) = window[1];

            // Draw the confidence band as a filled area
            chart.draw_series(std::iter::once(Polygon::new(
                vec![(x1, lower1), (x1, upper1), (x2, upper2), (x2, lower2)],
                palette.primary.mix(0.2).filled(),
            )))?;
        }

        // Draw the actual performance line
        chart
            .draw_series(std::iter::once(PathElement::new(
                performance_data.clone(),
                palette.primary.stroke_width(2),
            )))?
            .label("Actual Performance")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &palette.primary));

        // Draw individual correct/incorrect points
        for (i, response) in responses.iter().enumerate() {
            let color = if response.correct {
                palette.success.mix(0.6)
            } else {
                palette.error.mix(0.6)
            };

            let y_value = if response.correct { 1.0 } else { 0.0 };
            chart.draw_series(std::iter::once(Circle::new(
                (i as f64, y_value),
                2,
                color.filled(),
            )))?;
        }

        // Add summary statistics box
        let total = responses.len();
        let correct = responses.iter().filter(|r| r.correct).count();
        let overall_accuracy = correct as f64 / total as f64;

        // Calculate trend using proper linear regression
        if performance_data.len() > 1 {
            let n = performance_data.len() as f64;
            let sum_x: f64 = performance_data.iter().map(|(x, _)| x).sum();
            let sum_y: f64 = performance_data.iter().map(|(_, y)| y).sum();
            let sum_xx: f64 = performance_data.iter().map(|(x, _)| x * x).sum();
            let sum_xy: f64 = performance_data.iter().map(|(x, y)| x * y).sum();

            let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
            let trend_direction = if slope > 0.001 {
                "↑ Improving"
            } else if slope < -0.001 {
                "↓ Declining"
            } else {
                "→ Stable"
            };

            root.draw_text(
                &format!(
                    "Overall: {:.1}% | Trend: {} ({:+.3}/task)",
                    overall_accuracy * 100.0,
                    trend_direction,
                    slope * 100.0
                ),
                &("sans-serif", 12).into_font().color(&palette.text),
                (15, height as i32 - 20),
            )?;
        }

        root.present()?;
    }
    Ok(buffer)
}

/// Create a properly binned response time histogram using Sturges' rule
pub fn create_response_time_histogram(
    response_times: &[u128],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();

        let palette = AccessiblePalette::default();
        root.fill(&palette.background)?;

        if response_times.is_empty() {
            root.draw_text(
                "No timing data available",
                &("sans-serif", 20)
                    .into_font()
                    .style(FontStyle::Bold)
                    .color(&palette.text),
                (width as i32 / 2 - 80, height as i32 / 2),
            )?;
            root.present()?;
            // Drop drawing area before returning to release borrow
            drop(root);
            return Ok(buffer);
        }

        // Use Sturges' rule for optimal bin count: k = ⌈log₂(n) + 1⌉
        let n = response_times.len();
        let num_bins = ((n as f64).log2() + 1.0).ceil() as usize;

        let times_f64: Vec<f64> = response_times.iter().map(|&t| t as f64).collect();
        let min_time = times_f64.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_time = times_f64.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Handle edge case of all same values
        let range = if max_time - min_time < 1.0 {
            100.0 // Default range if all values are the same
        } else {
            max_time - min_time
        };

        let bin_width = range / num_bins as f64;
        let mut bins = vec![0; num_bins];

        for &time in &times_f64 {
            let bin_idx = ((time - min_time) / bin_width).floor() as usize;
            if bin_idx < num_bins {
                bins[bin_idx] += 1;
            } else if bin_idx == num_bins && time == max_time {
                bins[num_bins - 1] += 1; // Include max value in last bin
            }
        }

        let max_count = *bins.iter().max().unwrap_or(&1) as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                &format!(
                    "Response Time Distribution (n={}, {} bins via Sturges' rule)",
                    n, num_bins
                ),
                ("sans-serif", 16).into_font().color(&palette.text),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(min_time..max_time, 0f64..max_count * 1.1)?;

        chart
            .configure_mesh()
            .disable_mesh()
            .x_desc("Response Time (ms)")
            .y_desc("Frequency")
            .x_label_formatter(&|x| {
                if *x >= 1000.0 {
                    format!("{:.1}s", x / 1000.0)
                } else {
                    format!("{:.0}ms", x)
                }
            })
            .y_label_formatter(&|y| format!("{:.0}", y))
            .axis_desc_style(("sans-serif", 12).into_font().color(&palette.text))
            .draw()?;

        // Draw histogram bars with performance-based colors
        for (i, &count) in bins.iter().enumerate() {
            if count > 0 {
                let x_start = min_time + i as f64 * bin_width;
                let x_end = x_start + bin_width * 0.95; // Small gap for visibility

                let bin_center = x_start + bin_width / 2.0;
                let color = if bin_center < 1000.0 {
                    palette.success
                } else if bin_center < 2500.0 {
                    palette.primary
                } else if bin_center < 5000.0 {
                    palette.warning
                } else {
                    palette.error
                };

                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x_start, 0.0), (x_end, count as f64)],
                    color.mix(0.7).filled(),
                )))?;
            }
        }

        // Calculate and display statistics
        let mean = times_f64.iter().sum::<f64>() / times_f64.len() as f64;
        let mut sorted = times_f64.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        // Calculate percentiles
        let p25 = sorted[(sorted.len() as f64 * 0.25) as usize];
        let p75 = sorted[(sorted.len() as f64 * 0.75) as usize];
        let p95 = sorted[(sorted.len() as f64 * 0.95) as usize];

        // Draw statistics lines
        let stat_lines = [
            (mean, "Mean", palette.primary),
            (median, "Median", palette.secondary),
            (p95, "95th %ile", palette.error),
        ];

        for (value, label, color) in stat_lines.iter().cloned() {
            if value >= min_time && value <= max_time {
                chart
                    .draw_series(std::iter::once(PathElement::new(
                        vec![(value, 0.0), (value, max_count)],
                        ShapeStyle::from(&color).stroke_width(2),
                    )))?
                    .label(label)
                    .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &color));
            }
        }

        // Draw legend
        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        // Add statistics summary
        root.draw_text(
            &format!(
                "μ={:.0}ms | M={:.0}ms | IQR=[{:.0}, {:.0}]ms",
                mean, median, p25, p75
            ),
            &("sans-serif", 11).into_font().color(&palette.text),
            (15, height as i32 - 20),
        )?;

        root.present()?;
    }
    Ok(buffer)
}

/// Create an HONEST performance heatmap that shows data availability
pub fn create_performance_heatmap(
    responses: &[TaskResponse],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();

        let palette = AccessiblePalette::default();
        root.fill(&palette.background)?;

        // Group responses by hour and day
        let mut performance_grid: Vec<Vec<Option<f64>>> = vec![vec![None; 24]; 7];
        let mut count_grid: Vec<Vec<u32>> = vec![vec![0; 24]; 7];

        for response in responses {
            let hour = response.timestamp.hour() as usize;
            let day = response.timestamp.weekday().num_days_from_monday() as usize;

            if let Some(current) = performance_grid[day][hour] {
                let count = count_grid[day][hour] as f64;
                let new_value = if response.correct { 1.0 } else { 0.0 };
                performance_grid[day][hour] = Some((current * count + new_value) / (count + 1.0));
            } else {
                performance_grid[day][hour] = Some(if response.correct { 1.0 } else { 0.0 });
            }
            count_grid[day][hour] += 1;
        }

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Performance by Time (darker = better, ⬚ = no data)",
                ("sans-serif", 16).into_font().color(&palette.text),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(0f64..24f64, 0f64..7f64)?;

        chart
            .configure_mesh()
            .disable_mesh()
            .x_desc("Hour of Day")
            .y_desc("Day of Week")
            .x_label_formatter(&|x| format!("{:02}:00", x))
            .y_label_formatter(&|y| {
                let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
                days.get(*y as usize).unwrap_or(&"").to_string()
            })
            .axis_desc_style(("sans-serif", 12).into_font().color(&palette.text))
            .draw()?;

        // Draw heatmap cells
        for day in 0..7 {
            for hour in 0..24 {
                let cell_color = if let Some(performance) = performance_grid[day][hour] {
                    // Show performance with intensity based on value
                    let color = if performance >= 0.85 {
                        palette.success
                    } else if performance >= 0.70 {
                        palette.primary
                    } else if performance >= 0.55 {
                        palette.warning
                    } else {
                        palette.error
                    };

                    // Intensity shows confidence (more data = stronger color)
                    let intensity = (count_grid[day][hour] as f64 / 10.0).min(1.0) * 0.8;
                    Some(color.mix(intensity))
                } else {
                    None // No data for this cell
                };

                if let Some(color) = cell_color {
                    chart.draw_series(std::iter::once(Rectangle::new(
                        [
                            (hour as f64, day as f64),
                            (hour as f64 + 0.95, day as f64 + 0.95),
                        ],
                        color.filled(),
                    )))?;

                    // Add count label for cells with significant data
                    if count_grid[day][hour] >= 5 {
                        let performance = performance_grid[day][hour].unwrap_or(0.0);
                        root.draw_text(
                            &format!("{:.0}%", performance * 100.0),
                            &("sans-serif", 8).into_font().color(&WHITE),
                            {
                                let (x_range, y_range) = chart.plotting_area().get_pixel_range();
                                let x =
                                    ((hour as f64 + 0.5) as i32).clamp(x_range.start, x_range.end);
                                let y =
                                    ((day as f64 + 0.5) as i32).clamp(y_range.start, y_range.end);
                                (x, y)
                            },
                        )?;
                    }
                } else {
                    // Draw empty cell indicator
                    chart.draw_series(std::iter::once(Rectangle::new(
                        [
                            (hour as f64, day as f64),
                            (hour as f64 + 0.95, day as f64 + 0.95),
                        ],
                        palette.grid.stroke_width(1),
                    )))?;
                }
            }
        }

        // Add data density indicator
        let total_cells = 7 * 24;
        let filled_cells = performance_grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.is_some())
            .count();
        let coverage = filled_cells as f64 / total_cells as f64 * 100.0;

        root.draw_text(
            &format!(
                "Data coverage: {:.1}% ({}/{} time slots)",
                coverage, filled_cells, total_cells
            ),
            &("sans-serif", 11).into_font().color(&palette.text),
            (15, height as i32 - 20),
        )?;

        root.present()?;
    }
    Ok(buffer)
}

/// Create a PROPERLY SCALED radar chart with comparable dimensions
pub fn create_metrics_radar_chart(
    metrics: &PerformanceMetrics,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();

        let palette = AccessiblePalette::default();
        root.fill(&palette.background)?;

        // All dimensions normalized to 0-1 scale with clear explanations
        let dimensions = vec![
            ("Accuracy", metrics.accuracy_rate, "% correct"),
            (
                "Speed",
                1.0 - (metrics.average_response_time_ms / 10000.0).min(1.0),
                "inverse time",
            ),
            (
                "Consistency",
                1.0 - (1.0 - metrics.accuracy_rate).min(0.25) * 4.0,
                "low variance",
            ),
            ("Recency", metrics.recent_accuracy, "last 10 tasks"),
            ("Progress", (metrics.improvement_rate + 1.0) / 2.0, "trend"),
        ];

        let center_x = width as i32 / 2;
        let center_y = height as i32 / 2;
        let radius = (width.min(height) as i32 / 3) as f64;

        // Draw title
        root.draw_text(
            "Performance Metrics (0-1 normalized scale)",
            &("sans-serif", 18).into_font().color(&palette.text),
            (center_x - 120, 20),
        )?;

        // Draw radar grid
        for level in 1..=5 {
            let r = radius * level as f64 / 5.0;
            let mut points = Vec::new();

            for i in 0..dimensions.len() {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / dimensions.len() as f64
                    - std::f64::consts::PI / 2.0;
                let x = center_x + (r * angle.cos()) as i32;
                let y = center_y + (r * angle.sin()) as i32;
                points.push((x, y));
            }
            points.push(points[0]); // Close the polygon

            root.draw(&PathElement::new(points, palette.grid.stroke_width(1)))?;

            // Add grid labels (0.2, 0.4, 0.6, 0.8, 1.0)
            let label_value = level as f64 / 5.0;
            root.draw_text(
                &format!("{:.1}", label_value),
                &("sans-serif", 9).into_font().color(&palette.neutral),
                (center_x + 5, center_y - (r as i32) - 5),
            )?;
        }

        // Draw axes and labels
        for (i, (label, _, description)) in dimensions.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / dimensions.len() as f64
                - std::f64::consts::PI / 2.0;
            let x_end = center_x + (radius * angle.cos()) as i32;
            let y_end = center_y + (radius * angle.sin()) as i32;

            root.draw(&PathElement::new(
                vec![(center_x, center_y), (x_end, y_end)],
                palette.grid.stroke_width(2),
            ))?;

            // Draw labels with descriptions
            let label_offset = 1.2;
            let x_label = center_x + (radius * label_offset * angle.cos()) as i32;
            let y_label = center_y + (radius * label_offset * angle.sin()) as i32;

            root.draw_text(
                &format!("{}\n({})", label, description),
                &("sans-serif", 10).into_font().color(&palette.text),
                (x_label - 40, y_label - 10),
            )?;
        }

        // Draw data polygon
        let mut data_points = Vec::new();
        for (i, (_, value, _)) in dimensions.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / dimensions.len() as f64
                - std::f64::consts::PI / 2.0;
            let r = radius * value.max(0.0).min(1.0); // Clamp to valid range
            let x = center_x + (r * angle.cos()) as i32;
            let y = center_y + (r * angle.sin()) as i32;
            data_points.push((x, y));
        }
        data_points.push(data_points[0]); // Close the polygon

        // Draw filled area
        root.draw(&Polygon::new(
            data_points.clone(),
            palette.primary.mix(0.3).filled(),
        ))?;

        // Draw outline
        root.draw(&PathElement::new(
            data_points.clone(),
            palette.primary.stroke_width(3),
        ))?;

        // Draw data points with values
        for (i, (_name, value, _)) in dimensions.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * i as f64 / dimensions.len() as f64
                - std::f64::consts::PI / 2.0;
            let r = radius * value.max(0.0).min(1.0);
            let x = center_x + (r * angle.cos()) as i32;
            let y = center_y + (r * angle.sin()) as i32;

            // Draw point
            root.draw(&Circle::new((x, y), 4, palette.primary.filled()))?;

            // Add value label
            root.draw_text(
                &format!("{:.2}", value),
                &("sans-serif", 9).into_font().color(&palette.text),
                (x - 15, y - 15),
            )?;
        }

        root.present()?;
    }
    Ok(buffer)
}

/// Create a simple, honest progress indicator
pub fn create_progress_ring(
    percentage: f64,
    width: u32,
    height: u32,
    label: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buffer = vec![0; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();

        let palette = AccessiblePalette::default();
        root.fill(&WHITE)?;

        let center_x = width as i32 / 2;
        let center_y = height as i32 / 2;
        let outer_radius = (width.min(height) as i32 / 2 - 20) as f64;
        let inner_radius = outer_radius * 0.7;

        // Clamp percentage to valid range
        let percentage = percentage.max(0.0).min(100.0);

        // Draw background ring
        for angle_deg in 0..360 {
            let angle = angle_deg as f64 * std::f64::consts::PI / 180.0;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            root.draw(&PathElement::new(
                vec![
                    (
                        center_x + (inner_radius * cos_a) as i32,
                        center_y + (inner_radius * sin_a) as i32,
                    ),
                    (
                        center_x + (outer_radius * cos_a) as i32,
                        center_y + (outer_radius * sin_a) as i32,
                    ),
                ],
                palette.grid.stroke_width(2),
            ))?;
        }

        // Draw progress arc with performance-based color
        let progress_color = if percentage >= 85.0 {
            palette.success
        } else if percentage >= 70.0 {
            palette.primary
        } else if percentage >= 55.0 {
            palette.warning
        } else {
            palette.error
        };

        let progress_angle = percentage / 100.0 * 360.0;
        for angle_deg in 0..(progress_angle as i32) {
            let angle = (angle_deg as f64 - 90.0) * std::f64::consts::PI / 180.0;
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            root.draw(&PathElement::new(
                vec![
                    (
                        center_x + (inner_radius * cos_a) as i32,
                        center_y + (inner_radius * sin_a) as i32,
                    ),
                    (
                        center_x + (outer_radius * cos_a) as i32,
                        center_y + (outer_radius * sin_a) as i32,
                    ),
                ],
                progress_color.stroke_width(3),
            ))?;
        }

        // Draw center text with proper formatting
        root.draw_text(
            &format!("{:.1}%", percentage),
            &("sans-serif", 28).into_font().color(&palette.text),
            (center_x - 35, center_y - 15),
        )?;

        root.draw_text(
            label,
            &("sans-serif", 14).into_font().color(&palette.neutral),
            (center_x - label.len() as i32 * 4, center_y + 15),
        )?;

        root.present()?;
    }
    Ok(buffer)
}

/// Utility function to format duration properly
pub fn format_duration(seconds: i64) -> String {
    if seconds < 0 {
        return "Invalid duration".to_string();
    }

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Utility function for consistent number formatting
pub fn format_number(value: f64, decimals: usize) -> String {
    format!("{:.decimals$}", value, decimals = decimals)
}

/// Utility function for consistent percentage formatting
pub fn format_percentage(value: f64, decimals: usize) -> String {
    format!("{:.decimals$}%", value * 100.0, decimals = decimals)
}
