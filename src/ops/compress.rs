use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::config::CompressRequest;
use crate::container::hfs0::{encode_hfs0, Hfs0Archive};
use crate::container::nca::{NcaKeySet, TicketRecord};
use crate::container::nsp::{encode_pfs0, NspArchive};
use crate::container::xci::{encode_xci_like, XciArchive};
use crate::error::NszError;
use crate::ops::OperationReport;
use crate::parity::python_runner::{resolve_python_repo_root, run_nsz_cli};

const UNCOMPRESSABLE_HEADER_SIZE: usize = 0x4000;
const XCI_HFS0_FIRST_FILE_OFFSET: u64 = 0x8000;

/// Compresses supported inputs natively and falls back to Python `nsz` for unsupported formats.
pub fn run(request: &CompressRequest) -> Result<OperationReport, NszError> {
    if request.files.is_empty() {
        return Ok(OperationReport::default());
    }

    if let Some(out_dir) = &request.output_dir {
        fs::create_dir_all(out_dir)?;
    }

    let mut processed_files = Vec::new();
    let mut fallback_files = Vec::new();
    let keyset = resolve_keyset();
    let solid_threads = effective_solid_threads(request.threads);

    for file in &request.files {
        match normalized_extension(file) {
            Some("nsp") => {
                let input = fs::read(file)?;
                let output = compress_nsp_to_nsz(&input, request, keyset.as_ref(), solid_threads)?;
                let out_file = expected_compressed_output(file, request.output_dir.as_deref())
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: format!("could not resolve output path for {}", file.display()),
                    })?;
                fs::write(&out_file, output)?;
                processed_files.push(out_file);
            }
            Some("xci") => {
                let input = fs::read(file)?;
                let output = compress_xci_to_xcz(&input, request, keyset.as_ref(), solid_threads)?;
                let out_file = expected_compressed_output(file, request.output_dir.as_deref())
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: format!("could not resolve output path for {}", file.display()),
                    })?;
                fs::write(&out_file, output)?;
                processed_files.push(out_file);
            }
            Some("nca") => {
                let input = fs::read(file)?;
                let empty_tickets = HashMap::new();
                let plan = keyset.as_ref().and_then(|keys| {
                    crate::container::nca::build_compression_plan(&input, keys, &empty_tickets).ok()
                });
                let output = crate::ncz::compress::compress_nca_to_ncz_vec_with_plan(
                    &input,
                    request.level,
                    request.long_distance_mode,
                    solid_threads,
                    plan.as_ref(),
                )?;
                let out_file = expected_compressed_output(file, request.output_dir.as_deref())
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: format!("could not resolve output path for {}", file.display()),
                    })?;
                fs::write(&out_file, output)?;
                processed_files.push(out_file);
            }
            _ => fallback_files.push(file.clone()),
        }
    }

    if fallback_files.is_empty() {
        return Ok(OperationReport {
            processed_files,
            skipped_files: Vec::new(),
        });
    }

    let repo_root = resolve_python_repo_root(request.python_repo_root.as_deref());
    let mut args = build_python_compress_args(request);
    for file in &fallback_files {
        args.push(file.display().to_string());
    }
    run_nsz_cli(&repo_root, &args)?;

    for file in &fallback_files {
        if let Some(path) = expected_compressed_output(file, request.output_dir.as_deref()) {
            if path.exists() {
                processed_files.push(path);
            }
        }
    }

    Ok(OperationReport {
        processed_files,
        skipped_files: Vec::new(),
    })
}

