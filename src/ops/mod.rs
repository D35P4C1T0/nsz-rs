pub mod compress;
pub mod create;
pub mod decompress;
pub mod extract;
pub mod titlekeys;
pub mod undupe;
pub mod verify;

use std::path::PathBuf;

/// Common report returned by non-verify operations.
#[derive(Debug, Clone, Default)]
pub struct OperationReport {
    /// Files successfully processed by the operation.
    pub processed_files: Vec<PathBuf>,
    /// Files intentionally skipped by the operation.
    pub skipped_files: Vec<PathBuf>,
}

/// Report returned by verify operations.
#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    /// Files successfully verified by the operation.
    pub verified_files: Vec<PathBuf>,
}
