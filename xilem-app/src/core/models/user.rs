use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Core data structures for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub apple_user_id: Option<String>,
    pub github_user_id: Option<String>,
    pub oauth_provider_id: Option<String>,
    pub auth_provider: String,
    pub is_private_email: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// OAuth-related structures
#[derive(Debug, Serialize, Deserialize)]
pub struct AppleUserInfo {
    pub name: Option<AppleUserName>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppleUserName {
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
}