use std::path::Path;
use std::time::Duration;
use std::sync::mpsc::RecvTimeoutError;
use crate::core::{PboCore, PboConfig};
use crate::errors::{Result, PboError};

/// API for working with PBO files.
/// 
/// This is the main entry point for the library. Use the builder pattern to create
/// an instance with custom configuration, or use `new()` with default settings.
/// 
/// # Example
/// ```
/// use pbo_tools_rs::PboApi;
/// 
/// let api = PboApi::new(30); // 30 second timeout
/// ```
#[derive(Debug)]
pub struct PboApi {
    core: PboCore,
    timeout: Duration,
}

/// Builder for PboApi configuration.
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

    /// Build the PboApi instance
    pub fn build(self) -> PboApi {
        PboApi {
            core: PboCore::new(self.config),
            timeout: self.timeout.unwrap_or_else(|| Duration::from_secs(30)),
        }
    }
}

impl PboApi {
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

    /// Lists contents of a PBO file
    /// 
    /// Returns a tuple of (success, content) where content is either the file listing
    /// or error message if success is false.
    pub fn list_contents<P: AsRef<Path> + Send + Sync + 'static>(&self, pbo_path: P) -> Result<(bool, String)> {
        let result = self.with_timeout(|| self.core.list_contents(pbo_path.as_ref()))?;
        Ok((
            result.return_code == 0,
            if result.return_code == 0 { result.stdout } else { result.stderr },
        ))
    }

    /// Extracts contents of a PBO file
    /// 
    /// # Arguments
    /// * `pbo_path` - Path to the PBO file
    /// * `output_dir` - Directory where contents will be extracted
    /// * `file_filter` - Optional filter to extract specific files only
    pub fn extract<P, Q>(&self, pbo_path: P, output_dir: Q, file_filter: Option<&str>) -> Result<bool>
    where
        P: AsRef<Path> + Send + Sync + 'static,
        Q: AsRef<Path> + Send + Sync + 'static,
    {
        let result = self.with_timeout(|| {
            self.core.extract_files(pbo_path.as_ref(), output_dir.as_ref(), file_filter)
        })?;
        Ok(result.return_code == 0)
    }

    /// Gets the prefix from a PBO file
    pub fn get_prefix<P: AsRef<Path> + Send + Sync + 'static>(&self, pbo_path: P) -> Result<Option<String>> {
        let result = self.with_timeout(|| self.core.list_contents(pbo_path.as_ref()))?;
        Ok(self.core.extract_prefix(&result.stdout))
    }

    fn with_timeout<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            let _ = tx.send(f());
        });

        match rx.recv_timeout(self.timeout) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => {
                Err(PboError::Timeout(self.timeout.as_secs() as u32))
            },
            Err(RecvTimeoutError::Disconnected) => {
                Err(PboError::Extraction(crate::errors::ExtractError::Canceled(
                    "Operation was canceled".to_string()
                )))
            }
        }
    }
}