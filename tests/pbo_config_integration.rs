use pbo_tools_rs::core::{PboApi, PboConfig, PboApiOps};
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_custom_config_integration() {
    let config = PboConfig::builder()
        .add_bin_mapping("custom.bin", "custom.txt")
        .case_sensitive(true)
        .max_retries(5)
        .build();
        
    let api = PboApi::builder()
        .with_config(config)
        .with_timeout(30)
        .build();
        
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    let result = api.list_contents(test_pbo).unwrap();
    assert!(result.is_success());
}

#[test]
fn test_case_sensitivity_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config = PboConfig::builder()
        .case_sensitive(true)
        .build();
        
    let api = PboApi::builder()
        .with_config(config)
        .build();
        
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    let output_dir = temp_dir.path().join("case_test");
    
    // Should handle case-sensitive paths correctly
    let result = api.extract_files(test_pbo, &output_dir, Some("*.CPP"));
    assert!(result.is_err()); // Should fail because *.CPP won't match *.cpp with case sensitivity on
}