use crate::common;

use pbo_tools_rs::core::api::PboApi;
use pbo_tools_rs::core::PboApiOps;
use pbo_tools_rs::core::config::PboConfig;

#[test]
fn test_builder_default() {
    let api = PboApi::builder().build();
    let pbo_path = common::get_test_pbo_path();
    let result = api.list_contents(&pbo_path);
    assert!(result.is_ok());
}

#[test]
fn test_builder_with_config() {
    let api = PboApi::builder()
        .with_config(PboConfig::default())
        .build();
    
    let pbo_path = common::get_test_pbo_path();
    let result = api.list_contents(&pbo_path);
    assert!(result.is_ok());
}

#[test]
fn test_builder_chaining_order() {
    let api = PboApi::builder()
        .with_timeout(30)
        .with_config(PboConfig::default())
        .with_timeout(60)
        .build();
        
    let pbo_path = common::get_test_pbo_path();
    let result = api.list_contents(&pbo_path);
    assert!(result.is_ok());
}

#[test]
fn test_builder_zero_timeout() {
    // Should use default timeout when zero is provided
    let api = PboApi::builder()
        .with_timeout(0)
        .build();
        
    let pbo_path = common::get_test_pbo_path();
    let result = api.list_contents(&pbo_path);
    assert!(result.is_ok());
}