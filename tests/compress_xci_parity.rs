use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

#[test]
fn compress_xci_matches_python_for_fixture() {
    if std::env::var("NSZ_RUN_HEAVY_XCI_COMPRESS_PARITY")
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

    let temp_root = std::env::temp_dir().join(format!("nsz-rs-xci-parity-{}", std::process::id()));
    let baseline_home = temp_root.join("home");
    if !prepare_home_with_keys(&python_repo, &baseline_home) {
        return;
    }
    std::env::set_var("HOME", &baseline_home);

    let source_xci =
        PathBuf::from("/home/matteo/Documents/switch_games/xci_test/HEART of CROWN.xci");
    if !source_xci.exists() {
        return;
    }

    let baseline_out = temp_root.join("baseline");
    let rust_out = temp_root.join("rust");
    fs::create_dir_all(&baseline_out).unwrap();
    fs::create_dir_all(&rust_out).unwrap();

    nsz_rs::parity::python_runner::run_nsz_cli(
        &python_repo,
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
    assert_files_equal_or_panic(&baseline_xcz, &rust_xcz);

    let _ = fs::remove_dir_all(temp_root);
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

fn assert_files_equal_or_panic(left: &Path, right: &Path) {
    if files_equal(left, right) {
        return;
    }

    let offset = first_diff_offset(left, right);
    let left_size = fs::metadata(left).map(|m| m.len()).unwrap_or(0);
    let right_size = fs::metadata(right).map(|m| m.len()).unwrap_or(0);
    panic!(
        "byte mismatch: baseline={} ({} bytes) rust={} ({} bytes) first_diff_offset={:?}",
        left.display(),
        left_size,
        right.display(),
        right_size,
        offset
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

fn first_diff_offset(left: &Path, right: &Path) -> Option<u64> {
    let left_file = fs::File::open(left).ok()?;
    let right_file = fs::File::open(right).ok()?;
    let mut left_reader = BufReader::new(left_file);
    let mut right_reader = BufReader::new(right_file);
    let mut left_buf = vec![0u8; 1024 * 1024];
    let mut right_buf = vec![0u8; 1024 * 1024];
    let mut total = 0u64;

    loop {
        let read_left = left_reader.read(&mut left_buf).ok()?;
        let read_right = right_reader.read(&mut right_buf).ok()?;
        let shared = read_left.min(read_right);

        for i in 0..shared {
            if left_buf[i] != right_buf[i] {
                return Some(total + i as u64);
            }
        }

        if read_left != read_right {
            return Some(total + shared as u64);
        }
        if read_left == 0 {
            return None;
        }
        total += read_left as u64;
    }
}
