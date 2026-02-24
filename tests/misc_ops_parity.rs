use std::collections::BTreeMap;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

#[test]
fn misc_ops_match_python_behavior_on_fixture_set() {
    if std::env::var("NSZ_RUN_HEAVY_MISC_PARITY").ok().as_deref() != Some("1") {
        return;
    }

    let base_python_repo = PathBuf::from("/home/matteo/Documents/prog/python/nsz");
    let local_python =
        PathBuf::from("/home/matteo/Documents/prog/rust/nsz-rs/.venv-nsz-baseline/bin/python3");
    if local_python.exists() {
        std::env::set_var("NSZ_PYTHON_BIN", &local_python);
    }

    let temp_root = std::env::temp_dir().join(format!("nsz-rs-misc-parity-{}", std::process::id()));
    let baseline_home = temp_root.join("home");
    if !prepare_home_with_keys(&base_python_repo, &baseline_home) {
        return;
    }
    std::env::set_var("HOME", &baseline_home);

    let python_repo = temp_root.join("python-repo-main");
    prepare_isolated_python_repo(&base_python_repo, &python_repo).unwrap();

    let corpus_root = PathBuf::from("/home/matteo/Documents/switch_games/Bad Cheese [NSP]");
    let fixtures = apply_heavy_misc_mode(collect_files_with_extension(&corpus_root, "nsp"));
    for source_nsp in &fixtures {
        eprintln!("[misc-parity] fixture: {}", source_nsp.display());
        run_extract_create_parity_for_fixture(&python_repo, &temp_root, source_nsp);
        run_titlekeys_parity_for_fixture(&base_python_repo, &temp_root, source_nsp);
        run_undupe_parity_for_fixture(&python_repo, &temp_root, source_nsp);
    }

    let _ = fs::remove_dir_all(temp_root);
}

fn run_extract_create_parity_for_fixture(python_repo: &Path, temp_root: &Path, source_nsp: &Path) {
    let fixture_id = source_nsp
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("fixture")
        .replace('/', "_");

    let baseline_out = temp_root
        .join("misc-baseline")
        .join("extract")
        .join(&fixture_id);
    let rust_out = temp_root
        .join("misc-rust")
        .join("extract")
        .join(&fixture_id);
    fs::create_dir_all(&baseline_out).unwrap();
    fs::create_dir_all(&rust_out).unwrap();

    let extract_regex = String::from("^.*\\.cnmt\\.nca$");
    nsz_rs::parity::python_runner::run_nsz_cli(
        python_repo,
        &[
            "-x".to_string(),
            "-o".to_string(),
            baseline_out.display().to_string(),
            "--extractregex".to_string(),
            extract_regex.clone(),
            source_nsp.display().to_string(),
        ],
    )
    .unwrap();

    let extract_report = nsz_rs::extract(&nsz_rs::ExtractRequest {
        files: vec![source_nsp.to_path_buf()],
        output_dir: Some(rust_out.clone()),
        extract_regex: Some(extract_regex),
        python_repo_root: Some(python_repo.to_path_buf()),
    })
    .unwrap();

    let extracted_dir_name = source_nsp.file_stem().unwrap();
    let baseline_extract_dir = baseline_out.join(extracted_dir_name);
    let rust_extract_dir = rust_out.join(extracted_dir_name);
    assert!(
        extract_report.processed_files.contains(&rust_extract_dir),
        "extract report does not contain expected directory {}",
        rust_extract_dir.display()
    );
    assert_directory_files_equal_or_panic(&baseline_extract_dir, &rust_extract_dir, "extract");

    let relative_sources = collect_relative_files(&baseline_extract_dir);
    assert!(
        !relative_sources.is_empty(),
        "extract produced no sources for create parity on {}",
        source_nsp.display()
    );
    let baseline_sources = relative_sources
        .iter()
        .map(|rel| baseline_extract_dir.join(rel))
        .collect::<Vec<_>>();
    let rust_sources = relative_sources
        .iter()
        .map(|rel| rust_extract_dir.join(rel))
        .collect::<Vec<_>>();

    let baseline_create_out = temp_root
        .join("misc-baseline")
        .join("create")
        .join(format!("{fixture_id}.nsp"));
    let rust_create_out = temp_root
        .join("misc-rust")
        .join("create")
        .join(format!("{fixture_id}.nsp"));
    fs::create_dir_all(baseline_create_out.parent().unwrap()).unwrap();
    fs::create_dir_all(rust_create_out.parent().unwrap()).unwrap();

    let mut create_args = vec!["-c".to_string(), baseline_create_out.display().to_string()];
    for source in &baseline_sources {
        create_args.push(source.display().to_string());
    }
    nsz_rs::parity::python_runner::run_nsz_cli(python_repo, &create_args).unwrap();

    let create_report = nsz_rs::create(&nsz_rs::CreateRequest {
        output_file: Some(rust_create_out.clone()),
        sources: rust_sources,
        fix_padding: false,
        python_repo_root: Some(python_repo.to_path_buf()),
    })
    .unwrap();
    assert_eq!(create_report.processed_files, vec![rust_create_out.clone()]);
    assert_files_equal_or_panic(&baseline_create_out, &rust_create_out, "create");
}

