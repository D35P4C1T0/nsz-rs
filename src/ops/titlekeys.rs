use crate::config::TitleKeysRequest;
use crate::error::NszError;
use crate::ops::OperationReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

/// Runs title key extraction through Python `nsz`.
pub fn run(request: &TitleKeysRequest) -> Result<OperationReport, NszError> {
    if request.files.is_empty() {
        return Ok(OperationReport::default());
    }

    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut args = vec!["--titlekeys".to_string()];
    for file in &request.files {
        args.push(file.display().to_string());
    }

    run_nsz_cli(&repo_root, &args)?;

    Ok(OperationReport {
        processed_files: request.files.clone(),
        skipped_files: Vec::new(),
    })
}
