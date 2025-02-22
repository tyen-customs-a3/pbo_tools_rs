use pbo_tools_rs::core::{PboApi, PboApiOps};
use std::path::Path;
use tempfile::TempDir;
use std::fs;
use log::debug;
use std::sync::Once;

static INIT: Once = Once::new();

// Test fixture helper
fn setup() -> (PboApi, TempDir) {
    // Initialize logger
    INIT.call_once(|| {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .ok();
    });
    
    let temp_dir = TempDir::new().unwrap();
    let api = PboApi::builder()
        .with_timeout(30)
        .build();
    (api, temp_dir)
}

#[test]
fn test_list_contents_integration() {
    let (api, _temp_dir) = setup();
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    
    let result = api.list_contents(test_pbo).unwrap();
    assert!(result.is_success());
    assert!(!result.stdout.is_empty());
}

#[test]
fn test_list_contents_brief_integration() {
    let (api, _temp_dir) = setup();
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    
    let result = api.list_contents_brief(test_pbo).unwrap();
    assert!(result.is_success());
    assert!(!result.stdout.is_empty());
}

#[test]
fn test_extract_files_integration() {
    let (api, temp_dir) = setup();
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    let output_dir = temp_dir.path().join("extracted");
    
    let result = api.extract_files(test_pbo, &output_dir, None).unwrap();
    assert!(result.is_success());
    assert!(output_dir.exists());
    
    // Verify that files were actually extracted
    let entries: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!entries.is_empty());
    
    // Log the extracted files
    debug!("Extracted {} files:", entries.len());
    for entry in entries {
        debug!("  - {}", entry.path().display());
    }
}

#[test]
fn test_extract_with_filter_integration() {
    let (api, temp_dir) = setup();
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    let output_dir = temp_dir.path().join("filtered");
    
    // Extract only .paa files
    let result = api.extract_files(test_pbo, &output_dir, Some("*.paa")).unwrap();
    assert!(result.is_success());
    assert!(output_dir.exists());
    
    // Debug: Print the command output
    debug!("ExtractPBO stdout: {}", result.stdout);
    debug!("ExtractPBO stderr: {}", result.stderr);
    
    // Debug: Walk the directory recursively to see all files
    fn walk_dir(dir: &Path, depth: usize) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                let indent = "  ".repeat(depth);
                debug!("{}|- {}", indent, path.display());
                if path.is_dir() {
                    walk_dir(&path, depth + 1);
                }
            }
        }
    }
    
    debug!("Output directory contents:");
    walk_dir(&output_dir, 0);

    // For now, just verify the directory exists and contains something
    let entries: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!entries.is_empty(), "Output directory should not be empty");
}

#[test]
fn test_invalid_pbo_path() {
    let (api, _temp_dir) = setup();
    let invalid_pbo = Path::new("tests/data/nonexistent.pbo");
    
    let result = api.list_contents(invalid_pbo);
    assert!(result.is_err());
}