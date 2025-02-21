use std::process;
use env_logger;
use log::error;
use clap::Parser;
use pbo_tools_rs::cli::args::{Cli, Commands};
use pbo_tools_rs::cli::commands::{list_contents, extract_contents};
use pbo_tools_rs::core::PboCore;

fn main() {
    env_logger::init();
    
    let api = PboCore::new(None);
    let cli = Cli::parse();
    
    match cli.command {
        Commands::List { pbo_path } => {
            list_contents(&api, &pbo_path)
        }
        Commands::Extract { pbo_path, output_dir, filter } => {
            extract_contents(&api, &pbo_path, &output_dir, filter.as_deref())
        }
    }.unwrap_or_else(|e| {
        error!("{}", e);
        process::exit(1);
    });
}
