use std::path::Path;
use walkdir;
use crate::config::PboConfig;
use crate::errors::{PboError, ExtractError, FileSystemError, Result};
use crate::extract::{Extractor, DefaultExtractor, ExtractResult, list_contents};
use crate::file_ops::TempFileManager;

#[derive(Debug, Clone)]
pub struct PboConfig {
    bin_file_types: HashMap<String, String>,
    bad_pbo_indicators: Vec<String>,
}

impl PboConfig {
    pub fn builder() -> PboConfigBuilder {
        PboConfigBuilder::new()
    }

    pub fn get_bin_extension(&self, filename: &str) -> Option<&str> {
        self.bin_file_types.get(filename).map(|s| s.as_str())
    }

    pub fn is_bad_pbo(&self, message: &str) -> bool {
        self.bad_pbo_indicators.iter().any(|i| message.contains(i))
    }
}

#[derive(Default)]
pub struct PboConfigBuilder {
    bin_file_types: HashMap<String, String>,
    bad_pbo_indicators: Vec<String>,
}

impl PboConfigBuilder {
    pub fn new() -> Self {
        let mut builder = Self::default();
        // Set default mappings
        builder.bin_file_types.insert("config.bin".to_string(), "config.cpp".to_string());
        builder.bin_file_types.insert("model.bin".to_string(), "model.cfg".to_string());
        builder.bin_file_types.insert("stringtable.bin".to_string(), "stringtable.xml".to_string());
        builder.bin_file_types.insert("texheaders.bin".to_string(), "texheaders.txt".to_string());
        builder.bin_file_types.insert("script.bin".to_string(), "script.cpp".to_string());
        builder.bin_file_types.insert("default".to_string(), ".txt".to_string());

        // Set default bad PBO indicators
        builder.bad_pbo_indicators = vec![
            "DePbo:Pbo unknown header type",
            "Bad Sha detected",
            "Bad Sha",
            "this warning is set as an error",
        ].into_iter().map(String::from).collect();

        builder
    }

    pub fn add_bin_mapping(mut self, bin_file: impl Into<String>, target_ext: impl Into<String>) -> Self {
        self.bin_file_types.insert(bin_file.into(), target_ext.into());
        self
    }

    pub fn add_bad_indicator(mut self, indicator: impl Into<String>) -> Self {
        self.bad_pbo_indicators.push(indicator.into());
        self
    }

    pub fn build(self) -> PboConfig {
        PboConfig {
            bin_file_types: self.bin_file_types,
            bad_pbo_indicators: self.bad_pbo_indicators,
        }
    }
}

impl Default for PboConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

pub struct PboCore {
    temp_manager: TempFileManager,
    config: PboConfig,
    extractor: Box<dyn Extractor>,
}

impl PboCore {
    pub fn new(config: Option<PboConfig>) -> Self {
        Self {
            temp_manager: TempFileManager::new(),
            config: config.unwrap_or_default(),
            extractor: Box::new(DefaultExtractor::new()),
        }
    }

    pub fn list_contents(&self, pbo_path: &Path) -> Result<ExtractResult> {
        self.validate_pbo_exists(pbo_path)?;
        list_contents(pbo_path)
    }

    pub fn extract_files(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        self.validate_pbo(pbo_path)?;
        self.prepare_output_dir(output_dir, file_filter)?;
        
        let result = self.extractor.extract(pbo_path, output_dir, file_filter)?;
        
        if result.return_code == 0 {
            self.validate_extraction(output_dir)?;
            self.process_extracted_bins(output_dir)?;
        } else if self.config.is_bad_pbo(&result.stderr) {
            return Err(PboError::InvalidPbo(format!("Bad PBO format detected: {}", result.stderr)));
        }

        Ok(result)
    }

    fn validate_pbo_exists(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(PboError::InvalidPath(path.to_path_buf()));
        }
        Ok(())
    }

    fn validate_pbo(&self, path: &Path) -> Result<()> {
        self.validate_pbo_exists(path)?;
        let metadata = std::fs::metadata(path)
            .map_err(|e| PboError::Io(e))?;
        
        if metadata.len() < 8 {
            return Err(PboError::InvalidPbo("File too small to be a valid PBO".to_string()));
        }
        Ok(())
    }

    fn prepare_output_dir(&self, output_dir: &Path, file_filter: Option<&str>) -> Result<()> {
        std::fs::create_dir_all(output_dir)
            .map_err(|e| PboError::Io(e))?;

        if let Some(filter) = file_filter {
            let target = output_dir.join(filter);
            if target.exists() {
                std::fs::remove_file(&target)
                    .map_err(|e| PboError::Io(e))?;
            }
        }
        Ok(())
    }

    fn validate_extraction(&self, output_dir: &Path) -> Result<()> {
        let mut has_files = false;
        for entry in walkdir::WalkDir::new(output_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                has_files = true;
                break;
            }
        }
        
        if !has_files {
            return Err(PboError::Extraction(ExtractError::NoFiles));
        }
        Ok(())
    }

    fn process_extracted_bins(&self, output_dir: &Path) -> Result<()> {
        for entry in walkdir::WalkDir::new(output_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "bin"))
        {
            if let Some(new_name) = self.detect_bin_type(entry.path()) {
                let new_path = entry.path().with_file_name(new_name);
                std::fs::rename(entry.path(), &new_path)
                    .map_err(|e| PboError::FileSystem(FileSystemError::Rename {
                        from: entry.path().to_path_buf(),
                        to: new_path,
                        reason: e.to_string(),
                    }))?;
            }
        }
        Ok(())
    }

    fn detect_bin_type(&self, path: &Path) -> Option<String> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_lowercase())
            .and_then(|name| {
                self.config.get_bin_extension(&name).map(|ext| ext.to_string()).or_else(|| {
                    if name.ends_with(".bin") {
                        Some(format!("{}{}", 
                            name.trim_end_matches(".bin"),
                            self.config.get_bin_extension("default").unwrap()))
                    } else {
                        None
                    }
                })
            })
    }

    pub fn extract_prefix(&self, stdout: &str) -> Option<String> {
        stdout.lines()
            .find(|line| line.starts_with("prefix="))
            .map(|line| {
                line.split('=')
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches(';')
                    .replace('\\', "/")
            })
    }
}