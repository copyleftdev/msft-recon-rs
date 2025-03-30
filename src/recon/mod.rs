// This file declares the submodules within the `recon` module and the main orchestrator.

pub mod aad;
pub mod azure_svc;
pub mod client;
pub mod dns;
pub mod m365;
pub mod mdi;
pub mod tenant;

use reqwest::Client;
use tracing::{error, info, warn}; // Import tracing macros

use crate::config::CloudConfig;
use crate::error::ReconError;
use crate::models::ReconResults;

// Import check functions from submodules
use aad::{check_aad_connect_status, get_azure_ad_config};
use azure_svc::run_azure_service_checks;
use dns::run_dns_checks;
use m365::run_m365_checks;
use tenant::get_federation_info;

/// Orchestrates all reconnaissance checks.
///
/// Runs checks sequentially or concurrently where appropriate,
/// collecting results into the provided `ReconResults` struct.
/// Errors from individual checks are logged, but do not stop the overall process.
pub async fn run_all_checks(
    client: Client,
    domain: String, // Accept owned String
    cloud_config: CloudConfig,
) -> Result<ReconResults, ReconError> {
    info!(target = domain.as_str(), "Starting all reconnaissance checks...");
    let mut results = ReconResults::new(domain.clone()); // Initialize results with cloned domain

    // --- DNS Checks (Run first, as some later checks might depend on it) ---
    let dns_results_result = run_dns_checks(&domain).await;
    match dns_results_result {
        Ok(dns_res) => {
            info!(target = domain.as_str(), "DNS checks completed successfully.");
            results.dns_results = Some(dns_res); // Assign directly
        }
        Err(e) => {
            warn!(target = domain.as_str(), "DNS checks failed: {}", e);
            // Continue without DNS results
            results.dns_results = None;
        }
    };
    // Clone DNS results *after* handling the Result, if needed by subsequent tasks
    let dns_results_clone = results.dns_results.clone();

    // --- Tenant and AAD Info Checks (Can run concurrently) ---
    // Note: Some AAD/Tenant checks might ideally use DNS results, but run independently for now.
    let client_clone1 = client.clone();
    let domain_clone1 = domain.to_string();
    let config_clone1 = cloud_config.clone();
    let fed_info_handle = tokio::spawn(get_federation_info(client_clone1, domain_clone1, config_clone1));

    let client_clone2 = client.clone();
    let domain_clone2 = domain.to_string(); // Use a different clone if needed later
    let config_clone2 = cloud_config.clone();
    let aad_config_handle = tokio::spawn(get_azure_ad_config(client_clone2, domain_clone2, config_clone2));

    let client_clone3 = client.clone();
    let domain_clone3 = domain.to_string(); // Use a different clone if needed later
    let config_clone3 = cloud_config.clone();
    let aad_connect_handle = tokio::spawn(check_aad_connect_status(client_clone3, domain_clone3, config_clone3));

    // Await Tenant/AAD results
    let fed_info_result = fed_info_handle.await;
    let aad_config_result = aad_config_handle.await;
    let aad_connect_status_result = aad_connect_handle.await;

    // Properly handle JoinHandle<Result<T, E>> and assign Some(T) if Ok, None otherwise
    results.federation_info = match fed_info_result {
        Ok(Ok(fed_info)) => Some(fed_info),
        _ => None,
    };
    
    results.azure_ad_config = match aad_config_result {
        Ok(Ok(aad_config)) => Some(aad_config),
        _ => None,
    };
    
    results.aad_connect_status = match aad_connect_status_result {
        Ok(Ok(status)) => Some(status),
        _ => None,
    };

    // --- Service Checks (Can run concurrently, may depend on DNS/Tenant) ---
    // Pass DNS results if needed
    let client_clone4 = client.clone();
    let domain_clone4 = domain.to_string();
    let config_clone4 = cloud_config.clone();
    // Pass the cloned Option<DnsResults> from before
    let m365_handle = tokio::spawn(run_m365_checks(client_clone4, domain_clone4, config_clone4, dns_results_clone));

    let client_clone5 = client.clone();
    let domain_clone5 = domain.to_string();
    let config_clone5 = cloud_config.clone();
    let azure_svc_handle = tokio::spawn(run_azure_service_checks(client_clone5, domain_clone5, config_clone5));

    // Await Service results
    match m365_handle.await {
        Ok(m365_res_result) => { // Result<Result<M365Results, ReconError>, JoinError>
            match m365_res_result {
                Ok(m365_res) => {
                    info!(target = domain.as_str(), "M365 service checks completed.");
                    results.m365_results = Some(m365_res); // Assign the inner M365Results
                }
                Err(e) => {
                    warn!(target = domain.as_str(), "M365 service checks failed: {}", e);
                }
            }
        }
        Err(join_err) => { // Task failed to join (e.g., panic)
            error!(target = domain.as_str(), "M365 service check task failed: {}", join_err);
            results.m365_results = None;
        }
    }

    match azure_svc_handle.await {
        Ok(Ok(azure_res)) => { // Task completed successfully with Ok(azure_res)
            info!(target = domain.as_str(), "Azure service checks completed.");
            results.azure_service_results = Some(azure_res);
        }
        Ok(Err(e)) => {
            warn!(target = domain.as_str(), "Azure service checks failed: {}", e);
            // Continue without Azure service results
            results.azure_service_results = None;
        }
        Err(join_err) => { // Task failed to join (e.g., panic)
            error!(target = domain.as_str(), "Azure service check task failed: {}", join_err);
            results.azure_service_results = None;
        }
    }

    info!(target = domain.as_str(), "All reconnaissance checks finished.");
    Ok(results)
}