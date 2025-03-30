use serde::{Serialize, Deserialize};

// --- Core Tenant Information ---

#[derive(Debug, Serialize, Deserialize, Clone, Default)] // Default is useful for initializing
pub struct TenantInfo {
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>, // GUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_name: Option<String>, // e.g., contoso.onmicrosoft.com
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_instance_name: Option<String>, // e.g., microsoftonline.com
    // Moved federation_info, azure_ad_config, aad_connect_status to ReconResults
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likely_m365_usage: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct FederationInfo {
    pub is_federated: bool, // Derived from user realm check
    #[serde(rename = "NameSpaceType", skip_serializing_if = "Option::is_none")]
    pub name_space_type: Option<String>, // Managed, Federated, Unknown
    #[serde(rename = "FederationBrandName", skip_serializing_if = "Option::is_none")]
    pub federation_brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_url: Option<String>, // Authentication URL if federated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_instance_name: Option<String>, // e.g., MicrosoftOnline.com
    // Add other fields from getuserrealm.srf as needed
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct AzureAdConfig {
    // Fields from OpenID Connect config
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_region_scope: Option<String>,
    // Add other relevant fields
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AadConnectStatus {
    Hybrid,     // Inferred if SSO URL check succeeds
    CloudOnly,  // Inferred if SSO URL check fails/times out
    Unknown,    // If check could not be performed
}

// --- Service Check Results ---

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DnsResults {
    // MX Records
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mx_records: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mx_records_found: Option<bool>,
    // SPF Record
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spf_record: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spf_record_found: Option<bool>,
    // DMARC Record
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dmarc_record: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dmarc_record_found: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dmarc_policy: Option<String>, // e.g., none, quarantine, reject
    // Specific TXT Records for M365/Azure indicators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ms_txt_record: Option<String>, // ms=... record
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ms_txt_found: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ms_adfs_auth_txt_record: Option<String>, // MS-ADFS-Authentication=...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ms_adfs_auth_txt_found: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterpriseregistration_txt_record: Option<String>, // enterpriseregistration...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterpriseregistration_txt_found: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterpriseenrollment_txt_record: Option<String>, // enterpriseenrollment...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterpriseenrollment_txt_found: Option<bool>,
    // Lync/Skype/Teams DNS Records
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autodiscover_cname_or_a: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lyncdiscover_present: Option<bool>, // _sipfederationtls._tcp.<domain> or lyncdiscover.<domain> CNAME/A
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sip_cname_or_a_present: Option<bool>, // sip.<domain> CNAME/A
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sipfederationtls_tcp_present: Option<bool>, // _sipfederationtls._tcp.<domain> SRV
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sip_tls_present: Option<bool>, // _sip._tls.<domain> SRV
}

impl PartialEq for DnsResults {
    fn eq(&self, other: &Self) -> bool {
        // Compare fields that matter for equality
        self.mx_records == other.mx_records &&
        self.mx_records_found == other.mx_records_found &&
        self.spf_record == other.spf_record &&
        self.spf_record_found == other.spf_record_found &&
        self.dmarc_record == other.dmarc_record &&
        self.dmarc_record_found == other.dmarc_record_found &&
        self.dmarc_policy == other.dmarc_policy &&
        self.ms_txt_record == other.ms_txt_record &&
        self.ms_txt_found == other.ms_txt_found &&
        self.ms_adfs_auth_txt_record == other.ms_adfs_auth_txt_record &&
        self.ms_adfs_auth_txt_found == other.ms_adfs_auth_txt_found &&
        self.enterpriseregistration_txt_record == other.enterpriseregistration_txt_record &&
        self.enterpriseregistration_txt_found == other.enterpriseregistration_txt_found &&
        self.enterpriseenrollment_txt_record == other.enterpriseenrollment_txt_record &&
        self.enterpriseenrollment_txt_found == other.enterpriseenrollment_txt_found &&
        self.autodiscover_cname_or_a == other.autodiscover_cname_or_a &&
        self.lyncdiscover_present == other.lyncdiscover_present &&
        self.sip_cname_or_a_present == other.sip_cname_or_a_present &&
        self.sipfederationtls_tcp_present == other.sipfederationtls_tcp_present &&
        self.sip_tls_present == other.sip_tls_present
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct M365Results {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sharepoint_detected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teams_detected: Option<bool>, // Based on Lync/SIP DNS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_branding_accessible: Option<bool>, // Login page branding check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_auth_ews_enabled: Option<bool>, // EWS endpoint responsive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy_auth_activesync_enabled: Option<bool>, // ActiveSync endpoint responsive
    // Add Power Apps, etc.
}

impl PartialEq for M365Results {
    fn eq(&self, other: &Self) -> bool {
        self.sharepoint_detected == other.sharepoint_detected &&
        self.teams_detected == other.teams_detected &&
        self.tenant_branding_accessible == other.tenant_branding_accessible &&
        self.legacy_auth_ews_enabled == other.legacy_auth_ews_enabled &&
        self.legacy_auth_activesync_enabled == other.legacy_auth_activesync_enabled
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AzureServiceResults {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub probable_app_services: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub probable_storage_accounts: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub probable_cdn_endpoints: Vec<String>,
    // Add Key Vault, Functions, SWAs, ACR, Cog Services, B2C etc.
}

impl PartialEq for AzureServiceResults {
    fn eq(&self, other: &Self) -> bool {
        self.probable_app_services == other.probable_app_services &&
        self.probable_storage_accounts == other.probable_storage_accounts &&
        self.probable_cdn_endpoints == other.probable_cdn_endpoints
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AadAppResults {
    pub aad_apps_detected: Option<Vec<String>>,
    // Add more fields for specific discoveries
}

impl PartialEq for AadAppResults {
    fn eq(&self, other: &Self) -> bool {
        self.aad_apps_detected == other.aad_apps_detected
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SecurityServiceResults {
    // Security posture results
    pub mfa_enforced: Option<bool>,
    pub conditional_access_policies: Option<Vec<String>>,
    // Add more security service checks
}

impl PartialEq for SecurityServiceResults {
    fn eq(&self, other: &Self) -> bool {
        self.mfa_enforced == other.mfa_enforced &&
        self.conditional_access_policies == other.conditional_access_policies
    }
}

// --- Aggregated Results Structure ---

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ReconResults {
    pub domain: String,
    pub dns_results: Option<DnsResults>,
    pub tenant_info: Option<TenantInfo>,
    pub federation_info: Option<FederationInfo>,
    pub azure_ad_config: Option<AzureAdConfig>,
    pub aad_connect_status: Option<AadConnectStatus>,
    pub m365_results: Option<M365Results>,
    pub azure_service_results: Option<AzureServiceResults>,
    pub aad_app_results: Option<AadAppResults>,
    pub security_service_results: Option<SecurityServiceResults>,
    // Add other result categories as needed
}

impl ReconResults {
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            ..Default::default()
        }
    }
}