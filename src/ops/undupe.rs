use std::fs;

use crate::config::UndupeRequest;
use crate::error::NszError;
use crate::ops::OperationReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

/// Runs deduplication flow through Python `nsz`.
pub fn run(request: &UndupeRequest) -> Result<OperationReport, NszError> {
    if request.files.is_empty() {
        return Ok(OperationReport::default());
    }

    if let Some(out_dir) = &request.output_dir {
        fs::create_dir_all(out_dir)?;
    }

    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut args = Vec::new();
    if request.dry_run {
        args.push("--undupe-dryrun".to_string());
    } else {
        args.push("--undupe".to_string());
    }
    if request.rename {
        args.push("--undupe-rename".to_string());
    }
    if request.hardlink {
        args.push("--undupe-hardlink".to_string());
    }
    if request.old_versions {
        args.push("--undupe-old-versions".to_string());
    }
    if let Some(priority_list) = &request.priority_list {
        args.push("--undupe-prioritylist".to_string());
        args.push(priority_list.clone());
    }
    if let Some(whitelist) = &request.whitelist {
        args.push("--undupe-whitelist".to_string());
        args.push(whitelist.clone());
    }
    if let Some(blacklist) = &request.blacklist {
        args.push("--undupe-blacklist".to_string());
        args.push(blacklist.clone());
    }
    if let Some(out_dir) = &request.output_dir {
        args.push("-o".to_string());
        args.push(out_dir.display().to_string());
    }

    for file in &request.files {
        args.push(file.display().to_string());
    }

    run_nsz_cli(&repo_root, &args)?;

    Ok(OperationReport {
        processed_files: request.files.clone(),
        skipped_files: Vec::new(),
    })
}
