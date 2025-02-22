use std::path::Path;
use std::fmt::Debug;
use std::process::Command;
use log::{debug, warn};
use crate::error::{PboError, ExtractError, Result};
use super::result::ExtractResult;

/// ExtractPBO Command Line Interface Documentation
/// 
/// Syntax: extractpbo [-options...] PboName[.pbo|.xbo|.ifa]|FolderName|Extraction.lst|.txt  [destination]
///
/// Important: The order of arguments matters!
/// 1. Options must come before the PBO path
/// 2. PBO path must come before destination path
/// 3. Options can be catenated together (e.g. -PW instead of -P -W)
///
/// Arguments (in order):
/// - Options: All options must start with - or +
///   - `-P`: Don't pause execution
///   - `-W`: Treat warnings as errors
///   - `-F=filelist[,...]`: Extract specific file(s). Files are extracted to their correct position 
///     in the output folder tree. Supports basic wildcards (*.ext for all files with extension).
///     Multiple files can be separated by commas.
///   - `-L`: List contents only (do not extract)
///   - `-LB`: Brief directory-style output listing
///   - `-N`: Noisy (verbose) output
/// - PBO Path: Path to the source PBO file
/// - Destination Path: Optional output directory path. Must include drive letter.
///
/// Examples:
/// ```text
/// extractpbo -PW source.pbo D:/output           # Extract all files
/// extractpbo -PW -F=*.paa source.pbo D:/output  # Extract only .paa files
/// extractpbo -L source.pbo                      # List contents
/// ```
///
/// Extraction behavior:
/// 1. By default, creates a folder of the same name as the PBO in the destination
/// 2. For Arma PBOs, creates additional subfolders based on the detected prefix
///    Example: source.pbo -> destination/prefix/...
///
/// Notes:
/// - Destination paths MUST include a drive letter (relative paths not supported)
/// - The -F option's pattern is applied to the full file path within the PBO
/// - Error codes and output messages are used to determine operation success

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

    /// Execute the extractpbo command following the strict argument order:
    /// 1. Core options (-PW)
    /// 2. Operation-specific options (-F=pattern, -L, etc)
    /// 3. PBO path
    /// 4. Destination path (if any)
    fn run_extractpbo_command(&self, args: Vec<&str>, pbo_path: &Path) -> Result<ExtractResult> {
        debug!("Running extractpbo command with args: {:?}", args);
        debug!("PBO path: {:?}", pbo_path);
        
        let mut command = Command::new("extractpbo");
        
        // 1. Core options first (always used)
        command.arg("-PW");  // Combined: Don't pause (-P) and treat warnings as errors (-W)
        
        // 2. Operation-specific options (like -F=pattern or -L)
        let mut has_options = false;
        for arg in &args {
            if arg.starts_with('-') {
                command.arg(arg);
                has_options = true;
            }
        }
        if has_options {
            debug!("Added operation-specific options");
        }

        // 3. PBO path (required)
        if let Some(pbo_str) = pbo_path.to_str() {
            command.arg(pbo_str.replace("\\\\?\\", ""));
            debug!("Added PBO path");
        } else {
            return Err(PboError::InvalidPath(pbo_path.to_path_buf()));
        }

        // 4. Destination path (if any non-flag args remain)
        let mut added_dest = false;
        for arg in &args {
            if !arg.starts_with('-') {
                command.arg(arg);
                added_dest = true;
                debug!("Added destination path");
                break; // Only add the first non-flag argument as destination
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

impl ExtractorClone for DefaultExtractor {
    fn extract(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        debug!("DefaultExtractor::extract called");
        debug!("PBO path: {:?}", pbo_path);
        debug!("Output dir: {:?}", output_dir);
        debug!("File filter: {:?}", file_filter);
        
        // Create output directory if it doesn't exist
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir).map_err(|_e| PboError::InvalidPath(output_dir.to_path_buf()))?;
        }
        
        let mut args = Vec::new();
        
        // Add filter if present
        if let Some(filter) = file_filter {
            args.push(format!("-F={}", filter));
        }
        
        // Add output directory
        if let Some(out_str) = output_dir.canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.replace("\\\\?\\", "")))
        {
            args.push(out_str);
        } else {
            return Err(PboError::InvalidPath(output_dir.to_path_buf()));
        }
        
        // Convert args to string slices for command
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
        debug!("Calling run_extractpbo_command with args: {:?}", args);
        self.run_extractpbo_command(args, pbo_path)
    }

    fn list_contents(&self, pbo_path: &Path, brief: bool) -> Result<ExtractResult> {
        debug!("DefaultExtractor::list_contents called");
        debug!("PBO path: {:?}", pbo_path);
        debug!("Brief: {}", brief);
        
        let args = if brief { vec!["-LB"] } else { vec!["-L"] };
        debug!("Calling run_extractpbo_command with args: {:?}", args);
        self.run_extractpbo_command(args, pbo_path)
    }

    fn clone_box(&self) -> Box<dyn ExtractorClone> {
        Box::new(self.clone())
    }
}
