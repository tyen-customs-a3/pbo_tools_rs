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

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List contents of PBO file
    List {
        /// Path to PBO file
        pbo_path: PathBuf,

        /// Use brief directory-style output listing
        #[arg(short, long)]
        brief: bool,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
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

        /// Keep PBO name in output path
        #[arg(short, long)]
        keep_pbo_name: bool,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Don't treat warnings as errors
        #[arg(short = 'w', long)]
        ignore_warnings: bool,
    },
}
