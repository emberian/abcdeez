// API-specific request/response models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::models::{User, AppleUserInfo, ProtocolContent, ProtocolVersion, ProtocolVersionDiff, Session};

// Request/Response DTOs for API communication
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub learner_id: String,
    pub topology_type: String,
    pub topology_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponseRequest {
    pub task_type: String,
    pub task_data: serde_json::Value,
    pub user_answer: String,
    pub response_time_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResponseResponse {
    pub response_id: String,
    pub sequence_number: u32,
    pub correct: bool,
    pub intervention: Option<serde_json::Value>,
    pub next_task: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppleSignInRequest {
    pub identity_token: String,
    pub authorization_code: Option<String>,
    pub user_info: Option<AppleUserInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCallbackRequest {
    pub provider: String,
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthAuthUrlResponse {
    pub authorization_url: String,
    pub state: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLearnerRequest {
    pub display_name: Option<String>,
}