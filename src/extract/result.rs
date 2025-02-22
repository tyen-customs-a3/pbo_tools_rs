use std::fmt;
use log::debug;
use crate::error::types::{Result, PboError, ExtractError};

#[derive(Debug)]
pub struct ExtractResult {
    pub return_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ExtractResult {
    pub fn is_success(&self) -> bool {
        self.return_code == 0 && !self.has_error_indicators()
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

        error_indicators.iter().any(|&indicator| {
            self.stderr.contains(indicator) || self.stdout.contains(indicator)
        })
    }

    pub fn get_file_list(&self) -> Vec<String> {
        let mut files = Vec::new();
        debug!("Processing stdout for file list:\n{}", self.stdout);

        for line in self.stdout.lines() {
            debug!("Processing line: {}", line);
            
            if self.should_skip_line(line) {
                debug!("Skipping metadata line: {}", line);
                continue;
            }

            if let Some(file) = self.extract_filename(line) {
                debug!("Found file: {}", file);
                files.push(file);
            }
        }
        
        debug!("Final file list: {:?}", files);
        files
    }

    fn should_skip_line(&self, line: &str) -> bool {
        let skip_patterns = [
            "",
            "Active code page:",
            "ExtractPbo Version",
            "Opening",
            "prefix=",
            "==",
            "//",
            "Mikero=",
            "version=",
            "PboType=",
        ];

        line.is_empty() || skip_patterns.iter().any(|&pattern| line.starts_with(pattern))
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
                Some(part.replace('/', "\\").trim().to_string())
            }
        })
    }

    pub fn get_prefix(&self) -> Option<String> {
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
            Some(if !self.stderr.is_empty() {
                self.stderr.clone()
            } else {
                format!("Command failed with return code {}", self.return_code)
            })
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
        assert!(files.contains(&"data\\test.paa".to_string()));
        assert!(files.contains(&"models\\model.p3d".to_string()));
    }
}
