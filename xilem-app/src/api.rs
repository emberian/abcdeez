use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::models::*;

pub struct ApiClient {
    client: Client,
    base_url: String,
    auth_token: Arc<RwLock<Option<String>>>,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            auth_token: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_auth_token(&self, token: String) {
        let mut auth = self.auth_token.write().await;
        *auth = Some(token);
    }

    pub async fn clear_auth_token(&self) {
        let mut auth = self.auth_token.write().await;
        *auth = None;
    }

    async fn get_auth_header(&self) -> Option<String> {
        let auth = self.auth_token.read().await;
        auth.as_ref().map(|token| format!("Bearer {}", token))
    }

    async fn post<T: Serialize, R: DeserializeOwned>(&self, path: &str, body: &T) -> Result<R> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.post(&url).json(body);

        if let Some(auth_header) = self.get_auth_header().await {
            request = request.header("Authorization", auth_header);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("API request failed: {}", response.status()))
        }
    }

    async fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.get(&url);

        if let Some(auth_header) = self.get_auth_header().await {
            request = request.header("Authorization", auth_header);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("API request failed: {}", response.status()))
        }
    }

    // Authentication endpoints
    pub async fn login(&self, username: String, password: String) -> Result<User> {
        let request = LoginRequest { username, password };
        let response: TokenResponse = self.post("/api/auth/login", &request).await?;

        // Store the access token
        self.set_auth_token(response.access_token).await;

        // Get user info with the token
        let user: User = self.get("/api/auth/me").await?;
        Ok(user)
    }

    // OAuth Authentication endpoints
    pub async fn apple_signin(
        &self,
        identity_token: String,
        authorization_code: Option<String>,
        user_info: Option<AppleUserInfo>,
    ) -> Result<User> {
        let request = AppleSignInRequest {
            identity_token,
            authorization_code,
            user_info,
        };
        let response: TokenResponse = self.post("/api/auth/apple/signin", &request).await?;

        // Store the access token
        self.set_auth_token(response.access_token).await;

        // Return the user info from the response
        Ok(User {
            id: response.user.id,
            username: response.user.username,
            email: response.user.email,
            password_hash: String::new(),
            apple_user_id: None,
            github_user_id: None,
            oauth_provider_id: None,
            auth_provider: "apple".to_string(),
            is_private_email: None,
            created_at: response.user.created_at,
            updated_at: response.user.updated_at,
        })
    }

    pub async fn github_oauth_callback(
        &self,
        provider: String,
        code: String,
        state: String,
    ) -> Result<User> {
        let request = OAuthCallbackRequest {
            provider,
            code,
            state,
        };
        let response: TokenResponse = self.post("/api/auth/oauth/callback", &request).await?;

        // Store the access token
        self.set_auth_token(response.access_token).await;

        // Return the user info from the response
        Ok(User {
            id: response.user.id,
            username: response.user.username,
            email: response.user.email,
            password_hash: String::new(),
            apple_user_id: None,
            github_user_id: None,
            oauth_provider_id: None,
            auth_provider: "github".to_string(),
            is_private_email: Some(false),
            created_at: response.user.created_at,
            updated_at: response.user.updated_at,
        })
    }

    pub async fn get_oauth_authorization_url(
        &self,
        provider: &str,
    ) -> Result<OAuthAuthUrlResponse> {
        self.get(&format!("/api/auth/oauth/{}/authorize", provider))
            .await
    }

    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<User> {
        let request = RegisterRequest {
            username,
            email,
            password,
        };
        let user: User = self.post("/api/auth/register", &request).await?;

        // Backend returns user directly, no need to extract token separately
        Ok(user)
    }

    // Learner endpoints
    pub async fn create_learner(&self, display_name: Option<String>) -> Result<Learner> {
        let request = serde_json::json!({
            "display_name": display_name,
        });
        self.post("/api/learners", &request).await
    }

    pub async fn get_learner(&self, learner_id: &str) -> Result<Learner> {
        self.get(&format!("/api/learners/{}", learner_id)).await
    }

    // Session endpoints
    pub async fn create_session(
        &self,
        learner_id: String,
        topology_type: String,
        topology_data: Option<serde_json::Value>,
    ) -> Result<Session> {
        let request = CreateSessionRequest {
            learner_id,
            topology_type,
            topology_data,
        };
        self.post("/api/sessions", &request).await
    }

    pub async fn end_session(&self, session_id: &str) -> Result<()> {
        self.post::<_, ()>(
            &format!("/api/sessions/{}/complete", session_id),
            &serde_json::json!({}),
        )
        .await
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Session> {
        self.get(&format!("/api/sessions/{}", session_id)).await
    }

    // Response endpoints
    pub async fn submit_response(
        &self,
        session_id: &str,
        response: SubmitResponseRequest,
    ) -> Result<()> {
        self.post(
            &format!("/api/sessions/{}/responses", session_id),
            &response,
        )
        .await
    }

    pub async fn get_session_responses(&self, session_id: &str) -> Result<Vec<serde_json::Value>> {
        self.get(&format!("/api/sessions/{}/responses", session_id))
            .await
    }

    // Analytics endpoints
    pub async fn get_learner_performance(&self, learner_id: &str) -> Result<PerformanceMetrics> {
        // For now, return default metrics since we're using mock API
        Ok(PerformanceMetrics::default())
    }

    // Export endpoints
    pub async fn export_learner_data(&self, learner_id: &str) -> Result<ExportData> {
        let learner = self.get_learner(learner_id).await?;
        let sessions: Vec<Session> = self
            .get(&format!("/api/learners/{}/sessions", learner_id))
            .await?;
        let metrics = PerformanceMetrics::default();

        Ok(ExportData {
            learner,
            sessions,
            metrics,
            export_time: chrono::Utc::now(),
            demo_seed: None,
        })
    }
}

// Mock API client for testing without backend
pub struct MockApiClient;

impl MockApiClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn login(&self, username: String, _password: String) -> Result<User> {
        Ok(User {
            id: Uuid::new_v4().to_string(),
            username,
            email: "test@example.com".to_string(),
            password_hash: String::new(),
            apple_user_id: None,
            github_user_id: None,
            oauth_provider_id: None,
            auth_provider: "local".to_string(),
            is_private_email: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    pub async fn create_learner(&self, display_name: Option<String>) -> Result<serde_json::Value> {
        // Return a simplified learner for mock API
        // The actual Learner with core_model will be created in the app
        Ok(serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "user_id": Uuid::new_v4().to_string(),
            "display_name": display_name,
            "created_at": chrono::Utc::now(),
        }))
    }

    pub async fn create_session(
        &self,
        learner_id: String,
        topology_type: String,
        _topology_data: Option<serde_json::Value>,
    ) -> Result<Session> {
        Ok(Session {
            id: Uuid::new_v4().to_string(),
            learner_id,
            topology_type,
            topology: None,
            start_time: chrono::Utc::now(),
            end_time: None,
            status: "active".to_string(),
            summary: None,
            responses: Vec::new(),
        })
    }
}