fn run_titlekeys_parity_for_fixture(base_python_repo: &Path, temp_root: &Path, source_nsp: &Path) {
    let fixture_id = source_nsp
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("fixture")
        .replace('/', "_");

    let baseline_repo = temp_root
        .join("python-repo-titlekeys-baseline")
        .join(&fixture_id);
    let rust_repo = temp_root
        .join("python-repo-titlekeys-rust")
        .join(&fixture_id);
    prepare_isolated_python_repo(base_python_repo, &baseline_repo).unwrap();
    prepare_isolated_python_repo(base_python_repo, &rust_repo).unwrap();

    nsz_rs::parity::python_runner::run_nsz_cli(
        &baseline_repo,
        &["--titlekeys".to_string(), source_nsp.display().to_string()],
    )
    .unwrap();

    let report = nsz_rs::titlekeys(&nsz_rs::TitleKeysRequest {
        files: vec![source_nsp.to_path_buf()],
        python_repo_root: Some(rust_repo.clone()),
    })
    .unwrap();
    assert_eq!(report.processed_files, vec![source_nsp.to_path_buf()]);

    let baseline_titlekeys = baseline_repo.join("titlekeys.txt");
    let rust_titlekeys = rust_repo.join("titlekeys.txt");
    assert!(baseline_titlekeys.exists());
    assert!(rust_titlekeys.exists());
    assert_files_equal_or_panic(&baseline_titlekeys, &rust_titlekeys, "titlekeys");
}

fn run_undupe_parity_for_fixture(python_repo: &Path, temp_root: &Path, source_nsp: &Path) {
    let fixture_id = source_nsp
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("fixture")
        .replace('/', "_");
    let file_name = source_nsp.file_name().and_then(|n| n.to_str()).unwrap();

    let baseline_case = temp_root
        .join("misc-baseline")
        .join("undupe")
        .join(&fixture_id);
    let rust_case = temp_root.join("misc-rust").join("undupe").join(&fixture_id);
    let baseline_input = baseline_case.join("input");
    let rust_input = rust_case.join("input");
    fs::create_dir_all(&baseline_input).unwrap();
    fs::create_dir_all(&rust_input).unwrap();

    let baseline_a = baseline_input.join(format!("A-{file_name}"));
    let baseline_b = baseline_input.join(format!("B-{file_name}"));
    let rust_a = rust_input.join(format!("A-{file_name}"));
    let rust_b = rust_input.join(format!("B-{file_name}"));
    fs::write(&baseline_a, b"seed").unwrap();
    fs::write(&baseline_b, b"seed").unwrap();
    fs::write(&rust_a, b"seed").unwrap();
    fs::write(&rust_b, b"seed").unwrap();

    nsz_rs::parity::python_runner::run_nsz_cli(
        python_repo,
        &[
            "--undupe-dryrun".to_string(),
            baseline_input.display().to_string(),
        ],
    )
    .unwrap();

    let undupe_report = nsz_rs::undupe(&nsz_rs::UndupeRequest {
        files: vec![rust_input.clone()],
        output_dir: None,
        dry_run: true,
        rename: false,
        hardlink: false,
        priority_list: None,
        whitelist: None,
        blacklist: None,
        old_versions: false,
        python_repo_root: Some(python_repo.to_path_buf()),
    })
    .unwrap();
    assert_eq!(undupe_report.processed_files, vec![rust_input.clone()]);

    assert_directory_sizes_equal_or_panic(&baseline_input, &rust_input, "undupe");
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
            .map(|ext| ext.eq_ignore_ascii_case(extension))
            .unwrap_or(false);
        if matches {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn apply_heavy_misc_mode(mut fixtures: Vec<PathBuf>) -> Vec<PathBuf> {
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
    if fs::create_dir_all(&switch_dir).is_err() {
        return false;
    }

    let target = switch_dir.join("keys.txt");
    fs::copy(source_key, target).is_ok()
}

fn prepare_isolated_python_repo(base_repo: &Path, target_repo: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target_repo)?;
    fs::copy(base_repo.join("nsz.py"), target_repo.join("nsz.py"))?;
    link_or_copy_dir(&base_repo.join("nsz"), &target_repo.join("nsz"))
}

fn link_or_copy_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        if std::os::unix::fs::symlink(source, target).is_ok() {
            return Ok(());
        }
    }
    copy_dir_recursive(source, target)
}

