use chrono::Utc;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

use abcdeez_core::{
    hints::{HintLevel, InterventionAction, InterventionSystem, StruggleLevel},
    prelude::*,
    tasks::TaskResponse as CoreTaskResponse,
    AdaptiveScheduler, LearnerMetrics, TaskGenerator, TaskSession,
};

use crate::api::{ApiClientTrait, AdaptiveApiClient};
use crate::core::models::*;
use crate::core::services::*;
use crate::core::config::{AppConfig, ConfigManager};
use crate::storage::{ConnectivityMonitor, OfflineStorage, SyncStatus};
use crate::research::*;
use crate::utils::LittleCrab;

#[cfg(target_os = "ios")]
use crate::integrations::ios_auth;

// Application screens
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Screen {
    Welcome,
    DomainSelection,
    Training,
    Dashboard,
    Settings,
    ResearchDashboard,
    Visualizations,
    WidgetGallery,
}

#[derive(Debug, Clone, Default)]
pub struct IRBFormData {
    pub study_title: String,
    pub principal_investigator: String,
    pub institution: String,
    pub study_purpose: String,
    pub participant_population: String,
    pub data_collection_methods: String,
    pub risks: String,
    pub benefits: String,
    pub procedures: String,
}

// Main application state
pub struct AppData {
    // UI State
    pub current_screen: Screen,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub show_end_session_confirmation: bool,

    // Authentication
    pub current_user: Option<User>,
    pub is_guest_mode: bool, // Proper guest mode flag
    pub username_input: String,
    pub password_input: String,
    pub email_input: String,

    // OAuth Authentication
    #[cfg(target_os = "ios")]
    pub ios_auth_bridge: Option<ios_auth::IOSAuthBridge>,
    pub oauth_login_in_flight: bool,

    // Async request states
    pub login_request_in_flight: bool,
    pub create_session_in_flight: bool,
    pub submit_response_in_flight: bool,
    pub end_session_in_flight: bool,
    pub export_data_in_flight: bool,

    // Core library integration
    pub current_learner: Option<Learner>,
    pub current_session: Option<Session>,
    pub selected_domain: Domain,
    pub topology: Option<Topology>,
    pub adaptive_scheduler: Option<AdaptiveScheduler>,
    pub task_session: Option<TaskSession>,
    pub task_generator: Option<TaskGenerator>,

    // Training state
    pub current_task: Option<UITask>,
    pub task_start_time: Option<Instant>,
    pub selected_answer: Option<String>,
    pub selected_answer_index: Option<usize>,
    pub show_feedback: bool,
    pub last_response_correct: bool,
    pub current_hint: Option<String>,
    pub hint_level: HintLevel,

    // Intervention system
    pub intervention_system: Option<InterventionSystem>,
    pub struggle_level: StruggleLevel,

    // Performance metrics
    pub current_metrics: PerformanceMetrics,
    pub session_responses: Vec<CoreTaskResponse>,

    // Settings
    pub use_adaptive_scheduling: bool,
    pub enable_hints: bool,
    pub difficulty_level: f64,

    // Export
    pub export_data: Option<ExportData>,
    pub export_format: ExportFormat,

    // Configuration management
    pub config_manager: ConfigManager,

    // API client with fallback capability
    pub api_client: Arc<AdaptiveApiClient>,

    // Enhanced demo controller
    pub demo_controller: DemoController,
    pub demo_seed: Option<u64>,
    pub demo_rng: Option<StdRng>,

    // Demo preferences
    pub demo_auto_advance: bool,
    pub demo_show_tooltips: bool,
    pub demo_highlight_elements: bool,
    pub demo_speed: f64,
    pub preferred_demo: String,

    // Display preferences
    pub show_advanced_metrics: bool,
    pub show_response_times: bool,
    pub enable_animations: bool,
    pub anonymous_export: bool,

    // Offline support
    pub offline_storage: Option<Arc<OfflineStorage>>,
    pub sync_status: SyncStatus,
    pub connectivity_monitor: Arc<ConnectivityMonitor>,
    pub offline_mode: bool,
    pub auto_sync_enabled: bool,

