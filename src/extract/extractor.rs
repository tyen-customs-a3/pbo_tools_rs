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

    fn run_extractpbo_command(&self, args: &[&str], pbo_path: &Path) -> Result<ExtractResult> {
        debug!("Running extractpbo command with args: {:?}", args);
        debug!("PBO path: {:?}", pbo_path);
        
        let output = Command::new("extractpbo")
            .args(args)
            .arg(pbo_path)
            .output()
            .map_err(|e| {
                debug!("Command failed with error: {}", e);
                PboError::Extraction(ExtractError::CommandFailed {
                    cmd: "extractpbo".to_string(),
                    reason: e.to_string(),
                })
            })?;

        debug!("Command completed with status: {:?}", output.status);
        debug!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        debug!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

        Ok(ExtractResult {
            return_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

impl ExtractorClone for DefaultExtractor {
    fn extract(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        let mut args = vec!["-P"];  // Don't pause
        
        if let Some(filter) = file_filter {
            args.push("-F");
            args.push(filter);
        }
        
        args.push(pbo_path.to_str().unwrap());
        args.push(output_dir.to_str().unwrap());

        self.run_extractpbo_command(&args, pbo_path)
    }

    fn list_contents(&self, pbo_path: &Path, brief: bool) -> Result<ExtractResult> {
        let args = if brief {
            vec!["-P", "-LB"]  // Don't pause, Brief listing
        } else {
            vec!["-P", "-L"]   // Don't pause, Normal listing
        };
        
        self.run_extractpbo_command(&args, pbo_path)
    }

    fn clone_box(&self) -> Box<dyn ExtractorClone> {
        Box::new(self.clone())
    }
}
