use std::fmt;
use log::debug;

#[derive(Debug)]
pub struct ExtractResult {
    pub return_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ExtractResult {
    pub fn is_success(&self) -> bool {
        self.return_code == 0
    }

    pub fn get_file_list(&self) -> Vec<String> {
        let mut files = Vec::new();
        debug!("Processing stdout for file list:\n{}", self.stdout);

        for line in self.stdout.lines() {
            debug!("Processing line: {}", line);
            
            // Skip empty lines and header/metadata sections
            if line.is_empty() || 
               line.starts_with("Active code page:") || 
               line.starts_with("ExtractPbo Version") || 
               line.starts_with("Opening") ||
               line.starts_with("prefix=") || 
               line.starts_with("==") ||
               line.starts_with("//") ||
               line.starts_with("Mikero=") ||
               line.starts_with("version=") ||
               line.starts_with("PboType=") {
                debug!("Skipping header/metadata line: {}", line);
                continue;
            }

            // Extract filename from lines, handling both brief and detailed formats
            let file_part = if line.contains(':') {
                // Detailed format: "filename:timestamp: size bytes"
                line.split(':').next()
            } else {
                // Brief format: just "filename"
                Some(line)
            };

            if let Some(file_part) = file_part {
                if !file_part.is_empty() {
                    let normalized = file_part.replace('/', "\\").trim().to_string();
                    debug!("Found file: {} (normalized from {})", normalized, file_part);
                    files.push(normalized);
                }
            }
        }
        
        debug!("Final file list: {:?}", files);
        files
    }

    pub fn get_prefix(&self) -> Option<String> {
        self.stdout
            .lines()
            .find(|line| line.starts_with("prefix="))
            .and_then(|line| line.split('=').nth(1))
            .map(|prefix| prefix.trim().trim_end_matches(';').to_string())
    }
}

impl fmt::Display for ExtractResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_success() {
            write!(f, "{}", self.stdout)
        } else {
            write!(f, "Error ({}): {}", self.return_code, self.stderr)
        }
    }
}
