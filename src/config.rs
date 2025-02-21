use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PboConfig {
    bin_file_types: HashMap<String, String>,
    bad_pbo_indicators: Vec<String>,
}

impl PboConfig {
    pub fn builder() -> PboConfigBuilder {
        PboConfigBuilder::new()
    }

    pub fn get_bin_extension(&self, filename: &str) -> Option<&str> {
        self.bin_file_types.get(filename).map(|s| s.as_str())
    }

    pub fn is_bad_pbo(&self, message: &str) -> bool {
        self.bad_pbo_indicators.iter().any(|i| message.contains(i))
    }
}

#[derive(Default)]
pub struct PboConfigBuilder {
    bin_file_types: HashMap<String, String>,
    bad_pbo_indicators: Vec<String>,
}

impl PboConfigBuilder {
    pub fn new() -> Self {
        let mut builder = Self::default();
        // Set default mappings
        builder.bin_file_types.insert("config.bin".to_string(), "config.cpp".to_string());
        builder.bin_file_types.insert("model.bin".to_string(), "model.cfg".to_string());
        builder.bin_file_types.insert("stringtable.bin".to_string(), "stringtable.xml".to_string());
        builder.bin_file_types.insert("texheaders.bin".to_string(), "texheaders.txt".to_string());
        builder.bin_file_types.insert("script.bin".to_string(), "script.cpp".to_string());
        builder.bin_file_types.insert("default".to_string(), ".txt".to_string());

        // Set default bad PBO indicators
        builder.bad_pbo_indicators = vec![
            "DePbo:Pbo unknown header type",
            "Bad Sha detected",
            "Bad Sha",
            "this warning is set as an error",
        ].into_iter().map(String::from).collect();

        builder
    }

    pub fn add_bin_mapping(mut self, bin_file: impl Into<String>, target_ext: impl Into<String>) -> Self {
        self.bin_file_types.insert(bin_file.into(), target_ext.into());
        self
    }

    pub fn add_bad_indicator(mut self, indicator: impl Into<String>) -> Self {
        self.bad_pbo_indicators.push(indicator.into());
        self
    }

    pub fn build(self) -> PboConfig {
        PboConfig {
            bin_file_types: self.bin_file_types,
            bad_pbo_indicators: self.bad_pbo_indicators,
        }
    }
}

impl Default for PboConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PboConfig::default();
        assert!(config.get_bin_extension("config.bin").is_some());
        assert_eq!(config.get_bin_extension("config.bin"), Some("config.cpp"));
        assert_eq!(config.get_bin_extension("unknown.bin"), None);
    }

    #[test]
    fn test_custom_config() {
        let config = PboConfig::builder()
            .add_bin_mapping("custom.bin", "custom.txt")
            .add_bad_indicator("Custom bad indicator")
            .build();

        assert_eq!(config.get_bin_extension("custom.bin"), Some("custom.txt"));
        assert!(config.is_bad_pbo("Custom bad indicator"));
        assert!(!config.is_bad_pbo("Unrelated message"));
    }

    #[test]
    fn test_default_bad_indicators() {
        let config = PboConfig::default();
        assert!(config.is_bad_pbo("DePbo:Pbo unknown header type"));
        assert!(config.is_bad_pbo("Bad Sha detected"));
        assert!(!config.is_bad_pbo("Normal message"));
    }

    #[test]
    fn test_default_bin_mappings() {
        let config = PboConfig::default();
        let expected_mappings = [
            ("config.bin", "config.cpp"),
            ("model.bin", "model.cfg"),
            ("stringtable.bin", "stringtable.xml"),
            ("texheaders.bin", "texheaders.txt"),
            ("script.bin", "script.cpp"),
        ];

        for (bin, expected) in expected_mappings {
            assert_eq!(config.get_bin_extension(bin), Some(expected));
        }
    }

    #[test]
    fn test_builder_chaining() {
        let config = PboConfig::builder()
            .add_bin_mapping("test1.bin", "test1.txt")
            .add_bin_mapping("test2.bin", "test2.txt")
            .add_bad_indicator("indicator1")
            .add_bad_indicator("indicator2")
            .build();

        assert_eq!(config.get_bin_extension("test1.bin"), Some("test1.txt"));
        assert_eq!(config.get_bin_extension("test2.bin"), Some("test2.txt"));
        assert!(config.is_bad_pbo("indicator1"));
        assert!(config.is_bad_pbo("indicator2"));
    }
}