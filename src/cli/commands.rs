use std::path::PathBuf;
use crate::error::types::Result;
use crate::core::api::PboApiOps;

pub fn list_contents(api: &dyn PboApiOps, pbo_path: &PathBuf) -> Result<()> {
    let result = api.list_contents(pbo_path)?;
    println!("{}", result);
    Ok(())
}

pub fn extract_contents(
    api: &dyn PboApiOps,
    pbo_path: &PathBuf,
    output_dir: &PathBuf,
    filter: Option<&str>,
) -> Result<()> {
    let result = api.extract_files(pbo_path, output_dir, filter)?;
    println!("{}", result);
    Ok(())
}
