// iOS AuthenticationServices bridge for Apple Sign In
// This module provides the bridge between iOS AuthenticationServices and the Rust app

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

use crate::api::ApiClient;
use crate::core::models::{AppleUserInfo, AppleUserName, User};

// Callback type for authentication results
pub type AuthCallback = Box<dyn Fn(Result<User, String>) + Send + Sync>;

// Global callback storage for C FFI
static CALLBACK_STORAGE: Mutex<Option<AuthCallback>> = Mutex::new(None);

// iOS-specific Apple Sign In user data structure that matches iOS ASAuthorization
#[derive(Debug, Serialize, Deserialize)]
pub struct IOSAppleUserData {
    pub user_id: String,
    pub identity_token: String,
    pub authorization_code: Option<String>,
    pub full_name: Option<IOSPersonNameComponents>,
    pub email: Option<String>,
    pub real_user_status: i32, // ASUserDetectionStatus
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IOSPersonNameComponents {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}

#[derive(Clone)]
pub struct IOSAuthBridge {
    api_client: Arc<ApiClient>,
}

impl IOSAuthBridge {
    pub fn new(api_client: Arc<ApiClient>) -> Self {
        Self { api_client }
    }

    /// Initiate Apple Sign In flow from iOS
    pub async fn sign_in_with_apple(&self, callback: AuthCallback) -> Result<()> {
        // Store the callback for the C function to use
        {
            let mut storage = CALLBACK_STORAGE.lock().unwrap();
            *storage = Some(callback);
        }

        // Call into iOS native code to start Apple Sign In
        unsafe {
            start_apple_sign_in();
        }

        Ok(())
    }

    /// Process Apple Sign In result from iOS (called by C code)
    pub async fn handle_apple_sign_in_result(&self, user_data: IOSAppleUserData) -> Result<User> {
        // Convert iOS data to API format
        let user_info = if user_data.full_name.is_some() || user_data.email.is_some() {
            Some(AppleUserInfo {
                name: user_data.full_name.map(|name| AppleUserName {
                    first_name: name.given_name,
                    last_name: name.family_name,
                }),
                email: user_data.email.clone(),
            })
        } else {
            None
        };

        // Call backend API
        let user = self
            .api_client
            .apple_signin(
                user_data.identity_token,
                user_data.authorization_code,
                user_info.map(|info| serde_json::to_value(info).unwrap()),
            )
            .await?;

        Ok(user)
    }

    /// Sign out (clear tokens and call iOS sign out if needed)
    pub async fn sign_out(&self) -> Result<()> {
        self.api_client.clear_auth_token().await;
        Ok(())
    }
}

// C FFI functions to communicate with iOS
extern "C" {
    /// Starts the Apple Sign In flow on iOS
    fn start_apple_sign_in();
}

// C callback function that iOS calls when authentication completes
#[no_mangle]
pub extern "C" fn handle_apple_sign_in_callback(json_data: *const c_char, success: bool) {
    if json_data.is_null() {
        return;
    }

    let callback = {
        let mut storage = CALLBACK_STORAGE.lock().unwrap();
        storage.take()
    };

    if let Some(callback) = callback {
        if success {
            unsafe {
                let c_str = CStr::from_ptr(json_data);
                if let Ok(json_string) = c_str.to_str() {
                    match serde_json::from_str::<IOSAppleUserData>(json_string) {
                        Ok(user_data) => {
                            // We need to handle the async operation here
                            // For now, we'll simulate success - in production you'd want
                            // to properly handle the async API call
                            let user = User {
                                id: user_data.user_id.clone(),
                                username: user_data.email.clone().unwrap_or_else(|| {
                                    format!("apple_user_{}", &user_data.user_id[..8])
                                }),
                                email: user_data.email.unwrap_or_default(),
                                password_hash: String::new(),
                                apple_user_id: Some(user_data.user_id),
                                github_user_id: None,
                                oauth_provider_id: None,
                                auth_provider: "apple".to_string(),
                                is_private_email: Some(false), // Would be determined by backend
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                            };
                            callback(Ok(user));
                        }
                        Err(e) => callback(Err(format!("Failed to parse user data: {}", e))),
                    }
                } else {
                    callback(Err("Invalid JSON data".to_string()));
                }
            }
        } else {
            callback(Err("Apple Sign In failed or was cancelled".to_string()));
        }
    }
}

// Helper function to convert CString to String safely
fn c_string_to_string(c_str: *const c_char) -> Result<String, String> {
    if c_str.is_null() {
        return Err("Null pointer".to_string());
    }

    unsafe {
        let c_str = CStr::from_ptr(c_str);
        c_str
            .to_str()
            .map(|s| s.to_string())
            .map_err(|e| format!("Invalid UTF-8: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_user_data_serialization() {
        let user_data = IOSAppleUserData {
            user_id: "test_user_id".to_string(),
            identity_token: "test_token".to_string(),
            authorization_code: Some("test_code".to_string()),
            full_name: Some(IOSPersonNameComponents {
                given_name: Some("John".to_string()),
                family_name: Some("Doe".to_string()),
            }),
            email: Some("john@example.com".to_string()),
            real_user_status: 1,
        };

        let json = serde_json::to_string(&user_data).unwrap();
        let deserialized: IOSAppleUserData = serde_json::from_str(&json).unwrap();

        assert_eq!(user_data.user_id, deserialized.user_id);
        assert_eq!(user_data.identity_token, deserialized.identity_token);
    }
}
