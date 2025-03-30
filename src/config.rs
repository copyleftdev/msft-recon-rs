use crate::cli::CloudTarget; // Import the new Cli and CloudTarget
use crate::error::ReconError;
use config::{Config, File, FileFormat};
use serde::Deserialize;
use std::time::Duration;

/// Represents the configuration settings for a specific cloud environment.
#[derive(Debug, Clone, Deserialize)] // Clone is useful for passing relevant parts to tasks
pub struct CloudConfig {
    pub login_endpoint: String,
    pub login_microsoftonline_host: String,
    pub user_realm_endpoint: String,
    pub openid_config_endpoint: String, // Path relative to login_endpoint
    pub azure_ad_connect_check_url: String,
    pub sharepoint_host_suffix: String,
    pub cdn_host_suffix: String,
    pub ews_endpoint_host: String, // Exchange Web Services endpoint
    pub activesync_endpoint_host: String, // ActiveSync endpoint
    pub app_service_host_suffix: String, // For Azure App Services (.azurewebsites.net)
    pub storage_account_host_suffix: String, // For Azure Storage (.blob.core.windows.net)
    // Add other endpoint URLs as needed based on default.toml and checks
    // pub graph_endpoint: String, 
    // pub autodiscover_endpoint: String,
    // ... etc
}

/// Represents the overall application configuration.
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub clouds: Clouds,
    pub request_timeout_seconds: Option<u64>,
    pub default_user_agent: String,
}

/// Container for different cloud environment configurations.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Clouds {
    pub commercial: CloudConfig,
    pub gov: CloudConfig,
    pub cn: CloudConfig,
}

/// Loads the application configuration from files.
///
/// It merges configuration from `config/default.toml` and potentially
/// environment-specific files or environment variables.
pub fn load_config() -> Result<AppConfig, ReconError> {
    let settings = Config::builder()
        // Start with default values from config/default.toml
        .add_source(File::new("config/default", FileFormat::Toml))
        // TODO: Add environment-specific overrides if needed (e.g., config/production.toml)
        // .add_source(File::new("config/production", FileFormat::Toml).required(false))
        // TODO: Add environment variable overrides if needed (e.g., APP_PORT=8000)
        // .add_source(config::Environment::with_prefix("APP"))
        .build()?;

    settings.try_deserialize().map_err(ReconError::Config)
}

/// Selects the appropriate CloudConfig based on CLI arguments.
/// Returns a Result in case the specified cloud environment is not valid.
pub fn select_cloud_config<'a>(app_config: &'a AppConfig, cloud_target: &CloudTarget) -> Result<&'a CloudConfig, ReconError> {
    match cloud_target {
        CloudTarget::Commercial => Ok(&app_config.clouds.commercial),
        CloudTarget::Gcc | CloudTarget::GccHigh => Ok(&app_config.clouds.gov),
        CloudTarget::Dod => Ok(&app_config.clouds.gov), // DoD is also part of the US government cloud
    }
}

#[allow(dead_code)]
/// Gets the configured request timeout as a Duration.
pub fn get_timeout_duration(app_config: &AppConfig) -> Duration {
    app_config.request_timeout_seconds.map_or(Duration::from_secs(0), |s| Duration::from_secs(s))
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from the parent module (config)

    // Helper to ensure config/default.toml exists for tests
    // Note: This assumes the test is run from the project root
    fn ensure_config_file_exists() {
        if !std::path::Path::new("config/default.toml").exists() {
            panic!("config/default.toml not found. Ensure tests run from project root.");
        }
    }

    #[test]
    fn test_load_config_commercial() {
        ensure_config_file_exists();
        let config = load_config().expect("Failed to load config");
        assert_eq!(config.clouds.commercial.login_endpoint, "https://login.microsoftonline.com");
        assert_eq!(config.clouds.commercial.sharepoint_host_suffix, ".sharepoint.com");
        assert_eq!(config.clouds.commercial.cdn_host_suffix, ".azureedge.net");
    }

    #[test]
    fn test_load_config_gov() {
        ensure_config_file_exists();
        let config = load_config().expect("Failed to load config");
        assert_eq!(config.clouds.gov.login_endpoint, "https://login.microsoftonline.us");
        assert_eq!(config.clouds.gov.sharepoint_host_suffix, ".sharepoint.us");
        assert_eq!(config.clouds.gov.cdn_host_suffix, ".azureedge.us");
    }

    #[test]
    fn test_load_config_china() {
        ensure_config_file_exists();
        let config = load_config().expect("Failed to load config");
        assert_eq!(config.clouds.cn.login_endpoint, "https://login.partner.microsoftonline.cn");
        assert_eq!(config.clouds.cn.sharepoint_host_suffix, ".sharepoint.cn");
        assert_eq!(config.clouds.cn.cdn_host_suffix, ".azureedge.cn");
    }
}