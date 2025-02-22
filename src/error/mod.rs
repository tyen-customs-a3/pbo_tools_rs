pub mod types;

#[macro_export]
macro_rules! fs_err {
    // Basic case: operation and path
    ($op:ident, $path:expr) => {
        $op.map_err(|e| PboError::FileSystem(FileSystemError::$op {
            path: $path.to_path_buf(),
            reason: e.to_string(),
        }))
    };
    
    // Extended case: operation, path, and custom reason
    ($op:ident, $path:expr, $reason:expr) => {
        Err(PboError::FileSystem(FileSystemError::$op {
            path: $path.to_path_buf(),
            reason: $reason.to_string(),
        }))
    };
    
    // Copy/Rename case: operation with from and to paths
    ($op:ident, $from:expr, $to:expr) => {
        $op.map_err(|e| PboError::FileSystem(FileSystemError::$op {
            from: $from.to_path_buf(),
            to: $to.to_path_buf(),
            reason: e.to_string(),
        }))
    };
    
    // Copy/Rename case with custom reason
    ($op:ident, $from:expr, $to:expr, $reason:expr) => {
        Err(PboError::FileSystem(FileSystemError::$op {
            from: $from.to_path_buf(),
            to: $to.to_path_buf(),
            reason: $reason.to_string(),
        }))
    };
}

pub use types::*;