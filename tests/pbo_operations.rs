use pbo_tools_rs::core::{PboApi, PboApiOps};
use pbo_tools_rs::extract::ExtractOptions;
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
    
    // Debug: Print the output directory structure
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

    // Verify directory contains extracted files
    let entries: Vec<_> = walkdir::WalkDir::new(&output_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();
        
    assert!(!entries.is_empty(), "Output directory should not be empty");
    // Verify all extracted files match the filter or are $PBOPREFIX$.txt
    for entry in entries {
        let is_paa = entry.path().extension().map_or(false, |ext| ext == "paa");
        let is_pboprefix = entry.file_name().to_string_lossy() == "$PBOPREFIX$.txt";
        assert!(is_paa || is_pboprefix, 
            "Found file that doesn't match filter and isn't $PBOPREFIX$.txt: {}", 
            entry.path().display());
    }
}

#[test]
fn test_extract_with_custom_options() {
    let (api, temp_dir) = setup();
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    let output_dir = temp_dir.path().join("extracted_custom");
    
    let options = ExtractOptions {
        no_pause: true,
        warnings_as_errors: true,
        verbose: true,
        ..Default::default()
    };
    
    let result = api.extract_with_options(test_pbo, &output_dir, options).unwrap();
    assert!(result.is_success());
    assert!(output_dir.exists());
}

#[test]
fn test_list_with_custom_options() {
    let (api, _temp_dir) = setup();
    let test_pbo = Path::new("tests/data/mirrorform.pbo");
    
    let options = ExtractOptions {
        no_pause: true,
        warnings_as_errors: true,
        verbose: true,
        brief_listing: true,
        ..Default::default()
    };
    
    let result = api.list_with_options(test_pbo, options).unwrap();
    assert!(result.is_success());
    assert!(!result.stdout.is_empty());
    
    let files = result.get_file_list();
    assert!(!files.is_empty());
}