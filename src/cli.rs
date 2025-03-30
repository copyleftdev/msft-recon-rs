use clap::{Parser, ValueEnum};

/// Command-line arguments for msft-recon-rs.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// The target domain name for reconnaissance
    #[clap(short, long)]
    pub domain: String,

    /// Specify the target cloud environment
    #[clap(short, long, value_enum, default_value_t = CloudTarget::Commercial)]
    pub cloud: CloudTarget,

    /// Output results in JSON format
    #[clap(long)]
    pub json: bool,
    // Add other arguments like verbosity, output file etc. later if needed
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudTarget {
    Commercial,
    Gcc,
    GccHigh,
    Dod,
}

/// Parses command line arguments.
///
/// This function initializes and parses the arguments using clap.
/// It will exit the program with usage information if arguments are invalid.
#[allow(dead_code)]
pub fn parse_args() -> Cli {
    Cli::parse()
}

// Example usage in main.rs:
// let args = cli::parse_args();
// println!("Target Domain: {}", args.domain);
// println!("Target Cloud: {:?}", args.cloud);
// if args.json { println!("Output Format: JSON"); }