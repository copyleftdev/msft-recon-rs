use crate::config::CloudConfig;
use crate::error::ReconError;
use crate::models::AzureServiceResults;
use reqwest::Client;
use tracing::{debug, info, warn};

/// Performs Azure service checks.
pub async fn run_azure_service_checks(
    client: Client, // Pass cloned client
    domain: String, // Pass owned domain
    cloud_config: CloudConfig, // Pass cloned config
) -> Result<AzureServiceResults, ReconError> {
    info!(target = domain, "Starting Azure service checks");

    // Spawn tasks for independent checks
    let app_service_handle = tokio::spawn(check_app_services(
        client.clone(), // Clone for the task
        domain.clone(),
        cloud_config.clone(),
    ));
    let storage_handle = tokio::spawn(check_storage_account(
        client.clone(), // Clone for the task
        domain.clone(),
        cloud_config.clone(),
    ));
    let cdn_handle = tokio::spawn(check_cdn(
        client.clone(), // Clone for the task
        domain.clone(),
        cloud_config.clone(),
    ));

    // Await results
    let app_service_url = app_service_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "App Service check task failed");
        Err(ReconError::check_failed("App Service Check", e.to_string()))
    })?;
    let storage_url = storage_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "Storage Account check task failed");
        Err(ReconError::check_failed("Storage Account Check", e.to_string()))
    })?;
    let cdn_url = cdn_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "CDN check task failed");
        Err(ReconError::check_failed("CDN Check", e.to_string()))
    })?;

    // Combine results
    let results = AzureServiceResults {
        probable_app_services: app_service_url.map_or_else(Vec::new, |url| vec![url]),
        probable_storage_accounts: storage_url.map_or_else(Vec::new, |url| vec![url]),
        probable_cdn_endpoints: cdn_url.map_or_else(Vec::new, |url| vec![url]),
    };

    info!(target = domain.as_str(), "Finished Azure service checks");
    Ok(results)
}

/// Checks for the presence of Azure App Services.
///
/// Constructs the expected App Service URL (domain.azurewebsites.net) and probes it.
/// Returns Option<String> containing the URL if found, None otherwise.
async fn check_app_services(
    client: Client, // Expects owned client for task
    domain: String,
    cloud_config: CloudConfig,
) -> Result<Option<String>, ReconError> {
    // Construct the App Service URL (e.g., contoso.azurewebsites.net)
    // Use the primary domain name directly.
    let domain_prefix = domain.split('.').next().unwrap_or(&domain);
    let app_service_host = format!(
        "{}{}",
        domain_prefix,
        cloud_config.app_service_host_suffix // .azurewebsites.net, etc.
    );
    let url = format!("https://{}", app_service_host);

    debug!(target = domain.as_str(), url = url.as_str(), "Checking App Service URL");

    match client.get(&url).send().await {
        Ok(response) => {
            // Similar to SharePoint, any successful connection often indicates presence,
            // even if it results in a non-2xx status (e.g., 404, auth prompt).
            // The key is that the DNS name resolved and the service responded.
            info!(target = domain.as_str(), status = %response.status(), url = url.as_str(), "App Service check successful (implies presence)");
            Ok(Some(url))
        }
        Err(e) => {
            // Network errors (DNS resolution failure, connection refused)
            // strongly indicate the App Service name is *not* in use.
            if e.is_connect() || e.is_request() {
                info!(target = domain.as_str(), error = %e, "App Service check failed (implies absence)");
                Ok(None)
            } else {
                // Other errors (timeout, TLS) are less conclusive.
                warn!(target = domain.as_str(), error = %e, "App Service check inconclusive due to network error");
                Ok(None) // Treat inconclusive errors as absence for now
                // Err(ReconError::Network(e)) // Alternative: Propagate
            }
        }
    }
}

/// Checks for the presence of Azure Storage Accounts.
///
/// Constructs potential storage account URLs and probes them.
/// Returns Option<String> containing the URL if found, None otherwise.
async fn check_storage_account(
    client: Client,
    domain: String,
    cloud_config: CloudConfig,
) -> Result<Option<String>, ReconError> {
    // For storage accounts, we try common naming patterns based on the organization name:
    // 1. The simple domain name (e.g., "contoso" for contoso.com)
    // 2. The domain name with "storage" suffix (e.g., "contosostorage")
    // 3. The domain name with "data" suffix (e.g., "contosodata")
    
    let domain_prefix = domain.split('.').next().unwrap_or(&domain);
    let potential_names = vec![
        domain_prefix.to_string(),
        format!("{}storage", domain_prefix),
        format!("{}data", domain_prefix),
    ];
    
    for name in &potential_names {
        // Blob storage is the most common endpoint to check
        let storage_host = format!(
            "{}{}",
            name,
            cloud_config.storage_account_host_suffix
        );
        let url = format!("https://{}", storage_host);
        
        debug!(target = domain.as_str(), url = url.as_str(), "Checking Storage Account URL");
        
        match client.get(&url).send().await {
            Ok(response) => {
                // Storage accounts typically respond with 400 (Bad Request) if the account exists
                // but no container/blob is specified, or 404 if it's a valid name but no such account.
                // Status 400 is a strong positive indicator.
                let status = response.status();
                if status.is_client_error() || status.is_success() {
                    info!(target = domain.as_str(), status = %status, url = url.as_str(), "Storage Account check successful (implies presence)");
                    return Ok(Some(url));
                }
            },
            Err(e) => {
                if e.is_connect() || e.is_request() {
                    debug!(target = domain.as_str(), error = %e, "Storage Account check failed for {}", name);
                } else {
                    warn!(target = domain.as_str(), error = %e, "Storage Account check inconclusive for {}", name);
                }
                // Continue to next potential name
            }
        }
    }
    
    Ok(None)
}

/// Checks for the presence of Azure CDN endpoints.
///
/// Constructs the expected CDN URL (e.g., domain.azureedge.net) and probes it.
/// Returns Option<String> containing the URL if found, None otherwise.
async fn check_cdn(
    client: Client, // Expects owned client for task
    domain: String,
    cloud_config: CloudConfig,
) -> Result<Option<String>, ReconError> {
    // Construct the CDN URL (e.g., contoso.azureedge.net)
    let domain_prefix = domain.split('.').next().unwrap_or(&domain);
    let cdn_host_suffix = cloud_config.cdn_host_suffix;
    if cdn_host_suffix.is_empty() { // Check if suffix is configured
        debug!(target = domain, "CDN check skipped: no suffix in config");
        return Ok(None);
    }

    let cdn_host = format!("{}{}", domain_prefix, cdn_host_suffix);
    let url = format!("https://{}", cdn_host);
    debug!(target = domain, url = url.as_str(), "Probing potential CDN endpoint");

    // We just need to see if we get *any* response, status doesn't matter as much
    match client.get(&url).send().await { // Use the owned client
        Ok(_) => {
            info!(target = domain, url = url.as_str(), "CDN endpoint found");
            Ok(Some(url))
        }
        Err(e) => {
            // Network errors (DNS resolution failure, connection refused)
            // strongly indicate the CDN endpoint name is *not* in use.
            if e.is_connect() || e.is_request() {
                info!(target = domain, error = %e, "CDN check failed (implies absence)");
                Ok(None)
            } else {
                // Other errors (timeout, TLS) are less conclusive.
                warn!(target = domain, error = %e, "CDN check inconclusive due to network error");
                Ok(None) // Treat inconclusive errors as absence for now
            }
        }
    }
}