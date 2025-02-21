use std::path::PathBuf;
use pbo_tools_rs::{PboApi, PboError};
use log::{info, error};
use clap::{Parser, Subcommand};
use env_logger::Env;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Timeout in seconds for operations
    #[arg(short, long, default_value = "30")]
    timeout: u32,
}

#[derive(Subcommand)]
enum Commands {
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

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    let cli = Cli::parse();
    let api = PboApi::new(cli.timeout);

    if let Err(e) = match cli.command {
        Commands::List { pbo_path } => {
            list_contents(&api, &pbo_path)
        },
        Commands::Extract { pbo_path, output_dir, filter } => {
            extract_contents(&api, &pbo_path, &output_dir, filter.as_deref())
        },
    } {
        error!("Operation failed: {}", e);
        std::process::exit(1);
    }
}

fn list_contents(api: &PboApi, pbo_path: &PathBuf) -> Result<(), PboError> {
    info!("Listing contents of {}", pbo_path.display());
    let (success, content) = api.list_contents(pbo_path)?;
    if success {
        println!("{}", content);
        Ok(())
    } else {
        Err(PboError::InvalidPbo(content))
    }
}

fn extract_contents(api: &PboApi, pbo_path: &PathBuf, output_dir: &PathBuf, filter: Option<&str>) -> Result<(), PboError> {
    info!("Extracting {} to {}", pbo_path.display(), output_dir.display());
    if let Some(f) = filter {
        info!("Using filter: {}", f);
    }

    if api.extract(pbo_path, output_dir, filter)? {
        info!("Successfully extracted PBO to {}", output_dir.display());
        Ok(())
    } else {
        Err(PboError::Extraction(pbo_tools_rs::errors::ExtractError::CommandFailed {
            cmd: "extractpbo".to_string(),
            reason: "Extraction failed".to_string(),
        }))
    }
}
