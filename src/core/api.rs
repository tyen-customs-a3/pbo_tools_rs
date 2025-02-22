use std::path::{Path, PathBuf};
use std::time::Duration;
use std::sync::{mpsc, Arc};
use std::thread;
use log::{debug, warn};
use crate::error::types::{Result, PboError, ExtractError};
use crate::extract::{ExtractResult, ExtractorClone, DefaultExtractor};
use crate::fs::{TempFileManager, FileOperation};
use super::config::PboConfig;
use super::constants::DEFAULT_TIMEOUT;

pub trait PboApiOps {
    fn list_contents(&self, pbo_path: &Path) -> Result<ExtractResult>;
    fn list_contents_brief(&self, pbo_path: &Path) -> Result<ExtractResult>;
    fn extract_files(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult>;
}

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
        pbo_path.validate_path()
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
        self.validate_pbo_exists(pbo_path)?;
        let pbo_path = pbo_path.to_owned();
        let extractor = self.extractor.clone();
        
        self.with_timeout(move || {
            debug!("Listing contents of PBO: {}", pbo_path.display());
            let result = extractor.list_contents(&pbo_path, false)?;
            
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

    fn list_contents_brief(&self, pbo_path: &Path) -> Result<ExtractResult> {
        self.validate_pbo_exists(pbo_path)?;
        let pbo_path = pbo_path.to_owned();
        let extractor = self.extractor.clone();
        
        self.with_timeout(move || {
            debug!("Listing contents briefly of PBO: {}", pbo_path.display());
            extractor.list_contents(&pbo_path, true)
        })
    }

    fn extract_files(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        self.validate_pbo_exists(pbo_path)?;
        output_dir.ensure_parent_exists()?;
        
        let pbo_path = pbo_path.to_owned();
        let output_dir = output_dir.to_owned();
        let file_filter = file_filter.map(String::from);
        let extractor = self.extractor.clone();
        
        self.with_timeout(move || {
            debug!("Extracting files from PBO: {} to {}", pbo_path.display(), output_dir.display());
            debug!("Using filter: {:?}", file_filter);
            
            let result = extractor.extract(&pbo_path, &output_dir, file_filter.as_deref())?;
            
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
