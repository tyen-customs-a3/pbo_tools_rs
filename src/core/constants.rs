pub const DEFAULT_TIMEOUT: u32 = 30;

/// Default binary file mappings for PBO files
pub const DEFAULT_BIN_MAPPINGS: &[(&str, &str)] = &[
    ("config.bin", "config.cpp"),
    ("model.bin", "model.cfg"),
    ("stringtable.bin", "stringtable.xml"),
    ("texheaders.bin", "texheaders.txt"),
    ("script.bin", "script.cpp"),
];

/// Known warnings that should not fail the operation
pub const KNOWN_WARNINGS: &[&str] = &[
    // Normal warning indicating non-standard header fields
    "1st/last entry has non-zero real_size",
    // Warning about non-zero reserved fields (usually harmless)
    "reserved field non zero",
    // Missing SHA key (common in older PBOs)
    "no shakey on arma",
    // Missing prefix (common in mission PBOs)
    "arma pbo is missing a prefix",
];

/// Indicators that a PBO is corrupted or invalid
pub const BAD_PBO_INDICATORS: &[&str] = &[
    // Unknown PBO header type
    "DePbo:Pbo unknown header type",
    // SHA validation failures
    "Bad Sha detected",
    "Bad Sha",
    // Warning treated as error (configurable)
    "this warning is set as an error",
    // Corrupted file structure
    "residual bytes in file",
    // File access errors
    "Cannot open",
    // General operation failures
    "Error",
    "Failed",
];

/// Default retry count for operations
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Common file extensions in PBOs
pub const COMMON_PBO_EXTENSIONS: &[&str] = &["pbo", "xbo", "ifa"];

/// Common binary file extensions that may need conversion
pub const BINARY_EXTENSIONS: &[&str] = &["bin", "binpbo", "binconfig"];