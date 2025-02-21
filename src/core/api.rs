use std::path::Path;
use std::time::Duration;
use std::sync::mpsc::RecvTimeoutError;
use super::PboCore;
use crate::error::types::{Result, PboError};
use super::config::PboConfig;
use crate::extract::ExtractResult;

pub trait PboApi {
    fn list_contents(&self, pbo_path: &Path) -> Result<ExtractResult>;
    fn list_contents_brief(&self, pbo_path: &Path) -> Result<ExtractResult>;
    fn extract_files(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult>;
}

/// API for working with PBO files.
/// 
/// This is the main entry point for the library. Use the builder pattern to create
/// an instance with custom configuration, or use `new()` with default settings.
/// 
/// # Example
/// ```
/// use pbo_tools_rs::PboApiService;
/// 
/// let api = PboApiService::new(30); // 30 second timeout
/// ```
#[derive(Debug)]
pub struct PboApiService {
    core: PboCore,
    timeout: Duration,
}

/// Builder for PboApiService configuration.
#[derive(Default)]
pub struct PboApiBuilder {
    config: Option<PboConfig>,
    timeout: Option<Duration>,
}

impl PboApiBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom PBO configuration
    pub fn with_config(mut self, config: PboConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set operation timeout in seconds
    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout = Some(Duration::from_secs(u64::from(seconds)));
        self
    }

    /// Build the PboApiService instance
    pub fn build(self) -> PboApiService {
        PboApiService {
            core: PboCore::new(self.config),
            timeout: self.timeout.unwrap_or_else(|| Duration::from_secs(30)),
        }
    }
}

impl PboApiService {
    /// Creates a new builder for customizing the API
    pub fn builder() -> PboApiBuilder {
        PboApiBuilder::new()
    }

    /// Creates a new API instance with the specified timeout
    pub fn new(timeout_seconds: u32) -> Self {
        Self::builder()
            .with_timeout(timeout_seconds)
            .build()
    }

    /// Gets the prefix from a PBO file
    pub fn get_prefix<P: AsRef<Path> + Send + Sync + 'static>(&self, pbo_path: P) -> Result<Option<String>> {
        let pbo_path = pbo_path.as_ref().to_owned();
        let core = self.core.clone();
        let result = self.with_timeout(move || core.list_contents(&pbo_path))?;
        Ok(self.core.extract_prefix(&result.stdout))
    }

    fn with_timeout<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let timeout = self.timeout;
        
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });

        rx.recv_timeout(timeout)
            .map_err(|e| match e {
                RecvTimeoutError::Timeout => PboError::Timeout(timeout.as_secs() as u32),
                RecvTimeoutError::Disconnected => PboError::Extraction(
                    crate::error::types::ExtractError::Canceled("Operation was canceled".to_string())
                ),
            })?
    }
}

impl PboApi for PboApiService {
    fn list_contents(&self, pbo_path: &Path) -> Result<ExtractResult> {
        let pbo_path = pbo_path.to_owned();
        let core = self.core.clone();
        self.with_timeout(move || core.list_contents(&pbo_path))
    }

    fn list_contents_brief(&self, pbo_path: &Path) -> Result<ExtractResult> {
        let pbo_path = pbo_path.to_owned();
        let core = self.core.clone();
        self.with_timeout(move || core.list_contents_brief(&pbo_path))
    }

    fn extract_files(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        let pbo_path = pbo_path.to_owned();
        let output_dir = output_dir.to_owned();
        let file_filter = file_filter.map(String::from);
        let core = self.core.clone();
        
        self.with_timeout(move || {
            core.extract_files(&pbo_path, &output_dir, file_filter.as_deref())
        })
    }
}
