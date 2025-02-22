use pbo_tools_rs::core::{PboApi, PboConfig, PboApiOps};
use std::path::Path;

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