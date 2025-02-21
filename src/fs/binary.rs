use std::path::{Path, PathBuf};
use encoding_rs::WINDOWS_1252;
use crate::error::types::{PboError, FileSystemError, Result};

#[derive(Debug)]
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
        if let Ok(text) = String::from_utf8(self.0.clone()) {
            return Ok(text);
        }

        if self.appears_binary() {
            return Err(PboError::Encoding {
                context: "Content appears to be binary data".to_string(),
                path: PathBuf::new(),
            });
        }

        let (cow, _, had_errors) = WINDOWS_1252.decode(&self.0);
        if had_errors {
            return Err(PboError::Encoding {
                context: "Content contains invalid character sequences".to_string(),
                path: PathBuf::new(),
            });
        }

        Ok(cow.into_owned())
    }

    fn appears_binary(&self) -> bool {
        if self.0.is_empty() {
            return false;
        }

        let sample_size = std::cmp::min(512, self.0.len());
        let null_byte_threshold = 0.3;
        let null_bytes = self.0[..sample_size]
            .iter()
            .filter(|&&b| b == 0)
            .count();

        (null_bytes as f64 / sample_size as f64) > null_byte_threshold ||
            self.0.windows(2).any(|w| w == [0xFF, 0xFF])
    }
}

impl AsRef<[u8]> for BinaryContent {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub trait ReadBinaryContent {
    fn read_content(&self) -> Result<String>;
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
}
