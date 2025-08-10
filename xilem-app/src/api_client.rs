use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, span, warn, Level};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::core::models::*;

/// Trait for API client operations - allows both real and mock implementations
#[async_trait::async_trait]
pub trait ApiClientTrait: Send + Sync {
    async fn login(&self, username: String, password: String) -> Result<User>;
    async fn get_user_info(&self) -> Result<User>;
    async fn create_learner(&self, display_name: Option<String>) -> Result<serde_json::Value>;
    async fn get_learner(&self, learner_id: &str) -> Result<serde_json::Value>;
    async fn get_learner_sessions(&self, learner_id: &str) -> Result<Vec<serde_json::Value>>;
    async fn create_session(&self, request: CreateSessionRequest) -> Result<Session>;
    async fn get_session(&self, session_id: &str) -> Result<Session>;
    async fn complete_session(&self, session_id: &str) -> Result<()>;
    async fn submit_response(
        &self,
        session_id: &str,
        request: SubmitResponseRequest,
    ) -> Result<SubmitResponseResponse>;
    async fn get_session_responses(&self, session_id: &str) -> Result<Vec<serde_json::Value>>;
    async fn get_learner_performance(&self, learner_id: &str) -> Result<PerformanceData>;
    async fn health_check(&self) -> Result<()>;

