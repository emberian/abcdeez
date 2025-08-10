use xilem::{
    view::{button, flex, label, prose, Axis, FlexExt},
    TextAlignment, WidgetView,
};

use crate::{apple_signin_button::standard_apple_signin_button, components::*, AppData, Screen};

// Welcome/Login Screen
pub fn welcome_screen(data: &mut AppData) -> impl WidgetView<AppData> {
    flex((
        button("Adaptive Learning System", |data: &mut AppData| {
            data.try_crab_triple_click();
        }),

        prose("An intelligent learning system that adapts to your knowledge and optimizes your learning path using graph-based cognitive models."),

        card("Login", flex((
            labeled_input(
                "Username:",
                data.username_input.clone(),
                std::sync::Arc::new(|data: &mut AppData, value: String| {
                    data.username_input = value;
                }),
            ),
            labeled_input(
                "Password:",
                data.password_input.clone(),
                std::sync::Arc::new(|data: &mut AppData, value: String| {
                    data.password_input = value;
                }),
            ),
            // Render a visually different label when in-flight, otherwise an active button
            if data.login_request_in_flight {
                label("Logging in...").alignment(TextAlignment::Middle).into_any_flex()
            } else {
                button("Login",
                |data: &mut AppData| {
                    if !data.login_request_in_flight {
                        data.login();
                    }
                }).into_any_flex()
            },
            // OAuth Section - App Store Compliant
            label("── Or ──").alignment(TextAlignment::Middle),
            // App Store compliant Apple Sign In button
            standard_apple_signin_button(data),
        )).direction(Axis::Vertical)),

        card("Quick Start", flex((
            prose("Start learning immediately without creating an account")
                .alignment(TextAlignment::Middle),
            button("Start as Guest", |data: &mut AppData| {
                // Proper guest/anonymous user pattern
                data.current_user = None; // No fake user object
                data.is_guest_mode = true;
                data.create_learner();
                data.current_screen = Screen::DomainSelection;
            }),
            button("Quick Tour", |data: &mut AppData| {
                // Start the interactive guided tour
                data.demo_start();
            }),
            button("Training Demo", |data: &mut AppData| {
                // Start the full training demo
                data.demo_start_training();
            }),
            button("Demo Showcase", |data: &mut AppData| {
                // Run a short automated demo training sequence and navigate to Dashboard
                data.demo_showcase();
                data.current_screen = Screen::Dashboard;
            }),
            // Always-available navigation to avoid dead ends
            button("Go to Dashboard", |data: &mut AppData| {
                data.current_screen = Screen::Dashboard;
            }),
        )).direction(Axis::Vertical)),
    ))
    .direction(Axis::Vertical)
}
