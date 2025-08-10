use std::sync::Arc;

use xilem::{
    view::{button, flex, label, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::{components::*, visualization_components::*, AppData};

pub fn widget_gallery_screen(data: &AppData) -> impl WidgetView<AppData> {
    let demo_metrics = data.current_metrics.clone();
    let rt_vec: Vec<u128> = data
        .session_responses
        .iter()
        .map(|r| r.response_time_ms as u128)
        .collect();

    let demo_task = crate::models::UITask {
        core_task: abcdeez_core::tasks::Task {
            task_type: abcdeez_core::tasks::TaskType::Successor { item: "A".into() },
            prompt: "What comes after A?".into(),
            correct_answer: "B".into(),
            options: vec!["A".into(), "B".into(), "C".into()],
            difficulty: 0.5,
            operation: abcdeez_core::learner::OperationType::Successor,
        },
        display_prompt: "What comes after A?".into(),
        display_options: vec!["A".into(), "B".into(), "C".into()],
        hint: Some("Think alphabetically".into()),
        feedback_message: None,
    };

    let answer_on_select = Arc::new(|_data: &mut AppData, _idx: usize| {});

    let section1 = card(
        "Overview",
        flex((
            label("These are experimental/unused components. This page is hidden.")
                .brush(Color::from_rgb8(128, 128, 128))
                .alignment(TextAlignment::Middle),
        ))
        .direction(Axis::Vertical),
    );

    let section2 = card(
        "Basic Components",
        flex((
            card("Sample Card", label("Card content")),
            progress_bar(0.42, "Upload Progress".into()),
            metric_display(
                "Accuracy",
                format!("{:.1}%", demo_metrics.accuracy_rate * 100.0),
                Color::from_rgb8(0, 200, 0),
            ),
            task_card(&demo_task),
            answer_options(
                vec!["Alpha".into(), "Bravo".into(), "Charlie".into()],
                answer_on_select.clone(),
            ),
            performance_chart(&demo_metrics),
            loading_overlay("Processing data..."),
            toast_notification("Demo toast from gallery".into(), true),
            labeled_input("Your name", "Alice".into(), Arc::new(|_d, _s| {})),
            checkbox(false, "I agree", Arc::new(|_d, _b| {})),
        ))
        .direction(Axis::Vertical),
    );

    let section3 = card(
        "Charts",
        flex((
            learning_curve_chart(&data.session_responses, 400, 240),
            response_time_histogram_chart(&rt_vec, 400, 240),
            performance_heatmap_chart(&data.session_responses, 400, 240),
            metrics_radar_chart(&demo_metrics, 400, 240),
            progress_ring_chart(demo_metrics.accuracy_rate * 100.0, "Accuracy", 120),
            scatter_plot_chart(
                &[(0.0, 1.0), (1.0, 0.6), (2.0, 0.8)],
                400,
                240,
                "x",
                "y",
                "Demo Scatter",
            ),
            sparkline(
                &[0.1, 0.3, 0.2, 0.5, 0.7, 0.4],
                120,
                24,
                Color::from_rgb8(0, 123, 255),
            ),
        ))
        .direction(Axis::Vertical),
    );

    card(
        "🧪 Hidden Widget Gallery",
        flex((section1, section2, section3)).direction(Axis::Vertical),
    )
}
