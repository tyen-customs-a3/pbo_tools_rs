use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PboError>;

#[derive(Error, Debug)]
pub enum PboError {
    #[error("PBO extraction failed: {0}")]
    Extraction(#[from] ExtractError),

    #[error("Operation timed out after {0} seconds")]
    Timeout(u32),

    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid PBO format: {0}")]
    InvalidPbo(String),

    #[error("Content encoding error in {path}: {context}")]
    Encoding {
        context: String,
        path: PathBuf,
    },

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),
}

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("Command execution failed: {cmd} - {reason}")]
    CommandFailed {
        cmd: String,
        reason: String,
    },
    
    #[error("No files were extracted")]
    NoFiles,
    
    #[error("Bad PBO format: {reason}")]
    BadFormat {
        reason: String,
    },

    #[error("Extraction canceled: {0}")]
    Canceled(String),
}

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("Failed to create directory {path}: {reason}")]
    CreateDir {
        path: PathBuf,
        reason: String,
    },
    
    #[error("Failed to remove directory {path}: {reason}")]
    RemoveDir {
        path: PathBuf,
        reason: String,
    },
    
    #[error("Failed to rename file from {from} to {to}: {reason}")]
    Rename {
        from: PathBuf,
        to: PathBuf,
        reason: String,
    },
    
    #[error("Failed to read file {path}: {reason}")]
    Read {
        path: PathBuf,
        reason: String,
    },
}
