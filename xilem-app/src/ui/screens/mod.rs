// Screen modules for the Xilem UI application

mod dashboard;
mod domain_selection;
mod research_dashboard;
mod settings;
mod training;
mod visualizations;
mod welcome;
mod widget_gallery;

// Re-export all screen functions
pub use dashboard::dashboard_screen;
pub use domain_selection::domain_selection_screen;
pub use research_dashboard::research_dashboard_screen;
pub use settings::settings_screen;
pub use training::training_screen;
pub use visualizations::visualizations_screen;
pub use welcome::welcome_screen;
pub use widget_gallery::widget_gallery_screen;
