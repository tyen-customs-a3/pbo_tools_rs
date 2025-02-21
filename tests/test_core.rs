mod common;

use std::path::Path;
use pbo_tools_rs::core::PboCore;
use pbo_tools_rs::config::PboConfig;
use pbo_tools_rs::errors::PboError;
use crate::common::setup_test_data;

#[test]
fn test_core_new() {
    let core = PboCore::new(None);
    assert!(core.list_contents(Path::new("nonexistent.pbo")).is_err());
}

#[test]
fn test_core_with_custom_config() {
    let config = PboConfig::builder()
        .add_bin_mapping("custom.bin", "custom.txt")
        .build();
    let core = PboCore::new(Some(config));
    
    let (_temp, data_dir) = setup_test_data();
    let pbo_path = data_dir.join(common::SAMPLE_PBO_NAME);
    let result = core.list_contents(&pbo_path).unwrap();
    assert_eq!(result.return_code, 0);
}

#[test]
fn test_core_extract_files() {
    let core = PboCore::new(None);
    let (_temp, data_dir) = setup_test_data();
    let pbo_path = data_dir.join(common::SAMPLE_PBO_NAME);
    let output_dir = tempfile::tempdir().unwrap();

    let result = core.extract_files(&pbo_path, output_dir.path(), None).unwrap();
    assert_eq!(result.return_code, 0);
    assert!(output_dir.path().join("tc/mirrorform/config.cpp").exists());
}

#[test]
fn test_core_extract_with_filter() {
    let core = PboCore::new(None);
    let (_temp, data_dir) = setup_test_data();
    let pbo_path = data_dir.join(common::SAMPLE_PBO_NAME);
    let output_dir = tempfile::tempdir().unwrap();

    let result = core.extract_files(&pbo_path, output_dir.path(), Some("config.bin")).unwrap();
    assert_eq!(result.return_code, 0);
    assert!(output_dir.path().join("config.cpp").exists());
}

#[test]
fn test_core_invalid_pbo() {
    let core = PboCore::new(None);
    let temp = tempfile::tempdir().unwrap();
    let invalid_pbo = temp.path().join("invalid.pbo");
    std::fs::write(&invalid_pbo, b"invalid data").unwrap();

    let result = core.extract_files(&invalid_pbo, temp.path(), None);
    assert!(matches!(result, Err(PboError::InvalidPbo(_))));
}

#[test]
fn test_core_extract_prefix() {
    let core = PboCore::new(None);
    let sample_output = "prefix=tc/mirrorform;\nSome other content";
    let prefix = core.extract_prefix(sample_output);
    assert_eq!(prefix, Some("tc/mirrorform".to_string()));
}

#[test]
fn test_core_validate_pbo() {
    let core = PboCore::new(None);
    let temp = tempfile::tempdir().unwrap();
    let empty_pbo = temp.path().join("empty.pbo");
    std::fs::write(&empty_pbo, b"").unwrap();

    let result = core.extract_files(&empty_pbo, temp.path(), None);
    assert!(matches!(result, Err(PboError::InvalidPbo(_))));
}