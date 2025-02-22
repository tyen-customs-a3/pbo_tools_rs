use std::path::PathBuf;
use std::sync::Once;
use env_logger;
use log::LevelFilter;

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .ok();
    });
}

pub fn get_test_pbo_path() -> PathBuf {
    PathBuf::from("tests/data/mirrorform.pbo")
}

pub fn get_test_data_dir() -> PathBuf {
    PathBuf::from("tests/data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_pbo_path() {
        let path = get_test_pbo_path();
        assert!(path.exists(), "Test PBO file should exist");
        assert!(path.is_file(), "Test PBO path should be a file");
    }
}