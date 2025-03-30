use thiserror::Error;

/// Central error type for the msft-recon-rs application.
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ReconError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Network request error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("DNS resolution error: {0}")]
    Dns(#[from] trust_dns_resolver::error::ResolveError),

    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("CLI argument error: {0}")]
    CliArgs(String), // For custom CLI validation errors

    #[error("Reconnaissance check failed: {check_name} - {details}")]
    CheckFailed { check_name: String, details: String },

    #[error("Resource not found: {resource_type} for {target}")]
    NotFound { resource_type: String, target: String },

    #[error("Unexpected API response: {service} - Status: {status}, Body: {body:.100}...")] // Truncate long bodies
    UnexpectedApiResponse {
        service: String,
        status: reqwest::StatusCode,
        body: String,
    },

    #[error("Missing required data: {0}")]
    MissingData(String), 

    #[error("Other error: {0}")]
    Other(String), // General catch-all for miscellaneous errors

    // Add more specific error variants as needed during implementation.
}

// Optional: Implement helper functions if needed, e.g., for creating specific errors.
impl ReconError {
    pub fn check_failed(check_name: &str, details: impl Into<String>) -> Self {
        Self::CheckFailed {
            check_name: check_name.to_string(),
            details: details.into(),
        }
    }

    #[allow(dead_code)]
    pub fn not_found(resource_type: &str, target: &str) -> Self {
        Self::NotFound {
            resource_type: resource_type.to_string(),
            target: target.to_string(),
        }
    }
    
    pub fn cli_error(message: impl Into<String>) -> Self {
        Self::CliArgs(message.into())
    }
}