use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CompressRequest {
    pub level: i32,
    pub block_size_exponent: u8,
    pub multi: i32,
    pub threads: i32,
}

impl Default for CompressRequest {
    fn default() -> Self {
        Self {
            level: 18,
            block_size_exponent: 20,
            multi: 4,
            threads: -1,
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
pub struct ExtractRequest;

#[derive(Debug, Clone, Default)]
pub struct CreateRequest;

#[derive(Debug, Clone, Default)]
pub struct TitleKeysRequest;

#[derive(Debug, Clone, Default)]
pub struct UndupeRequest;
