use crate::error::ReconError;
use crate::models::ReconResults;
use serde_json;
use std::io::{self, Write};

/// Helper function to print a field with boolean value.
fn print_bool_field(writer: &mut impl Write, label: &str, value: Option<bool>) -> io::Result<()> {
    let display_value = match value {
        Some(true) => "Yes",
        Some(false) => "No",
        None => "Unknown",
    };
    writeln!(writer, "  {}: {}", label, display_value)
}

/// Helper function to print a field with string value.
fn print_string_field(writer: &mut impl Write, label: &str, value: Option<&str>) -> io::Result<()> {
    let display_value = value.unwrap_or("Not Available");
    writeln!(writer, "  {}: {}", label, display_value)
}

/// Helper function to print a field with vector of strings.
fn print_vec_field(writer: &mut impl Write, label: &str, values: &[String]) -> io::Result<()> {
    if values.is_empty() {
        writeln!(writer, "  {}: None Found", label)
    } else {
        writeln!(writer, "  {}:", label)?;
        for value in values {
            writeln!(writer, "    - {}", value)?;
        }
        Ok(())
    }
}

/// Prints the reconnaissance results to standard output.
///
/// Formats the output as JSON if `json_output` is true, otherwise prints
/// a human-readable summary.
pub fn print_results(results: &ReconResults, json_output: bool) -> Result<(), ReconError> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if json_output {
        // Serialize the entire results struct to JSON
        let json_string = serde_json::to_string_pretty(results)?;
        writeln!(handle, "{}", json_string)?;
    } else {
        // Print human-readable output
        writeln!(handle, "--- Reconnaissance Results for: {} ---", results.domain)?;

        if let Some(dns) = &results.dns_results {
            writeln!(handle, "\n[+] DNS Records:")?;
            print_bool_field(&mut handle, "MX Records Found", dns.mx_records_found)?;
            print_bool_field(&mut handle, "SPF Record Found", dns.spf_record_found)?;
            print_bool_field(&mut handle, "DMARC Record Found", dns.dmarc_record_found)?;
            // Handle autodiscover which is now Option<String> not Option<bool>
            print_bool_field(&mut handle, "Autodiscover Present", Some(dns.autodiscover_cname_or_a.is_some()))?;
            print_bool_field(&mut handle, "LyncDiscover Present", dns.lyncdiscover_present)?;
            print_bool_field(&mut handle, "SIP CName/A Present", dns.sip_cname_or_a_present)?;
        }

        if let Some(tenant) = &results.tenant_info {
            writeln!(handle, "\n[+] Tenant Information:")?;
            print_string_field(&mut handle, "Tenant ID", tenant.tenant_id.as_deref())?;
            print_string_field(&mut handle, "Tenant Name", tenant.tenant_name.as_deref())?;
            print_string_field(&mut handle, "Cloud Instance Name", tenant.cloud_instance_name.as_deref())?;
            print_bool_field(&mut handle, "Likely M365 Usage", tenant.likely_m365_usage)?;
        }

        // Federation info is now a top-level field in ReconResults
        if let Some(federation) = &results.federation_info {
            writeln!(handle, "\n[+] Federation Information:")?;
            print_bool_field(&mut handle, "Is Federated", Some(federation.is_federated))?;
            print_string_field(&mut handle, "Federation Brand Name", federation.federation_brand_name.as_deref())?;
            print_string_field(&mut handle, "Namespace Type", federation.name_space_type.as_deref())?;
            print_string_field(&mut handle, "Authentication URL", federation.auth_url.as_deref())?;
            print_string_field(&mut handle, "Cloud Instance Name", federation.cloud_instance_name.as_deref())?;
        }

        // Azure AD config is now a top-level field in ReconResults
        if let Some(aad_config) = &results.azure_ad_config {
            writeln!(handle, "\n[+] Azure AD OpenID Config:")?;
            print_string_field(&mut handle, "Issuer", aad_config.issuer.as_deref())?;
            print_string_field(&mut handle, "Authorization Endpoint", aad_config.authorization_endpoint.as_deref())?;
            print_string_field(&mut handle, "Token Endpoint", aad_config.token_endpoint.as_deref())?;
            print_string_field(&mut handle, "JWKS URI", aad_config.jwks_uri.as_deref())?;
            print_string_field(&mut handle, "Tenant Region Scope", aad_config.tenant_region_scope.as_deref())?;
        }

        // AAD Connect status is now a top-level field in ReconResults
        if let Some(aad_connect) = &results.aad_connect_status {
            writeln!(handle, "\n[+] Azure AD Connect Status:")?;
            match aad_connect {
                crate::models::AadConnectStatus::Hybrid => writeln!(handle, "  Status: Hybrid")?,
                crate::models::AadConnectStatus::CloudOnly => writeln!(handle, "  Status: Cloud Only")?,
                crate::models::AadConnectStatus::Unknown => writeln!(handle, "  Status: Unknown")?,
            }
        }

        if let Some(m365) = &results.m365_results {
            writeln!(handle, "\n[+] M365 Services:")?;
            print_bool_field(&mut handle, "SharePoint Detected", m365.sharepoint_detected)?;
            print_bool_field(&mut handle, "Teams Detected (via DNS)", m365.teams_detected)?;
            print_bool_field(&mut handle, "Tenant Branding Accessible", m365.tenant_branding_accessible)?;
            print_bool_field(&mut handle, "Legacy Auth (EWS)", m365.legacy_auth_ews_enabled)?;
            print_bool_field(&mut handle, "Legacy Auth (ActiveSync)", m365.legacy_auth_activesync_enabled)?;
        }

        if let Some(azure) = &results.azure_service_results {
            writeln!(handle, "\n[+] Azure Services:")?;
            print_vec_field(&mut handle, "Probable App Services", &azure.probable_app_services)?;
            print_vec_field(&mut handle, "Probable Storage Accounts", &azure.probable_storage_accounts)?;
            print_vec_field(&mut handle, "Probable CDN Endpoints", &azure.probable_cdn_endpoints)?;
        }

        writeln!(handle, "\n--- End of Report ---")?;
    }

    Ok(())
}