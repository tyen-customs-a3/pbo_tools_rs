pub mod api;
pub mod binary;
pub mod config;
pub mod core;
pub mod errors;
pub mod extract;
pub mod file_ops;

pub use api::PboApi;
pub use config::PboConfig;
pub use errors::{PboError, ExtractError, FileSystemError, Result};

pub type FileContent = std::collections::HashMap<String, String>;

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");