use std::fmt::Write as _;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

#[test]
fn decompress_verify_matches_python_for_fixture() {
    if std::env::var("NSZ_RUN_HEAVY_PARITY").ok().as_deref() != Some("1") {
        return;
    }

    let python_repo = PathBuf::from("/home/matteo/Documents/prog/python/nsz");
    let local_python =
        PathBuf::from("/home/matteo/Documents/prog/rust/nsz-rs/.venv-nsz-baseline/bin/python3");
    if local_python.exists() {
        std::env::set_var("NSZ_PYTHON_BIN", &local_python);
    }
    let temp_root = std::env::temp_dir().join(format!("nsz-rs-task9-{}", std::process::id()));
    let baseline_home = temp_root.join("home");
    if !prepare_home_with_keys(&python_repo, &baseline_home) {
        return;
    }
    std::env::set_var("HOME", &baseline_home);

    let corpus_root = PathBuf::from("/home/matteo/Documents/switch_games/Bad Cheese [NSP]");
    let nsz_fixtures = collect_files_with_extension(&corpus_root, "nsz");
    let nsz_fixtures = apply_heavy_fixture_mode(nsz_fixtures);
    for source_nsz in &nsz_fixtures {
        let fixture_id = source_nsz
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("fixture")
            .replace('/', "_");
        let baseline_out = temp_root.join("baseline").join(&fixture_id);
        let rust_out = temp_root.join("rust").join(&fixture_id);
        fs::create_dir_all(&baseline_out).unwrap();
        fs::create_dir_all(&rust_out).unwrap();

        nsz_rs::parity::python_runner::run_nsz_cli(
            &python_repo,
            &[
                "-D".to_string(),
                "-o".to_string(),
                baseline_out.display().to_string(),
                source_nsz.display().to_string(),
            ],
        )
        .unwrap();

        let report = nsz_rs::decompress(&nsz_rs::DecompressRequest {
            files: vec![source_nsz.clone()],
            output_dir: Some(rust_out.clone()),
            fix_padding: false,
            python_repo_root: Some(PathBuf::from("/does/not/exist")),
        })
        .unwrap();

        let output_name = source_nsz
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap()
            .replace(".nsz", ".nsp");
        let baseline_nsp = baseline_out.join(&output_name);
        let rust_nsp = rust_out.join(&output_name);

        assert!(report.processed_files.contains(&rust_nsp));
        assert!(files_equal(&baseline_nsp, &rust_nsp));

        nsz_rs::parity::python_runner::run_nsz_cli(
            &python_repo,
            &["-V".to_string(), source_nsz.display().to_string()],
        )
        .unwrap();
        let verify_nsz = nsz_rs::verify(&nsz_rs::VerifyRequest {
            files: vec![source_nsz.clone()],
            fix_padding: false,
            python_repo_root: Some(PathBuf::from("/does/not/exist")),
        })
        .unwrap();
        assert_eq!(verify_nsz.verified_files, vec![source_nsz.clone()]);

        let verify_decompressed_nsp = nsz_rs::verify(&nsz_rs::VerifyRequest {
            files: vec![rust_nsp.clone()],
            fix_padding: false,
            python_repo_root: Some(PathBuf::from("/does/not/exist")),
        })
        .unwrap();
        assert_eq!(verify_decompressed_nsp.verified_files, vec![rust_nsp]);
    }

    let plain_nsp_fixtures = collect_files_with_extension(&corpus_root, "nsp");
    let plain_nsp_fixtures = apply_heavy_fixture_mode(plain_nsp_fixtures);
    for source_nsp in &plain_nsp_fixtures {
        nsz_rs::parity::python_runner::run_nsz_cli(
            &python_repo,
            &["-V".to_string(), source_nsp.display().to_string()],
        )
        .unwrap();
        let verify_nsp = nsz_rs::verify(&nsz_rs::VerifyRequest {
            files: vec![source_nsp.clone()],
            fix_padding: false,
            python_repo_root: Some(PathBuf::from("/does/not/exist")),
        })
        .unwrap();
        assert_eq!(verify_nsp.verified_files, vec![source_nsp.clone()]);
    }

    if std::env::var("NSZ_RUN_HEAVY_COMPRESS_PARITY")
        .ok()
        .as_deref()
        == Some("1")
    {
        run_compress_parity_checks(&python_repo, &temp_root, &corpus_root);
    }

    let _ = fs::remove_dir_all(temp_root);
}

#[test]
fn compress_matches_python_for_fixture() {
    if std::env::var("NSZ_RUN_HEAVY_COMPRESS_PARITY")
        .ok()
        .as_deref()
        != Some("1")
    {
        return;
    }

    let python_repo = PathBuf::from("/home/matteo/Documents/prog/python/nsz");
    let local_python =
        PathBuf::from("/home/matteo/Documents/prog/rust/nsz-rs/.venv-nsz-baseline/bin/python3");
    if local_python.exists() {
        std::env::set_var("NSZ_PYTHON_BIN", &local_python);
    }
    let temp_root =
        std::env::temp_dir().join(format!("nsz-rs-compress-parity-{}", std::process::id()));
    let baseline_home = temp_root.join("home");
    if !prepare_home_with_keys(&python_repo, &baseline_home) {
        return;
    }
    std::env::set_var("HOME", &baseline_home);

    let corpus_root = PathBuf::from("/home/matteo/Documents/switch_games/Bad Cheese [NSP]");
    run_compress_parity_checks(&python_repo, &temp_root, &corpus_root);

    let _ = fs::remove_dir_all(temp_root);
}

