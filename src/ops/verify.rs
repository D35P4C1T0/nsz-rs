use sha2::{Digest, Sha256};
use std::path::Path;

use crate::config::VerifyRequest;
use crate::container::nsp::NspArchive;
use crate::error::NszError;
use crate::ops::VerifyReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

pub fn run(request: &VerifyRequest) -> Result<VerifyReport, NszError> {
    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut verified_files = Vec::new();

    for file in &request.files {
        match normalized_extension(file) {
            Some("ncz") => {
                let input = std::fs::read(file)?;
                let decompressed = crate::ncz::decompress::decompress_ncz_to_vec(&input)?;
                verify_hash_against_stem(file, &decompressed)?;
                verified_files.push(file.clone());
                continue;
            }
            Some("nsp") => {
                let input = std::fs::read(file)?;
                verify_nsp_like_container(&input, false)?;
                verified_files.push(file.clone());
                continue;
            }
            Some("nsz") => {
                let input = std::fs::read(file)?;
                verify_nsp_like_container(&input, true)?;
                verified_files.push(file.clone());
                continue;
            }
            _ => {}
        }

        let mut args = vec!["-V".to_string()];
        if request.fix_padding {
            args.push("-F".to_string());
        }
        args.push(file.display().to_string());

        run_nsz_cli(&repo_root, &args)?;
        verified_files.push(file.clone());
    }

    Ok(VerifyReport { verified_files })
}

fn verify_nsp_like_container(data: &[u8], compressed: bool) -> Result<(), NszError> {
    let archive = NspArchive::from_bytes(data)?;
    for entry in archive.entries() {
        let name_path = Path::new(&entry.name);
        let ext = name_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();
        if ext.eq_ignore_ascii_case("nca") {
            let is_cnmt = name_path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_ascii_lowercase().ends_with(".cnmt.nca"))
                .unwrap_or(false);
            if is_cnmt {
                continue;
            }
            let bytes = archive.entry_bytes(data, entry);
            verify_hash_against_entry_name(&entry.name, bytes)?;
            continue;
        }

        if compressed && ext.eq_ignore_ascii_case("ncz") {
            let bytes = archive.entry_bytes(data, entry);
            let decompressed = crate::ncz::decompress::decompress_ncz_to_vec(bytes)?;
            verify_hash_against_entry_name(&entry.name, &decompressed)?;
        }
    }
    Ok(())
}

fn verify_hash_against_stem(path: &Path, bytes: &[u8]) -> Result<(), NszError> {
    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        return verify_hash_against_expected(stem, bytes);
    }
    Ok(())
}

fn verify_hash_against_entry_name(name: &str, bytes: &[u8]) -> Result<(), NszError> {
    let stem = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    verify_hash_against_expected(stem, bytes)
}

fn verify_hash_against_expected(expected_stem: &str, bytes: &[u8]) -> Result<(), NszError> {
    if expected_stem.len() < 32 {
        return Ok(());
    }

    let hash = format!("{:x}", Sha256::digest(bytes));
    let expected_prefix = &expected_stem[..32];
    if !expected_prefix.eq_ignore_ascii_case(&hash[..32]) {
        return Err(NszError::ParityMismatch {
            operation: "verify".to_string(),
            expected_sha256: expected_prefix.to_lowercase(),
            actual_sha256: hash,
            first_diff_offset: 0,
        });
    }
    Ok(())
}

fn normalized_extension(path: &Path) -> Option<&str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("ncz") => Some("ncz"),
        Some(ext) if ext.eq_ignore_ascii_case("nsp") => Some("nsp"),
        Some(ext) if ext.eq_ignore_ascii_case("nsz") => Some("nsz"),
        Some(ext) if ext.eq_ignore_ascii_case("xcz") => Some("xcz"),
        Some(ext) if ext.eq_ignore_ascii_case("xci") => Some("xci"),
        _ => None,
    }
}
