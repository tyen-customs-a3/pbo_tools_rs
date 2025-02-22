use std::path::Path;
use std::fmt::Debug;
use std::process::Command;
use log::{debug, warn};
use crate::error::types::{Result, PboError, ExtractError, FileSystemError};
use crate::core::constants::{COMMON_PBO_EXTENSIONS, BAD_PBO_INDICATORS};
use super::result::ExtractResult;

/// ExtractPBO Command Line Interface Documentation
/// 
/// Syntax: extractpbo [-options...] PboName[.pbo|.xbo|.ifa]|FolderName|Extraction.lst|.txt  [destination]
///
/// Important: The order of arguments matters!
/// 1. Options must come before the PBO path
/// 2. PBO path must come before destination path
/// 3. Options can be catenated together (e.g. -PW instead of -P -W)
///
/// Arguments (in order):
/// - Options: All options must start with - or +
///   - `-P`: Don't pause execution
///   - `-W`: Treat warnings as errors
///   - `-F=filelist[,...]`: Extract specific file(s). Files are extracted to their correct position 
///     in the output folder tree. Supports basic wildcards (*.ext for all files with extension).
///     Multiple files can be separated by commas.
///   - `-L`: List contents only (do not extract)
///   - `-LB`: Brief directory-style output listing
///   - `-N`: Noisy (verbose) output
/// - PBO Path: Path to the source PBO file
/// - Destination Path: Optional output directory path. Must include drive letter.
///
/// Examples:
/// ```text
/// extractpbo -PW source.pbo D:/output           # Extract all files
/// extractpbo -PW -F=*.paa source.pbo D:/output  # Extract only .paa files
/// extractpbo -L source.pbo                      # List contents
/// ```
///
/// Extraction behavior:
/// 1. By default, creates a folder of the same name as the PBO in the destination
/// 2. For Arma PBOs, creates additional subfolders based on the detected prefix
///    Example: source.pbo -> destination/prefix/...
///
/// Notes:
/// - Destination paths MUST include a drive letter (relative paths not supported)
/// - The -F option's pattern is applied to the full file path within the PBO
/// - Error codes and output messages are used to determine operation success

// Combining the traits into a single trait to avoid trait object limitations
#[derive(Debug, Clone, Default)]
pub struct ExtractOptions {
    /// Don't pause execution (-P)
    pub no_pause: bool,
    /// Treat warnings as errors (-W)
    pub warnings_as_errors: bool,
    /// Extract specific file(s) (-F=filelist[,...])
    pub file_filter: Option<String>,
    /// Noisy (verbose) output (-N)
    pub verbose: bool,
    /// Brief directory-style output listing (-LB)
    pub brief_listing: bool,
}

impl ExtractOptions {
    pub fn validate(&self) -> Result<()> {
        // Can't use brief_listing with extraction operations
        if self.brief_listing && (self.file_filter.is_some()) {
            return Err(PboError::ValidationFailed(
                "Brief listing option cannot be used with extraction options".to_string()
            ));
        }

        // Validate file filter format if present
        if let Some(filter) = &self.file_filter {
            if filter.contains(['<', '>', '|', '"', '\'']) {
                return Err(PboError::ValidationFailed(
                    "File filter contains invalid characters".to_string()
                ));
            }
        }

        Ok(())
    }

    pub fn for_listing() -> Self {
        Self {
            no_pause: true,
            warnings_as_errors: true,
            ..Default::default()
        }
    }

    pub fn for_brief_listing() -> Self {
        Self {
            no_pause: true,
            warnings_as_errors: true,
            brief_listing: true,
            ..Default::default()
        }
    }

    pub fn for_extraction() -> Self {
        Self {
            no_pause: true,
            warnings_as_errors: true,
            ..Default::default()
        }
    }
}

pub trait ExtractorClone: Send + Sync + Debug {
    /// Extract files from a PBO with custom options
    fn extract_with_options(&self, pbo_path: &Path, output_dir: &Path, options: ExtractOptions) -> Result<ExtractResult>;
    
    /// List contents of a PBO with custom options
    fn list_with_options(&self, pbo_path: &Path, options: ExtractOptions) -> Result<ExtractResult>;
    
    // Default implementations for backward compatibility
    fn extract(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        let options = ExtractOptions {
            no_pause: true,
            warnings_as_errors: true,
            file_filter: file_filter.map(String::from),
            ..Default::default()
        };
        self.extract_with_options(pbo_path, output_dir, options)
    }
    
    fn list_contents(&self, pbo_path: &Path, brief: bool) -> Result<ExtractResult> {
        let options = ExtractOptions {
            no_pause: true,
            warnings_as_errors: true,
            brief_listing: brief,
            ..Default::default()
        };
        self.list_with_options(pbo_path, options)
    }

