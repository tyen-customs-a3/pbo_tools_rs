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

pub struct TestFixtures {
    pub test_data_dir: PathBuf,
}

impl TestFixtures {
    pub fn new() -> Self {
        let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data");
            
        Self { test_data_dir }
    }
    
    pub fn get_test_pbo_path(&self) -> PathBuf {
        self.test_data_dir.join("mirrorform.pbo")
    }
}

impl Default for TestFixtures {
    fn default() -> Self {
        Self::new()
    }
}

// For backwards compatibility
pub fn get_test_pbo_path() -> PathBuf {
    TestFixtures::new().get_test_pbo_path()
}