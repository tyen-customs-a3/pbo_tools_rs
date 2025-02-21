use pbo_tools_rs::core::PboCore;
use pbo_tools_rs::core::config::PboConfig;
use pbo_tools_rs::extract::ExtractResult;
use pbo_tools_rs::core::api::PboApi;
use std::path::PathBuf;
use env_logger;
use log::debug;

fn setup() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

fn get_test_pbo_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("mirrorform.pbo")
}

#[test]
fn test_list_contents() {
    setup();
    let core = PboCore::new(None);
    let pbo_path = get_test_pbo_path();
    debug!("Testing PBO path: {:?}", pbo_path);
    
    let result = core.list_contents(&pbo_path).expect("Failed to list PBO contents");
    debug!("List contents returned: {:?}", result);
    
    assert!(result.is_success());
    let files = result.get_file_list();
    debug!("Parsed file list: {:?}", files);
    
    assert!(!files.is_empty(), "PBO should contain files");
    
    // Verify we can get at least some expected files
    let has_config = files.iter().any(|f| f.contains("config.bin"));
    let has_model = files.iter().any(|f| f.contains("uniform\\mirror.p3d"));
    debug!("Found config.bin: {}", has_config);
    debug!("Found uniform\\mirror.p3d: {}", has_model);
    
    assert!(has_config, "PBO should contain config.bin");
    assert!(has_model, "PBO should contain uniform/mirror.p3d");
}

#[test]
fn test_list_contents_brief() {
    let core = PboCore::new(None);
    let pbo_path = get_test_pbo_path();
    let result = core.list_contents_brief(&pbo_path).expect("Failed to list PBO contents briefly");
    
    assert!(result.is_success());
    let files = result.get_file_list();
    assert!(!files.is_empty(), "PBO should contain files");
    
    // Brief listing should still contain basic file information
    let has_files = files.iter().any(|f| !f.trim().is_empty());
    assert!(has_files, "Brief listing should contain file entries");
}

#[test]
fn test_get_prefix() {
    let core = PboCore::new(None);
    let pbo_path = get_test_pbo_path();
    let result = core.list_contents(&pbo_path).expect("Failed to list PBO contents");
    
    let prefix = result.get_prefix();
    assert!(prefix.is_some(), "PBO should have a prefix");
    assert_eq!(prefix.unwrap(), "tc\\mirrorform", "PBO prefix should match");
}

#[test]
fn test_invalid_pbo_path() {
    let core = PboCore::new(None);
    let invalid_path = PathBuf::from("nonexistent.pbo");
    let result = core.list_contents(&invalid_path);
    
    assert!(result.is_err(), "Should fail with invalid PBO path");
}

#[test]
fn test_extract_result_display() {
    let success_result = ExtractResult {
        return_code: 0,
        stdout: "Test output".to_string(),
        stderr: String::new(),
    };
    
    let error_result = ExtractResult {
        return_code: 1,
        stdout: String::new(),
        stderr: "Test error".to_string(),
    };
    
    assert_eq!(success_result.to_string(), "Test output");
    assert_eq!(error_result.to_string(), "Error (1): Test error");
}

#[test]
fn test_pbo_config_with_custom_settings() {
    let config = PboConfig::default();
    let core = PboCore::new(Some(config));
    let pbo_path = get_test_pbo_path();
    let result = core.list_contents(&pbo_path).expect("Failed to list PBO contents");
    
    assert!(result.is_success());
}