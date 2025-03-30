mod cli;
mod config;
mod error;
mod models;
mod output;
mod recon;

use clap::Parser;
use tracing::{error, info, Level};
use tracing_subscriber::{fmt, EnvFilter};

// Use `crate::` for modules within the same crate (binary)
use crate::cli::Cli;
use crate::config::{load_config, select_cloud_config};
use crate::error::ReconError;
use crate::output::print_results;
use crate::recon::client::new_client;
use crate::recon::run_all_checks;

#[tokio::main]
async fn main() -> Result<(), ReconError> {
    // 1. Setup Logging
    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| ReconError::Other(format!("Failed to set tracing subscriber: {}", e)))?;

    info!("MSFTRecon-RS starting...");

    // 2. Parse CLI Arguments
    let cli = Cli::parse();
    info!("Starting MSFTRecon-RS");

    if cli.domain.is_empty() {
        error!("Domain argument is required");
        return Err(ReconError::cli_error("Domain argument is required"));
    }

    // 3. Load Configuration
    info!("Loading configuration");
    let app_config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load application configuration: {}", e);
            return Err(e);
        }
    };

    let cloud_config = match select_cloud_config(&app_config, &cli.cloud) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to select cloud configuration: {}", e);
            return Err(e);
        }
    };
    info!("Using cloud configuration: {:?}", cli.cloud);

    // Initialize HTTP Client
    let client = match new_client(&app_config) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to setup HTTP client: {}", e);
            return Err(e);
        }
    };
    info!("HTTP Client initialized");

    // --- Run Reconnaissance Checks ---
    info!(target = &cli.domain, "Starting reconnaissance...");
    match run_all_checks(client, cli.domain.clone(), cloud_config.clone()).await {
        Ok(results) => {
            info!(target = &cli.domain, "Reconnaissance finished.");

            // --- Output Results ---
            match print_results(&results, cli.json) {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("Failed to output results: {}", e);
                    Err(e)
                }
            }
        }
        Err(e) => {
            error!("Reconnaissance run failed: {}", e);
            Err(e)
        }
    }
}