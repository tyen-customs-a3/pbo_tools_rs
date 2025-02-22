use crate::common;

use pbo_tools_rs::core::api::PboApi;
use pbo_tools_rs::core::PboApiOps;
use pbo_tools_rs::error::types::PboError;
use std::thread;
use tempfile::tempdir;

#[test]
fn test_operation_timeout() {
    let api = PboApi::new(1); // 1 second timeout
    let pbo_path = common::get_test_pbo_path();
    let temp_dir = tempdir().expect("Failed to create temp directory");
    
    // Create a large filter string to force timeout
    let many_filters: Vec<_> = (0..10000).map(|i| format!("file{}.txt", i)).collect();
    let filter = many_filters.join("|");
    
    let result = api.extract_files(&pbo_path, temp_dir.path(), Some(&filter));
    assert!(matches!(result, Err(PboError::Timeout(1))));
}

#[test]
fn test_parallel_operations() {
    let api = PboApi::new(30);
    let pbo_path = common::get_test_pbo_path();
    
    let mut handles = vec![];
    
    // Spawn multiple threads doing operations simultaneously
    for _ in 0..3 {
        let api = api.clone();
        let pbo_path = pbo_path.clone();
        
        let handle = thread::spawn(move || {
            let result = api.list_contents(&pbo_path);
            assert!(result.is_ok());
        });
        
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_extract() {
    let api = PboApi::new(30);
    let pbo_path = common::get_test_pbo_path();
    
    let mut handles = vec![];
    
    // Spawn threads doing different types of operations
    for i in 0..3 {
        let api = api.clone();
        let pbo_path = pbo_path.clone();
        let temp_dir = tempdir().expect("Failed to create temp directory");
        
        let handle = thread::spawn(move || {
            match i {
                0 => api.list_contents(&pbo_path),
                1 => api.list_contents_brief(&pbo_path),
                _ => api.extract_files(&pbo_path, temp_dir.path(), Some("config.bin")),
            }.expect("Operation should succeed");
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}