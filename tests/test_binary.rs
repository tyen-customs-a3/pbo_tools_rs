use std::path::Path;
use pbo_tools_rs::binary::{BinaryContent, ReadBinaryContent};
use pbo_tools_rs::errors::PboError;

#[test]
fn test_binary_content_from_file() {
    let temp = tempfile::tempdir().unwrap();
    let test_file = temp.path().join("test.bin");
    std::fs::write(&test_file, b"Hello, World!").unwrap();
    
    let content = BinaryContent::from_file(&test_file).unwrap();
    assert_eq!(content.as_ref(), b"Hello, World!");
}

#[test]
fn test_utf8_content() {
    let temp = tempfile::tempdir().unwrap();
    let test_file = temp.path().join("test_utf8.txt");
    std::fs::write(&test_file, "Hello, 世界!").unwrap();
    
    let content = BinaryContent::from_file(&test_file).unwrap();
    assert_eq!(content.decode_text().unwrap(), "Hello, 世界!");
}

#[test]
fn test_windows1252_content() {
    let temp = tempfile::tempdir().unwrap();
    let test_file = temp.path().join("test_win1252.txt");
    std::fs::write(&test_file, &[0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0xA3]).unwrap(); // "Hello, £"
    
    let content = BinaryContent::from_file(&test_file).unwrap();
    assert_eq!(content.decode_text().unwrap(), "Hello, £");
}

#[test]
fn test_binary_detection() {
    let temp = tempfile::tempdir().unwrap();
    let test_file = temp.path().join("test.bin");
    // Create file with binary content (null bytes and 0xFF sequences)
    std::fs::write(&test_file, &[0x00, 0x00, 0xFF, 0xFF, 0x00]).unwrap();
    
    let content = BinaryContent::from_file(&test_file).unwrap();
    assert!(matches!(content.decode_text(), Err(PboError::Encoding { .. })));
}

#[test]
fn test_read_content_trait() {
    let temp = tempfile::tempdir().unwrap();
    let test_file = temp.path().join("test.txt");
    std::fs::write(&test_file, "Hello, World!").unwrap();
    
    let content = test_file.read_content().unwrap();
    assert_eq!(content, "Hello, World!");
}

#[test]
fn test_nonexistent_file() {
    let result = Path::new("nonexistent.txt").read_content();
    assert!(matches!(result, Err(PboError::FileSystem(_))));
}

#[test]
fn test_empty_file() {
    let temp = tempfile::tempdir().unwrap();
    let test_file = temp.path().join("empty.txt");
    std::fs::write(&test_file, "").unwrap();
    
    let content = test_file.read_content().unwrap();
    assert_eq!(content, "");
}