use crate::common;

use pbo_tools_rs::core::api::PboApi;
use pbo_tools_rs::core::PboApiOps;
use tempfile::tempdir;
use std::fs::File;

#[test]
fn test_extract_all_files() {
    let api = PboApi::new(30);
    let pbo_path = common::get_test_pbo_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    let result = api.extract_files(&pbo_path, temp_dir.path(), None);
    assert!(result.is_ok());
    
    let config_exists = temp_dir.path().join("config.bin").exists();
    let p3d_exists = temp_dir.path().join("uniform").join("mirror.p3d").exists();
    assert!(config_exists, "config.bin should be extracted");
    assert!(p3d_exists, "mirror.p3d should be extracted");
}

#[test]
fn test_extract_single_file() {
    let api = PboApi::new(30);
    let pbo_path = common::get_test_pbo_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    let result = api.extract_files(&pbo_path, temp_dir.path(), Some("config.bin"));
    assert!(result.is_ok());
    
    let config_exists = temp_dir.path().join("config.bin").exists();
    let p3d_exists = temp_dir.path().join("uniform").join("mirror.p3d").exists();
    
    assert!(config_exists, "config.bin should be extracted");
    assert!(!p3d_exists, "mirror.p3d should not be extracted");
}

#[test]
fn test_extract_with_pattern() {
    let api = PboApi::new(30);
    let pbo_path = common::get_test_pbo_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    let result = api.extract_files(&pbo_path, temp_dir.path(), Some("*.p3d"));
    assert!(result.is_ok());
    
    let config_exists = temp_dir.path().join("config.bin").exists();
    let p3d_exists = temp_dir.path().join("uniform").join("mirror.p3d").exists();
    
    assert!(!config_exists, "config.bin should not be extracted");
    assert!(p3d_exists, "mirror.p3d should be extracted");
}

#[test]
fn test_extract_multiple_patterns() {
    let api = PboApi::new(30);
    let pbo_path = common::get_test_pbo_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    let result = api.extract_files(&pbo_path, temp_dir.path(), Some("config.bin|*.p3d"));
    assert!(result.is_ok());
    
    let config_exists = temp_dir.path().join("config.bin").exists();
    let p3d_exists = temp_dir.path().join("uniform").join("mirror.p3d").exists();
    
    assert!(config_exists, "config.bin should be extracted");
    assert!(p3d_exists, "mirror.p3d should be extracted");
}

#[test]
fn test_extract_to_existing_files() {
    let api = PboApi::new(30);
    let pbo_path = common::get_test_pbo_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    // Create conflicting file
    let _ = File::create(temp_dir.path().join("config.bin")).unwrap();
    
    let result = api.extract_files(&pbo_path, temp_dir.path(), None);
    assert!(result.is_ok());
    
    // Verify file was overwritten
    let metadata = std::fs::metadata(temp_dir.path().join("config.bin")).unwrap();
    assert!(metadata.len() > 0, "File should be overwritten with content");
}