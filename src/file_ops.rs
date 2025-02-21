use std::path::{Path, PathBuf};
use std::collections::HashSet;
use std::sync::Mutex;
use std::ops::Deref;
use log::warn;
use uuid::Uuid;
use crate::errors::{PboError, FileSystemError, Result};

#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
    _guard: (),  // Private field to prevent construction outside of this module
}

impl TempDir {
    fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let path = base_path.as_ref().join(format!("extract_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path)
            .map_err(|e| PboError::FileSystem(FileSystemError::CreateDir {
                path: path.clone(),
                reason: e.to_string(),
            }))?;
        Ok(Self { 
            path,
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
}

impl Drop for TempDir {
    fn drop(&mut self) {
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

#[derive(Debug, Default)]
pub struct TempFileManager {
    temp_base: PathBuf,
    active_dirs: Mutex<HashSet<PathBuf>>,
}

impl TempFileManager {
    pub fn new() -> Self {
        let temp_base = std::env::temp_dir().join("pbo_tools");
        Self {
            temp_base,
            active_dirs: Mutex::new(HashSet::new()),
        }
    }

    pub fn create_temp_dir(&self) -> Result<TempDir> {
        std::fs::create_dir_all(&self.temp_base)
            .map_err(|e| PboError::FileSystem(FileSystemError::CreateDir {
                path: self.temp_base.clone(),
                reason: e.to_string(),
            }))?;

        let temp_dir = TempDir::new(&self.temp_base)?;
        self.active_dirs.lock()
            .map_err(|e| PboError::FileSystem(FileSystemError::CreateDir {
                path: self.temp_base.clone(),
                reason: format!("Failed to acquire lock: {}", e),
            }))?
            .insert(temp_dir.path().to_path_buf());
        
        Ok(temp_dir)
    }

    pub fn temp_base(&self) -> &Path {
        &self.temp_base
    }

    #[cfg(test)]
    pub(crate) fn active_dir_count(&self) -> usize {
        self.active_dirs.lock().unwrap().len()
    }
}