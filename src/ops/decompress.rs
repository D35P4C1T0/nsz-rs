use std::fs;
use std::path::{Path, PathBuf};

use crate::config::DecompressRequest;
use crate::container::nsp::{encode_pfs0, NspArchive};
use crate::error::NszError;
use crate::ops::OperationReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

pub fn run(request: &DecompressRequest) -> Result<OperationReport, NszError> {
    let out_dir = request
        .output_dir
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join("nsz-rs-out"));

    fs::create_dir_all(&out_dir)?;

    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut processed_files = Vec::new();

    for file in &request.files {
        match normalized_extension(file) {
            Some("ncz") => {
                let input = fs::read(file)?;
                let output = crate::ncz::decompress::decompress_ncz_to_vec(&input)?;
                let out_file = expected_decompressed_output(file, &out_dir).ok_or_else(|| {
                    NszError::ContainerFormat {
                        message: format!("could not resolve output path for {}", file.display()),
                    }
                })?;
                fs::write(&out_file, output)?;
                processed_files.push(out_file);
                continue;
            }
            Some("nsz") => {
                let input = fs::read(file)?;
                let output = decompress_nsz_to_nsp(&input)?;
                let out_file = expected_decompressed_output(file, &out_dir).ok_or_else(|| {
                    NszError::ContainerFormat {
                        message: format!("could not resolve output path for {}", file.display()),
                    }
                })?;
                fs::write(&out_file, output)?;
                processed_files.push(out_file);
                continue;
            }
            _ => {}
        }

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

fn decompress_nsz_to_nsp(data: &[u8]) -> Result<Vec<u8>, NszError> {
    let archive = NspArchive::from_bytes(data)?;
    let mut output_entries = Vec::with_capacity(archive.entries().len());

    for entry in archive.entries() {
        let entry_bytes = archive.entry_bytes(data, entry);
        if entry.name.to_ascii_lowercase().ends_with(".ncz") {
            let mut new_name = PathBuf::from(&entry.name);
            new_name.set_extension("nca");
            let name = new_name
                .to_str()
                .ok_or_else(|| NszError::ContainerFormat {
                    message: format!("invalid UTF-8 output name for {}", entry.name),
                })?
                .to_string();
            let output = crate::ncz::decompress::decompress_ncz_to_vec(entry_bytes)?;
            output_entries.push((name, output));
        } else {
            output_entries.push((entry.name.clone(), entry_bytes.to_vec()));
        }
    }

    encode_pfs0(
        &output_entries,
        archive.first_file_offset(),
        archive.string_table_size(),
    )
}

fn normalized_extension(path: &Path) -> Option<&str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("ncz") => Some("ncz"),
        Some(ext) if ext.eq_ignore_ascii_case("nsz") => Some("nsz"),
        Some(ext) if ext.eq_ignore_ascii_case("xcz") => Some("xcz"),
        Some(ext) if ext.eq_ignore_ascii_case("nsp") => Some("nsp"),
        Some(ext) if ext.eq_ignore_ascii_case("xci") => Some("xci"),
        _ => None,
    }
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
