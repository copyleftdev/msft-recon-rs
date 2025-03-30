use crate::config::CloudConfig;
use crate::error::ReconError;
use crate::models::{DnsResults, M365Results};
use reqwest::Client;
use tracing::{debug, info, warn};

/// Performs M365 service checks.
/// This might be called after initial DNS/Tenant info is gathered.
pub async fn run_m365_checks(
    client: Client, // Pass cloned client
    domain: String, // Pass owned domain
    cloud_config: CloudConfig, // Pass cloned config
    dns_results: Option<DnsResults>, // Pass owned/cloned Option<DnsResults>
) -> Result<M365Results, ReconError> {
    info!(target = domain, "Running M365 service checks...");

    // Spawn tasks for independent checks
    let client_clone1 = client.clone();
    let domain_clone1 = domain.clone();
    let config_clone1 = cloud_config.clone();
    let sharepoint_handle = tokio::spawn(async move {
        check_sharepoint(client_clone1, domain_clone1, config_clone1).await
    });
    
    let client_clone2 = client.clone();
    let domain_clone2 = domain.clone();
    let config_clone2 = cloud_config.clone();
    let branding_handle = tokio::spawn(async move {
        check_tenant_branding(client_clone2, domain_clone2, config_clone2).await
    });
    
    let client_clone3 = client.clone();
    let domain_clone3 = domain.clone();
    let config_clone3 = cloud_config.clone();
    let ews_handle = tokio::spawn(async move {
        check_legacy_auth_ews(client_clone3, domain_clone3, config_clone3).await
    });
    
    let client_clone4 = client.clone();
    let domain_clone4 = domain.clone();
    let config_clone4 = cloud_config.clone();
    let activesync_handle = tokio::spawn(async move {
        check_legacy_auth_activesync(client_clone4, domain_clone4, config_clone4).await
    });
    // Add handles for power apps etc. later

    // Await results
    let sharepoint_detected = sharepoint_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "SharePoint check task failed");
        Err(ReconError::check_failed("SharePoint Check", e.to_string()))
    })?;
    let branding_accessible = branding_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "Tenant Branding check task failed");
        Err(ReconError::check_failed("Tenant Branding Check", e.to_string()))
    })?;
    let ews_enabled = ews_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "Legacy Auth (EWS) check task failed");
        Err(ReconError::check_failed("Legacy Auth (EWS) Check", e.to_string()))
    })?;
    let activesync_enabled = activesync_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "Legacy Auth (ActiveSync) check task failed");
        Err(ReconError::check_failed("Legacy Auth (ActiveSync) Check", e.to_string()))
    })?;

    // Combine results
    let results = M365Results {
        sharepoint_detected: Some(sharepoint_detected),
        // Determine Teams presence based on DNS results (already gathered)
        teams_detected: Some(check_teams_dns(dns_results.as_ref())),
        tenant_branding_accessible: Some(branding_accessible),
        legacy_auth_ews_enabled: Some(ews_enabled),
        legacy_auth_activesync_enabled: Some(activesync_enabled),
    };

    info!(target = domain.as_str(), "Finished M365 service checks");
    Ok(results)
}

/// Helper function to extract tenant name from domain
fn get_tenant_name(domain: &str) -> String {
    // Usually tenant name is the first part of the domain
    // (e.g., "contoso" from "contoso.com")
    domain.split('.').next().unwrap_or(domain).to_string()
}

/// Check if SharePoint Online is accessible for the domain
async fn check_sharepoint(
    client: Client,
    domain: String,
    config: CloudConfig,
) -> Result<bool, ReconError> {
    let tenant_name = get_tenant_name(&domain);
    let url = format!("https://{}{}", tenant_name, config.sharepoint_host_suffix);
    debug!(domain = domain.as_str(), url = url.as_str(), "Checking SharePoint Online");
    
    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status();
            debug!(domain = domain.as_str(), status = %status, "SharePoint response");
            // SharePoint returns 200 or 302 if tenant exists, even if not authenticated
            Ok(status.is_success() || status.is_redirection())
        }
        Err(e) => {
            // Connection error could be firewall block or no such tenant
            debug!(domain = domain.as_str(), error = %e, "SharePoint request failed");
            Ok(false) // Consider this as "not detected" rather than error
        }
    }
}

/// Check if tenant branding is accessible
async fn check_tenant_branding(
    client: Client, 
    domain: String, 
    config: CloudConfig
) -> Result<bool, ReconError> {
    let tenant_name = get_tenant_name(&domain);
    let url = format!("https://{}{}/common/branding/favicon.ico", 
                    config.login_microsoftonline_host,
                    tenant_name);
    
    debug!(domain = domain.as_str(), url = url.as_str(), "Checking tenant branding");
    
    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status();
            debug!(domain = domain.as_str(), status = %status, "Branding response");
            Ok(status.is_success())
        }
        Err(e) => {
            debug!(domain = domain.as_str(), error = %e, "Branding request failed");
            Ok(false) // Consider this as "not accessible" rather than error
        }
    }
}

/// Check if legacy auth (EWS) is allowed
async fn check_legacy_auth_ews(
    client: Client,
    domain: String,
    config: CloudConfig,
) -> Result<bool, ReconError> {
    // This is a simplified check - real validation would need actual EWS API calls
    let url = format!("https://{}{}/EWS/Exchange.asmx", 
                      domain, 
                      config.ews_endpoint_host);
    
    debug!(domain = domain.as_str(), url = url.as_str(), "Checking legacy auth (EWS)");
    
    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status();
            debug!(domain = domain.as_str(), status = %status, "EWS response");
            // 401 Unauthorized is expected for EWS endpoint if it exists
            // 200 OK would be unusual but indicates it exists
            Ok(status.is_client_error() || status.is_success())
        }
        Err(e) => {
            debug!(domain = domain.as_str(), error = %e, "EWS request failed");
            Ok(false) // Consider this as "not enabled" rather than error
        }
    }
}

/// Check if legacy auth (ActiveSync) is allowed
async fn check_legacy_auth_activesync(
    client: Client,
    domain: String,
    config: CloudConfig,
) -> Result<bool, ReconError> {
    let url = format!("https://{}{}/Microsoft-Server-ActiveSync", 
                     domain, 
                     config.activesync_endpoint_host);
                     
    debug!(domain = domain.as_str(), url = url.as_str(), "Checking legacy auth (ActiveSync)");
    
    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status();
            debug!(domain = domain.as_str(), status = %status, "ActiveSync response");
            // 401 Unauthorized is expected for ActiveSync endpoint if it exists
            // 403 Forbidden might indicate it exists but is blocked
            // 200 OK would be unusual but indicates it exists
            Ok(status.is_client_error() || status.is_success())
        }
        Err(e) => {
            debug!(domain = domain.as_str(), error = %e, "ActiveSync request failed");
            Ok(false) // Consider this as "not enabled" rather than error
        }
    }
}

/// Checks for Teams presence using DNS records.
fn check_teams_dns(dns_results: Option<&DnsResults>) -> bool {
    dns_results.as_ref().map_or(false, |dns| {
        dns.lyncdiscover_present.unwrap_or(false) || dns.sip_cname_or_a_present.unwrap_or(false)
    })
}

// TODO: Implement check_power_apps