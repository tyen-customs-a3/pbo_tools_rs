mod common;

use pbo_tools_rs::{PboApi, PboError};
use pbo_tools_rs::errors::ExtractError;
use crate::common::{setup_test_data, EXPECTED_PREFIX};

#[test]
fn test_list_contents_success() {
    let (_temp, data_dir) = setup_test_data();
    let pbo_path = data_dir.join(common::SAMPLE_PBO_NAME);
    let api = PboApi::new(30);
    
    let result = api.list_contents(&pbo_path).unwrap();
    assert!(result.0);
    assert!(!result.1.is_empty());
    assert!(result.1.lines().any(|line| line.contains("config.bin")));
}

#[test]
fn test_list_contents_error() {
    let api = PboApi::new(30);
    let result = api.list_contents("nonexistent.pbo");
    assert!(matches!(result, Err(PboError::InvalidPath(_))));
}

#[test]
fn test_extract_success() {
    let (_temp, data_dir) = setup_test_data();
    let pbo_path = data_dir.join(common::SAMPLE_PBO_NAME);
    let output_dir = tempfile::tempdir().unwrap();
    
    let api = PboApi::new(30);
    assert!(api.extract(&pbo_path, output_dir.path(), None).unwrap());
    
    let config_path = output_dir.path().join("tc/mirrorform/config.cpp");
    let rvmat_path = output_dir.path().join("tc/mirrorform/uniform/mirror.rvmat");
    assert!(config_path.exists());
    assert!(rvmat_path.exists());
}

#[test]
fn test_extract_with_filter() {
    let (_temp, data_dir) = setup_test_data();
    let pbo_path = data_dir.join(common::SAMPLE_PBO_NAME);
    let output_dir = tempfile::tempdir().unwrap();
    
    let api = PboApi::new(30);
    assert!(api.extract(&pbo_path, output_dir.path(), Some("config.bin")).unwrap());
    
    let config_path = output_dir.path().join("config.cpp");
    assert!(config_path.exists());
}

#[test]
fn test_get_prefix() {
    let (_temp, data_dir) = setup_test_data();
    let pbo_path = data_dir.join(common::SAMPLE_PBO_NAME);
    let api = PboApi::new(30);
    
    let prefix = api.get_prefix(&pbo_path).unwrap();
    assert_eq!(prefix, Some(EXPECTED_PREFIX.to_string()));
}

#[test]
fn test_invalid_pbo() {
    let temp = tempfile::tempdir().unwrap();
    let invalid_pbo = temp.path().join("invalid.pbo");
    std::fs::write(&invalid_pbo, b"invalid data").unwrap();
    
    let api = PboApi::new(30);
    let result = api.extract(&invalid_pbo, temp.path(), None);
    assert!(matches!(result, Err(PboError::InvalidPbo(_))));
}

#[test]
fn test_timeout() {
    let api = PboApi::new(1); // 1 second timeout
    let result = api.extract("nonexistent.pbo", "nonexistent", None);
    assert!(matches!(result, Err(PboError::InvalidPath(_))));
}