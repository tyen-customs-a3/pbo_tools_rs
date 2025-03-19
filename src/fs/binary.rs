use std::path::Path;
use log::{debug, info};
use crate::error::types::{PboError, FileSystemError, Result};
use std::fs;
use crate::core::config::PboConfig;

pub fn convert_binary_file(input: &Path, output: &Path) -> Result<()> {
    debug!("Converting binary file from {:?} to {:?}", input, output);
    
    // Ensure parent directory exists
    if let Some(parent) = output.parent() {
        debug!("Creating parent directory: {:?}", parent);
        fs::create_dir_all(parent).map_err(|e| {
            PboError::FileSystem(FileSystemError::CreateDir {
                path: parent.to_path_buf(),
                reason: e.to_string(),
            })
        })?;
    }

    // Check if source file exists
    if !input.exists() {
        let err = PboError::FileSystem(FileSystemError::ReadFile {
            path: input.to_path_buf(),
            reason: "Source file does not exist".to_string(),
        });
        debug!("Error: {}", err);
        return Err(err);
    }

    debug!("Renaming file");
    fs::rename(input, output).map_err(|e| {
        let err = PboError::FileSystem(FileSystemError::WriteFile {
            path: output.to_path_buf(),
            reason: e.to_string(),
        });
        debug!("Error during rename: {}", err);
        err
    })?;

    info!("Successfully converted {:?} to {:?}", input, output);
    Ok(())
}

pub fn process_binary_files(source_dir: &Path, config: &PboConfig) -> Result<()> {
    if !source_dir.is_dir() {
        debug!("Source directory {:?} is not a directory", source_dir);
        return Ok(());
    }

    debug!("Processing binary files in {:?}", source_dir);
    for entry in fs::read_dir(source_dir).map_err(|e| {
        PboError::FileSystem(FileSystemError::ReadFile {
            path: source_dir.to_path_buf(),
            reason: e.to_string(),
        })
    })? {
        let entry = entry.map_err(|e| {
            PboError::FileSystem(FileSystemError::ReadFile {
                path: source_dir.to_path_buf(),
                reason: e.to_string(),
            })
        })?;

        let path = entry.path();
        if path.is_dir() {
            debug!("Found directory: {:?}, recursing", path);
            process_binary_files(&path, config)?;
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            debug!("Processing file: {}", name);
            
            // Try exact filename match first
            let target_ext = config.get_bin_extension(name);
            
            if let Some(ext) = target_ext {
                debug!("Found mapping for {}: new extension will be {}", name, ext);
                // Get the filename without extension
                let stem = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unnamed");
                
                let new_path = path.with_file_name(format!("{}.{}", stem, ext));
                convert_binary_file(&path, &new_path)?;
            } else {
                debug!("No mapping found for {}, skipping", name);
            }
        }
    }

    info!("Completed processing binary files in {:?}", source_dir);
    Ok(())
}

// Remove duplicated tests since they are covered in binary_handling.rs
