use std::fmt;
use std::path::Path;
use std::process::Command;
use crate::errors::{PboError, ExtractError};

#[derive(Debug)]
pub struct ExtractResult {
    pub return_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl fmt::Display for ExtractResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.return_code == 0 {
            write!(f, "{}", self.stdout)
        } else {
            write!(f, "Error ({}): {}", self.return_code, self.stderr)
        }
    }
}

pub trait Extractor {
    fn extract(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult, PboError>;
}

pub struct DefaultExtractor;

impl DefaultExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl Extractor for DefaultExtractor {
    fn extract(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult, PboError> {
        let mut cmd = Command::new("extractpbo")
            .or_else(|_| Err(PboError::CommandNotFound("extractpbo".into())))?;

        cmd.args(["-S", "-P", "-Y"]);
        
        if let Some(filter) = file_filter {
            cmd.arg(format!("-F={}", filter));
        }
        
        cmd.args([pbo_path.as_os_str(), output_dir.as_os_str()]);

        let output = cmd.output()
            .map_err(|e| PboError::Extraction(ExtractError::CommandFailed {
                cmd: "extractpbo".into(),
                reason: e.to_string(),
            }))?;

        Ok(ExtractResult {
            return_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

pub fn list_contents(pbo_path: &Path) -> Result<ExtractResult, PboError> {
    let output = Command::new("extractpbo")
        .args(["-LBP", &pbo_path.to_string_lossy()])
        .output()
        .map_err(|e| PboError::Extraction(ExtractError::CommandFailed {
            cmd: "extractpbo".to_string(),
            reason: e.to_string(),
        }))?;

    Ok(ExtractResult {
        return_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}