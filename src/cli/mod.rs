pub mod args;
pub mod commands;

use log::debug;
use crate::core::api::{PboApi, PboApiOps};
use crate::error::types::{Result, PboError};
use self::args::{Commands, Cli};

pub struct CliProcessor {
    api: PboApi,
}

impl CliProcessor {
    pub fn new(timeout: u32) -> Self {
        debug!("Creating new CliProcessor with timeout: {} seconds", timeout);
        Self {
            api: PboApi::builder()
                .with_timeout(timeout)
                .build(),
        }
    }

    pub fn process_command(&self, command: Commands) -> Result<()> {
        debug!("Processing command: {:?}", command);
        match command {
            Commands::List { pbo_path } => {
                debug!("Listing contents of PBO: {}", pbo_path.display());
                self.api.list_contents(&pbo_path)
                    .and_then(|result| {
                        if result.is_success() {
                            println!("Files in PBO:");
                            for file in result.get_file_list() {
                                println!("  {}", file);
                            }
                            Ok(())
                        } else {
                            Err(PboError::Extraction(result.get_error_message()
                                .map(|msg| crate::error::types::ExtractError::CommandFailed {
                                    cmd: "extractpbo".to_string(),
                                    reason: msg,
                                })
                                .unwrap_or_else(|| crate::error::types::ExtractError::NoFiles)))
                        }
                    })
            }
            Commands::Extract { pbo_path, output_dir, filter } => {
                debug!("Extracting from PBO: {} to {}", pbo_path.display(), output_dir.display());
                debug!("Using filter: {:?}", filter);
                debug!("Current directory: {:?}", std::env::current_dir().unwrap_or_default());
                
                // Ensure output directory exists
                std::fs::create_dir_all(&output_dir)
                    .map_err(|e| PboError::FileSystem(crate::error::types::FileSystemError::CreateDir {
                        path: output_dir.clone(),
                        reason: e.to_string(),
                    }))?;

                debug!("Created output directory: {}", output_dir.display());

                let result = self.api.extract_files(&pbo_path, &output_dir, filter.as_deref());
                debug!("Extract result: {:?}", result);
                
                result.and_then(|result| {
                    if result.is_success() {
                        println!("Extracted files:");
                        for file in result.get_file_list() {
                            println!("  {}", file);
                        }
                        if let Some(prefix) = result.get_prefix() {
                            println!("\nPBO Prefix: {}", prefix);
                        }
                        Ok(())
                    } else {
                        debug!("Extraction failed: {}", result);
                        Err(PboError::Extraction(result.get_error_message()
                            .map(|msg| crate::error::types::ExtractError::CommandFailed {
                                cmd: "extractpbo".to_string(),
                                reason: msg,
                            })
                            .unwrap_or_else(|| crate::error::types::ExtractError::NoFiles)))
                    }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use crate::test_utils;

    #[test]
    fn test_cli_list_command() {
        test_utils::setup();
        let cli = CliProcessor::new(10);
        let test_pbo = test_utils::get_test_pbo_path();
        let result = cli.process_command(Commands::List { 
            pbo_path: test_pbo 
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_extract_command() {
        test_utils::setup();
        let cli = CliProcessor::new(10);
        let test_pbo = test_utils::get_test_pbo_path();
        let temp_dir = tempdir().unwrap();
        
        let result = cli.process_command(Commands::Extract { 
            pbo_path: test_pbo,
            output_dir: temp_dir.path().to_path_buf(),
            filter: None,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_with_invalid_paths() {
        test_utils::setup();
        let cli = CliProcessor::new(30);
        let invalid_pbo = PathBuf::from("nonexistent.pbo");
        
        let result = cli.process_command(Commands::List { 
            pbo_path: invalid_pbo.clone() 
        });
        assert!(result.is_err());

        let result = cli.process_command(Commands::Extract { 
            pbo_path: invalid_pbo,
            output_dir: PathBuf::from("output"),
            filter: None,
        });
        assert!(result.is_err());
    }
}
