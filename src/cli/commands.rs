use std::path::PathBuf;
use crate::error::types::Result;
use crate::core::api::PboApiOps;
use crate::extract::ExtractOptions;

pub fn list_contents(api: &dyn PboApiOps, pbo_path: &PathBuf, brief: bool, verbose: bool) -> Result<()> {
    let options = ExtractOptions {
        no_pause: true,
        warnings_as_errors: true,
        brief_listing: brief,
        verbose,
        ..Default::default()
    };
    
    let result = api.list_with_options(pbo_path, options)?;
    println!("{}", result);
    Ok(())
}

pub fn extract_contents(
    api: &dyn PboApiOps,
    pbo_path: &PathBuf,
    output_dir: &PathBuf,
    filter: Option<String>,
    verbose: bool,
    ignore_warnings: bool,
) -> Result<()> {
    let options = ExtractOptions {
        no_pause: true,
        warnings_as_errors: !ignore_warnings,
        file_filter: filter,
        verbose,
        ..Default::default()
    };
    
    let result = api.extract_with_options(pbo_path, output_dir, options)?;
    println!("{}", result);
    Ok(())
}
