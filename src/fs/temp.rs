use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use tempfile::{Builder, TempDir};
use uuid::Uuid;
use std::time::Duration;
use crate::error::types::{Result, FileSystemError, PboError};

#[derive(Debug, Clone)]
pub struct TempFileManager {
    temp_dirs: Arc<Mutex<HashSet<PathBuf>>>,
    root_dir: Arc<TempDir>,
}

impl TempFileManager {
    pub fn new() -> Self {
        let root_dir = Builder::new()
            .prefix("pbo_tools_")
            .tempdir()
            .expect("Failed to create root temp directory");
            
        Self {
            temp_dirs: Arc::new(Mutex::new(HashSet::new())),
            root_dir: Arc::new(root_dir),
        }
    }

    pub fn create_temp_dir(&self) -> Result<PathBuf> {
        let unique_name = format!("temp_{}", Uuid::new_v4());
        let path = self.root_dir.path().join(unique_name);
        
        std::fs::create_dir_all(&path).map_err(|e| {
            PboError::FileSystem(FileSystemError::CreateDir {
                path: path.clone(),
                reason: e.to_string(),
            })
        })?;
        
        self.temp_dirs.lock()
            .map_err(|_| PboError::FileSystem(FileSystemError::PathValidation(
                "Failed to lock temp dirs".to_string()
            )))?
            .insert(path.clone());
            
        Ok(path)
    }

    pub fn cleanup_temp_dir(&self, path: &Path) -> Result<()> {
        let mut temp_dirs = self.temp_dirs.lock()
            .map_err(|_| PboError::FileSystem(FileSystemError::PathValidation(
                "Failed to lock temp dirs".to_string()
            )))?;
            
        if temp_dirs.remove(path) {
            if path.exists() {
                std::fs::remove_dir_all(path).map_err(|e| {
                    PboError::FileSystem(FileSystemError::Delete {
                        path: path.to_path_buf(),
                        reason: e.to_string(),
                    })
                })?;
            }
        }
        
        Ok(())
    }
}

impl Default for TempFileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TempFileManager {
    fn drop(&mut self) {
        if let Ok(temp_dirs) = self.temp_dirs.lock() {
            for path in temp_dirs.iter() {
                if path.exists() {
                    let _ = std::fs::remove_dir_all(path);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_temp_dir_cleanup() {
        let manager = TempFileManager::new();
        let temp_path = {
            let temp_dir = manager.create_temp_dir().unwrap();
            let path = temp_dir.to_path_buf();
            assert!(path.exists());
            manager.cleanup_temp_dir(&path).unwrap(); // Add explicit cleanup
            path
        };
        // Now we should verify it's gone
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_temp_dir_expiration() {
        let manager = TempFileManager::new();
        let temp_dir_path = {
            let temp_dir = manager.create_temp_dir().unwrap();
            let path = temp_dir.to_path_buf();
            // Store in active set
            path
        };
        
        thread::sleep(Duration::from_millis(200));
        manager.cleanup_temp_dir(&temp_dir_path).unwrap();
        
        // Check the path was removed from active set
        let guard = manager.temp_dirs.lock().unwrap();
        assert!(!guard.contains(&temp_dir_path));
    }
}