fn compress_nsp_to_nsz(
    data: &[u8],
    request: &CompressRequest,
    keyset: Option<&NcaKeySet>,
    solid_threads: i32,
) -> Result<Vec<u8>, NszError> {
    let profile = std::env::var("NSZ_PROFILE_COMPRESS").ok().as_deref() == Some("1");
    let total_started = Instant::now();
    let archive = NspArchive::from_bytes(data)?;
    let parse_elapsed = total_started.elapsed();
    let tickets_started = Instant::now();
    let tickets = collect_nsp_tickets(&archive, data);
    let tickets_elapsed = tickets_started.elapsed();
    let largest_convertible_nca = archive
        .entries()
        .iter()
        .filter(|entry| {
            entry.name.to_ascii_lowercase().ends_with(".nca")
                && !entry.name.to_ascii_lowercase().ends_with(".cnmt.nca")
                && entry.size as usize > UNCOMPRESSABLE_HEADER_SIZE
        })
        .map(|entry| entry.size)
        .max();
    let mut output_entries: Vec<(String, Cow<'_, [u8]>)> =
        Vec::with_capacity(archive.entries().len());
    let mut converted_entries = 0usize;
    let mut converted_bytes = 0u64;
    let mut passthrough_entries = 0usize;
    let mut passthrough_bytes = 0u64;
    let mut convert_elapsed = std::time::Duration::default();

    for entry in archive.entries() {
        let entry_bytes = archive.entry_bytes(data, entry);
        if should_convert_nca_entry(
            &entry.name,
            entry_bytes,
            entry.size,
            largest_convertible_nca,
            keyset.as_ref().map(|keys| &keys.header_key),
        ) {
            let mut new_name = PathBuf::from(&entry.name);
            new_name.set_extension("ncz");
            let name = new_name
                .to_str()
                .ok_or_else(|| NszError::ContainerFormat {
                    message: format!("invalid UTF-8 output name for {}", entry.name),
                })?
                .to_string();
            let plan =
                keyset.and_then(|keys| {
                    match crate::container::nca::build_compression_plan(entry_bytes, keys, &tickets)
                    {
                        Ok(plan) => Some(plan),
                        Err(err) => {
                            debug_plan_failure(&entry.name, &err);
                            None
                        }
                    }
                });
            let nca_started = Instant::now();
            let output = crate::ncz::compress::compress_nca_to_ncz_vec_with_plan(
                entry_bytes,
                request.level,
                request.long_distance_mode,
                solid_threads,
                plan.as_ref(),
            )?;
            convert_elapsed += nca_started.elapsed();
            converted_entries += 1;
            converted_bytes = converted_bytes.saturating_add(entry.size);
            output_entries.push((name, Cow::Owned(output)));
        } else {
            passthrough_entries += 1;
            passthrough_bytes = passthrough_bytes.saturating_add(entry.size);
            output_entries.push((entry.name.clone(), Cow::Borrowed(entry_bytes)));
        }
    }

    let encode_started = Instant::now();
    let encoded = encode_pfs0(
        &output_entries,
        archive.first_file_offset(),
        archive.string_table_size(),
    )?;
    let encode_elapsed = encode_started.elapsed();
    if profile {
        eprintln!(
            "[profile][compress_nsp] entries={} converted_entries={} converted_bytes={} passthrough_entries={} passthrough_bytes={} parse_ms={} tickets_ms={} convert_ms={} encode_ms={} total_ms={}",
            archive.entries().len(),
            converted_entries,
            converted_bytes,
            passthrough_entries,
            passthrough_bytes,
            parse_elapsed.as_millis(),
            tickets_elapsed.as_millis(),
            convert_elapsed.as_millis(),
            encode_elapsed.as_millis(),
            total_started.elapsed().as_millis()
        );
    }
    Ok(encoded)
}

fn compress_xci_to_xcz(
    data: &[u8],
    request: &CompressRequest,
    keyset: Option<&NcaKeySet>,
    solid_threads: i32,
) -> Result<Vec<u8>, NszError> {
    let use_block_ncz = request.block || !request.solid;
    let xci = XciArchive::from_bytes(data)?;
    let root_bytes = xci.root_hfs0_bytes(data)?;
    let root = xci.root_hfs0_archive(data)?;

    let mut root_output_entries = Vec::with_capacity(root.entries().len());
    let mut trailing_padding_to_trim = 0usize;
    for (index, partition) in root.entries().iter().enumerate() {
        let is_last_partition = index + 1 == root.entries().len();
        let partition_name = partition.name.to_ascii_lowercase();
        if !request.keep && partition_name != "secure" {
            let empty_entries: [(String, Cow<'_, [u8]>); 0] = [];
            let empty_partition = encode_hfs0(&empty_entries, 0x11, 1)?;
            let aligned_empty_partition = align_xci_partition_size(empty_partition);
            if is_last_partition && root.entries().len() > 1 {
                trailing_padding_to_trim = aligned_empty_partition.len().saturating_sub(0x11);
            }
            root_output_entries.push((partition.name.clone(), aligned_empty_partition));
            continue;
        }

        let partition_bytes = root.entry_bytes(root_bytes, partition);
        let partition_archive =
            Hfs0Archive::from_bytes(partition_bytes).map_err(|err| NszError::ContainerFormat {
                message: format!(
                    "failed to parse XCI partition '{}' as HFS0: {err}",
                    partition.name
                ),
            })?;
        let partition_tickets = collect_hfs0_tickets(&partition_archive, partition_bytes);
        let largest_convertible_nca = partition_archive
            .entries()
            .iter()
            .filter(|entry| {
                entry.name.to_ascii_lowercase().ends_with(".nca")
                    && !entry.name.to_ascii_lowercase().ends_with(".cnmt.nca")
                    && entry.size as usize > UNCOMPRESSABLE_HEADER_SIZE
            })
            .map(|entry| entry.size)
            .max();

        let mut partition_output_entries: Vec<(String, Cow<'_, [u8]>)> =
            Vec::with_capacity(partition_archive.entries().len());
        for entry in partition_archive.entries() {
            let entry_bytes = partition_archive.entry_bytes(partition_bytes, entry);
            if should_convert_nca_entry(
                &entry.name,
                entry_bytes,
                entry.size,
                largest_convertible_nca,
                keyset.as_ref().map(|keys| &keys.header_key),
            ) {
                let mut new_name = PathBuf::from(&entry.name);
                new_name.set_extension("ncz");
                let name = new_name
                    .to_str()
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: format!("invalid UTF-8 output name for {}", entry.name),
                    })?
                    .to_string();
                let plan = keyset.and_then(|keys| {
                    crate::container::nca::build_compression_plan(
                        entry_bytes,
                        keys,
                        &partition_tickets,
                    )
                    .inspect_err(|err| {
                        debug_plan_failure(&entry.name, err);
                    })
                    .ok()
                });
                let output = if use_block_ncz {
                    crate::ncz::compress::compress_nca_to_ncz_block_vec_with_plan(
                        entry_bytes,
                        request.level,
                        request.long_distance_mode,
                        request.block_size_exponent,
                        plan.as_ref(),
                    )?
                } else {
                    crate::ncz::compress::compress_nca_to_ncz_vec_with_plan(
                        entry_bytes,
                        request.level,
                        request.long_distance_mode,
                        solid_threads,
                        plan.as_ref(),
                    )?
                };
                partition_output_entries.push((name, Cow::Owned(output)));
            } else {
                partition_output_entries.push((entry.name.clone(), Cow::Borrowed(entry_bytes)));
            }
        }

        let partition_output =
            encode_hfs0(&partition_output_entries, XCI_HFS0_FIRST_FILE_OFFSET, 0)?;
        let aligned_partition_output = align_xci_partition_size(partition_output.clone());
        if is_last_partition && root.entries().len() > 1 {
            trailing_padding_to_trim = aligned_partition_output.len() - partition_output.len();
        }
        root_output_entries.push((partition.name.clone(), aligned_partition_output));
    }

    let root_output = encode_hfs0(&root_output_entries, XCI_HFS0_FIRST_FILE_OFFSET, 0)?;
    let mut output = encode_xci_like(data, &xci, &root_output)?;
    if trailing_padding_to_trim > 0 && output.len() > trailing_padding_to_trim {
        let trim_start = output.len() - trailing_padding_to_trim;
        if output[trim_start..].iter().all(|byte| *byte == 0) {
            output.truncate(trim_start);
        }
    }
    Ok(output)
}

fn align_xci_partition_size(mut bytes: Vec<u8>) -> Vec<u8> {
    let padding = 0x200 - (bytes.len() % 0x200);
    bytes.resize(bytes.len() + padding, 0);
    bytes
}

fn collect_nsp_tickets(archive: &NspArchive, data: &[u8]) -> HashMap<[u8; 16], TicketRecord> {
    let mut out = HashMap::new();
    for entry in archive.entries() {
        if !entry.name.to_ascii_lowercase().ends_with(".tik") {
            continue;
        }
        let bytes = archive.entry_bytes(data, entry);
        if let Ok(ticket) = crate::container::nca::parse_ticket_record(bytes) {
            out.insert(ticket.rights_id, ticket);
        }
    }
    out
}

fn collect_hfs0_tickets(archive: &Hfs0Archive, data: &[u8]) -> HashMap<[u8; 16], TicketRecord> {
    let mut out = HashMap::new();
    for entry in archive.entries() {
        if !entry.name.to_ascii_lowercase().ends_with(".tik") {
            continue;
        }
        let bytes = archive.entry_bytes(data, entry);
        if let Ok(ticket) = crate::container::nca::parse_ticket_record(bytes) {
            out.insert(ticket.rights_id, ticket);
        }
    }
    out
}

fn should_convert_nca_entry(
    name: &str,
    bytes: &[u8],
    size: u64,
    fallback_largest_size: Option<u64>,
    header_key: Option<&[u8; 32]>,
) -> bool {
    let name_lower = name.to_ascii_lowercase();
    let has_nca_ext = Path::new(name)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|ext| ext.eq_ignore_ascii_case("nca"));
    if !has_nca_ext || name_lower.ends_with(".cnmt.nca") {
        return false;
    }
    if bytes.len() <= UNCOMPRESSABLE_HEADER_SIZE {
        return false;
    }

    if let Some(key) = header_key {
        match crate::container::nca::analyze_for_compression(bytes, key) {
            Ok(meta) => return meta.is_compressible(),
            Err(err) => {
                if std::env::var("NSZ_DEBUG_COMPRESS_PLAN").ok().as_deref() == Some("1") {
                    eprintln!("[compress-plan] analyze failed for {name}: {err}");
                }
            }
        }
    }

    Some(size) == fallback_largest_size
}

const fn effective_solid_threads(request_threads: i32) -> i32 {
    if request_threads > 0 {
        request_threads
    } else {
        3
    }
}

fn debug_plan_failure(entry_name: &str, error: &NszError) {
    if std::env::var("NSZ_DEBUG_COMPRESS_PLAN").ok().as_deref() == Some("1") {
        eprintln!("[compress-plan] failed for {entry_name}: {error}");
    }
}

fn resolve_keyset() -> Option<NcaKeySet> {
    for candidate in candidate_key_paths() {
        let Ok(content) = fs::read_to_string(&candidate) else {
            continue;
        };
        if let Ok(keyset) = NcaKeySet::from_keys_str(&content) {
            return Some(keyset);
        }
    }
    None
}

fn candidate_key_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(path) = std::env::var("NSZ_KEYS_FILE") {
        out.push(PathBuf::from(path));
    }
    out.push(PathBuf::from("prod.keys"));
    out.push(PathBuf::from("keys.txt"));
    out.push(PathBuf::from(
        "/home/matteo/Documents/prog/python/nsz/prod.keys",
    ));
    out.push(PathBuf::from(
        "/home/matteo/Documents/prog/python/nsz/keys.txt",
    ));

    if let Ok(home) = std::env::var("HOME") {
        let home_dir = PathBuf::from(home);
        out.push(home_dir.join(".switch").join("prod.keys"));
        out.push(home_dir.join(".switch").join("keys.txt"));
    }

    out
}

