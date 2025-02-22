use crate::common;

use pbo_tools_rs::core::api::PboApi;
use pbo_tools_rs::core::PboApiOps;
use std::path::PathBuf;
use tempfile::tempdir;
use std::fs::File;

#[test]
fn test_nonexistent_pbo() {
    let api = PboApi::new(30);
    let invalid_path = PathBuf::from("nonexistent.pbo");
    let result = api.list_contents(&invalid_path);
    assert!(result.is_err());
}

#[test]
fn test_invalid_output_directory() {
    let api = PboApi::new(30);
    let pbo_path = common::get_test_pbo_path();
    let invalid_dir = PathBuf::from("/nonexistent/directory");
    
    let result = api.extract_files(&pbo_path, &invalid_dir, None);
    assert!(result.is_err());
}

#[test]
fn test_empty_pbo() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let empty_pbo = temp_dir.path().join("empty.pbo");
    File::create(&empty_pbo).expect("Failed to create empty PBO");
    
    let api = PboApi::new(30);
    let result = api.list_contents(&empty_pbo);
    assert!(result.is_err());
}

#[test]
fn test_extract_prefix_edge_cases() {
    let api = PboApi::new(30);
    
    // Empty input
    assert_eq!(api.extract_prefix(""), None);
    
    // Empty prefix value
    assert_eq!(api.extract_prefix("prefix="), Some("".to_string()));
    
    // Prefix with spaces and special chars
    let sample = "prefix=test\\path with spaces;";
    assert_eq!(api.extract_prefix(sample), Some("test\\path with spaces".to_string()));
    
    // Multiple prefix entries
    let multiple = "prefix=first;prefix=second;";
    assert_eq!(api.extract_prefix(multiple), Some("first".to_string()));
}

#[test]
fn test_extract_with_invalid_filter() {
    let api = PboApi::new(30);
    let pbo_path = common::get_test_pbo_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    // Non-matching filter
    let result = api.extract_files(&pbo_path, temp_dir.path(), Some("nonexistent_file.txt"));
    assert!(result.is_ok());
    let files = std::fs::read_dir(temp_dir.path()).unwrap();
    assert_eq!(files.count(), 0, "No files should be extracted with non-matching filter");
    
    // Empty filter
    let result = api.extract_files(&pbo_path, temp_dir.path(), Some(""));
    assert!(result.is_ok());
    let files = std::fs::read_dir(temp_dir.path()).unwrap();
    assert_eq!(files.count(), 0, "No files should be extracted with empty filter");
}