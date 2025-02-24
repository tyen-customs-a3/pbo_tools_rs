# PBO Tools RS

A Rust library and CLI tool for working with PBO (Packed Binary Object) files. This toolkit provides functionality for listing, extracting, and managing PBO files with robust error handling and configuration options.

## Features

- List contents of PBO files (standard and brief formats)
- Extract files with optional filtering
- Binary file conversion handling
- Configurable timeout and retry mechanisms
- Progress tracking and detailed logging
- Error handling and validation
- Temporary file management

## Prerequisites

Before using this tool, ensure you have:
- Rust toolchain installed
- Mikero's Tools ExtractPbo binary in your system PATH
  - Download from: [Mikero's Tools](https://mikero.bytex.digital/Downloads)
  - Add the installation directory to your system's PATH environment variable

## Installation

To install the CLI tool:

```bash
cargo install pbo_tools
```

## Usage

### Command Line Interface

List contents of a PBO file:
```bash
pbo_tools list path/to/file.pbo
```

Extract files from a PBO:
```bash
pbo_tools extract path/to/file.pbo output/dir
```

Options:
- `--brief` - Use brief directory-style output listing
- `--verbose` - Enable verbose output
- `--filter` - Extract specific files (supports wildcards)
- `--ignore-warnings` - Don't treat warnings as errors
- `--timeout` - Set operation timeout in seconds

### Library Usage

Basic usage:
```rust
use pbo_tools::core::PboApi;
use std::path::Path;

let api = PboApi::builder()
    .with_timeout(30)
    .build();

// List contents
let pbo_path = Path::new("mission.pbo");
let result = api.list_contents(&pbo_path).unwrap();
println!("Files in PBO: {:?}", result.get_file_list());

// Extract specific files
let output_dir = Path::new("output");
api.extract_files(&pbo_path, &output_dir, Some("*.cpp")).unwrap();
```

Advanced configuration:
```rust
use pbo_tools::core::{PboApi, PboConfig};

let config = PboConfig::builder()
    .case_sensitive(true)
    .max_retries(5)
    .build();

let api = PboApi::builder()
    .with_config(config)
    .with_timeout(30)
    .build();
```

## Project Structure

- `src/cli` - Command-line interface implementation
- `src/core` - Core API and configuration
- `src/error` - Error types and handling
- `src/extract` - PBO extraction functionality
- `src/fs` - File system operations
- `tests` - Integration and unit tests

## Error Handling

The library uses custom error types for different scenarios:
- `PboError` - Main error enum
- `ExtractError` - Extraction-specific errors
- `FileSystemError` - File system operation errors

All operations return a `Result` type for proper error handling.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under MIT