    // Protocol Versioning API methods
    async fn get_protocol_versions(&self, experiment_id: &str) -> Result<Vec<ProtocolVersion>>;
    async fn get_protocol_version(
        &self,
        experiment_id: &str,
        version: &str,
    ) -> Result<ProtocolVersion>;
    async fn create_protocol_version(
        &self,
        experiment_id: &str,
        content: ProtocolContent,
    ) -> Result<ProtocolVersion>;
    async fn update_protocol_version(
        &self,
        experiment_id: &str,
        version_id: &str,
        content: ProtocolContent,
    ) -> Result<ProtocolVersion>;
    async fn compare_protocol_versions(
        &self,
        experiment_id: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<ProtocolVersionDiff>;
    async fn set_current_protocol_version(
        &self,
        experiment_id: &str,
        version_id: &str,
    ) -> Result<()>;
}

/// Real API client that connects to the backend server
#[derive(Debug)]
pub struct RealApiClient {
    client: Client,
    base_url: String,
    auth_token: Arc<RwLock<Option<String>>>,
    retry_attempts: u32,
}

impl RealApiClient {
    pub fn new(config: &AppConfig) -> Self {
        let client = Client::builder()
            .timeout(config.api_timeout())
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.api_url().to_string(),
            auth_token: Arc::new(RwLock::new(config.api.auth_token.clone())),
            retry_attempts: config.retry_attempts(),
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

    async fn post_with_retry<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<R> {
        let mut last_error = None;

        for attempt in 1..=self.retry_attempts {
            match self.post(path, body).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!(
                        "API request attempt {}/{} failed: {}",
                        attempt, self.retry_attempts, e
                    );
                    last_error = Some(e);

                    if attempt < self.retry_attempts {
                        // Exponential backoff
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    async fn get_with_retry<R: DeserializeOwned>(&self, path: &str) -> Result<R> {
        let mut last_error = None;

        for attempt in 1..=self.retry_attempts {
            match self.get(path).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!(
                        "API request attempt {}/{} failed: {}",
                        attempt, self.retry_attempts, e
                    );
                    last_error = Some(e);

                    if attempt < self.retry_attempts {
                        // Exponential backoff
                        let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    async fn post<T: Serialize, R: DeserializeOwned>(&self, path: &str, body: &T) -> Result<R> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.post(&url).json(body);

        if let Some(auth_header) = self.get_auth_header().await {
            request = request.header("Authorization", auth_header);
        }

        debug!("POST {}", url);
        let response = request.send().await?;

        if response.status().is_success() {
            let json = response.json().await?;
            debug!("POST {} succeeded", url);
            Ok(json)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("POST {} failed with status {}: {}", url, status, body);
            Err(anyhow::anyhow!("API request failed: {} - {}", status, body))
        }
    }

    async fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.get(&url);

        if let Some(auth_header) = self.get_auth_header().await {
            request = request.header("Authorization", auth_header);
        }

        debug!("GET {}", url);
        let response = request.send().await?;

        if response.status().is_success() {
            let json = response.json().await?;
            debug!("GET {} succeeded", url);
            Ok(json)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("GET {} failed with status {}: {}", url, status, body);
            Err(anyhow::anyhow!("API request failed: {} - {}", status, body))
        }
    }
}

#[async_trait::async_trait]
impl ApiClientTrait for RealApiClient {
    #[instrument(level = "info", fields(username = %username), skip(password))]
    async fn login(&self, username: String, password: String) -> Result<User> {
        info!(username = %username, "Attempting user login");

        let request = LoginRequest {
            username: username.clone(),
            password,
        };
        let response: TokenResponse = self
            .post_with_retry("/api/auth/login", &request)
            .await
            .map_err(|e| {
                warn!(username = %username, error = %e, "Login request failed");
                e
            })?;

        // Store the token
        self.set_auth_token(response.access_token).await;
        debug!(username = %username, "Authentication token stored");

        // Get user info
        let user = self.get_user_info().await?;
        info!(
            username = %username,
            user_id = %user.id,
            "Login completed successfully"
        );
        Ok(user)
    }

    async fn get_user_info(&self) -> Result<User> {
        self.get_with_retry("/api/auth/me").await
    }

    async fn create_learner(&self, display_name: Option<String>) -> Result<serde_json::Value> {
        let request = CreateLearnerRequest { display_name };
        self.post_with_retry("/api/learners", &request).await
    }

    async fn get_learner(&self, learner_id: &str) -> Result<serde_json::Value> {
        let path = format!("/api/learners/{}", learner_id);
        self.get_with_retry(&path).await
    }

    async fn get_learner_sessions(&self, learner_id: &str) -> Result<Vec<serde_json::Value>> {
        let path = format!("/api/learners/{}/sessions", learner_id);
        self.get_with_retry(&path).await
    }

    async fn create_session(&self, request: CreateSessionRequest) -> Result<Session> {
        self.post_with_retry("/api/sessions", &request).await
    }

    async fn get_session(&self, session_id: &str) -> Result<Session> {
        let path = format!("/api/sessions/{}", session_id);
        self.get_with_retry(&path).await
    }

    async fn complete_session(&self, session_id: &str) -> Result<()> {
        let path = format!("/api/sessions/{}/complete", session_id);
        let _: serde_json::Value = self.post_with_retry(&path, &serde_json::json!({})).await?;
        Ok(())
    }

    async fn submit_response(
        &self,
        session_id: &str,
        request: SubmitResponseRequest,
    ) -> Result<SubmitResponseResponse> {
        let path = format!("/api/sessions/{}/responses", session_id);
        self.post_with_retry(&path, &request).await
    }

    async fn get_session_responses(&self, session_id: &str) -> Result<Vec<serde_json::Value>> {
        let path = format!("/api/sessions/{}/responses", session_id);
        self.get_with_retry(&path).await
    }

    async fn get_learner_performance(&self, learner_id: &str) -> Result<PerformanceData> {
        let path = format!("/api/learners/{}/stats", learner_id);
        self.get_with_retry(&path).await
    }

    async fn health_check(&self) -> Result<()> {
        let _: serde_json::Value = self.get_with_retry("/api/health").await?;
        Ok(())
    }

    // Protocol Versioning API implementations
    async fn get_protocol_versions(&self, experiment_id: &str) -> Result<Vec<ProtocolVersion>> {
        self.get_with_retry(&format!(
            "/api/experiments/{}/protocol-versions",
            experiment_id
        ))
        .await
    }

    async fn get_protocol_version(
        &self,
        experiment_id: &str,
        version: &str,
    ) -> Result<ProtocolVersion> {
        self.get_with_retry(&format!(
            "/api/experiments/{}/protocol-versions/{}",
            experiment_id, version
        ))
        .await
    }

    async fn create_protocol_version(
        &self,
        experiment_id: &str,
        content: ProtocolContent,
    ) -> Result<ProtocolVersion> {
        let request = serde_json::json!({
            "content": content,
            "message": "Created via UI"
        });
        self.post_with_retry(
            &format!("/api/experiments/{}/protocol-versions", experiment_id),
            &request,
        )
        .await
    }

    async fn update_protocol_version(
        &self,
        experiment_id: &str,
        version_id: &str,
        content: ProtocolContent,
    ) -> Result<ProtocolVersion> {
        let request = serde_json::json!({
            "content": content,
            "message": "Updated via UI"
        });
        self.post_with_retry(
            &format!(
                "/api/experiments/{}/protocol-versions/{}",
                experiment_id, version_id
            ),
            &request,
        )
        .await
    }

    async fn compare_protocol_versions(
        &self,
        experiment_id: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<ProtocolVersionDiff> {
        self.get_with_retry(&format!(
            "/api/experiments/{}/protocol-versions/{}/compare/{}",
            experiment_id, from_version, to_version
        ))
        .await
    }

    async fn set_current_protocol_version(
        &self,
        experiment_id: &str,
        version_id: &str,
    ) -> Result<()> {
        let request = serde_json::json!({
            "version_id": version_id
        });
        let _: serde_json::Value = self
            .post_with_retry(
                &format!(
                    "/api/experiments/{}/protocol-versions/set-current",
                    experiment_id
                ),
                &request,
            )
            .await?;
        Ok(())
    }
}

/// Mock API client for testing and offline mode
pub struct MockApiClient {
    user_id: String,
}

impl MockApiClient {
    pub fn new() -> Self {
        Self {
            user_id: Uuid::new_v4().to_string(),
        }
    }
}

#[async_trait::async_trait]
impl ApiClientTrait for MockApiClient {
    async fn login(&self, username: String, _password: String) -> Result<User> {
        info!("Mock login for user: {}", username);
        Ok(User {
            id: self.user_id.clone(),
            username,
            email: "test@example.com".to_string(),
            password_hash: String::new(),
            apple_user_id: None,
            github_user_id: None,
            oauth_provider_id: None,
            auth_provider: "mock".to_string(),
            is_private_email: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    async fn get_user_info(&self) -> Result<User> {
        self.login("mock_user".to_string(), "".to_string()).await
    }

    async fn create_learner(&self, display_name: Option<String>) -> Result<serde_json::Value> {
        info!("Mock creating learner: {:?}", display_name);
        Ok(serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "user_id": self.user_id,
            "display_name": display_name,
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }))
    }

    async fn get_learner(&self, learner_id: &str) -> Result<serde_json::Value> {
        info!("Mock getting learner: {}", learner_id);
        Ok(serde_json::json!({
            "id": learner_id,
            "user_id": self.user_id,
            "display_name": "Mock Learner",
            "created_at": chrono::Utc::now(),
            "updated_at": chrono::Utc::now(),
        }))
    }

    async fn get_learner_sessions(&self, learner_id: &str) -> Result<Vec<serde_json::Value>> {
        info!("Mock getting sessions for learner: {}", learner_id);
        Ok(vec![serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "learner_id": learner_id,
            "status": "completed",
            "created_at": chrono::Utc::now(),
        })])
    }

    async fn create_session(&self, request: CreateSessionRequest) -> Result<Session> {
        info!("Mock creating session for learner: {}", request.learner_id);
        Ok(Session {
            id: Uuid::new_v4().to_string(),
            learner_id: request.learner_id,
            topology_type: "default".to_string(),
            topology: None, // Will be populated by UI
            start_time: chrono::Utc::now(),
            end_time: None,
            status: "active".to_string(),
            summary: None,
            responses: Vec::new(),
        })
    }

    async fn get_session(&self, session_id: &str) -> Result<Session> {
        info!("Mock getting session: {}", session_id);
        Ok(Session {
            id: session_id.to_string(),
            learner_id: self.user_id.clone(),
            topology_type: "default".to_string(),
            topology: None,
            start_time: chrono::Utc::now(),
            end_time: None,
            status: "active".to_string(),
            summary: None,
            responses: Vec::new(),
        })
    }

    async fn complete_session(&self, session_id: &str) -> Result<()> {
        info!("Mock completing session: {}", session_id);
        Ok(())
    }

    async fn submit_response(
        &self,
        session_id: &str,
        request: SubmitResponseRequest,
    ) -> Result<SubmitResponseResponse> {
        info!("Mock submitting response for session: {}", session_id);
        Ok(SubmitResponseResponse {
            response_id: Uuid::new_v4().to_string(),
            sequence_number: 1,
            correct: request.user_answer == "mock_correct_answer",
            intervention: None,
            next_task: None,
        })
    }

    async fn get_session_responses(&self, session_id: &str) -> Result<Vec<serde_json::Value>> {
        info!("Mock getting responses for session: {}", session_id);
        Ok(vec![serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "session_id": session_id,
            "task_type": "PairwiseOrder",
            "correct": true,
            "response_time_ms": 1500,
            "created_at": chrono::Utc::now(),
        })])
    }

    async fn get_learner_performance(&self, learner_id: &str) -> Result<PerformanceData> {
        info!("Mock getting performance for learner: {}", learner_id);
        Ok(PerformanceData {
            overall_accuracy: 0.85,
            average_response_time: 1200.0,
            total_sessions: 5,
            total_tasks: 100,
            learning_metrics: serde_json::json!({
                "bidirectionality_index": 0.1,
                "distance_effect_slope": 0.05,
                "chunk_penalty": 0.15,
            }),
        })
    }

    async fn health_check(&self) -> Result<()> {
        info!("Mock health check - always healthy!");
        Ok(())
    }

    // Protocol Versioning mock implementations
    async fn get_protocol_versions(&self, experiment_id: &str) -> Result<Vec<ProtocolVersion>> {
        info!(
            "Mock getting protocol versions for experiment: {}",
            experiment_id
        );
        use chrono::Utc;
        Ok(vec![ProtocolVersion {
            id: "version-1".to_string(),
            experiment_id: experiment_id.to_string(),
            version: "1.0.0".to_string(),
            commit_hash: "abc123def456".to_string(),
            author: "Mock Researcher".to_string(),
            message: "Initial protocol version".to_string(),
            created_at: Utc::now(),
            protocol_content: ProtocolContent {
                name: "Mock Protocol".to_string(),
                description: "A mock protocol for testing".to_string(),
                experiment_design: ExperimentDesignConfig {
                    design_type: "between-subjects".to_string(),
                    conditions: vec![
                        ExperimentCondition {
                            name: "Control".to_string(),
                            description: "Control condition".to_string(),
                            parameters: std::collections::HashMap::new(),
                            is_control: true,
                        },
                        ExperimentCondition {
                            name: "Treatment".to_string(),
                            description: "Treatment condition".to_string(),
                            parameters: std::collections::HashMap::new(),
                            is_control: false,
                        },
                    ],
                    randomization: RandomizationConfig {
                        method: "simple".to_string(),
                        seed: Some(12345),
                        stratification: vec![],
                        block_size: Some(4),
                    },
                    sample_size: SampleSizeConfig {
                        target_n: 100,
                        power: 0.8,
                        effect_size: 0.5,
                        alpha: 0.05,
                        justification: "Power analysis based on pilot study".to_string(),
                    },
                },
                data_collection_plan: DataCollectionPlan {
                    duration_weeks: 12,
                    sessions_per_participant: 3,
                    data_types: vec!["response_time".to_string(), "accuracy".to_string()],
                    quality_criteria: vec!["response_time > 100ms".to_string()],
                },
                analysis_plan: AnalysisPlan {
                    primary_analyses: vec!["t-test".to_string()],
                    secondary_analyses: vec!["correlation".to_string()],
                    statistical_tests: vec!["independent_t_test".to_string()],
                    multiple_comparison_correction: Some("bonferroni".to_string()),
                },
                compliance_requirements: ComplianceRequirements {
                    irb_required: true,
                    consent_required: true,
                    data_retention_years: 5,
                    privacy_level: "high".to_string(),
                },
            },
            is_current: true,
            is_draft: false,
        }])
    }

    async fn get_protocol_version(
        &self,
        experiment_id: &str,
        version: &str,
    ) -> Result<ProtocolVersion> {
        info!(
            "Mock getting protocol version {} for experiment: {}",
            version, experiment_id
        );
        let versions = self.get_protocol_versions(experiment_id).await?;
        versions
            .into_iter()
            .find(|v| v.version == version)
            .ok_or_else(|| anyhow::anyhow!("Version not found: {}", version))
    }

    async fn create_protocol_version(
        &self,
        experiment_id: &str,
        content: ProtocolContent,
    ) -> Result<ProtocolVersion> {
        info!(
            "Mock creating protocol version for experiment: {}",
            experiment_id
        );
        use chrono::Utc;
        Ok(ProtocolVersion {
            id: format!("version-{}", uuid::Uuid::new_v4()),
            experiment_id: experiment_id.to_string(),
            version: "1.1.0".to_string(),
            commit_hash: "def456abc789".to_string(),
            author: "Mock User".to_string(),
            message: "Created via UI".to_string(),
            created_at: Utc::now(),
            protocol_content: content,
            is_current: false,
            is_draft: true,
        })
    }

    async fn update_protocol_version(
        &self,
        experiment_id: &str,
        version_id: &str,
        content: ProtocolContent,
    ) -> Result<ProtocolVersion> {
        info!(
            "Mock updating protocol version {} for experiment: {}",
            version_id, experiment_id
        );
        use chrono::Utc;
        Ok(ProtocolVersion {
            id: version_id.to_string(),
            experiment_id: experiment_id.to_string(),
            version: "1.1.1".to_string(),
            commit_hash: "ghi789jkl012".to_string(),
            author: "Mock User".to_string(),
            message: "Updated via UI".to_string(),
            created_at: Utc::now(),
            protocol_content: content,
            is_current: false,
            is_draft: true,
        })
    }

    async fn compare_protocol_versions(
        &self,
        experiment_id: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<ProtocolVersionDiff> {
        info!(
            "Mock comparing protocol versions {} -> {} for experiment: {}",
            from_version, to_version, experiment_id
        );
        Ok(ProtocolVersionDiff {
            from_version: from_version.to_string(),
            to_version: to_version.to_string(),
            changes: vec![ProtocolChange {
                field: "sample_size.target_n".to_string(),
                change_type: ChangeType::Modified,
                old_value: Some(serde_json::json!(80)),
                new_value: Some(serde_json::json!(100)),
                description: "Increased target sample size".to_string(),
                impact_level: ImpactLevel::Minor,
            }],
            summary: "Minor changes to sample size configuration".to_string(),
            compatibility: CompatibilityStatus::Compatible,
        })
    }

    async fn set_current_protocol_version(
        &self,
        experiment_id: &str,
        version_id: &str,
    ) -> Result<()> {
        info!(
            "Mock setting current protocol version {} for experiment: {}",
            version_id, experiment_id
        );
        Ok(())
    }
}

/// Adaptive API client that tries real API first, falls back to mock if configured
pub struct AdaptiveApiClient {
    real_client: RealApiClient,
    mock_client: MockApiClient,
    fallback_to_mock: bool,
    is_real_available: Arc<RwLock<bool>>,
}

impl AdaptiveApiClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            real_client: RealApiClient::new(config),
            mock_client: MockApiClient::new(),
            fallback_to_mock: config.should_fallback_to_mock(),
            is_real_available: Arc::new(RwLock::new(true)),
        }
    }

    /// Check if real API is available
    pub async fn check_real_api_availability(&self) -> bool {
        match self.real_client.health_check().await {
            Ok(()) => {
                let mut available = self.is_real_available.write().await;
                *available = true;
                info!("Real API is available");
                true
            }
            Err(e) => {
                let mut available = self.is_real_available.write().await;
                *available = false;
                warn!("Real API is not available: {}", e);
                false
            }
        }
    }

    /// Execute operation with fallback logic
    async fn execute_with_fallback<F, R>(&self, real_op: F) -> Result<R>
    where
        F: std::future::Future<Output = Result<R>>,
        R: Send,
    {
        // Always try real API first
        match real_op.await {
            Ok(result) => {
                // Success - mark API as available
                let mut available = self.is_real_available.write().await;
                *available = true;
                Ok(result)
            }
            Err(e) => {
                // Failed - mark API as unavailable
                let mut available = self.is_real_available.write().await;
                *available = false;

                if self.fallback_to_mock {
                    warn!("Real API failed, falling back to mock: {}", e);
                    // Note: We can't easily fallback here without duplicating logic
                    // The caller should check is_real_available and use mock directly
                    Err(e)
                } else {
                    error!("Real API failed and fallback disabled: {}", e);
                    Err(e)
                }
            }
        }
    }

    pub async fn is_using_real_api(&self) -> bool {
        *self.is_real_available.read().await
    }

    pub async fn set_auth_token(&self, token: String) {
        self.real_client.set_auth_token(token).await;
    }

    pub async fn clear_auth_token(&self) {
        self.real_client.clear_auth_token().await;
    }
}

#[async_trait::async_trait]
impl ApiClientTrait for AdaptiveApiClient {
    async fn login(&self, username: String, password: String) -> Result<User> {
        match self
            .real_client
            .login(username.clone(), password.clone())
            .await
        {
            Ok(user) => {
                let mut available = self.is_real_available.write().await;
                *available = true;
                Ok(user)
            }
            Err(e) => {
                let mut available = self.is_real_available.write().await;
                *available = false;

                if self.fallback_to_mock {
                    warn!("Real login failed, using mock: {}", e);
                    self.mock_client.login(username, password).await
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn get_user_info(&self) -> Result<User> {
        if *self.is_real_available.read().await {
            match self.real_client.get_user_info().await {
                Ok(user) => Ok(user),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real get_user_info failed, using mock: {}", e);
                        self.mock_client.get_user_info().await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.get_user_info().await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn create_learner(&self, display_name: Option<String>) -> Result<serde_json::Value> {
        if *self.is_real_available.read().await {
            match self.real_client.create_learner(display_name.clone()).await {
                Ok(learner) => Ok(learner),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real create_learner failed, using mock: {}", e);
                        self.mock_client.create_learner(display_name).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.create_learner(display_name).await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn get_learner(&self, learner_id: &str) -> Result<serde_json::Value> {
        if *self.is_real_available.read().await {
            match self.real_client.get_learner(learner_id).await {
                Ok(learner) => Ok(learner),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real get_learner failed, using mock: {}", e);
                        self.mock_client.get_learner(learner_id).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.get_learner(learner_id).await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn get_learner_sessions(&self, learner_id: &str) -> Result<Vec<serde_json::Value>> {
        if *self.is_real_available.read().await {
            match self.real_client.get_learner_sessions(learner_id).await {
                Ok(sessions) => Ok(sessions),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real get_learner_sessions failed, using mock: {}", e);
                        self.mock_client.get_learner_sessions(learner_id).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.get_learner_sessions(learner_id).await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn create_session(&self, request: CreateSessionRequest) -> Result<Session> {
        if *self.is_real_available.read().await {
            match self.real_client.create_session(request.clone()).await {
                Ok(session) => Ok(session),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real create_session failed, using mock: {}", e);
                        self.mock_client.create_session(request).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.create_session(request).await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn get_session(&self, session_id: &str) -> Result<Session> {
        if *self.is_real_available.read().await {
            match self.real_client.get_session(session_id).await {
                Ok(session) => Ok(session),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real get_session failed, using mock: {}", e);
                        self.mock_client.get_session(session_id).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.get_session(session_id).await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn complete_session(&self, session_id: &str) -> Result<()> {
        if *self.is_real_available.read().await {
            match self.real_client.complete_session(session_id).await {
                Ok(()) => Ok(()),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real complete_session failed, using mock: {}", e);
                        self.mock_client.complete_session(session_id).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.complete_session(session_id).await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn submit_response(
        &self,
        session_id: &str,
        request: SubmitResponseRequest,
    ) -> Result<SubmitResponseResponse> {
        if *self.is_real_available.read().await {
            match self
                .real_client
                .submit_response(session_id, request.clone())
                .await
            {
                Ok(response) => Ok(response),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real submit_response failed, using mock: {}", e);
                        self.mock_client.submit_response(session_id, request).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.submit_response(session_id, request).await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn get_session_responses(&self, session_id: &str) -> Result<Vec<serde_json::Value>> {
        if *self.is_real_available.read().await {
            match self.real_client.get_session_responses(session_id).await {
                Ok(responses) => Ok(responses),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real get_session_responses failed, using mock: {}", e);
                        self.mock_client.get_session_responses(session_id).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.get_session_responses(session_id).await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn get_learner_performance(&self, learner_id: &str) -> Result<PerformanceData> {
        if *self.is_real_available.read().await {
            match self.real_client.get_learner_performance(learner_id).await {
                Ok(performance) => Ok(performance),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real get_learner_performance failed, using mock: {}", e);
                        self.mock_client.get_learner_performance(learner_id).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.get_learner_performance(learner_id).await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn health_check(&self) -> Result<()> {
        self.real_client.health_check().await
    }

    // Protocol Versioning implementations with fallback
    async fn get_protocol_versions(&self, experiment_id: &str) -> Result<Vec<ProtocolVersion>> {
        if *self.is_real_available.read().await {
            match self.real_client.get_protocol_versions(experiment_id).await {
                Ok(versions) => Ok(versions),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real get_protocol_versions failed, using mock: {}", e);
                        self.mock_client.get_protocol_versions(experiment_id).await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client.get_protocol_versions(experiment_id).await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn get_protocol_version(
        &self,
        experiment_id: &str,
        version: &str,
    ) -> Result<ProtocolVersion> {
        if *self.is_real_available.read().await {
            match self
                .real_client
                .get_protocol_version(experiment_id, version)
                .await
            {
                Ok(protocol_version) => Ok(protocol_version),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real get_protocol_version failed, using mock: {}", e);
                        self.mock_client
                            .get_protocol_version(experiment_id, version)
                            .await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client
                .get_protocol_version(experiment_id, version)
                .await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn create_protocol_version(
        &self,
        experiment_id: &str,
        content: ProtocolContent,
    ) -> Result<ProtocolVersion> {
        if *self.is_real_available.read().await {
            match self
                .real_client
                .create_protocol_version(experiment_id, content.clone())
                .await
            {
                Ok(version) => Ok(version),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real create_protocol_version failed, using mock: {}", e);
                        self.mock_client
                            .create_protocol_version(experiment_id, content)
                            .await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client
                .create_protocol_version(experiment_id, content)
                .await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn update_protocol_version(
        &self,
        experiment_id: &str,
        version_id: &str,
        content: ProtocolContent,
    ) -> Result<ProtocolVersion> {
        if *self.is_real_available.read().await {
            match self
                .real_client
                .update_protocol_version(experiment_id, version_id, content.clone())
                .await
            {
                Ok(version) => Ok(version),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real update_protocol_version failed, using mock: {}", e);
                        self.mock_client
                            .update_protocol_version(experiment_id, version_id, content)
                            .await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client
                .update_protocol_version(experiment_id, version_id, content)
                .await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn compare_protocol_versions(
        &self,
        experiment_id: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<ProtocolVersionDiff> {
        if *self.is_real_available.read().await {
            match self
                .real_client
                .compare_protocol_versions(experiment_id, from_version, to_version)
                .await
            {
                Ok(diff) => Ok(diff),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!("Real compare_protocol_versions failed, using mock: {}", e);
                        self.mock_client
                            .compare_protocol_versions(experiment_id, from_version, to_version)
                            .await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client
                .compare_protocol_versions(experiment_id, from_version, to_version)
                .await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }

    async fn set_current_protocol_version(
        &self,
        experiment_id: &str,
        version_id: &str,
    ) -> Result<()> {
        if *self.is_real_available.read().await {
            match self
                .real_client
                .set_current_protocol_version(experiment_id, version_id)
                .await
            {
                Ok(()) => Ok(()),
                Err(e) => {
                    if self.fallback_to_mock {
                        warn!(
                            "Real set_current_protocol_version failed, using mock: {}",
                            e
                        );
                        self.mock_client
                            .set_current_protocol_version(experiment_id, version_id)
                            .await
                    } else {
                        Err(e)
                    }
                }
            }
        } else if self.fallback_to_mock {
            self.mock_client
                .set_current_protocol_version(experiment_id, version_id)
                .await
        } else {
            Err(anyhow::anyhow!(
                "Real API unavailable and fallback disabled"
            ))
        }
    }
}
