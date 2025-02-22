use crate::common;
use pbo_tools_rs::core::PboCore;
use pbo_tools_rs::core::PboApiOps;
use pbo_tools_rs::core::config::PboConfig;
use pbo_tools_rs::extract::ExtractResult;
use log::debug;

#[test]
fn test_list_contents() {
    common::setup();
    let core = PboCore::new(None);
    let pbo_path = common::get_test_pbo_path();
    debug!("Testing PBO path: {:?}", pbo_path);
    
    let result = core.list_contents(&pbo_path).expect("Failed to list PBO contents");
    debug!("List contents returned: {:?}", result);
    
    assert!(result.is_success());
    let files = result.get_file_list();
    debug!("Parsed file list: {:?}", files);
    
    assert!(!files.is_empty(), "PBO should contain files");
    
    let has_config = files.iter().any(|f| f.contains("config.bin"));
    let has_model = files.iter().any(|f| f.contains("uniform\\mirror.p3d"));
    
    assert!(has_config, "PBO should contain config.bin");
    assert!(has_model, "PBO should contain uniform/mirror.p3d");
}

#[test]
fn test_list_contents_brief() {
    let core = PboCore::new(None);
    let pbo_path = common::get_test_pbo_path();
    let result = core.list_contents_brief(&pbo_path).expect("Failed to list PBO contents briefly");
    
    assert!(result.is_success());
    let files = result.get_file_list();
    assert!(!files.is_empty(), "PBO should contain files");
}

#[test]
fn test_pbo_config_with_custom_settings() {
    let config = PboConfig::default();
    let core = PboCore::new(Some(config));
    let pbo_path = common::get_test_pbo_path();
    let result = core.list_contents(&pbo_path).expect("Failed to list PBO contents");
    
    assert!(result.is_success());
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