    fn clone_box(&self) -> Box<dyn ExtractorClone>;
}

impl Clone for Box<dyn ExtractorClone> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct DefaultExtractor;

impl DefaultExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Execute the extractpbo command following the strict argument order:
    /// 1. Core options (-PW)
    /// 2. Operation-specific options (-F=pattern, -L, etc)
    /// 3. PBO path
    /// 4. Destination path (if any)
    fn run_extractpbo_command(&self, args: Vec<&str>, pbo_path: &Path) -> Result<ExtractResult> {
        debug!("Running extractpbo command with args: {:?}", args);
        debug!("PBO path: {:?}", pbo_path);
        
        let mut command = Command::new("extractpbo");
        
        // Validate PBO path exists and is accessible
        if !pbo_path.exists() {
            return Err(PboError::InvalidPath(pbo_path.to_path_buf()));
        }

        if !pbo_path.extension().map_or(false, |ext| {
            COMMON_PBO_EXTENSIONS.contains(&ext.to_str().unwrap_or(""))
        }) {
            return Err(PboError::InvalidFormat(format!(
                "File {} does not have a valid PBO extension", 
                pbo_path.display()
            )));
        }
        
        // 1. Core options first (always used)
        command.arg("-PW");  // Combined: Don't pause (-P) and treat warnings as errors (-W)
        
        // 2. Operation-specific options (like -F=pattern or -L)
        let mut has_options = false;
        for arg in &args {
            if arg.starts_with('-') {
                // Special validation for -F option which can contain wildcards
                if arg.starts_with("-F=") {
                    command.arg(arg);
                    has_options = true;
                    continue;
                }
                
                // For other options, only allow letters and numbers
                if !arg.chars().skip(1).all(|c| c.is_ascii_alphanumeric() || c == 'B') {
                    return Err(PboError::ValidationFailed(
                        format!("Invalid option format: {}", arg)
                    ));
                }
                command.arg(arg);
                has_options = true;
            }
        }
        if has_options {
            debug!("Added operation-specific options");
        }

        // 3. PBO path (required)
        if let Some(pbo_str) = pbo_path.to_str() {
            command.arg(pbo_str.replace("\\\\?\\", ""));
            debug!("Added PBO path");
        } else {
            return Err(PboError::InvalidPath(pbo_path.to_path_buf()));
        }

        // 4. Destination path (if any non-flag args remain)
        let mut _added_dest = false;
        for arg in &args {
            if (!arg.starts_with('-')) {
                // Validate destination path
                let dest_path = Path::new(arg);
                if dest_path.to_str().map_or(true, |s| s.contains(['<', '>', '|', '"', '\''])) {
                    return Err(PboError::ValidationFailed(
                        format!("Invalid destination path: {}", arg)
                    ));
                }
                command.arg(arg);
                _added_dest = true;
                debug!("Added destination path");
                break; // Only add the first non-flag argument as destination
            }
        }

        debug!("Full command: {:?}", command);
        
        // Execute command with proper error handling
        match command.output() {
            Ok(output) => {
                debug!("Command completed with status: {:?}", output.status);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                debug!("Stdout: {}", stdout);
                debug!("Stderr: {}", stderr);

                // Check for specific error patterns in the output
                if BAD_PBO_INDICATORS.iter().any(|&indicator| {
                    stderr.contains(indicator) || stdout.contains(indicator)
                }) {
                    return Err(PboError::ValidationFailed(
                        format!("PBO validation failed:\n{}", stderr)
                    ));
                }

                Ok(ExtractResult {
                    return_code: output.status.code().unwrap_or(-1),
                    stdout: stdout.to_string(),
                    stderr: stderr.to_string(),
                })
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => 
                    Err(PboError::CommandNotFound("extractpbo".to_string())), 
                std::io::ErrorKind::PermissionDenied =>
                    Err(PboError::FileSystem(FileSystemError::PathValidation(
                        "Permission denied".to_string()
                    ))),
                _ => Err(PboError::Extraction(ExtractError::CommandFailed {
                    cmd: "extractpbo".to_string(),
                    reason: e.to_string(),
                }))
            }
        }
    }
}

