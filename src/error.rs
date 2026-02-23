use thiserror::Error;

#[derive(Debug, Error)]
pub enum NszError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not parse python baseline version from {path}")]
    BaselineVersionParse { path: String },
    #[error("missing required key: {key}")]
    MissingRequiredKey { key: String },
    #[error("container format error: {message}")]
    ContainerFormat { message: String },
    #[error("unsupported feature: {feature}")]
    UnsupportedFeature { feature: String },
    #[error("external command failed: {command} (status: {status}) {stderr}")]
    ExternalCommand {
        command: String,
        status: i32,
        stderr: String,
    },
    #[error(
        "parity mismatch in {operation}: expected sha256 {expected_sha256}, actual sha256 {actual_sha256}, first diff offset {first_diff_offset}"
    )]
    ParityMismatch {
        operation: String,
        expected_sha256: String,
        actual_sha256: String,
        first_diff_offset: u64,
    },
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}
