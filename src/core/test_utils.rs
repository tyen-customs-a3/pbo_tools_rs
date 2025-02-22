use std::path::PathBuf;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;
use super::api::PboApi;
use crate::error::types::Result;

pub struct TestFixture {
    pub temp_dir: TempDir,
    pub api: PboApi,
}

impl TestFixture {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let api = PboApi::builder()
            .with_timeout(30)
            .build();
        Self { temp_dir, api }
    }

    pub fn create_test_pbo(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        File::create(&path)
            .and_then(|mut file| file.write_all(content))
            .expect("Failed to create test PBO");
        path
    }

    pub fn create_test_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
        File::create(&path)
            .and_then(|mut file| file.write_all(content))
            .expect("Failed to create test file");
        path
    }

    pub fn create_test_dir(&self, name: &str) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        fs::create_dir_all(&path).expect("Failed to create test directory");
        path
    }

    pub fn cleanup(self) -> Result<()> {
        Ok(()) // TempDir handles cleanup automatically when dropped
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_fixture_creation() {
        let fixture = TestFixture::new();
        assert!(fixture.temp_dir.path().exists());
    }

    #[test]
    fn test_create_test_pbo() {
        let fixture = TestFixture::new();
        let test_data = b"test pbo content";
        let pbo_path = fixture.create_test_pbo("test.pbo", test_data);
        
        assert!(pbo_path.exists());
        let content = fs::read(&pbo_path).unwrap();
        assert_eq!(content, test_data);
    }

    #[test]
    fn test_create_test_file() {
        let fixture = TestFixture::new();
        let test_data = b"test file content";
        let file_path = fixture.create_test_file("subdir/test.txt", test_data);
        
        assert!(file_path.exists());
        assert!(file_path.parent().unwrap().exists());
        let content = fs::read(&file_path).unwrap();
        assert_eq!(content, test_data);
    }

    #[test]
    fn test_create_test_dir() {
        let fixture = TestFixture::new();
        let dir_path = fixture.create_test_dir("nested/test/dir");
        
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_cleanup() {
        let fixture = TestFixture::new();
        let dir_path = fixture.temp_dir.path().to_owned();
        fixture.cleanup().unwrap();
        assert!(!dir_path.exists());
    }
}