fn run_compress_parity_checks(python_repo: &Path, temp_root: &Path, corpus_root: &Path) {
    let nsp_fixtures = apply_heavy_compress_mode(collect_files_with_extension(corpus_root, "nsp"));
    for source_nsp in &nsp_fixtures {
        eprintln!("[compress-parity] nsp fixture: {}", source_nsp.display());
        let fixture_id = source_nsp
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("fixture")
            .replace('/', "_");
        let baseline_out = temp_root.join("compress-baseline").join(&fixture_id);
        let rust_out = temp_root.join("compress-rust").join(&fixture_id);
        fs::create_dir_all(&baseline_out).unwrap();
        fs::create_dir_all(&rust_out).unwrap();

        nsz_rs::parity::python_runner::run_nsz_cli(
            python_repo,
            &[
                "-C".to_string(),
                "-o".to_string(),
                baseline_out.display().to_string(),
                source_nsp.display().to_string(),
            ],
        )
        .unwrap();

        let report = nsz_rs::compress(&nsz_rs::CompressRequest {
            files: vec![source_nsp.clone()],
            output_dir: Some(rust_out.clone()),
            python_repo_root: Some(PathBuf::from("/does/not/exist")),
            ..Default::default()
        })
        .unwrap();

        let output_name = source_nsp
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap()
            .replace(".nsp", ".nsz");
        let baseline_nsz = baseline_out.join(&output_name);
        let rust_nsz = rust_out.join(&output_name);
        assert!(report.processed_files.contains(&rust_nsz));

        assert_files_equal_or_panic(&baseline_nsz, &rust_nsz, Some("nsp"));
    }

    let include_xci = std::env::var("NSZ_HEAVY_COMPRESS_INCLUDE_XCI")
        .ok()
        .as_deref()
        == Some("1");
    if !include_xci {
        return;
    }

    let mut xci_fixtures = collect_files_with_extension(corpus_root, "xci");
    let extra_xci =
        PathBuf::from("/home/matteo/Documents/switch_games/xci_test/HEART of CROWN.xci");
    if extra_xci.exists() {
        xci_fixtures.push(extra_xci);
    }
    xci_fixtures.sort();
    xci_fixtures.dedup();
    let xci_fixtures = apply_heavy_compress_mode(xci_fixtures);

    for source_xci in &xci_fixtures {
        eprintln!("[compress-parity] xci fixture: {}", source_xci.display());
        let fixture_id = source_xci
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("fixture")
            .replace('/', "_");
        let baseline_out = temp_root
            .join("compress-baseline")
            .join(format!("{fixture_id}-xci"));
        let rust_out = temp_root
            .join("compress-rust")
            .join(format!("{fixture_id}-xci"));
        fs::create_dir_all(&baseline_out).unwrap();
        fs::create_dir_all(&rust_out).unwrap();

        nsz_rs::parity::python_runner::run_nsz_cli(
            python_repo,
            &[
                "-C".to_string(),
                "-o".to_string(),
                baseline_out.display().to_string(),
                source_xci.display().to_string(),
            ],
        )
        .unwrap();

        let report = nsz_rs::compress(&nsz_rs::CompressRequest {
            files: vec![source_xci.clone()],
            output_dir: Some(rust_out.clone()),
            python_repo_root: Some(PathBuf::from("/does/not/exist")),
            ..Default::default()
        })
        .unwrap();

        let output_name = source_xci
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap()
            .replace(".xci", ".xcz");
        let baseline_xcz = baseline_out.join(&output_name);
        let rust_xcz = rust_out.join(&output_name);
        assert!(report.processed_files.contains(&rust_xcz));

        assert_files_equal_or_panic(&baseline_xcz, &rust_xcz, Some("xci"));
    }
}

fn collect_files_with_extension(root: &Path, extension: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return out;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let matches = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case(extension));
        if matches {
            out.push(path);
        }
    }

    out.sort();
    out
}

fn apply_heavy_fixture_mode(mut fixtures: Vec<PathBuf>) -> Vec<PathBuf> {
    let mode = std::env::var("NSZ_HEAVY_PARITY_MODE")
        .unwrap_or_else(|_| "full".to_string())
        .to_ascii_lowercase();
    if mode == "fast" && fixtures.len() > 1 {
        fixtures.truncate(1);
    }

    if let Ok(limit) = std::env::var("NSZ_HEAVY_PARITY_MAX_FILES") {
        if let Ok(parsed) = limit.parse::<usize>() {
            if parsed < fixtures.len() {
                fixtures.truncate(parsed);
            }
        }
    }

    fixtures
}

