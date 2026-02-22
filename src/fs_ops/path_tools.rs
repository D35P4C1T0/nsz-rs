use std::path::{Path, PathBuf};

pub fn change_extension(file_path: &Path, new_extension: &str) -> PathBuf {
    let ext = new_extension.trim_start_matches('.');
    file_path.with_extension(ext)
}