fn copy_dir_recursive(source: &Path, target: &Path) -> std::io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let from = entry.path();
        let to = target.join(entry.file_name());
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ty.is_file() {
            fs::copy(from, to)?;
        }
    }
    Ok(())
}

fn collect_relative_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_relative_files_inner(root, root, &mut out);
    out.sort();
    out
}

fn collect_relative_files_inner(root: &Path, current: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_relative_files_inner(root, &path, out);
            continue;
        }
        if path.is_file() {
            let rel = path.strip_prefix(root).unwrap().to_path_buf();
            out.push(rel);
        }
    }
}

fn assert_directory_files_equal_or_panic(left: &Path, right: &Path, label: &str) {
    let left_files = collect_relative_files(left);
    let right_files = collect_relative_files(right);
    assert_eq!(
        left_files,
        right_files,
        "{label} file-list mismatch: left={} right={}",
        left.display(),
        right.display()
    );

    for rel in &left_files {
        let left_file = left.join(rel);
        let right_file = right.join(rel);
        assert_files_equal_or_panic(&left_file, &right_file, label);
    }
}

fn assert_directory_sizes_equal_or_panic(left: &Path, right: &Path, label: &str) {
    let left_sizes = collect_relative_sizes(left);
    let right_sizes = collect_relative_sizes(right);
    assert_eq!(
        left_sizes,
        right_sizes,
        "{label} size-map mismatch: left={} right={}",
        left.display(),
        right.display()
    );
}

fn collect_relative_sizes(root: &Path) -> BTreeMap<PathBuf, u64> {
    let mut out = BTreeMap::new();
    collect_relative_sizes_inner(root, root, &mut out);
    out
}

fn collect_relative_sizes_inner(root: &Path, current: &Path, out: &mut BTreeMap<PathBuf, u64>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_relative_sizes_inner(root, &path, out);
            continue;
        }
        if path.is_file() {
            let rel = path.strip_prefix(root).unwrap().to_path_buf();
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            out.insert(rel, size);
        }
    }
}

fn assert_files_equal_or_panic(left: &Path, right: &Path, label: &str) {
    if files_equal(left, right) {
        return;
    }

    let left_size = fs::metadata(left).map(|m| m.len()).unwrap_or(0);
    let right_size = fs::metadata(right).map(|m| m.len()).unwrap_or(0);
    panic!(
        "{label} byte mismatch: left={} ({} bytes) right={} ({} bytes)",
        left.display(),
        left_size,
        right.display(),
        right_size
    );
}

fn files_equal(left: &Path, right: &Path) -> bool {
    let left_meta = match fs::metadata(left) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let right_meta = match fs::metadata(right) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if left_meta.len() != right_meta.len() {
        return false;
    }

    let left_file = match fs::File::open(left) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let right_file = match fs::File::open(right) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let mut left_reader = BufReader::new(left_file);
    let mut right_reader = BufReader::new(right_file);
    let mut left_buf = vec![0u8; 1024 * 1024];
    let mut right_buf = vec![0u8; 1024 * 1024];

    loop {
        let read_left = match left_reader.read(&mut left_buf) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let read_right = match right_reader.read(&mut right_buf) {
            Ok(v) => v,
            Err(_) => return false,
        };
        if read_left != read_right {
            return false;
        }
        if read_left == 0 {
            return true;
        }
        if left_buf[..read_left] != right_buf[..read_right] {
            return false;
        }
    }
}
