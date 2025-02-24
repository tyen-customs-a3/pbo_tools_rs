use pbo_tools::core::{PboConfig, PboApi, PboApiOps};
use pbo_tools::fs::{convert_binary_file, process_binary_files};
use std::path::Path;
use tempfile::TempDir;
use std::fs;
use log::{info, debug};

fn init() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

#[test]
fn test_binary_file_conversion() {
    init();
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.bin");
    let output_path = temp_dir.path().join("test.cpp");

    debug!("Creating test file at {:?}", input_path);
    fs::write(&input_path, "test content").unwrap();
    assert!(input_path.exists(), "Input file should exist: {:?}", input_path);

    debug!("Converting binary file");
    convert_binary_file(&input_path, &output_path).unwrap();

    debug!("Verifying file states after conversion");
    assert!(!input_path.exists(), "Original file should not exist: {:?}", input_path);
    assert!(output_path.exists(), "New file should exist: {:?}", output_path);
    
    if let Ok(content) = fs::read_to_string(&output_path) {
        debug!("Output file contents: {}", content);
    }
}

#[test]
fn test_binary_file_batch_processing() {
    init();
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    
    debug!("Creating test directory at {:?}", bin_dir);
    fs::create_dir(&bin_dir).unwrap();

    // Create test files
    let files = [
        ("config.bin", "cpp"),
        ("script.bin", "cpp"),
        ("model.bin", "cfg"),
    ];

    for (bin_name, _) in &files {
        let bin_path = bin_dir.join(bin_name);
        debug!("Creating test file: {:?}", bin_path);
        fs::write(&bin_path, "content").unwrap();
        assert!(bin_path.exists(), "Test file should exist: {:?}", bin_path);
    }

    let config = PboConfig::builder()
        .add_bin_mapping("config.bin", "cpp")
        .add_bin_mapping("script.bin", "cpp")
        .add_bin_mapping("model.bin", "cfg")
        .build();

    debug!("Processing binary files");
    process_binary_files(&bin_dir, &config).unwrap();

    // Verify renames
    for (bin_name, target_ext) in &files {
        let bin_path = bin_dir.join(bin_name);
        let stem = Path::new(bin_name).file_stem().unwrap().to_str().unwrap();
        let target_path = bin_dir.join(format!("{}.{}", stem, target_ext));

        debug!("Verifying file states for {}:", bin_name);
        debug!("  Source: {:?} exists: {}", bin_path, bin_path.exists());
        debug!("  Target: {:?} exists: {}", target_path, target_path.exists());

        assert!(!bin_path.exists(), "Binary file should be removed: {:?}", bin_path);
        assert!(target_path.exists(), "Target file should exist: {:?}", target_path);
    }
}

#[test]
fn test_binary_conversion_with_custom_mappings() {
    init();
    let temp_dir = TempDir::new().unwrap();
    let source_dir = temp_dir.path();
    let input_path = source_dir.join("custom.bin");
    
    debug!("Creating test file at {:?}", input_path);
    fs::write(&input_path, "test content").unwrap();
    assert!(input_path.exists(), "Input file should exist: {:?}", input_path);
    
    let config = PboConfig::builder()
        .add_bin_mapping("custom.bin", "txt")
        .build();

    debug!("Processing binary files with custom mapping");
    process_binary_files(source_dir, &config).unwrap();

    let output_path = source_dir.join("custom.txt");
    
    debug!("Verifying file states:");
    debug!("  Source: {:?} exists: {}", input_path, input_path.exists());
    debug!("  Target: {:?} exists: {}", output_path, output_path.exists());

    assert!(!input_path.exists(), "Binary file should be removed: {:?}", input_path);
    assert!(output_path.exists(), "Target file should exist: {:?}", output_path);
}