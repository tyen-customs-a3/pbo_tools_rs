use std::fmt;
use log::{debug, trace, warn};
use crate::error::types::{Result, PboError, ExtractError};

#[derive(Debug)]
pub struct ExtractResult {
    pub return_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ExtractResult {
    pub fn is_success(&self) -> bool {
        let return_code_ok = self.return_code == 0;
        let no_errors = !self.has_error_indicators();
        debug!("Checking extraction success - return code: {}, has errors: {}", self.return_code, !no_errors);
        return_code_ok && no_errors
    }

    fn has_error_indicators(&self) -> bool {
        let known_warnings = [
            "1st/last entry has non-zero real_size",
            "reserved field non zero",
            "no shakey on arma",
            "arma pbo is missing a prefix",
        ];

        let error_indicators = [
            "Error",
            "Failed",
            "Cannot open",
            "Bad Sha",
            "unknown header type",
            "this warning is set as an error",
            "residual bytes in file",
        ];

        let mut is_error = false;
        
        // Check for warnings first
        for warning in &known_warnings {
            if self.stderr.contains(warning) || self.stdout.contains(warning) {
                debug!("Found known warning: {}", warning);
                // These are just warnings, don't fail the operation
            }
        }

        // Then check for actual errors
        for indicator in &error_indicators {
            if self.stderr.contains(indicator) || self.stdout.contains(indicator) {
                warn!("Found error indicator: {}", indicator);
                is_error = true;
                break;
            }
        }

        debug!("Error indicator check result: {}", is_error);
        is_error
    }

    pub fn get_file_list(&self) -> Vec<String> {
        let mut files = Vec::new();
        debug!("Processing stdout for file list, stdout length: {}", self.stdout.len());
        trace!("Stdout contents:\n{}", self.stdout);
        trace!("Stderr contents:\n{}", self.stderr);

        for (i, line) in self.stdout.lines().enumerate() {
            let line = line.trim();
            trace!("Processing line {}: '{}'", i, line);
            
            if line.is_empty() {
                trace!("Skipping empty line {}", i);
                continue;
            }
            
            if self.should_skip_line(line) {
                debug!("Skipping metadata line {}: '{}'", i, line);
                continue;
            }

            if let Some(file) = self.extract_filename(line) {
                debug!("Adding file from line {}: '{}'", i, file);
                files.push(file);
            }
        }
        
        debug!("Final file list ({} files): {:?}", files.len(), files);
        files
    }

    fn should_skip_line(&self, line: &str) -> bool {
        let skip_patterns = [
            "Active code page:",
            "ExtractPbo Version",
            "Opening pbo archive",
            "prefix=",
            "Mikero=",
            "version=",
            "PboType=",
            "===",
            "//",
            "Created by",
            "Author:",
            "BinPatches=",
            "ReportInvalidFiles=",
            "SearchForBinFiles=",
        ];

        let should_skip = line.is_empty() || skip_patterns.iter().any(|&pattern| line.contains(pattern));
        if should_skip {
            trace!("Skipping line due to pattern match: '{}'", line);
        }
        should_skip
    }

    fn extract_filename(&self, line: &str) -> Option<String> {
        // Extract filename, handling different formats:
        // 1. Brief format: just "filename"
        // 2. Detailed format: "filename:timestamp: size bytes"
        // 3. Extracted format: "Extracting filename..."
        if line.starts_with("Extracting ") {
            line.strip_prefix("Extracting ")
                .map(|s| s.trim_end_matches("...").trim().replace('\\', "/"))
        } else if line.contains(':') {
            // Detailed format
            line.split(':')
                .next()
                .map(|s| s.trim().replace('\\', "/"))
        } else {
            // Brief format
            Some(line.replace('\\', "/"))
        }
        .filter(|s| !s.is_empty())
    }

    pub fn get_prefix(&self) -> Option<String> {
        debug!("Searching for prefix in stdout (length: {})", self.stdout.len());
        trace!("Full stdout content:\n{}", self.stdout);
        
        self.stdout
            .lines()
            .find(|line| line.starts_with("prefix="))
            .and_then(|line| {
                line.split('=')
                    .nth(1)
                    .map(|prefix| prefix.trim().trim_end_matches(';').to_string())
            })
            .filter(|prefix| !prefix.is_empty())
    }

    pub fn get_error_message(&self) -> Option<String> {
        if !self.is_success() {
            let mut msg = String::new();
            
            // Add stderr if not empty
            if !self.stderr.is_empty() {
                msg.push_str(&self.stderr);
            }
            
            // Add return code if non-zero
            if self.return_code != 0 {
                if !msg.is_empty() {
                    msg.push_str("\n");
                }
                msg.push_str(&format!("Command failed with return code {}", self.return_code));
            }
            
            // Use a default message if we have nothing else
            if msg.is_empty() {
                msg = "Unknown error occurred".to_string();
            }
            
            debug!("Error message: {}", msg);
            Some(msg)
        } else {
            None
        }
    }
}

impl fmt::Display for ExtractResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stdout = self.stdout.trim();
        let stderr = self.stderr.trim();
        
        if self.is_success() {
            // If we have actual content, display it
            if !stdout.is_empty() {
                write!(f, "{}", stdout)?;
            }
            
            // Show any non-error stderr output (like warnings) if present
            if !stderr.is_empty() {
                if !stdout.is_empty() {
                    write!(f, "\n")?;
                }
                write!(f, "{}", stderr)?;
            }
            
            Ok(())
        } else {
            // For errors, show both stdout and stderr if they contain unique information
            let mut output = String::new();
            
            if !stdout.is_empty() {
                output.push_str(stdout);
            }
            
            if !stderr.is_empty() {
                if !output.is_empty() {
                    output.push_str("\n");
                }
                output.push_str(stderr);
            }
            
            if output.is_empty() {
                write!(f, "Error ({}): Unknown error", self.return_code)
            } else {
                write!(f, "Error ({}): {}", self.return_code, output)
            }
        }
    }
}

pub trait ResultProcessor {
    fn process_output(&self) -> Result<Vec<String>>;
    fn process_prefix(&self) -> Option<String>;
}

impl ResultProcessor for ExtractResult {
    fn process_output(&self) -> Result<Vec<String>> {
        if !self.is_success() {
            return Err(PboError::Extraction(ExtractError::CommandFailed {
                cmd: "extractpbo".to_string(),
                reason: self.get_error_message()
                    .unwrap_or_else(|| "Unknown error".to_string()),
            }));
        }

        let files = self.get_file_list();
        if files.is_empty() {
            return Err(PboError::Extraction(ExtractError::NoFiles));
        }

        Ok(files)
    }

    fn process_prefix(&self) -> Option<String> {
        self.get_prefix()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_detection() {
        let result = ExtractResult {
            return_code: 0,
            stdout: String::new(),
            stderr: "Bad Sha detected".to_string(),
        };
        assert!(!result.is_success());

        let result = ExtractResult {
            return_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(result.is_success());
    }

    #[test]
    fn test_file_list_parsing() {
        let result = ExtractResult {
            return_code: 0,
            stdout: "config.bin\ndata/test.paa\nmodels/model.p3d".to_string(),
            stderr: String::new(),
        };
        
        let files = result.get_file_list();
        assert_eq!(files.len(), 3);
        assert!(files.contains(&"config.bin".to_string()));
        assert!(files.contains(&"data/test.paa".to_string()));
        assert!(files.contains(&"models/model.p3d".to_string()));
    }
}
