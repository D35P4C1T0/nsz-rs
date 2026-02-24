use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[test]
fn benchmark_python_vs_rust_on_same_inputs() {
    if std::env::var("NSZ_RUN_HEAVY_SPEED_BENCH").ok().as_deref() != Some("1") {
        return;
    }

    cleanup_tmp_artifacts();

    let python_repo = PathBuf::from("/home/matteo/Documents/prog/python/nsz");
    let local_python =
        PathBuf::from("/home/matteo/Documents/prog/rust/nsz-rs/.venv-nsz-baseline/bin/python3");
    if local_python.exists() {
        std::env::set_var("NSZ_PYTHON_BIN", &local_python);
    }

    let temp_root = std::env::temp_dir().join(format!("nsz-rs-speed-bench-{}", std::process::id()));
    let baseline_home = temp_root.join("home");
    if !prepare_home_with_keys(&python_repo, &baseline_home) {
        return;
    }
    std::env::set_var("HOME", &baseline_home);

    let corpus_root = PathBuf::from("/home/matteo/Documents/switch_games/Bad Cheese [NSP]");
    let compress_input =
        match benchmark_input_from_env_or_corpus("NSZ_BENCH_COMPRESS_INPUT", &corpus_root, "nsp") {
            Some(path) => path,
            None => return,
        };

    let compress_baseline_out = temp_root.join("compress-baseline");
    let compress_rust_out = temp_root.join("compress-rust");
    fs::create_dir_all(&compress_baseline_out).unwrap();
    fs::create_dir_all(&compress_rust_out).unwrap();

    let python_compress_elapsed = elapsed(|| {
        nsz_rs::parity::python_runner::run_nsz_cli(
            &python_repo,
            &[
                "-C".to_string(),
                "-o".to_string(),
                compress_baseline_out.display().to_string(),
                compress_input.display().to_string(),
            ],
        )
    })
    .unwrap();

    let rust_compress_elapsed = elapsed(|| {
        nsz_rs::compress(&nsz_rs::CompressRequest {
            files: vec![compress_input.clone()],
            output_dir: Some(compress_rust_out.clone()),
            python_repo_root: Some(PathBuf::from("/does/not/exist")),
            ..Default::default()
        })
        .map(|_| ())
    })
    .unwrap();

    let compressed_name = compress_input
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap()
        .replace(".nsp", ".nsz");
    let baseline_nsz = compress_baseline_out.join(&compressed_name);
    let rust_nsz = compress_rust_out.join(&compressed_name);
    assert!(files_equal(&baseline_nsz, &rust_nsz));

    eprintln!(
        "[speed-bench][compress] input={} python_ms={} rust_ms={} speedup={:.3}x",
        compress_input.display(),
        python_compress_elapsed.as_millis(),
        rust_compress_elapsed.as_millis(),
        speedup_ratio(python_compress_elapsed, rust_compress_elapsed)
    );

    let decompress_input =
        benchmark_input_from_env_or_corpus("NSZ_BENCH_DECOMPRESS_INPUT", &corpus_root, "nsz")
            .unwrap_or_else(|| baseline_nsz.clone());
    let decompress_baseline_out = temp_root.join("decompress-baseline");
    let decompress_rust_out = temp_root.join("decompress-rust");
    fs::create_dir_all(&decompress_baseline_out).unwrap();
    fs::create_dir_all(&decompress_rust_out).unwrap();

    let python_decompress_elapsed = elapsed(|| {
        nsz_rs::parity::python_runner::run_nsz_cli(
            &python_repo,
            &[
                "-D".to_string(),
                "-o".to_string(),
                decompress_baseline_out.display().to_string(),
                decompress_input.display().to_string(),
            ],
        )
    })
    .unwrap();

    let rust_decompress_elapsed = elapsed(|| {
        nsz_rs::decompress(&nsz_rs::DecompressRequest {
            files: vec![decompress_input.clone()],
            output_dir: Some(decompress_rust_out.clone()),
            fix_padding: false,
            python_repo_root: Some(PathBuf::from("/does/not/exist")),
        })
        .map(|_| ())
    })
    .unwrap();

    let decompressed_name = decompress_input
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap()
        .replace(".nsz", ".nsp");
    let baseline_nsp = decompress_baseline_out.join(&decompressed_name);
    let rust_nsp = decompress_rust_out.join(&decompressed_name);
    assert!(files_equal(&baseline_nsp, &rust_nsp));

    eprintln!(
        "[speed-bench][decompress] input={} python_ms={} rust_ms={} speedup={:.3}x",
        decompress_input.display(),
        python_decompress_elapsed.as_millis(),
        rust_decompress_elapsed.as_millis(),
        speedup_ratio(python_decompress_elapsed, rust_decompress_elapsed)
    );

    let _ = fs::remove_dir_all(temp_root);
}

fn elapsed<T, E>(run: impl FnOnce() -> Result<T, E>) -> Result<Duration, E> {
    let start = Instant::now();
    run()?;
    Ok(start.elapsed())
}

fn speedup_ratio(python: Duration, rust: Duration) -> f64 {
    let rust_secs = rust.as_secs_f64();
    if rust_secs <= 0.0 {
        return f64::INFINITY;
    }
    python.as_secs_f64() / rust_secs
}

fn benchmark_input_from_env_or_corpus(
    env_key: &str,
    corpus_root: &Path,
    ext: &str,
) -> Option<PathBuf> {
    if let Ok(path) = std::env::var(env_key) {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }

    let mut fixtures = collect_files_with_extension(corpus_root, ext);
    fixtures.sort();
    fixtures.into_iter().next()
}

fn collect_files_with_extension(root: &Path, ext: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_recursive(root, ext, &mut files);
    files
}

fn collect_recursive(root: &Path, ext: &str, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_recursive(&path, ext, files);
            continue;
        }

        if path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case(ext))
            == Some(true)
        {
            files.push(path);
        }
    }
}

fn prepare_home_with_keys(python_repo: &Path, home_dir: &Path) -> bool {
    let switch_dir = home_dir.join(".switch");
    if fs::create_dir_all(&switch_dir).is_err() {
        return false;
    }

    let source_candidates = [
        PathBuf::from("/home/matteo/Documents/prog/rust/nsz-rs/prod.keys"),
        PathBuf::from("/home/matteo/Documents/prog/rust/nsz-rs/keys.txt"),
        python_repo.join("prod.keys"),
        python_repo.join("keys.txt"),
    ];

    source_candidates.iter().any(|source_key| {
        if !source_key.exists() {
            return false;
        }
        let target = switch_dir.join(source_key.file_name().unwrap());
        fs::copy(source_key, target).is_ok()
    })
}

fn files_equal(left: &Path, right: &Path) -> bool {
    let left_bytes = match fs::read(left) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let right_bytes = match fs::read(right) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    left_bytes == right_bytes
}

fn cleanup_tmp_artifacts() {
    let tmp_dir = std::env::temp_dir();
    let Ok(entries) = fs::read_dir(&tmp_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with("nsz-rs-") {
            let _ = fs::remove_dir_all(path);
        }
    }
}
