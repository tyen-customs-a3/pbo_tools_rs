use std::path::Path;
use super::config::PboConfig;
use crate::error::types::{PboError, Result};
use crate::extract::{ExtractorClone, DefaultExtractor, ExtractResult, ExtractOptions};
use crate::fs::TempFileManager;
use super::api::PboApiOps;

#[derive(Debug, Clone)]
pub struct PboCore {
    temp_manager: TempFileManager,
    config: PboConfig,
    extractor: Box<dyn ExtractorClone>,
}

impl PboCore {
    pub fn new(config: Option<PboConfig>) -> Self {
        Self {
            temp_manager: TempFileManager::new(),
            config: config.unwrap_or_default(),
            extractor: Box::new(DefaultExtractor::new()),
        }
    }

    fn validate_pbo_exists(&self, pbo_path: &Path) -> Result<()> {
        if !pbo_path.exists() {
            return Err(PboError::InvalidPath(pbo_path.to_path_buf()));
        }
        Ok(())
    }

    pub fn extract_prefix(&self, output: &str) -> Option<String> {
        output
            .lines()
            .find(|line| line.starts_with("prefix="))
            .map(|line| line.split('=').nth(1).unwrap_or("").trim().trim_end_matches(';').to_string())
    }
}

impl PboApiOps for PboCore {
    fn list_contents(&self, pbo_path: &Path) -> Result<ExtractResult> {
        self.validate_pbo_exists(pbo_path)?;
        let options = ExtractOptions {
            no_pause: true,
            warnings_as_errors: true,
            ..Default::default()
        };
        self.extractor.list_with_options(pbo_path, options)
    }

    fn list_contents_brief(&self, pbo_path: &Path) -> Result<ExtractResult> {
        self.validate_pbo_exists(pbo_path)?;
        let options = ExtractOptions {
            no_pause: true,
            warnings_as_errors: true,
            brief_listing: true,
            ..Default::default()
        };
        self.extractor.list_with_options(pbo_path, options)
    }

    fn extract_files(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        self.validate_pbo_exists(pbo_path)?;
        let options = ExtractOptions {
            no_pause: true,
            warnings_as_errors: true,
            file_filter: file_filter.map(String::from),
            ..Default::default()
        };
        self.extractor.extract_with_options(pbo_path, output_dir, options)
    }

    fn list_with_options(&self, pbo_path: &Path, options: ExtractOptions) -> Result<ExtractResult> {
        self.validate_pbo_exists(pbo_path)?;
        self.extractor.list_with_options(pbo_path, options)
    }

    fn extract_with_options(&self, pbo_path: &Path, output_dir: &Path, options: ExtractOptions) -> Result<ExtractResult> {
        self.validate_pbo_exists(pbo_path)?;
        self.extractor.extract_with_options(pbo_path, output_dir, options)
    }
}