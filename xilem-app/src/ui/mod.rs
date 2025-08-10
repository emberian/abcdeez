// User interface module

pub mod app;
pub mod app_methods;
pub mod state;
pub mod components;
pub mod screens;

// Re-export commonly used items
pub use app::app_logic;
pub use state::{AppData, Screen, IRBFormData};