use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Timeout in seconds for operations
    #[arg(short, long, default_value = "30")]
    pub timeout: u32,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List contents of PBO file
    List {
        /// Path to PBO file
        pbo_path: PathBuf,
    },
    /// Extract PBO file contents
    Extract {
        /// Path to PBO file
        pbo_path: PathBuf,
        /// Output directory
        output_dir: PathBuf,
        /// Optional filter for specific files
        #[arg(short, long)]
        filter: Option<String>,
    },
}
