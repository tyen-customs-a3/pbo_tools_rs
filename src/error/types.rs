use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PboError>;

#[derive(Error, Debug)]
pub enum PboError {
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),
    
    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),
    
    #[error("Extraction error: {0}")]
    Extraction(#[from] ExtractError),
    
    #[error("Command not found: {0}")]
    CommandNotFound(String),
    
    #[error("Operation timed out after {0} seconds")]
    Timeout(u32),

    #[error("Invalid PBO format: {0}")]
    InvalidFormat(String),

    #[error("PBO validation failed: {0}")]
    ValidationFailed(String),

    #[error("Encoding error: {context} for {}", .path.display())]
    Encoding {
        context: String,
        path: PathBuf,
    },
}

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("Command failed: {cmd} - {reason}")]
    CommandFailed {
        cmd: String,
        reason: String,
    },

    #[error("No files found in PBO")]
    NoFiles,

    #[error("Operation canceled: {0}")]
    Canceled(String),

    #[error("Invalid file filter: {0}")]
    InvalidFilter(String),
}

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("Failed to create directory {path}: {reason}")]
    CreateDir {
        path: PathBuf,
        reason: String,
    },

    #[error("Failed to read file {path}: {reason}")]
    ReadFile {
        path: PathBuf,
        reason: String,
    },

    #[error("Failed to write file {path}: {reason}")]
    WriteFile {
        path: PathBuf,
        reason: String,
    },

    #[error("Failed to delete {path}: {reason}")]
    Delete {
        path: PathBuf,
        reason: String,
    },

    #[error("Failed to remove directory {path}: {reason}")]
    RemoveDir {
        path: PathBuf,
        reason: String,
    },

    #[error("Invalid file name: {0}")]
    InvalidFileName(String),

    #[error("Path validation failed: {0}")]
    PathValidation(String),

    #[error("Failed to read from {}", .path.display())]
    Read {
        path: PathBuf,
        reason: String,
    },

    #[error("Failed to write to {}", .path.display())]
    Write {
        path: PathBuf,
        reason: String,
    },
}
