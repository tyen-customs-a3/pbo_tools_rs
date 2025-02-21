mod common;
use std::path::Path;
use pbo_tools_rs::extract::{DefaultExtractor, Extractor, ExtractResult};
use pbo_tools_rs::errors::PboError;
use crate::common::setup_test_data;

#[test]
fn test_extract_result_display() {
    let success = ExtractResult {
        return_code: 0,
        stdout: "Success output".to_string(),
        stderr: "".to_string(),
    };
    assert_eq!(success.to_string(), "Success output");

    let error = ExtractResult {
        return_code: 1,
        stdout: "".to_string(),
        stderr: "Error message".to_string(),
    };
    assert_eq!(error.to_string(), "Error (1): Error message");
}

#[test]
fn test_default_extractor_extract() {
    let (_temp, data_dir) = setup_test_data();
    let pbo_path = data_dir.join(common::SAMPLE_PBO_NAME);
    let output_dir = tempfile::tempdir().unwrap();
    let extractor = DefaultExtractor::new();

    let result = extractor.extract(&pbo_path, output_dir.path(), None).unwrap();
    assert_eq!(result.return_code, 0);
    assert!(!result.stdout.is_empty());
}

#[test]
fn test_default_extractor_with_filter() {
    let (_temp, data_dir) = setup_test_data();
    let pbo_path = data_dir.join(common::SAMPLE_PBO_NAME);
    let output_dir = tempfile::tempdir().unwrap();
    let extractor = DefaultExtractor::new();

    let result = extractor.extract(&pbo_path, output_dir.path(), Some("config.bin")).unwrap();
    assert_eq!(result.return_code, 0);
    assert!(!result.stdout.is_empty());
}

#[test]
fn test_invalid_extractor_path() {
    let extractor = DefaultExtractor::new();
    let result = extractor.extract(
        Path::new("nonexistent.pbo"),
        Path::new("output"),
        None
    );
    assert!(matches!(result, Err(PboError::CommandNotFound(_))));
}