pub mod cli;
pub mod core;
pub mod error;
pub mod extract;
pub mod fs;

pub use core::api::PboApi;
pub use core::config::PboConfig;
pub use error::types::{PboError, ExtractError, FileSystemError, Result};

/// Version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");