use std::fs;
use std::path::Path;

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
