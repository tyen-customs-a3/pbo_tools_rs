use std::path::Path;
use std::fmt::Debug;
use std::process::Command;
use log::debug;
use crate::error::{PboError, ExtractError, Result};
use super::result::ExtractResult;

// Combining the traits into a single trait to avoid trait object limitations
pub trait ExtractorClone: Send + Sync + Debug {
    fn extract(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult>;
    fn list_contents(&self, pbo_path: &Path, brief: bool) -> Result<ExtractResult>;
    fn clone_box(&self) -> Box<dyn ExtractorClone>;
}

impl Clone for Box<dyn ExtractorClone> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct DefaultExtractor;

impl DefaultExtractor {
    pub fn new() -> Self {
        Self
    }

    fn run_extractpbo_command(&self, mut args: Vec<&str>, pbo_path: &Path) -> Result<ExtractResult> {
        debug!("Running extractpbo command with args: {:?}", args);
        debug!("PBO path: {:?}", pbo_path);
        
        // Basic path validation
        if !pbo_path.exists() || !pbo_path.is_file() {
            return Err(PboError::InvalidPath(pbo_path.to_path_buf()));
        }
        
        let mut command = Command::new("extractpbo");
        
        // Filter out any empty arguments and validate them
        args.retain(|&arg| {
            if arg.is_empty() {
                debug!("Removing empty argument");
                return false;
            }
            if arg.contains(char::is_control) {
                debug!("Removing argument with control characters");
                return false;
            }
            true
        });

        // Add -P flag first
        command.arg("-P");
        
        // Add any filter arguments before paths
        for arg in args.iter() {
            if arg.starts_with("-") {
                command.arg(arg);
            }
        }
        
        // Add PBO path with validation
        match pbo_path.to_str() {
            Some(pbo_str) if !pbo_str.contains(char::is_control) => {
                command.arg(pbo_str);
            }
            _ => return Err(PboError::InvalidPath(pbo_path.to_path_buf())),
        }
        
        // Add destination path if present with validation
        for arg in args.iter() {
            if !arg.starts_with("-") {
                let path = Path::new(arg);
                if path.exists() {
                    match path.to_str() {
                        Some(path_str) if !path_str.contains(char::is_control) => {
                            command.arg(path_str);
                        }
                        _ => return Err(PboError::InvalidPath(path.to_path_buf())),
                    }
                }
            }
        }

        debug!("Full command: {:?}", command);
        
        // Execute the command with enhanced error handling
        let output = command
            .output()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => PboError::CommandNotFound("extractpbo".to_string()),
                _ => PboError::Extraction(ExtractError::CommandFailed {
                    cmd: "extractpbo".to_string(),
                    reason: e.to_string(),
                })
            })?;

        debug!("Command completed with status: {:?}", output.status);
        debug!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        debug!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

        // Check for known error indicators in the output
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("Bad Sha") || stderr.contains("unknown header type") {
            return Err(PboError::Extraction(ExtractError::BadFormat {
                reason: stderr.to_string(),
            }));
        }

        Ok(ExtractResult {
            return_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: stderr.to_string(),
        })
    }
}

impl ExtractorClone for DefaultExtractor {
    fn extract(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        let mut args = Vec::new();
        
        if let Some(filter) = file_filter {
            args.push("-F");
            args.push(filter);
        }
        
        if let Some(out_str) = output_dir.to_str() {
            args.push(out_str);
        } else {
            return Err(PboError::InvalidPath(output_dir.to_path_buf()));
        }

        self.run_extractpbo_command(args, pbo_path)
    }

    fn list_contents(&self, pbo_path: &Path, brief: bool) -> Result<ExtractResult> {
        let args = if brief {
            vec!["-LB"]  // Brief listing
        } else {
            vec!["-L"]   // Normal listing
        };
        
        self.run_extractpbo_command(args, pbo_path)
    }

    fn clone_box(&self) -> Box<dyn ExtractorClone> {
        Box::new(self.clone())
    }
}
