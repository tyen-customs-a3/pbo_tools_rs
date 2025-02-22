use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PboConfig {
    bin_file_types: HashMap<String, String>,
    bad_pbo_indicators: Vec<String>,
    case_sensitive: bool,
    ignore_path_validation: bool,
    max_retries: u32,
}

impl PboConfig {
    pub fn builder() -> PboConfigBuilder {
        PboConfigBuilder::new()
    }

    pub fn get_bin_extension(&self, filename: &str) -> Option<&str> {
        let lookup_name = if self.case_sensitive {
            filename.to_string()
        } else {
            filename.to_lowercase()
        };
        self.bin_file_types.get(&lookup_name).map(|s| s.as_str())
    }

    pub fn is_bad_pbo(&self, message: &str) -> bool {
        self.bad_pbo_indicators.iter().any(|i| message.contains(i))
    }

    pub fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    pub fn should_ignore_path_validation(&self) -> bool {
        self.ignore_path_validation
    }

    pub fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

#[derive(Default)]
pub struct PboConfigBuilder {
    bin_file_types: HashMap<String, String>,
    bad_pbo_indicators: Vec<String>,
    case_sensitive: bool,
    ignore_path_validation: bool,
    max_retries: u32,
}

impl PboConfigBuilder {
    pub fn new() -> Self {
        let mut builder = Self {
            case_sensitive: false,
            ignore_path_validation: false,
            max_retries: 3,
            ..Default::default()
        };

        // Set default mappings
        let default_mappings = [
            ("config.bin", "config.cpp"),
            ("model.bin", "model.cfg"),
            ("stringtable.bin", "stringtable.xml"),
            ("texheaders.bin", "texheaders.txt"),
            ("script.bin", "script.cpp"),
            ("default", ".txt"),
        ];

        for (bin, ext) in default_mappings.iter() {
            builder.bin_file_types.insert(bin.to_string(), ext.to_string());
        }

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
        let key = if !self.case_sensitive {
            bin_file.into().to_lowercase()
        } else {
            bin_file.into()
        };
        self.bin_file_types.insert(key, target_ext.into());
        self
    }

    pub fn add_bad_indicator(mut self, indicator: impl Into<String>) -> Self {
        self.bad_pbo_indicators.push(indicator.into());
        self
    }

    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    pub fn ignore_path_validation(mut self, ignore: bool) -> Self {
        self.ignore_path_validation = ignore;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn build(self) -> PboConfig {
        PboConfig {
            bin_file_types: self.bin_file_types,
            bad_pbo_indicators: self.bad_pbo_indicators,
            case_sensitive: self.case_sensitive,
            ignore_path_validation: self.ignore_path_validation,
            max_retries: self.max_retries,
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
        assert!(!config.is_case_sensitive());
        assert!(!config.should_ignore_path_validation());
        assert_eq!(config.max_retries(), 3);
        assert_eq!(config.get_bin_extension("config.bin"), Some("config.cpp"));
        assert_eq!(config.get_bin_extension("unknown.bin"), None);
    }

    #[test]
    fn test_case_sensitivity() {
        let config = PboConfig::builder()
            .case_sensitive(true)
            .add_bin_mapping("Test.bin", "test.txt")
            .build();

        assert!(config.is_case_sensitive());
        assert_eq!(config.get_bin_extension("Test.bin"), Some("test.txt"));
        assert_eq!(config.get_bin_extension("test.bin"), None);

        let config = PboConfig::builder()
            .case_sensitive(false)
            .add_bin_mapping("Test.bin", "test.txt")
            .build();

        assert!(!config.is_case_sensitive());
        assert_eq!(config.get_bin_extension("Test.bin"), Some("test.txt"));
        assert_eq!(config.get_bin_extension("test.bin"), Some("test.txt"));
    }

    #[test]
    fn test_custom_config() {
        let config = PboConfig::builder()
            .add_bin_mapping("custom.bin", "custom.txt")
            .add_bad_indicator("Custom bad indicator")
            .max_retries(5)
            .build();

        assert_eq!(config.get_bin_extension("custom.bin"), Some("custom.txt"));
        assert!(config.is_bad_pbo("Custom bad indicator"));
        assert!(!config.is_bad_pbo("Unrelated message"));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_path_validation() {
        let config = PboConfig::builder()
            .ignore_path_validation(true)
            .build();

        assert!(config.should_ignore_path_validation());
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
            .case_sensitive(true)
            .max_retries(10)
            .build();

        assert_eq!(config.get_bin_extension("test1.bin"), Some("test1.txt"));
        assert_eq!(config.get_bin_extension("test2.bin"), Some("test2.txt"));
        assert!(config.is_bad_pbo("indicator1"));
        assert!(config.is_bad_pbo("indicator2"));
        assert!(config.is_case_sensitive());
        assert_eq!(config.max_retries(), 10);
    }
}
