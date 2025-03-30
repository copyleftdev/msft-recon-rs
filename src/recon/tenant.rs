use crate::config::CloudConfig;
use crate::error::ReconError;
use crate::models::FederationInfo;
use reqwest::Client;
use tracing::{debug, warn};

/// Fetches federation information using the getuserrealm.srf endpoint.
///
/// This function attempts to determine if a domain is Managed or Federated
/// and extracts related details from the XML response.
pub async fn get_federation_info(
    client: Client, // Pass cloned client
    domain: String, // Pass owned domain
    cloud_config: CloudConfig, // Pass cloned config
) -> Result<FederationInfo, ReconError> {
    // Construct the URL. We need a placeholder user for the query.
    let url = format!(
        "{}?login=recon@{}.&xml=1",
        cloud_config.user_realm_endpoint,
        domain
    );
    debug!(target = domain, url = url.as_str(), "Querying GetUserRealm");

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        warn!(target = domain, status = %response.status(), url = url.as_str(), "GetUserRealm request failed");
        // Consider specific handling for certain status codes if needed
        return Err(ReconError::UnexpectedApiResponse {
            service: "GetUserRealm".to_string(),
            status: response.status(),
            body: response.text().await.unwrap_or_else(|_| "<failed to read body>".to_string()),
        });
    }

    let body = response.text().await?;
    debug!(target = domain, "GetUserRealm response body received");

    // Basic XML parsing using string searching (fragile, but avoids new dependencies for now)
    let name_space_type = extract_xml_tag_value(&body, "NameSpaceType").unwrap_or("Unknown".to_string());
    let federation_brand_name = extract_xml_tag_value(&body, "FederationBrandName");
    // TODO: Extract CloudInstanceName as well if needed, although config already provides endpoints.
    // let cloud_instance_name = extract_xml_tag_value(&body, "CloudInstanceName");

    Ok(FederationInfo {
        is_federated: true, // If we got here, we have federation info, so it's federated
        name_space_type: Some(name_space_type), // Convert String to Option<String>
        federation_brand_name, // Already an Option<String>
        auth_url: None, // Could extract from XML if needed
        cloud_instance_name: None, // Could extract from XML if needed
    })
}

/// Simple helper to extract the value from an XML tag.
/// Example: <Tag>Value</Tag> -> "Value"
fn extract_xml_tag_value(xml: &str, tag_name: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag_name);
    let end_tag = format!("</{}>", tag_name);

    xml.find(&start_tag)
        .and_then(|start_index| {
            let value_start = start_index + start_tag.len();
            xml[value_start..]
                .find(&end_tag)
                .map(|end_index| xml[value_start..value_start + end_index].to_string())
        })
}