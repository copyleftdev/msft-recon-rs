// End-to-end integration tests for the CLI
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

/// Test that the CLI performs a complete end-to-end recon flow correctly
#[tokio::test]
async fn test_end_to_end_recon_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Start a mock server
    let mock_server = MockServer::start().await;
    
    // Setup mocks for Microsoft Azure AD & other services
    setup_m365_service_mocks(&mock_server).await;
    
    // Create a temporary configuration file that points to our mock server
    let config_dir = PathBuf::from("config");
    fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("default.toml");
    
    // Backup existing config if it exists
    let backup_path = config_dir.join("default.toml.bak");
    let had_existing_config = config_path.exists();
    if had_existing_config {
        fs::copy(&config_path, &backup_path)?;
    }
    
    // Create our test config
    create_test_config(&config_path, &mock_server.uri())?;
    
    // Ensure the config is restored after the test
    struct CleanupGuard {
        config_path: PathBuf,
        backup_path: PathBuf,
        had_existing_config: bool,
    }
    
    impl Drop for CleanupGuard {
        fn drop(&mut self) {
            if self.had_existing_config {
                // Restore backup
                let _ = fs::copy(&self.backup_path, &self.config_path);
                let _ = fs::remove_file(&self.backup_path);
            } else {
                // Remove test config
                let _ = fs::remove_file(&self.config_path);
            }
        }
    }
    
    let _cleanup = CleanupGuard {
        config_path: config_path.clone(),
        backup_path: backup_path.clone(),
        had_existing_config,
    };
    
    // Run the command with json output
    let test_domain = "contoso.com";
    let mut cmd = Command::cargo_bin("msft-recon-rs")?;
    cmd.arg("--domain")
       .arg(test_domain)
       .arg("--cloud")
       .arg("commercial")
       .arg("--json");
    
    // Execute and check for success
    let assert = cmd.assert()
                  .success()
                  .stdout(predicate::str::contains(test_domain));
    
    // Get the full output
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    println!("Command output: {}", output);
    
    // Extract the JSON portion from the output
    // This is more robust than parsing the entire output as JSON
    let json_start = output.find('{').ok_or("No JSON found in output")?;
    let json_end = output.rfind('}').ok_or("Incomplete JSON in output")?;
    let json_str = &output[json_start..=json_end];
    
    // Parse the extracted JSON
    let output_json: Value = serde_json::from_str(json_str)?;
    
    // Verify that the output JSON contains expected fields
    assert!(output_json.get("domain").is_some(), "Output doesn't contain 'domain' field");
    assert!(output_json.get("checks").is_some() || output_json.get("dns_results").is_some(), 
        "Output doesn't contain expected result fields");
    
    // Test successful! Our end-to-end flow works
    Ok(())
}

/// Creates a test configuration file pointing to our mock server
fn create_test_config(config_path: &PathBuf, mock_server_uri: &str) -> Result<(), io::Error> {
    // Create a config file that exactly matches the AppConfig and CloudConfig struct requirements
    let config_content = format!(
        r#"# Test configuration for msft-recon-rs
default_user_agent = "MSFTRecon-RS Test/1.0"
request_timeout_seconds = 30

[clouds.commercial]
login_endpoint = "{0}/login"
login_microsoftonline_host = "{0}/login.microsoftonline.com"
user_realm_endpoint = "{0}/GetUserRealm.srf"
openid_config_endpoint = "/.well-known/openid-configuration"
azure_ad_connect_check_url = "{0}/autologon.microsoftazuread-sso.com/"
sharepoint_host_suffix = ".sharepoint.com"
cdn_host_suffix = ".azureedge.net"
ews_endpoint_host = "outlook.office365.com"
activesync_endpoint_host = "outlook.office365.com"
app_service_host_suffix = ".azurewebsites.net"
storage_account_host_suffix = ".blob.core.windows.net"

[clouds.gov]
login_endpoint = "{0}/login.microsoftonline.us"
login_microsoftonline_host = "{0}/login.microsoftonline.us"
user_realm_endpoint = "{0}/GetUserRealm.srf"
openid_config_endpoint = "/.well-known/openid-configuration"
azure_ad_connect_check_url = "{0}/autologon.microsoftazuread-sso.com/"
sharepoint_host_suffix = ".sharepoint.us"
cdn_host_suffix = ".azureedge.us"
ews_endpoint_host = "outlook.office365.us"
activesync_endpoint_host = "outlook.office365.us"
app_service_host_suffix = ".azurewebsites.us"
storage_account_host_suffix = ".blob.core.windows.us"

[clouds.cn]
login_endpoint = "{0}/login.partner.microsoftonline.cn"
login_microsoftonline_host = "{0}/login.partner.microsoftonline.cn"
user_realm_endpoint = "{0}/GetUserRealm.srf"
openid_config_endpoint = "/.well-known/openid-configuration"
azure_ad_connect_check_url = "{0}/"
sharepoint_host_suffix = ".sharepoint.cn"
cdn_host_suffix = ".azureedge.cn"
ews_endpoint_host = "outlook.partner.microsoftonline.cn"
activesync_endpoint_host = "outlook.partner.microsoftonline.cn"
app_service_host_suffix = ".chinacloudsites.cn"
storage_account_host_suffix = ".blob.core.chinacloudapi.cn"
"#,
        mock_server_uri
    );
    
    println!("Creating test config at: {}", config_path.display());
    
    // Write the config file
    let mut file = fs::File::create(config_path)?;
    file.write_all(config_content.as_bytes())?;
    
    // Validate that the file exists and has content
    let metadata = fs::metadata(config_path)?;
    println!("Config file size: {} bytes", metadata.len());
    
    // Read back the file to check content
    let mut file = fs::File::open(config_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    // Verify first characters of config
    if !content.is_empty() {
        let preview = if content.len() > 20 { &content[..20] } else { &content };
        println!("Config starts with: '{}'", preview);
    } else {
        println!("Warning: Config file is empty!");
    }
    
    Ok(())
}

/// Sets up mocks for Microsoft 365 services
async fn setup_m365_service_mocks(mock_server: &MockServer) {
    // Configure mock responses for each service

    // Microsoft 365 services
    // Mock Teams detection (should return 200 for successful detection)
    Mock::given(method("GET"))
        .and(path("/_vti_bin/client.svc/ProcessQuery"))
        .respond_with(ResponseTemplate::new(200))
        .mount(mock_server)
        .await;
        
    // Mock SharePoint (should return 404 to simulate not found)
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(404))
        .mount(mock_server)
        .await;
    
    // Mock other M365 services as needed...
}