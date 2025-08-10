// Configuration management for the adaptive learning system
// Supports environment variables, config files, and GUI settings

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub ui: UiConfig,
    pub demo: DemoConfig,
    pub data: DataConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub fallback_to_mock: bool,
    pub auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_advanced_metrics: bool,
    pub show_response_times: bool,
    pub enable_animations: bool,
    pub demo_auto_advance: bool,
    pub demo_show_tooltips: bool,
    pub demo_highlight_elements: bool,
    pub demo_speed_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoConfig {
    pub preferred_scenario: String,
    pub auto_advance: bool,
    pub show_tooltips: bool,
    pub highlight_elements: bool,
    pub speed_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    pub anonymous_export: bool,
    pub auto_sync: bool,
    pub local_storage_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            ui: UiConfig::default(),
            demo: DemoConfig::default(),
            data: DataConfig::default(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://abcdeez.fg-goose.online/api/v1".to_string(),
            timeout_seconds: 30,
            retry_attempts: 3,
            fallback_to_mock: true,
            auth_token: None,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_advanced_metrics: true,
            show_response_times: true,
            enable_animations: true,
            demo_auto_advance: false,
            demo_show_tooltips: true,
            demo_highlight_elements: true,
            demo_speed_multiplier: 1.0,
        }
    }
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            preferred_scenario: "QuickTour".to_string(),
            auto_advance: false,
            show_tooltips: true,
            highlight_elements: true,
            speed_multiplier: 1.0,
        }
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            anonymous_export: false,
            auto_sync: true,
            local_storage_path: None,
        }
    }
}

pub struct ConfigManager {
    config: AppConfig,
    config_file_path: std::path::PathBuf,
}

impl ConfigManager {
    /// Create a new ConfigManager with default config file location
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_file_path = config_dir.join("abcdeez.toml");

        let mut manager = Self {
            config: AppConfig::default(),
            config_file_path,
        };

        manager.load()?;
        Ok(manager)
    }

    /// Create ConfigManager with custom config file path
    pub fn with_config_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut manager = Self {
            config: AppConfig::default(),
            config_file_path: path.as_ref().to_path_buf(),
        };

        manager.load()?;
        Ok(manager)
    }

    /// Load configuration from multiple sources in priority order:
    /// 1. Environment variables (highest priority)
    /// 2. Config file
    /// 3. Defaults (lowest priority)
    pub fn load(&mut self) -> Result<()> {
        // Start with defaults
        self.config = AppConfig::default();

        // Override with config file if it exists
        if self.config_file_path.exists() {
            self.load_from_file()?;
        }

        // Override with environment variables (highest priority)
        self.load_from_env();

        Ok(())
    }

    /// Load configuration from TOML file
    fn load_from_file(&mut self) -> Result<()> {
        let content = std::fs::read_to_string(&self.config_file_path)?;
        self.config = toml::from_str(&content)?;
        Ok(())
    }

    /// Load configuration from environment variables
    fn load_from_env(&mut self) {
        // API Configuration
        if let Ok(url) = std::env::var("ABCDEEZ_API_URL") {
            self.config.api.base_url = url;
        }

        if let Ok(timeout) = std::env::var("ABCDEEZ_API_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                self.config.api.timeout_seconds = timeout;
            }
        }

        if let Ok(retries) = std::env::var("ABCDEEZ_API_RETRIES") {
            if let Ok(retries) = retries.parse() {
                self.config.api.retry_attempts = retries;
            }
        }

        if let Ok(fallback) = std::env::var("ABCDEEZ_API_FALLBACK_TO_MOCK") {
            self.config.api.fallback_to_mock = fallback.parse().unwrap_or(true);
        }

        if let Ok(token) = std::env::var("ABCDEEZ_API_TOKEN") {
            self.config.api.auth_token = Some(token);
        }

        // UI Configuration
        if let Ok(advanced) = std::env::var("ABCDEEZ_UI_SHOW_ADVANCED") {
            self.config.ui.show_advanced_metrics = advanced.parse().unwrap_or(true);
        }

        if let Ok(animations) = std::env::var("ABCDEEZ_UI_ENABLE_ANIMATIONS") {
            self.config.ui.enable_animations = animations.parse().unwrap_or(true);
        }

        // Data Configuration
        if let Ok(anonymous) = std::env::var("ABCDEEZ_DATA_ANONYMOUS") {
            self.config.data.anonymous_export = anonymous.parse().unwrap_or(false);
        }

        if let Ok(storage_path) = std::env::var("ABCDEEZ_DATA_STORAGE_PATH") {
            self.config.data.local_storage_path = Some(storage_path);
        }
    }

    /// Save current configuration to file
    pub fn save(&self) -> Result<()> {
        // Ensure config directory exists
        if let Some(parent) = self.config_file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_file_path, content)?;

        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get mutable reference to configuration for GUI updates
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// Update configuration and save to file
    pub fn update_config<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        updater(&mut self.config);
        self.save()
    }

    /// Get platform-appropriate config directory
    fn get_config_dir() -> Result<std::path::PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let app_data = std::env::var("APPDATA")
                .map_err(|_| anyhow::anyhow!("Could not find APPDATA directory"))?;
            Ok(std::path::PathBuf::from(app_data).join("abcdeez"))
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME")
                .map_err(|_| anyhow::anyhow!("Could not find HOME directory"))?;
            Ok(std::path::PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("abcdeez"))
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
                Ok(std::path::PathBuf::from(xdg_config).join("abcdeez"))
            } else {
                let home = std::env::var("HOME")
                    .map_err(|_| anyhow::anyhow!("Could not find HOME directory"))?;
                Ok(std::path::PathBuf::from(home)
                    .join(".config")
                    .join("abcdeez"))
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            // Fallback for other platforms
            Ok(std::path::PathBuf::from(".").join("config"))
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate API URL
        if self.config.api.base_url.is_empty() {
            return Err(anyhow::anyhow!("API base URL cannot be empty"));
        }

        // Try to parse as URL
        url::Url::parse(&self.config.api.base_url)
            .map_err(|e| anyhow::anyhow!("Invalid API URL: {}", e))?;

        // Validate timeout
        if self.config.api.timeout_seconds == 0 {
            return Err(anyhow::anyhow!("API timeout must be greater than 0"));
        }

        // Validate speed multiplier
        if self.config.demo.speed_multiplier <= 0.0 {
            return Err(anyhow::anyhow!("Demo speed multiplier must be positive"));
        }

        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset_to_defaults(&mut self) -> Result<()> {
        self.config = AppConfig::default();
        self.save()
    }

    /// Export configuration as JSON for debugging
    pub fn export_as_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.config)?)
    }
}