    // API Configuration UI state
    pub api_url_input: String,
    pub api_timeout_input: String,
    pub api_fallback_enabled: bool,
    pub show_api_settings: bool,
    pub api_connection_status: String,

    // Runtime for async operations
    pub runtime: Arc<tokio::runtime::Runtime>,

    // Easter egg: The little crab
    pub little_crab: Option<LittleCrab>,
    pub crab_trigger_clicks: usize,
    pub last_click_time: Option<std::time::Instant>,

    // Research mode
    pub research_controller: Option<ResearchController>,
    pub show_experiment_setup: bool,
    pub selected_experiment_type: Option<ExperimentType>,
    pub experiment_control_group: bool,
    pub research_data_collection_enabled: bool,
    pub research_privacy_mode: bool,
    // IRB compliance UI state
    pub show_irb_form: bool,
    pub show_irb_documents: bool,
    pub irb_form_data: IRBFormData,
    // Federation state
    pub federation_network: Option<federation::FederationNetwork>,
    pub show_federation_setup: bool,
    pub show_protocol_browser: bool,
    pub show_study_coordination: bool,
    pub federation_status: federation::ComplianceStatus,

    // Protocol Versioning state
    pub protocol_versions: Vec<ProtocolVersion>,
    pub current_protocol_version: Option<String>,
    pub selected_protocol_version: Option<String>,
    pub protocol_version_diff: Option<ProtocolVersionDiff>,
    pub show_protocol_version_history: bool,
    pub show_protocol_comparison: bool,
    pub show_protocol_editor: bool,
    pub protocol_loading: bool,
    pub protocol_creation_in_flight: bool,
    pub protocol_update_in_flight: bool,

    // Protocol Editor Form State
    pub protocol_editor_name: String,
    pub protocol_editor_description: String,
    pub protocol_editor_design_type: String,
    pub protocol_editor_target_n: String,
    pub protocol_editor_power: String,
    pub protocol_editor_effect_size: String,
    pub protocol_editor_alpha: String,
    pub protocol_editor_duration_weeks: String,
    pub protocol_editor_sessions_per_participant: String,
    pub protocol_editor_data_retention_years: String,
    pub protocol_editor_version_message: String,

    // PWA integration (wasm only)
    #[cfg(target_arch = "wasm32")]
    pub pwa_initialized: bool,
}

