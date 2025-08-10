// API integration module

pub mod client;
pub mod models;

// Re-export the main types
pub use client::{ApiClientTrait, AdaptiveApiClient};
pub use models::*;