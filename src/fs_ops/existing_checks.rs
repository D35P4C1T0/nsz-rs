use std::fs;
use std::path::{Path, PathBuf};

use crate::error::NszError;

use super::path_tools::change_extension;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteDecision {
    Allow,
    AllowOverwrite,
    DenyDuplicate,
}

pub fn target_path_for(source_file: &Path, target_extension: &str, target_dir: &Path) -> PathBuf {
    let file_name = source_file.file_name().unwrap_or_default();
    change_extension(&target_dir.join(file_name), target_extension)
}

pub fn allow_write_outfile(
    source_file: &Path,
    target_extension: &str,
    target_dir: &Path,
    overwrite: bool,
) -> Result<WriteDecision, NszError> {
    let target = target_path_for(source_file, target_extension, target_dir);

    if !target.exists() {
        return Ok(WriteDecision::Allow);
    }

    if overwrite {
        fs::remove_file(&target)?;
        return Ok(WriteDecision::AllowOverwrite);
    }

    Ok(WriteDecision::DenyDuplicate)
}
