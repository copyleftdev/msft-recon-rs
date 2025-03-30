use crate::config::AppConfig;
use crate::error::ReconError;
use reqwest::{Client, header};
use std::time::Duration;

/// Creates a new shared reqwest HTTP client instance.
///
/// Configures the client with a timeout and a default user agent
/// based on the application configuration.
pub fn new_client(config: &AppConfig) -> Result<Client, ReconError> {
    // Default to 30 seconds if not specified in config
    let timeout = Duration::from_secs(config.request_timeout_seconds.unwrap_or(30));
    let user_agent = &config.default_user_agent;

    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_str(user_agent)
        .map_err(|e| ReconError::Config(config::ConfigError::Foreign(Box::new(e))))? // Convert header error to ConfigError
    );

    let client = Client::builder()
        .timeout(timeout)
        .default_headers(headers)
        // TODO: Configure TLS settings if necessary (e.g., accept invalid certs - use with caution!)
        // .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| ReconError::Config(config::ConfigError::Foreign(Box::new(e))))?; // Convert reqwest client error to ConfigError

    Ok(client)
}