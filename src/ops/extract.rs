use std::fs;
use std::path::PathBuf;

use crate::config::ExtractRequest;
use crate::error::NszError;
use crate::ops::OperationReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

pub fn run(request: &ExtractRequest) -> Result<OperationReport, NszError> {
    if request.files.is_empty() {
        return Ok(OperationReport::default());
    }

    if let Some(out_dir) = &request.output_dir {
        fs::create_dir_all(out_dir)?;
    }

    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut args = vec!["-x".to_string()];
    if let Some(out_dir) = &request.output_dir {
        args.push("-o".to_string());
        args.push(out_dir.display().to_string());
    }
    if let Some(extract_regex) = &request.extract_regex {
        args.push("--extractregex".to_string());
        args.push(extract_regex.clone());
    }

    for file in &request.files {
        args.push(file.display().to_string());
    }

    run_nsz_cli(&repo_root, &args)?;

    let processed_files = request
        .files
        .iter()
        .map(|file| {
            let stem = file.file_stem().unwrap_or_default();
            if let Some(out_dir) = &request.output_dir {
                out_dir.join(stem)
            } else {
                file.parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .join(stem)
            }
        })
        .collect::<Vec<PathBuf>>();

    Ok(OperationReport {
        processed_files,
        skipped_files: Vec::new(),
    })
}
