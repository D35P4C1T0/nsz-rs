//! Public facade for `nsz-rs` high-level operations.
#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::struct_excessive_bools,
    clippy::fn_params_excessive_bools,
    clippy::similar_names,
    clippy::implicit_hasher,
    clippy::needless_pass_by_value,
    clippy::useless_conversion
)]

pub mod config;
pub mod container;
pub mod crypto;
pub mod error;
pub mod fs_ops;
pub mod ncz;
pub mod ops;
pub mod parity;
#[cfg(feature = "python")]
mod python;

pub use config::{
    CompressRequest, CreateRequest, DecompressRequest, ExtractRequest, TitleKeysRequest,
    UndupeRequest, VerifyRequest,
};
pub use error::NszError;
pub use ops::{OperationReport, VerifyReport};

/// Compresses input files according to [`CompressRequest`].
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use nsz_rs::{compress, CompressRequest, NszError};
///
/// fn run() -> Result<(), NszError> {
///     let request = CompressRequest {
///         files: vec![PathBuf::from("/games/game.nsp")],
///         output_dir: Some(PathBuf::from("/tmp/out")),
///         ..Default::default()
///     };
///     let _report = compress(&request)?;
///     Ok(())
/// }
/// ```
pub fn compress(request: &CompressRequest) -> Result<OperationReport, NszError> {
    ops::compress::run(request)
}

/// Decompresses input files according to [`DecompressRequest`].
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use nsz_rs::{decompress, DecompressRequest, NszError};
///
/// fn run() -> Result<(), NszError> {
///     let request = DecompressRequest {
///         files: vec![PathBuf::from("/games/game.nsz")],
///         output_dir: Some(PathBuf::from("/tmp/out")),
///         ..Default::default()
///     };
///     let _report = decompress(&request)?;
///     Ok(())
/// }
/// ```
pub fn decompress(request: &DecompressRequest) -> Result<OperationReport, NszError> {
    ops::decompress::run(request)
}

/// Verifies content integrity according to [`VerifyRequest`].
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use nsz_rs::{verify, VerifyRequest, NszError};
///
/// fn run() -> Result<(), NszError> {
///     let request = VerifyRequest {
///         files: vec![PathBuf::from("/games/game.nsp")],
///         ..Default::default()
///     };
///     let _report = verify(&request)?;
///     Ok(())
/// }
/// ```
pub fn verify(request: &VerifyRequest) -> Result<VerifyReport, NszError> {
    ops::verify::run(request)
}

/// Extracts files from supported containers according to [`ExtractRequest`].
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use nsz_rs::{extract, ExtractRequest, NszError};
///
/// fn run() -> Result<(), NszError> {
///     let request = ExtractRequest {
///         files: vec![PathBuf::from("/games/game.nsp")],
///         output_dir: Some(PathBuf::from("/tmp/extract")),
///         ..Default::default()
///     };
///     let _report = extract(&request)?;
///     Ok(())
/// }
/// ```
pub fn extract(request: &ExtractRequest) -> Result<OperationReport, NszError> {
    ops::extract::run(request)
}

/// Creates a new container according to [`CreateRequest`].
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use nsz_rs::{create, CreateRequest, NszError};
///
/// fn run() -> Result<(), NszError> {
///     let request = CreateRequest {
///         output_file: Some(PathBuf::from("/tmp/out.nsp")),
///         sources: vec![PathBuf::from("/tmp/extract")],
///         ..Default::default()
///     };
///     let _report = create(&request)?;
///     Ok(())
/// }
/// ```
pub fn create(request: &CreateRequest) -> Result<OperationReport, NszError> {
    ops::create::run(request)
}

/// Writes title key information according to [`TitleKeysRequest`].
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use nsz_rs::{titlekeys, TitleKeysRequest, NszError};
///
/// fn run() -> Result<(), NszError> {
///     let request = TitleKeysRequest {
///         files: vec![PathBuf::from("/games/game.nsp")],
///         ..Default::default()
///     };
///     let _report = titlekeys(&request)?;
///     Ok(())
/// }
/// ```
pub fn titlekeys(request: &TitleKeysRequest) -> Result<OperationReport, NszError> {
    ops::titlekeys::run(request)
}

/// Deduplicates files according to [`UndupeRequest`].
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use nsz_rs::{undupe, UndupeRequest, NszError};
///
/// fn run() -> Result<(), NszError> {
///     let request = UndupeRequest {
///         files: vec![PathBuf::from("/games")],
///         dry_run: true,
///         ..Default::default()
///     };
///     let _report = undupe(&request)?;
///     Ok(())
/// }
/// ```
pub fn undupe(request: &UndupeRequest) -> Result<OperationReport, NszError> {
    ops::undupe::run(request)
}