fn build_python_compress_args(request: &CompressRequest) -> Vec<String> {
    let mut args = vec![
        "-C".to_string(),
        "-l".to_string(),
        request.level.to_string(),
        "-s".to_string(),
        request.block_size_exponent.to_string(),
        "-m".to_string(),
        request.multi.to_string(),
        "-t".to_string(),
        request.threads.to_string(),
    ];

    if request.long_distance_mode {
        args.push("-L".to_string());
    }
    if request.block {
        args.push("-B".to_string());
    }
    if request.solid {
        args.push("-S".to_string());
    }
    if request.verify {
        args.push("-V".to_string());
    }
    if request.quick_verify {
        args.push("-Q".to_string());
    }
    if request.keep {
        args.push("-K".to_string());
    }
    if request.fix_padding {
        args.push("-F".to_string());
    }
    if request.parse_cnmt {
        args.push("-p".to_string());
    }
    if request.always_parse_cnmt {
        args.push("-P".to_string());
    }
    if request.overwrite {
        args.push("-w".to_string());
    }
    if request.rm_old_version {
        args.push("-r".to_string());
    }
    if request.rm_source {
        args.push("--rm-source".to_string());
    }
    if let Some(out_dir) = &request.output_dir {
        args.push("-o".to_string());
        args.push(out_dir.display().to_string());
    }

    args
}

fn normalized_extension(path: &Path) -> Option<&str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("nsp") => Some("nsp"),
        Some(ext) if ext.eq_ignore_ascii_case("xci") => Some("xci"),
        Some(ext) if ext.eq_ignore_ascii_case("nca") => Some("nca"),
        _ => None,
    }
}

fn expected_compressed_output(input: &Path, output_dir: Option<&Path>) -> Option<PathBuf> {
    let mut output_name = PathBuf::from(input.file_name()?);
    let extension = input.extension()?.to_string_lossy().to_ascii_lowercase();

    output_name.set_extension(match extension.as_str() {
        "nsp" => "nsz",
        "xci" => "xcz",
        "nca" => "ncz",
        _ => return None,
    });

    if let Some(out_dir) = output_dir {
        return Some(out_dir.join(output_name));
    }

    Some(input.parent()?.join(output_name))
}
