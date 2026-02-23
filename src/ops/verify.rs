use sha2::{Digest, Sha256};

use crate::config::VerifyRequest;
use crate::error::NszError;
use crate::ops::VerifyReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

pub fn run(request: &VerifyRequest) -> Result<VerifyReport, NszError> {
    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut verified_files = Vec::new();

    for file in &request.files {
        if file
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("ncz"))
            .unwrap_or(false)
        {
            let input = std::fs::read(file)?;
            let decompressed = crate::ncz::decompress::decompress_ncz_to_vec(&input)?;
            let hash = format!("{:x}", Sha256::digest(&decompressed));
            if let Some(stem) = file.file_stem().and_then(|s| s.to_str()) {
                if stem.len() >= 32 {
                    let expected_prefix = &hash[..32];
                    if !stem[..32].eq_ignore_ascii_case(expected_prefix) {
                        return Err(NszError::ParityMismatch {
                            operation: "verify".to_string(),
                            expected_sha256: stem[..32].to_lowercase(),
                            actual_sha256: hash,
                            first_diff_offset: 0,
                        });
                    }
                }
            }
            verified_files.push(file.clone());
            continue;
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
