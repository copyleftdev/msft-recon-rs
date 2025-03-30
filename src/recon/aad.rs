use crate::config::CloudConfig;
use crate::error::ReconError;
use crate::models::{AadConnectStatus, AzureAdConfig};
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, info, warn};
use url::Url;

// Temporary struct to deserialize only the fields we need from OpenID config
#[derive(Debug, Deserialize)]
struct OpenIdConfigResponse {
    issuer: Option<String>,
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
    jwks_uri: Option<String>,
    tenant_region_scope: Option<String>,
    // We can ignore other fields
}

/// Fetches the Azure AD OpenID configuration.
pub async fn get_azure_ad_config(
    client: Client, // Pass cloned client
    _domain: String, // Keep domain for potential future use, mark unused for now
    config: CloudConfig, // Pass cloned config
) -> Result<AzureAdConfig, ReconError> {
    // Construct the full OpenID configuration URL
    let base_url = Url::parse(&config.login_endpoint)?;
    let config_url = base_url.join(&config.openid_config_endpoint)?;

    debug!(target = _domain, url = config_url.as_str(), "Querying OpenID Config");

    let response = client.get(config_url.clone()).send().await?;

    if !response.status().is_success() {
        warn!(target = _domain, status = %response.status(), url = config_url.as_str(), "OpenID Config request failed");
        return Err(ReconError::UnexpectedApiResponse {
            service: "OpenID Configuration".to_string(),
            status: response.status(),
            body: response.text().await.unwrap_or_else(|_| "<failed to read body>".to_string()),
        });
    }

    let config_data: OpenIdConfigResponse = response.json().await?;
    debug!(target = _domain, "OpenID Config response parsed successfully");

    // Map the deserialized fields to our AzureAdConfig model
    Ok(AzureAdConfig {
        issuer: config_data.issuer,
        authorization_endpoint: config_data.authorization_endpoint,
        token_endpoint: config_data.token_endpoint,
        jwks_uri: config_data.jwks_uri,
        tenant_region_scope: config_data.tenant_region_scope,
    })
}

/// Checks the Azure AD Connect status by probing the Seamless SSO endpoint.
///
/// Infers Hybrid status if the endpoint is reachable, CloudOnly otherwise.
pub async fn check_aad_connect_status(
    client: Client, // Pass cloned client
    domain: String, // Pass owned domain
    config: CloudConfig, // Pass cloned config
) -> Result<AadConnectStatus, ReconError> {
    let url = &config.azure_ad_connect_check_url;

    // If URL is empty in config (e.g., for CN cloud), assume Unknown or CloudOnly?
    if url.is_empty() {
        warn!(target = domain, "AAD Connect check URL is not configured for this cloud. Returning Unknown.");
        return Ok(AadConnectStatus::Unknown);
    }

    debug!(target = domain, url = url.as_str(), "Checking AAD Connect status (SSO endpoint)");

    match client.get(url).send().await {
        Ok(response) => {
            // Consider any successful response (2xx) or redirect (3xx) as an indication
            // that *something* related to AAD Connect/SSO is configured.
            if response.status().is_success() || response.status().is_redirection() {
                info!(target = domain, status = %response.status(), "AAD Connect check endpoint reachable. Inferring Hybrid.");
                Ok(AadConnectStatus::Hybrid)
            } else {
                // Unexpected HTTP status code (4xx, 5xx)
                warn!(target = domain, status = %response.status(), "AAD Connect check endpoint returned non-success/redirect status. Inferring CloudOnly/Unknown.");
                // Let's lean towards CloudOnly if we get a specific error response, but could be Unknown.
                Ok(AadConnectStatus::CloudOnly) 
            }
        }
        Err(e) => {
            // Network errors (timeout, connection refused, DNS resolution failure for the SSO domain)
            // strongly suggest that Seamless SSO is not in use.
            if e.is_timeout() || e.is_connect() || e.is_request() {
                info!(target = domain, error = %e, "AAD Connect check endpoint unreachable. Inferring CloudOnly.");
                Ok(AadConnectStatus::CloudOnly)
            } else {
                // Other errors (e.g., TLS issues, proxy errors) are ambiguous.
                 warn!(target = domain, error = %e, "AAD Connect check failed with unexpected network error. Returning Unknown.");
                Ok(AadConnectStatus::Unknown)
                 // Or propagate the error?
                 // Err(ReconError::Network(e))
            }
        }
    }
}