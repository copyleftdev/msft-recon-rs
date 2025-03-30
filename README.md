# MSFT-Recon-RS

[![Rust CI/CD](https://github.com/username/msft-recon-rs/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/username/msft-recon-rs/actions/workflows/rust-ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)

A Rust-based reconnaissance tool for Microsoft Azure and Microsoft 365 environments. This tool helps security professionals and administrators identify exposed services, tenant information, and potential configuration issues in Microsoft cloud environments.

## Features

- **DNS Reconnaissance**: Detect Microsoft-related DNS records and service configurations
- **Microsoft 365 Service Detection**: Identify SharePoint, Teams, Exchange, and other M365 services
- **Azure AD Information Gathering**: Collect tenant information, federation status, and AAD Connect configuration
- **Azure Service Enumeration**: Discover Azure App Services, Storage Accounts, and other Azure resources
- **Multi-Cloud Support**: Works with Commercial, Government (GCC/GCC-High), and China cloud environments
- **JSON Output**: Structured data output for integration with other tools and reporting

## Installation

### From Source

Prerequisites:
- Rust 1.70+ and Cargo
- OpenSSL development libraries

```bash
# Clone the repository
git clone https://github.com/username/msft-recon-rs.git
cd msft-recon-rs

# Build the project
cargo build --release

# The binary will be available at ./target/release/msft-recon-rs
```

### Using Docker

```bash
# Build the Docker image
docker build -t msft-recon-rs .

# Run the tool with Docker
docker run msft-recon-rs --domain example.com --cloud commercial
```

### Using Docker Compose

```bash
# Start the services defined in docker-compose.yml
docker-compose up
```

## Usage

Basic usage:

```bash
# Run reconnaissance against a domain
msft-recon-rs --domain example.com --cloud commercial

# Output results in JSON format
msft-recon-rs --domain example.com --cloud commercial --json

# Use a specific configuration file
MSFT_RECON_CONFIG=/path/to/config.toml msft-recon-rs --domain example.com --cloud commercial
```

### Command-line options

```
USAGE:
    msft-recon-rs [OPTIONS] --domain <DOMAIN> --cloud <CLOUD>

OPTIONS:
    -d, --domain <DOMAIN>      Target domain to perform reconnaissance on
    -c, --cloud <CLOUD>        Cloud environment to use (commercial, gov, cn)
    -j, --json                 Output results in JSON format
    -h, --help                 Print help information
    -V, --version              Print version information
```

## Configuration

The tool uses a TOML configuration file to define endpoints and settings for different cloud environments. The default configuration is provided at `config/default.toml`.

Example configuration:

```toml
# Default settings
default_user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.0.0 Safari/537.36"
request_timeout_seconds = 10

[clouds.commercial]
login_endpoint = "https://login.microsoftonline.com"
login_microsoftonline_host = "login.microsoftonline.com"
# Additional endpoints...
```

## Project Structure

The project follows Rust Clean Architecture principles:

```
msft-recon-rs/
├── src/                 # Source code
│   ├── cli.rs           # Command-line interface
│   ├── config.rs        # Configuration handling
│   ├── error.rs         # Error types
│   ├── models.rs        # Data structures
│   ├── output.rs        # Output formatting
│   ├── recon/           # Reconnaissance modules
│   │   ├── aad.rs       # Azure AD reconnaissance
│   │   ├── azure_svc.rs # Azure services reconnaissance
│   │   ├── dns.rs       # DNS reconnaissance
│   │   ├── m365.rs      # Microsoft 365 reconnaissance
│   │   └── mod.rs       # Module exports
│   └── main.rs          # Application entry point
├── tests/               # Integration tests
│   └── cli_tests.rs     # End-to-end CLI tests
├── config/              # Configuration files
│   └── default.toml     # Default configuration
└── Cargo.toml           # Project dependencies
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_end_to_end_recon_flow
```

### Code Quality

```bash
# Run clippy lints
cargo clippy --all-features -- -D warnings

# Check formatting
cargo fmt --all -- --check
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- The Rust community for excellent libraries and tools
- Microsoft for their comprehensive API documentation