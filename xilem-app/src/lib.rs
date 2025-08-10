// Copyright 2024 Graph Learning System Authors
// SPDX-License-Identifier: Apache-2.0

// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

// Main modules
pub mod core;
pub mod api;
pub mod storage;
pub mod research;
pub mod ui;
pub mod integrations;
pub mod utils;

// Re-exports for backward compatibility and convenience
pub use core::models::*;
pub use ui::{AppData, Screen};

// Main entry point
pub fn run() {
    // Create the platform event loop here
    let event_loop = xilem::EventLoop::with_user_event();

    let data = AppData::default();

    let app = xilem::Xilem::new(data, ui::app_logic);
    // Run the windowed application with a title using the Xilem runtime.
    // We use the windowed runner to create a visible window for the demo / showcase.
    let _ = app.run_windowed(
        event_loop,
        "Adaptive Learning System - Alphabet Terminal".into(),
    );
}