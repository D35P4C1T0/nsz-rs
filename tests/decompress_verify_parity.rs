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
    let source_nsz = corpus_root.join("Bad Cheese [0100BAE021208000][v0].nsz");

    let baseline_out = temp_root.join("baseline");
    let rust_out = temp_root.join("rust");
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
        python_repo_root: Some(python_repo.clone()),
    })
    .unwrap();

    let baseline_nsp = baseline_out.join("Bad Cheese [0100BAE021208000][v0].nsp");
    let rust_nsp = rust_out.join("Bad Cheese [0100BAE021208000][v0].nsp");

    assert!(report.processed_files.contains(&rust_nsp));
    assert!(files_equal(&baseline_nsp, &rust_nsp));

    let verify = nsz_rs::verify(&nsz_rs::VerifyRequest {
        files: vec![rust_nsp.clone()],
        fix_padding: false,
        python_repo_root: Some(python_repo),
    })
    .unwrap();

    assert_eq!(verify.verified_files, vec![rust_nsp]);

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
    if std::fs::create_dir_all(&switch_dir).is_err() {
        return false;
    }

    let target = switch_dir.join("keys.txt");
    std::fs::copy(source_key, target).is_ok()
}

fn files_equal(a: &Path, b: &Path) -> bool {
    let meta_a = match fs::metadata(a) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let meta_b = match fs::metadata(b) {
        Ok(v) => v,
        Err(_) => return false,
    };

    if meta_a.len() != meta_b.len() {
        return false;
    }

    let file_a = match fs::File::open(a) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let file_b = match fs::File::open(b) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let mut reader_a = BufReader::new(file_a);
    let mut reader_b = BufReader::new(file_b);

    let mut buf_a = vec![0u8; 1024 * 1024];
    let mut buf_b = vec![0u8; 1024 * 1024];

    loop {
        let read_a = match reader_a.read(&mut buf_a) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let read_b = match reader_b.read(&mut buf_b) {
            Ok(v) => v,
            Err(_) => return false,
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