// Convenience functions for common configuration access patterns
impl AppConfig {
    pub fn api_url(&self) -> &str {
        &self.api.base_url
    }

    pub fn should_fallback_to_mock(&self) -> bool {
        self.api.fallback_to_mock
    }

    pub fn api_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.api.timeout_seconds)
    }

    pub fn retry_attempts(&self) -> u32 {
        self.api.retry_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(
            config.api.base_url,
            "https://abcdeez.fg-goose.online/api/v1"
        );
        assert_eq!(config.api.timeout_seconds, 30);
        assert_eq!(config.api.retry_attempts, 3);
        assert!(config.api.fallback_to_mock);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.api.base_url, deserialized.api.base_url);
        assert_eq!(config.api.timeout_seconds, deserialized.api.timeout_seconds);
    }

    #[test]
    fn test_config_file_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path();

        // Create manager with custom path
        let mut manager = ConfigManager::with_config_file(config_path).unwrap();

        // Modify config
        manager.config_mut().api.base_url = "https://custom.example.com/api/v1".to_string();

        // Save and reload
        manager.save().unwrap();
        manager.load().unwrap();

        assert_eq!(
            manager.config().api.base_url,
            "https://custom.example.com/api/v1"
        );
    }

    #[test]
    fn test_env_var_override() {
        std::env::set_var("ABCDEEZ_API_URL", "https://env.example.com/api/v1");
        std::env::set_var("ABCDEEZ_API_TIMEOUT", "60");
        std::env::set_var("ABCDEEZ_API_FALLBACK_TO_MOCK", "false");

        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = ConfigManager::with_config_file(temp_file.path()).unwrap();

        assert_eq!(
            manager.config().api.base_url,
            "https://env.example.com/api/v1"
        );
        assert_eq!(manager.config().api.timeout_seconds, 60);
        assert!(!manager.config().api.fallback_to_mock);

        // Clean up
        std::env::remove_var("ABCDEEZ_API_URL");
        std::env::remove_var("ABCDEEZ_API_TIMEOUT");
        std::env::remove_var("ABCDEEZ_API_FALLBACK_TO_MOCK");
    }

    #[test]
    fn test_validation() {
        let mut config = AppConfig::default();
        let manager = ConfigManager::with_config_file("/tmp/test").unwrap();

        // Valid config should pass
        assert!(manager.validate().is_ok());

        // Invalid URL should fail
        config.api.base_url = "not-a-url".to_string();
        let manager = ConfigManager {
            config,
            config_file_path: "/tmp/test".into(),
        };
        assert!(manager.validate().is_err());
    }
}