impl ExtractorClone for DefaultExtractor {
    fn extract_with_options(&self, pbo_path: &Path, output_dir: &Path, options: ExtractOptions) -> Result<ExtractResult> {
        debug!("DefaultExtractor::extract_with_options called");
        debug!("PBO path: {:?}", pbo_path);
        debug!("Output dir: {:?}", output_dir);
        debug!("Options: {:?}", options);
        
        options.validate()?;

        // Create output directory if it doesn't exist
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir).map_err(|_e| PboError::InvalidPath(output_dir.to_path_buf()))?;
        }

        let mut args = Vec::new();
        
        // Build options string
        let mut opts = String::new();
        if options.no_pause { opts.push('P'); }
        if options.warnings_as_errors { opts.push('W'); }
        if options.verbose { opts.push('N'); }
        // Removed keep_pbo_name option as it's not supported
        if !opts.is_empty() { args.push(format!("-{}", opts)); }
        
        // Add file filter if present
        if let Some(filter) = options.file_filter {
            args.push(format!("-F={}", filter));
        }
        
        // Add output directory
        if let Some(out_str) = output_dir.canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.replace("\\\\?\\", "")))
        {
            args.push(out_str);
        } else {
            return Err(PboError::InvalidPath(output_dir.to_path_buf()));
        }
        
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
        self.run_extractpbo_command(args, pbo_path)
    }

    fn list_with_options(&self, pbo_path: &Path, options: ExtractOptions) -> Result<ExtractResult> {
        debug!("DefaultExtractor::list_with_options called");
        debug!("PBO path: {:?}", pbo_path);
        debug!("Options: {:?}", options);
        
        options.validate()?;

        let mut args = Vec::new();
        
        // Build options string
        let mut opts = String::new();
        if options.no_pause { opts.push('P'); }
        if options.warnings_as_errors { opts.push('W'); }
        if options.verbose { opts.push('N'); }
        opts.push('L');
        if options.brief_listing { opts.push('B'); }
        args.push(format!("-{}", opts));
        
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
        self.run_extractpbo_command(args, pbo_path)
    }

    fn extract(&self, pbo_path: &Path, output_dir: &Path, file_filter: Option<&str>) -> Result<ExtractResult> {
        debug!("DefaultExtractor::extract called");
        debug!("PBO path: {:?}", pbo_path);
        debug!("Output dir: {:?}", output_dir);
        debug!("File filter: {:?}", file_filter);
        
        // Create output directory if it doesn't exist
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir).map_err(|_e| PboError::InvalidPath(output_dir.to_path_buf()))?;
        }
        
        let mut args = Vec::new();
        
        // Add filter if present
        if let Some(filter) = file_filter {
            args.push(format!("-F={}", filter));
        }
        
        // Add output directory
        if let Some(out_str) = output_dir.canonicalize()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.replace("\\\\?\\", "")))
        {
            args.push(out_str);
        } else {
            return Err(PboError::InvalidPath(output_dir.to_path_buf()));
        }
        
        // Convert args to string slices for command
        let args: Vec<&str> = args.iter().map(AsRef::as_ref).collect();
        debug!("Calling run_extractpbo_command with args: {:?}", args);
        self.run_extractpbo_command(args, pbo_path)
    }

    fn list_contents(&self, pbo_path: &Path, brief: bool) -> Result<ExtractResult> {
        debug!("DefaultExtractor::list_contents called");
        debug!("PBO path: {:?}", pbo_path);
        debug!("Brief: {}", brief);
        
        let args = if brief { vec!["-LB"] } else { vec!["-L"] };
        debug!("Calling run_extractpbo_command with args: {:?}", args);
        self.run_extractpbo_command(args, pbo_path)
    }

    fn clone_box(&self) -> Box<dyn ExtractorClone> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_options_validation() {
        // Test invalid combination: brief listing with extraction options
        let options = ExtractOptions {
            brief_listing: true,
            file_filter: Some("*.cpp".to_string()),
            ..Default::default()
        };
        assert!(matches!(options.validate(), Err(PboError::ValidationFailed(_))));

        // Test invalid file filter characters
        let options = ExtractOptions {
            file_filter: Some("*.cpp|*.hpp".to_string()),
            ..Default::default()
        };
        assert!(matches!(options.validate(), Err(PboError::ValidationFailed(_))));

        // Test valid options
        let options = ExtractOptions {
            no_pause: true,
            warnings_as_errors: true,
            file_filter: Some("*.cpp".to_string()),
            verbose: true,
            ..Default::default()
        };
        assert!(options.validate().is_ok());
    }

    #[test]
    fn test_extract_options_factory_methods() {
        let listing = ExtractOptions::for_listing();
        assert!(listing.no_pause);
        assert!(listing.warnings_as_errors);
        assert!(!listing.brief_listing);
        assert!(listing.validate().is_ok());

        let brief = ExtractOptions::for_brief_listing();
        assert!(brief.brief_listing);
        assert!(brief.validate().is_ok());

        let extraction = ExtractOptions::for_extraction();
        assert!(extraction.no_pause);
        assert!(extraction.warnings_as_errors);
        assert!(!extraction.brief_listing);
        assert!(extraction.validate().is_ok());
    }
}
