use std::path::PathBuf;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use crate::{
    CompressRequest, CreateRequest, DecompressRequest, ExtractRequest, NszError, TitleKeysRequest,
    UndupeRequest, VerifyRequest,
};

fn map_error(err: NszError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn map_paths(paths: Vec<PathBuf>) -> Vec<String> {
    paths
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

fn map_input_files(files: Vec<String>) -> Vec<PathBuf> {
    files.into_iter().map(PathBuf::from).collect()
}

#[pyfunction]
#[pyo3(
    signature = (
        files,
        output_dir = None,
        level = 18,
        long_distance_mode = false,
        block = false,
        solid = false,
        block_size_exponent = 20,
        verify = false,
        quick_verify = false,
        keep = false,
        fix_padding = false,
        parse_cnmt = false,
        always_parse_cnmt = false,
        multi = 4,
        threads = -1,
        overwrite = false,
        rm_old_version = false,
        rm_source = false
    )
)]
fn compress(
    files: Vec<String>,
    output_dir: Option<String>,
    level: i32,
    long_distance_mode: bool,
    block: bool,
    solid: bool,
    block_size_exponent: u8,
    verify: bool,
    quick_verify: bool,
    keep: bool,
    fix_padding: bool,
    parse_cnmt: bool,
    always_parse_cnmt: bool,
    multi: i32,
    threads: i32,
    overwrite: bool,
    rm_old_version: bool,
    rm_source: bool,
) -> PyResult<Vec<String>> {
    let request = CompressRequest {
        files: map_input_files(files),
        output_dir: output_dir.map(PathBuf::from),
        level,
        long_distance_mode,
        block,
        solid,
        block_size_exponent,
        verify,
        quick_verify,
        keep,
        fix_padding,
        parse_cnmt,
        always_parse_cnmt,
        multi,
        threads,
        overwrite,
        rm_old_version,
        rm_source,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    };
    let report = crate::compress(&request).map_err(map_error)?;
    Ok(map_paths(report.processed_files))
}

#[pyfunction]
#[pyo3(signature = (files, output_dir = None, fix_padding = false))]
fn decompress(
    files: Vec<String>,
    output_dir: Option<String>,
    fix_padding: bool,
) -> PyResult<Vec<String>> {
    let request = DecompressRequest {
        files: map_input_files(files),
        output_dir: output_dir.map(PathBuf::from),
        fix_padding,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    };
    let report = crate::decompress(&request).map_err(map_error)?;
    Ok(map_paths(report.processed_files))
}

#[pyfunction]
#[pyo3(signature = (files, fix_padding = false))]
fn verify(files: Vec<String>, fix_padding: bool) -> PyResult<Vec<String>> {
    let request = VerifyRequest {
        files: map_input_files(files),
        fix_padding,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    };
    let report = crate::verify(&request).map_err(map_error)?;
    Ok(map_paths(report.verified_files))
}

#[pyfunction]
#[pyo3(signature = (files, output_dir = None, extract_regex = None))]
fn extract(
    files: Vec<String>,
    output_dir: Option<String>,
    extract_regex: Option<String>,
) -> PyResult<Vec<String>> {
    let request = ExtractRequest {
        files: map_input_files(files),
        output_dir: output_dir.map(PathBuf::from),
        extract_regex,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    };
    let report = crate::extract(&request).map_err(map_error)?;
    Ok(map_paths(report.processed_files))
}

#[pyfunction]
#[pyo3(signature = (sources, output_file = None, fix_padding = false))]
fn create(
    sources: Vec<String>,
    output_file: Option<String>,
    fix_padding: bool,
) -> PyResult<Vec<String>> {
    let request = CreateRequest {
        output_file: output_file.map(PathBuf::from),
        sources: map_input_files(sources),
        fix_padding,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    };
    let report = crate::create(&request).map_err(map_error)?;
    Ok(map_paths(report.processed_files))
}

#[pyfunction]
fn titlekeys(files: Vec<String>) -> PyResult<Vec<String>> {
    let request = TitleKeysRequest {
        files: map_input_files(files),
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    };
    let report = crate::titlekeys(&request).map_err(map_error)?;
    Ok(map_paths(report.processed_files))
}

#[pyfunction]
#[pyo3(
    signature = (
        files,
        output_dir = None,
        dry_run = false,
        rename = false,
        hardlink = false,
        priority_list = None,
        whitelist = None,
        blacklist = None,
        old_versions = false
    )
)]
fn undupe(
    files: Vec<String>,
    output_dir: Option<String>,
    dry_run: bool,
    rename: bool,
    hardlink: bool,
    priority_list: Option<String>,
    whitelist: Option<String>,
    blacklist: Option<String>,
    old_versions: bool,
) -> PyResult<Vec<String>> {
    let request = UndupeRequest {
        files: map_input_files(files),
        output_dir: output_dir.map(PathBuf::from),
        dry_run,
        rename,
        hardlink,
        priority_list,
        whitelist,
        blacklist,
        old_versions,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    };
    let report = crate::undupe(&request).map_err(map_error)?;
    Ok(map_paths(report.processed_files))
}

#[pyfunction]
fn rust_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pymodule]
fn _native(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(compress, module)?)?;
    module.add_function(wrap_pyfunction!(decompress, module)?)?;
    module.add_function(wrap_pyfunction!(verify, module)?)?;
    module.add_function(wrap_pyfunction!(extract, module)?)?;
    module.add_function(wrap_pyfunction!(create, module)?)?;
    module.add_function(wrap_pyfunction!(titlekeys, module)?)?;
    module.add_function(wrap_pyfunction!(undupe, module)?)?;
    module.add_function(wrap_pyfunction!(rust_version, module)?)?;
    Ok(())
}