impl Default for AppData {
    fn default() -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        Self {
            current_screen: Screen::Welcome,
            error_message: None,
            success_message: None,
            show_end_session_confirmation: false,
            current_user: None,
            is_guest_mode: false,
            username_input: String::new(),
            password_input: String::new(),
            email_input: String::new(),

            // OAuth Authentication
            #[cfg(target_os = "ios")]
            ios_auth_bridge: None,
            oauth_login_in_flight: false,
            current_learner: None,
            current_session: None,
            selected_domain: Domain::Alphabet,
            topology: None,
            adaptive_scheduler: None,
            task_session: None,
            task_generator: None,
            current_task: None,
            task_start_time: None,
            selected_answer: None,
            selected_answer_index: None,
            show_feedback: false,
            last_response_correct: false,
            current_hint: None,
            hint_level: HintLevel::Confirmation,
            intervention_system: None,
            struggle_level: StruggleLevel::None,
            current_metrics: PerformanceMetrics::default(),
            session_responses: Vec::new(),
            use_adaptive_scheduling: true,
            enable_hints: true,
            difficulty_level: 0.5,
            export_data: None,
            export_format: ExportFormat::Json,
            config_manager: ConfigManager::new().unwrap_or_else(|e| {
                tracing::warn!("Failed to initialize config manager: {}", e);
                // Create a fallback with default config
                ConfigManager::with_config_file("./config.toml")
                    .unwrap_or_else(|_| panic!("Failed to create fallback config manager"))
            }),
            api_client: {
                let config_manager = ConfigManager::new().unwrap_or_else(|e| {
                    tracing::warn!("Failed to initialize config for API client: {}", e);
                    ConfigManager::with_config_file("./config.toml").unwrap()
                });
                Arc::new(AdaptiveApiClient::new(config_manager.config()))
            },

            // Async request states
            login_request_in_flight: false,
            create_session_in_flight: false,
            submit_response_in_flight: false,
            end_session_in_flight: false,
            export_data_in_flight: false,

            // Enhanced demo controller
            demo_controller: DemoController::new(),
            demo_seed: None,
            demo_rng: None,

            // Demo preferences
            demo_auto_advance: true,
            demo_show_tooltips: true,
            demo_highlight_elements: true,
            demo_speed: 1.0,
            preferred_demo: "quick_tour".to_string(),

            // Display preferences
            show_advanced_metrics: false,
            show_response_times: true,
            enable_animations: true,
            anonymous_export: false,

            // Offline support
            offline_storage: None,
            sync_status: SyncStatus {
                online: true,
                last_sync: None,
                pending_count: 0,
                failed_count: 0,
                sync_progress: None,
            },
            connectivity_monitor: Arc::new(ConnectivityMonitor::new()),
            offline_mode: false,
            auto_sync_enabled: true,

            // API Configuration UI state (initialize from config)
            api_url_input: {
                let config_manager_temp = ConfigManager::new()
                    .unwrap_or_else(|_| ConfigManager::with_config_file("./config.toml").unwrap());
                config_manager_temp.config().api_url().to_string()
            },
            api_timeout_input: {
                let config_manager_temp = ConfigManager::new()
                    .unwrap_or_else(|_| ConfigManager::with_config_file("./config.toml").unwrap());
                config_manager_temp.config().api.timeout_seconds.to_string()
            },
            api_fallback_enabled: {
                let config_manager_temp = ConfigManager::new()
                    .unwrap_or_else(|_| ConfigManager::with_config_file("./config.toml").unwrap());
                config_manager_temp.config().should_fallback_to_mock()
            },
            show_api_settings: false,
            api_connection_status: "Not tested".to_string(),

            // Runtime for async operations
            runtime: Arc::new(runtime),

            // Easter egg: The little crab
            little_crab: Some(crate::utils::easter_egg::init_random_crab()),
            crab_trigger_clicks: 0,
            last_click_time: None,

            // Research mode
            research_controller: None,
            show_experiment_setup: false,
            selected_experiment_type: None,
            experiment_control_group: false,
            research_data_collection_enabled: true,
            research_privacy_mode: false,
            // IRB compliance UI state
            show_irb_form: false,
            show_irb_documents: false,
            irb_form_data: IRBFormData::default(),
            // Federation state
            federation_network: None,
            show_federation_setup: false,
            show_protocol_browser: false,
            show_study_coordination: false,
            federation_status: crate::research::federation::ComplianceStatus::NonCompliant,

            // Protocol Versioning state
            protocol_versions: Vec::new(),
            current_protocol_version: None,
            selected_protocol_version: None,
            protocol_version_diff: None,
            show_protocol_version_history: false,
            show_protocol_comparison: false,
            show_protocol_editor: false,
            protocol_loading: false,
            protocol_creation_in_flight: false,
            protocol_update_in_flight: false,

            // Protocol Editor Form State
            protocol_editor_name: "Learning Protocol".to_string(),
            protocol_editor_description: "A protocol for studying learning efficiency".to_string(),
            protocol_editor_design_type: "between-subjects".to_string(),
            protocol_editor_target_n: "120".to_string(),
            protocol_editor_power: "0.8".to_string(),
            protocol_editor_effect_size: "0.5".to_string(),
            protocol_editor_alpha: "0.05".to_string(),
            protocol_editor_duration_weeks: "8".to_string(),
            protocol_editor_sessions_per_participant: "5".to_string(),
            protocol_editor_data_retention_years: "7".to_string(),
            protocol_editor_version_message: "Updated protocol version".to_string(),

            #[cfg(target_arch = "wasm32")]
            pwa_initialized: false,
        }
    }
}