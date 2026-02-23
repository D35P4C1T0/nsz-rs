pub mod compress;
pub mod decompress;
pub mod verify;

use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct OperationReport {
    pub processed_files: Vec<PathBuf>,
    pub skipped_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    pub verified_files: Vec<PathBuf>,
}
