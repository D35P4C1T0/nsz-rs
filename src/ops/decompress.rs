use std::fs;
use std::path::{Path, PathBuf};

use crate::config::DecompressRequest;
use crate::error::NszError;
use crate::ops::OperationReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

pub fn run(request: &DecompressRequest) -> Result<OperationReport, NszError> {
    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let out_dir = request
        .output_dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("nsz-rs-out"));

    fs::create_dir_all(&out_dir)?;

    let mut processed_files = Vec::new();

    for file in &request.files {
        let mut args = vec![
            "-D".to_string(),
            "-o".to_string(),
            out_dir.display().to_string(),
        ];
        if request.fix_padding {
            args.push("-F".to_string());
        }
        args.push(file.display().to_string());

        run_nsz_cli(&repo_root, &args)?;
        if let Some(out) = expected_decompressed_output(file, &out_dir) {
            processed_files.push(out);
        }
    }

    Ok(OperationReport {
        processed_files,
        skipped_files: Vec::new(),
    })
}

fn expected_decompressed_output(input: &Path, out_dir: &Path) -> Option<PathBuf> {
    let name = input.file_name()?.to_os_string();
    let mut path = PathBuf::from(name);
    let ext = input.extension()?.to_string_lossy().to_ascii_lowercase();

    path.set_extension(match ext.as_str() {
        "nsz" => "nsp",
        "xcz" => "xci",
        "ncz" => "nca",
        _ => return None,
    });

    Some(out_dir.join(path))
}
