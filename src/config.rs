use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CompressRequest {
    pub files: Vec<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub level: i32,
    pub long_distance_mode: bool,
    pub block: bool,
    pub solid: bool,
    pub block_size_exponent: u8,
    pub verify: bool,
    pub quick_verify: bool,
    pub keep: bool,
    pub fix_padding: bool,
    pub parse_cnmt: bool,
    pub always_parse_cnmt: bool,
    pub multi: i32,
    pub threads: i32,
    pub overwrite: bool,
    pub rm_old_version: bool,
    pub rm_source: bool,
    pub python_repo_root: Option<PathBuf>,
}

impl Default for CompressRequest {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            output_dir: None,
            level: 18,
            long_distance_mode: false,
            block: false,
            solid: false,
            block_size_exponent: 20,
            verify: false,
            quick_verify: false,
            keep: false,
            fix_padding: false,
            parse_cnmt: false,
            always_parse_cnmt: false,
            multi: 4,
            threads: -1,
            overwrite: false,
            rm_old_version: false,
            rm_source: false,
            python_repo_root: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DecompressRequest {
    pub files: Vec<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub fix_padding: bool,
    pub python_repo_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct VerifyRequest {
    pub files: Vec<PathBuf>,
    pub fix_padding: bool,
    pub python_repo_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractRequest {
    pub files: Vec<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub extract_regex: Option<String>,
    pub python_repo_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct CreateRequest {
    pub output_file: Option<PathBuf>,
    pub sources: Vec<PathBuf>,
    pub fix_padding: bool,
    pub python_repo_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct TitleKeysRequest {
    pub files: Vec<PathBuf>,
    pub python_repo_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct UndupeRequest {
    pub files: Vec<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub dry_run: bool,
    pub rename: bool,
    pub hardlink: bool,
    pub priority_list: Option<String>,
    pub whitelist: Option<String>,
    pub blacklist: Option<String>,
    pub old_versions: bool,
    pub python_repo_root: Option<PathBuf>,
}
