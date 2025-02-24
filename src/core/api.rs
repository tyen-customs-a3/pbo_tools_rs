use std::path::{Path, PathBuf};
use std::time::Duration;
use std::sync::{mpsc, Arc};
use std::thread;
use log::{debug, warn};
use crate::error::types::{Result, PboError, ExtractError};
use crate::extract::{ExtractResult, ExtractorClone, DefaultExtractor, ExtractOptions};
use crate::fs::{TempFileManager, FileOperation};
use super::config::PboConfig;
use super::constants::DEFAULT_TIMEOUT;

/// Core trait defining operations available for PBO files.
/// 
/// This trait provides the main interface for working with PBO files, including:
/// - Listing contents (with various detail levels)
/// - Extracting files (with filtering and customization options)
/// - Advanced operations with custom options
///
/// # Examples
///
/// ```no_run
/// use pbo_tools::core::{PboApi, PboApiOps};
/// use std::path::Path;
///
/// let api = PboApi::new(30); // 30 second timeout
/// let pbo_path = Path::new("mission.pbo");
///
/// // List contents
/// let result = api.list_contents(&pbo_path).unwrap();
/// println!("Files in PBO: {:?}", result.get_file_list());
///
/// // Extract specific files
/// let output_dir = Path::new("output");
/// api.extract_files(&pbo_path, &output_dir, Some("*.cpp")).unwrap();
/// ```
pub trait PboApiOps {
    /// List contents of a PBO file with standard output format
    fn list_contents(&self, pbo_path: &Path) -> Result<ExtractResult>;
    
    /// List contents of a PBO file in brief directory-style format
    fn list_contents_brief(&self, pbo_path: &Path) -> Result<ExtractResult>;
    
    /// Extract files from a PBO with optional file filtering
    fn extract_files(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult>;
    
    /// List contents with custom options for fine-grained control
    fn list_with_options(&self, pbo_path: &Path, options: ExtractOptions) -> Result<ExtractResult>;
    
    /// Extract files with custom options for fine-grained control
    fn extract_with_options(&self, pbo_path: &Path, output_dir: &Path, options: ExtractOptions) -> Result<ExtractResult>;
}

/// Main API for working with PBO files.
///
/// PboApi provides a high-level interface for PBO operations with:
/// - Configurable timeout handling
/// - Custom configuration options
/// - Error handling and validation
/// - Progress tracking and logging
///
/// # Examples
///
/// Basic usage:
/// ```no_run
/// use pbo_tools::core::PboApi;
///
/// let api = PboApi::builder()
///     .with_timeout(30)
///     .build();
/// ```
///
/// Advanced configuration:
/// ```no_run
/// use pbo_tools::core::{PboApi, PboConfig};
///
/// let config = PboConfig::builder()
///     .case_sensitive(true)
///     .max_retries(5)
///     .build();
///
/// let api = PboApi::builder()
///     .with_config(config)
///     .with_timeout(30)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct PboApi {
    temp_manager: TempFileManager,
    config: Arc<PboConfig>,
    extractor: Box<dyn ExtractorClone>,
    timeout: Duration,
}

impl PboApi {
    pub fn builder() -> PboApiBuilder {
        PboApiBuilder::new()
    }

    pub fn new(timeout_seconds: u32) -> Self {
        Self::builder()
            .with_timeout(timeout_seconds)
            .build()
    }

    pub fn extract_prefix(&self, output: &str) -> Option<String> {
        output
            .lines()
            .find(|line| line.starts_with("prefix="))
            .and_then(|line| {
                line.split('=')
                    .nth(1)
                    .map(|prefix| prefix.trim().trim_end_matches(';').to_string())
            })
            .filter(|prefix| !prefix.is_empty())
    }

    fn validate_pbo_exists(&self, pbo_path: &Path) -> Result<()> {
        if !pbo_path.exists() {
            return Err(PboError::InvalidPath(pbo_path.to_path_buf()));
        }
        Ok(())
    }

    fn validate_output_dir(&self, output_dir: &Path) -> Result<()> {
        if !output_dir.exists() {
            // Try to create it
            if let Some(parent) = output_dir.parent() {
                if !parent.exists() {
                    return Err(PboError::InvalidPath(output_dir.to_path_buf()));
                }
            }
        }
        Ok(())
    }

    fn with_timeout<T, F>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let (cancel_tx, cancel_rx) = mpsc::channel();
        let timeout = self.timeout;

        let handle = thread::spawn(move || {
            // Set up cancellation check
            let start = std::time::Instant::now();
            let result = operation();

            // Check for cancellation periodically
            if cancel_rx.try_recv().is_ok() {
                debug!("Operation was canceled after {} ms", start.elapsed().as_millis());
                return;
            }

            // Try to send result back
            if tx.send(result).is_err() {
                warn!("Failed to send operation result - receiver dropped");
            }
        });

