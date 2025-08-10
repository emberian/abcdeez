use xilem::{
    view::{button, flex, label, prose, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::{components::*, models::*, AppData};

// Domain Selection Screen
pub fn domain_selection_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    let domains = vec![
        Domain::Alphabet,
        Domain::DaysOfWeek,
        Domain::Music,
        Domain::Mathematics,
    ];

    let domain_cards = domains
        .into_iter()
        .map(|domain| {
            let is_selected = data.selected_domain == domain;
            let header = prose(domain.description()).alignment(TextAlignment::Start);

            // Add topology preview
            let preview_text = match domain {
                Domain::Alphabet => "Preview: A → B → C → D → ... → Z",
                Domain::DaysOfWeek => "Preview: Mon → Tue → Wed → Thu → Fri → Sat → Sun → Mon",
                Domain::Music => "Preview: C → D → E → F → G → A → B → C",
                Domain::Mathematics => "Preview: 1 + 1 = 2, 2 + 2 = 4, 3 × 3 = 9, ...",
                Domain::Custom(_) => "Preview: Custom learning domain",
            };

            let preview = prose(preview_text)
                .brush(Color::from_rgb8(100, 100, 100))
                .alignment(TextAlignment::Start);

            // Enhanced select button with better feedback
            let d_cloned = domain.clone();
            let button_text = if is_selected {
                "✓ Selected"
            } else {
                "Select"
            };

            let select_button = button(button_text, move |data: &mut AppData| {
                if data.selected_domain != d_cloned {
                    data.selected_domain = d_cloned.clone();
                    data.create_learner();
                    data.success_message =
                        Some(format!("Selected {} domain", d_cloned.display_name()));
                }
            });

            // Add visual indicator for selected domain
            let card_content = if is_selected {
                card(
                    &format!("✓ {}", domain.display_name()),
                    flex((header, preview, select_button)).direction(Axis::Vertical),
                )
            } else {
                card(
                    domain.display_name(),
                    flex((header, preview, select_button)).direction(Axis::Vertical),
                )
            };

            card_content
        })
        .collect::<Vec<_>>();

    flex((
        label("Select Learning Domain")
            .brush(Color::from_rgb8(0, 128, 255))
            .alignment(TextAlignment::Middle),
        prose("Choose a domain to begin your adaptive training session. Each domain offers unique learning challenges.")
            .alignment(TextAlignment::Middle),
        // Current selection card (always rendered; shows placeholders when not selected)
        {
            let (domain_label, nodes_label) = if let Some(topology) = &data.topology {
                (
                    format!("Domain: {}", data.selected_domain.display_name()),
                    format!("Total nodes: {}", topology.nodes.len()),
                )
            } else {
                (
                    "Domain: (none)".to_string(),
                    "Total nodes: —".to_string(),
                )
            };

            card(
                "Current Selection",
                flex((
                    label(domain_label).alignment(TextAlignment::Start),
                    label(nodes_label).alignment(TextAlignment::Start),
                ))
                .direction(Axis::Vertical),
            )
        },
        flex(domain_cards).direction(Axis::Vertical),
        card(
            "Session Settings",
            flex((
                checkbox(
                    data.use_adaptive_scheduling,
                    "Use Adaptive Scheduling (AI-powered task selection)",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.use_adaptive_scheduling = checked;
                    }),
                ),
                checkbox(
                    data.enable_hints,
                    "Enable Hints (Show help when struggling)",
                    std::sync::Arc::new(|data: &mut AppData, checked: bool| {
                        data.enable_hints = checked;
                    }),
                ),
            ))
            .direction(Axis::Vertical),
        ),
        button(
            if data.create_session_in_flight {
                "Starting session..."
            } else {
                "Start Training Session"
            },
            |data: &mut AppData| {
                if !data.create_session_in_flight {
                    data.start_session();
                }
            },
        ),
    ))
    .direction(Axis::Vertical)
}
