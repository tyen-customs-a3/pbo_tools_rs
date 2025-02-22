mod binary;
mod temp;
mod traits;

pub use binary::{convert_binary_file, process_binary_files};
pub use temp::TempFileManager;
pub use traits::FileOperation;