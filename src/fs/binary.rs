use std::path::{Path, PathBuf};
use std::io::Write;
use encoding_rs::{WINDOWS_1252, UTF_8};
use log::debug;
use crate::error::types::{PboError, FileSystemError, Result};

#[derive(Debug, Clone)]
pub struct BinaryContent(Vec<u8>);

impl BinaryContent {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read(path)
            .map_err(|e| PboError::FileSystem(FileSystemError::Read {
                path: path.to_path_buf(),
                reason: e.to_string(),
            }))?;
        Ok(Self(content))
    }

    pub fn decode_text(&self) -> Result<String> {
        // First try UTF-8 with BOM detection
        if self.0.len() >= 3 && self.0[0..3] == [0xEF, 0xBB, 0xBF] {
            return String::from_utf8(self.0[3..].to_vec())
                .map_err(|_| PboError::Encoding {
                    context: "Invalid UTF-8 with BOM".to_string(),
                    path: PathBuf::new(),
                });
        }

        // Try UTF-8 without BOM
        let text = UTF_8.decode(&self.0).0.into_owned();
        if !text.contains('\0') {
            return Ok(text);
        }

        // Then try Windows-1252 if it's not binary
        if (!self.appears_binary()) {
            let (cow, _, had_errors) = WINDOWS_1252.decode(&self.0);
            if !had_errors && !cow.contains('\0') {
                return Ok(cow.into_owned());
            }
        }

        Err(PboError::Encoding {
            context: if self.appears_binary() {
                "Content appears to be binary data".to_string()
            } else {
                "Content contains invalid character sequences".to_string()
            },
            path: PathBuf::new(),
        })
    }

    fn appears_binary(&self) -> bool {
        if self.0.is_empty() {
            return false;
        }

        // Statistical analysis for binary content
        let sample_size = std::cmp::min(1024, self.0.len());
        let null_byte_threshold = 0.1;  // Lower threshold to catch more binary files
        let control_char_threshold = 0.15;
        let byte_distribution_threshold = 0.7; // High concentration of specific byte ranges

        let mut byte_counts = [0u32; 256];
        let mut null_bytes = 0;
        let mut control_chars = 0;

        // Count byte frequencies
        for &byte in &self.0[..sample_size] {
            byte_counts[byte as usize] += 1;
            if byte == 0 {
                null_bytes += 1;
            } else if (byte < 32 && byte != 9 && byte != 10 && byte != 13) || byte == 127 {
                control_chars += 1;
            }
        }

        // Check for byte distribution anomalies that suggest binary content
        let max_byte_freq = *byte_counts.iter().max().unwrap_or(&0) as f64 / sample_size as f64;
        if max_byte_freq > byte_distribution_threshold {
            debug!("Binary content detected: unusual byte distribution");
            return true;
        }

        // Check null bytes and control characters
        let null_byte_ratio = null_bytes as f64 / sample_size as f64;
        let control_char_ratio = control_chars as f64 / sample_size as f64;

        if null_byte_ratio > null_byte_threshold {
            debug!("Binary content detected: high null byte ratio ({})", null_byte_ratio);
            return true;
        }

        if control_char_ratio > control_char_threshold {
            debug!("Binary content detected: high control char ratio ({})", control_char_ratio);
            return true;
        }

        // Check for UTF-16/UTF-32 BOMs
        if self.0.windows(2).any(|w| w == [0xFF, 0xFE] || w == [0xFE, 0xFF]) {
            debug!("Binary content detected: UTF-16 BOM");
            return true;
        }

        false
    }

    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        std::fs::write(path, &self.0)
            .map_err(|e| PboError::FileSystem(FileSystemError::Write {
                path: path.to_path_buf(),
                reason: e.to_string(),
            }))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }
}

impl AsRef<[u8]> for BinaryContent {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for BinaryContent {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}

pub trait ReadBinaryContent {
    fn read_content(&self) -> Result<String>;
    fn read_binary(&self) -> Result<BinaryContent>;
}

impl ReadBinaryContent for Path {
    fn read_content(&self) -> Result<String> {
        let content = BinaryContent::from_file(self)?;
        content.decode_text().map_err(|e| match e {
            PboError::Encoding { context, .. } => PboError::Encoding {
                context,
                path: self.to_path_buf(),
            },
            _ => e,
        })
    }

    fn read_binary(&self) -> Result<BinaryContent> {
        BinaryContent::from_file(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_binary_detection() {
        let temp_dir = tempdir().unwrap();

        // Test UTF-8 text
        let text_path = temp_dir.path().join("text.txt");
        File::create(&text_path).unwrap().write_all(b"Hello, world!").unwrap();
        let content = BinaryContent::from_file(&text_path).unwrap();
        assert!(!content.appears_binary());

        // Test binary data
        let bin_path = temp_dir.path().join("binary.bin");
        let binary_data: Vec<u8> = (0..255).collect();
        File::create(&bin_path).unwrap().write_all(&binary_data).unwrap();
        let content = BinaryContent::from_file(&bin_path).unwrap();
        assert!(content.appears_binary());
    }

    #[test]
    fn test_encoding_detection() {
        let temp_dir = tempdir().unwrap();

        // UTF-8 with BOM
        let utf8_path = temp_dir.path().join("utf8_bom.txt");
        let bom = [0xEF, 0xBB, 0xBF];
        let mut file = File::create(&utf8_path).unwrap();
        file.write_all(&bom).unwrap();
        file.write_all(b"Hello").unwrap();
        let content = BinaryContent::from_file(&utf8_path).unwrap();
        assert!(content.decode_text().is_ok());

        // Windows-1252 text
        let win_path = temp_dir.path().join("windows.txt");
        let win_text = vec![0xC4, 0xD6, 0xDC]; // Ä Ö Ü in Windows-1252
        File::create(&win_path).unwrap().write_all(&win_text).unwrap();
        let content = BinaryContent::from_file(&win_path).unwrap();
        assert!(content.decode_text().is_ok());
    }

    #[test]
    fn test_memory_management() {
        let data = vec![0u8; 1000];
        let mut content = BinaryContent::from(data);
        assert_eq!(content.capacity(), 1000);
        content.shrink_to_fit();
        assert_eq!(content.capacity(), content.len());
    }
}
