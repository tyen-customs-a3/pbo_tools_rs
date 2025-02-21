use std::path::PathBuf;
use assert_fs::TempDir;

pub const SAMPLE_PBO_NAME: &str = "mirrorform.pbo";
pub const EXPECTED_PREFIX: &str = "tc/mirrorform";

pub fn setup_test_data() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap();
    let data_dir = temp.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();
    
    // Copy test PBO file from original project's test data
    let source_pbo = PathBuf::from("data").join(SAMPLE_PBO_NAME);
    let target_pbo = data_dir.join(SAMPLE_PBO_NAME);
    std::fs::copy(source_pbo, &target_pbo).unwrap();
    
    (temp, data_dir)
}