fn apply_heavy_compress_mode(mut fixtures: Vec<PathBuf>) -> Vec<PathBuf> {
    let mode = std::env::var("NSZ_HEAVY_PARITY_MODE")
        .unwrap_or_else(|_| "full".to_string())
        .to_ascii_lowercase();

    fixtures.sort();
    if mode == "fast" && fixtures.len() > 1 {
        fixtures.sort_by_key(|path| fs::metadata(path).map(|m| m.len()).unwrap_or(u64::MAX));
        fixtures.truncate(1);
    }

    if let Ok(limit) = std::env::var("NSZ_HEAVY_PARITY_MAX_FILES") {
        if let Ok(parsed) = limit.parse::<usize>() {
            if parsed < fixtures.len() {
                fixtures.truncate(parsed);
            }
        }
    }

    fixtures
}

fn prepare_home_with_keys(python_repo: &Path, target_home: &Path) -> bool {
    let mut candidates = vec![
        python_repo.join("prod.keys"),
        python_repo.join("keys.txt"),
        PathBuf::from("prod.keys"),
        PathBuf::from("keys.txt"),
    ];

    if let Ok(home) = std::env::var("HOME") {
        let home_dir = PathBuf::from(home);
        candidates.push(home_dir.join(".switch").join("prod.keys"));
        candidates.push(home_dir.join(".switch").join("keys.txt"));
    }

    let Some(source_key) = candidates.into_iter().find(|p| p.exists()) else {
        return false;
    };

    let switch_dir = target_home.join(".switch");
    if std::fs::create_dir_all(&switch_dir).is_err() {
        return false;
    }

    let target = switch_dir.join("keys.txt");
    std::fs::copy(source_key, target).is_ok()
}

fn files_equal(a: &Path, b: &Path) -> bool {
    let Ok(meta_a) = fs::metadata(a) else {
        return false;
    };
    let Ok(meta_b) = fs::metadata(b) else {
        return false;
    };

    if meta_a.len() != meta_b.len() {
        return false;
    }

    let Ok(file_a) = fs::File::open(a) else {
        return false;
    };
    let Ok(file_b) = fs::File::open(b) else {
        return false;
    };

    let mut reader_a = BufReader::new(file_a);
    let mut reader_b = BufReader::new(file_b);

    let mut buf_a = vec![0u8; 1024 * 1024];
    let mut buf_b = vec![0u8; 1024 * 1024];

    loop {
        let Ok(read_a) = reader_a.read(&mut buf_a) else {
            return false;
        };
        let Ok(read_b) = reader_b.read(&mut buf_b) else {
            return false;
        };

        if read_a != read_b {
            return false;
        }
        if read_a == 0 {
            return true;
        }
        if buf_a[..read_a] != buf_b[..read_b] {
            return false;
        }
    }
}

fn assert_files_equal_or_panic(a: &Path, b: &Path, kind: Option<&str>) {
    if files_equal(a, b) {
        return;
    }

    let offset = first_diff_offset(a, b);
    let size_a = fs::metadata(a).map(|m| m.len()).unwrap_or(0);
    let size_b = fs::metadata(b).map(|m| m.len()).unwrap_or(0);
    let mut details = format!(
        "byte mismatch: baseline={} ({} bytes) rust={} ({} bytes) first_diff_offset={:?}",
        a.display(),
        size_a,
        b.display(),
        size_b,
        offset
    );

    if kind == Some("nsp") {
        let baseline_entries = nsp_entry_names(a);
        let rust_entries = nsp_entry_names(b);
        let _ = write!(
            details,
            " baseline_entries={baseline_entries:?} rust_entries={rust_entries:?}"
        );
    }

    panic!("{details}");
}

fn first_diff_offset(a: &Path, b: &Path) -> Option<u64> {
    let file_a = fs::File::open(a).ok()?;
    let file_b = fs::File::open(b).ok()?;
    let mut reader_a = BufReader::new(file_a);
    let mut reader_b = BufReader::new(file_b);
    let mut buf_a = vec![0u8; 1024 * 1024];
    let mut buf_b = vec![0u8; 1024 * 1024];
    let mut total = 0u64;

    loop {
        let read_a = reader_a.read(&mut buf_a).ok()?;
        let read_b = reader_b.read(&mut buf_b).ok()?;
        let shared = read_a.min(read_b);

        for i in 0..shared {
            if buf_a[i] != buf_b[i] {
                return Some(total + i as u64);
            }
        }

        if read_a != read_b {
            return Some(total + shared as u64);
        }
        if read_a == 0 {
            return None;
        }
        total += read_a as u64;
    }
}

fn nsp_entry_names(path: &Path) -> Vec<String> {
    let Ok(data) = fs::read(path) else {
        return Vec::new();
    };
    let Ok(archive) = nsz_rs::container::nsp::NspArchive::from_bytes(&data) else {
        return Vec::new();
    };
    archive
        .entries()
        .iter()
        .map(|entry| entry.name.clone())
        .collect()
}
