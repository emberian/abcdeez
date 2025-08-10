// Platform integrations module

#[cfg(target_os = "ios")]
pub mod ios_auth;
pub mod apple_signin;

// Re-export integration items based on platform
#[cfg(target_os = "ios")]
pub use ios_auth::*;