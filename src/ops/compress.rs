use std::fs;
use std::path::{Path, PathBuf};

use crate::config::CompressRequest;
use crate::container::hfs0::{encode_hfs0, Hfs0Archive};
use crate::container::nsp::{encode_pfs0, NspArchive};
use crate::container::xci::{encode_xci_like, XciArchive};
use crate::error::NszError;
use crate::ops::OperationReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

pub fn run(request: &CompressRequest) -> Result<OperationReport, NszError> {
    if request.files.is_empty() {
        return Ok(OperationReport::default());
    }

    if let Some(out_dir) = &request.output_dir {
        fs::create_dir_all(out_dir)?;
    }

    let mut processed_files = Vec::new();
    let mut fallback_files = Vec::new();

    for file in &request.files {
        match normalized_extension(file) {
            Some("nsp") => {
                let input = fs::read(file)?;
                let output = compress_nsp_to_nsz(&input, request.level)?;
                let out_file = expected_compressed_output(file, request.output_dir.as_deref())
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: format!("could not resolve output path for {}", file.display()),
                    })?;
                fs::write(&out_file, output)?;
                processed_files.push(out_file);
                continue;
            }
            Some("xci") => {
                let input = fs::read(file)?;
                let output = compress_xci_to_xcz(&input, request.level)?;
                let out_file = expected_compressed_output(file, request.output_dir.as_deref())
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: format!("could not resolve output path for {}", file.display()),
                    })?;
                fs::write(&out_file, output)?;
                processed_files.push(out_file);
                continue;
            }
            Some("nca") => {
                let input = fs::read(file)?;
                let output = crate::ncz::compress::compress_nca_to_ncz_vec(&input, request.level)?;
                let out_file = expected_compressed_output(file, request.output_dir.as_deref())
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: format!("could not resolve output path for {}", file.display()),
                    })?;
                fs::write(&out_file, output)?;
                processed_files.push(out_file);
                continue;
            }
            _ => fallback_files.push(file.clone()),
        }
    }

    if fallback_files.is_empty() {
        return Ok(OperationReport {
            processed_files,
            skipped_files: Vec::new(),
        });
    }

    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut args = build_python_compress_args(request);
    for file in &fallback_files {
        args.push(file.display().to_string());
    }
    run_nsz_cli(&repo_root, &args)?;

    for file in &fallback_files {
        if let Some(path) = expected_compressed_output(file, request.output_dir.as_deref()) {
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

fn compress_nsp_to_nsz(data: &[u8], level: i32) -> Result<Vec<u8>, NszError> {
    let archive = NspArchive::from_bytes(data)?;
    let mut output_entries = Vec::with_capacity(archive.entries().len());

    for entry in archive.entries() {
        let entry_bytes = archive.entry_bytes(data, entry);
        if entry.name.to_ascii_lowercase().ends_with(".nca") {
            let mut new_name = PathBuf::from(&entry.name);
            new_name.set_extension("ncz");
            let name = new_name
                .to_str()
                .ok_or_else(|| NszError::ContainerFormat {
                    message: format!("invalid UTF-8 output name for {}", entry.name),
                })?
                .to_string();
            let output = crate::ncz::compress::compress_nca_to_ncz_vec(entry_bytes, level)?;
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

fn compress_xci_to_xcz(data: &[u8], level: i32) -> Result<Vec<u8>, NszError> {
    let xci = XciArchive::from_bytes(data)?;
    let root_bytes = xci.root_hfs0_bytes(data)?;
    let root = xci.root_hfs0_archive(data)?;

    let mut root_output_entries = Vec::with_capacity(root.entries().len());
    for partition in root.entries() {
        let partition_bytes = root.entry_bytes(root_bytes, partition);
        let partition_archive = Hfs0Archive::from_bytes(partition_bytes)?;

        let mut partition_output_entries = Vec::with_capacity(partition_archive.entries().len());
        for entry in partition_archive.entries() {
            let entry_bytes = partition_archive.entry_bytes(partition_bytes, entry);
            if entry.name.to_ascii_lowercase().ends_with(".nca") {
                let mut new_name = PathBuf::from(&entry.name);
                new_name.set_extension("ncz");
                let name = new_name
                    .to_str()
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: format!("invalid UTF-8 output name for {}", entry.name),
                    })?
                    .to_string();
                let output = crate::ncz::compress::compress_nca_to_ncz_vec(entry_bytes, level)?;
                partition_output_entries.push((name, output));
            } else {
                partition_output_entries.push((entry.name.clone(), entry_bytes.to_vec()));
            }
        }

        let partition_output = encode_hfs0(
            &partition_output_entries,
            partition_archive.first_file_offset(),
            partition_archive.string_table_size(),
        )?;
        root_output_entries.push((partition.name.clone(), partition_output));
    }

    let root_output = encode_hfs0(
        &root_output_entries,
        root.first_file_offset(),
        root.string_table_size(),
    )?;
    encode_xci_like(data, &xci, &root_output)
}

fn build_python_compress_args(request: &CompressRequest) -> Vec<String> {
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
    if let Some(out_dir) = &request.output_dir {
        args.push("-o".to_string());
        args.push(out_dir.display().to_string());
    }

    args
}

fn normalized_extension(path: &Path) -> Option<&str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("nsp") => Some("nsp"),
        Some(ext) if ext.eq_ignore_ascii_case("xci") => Some("xci"),
        Some(ext) if ext.eq_ignore_ascii_case("nca") => Some("nca"),
        _ => None,
    }
}

fn expected_compressed_output(input: &Path, output_dir: Option<&Path>) -> Option<PathBuf> {
    let mut output_name = PathBuf::from(input.file_name()?);
    let extension = input.extension()?.to_string_lossy().to_ascii_lowercase();

    output_name.set_extension(match extension.as_str() {
        "nsp" => "nsz",
        "xci" => "xcz",
        "nca" => "ncz",
        _ => return None,
    });

    if let Some(out_dir) = output_dir {
        return Some(out_dir.join(output_name));
    }

    Some(input.parent()?.join(output_name))
}
