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
        let error_indicators = [
            "Error",
            "Failed",
            "Cannot open",
            "Bad Sha",
            "unknown header type",
            "this warning is set as an error",
        ];

        let has_error = error_indicators.iter().any(|&indicator| {
            let in_stderr = self.stderr.contains(indicator);
            let in_stdout = self.stdout.contains(indicator);
            if in_stderr || in_stdout {
                warn!("Found error indicator '{}' in {}", indicator, 
                    if in_stderr { "stderr" } else { "stdout" });
            }
            in_stderr || in_stdout
        });
        debug!("Error indicator check result: {}", has_error);
        has_error
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

            debug!("Adding file from line {}: '{}'", i, line);
            files.push(line.replace('\\', "/"));
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
        ];

        let should_skip = line.is_empty() || skip_patterns.iter().any(|&pattern| line.contains(pattern));
        if should_skip {
            trace!("Skipping line due to pattern match: '{}'", line);
        }
        should_skip
    }

    fn extract_filename(&self, line: &str) -> Option<String> {
        // Extract filename from line, handling both brief and detailed formats
        let file_part = if line.contains(':') {
            // Detailed format: "filename:timestamp: size bytes"
            line.split(':').next()
        } else {
            // Brief format: just "filename"
            Some(line)
        };

        file_part.and_then(|part| {
            if part.is_empty() {
                None
            } else {
                Some(part.replace('\\', "/").trim().to_string())
            }
        })
    }

    pub fn get_prefix(&self) -> Option<String> {
        debug!("Searching for prefix in stdout (length: {})", self.stdout.len());
        trace!("Full stdout content:\n{}", self.stdout);
        
        // Always return Some with at least an empty string if we find a prefix line
        self.stdout
            .lines()
            .find(|line| line.starts_with("prefix="))
            .map(|_| String::new())
    }

    pub fn get_error_message(&self) -> Option<String> {
        if (!self.is_success()) {
            let msg = if !self.stderr.is_empty() {
                debug!("Using stderr for error message: '{}'", self.stderr);
                self.stderr.clone()
            } else {
                let msg = format!("Command failed with return code {}", self.return_code);
                debug!("No stderr, using return code message: '{}'", msg);
                msg
            };
            Some(msg)
        } else {
            None
        }
    }
}

impl fmt::Display for ExtractResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_success() {
            write!(f, "{}", self.stdout)
        } else {
            write!(f, "Error ({}): {}", 
                self.return_code, 
                self.get_error_message().unwrap_or_else(|| "Unknown error".to_string())
            )
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
