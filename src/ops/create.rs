use crate::config::CreateRequest;
use crate::error::NszError;
use crate::ops::OperationReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

/// Runs create flow through Python `nsz` and reports the created container path.
pub fn run(request: &CreateRequest) -> Result<OperationReport, NszError> {
    let output_file = request
        .output_file
        .as_ref()
        .ok_or_else(|| NszError::ContainerFormat {
            message: "create request missing output file".to_string(),
        })?;

    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut args = vec!["-c".to_string(), output_file.display().to_string()];
    if request.fix_padding {
        args.push("-F".to_string());
    }
    for source in &request.sources {
        args.push(source.display().to_string());
    }

    run_nsz_cli(&repo_root, &args)?;

    Ok(OperationReport {
        processed_files: vec![output_file.clone()],
        skipped_files: Vec::new(),
    })
}
