use pbo_tools_rs::file_ops::TempFileManager;
use std::path::Path;

#[test]
fn test_temp_manager_create() {
    let manager = TempFileManager::new();
    let temp_dir = manager.create_temp_dir().unwrap();
    assert!(temp_dir.exists());
    assert!(temp_dir.path().starts_with(std::env::temp_dir().join("pbo_tools")));
}

#[test]
fn test_temp_manager_multiple_dirs() {
    let manager = TempFileManager::new();
    let dir1 = manager.create_temp_dir().unwrap();
    let dir2 = manager.create_temp_dir().unwrap();
    
    assert!(dir1.exists());
    assert!(dir2.exists());
    assert_ne!(dir1.path(), dir2.path());
}

#[test]
fn test_temp_dir_drop() {
    let temp_dir_path;
    {
        let manager = TempFileManager::new();
        let temp_dir = manager.create_temp_dir().unwrap();
        temp_dir_path = temp_dir.path().to_path_buf();
        assert!(temp_dir.exists());
    } // both manager and temp_dir get dropped here
    assert!(!temp_dir_path.exists());
}

#[test]
fn test_temp_dir_path_methods() {
    let manager = TempFileManager::new();
    let temp_dir = manager.create_temp_dir().unwrap();
    
    // Test AsRef<Path> implementation
    let _: &Path = temp_dir.as_ref();
    
    // Test direct path access
    assert_eq!(temp_dir.path(), temp_dir.as_ref());
}