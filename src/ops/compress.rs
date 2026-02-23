use std::fs;
use std::path::{Path, PathBuf};

use crate::config::CompressRequest;
use crate::error::NszError;
use crate::ops::OperationReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

pub fn run(request: &CompressRequest) -> Result<OperationReport, NszError> {
    if request.files.is_empty() {
        return Ok(OperationReport::default());
    }

    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut args = vec![
        "-C".to_string(),
        "-l".to_string(),
        request.level.to_string(),
        "-s".to_string(),
        request.block_size_exponent.to_string(),
        "-m".to_string(),
        request.multi.to_string(),
        "-t".to_string(),
        request.threads.to_string(),
    ];

    if request.long_distance_mode {
        args.push("-L".to_string());
    }
    if request.block {
        args.push("-B".to_string());
    }
    if request.solid {
        args.push("-S".to_string());
    }
    if request.verify {
        args.push("-V".to_string());
    }
    if request.quick_verify {
        args.push("-Q".to_string());
    }
    if request.keep {
        args.push("-K".to_string());
    }
    if request.fix_padding {
        args.push("-F".to_string());
    }
    if request.parse_cnmt {
        args.push("-p".to_string());
    }
    if request.always_parse_cnmt {
        args.push("-P".to_string());
    }
    if request.overwrite {
        args.push("-w".to_string());
    }
    if request.rm_old_version {
        args.push("-r".to_string());
    }
    if request.rm_source {
        args.push("--rm-source".to_string());
    }

    let output_dir = request.output_dir.clone();
    if let Some(out_dir) = &output_dir {
        fs::create_dir_all(out_dir)?;
        args.push("-o".to_string());
        args.push(out_dir.display().to_string());
    }

    for file in &request.files {
        args.push(file.display().to_string());
    }

    run_nsz_cli(&repo_root, &args)?;

    let mut processed_files = Vec::new();
    for file in &request.files {
        if let Some(path) = expected_compressed_output(file, output_dir.as_deref()) {
            if path.exists() {
                processed_files.push(path);
            }
        }
    }

    Ok(OperationReport {
        processed_files,
        skipped_files: Vec::new(),
    })
}

fn expected_compressed_output(input: &Path, output_dir: Option<&Path>) -> Option<PathBuf> {
    let mut output_name = PathBuf::from(input.file_name()?);
    let extension = input.extension()?.to_string_lossy().to_ascii_lowercase();

    output_name.set_extension(match extension.as_str() {
        "nsp" => "nsz",
        "xci" => "xcz",
        _ => return None,
    });

    if let Some(out_dir) = output_dir {
        return Some(out_dir.join(output_name));
    }

    Some(input.parent()?.join(output_name))
}