        // Wait for result with timeout
        let result = match rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                debug!("Operation timed out after {} seconds", timeout.as_secs());
                // Try to cancel the operation
                if let Err(e) = cancel_tx.send(()) {
                    warn!("Failed to send cancellation signal: {}", e);
                }
                
                // Give the thread a short time to clean up
                let _ = thread::spawn(move || {
                    thread::sleep(Duration::from_secs(1));
                    if let Err(e) = handle.join() {
                        warn!("Operation thread did not terminate cleanly: {:?}", e);
                    }
                });

                Err(PboError::Timeout(timeout.as_secs() as u32))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                warn!("Operation thread terminated unexpectedly");
                Err(PboError::Extraction(ExtractError::Canceled(
                    "Operation thread terminated unexpectedly".to_string()
                )))
            }
        };

        result
    }
}

impl PboApiOps for PboApi {
    fn list_contents(&self, pbo_path: &Path) -> Result<ExtractResult> {
        let options = ExtractOptions {
            no_pause: true,
            warnings_as_errors: true,
            ..Default::default()
        };
        self.list_with_options(pbo_path, options)
    }

    fn list_contents_brief(&self, pbo_path: &Path) -> Result<ExtractResult> {
        let options = ExtractOptions {
            no_pause: true,
            warnings_as_errors: true,
            brief_listing: true,
            ..Default::default()
        };
        self.list_with_options(pbo_path, options)
    }

    fn extract_files(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        let options = ExtractOptions {
            no_pause: true,
            warnings_as_errors: true,
            file_filter: file_filter.map(String::from),
            ..Default::default()
        };
        self.extract_with_options(pbo_path, output_dir, options)
    }

    fn list_with_options(&self, pbo_path: &Path, options: ExtractOptions) -> Result<ExtractResult> {
        self.validate_pbo_exists(pbo_path)?;
        let pbo_path = pbo_path.to_owned();
        let extractor = self.extractor.clone();
        let options = options.clone();
        
        self.with_timeout(move || {
            debug!("Listing contents of PBO with options: {:?}", options);
            let result = extractor.list_with_options(&pbo_path, options)?;
            
            if !result.is_success() {
                debug!("PBO listing failed: {}", result);
                return Err(PboError::Extraction(ExtractError::CommandFailed {
                    cmd: "extractpbo".to_string(),
                    reason: result.get_error_message()
                        .unwrap_or_else(|| "Unknown error".to_string()),
                }));
            }
            
            Ok(result)
        })
    }

    fn extract_with_options(&self, pbo_path: &Path, output_dir: &Path, options: ExtractOptions) -> Result<ExtractResult> {
        self.validate_pbo_exists(pbo_path)?;
        self.validate_output_dir(output_dir)?;
        
        // Validate file filter
        if let Some(filter) = &options.file_filter {
            if filter.trim().is_empty() {
                return Err(PboError::ValidationFailed("File filter cannot be empty".to_string()));
            }
            
            // Validate regex patterns specifically (patterns that don't use glob wildcards)
            if !filter.contains('*') && !filter.contains('?') {
                // If it's not a glob pattern, treat it as regex and validate it
                if let Err(_) = regex::Regex::new(filter) {
                    return Err(PboError::ValidationFailed(format!("Invalid file filter pattern: {}", filter)));
                }
            }
        }
        
        let pbo_path = pbo_path.to_owned();
        let output_dir = output_dir.to_owned();
        let extractor = self.extractor.clone();
        let options = options.clone();
        
        self.with_timeout(move || {
            debug!("Extracting files with options: {:?}", options);
            let result = extractor.extract_with_options(&pbo_path, &output_dir, options)?;
            
            if !result.is_success() {
                debug!("PBO extraction failed: {}", result);
                return Err(PboError::Extraction(ExtractError::CommandFailed {
                    cmd: "extractpbo".to_string(),
                    reason: result.get_error_message()
                        .unwrap_or_else(|| "Unknown error".to_string()),
                }));
            }
            
            Ok(result)
        })
    }
}

/// Builder for creating customized PboApi instances.
///
/// The builder pattern allows for flexible configuration of:
/// - Operation timeout
/// - PBO handling configuration
/// - Custom extractors (for testing or specialized use cases)
///
/// # Examples
///
/// ```no_run
/// use pbo_tools::core::PboApi;
///
/// let api = PboApi::builder()
///     .with_timeout(60)  // 60 second timeout
///     .build();
/// ```
#[derive(Default)]
pub struct PboApiBuilder {
    config: Option<PboConfig>,
    timeout: Option<Duration>,
}

impl PboApiBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: PboConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout = Some(Duration::from_secs(u64::from(seconds.max(1))));
        self
    }

    pub fn build(self) -> PboApi {
        PboApi {
            temp_manager: TempFileManager::new(),
            config: Arc::new(self.config.unwrap_or_default()),
            extractor: Box::new(DefaultExtractor::new()),
            timeout: self.timeout.unwrap_or_else(|| Duration::from_secs(u64::from(DEFAULT_TIMEOUT))),
        }
    }
}
