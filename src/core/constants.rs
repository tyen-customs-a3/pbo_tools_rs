pub const DEFAULT_TIMEOUT: u32 = 30;
pub const DEFAULT_BIN_MAPPINGS: &[(&str, &str)] = &[
    ("config.bin", "config.cpp"),
    ("model.bin", "model.cfg"),
    ("stringtable.bin", "stringtable.xml"),
    ("texheaders.bin", "texheaders.txt"),
    ("script.bin", "script.cpp"),
];

pub const BAD_PBO_INDICATORS: &[&str] = &[
    "DePbo:Pbo unknown header type",
    "Bad Sha detected",
    "Bad Sha",
    "this warning is set as an error",
];