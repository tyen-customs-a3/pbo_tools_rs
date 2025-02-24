use pbo_tools::core::{PboApi, PboApiOps};
use pbo_tools::extract::ExtractOptions;
use pbo_tools::error::types::{PboError, ExtractError};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[test]
fn test_invalid_pbo_path() {
    let api = PboApi::new(30);
    let nonexistent = PathBuf::from("nonexistent.pbo");
    
    match api.list_contents(&nonexistent) {
        Err(PboError::InvalidPath(path)) => {
            assert_eq!(path, nonexistent);
        }
        other => panic!("Expected InvalidPath error, got {:?}", other),
    }
}

#[test]
fn test_invalid_output_dir() {
    let api = PboApi::new(30);
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    // Use an absolute path with multiple missing parent directories to ensure InvalidPath error
    let invalid_dir = PathBuf::from("/nonexistent/deep/nested/path/tc/mirrorform");
    
    match api.extract_files(test_pbo, &invalid_dir, None) {
        Err(PboError::InvalidPath(path)) => {
            assert_eq!(path, invalid_dir);
        }
        other => panic!("Expected InvalidPath error, got {:?}", other),
    }
}

#[test]
fn test_invalid_file_filter() {
    let api = PboApi::new(30);
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    let temp_dir = TempDir::new().unwrap();
    
    let options = ExtractOptions {
        file_filter: Some("[[invalid-regex".to_string()),
        ..Default::default()
    };
    
    match api.extract_with_options(test_pbo, temp_dir.path(), options) {
        Err(PboError::ValidationFailed(msg)) => {
            assert!(msg.contains("Invalid file filter pattern"));
        }
        other => panic!("Expected ValidationFailed error, got {:?}", other),
    }
}

#[test]
fn test_validation_failures() {
    let api = PboApi::new(30);
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    let temp_dir = TempDir::new().unwrap();
    
    // Test with empty file filter
    let options = ExtractOptions {
        file_filter: Some("".to_string()),
        ..Default::default()
    };
    
    match api.extract_with_options(test_pbo, temp_dir.path(), options) {
        Err(PboError::ValidationFailed(msg)) => {
            assert_eq!(msg, "File filter cannot be empty");
        }
        other => panic!("Expected ValidationFailed error, got {:?}", other),
    }
}