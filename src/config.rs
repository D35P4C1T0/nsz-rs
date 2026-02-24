use std::path::PathBuf;

/// High-level compression request options.
#[derive(Debug, Clone)]
pub struct CompressRequest {
    /// Input files to compress.
    pub files: Vec<PathBuf>,
    /// Destination directory for generated compressed files.
    pub output_dir: Option<PathBuf>,
    /// Zstandard compression level.
    pub level: i32,
    /// Enables zstd long-distance matching.
    pub long_distance_mode: bool,
    /// Uses NCZ block-mode output when supported.
    pub block: bool,
    /// Uses NCZ solid-mode output when supported.
    pub solid: bool,
    /// Base-2 exponent for NCZ block size when block mode is enabled.
    pub block_size_exponent: u8,
    /// Requests post-compress verification.
    pub verify: bool,
    /// Requests quick post-compress verification.
    pub quick_verify: bool,
    /// Keeps additional non-secure partitions for XCI/XCZ flows.
    pub keep: bool,
    /// Applies padding fixes in compatibility paths.
    pub fix_padding: bool,
    /// Enables CNMT parsing behavior parity flags.
    pub parse_cnmt: bool,
    /// Forces CNMT parsing in all cases.
    pub always_parse_cnmt: bool,
    /// Multiprocessing worker count for compatibility CLI paths.
    pub multi: i32,
    /// Thread count hint for compression internals.
    pub threads: i32,
    /// Overwrites existing outputs when true.
    pub overwrite: bool,
    /// Removes older output versions when enabled by compatibility flow.
    pub rm_old_version: bool,
    /// Removes source files after successful compression.
    pub rm_source: bool,
    /// Optional Python baseline repository root for compatibility fallback.
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

/// High-level decompression request options.
#[derive(Debug, Clone, Default)]
pub struct DecompressRequest {
    /// Input files to decompress.
    pub files: Vec<PathBuf>,
    /// Destination directory for decompressed outputs.
    pub output_dir: Option<PathBuf>,
    /// Applies padding fixes in compatibility paths.
    pub fix_padding: bool,
    /// Optional Python baseline repository root for compatibility fallback.
    pub python_repo_root: Option<PathBuf>,
}

/// High-level verification request options.
#[derive(Debug, Clone, Default)]
pub struct VerifyRequest {
    /// Input files to verify.
    pub files: Vec<PathBuf>,
    /// Applies padding fixes in compatibility paths.
    pub fix_padding: bool,
    /// Optional Python baseline repository root for compatibility fallback.
    pub python_repo_root: Option<PathBuf>,
}

/// High-level extraction request options.
#[derive(Debug, Clone, Default)]
pub struct ExtractRequest {
    /// Input container files to extract from.
    pub files: Vec<PathBuf>,
    /// Destination directory for extracted files.
    pub output_dir: Option<PathBuf>,
    /// Optional regex used by compatibility extraction flow.
    pub extract_regex: Option<String>,
    /// Optional Python baseline repository root for compatibility fallback.
    pub python_repo_root: Option<PathBuf>,
}

/// High-level create-container request options.
#[derive(Debug, Clone, Default)]
pub struct CreateRequest {
    /// Output container file path.
    pub output_file: Option<PathBuf>,
    /// Input files/directories used to create the output container.
    pub sources: Vec<PathBuf>,
    /// Applies padding fixes in compatibility paths.
    pub fix_padding: bool,
    /// Optional Python baseline repository root for compatibility fallback.
    pub python_repo_root: Option<PathBuf>,
}

/// High-level titlekey export request options.
#[derive(Debug, Clone, Default)]
pub struct TitleKeysRequest {
    /// Input files to inspect for titlekey extraction.
    pub files: Vec<PathBuf>,
    /// Optional Python baseline repository root for compatibility fallback.
    pub python_repo_root: Option<PathBuf>,
}

/// High-level deduplication request options.
#[derive(Debug, Clone, Default)]
pub struct UndupeRequest {
    /// Input files/directories to scan for deduplication.
    pub files: Vec<PathBuf>,
    /// Optional output directory for deduplication actions.
    pub output_dir: Option<PathBuf>,
    /// Runs dedupe in dry-run mode.
    pub dry_run: bool,
    /// Renames duplicates when possible.
    pub rename: bool,
    /// Uses hardlinks for deduplicated files when possible.
    pub hardlink: bool,
    /// Optional priority regex list for selecting preferred files.
    pub priority_list: Option<String>,
    /// Optional whitelist regex list.
    pub whitelist: Option<String>,
    /// Optional blacklist regex list.
    pub blacklist: Option<String>,
    /// Includes old-version handling behavior in compatibility mode.
    pub old_versions: bool,
    /// Optional Python baseline repository root for compatibility fallback.
    pub python_repo_root: Option<PathBuf>,
}
