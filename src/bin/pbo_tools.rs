use std::process;
use env_logger;
use log::error;
use clap::Parser;
use pbo_tools_rs::cli::args::Cli;
use pbo_tools_rs::cli::CliProcessor;
use pbo_tools_rs::core::constants::DEFAULT_TIMEOUT;

fn main() {
    env_logger::init();
    
    let cli = Cli::parse();
    let processor = CliProcessor::new(DEFAULT_TIMEOUT);
    
    if let Err(e) = processor.process_command(cli.command) {
        error!("{}", e);
        std::process::exit(1);
    }
}
