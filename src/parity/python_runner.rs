use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::NszError;

pub fn query_version(repo_root: &str) -> Result<String, NszError> {
    let setup_path = Path::new(repo_root).join("setup.py");
    let setup_py = fs::read_to_string(&setup_path)?;

    for line in setup_py.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("version") {
            continue;
        }

        if let Some((_, rhs)) = trimmed.split_once('=') {
            let candidate = rhs.trim().trim_end_matches(',').trim();
            if candidate.len() >= 2
                && ((candidate.starts_with('\'') && candidate.ends_with('\''))
                    || (candidate.starts_with('"') && candidate.ends_with('"')))
            {
                return Ok(candidate[1..candidate.len() - 1].to_string());
            }
        }
    }

    Err(NszError::BaselineVersionParse {
        path: setup_path.display().to_string(),
    })
}

pub fn resolve_python_repo_root(explicit: Option<&Path>) -> PathBuf {
    if let Some(path) = explicit {
        return path.to_path_buf();
    }

    if let Ok(env_root) = std::env::var("NSZ_PYTHON_REPO") {
        return PathBuf::from(env_root);
    }

    PathBuf::from("/home/matteo/Documents/prog/python/nsz")
}

pub fn run_nsz_cli(repo_root: &Path, args: &[String]) -> Result<(), NszError> {
    let python_bin = std::env::var("NSZ_PYTHON_BIN").unwrap_or_else(|_| "python3".to_string());
    let output = Command::new(&python_bin)
        .current_dir(repo_root)
        .arg("nsz.py")
        .args(args)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let status = output.status.code().unwrap_or(-1);
    let command = format!("{python_bin} nsz.py {}", args.join(" "));
    Err(NszError::ExternalCommand {
        command,
        status,
        stderr,
    })
}
