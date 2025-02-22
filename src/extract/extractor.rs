use std::path::Path;
use std::fmt::Debug;
use std::process::Command;
use log::{debug, warn};
use crate::error::{PboError, ExtractError, Result};
use super::result::ExtractResult;

/// ExtractPBO Command Line Arguments
/// 
/// PBO files can have extensions: .pbo, .xbo, .ebo, or any extension containing 'pbo'
/// 
/// Arguments:
/// - `-F filelist[,...]`: Extract specific file(s). Files are extracted to their correct position 
///   in the output folder tree. Supports basic wildcards (*.ext for all files with extension).
///   Multiple files can be separated by commas.
/// - `-L`: List contents only (do not extract)
/// - `-LB`: Brief directory-style output listing
/// - `-N`: Noisy (verbose) output
/// - `-P`: Don't pause execution
/// - `-W`: Treat warnings as errors
///
/// Extraction behavior:
/// 1. By default, creates a folder of the same name as the PBO in the same folder
/// 2. For Arma PBOs, creates subfolders based on detected prefix:
///    - Default: pbo thing.pbo -> thing/prefix/...
///    - With -K: pbo thing.pbo -> destination/thing/...
///    - With -k: pbo thing.pbo -> destination/thing/prefix/...
///
/// Destination paths:
/// - Must include drive letter (relative paths not supported)
/// - Format: ExtractPBO [options] source.pbo D:/destination

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

    fn run_extractpbo_command(&self, args: Vec<&str>, pbo_path: &Path) -> Result<ExtractResult> {
        debug!("Running extractpbo command with args: {:?}", args);
        debug!("PBO path: {:?}", pbo_path);
        
        // Basic path validation
        if !pbo_path.exists() || !pbo_path.is_file() {
            return Err(PboError::InvalidPath(pbo_path.to_path_buf()));
        }
        
        let mut command = Command::new("extractpbo");
        
        // Always add core flags first with proper prefix (universal options in lowercase)
        command.args(["-p", "-w"]); // Don't pause and treat warnings as errors
        
        // Add all option flags first (anything starting with - or +)
        for arg in &args {
            if arg.starts_with('+') || arg.starts_with('-') {
                // Only convert universal options to lowercase
                if arg.ends_with('P') || arg.ends_with('W') || arg.ends_with('?') || arg.ends_with('#') {
                    if arg.starts_with('+') {
                        command.arg(&format!("-{}", &arg[1..].to_lowercase()));
                    } else {
                        command.arg(&arg.to_lowercase());
                    }
                } else {
                    // Keep special options in original case
                    if arg.starts_with('+') {
                        command.arg(&format!("-{}", &arg[1..]));
                    } else {
                        command.arg(arg);
                    }
                }
            }
        }

        // Add PBO path next
        command.arg(pbo_path);

        // Finally add destination path if present (any non-flag argument)
        for arg in &args {
            if !arg.starts_with('-') && !arg.starts_with('+') {
                command.arg(arg);
                break;
            }
        }

        debug!("Full command: {:?}", command);
        
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

        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(ExtractResult {
            return_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: stderr.to_string(),
        })
    }
}

// Keep only the ExtractorClone implementation
impl ExtractorClone for DefaultExtractor {
    fn extract(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        debug!("DefaultExtractor::extract called");
        debug!("PBO path: {:?}", pbo_path);
        debug!("Output dir: {:?}", output_dir);
        debug!("File filter: {:?}", file_filter);
        
        let mut command_args = Vec::new();
        
        if let Some(filter) = file_filter {
            command_args.push(format!("-F={}", filter));
        }
        
        // Ensure output dir has a drive letter
        if let Some(out_str) = output_dir.to_str() {
            if !out_str.contains(':') {
                warn!("Output path should include drive letter: {}", out_str);
                return Err(PboError::InvalidPath(output_dir.to_path_buf()));
            }
            command_args.push(out_str.to_string());
        } else {
            return Err(PboError::InvalidPath(output_dir.to_path_buf()));
        }
        
        let args: Vec<&str> = command_args.iter().map(|s| s.as_str()).collect();
        debug!("Calling run_extractpbo_command with args: {:?}", args);
        self.run_extractpbo_command(args, pbo_path)
    }

    fn list_contents(&self, pbo_path: &Path, brief: bool) -> Result<ExtractResult> {
        debug!("DefaultExtractor::list_contents called");
        debug!("PBO path: {:?}", pbo_path);
        debug!("Brief: {}", brief);
        
        let flag = if brief { "-LB" } else { "-L" };
        let args = vec![flag.to_string()];
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        
        debug!("Calling run_extractpbo_command with flag: {}", flag);
        self.run_extractpbo_command(args_ref, pbo_path)
    }

    fn clone_box(&self) -> Box<dyn ExtractorClone> {
        Box::new(self.clone())
    }
}
