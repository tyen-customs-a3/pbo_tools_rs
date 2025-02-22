use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::sync::Mutex;
use std::ops::Deref;
use std::time::{Duration, SystemTime};
use log::{warn, debug};
use uuid::Uuid;
use crate::error::types::{PboError, FileSystemError, Result};

#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
    created_at: SystemTime,
    _guard: (),
}

impl TempDir {
    fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let uuid = Uuid::new_v4();
        let path = base_path.as_ref().join(format!(".tmp{}", uuid.simple()));
        std::fs::create_dir_all(&path)
            .map_err(|e| PboError::FileSystem(FileSystemError::CreateDir {
                path: path.clone(),
                reason: e.to_string(),
            }))?;
            
        debug!("Created temp directory: {}", path.display());
        
        Ok(Self { 
            path,
            created_at: SystemTime::now(),
            _guard: (), 
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn join(&self, path: impl AsRef<Path>) -> PathBuf {
        self.path.join(path)
    }

    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::from_secs(0))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        debug!("Cleaning up temp directory: {}", self.path.display());
        if let Err(e) = std::fs::remove_dir_all(&self.path) {
            warn!("Failed to cleanup temp dir {}: {}", self.path.display(), e);
        }
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Deref for TempDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

#[derive(Debug)]
pub struct TempFileManager {
    temp_base: PathBuf,
    active_dirs: Mutex<HashSet<PathBuf>>,
    max_age: Duration,
}

impl Clone for TempFileManager {
    fn clone(&self) -> Self {
        let current_dirs = self.active_dirs.lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        
        Self {
            temp_base: self.temp_base.clone(),
            active_dirs: Mutex::new(current_dirs),
            max_age: self.max_age,
        }
    }
}

impl Default for TempFileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TempFileManager {
    pub fn new() -> Self {
        Self::with_max_age(Duration::from_secs(3600)) // 1 hour default
    }

    pub fn with_max_age(max_age: Duration) -> Self {
        let temp_base = std::env::temp_dir()
            .join("pbo_tools")
            .canonicalize()
            .unwrap_or_else(|_| std::env::temp_dir().join("pbo_tools"));
            
        debug!("Initialized TempFileManager with base path: {}", temp_base.display());
        
        Self {
            temp_base,
            active_dirs: Mutex::new(HashSet::new()),
            max_age,
        }
    }

    pub fn create_temp_dir(&self) -> Result<TempDir> {
        self.cleanup_old_dirs()?;
        
        std::fs::create_dir_all(&self.temp_base)
            .map_err(|e| PboError::FileSystem(FileSystemError::CreateDir {
                path: self.temp_base.clone(),
                reason: e.to_string(),
            }))?;

        let temp_dir = TempDir::new(&self.temp_base)?;
        
        if let Ok(mut active_dirs) = self.active_dirs.lock() {
            active_dirs.insert(temp_dir.path().to_path_buf());
            debug!("Added temp directory to active set: {}", temp_dir.path().display());
        } else {
            warn!("Failed to acquire lock for active_dirs, directory won't be tracked");
        }
        
        Ok(temp_dir)
    }

    fn cleanup_old_dirs(&self) -> Result<()> {
        if let Ok(mut active_dirs) = self.active_dirs.lock() {
            let expired: Vec<_> = active_dirs
                .iter()
                .filter(|path| {
                    path.metadata()
                        .and_then(|m| m.created())
                        .map(|created| {
                            SystemTime::now()
                                .duration_since(created)
                                .map(|age| age > self.max_age)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
                })
                .cloned()
                .collect();

            for path in expired {
                debug!("Removing expired temp directory: {}", path.display());
                if let Err(e) = std::fs::remove_dir_all(&path) {
                    warn!("Failed to remove expired temp dir {}: {}", path.display(), e);
                }
                active_dirs.remove(&path);
            }
        }
        
        Ok(())
    }

    pub fn temp_base(&self) -> &Path {
        &self.temp_base
    }

    #[cfg(test)]
    pub(crate) fn active_dir_count(&self) -> usize {
        self.active_dirs.lock().unwrap().len()
    }

    #[cfg(test)]
    pub(crate) fn set_max_age(&mut self, max_age: Duration) {
        self.max_age = max_age;
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
            let path = temp_dir.path().to_path_buf();
            assert!(temp_dir.exists());
            path
        };
        // temp_dir is dropped here
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_temp_dir_expiration() {
        let manager = TempFileManager::with_max_age(Duration::from_millis(100));
        let temp_dir_path = {
            let temp_dir = manager.create_temp_dir().unwrap();
            let path = temp_dir.path().to_path_buf();
            // Store in active set
            path
        };
        
        thread::sleep(Duration::from_millis(200));
        manager.cleanup_old_dirs().unwrap();
        
        // Check the path was removed from active set
        let guard = manager.active_dirs.lock().unwrap();
        assert!(!guard.contains(&temp_dir_path));
    }
}
