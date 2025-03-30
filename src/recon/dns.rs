use crate::error::ReconError;
use crate::models::DnsResults;
use tracing::{debug, info, warn};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::proto::rr::RecordType;
use trust_dns_resolver::TokioAsyncResolver;

/// Performs all DNS-related reconnaissance checks concurrently.
pub async fn run_dns_checks(domain: &str) -> Result<DnsResults, ReconError> {
    info!(target = domain, "Starting DNS checks");
    // Create a resolver instance. Cache results for efficiency within this run.
    // Using Google's public DNS servers as a default, could be made configurable.
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::google(),
        ResolverOpts::default(),
    );

    let domain = domain.to_string(); // Clone domain for use in tasks

    // Spawn tasks for each DNS check
    let mx_handle = tokio::spawn(get_mx_records(resolver.clone(), domain.clone()));
    let txt_handle = tokio::spawn(get_txt_records(resolver.clone(), domain.clone()));
    let autodiscover_handle = tokio::spawn(check_autodiscover(resolver.clone(), domain.clone()));
    let lync_handle = tokio::spawn(check_record_presence(resolver.clone(), format!("lyncdiscover.{}", domain)));
    let sip_handle = tokio::spawn(check_record_presence(resolver.clone(), format!("sip.{}", domain)));

    // Await results, handling potential errors
    let mx_records = mx_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "MX lookup task failed");
        Err(ReconError::check_failed("MX Lookup", e.to_string()))
    })?;
    
    let txt_records = txt_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "TXT lookup task failed");
        Err(ReconError::check_failed("TXT Lookup", e.to_string()))
    })?;
    
    // Extract SPF and DMARC records
    let spf_record = txt_records.iter()
        .find(|txt| txt.to_lowercase().starts_with("v=spf1"))
        .map(|s| s.to_string());
    
    let dmarc_record = txt_records.iter()
        .find(|txt| txt.to_lowercase().starts_with("v=dmarc1"))
        .map(|s| s.to_string());
    
    let autodiscover_cname_or_a = autodiscover_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "Autodiscover check task failed");
        Err(ReconError::check_failed("Autodiscover Check", e.to_string()))
    })?;
    
    let lyncdiscover_present = lync_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "LyncDiscover check task failed");
        Err(ReconError::check_failed("LyncDiscover Check", e.to_string()))
    })?;
    
    let sip_cname_or_a_present = sip_handle.await.unwrap_or_else(|e| {
        warn!(domain = domain.as_str(), error = %e, "SIP check task failed");
        Err(ReconError::check_failed("SIP Check", e.to_string()))
    })?;

    info!(target = domain.as_str(), "Finished DNS checks");
    Ok(DnsResults {
        mx_records: Some(mx_records.clone()),
        mx_records_found: Some(!mx_records.is_empty()),
        spf_record: spf_record.clone(),
        spf_record_found: Some(spf_record.is_some()),
        dmarc_record: dmarc_record.clone(),
        dmarc_record_found: Some(dmarc_record.is_some()),
        dmarc_policy: extract_dmarc_policy(dmarc_record.as_deref()),
        ms_txt_record: None, // TODO: Extract MS TXT record if needed
        ms_txt_found: None,
        ms_adfs_auth_txt_record: None, // TODO: Extract ADFS auth TXT record if needed
        ms_adfs_auth_txt_found: None,
        enterpriseregistration_txt_record: None, // TODO: Add check if needed
        enterpriseregistration_txt_found: None,
        enterpriseenrollment_txt_record: None, // TODO: Add check if needed
        enterpriseenrollment_txt_found: None,
        autodiscover_cname_or_a,
        lyncdiscover_present: Some(lyncdiscover_present),
        sip_cname_or_a_present: Some(sip_cname_or_a_present),
        sipfederationtls_tcp_present: None, // TODO: Add check if needed
        sip_tls_present: None, // TODO: Add check if needed
    })
}

/// Resolves MX records for the given domain.
async fn get_mx_records(resolver: TokioAsyncResolver, domain: String) -> Result<Vec<String>, ReconError> {
    debug!(domain = domain.as_str(), "Querying MX records");
    let response = resolver.mx_lookup(domain.as_str()).await?;
    let records: Vec<String> = response
        .iter()
        .map(|mx| mx.exchange().to_string())
        .collect();
    debug!(domain = domain.as_str(), count = records.len(), "Found MX records");
    Ok(records)
}

/// Resolves TXT records for the given domain.
async fn get_txt_records(resolver: TokioAsyncResolver, domain: String) -> Result<Vec<String>, ReconError> {
    debug!(domain = domain.as_str(), "Querying TXT records");
    let response = resolver.txt_lookup(domain.as_str()).await?;
    let records: Vec<String> = response
        .iter()
        .flat_map(|txt| txt.iter().map(|bytes| String::from_utf8_lossy(bytes).to_string()))
        .collect();
    debug!(domain = domain.as_str(), count = records.len(), "Found TXT records");
    Ok(records)
}

/// Checks for Autodiscover CNAME or A record.
async fn check_autodiscover(resolver: TokioAsyncResolver, domain: String) -> Result<Option<String>, ReconError> {
    let autodiscover_domain = format!("autodiscover.{}", domain);
    debug!(domain = autodiscover_domain.as_str(), "Checking autodiscover");
    
    // First try CNAME lookup
    match resolver.lookup(autodiscover_domain.as_str(), RecordType::CNAME).await {
        Ok(cname_response) => {
            if let Some(cname) = cname_response.iter().next() {
                if let Some(name) = cname.as_cname() {
                    return Ok(Some(name.to_string()));
                }
            }
        }
        Err(e) => debug!(error = %e, "CNAME lookup failed, will try A record"),
    }
    
    // If CNAME fails, try A record
    match resolver.lookup_ip(autodiscover_domain.as_str()).await {
        Ok(a_response) => {
            if let Some(ip) = a_response.iter().next() {
                return Ok(Some(ip.to_string()));
            }
        }
        Err(e) => debug!(error = %e, "A record lookup failed"),
    }
    
    Ok(None) // No autodiscover record found
}

/// Generic check if a DNS record exists (A or CNAME).
async fn check_record_presence(resolver: TokioAsyncResolver, domain: String) -> Result<bool, ReconError> {
    debug!(domain = domain.as_str(), "Checking record presence");
    
    // Try A/AAAA lookup
    let ip_result = resolver.lookup_ip(domain.as_str()).await;
    if let Ok(response) = ip_result {
        if response.iter().next().is_some() {
            return Ok(true);
        }
    }
    
    // Try CNAME lookup
    let cname_result = resolver.lookup(domain.as_str(), RecordType::CNAME).await;
    if let Ok(response) = cname_result {
        if response.iter().next().is_some() {
            return Ok(true);
        }
    }
    
    Ok(false)
}

/// Extract the DMARC policy from a DMARC record.
fn extract_dmarc_policy(dmarc_record: Option<&str>) -> Option<String> {
    dmarc_record.and_then(|record| {
        record
            .split(';')
            .map(str::trim)
            .find(|part| part.to_lowercase().starts_with("p="))
            .map(|policy_part| policy_part[2..].to_string())
    })
}