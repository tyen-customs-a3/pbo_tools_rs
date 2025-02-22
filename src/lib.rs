pub mod cli;
pub mod core;
pub mod error;
pub mod extract;
pub mod fs;
#[cfg(test)]
pub mod test_utils;

// Re-export commonly used types for easier access
pub use core::{
    api::{PboApi, PboApiOps},
    config::PboConfig,
    constants::{DEFAULT_TIMEOUT, DEFAULT_MAX_RETRIES},
};
pub use error::types::{PboError, ExtractError, FileSystemError, Result};
pub use extract::{ExtractOptions, ExtractResult};

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");