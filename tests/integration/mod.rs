use pbo_tools_rs::core::api::PboApi;
use pbo_tools_rs::core::PboApiOps;

#[test]
fn test_integration() {
    let api = PboApi::new(30);
    assert!(api.list_contents(std::path::Path::new("tests/data/mirrorform.pbo")).is_ok());
}