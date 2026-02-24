use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn compress_invokes_cli_for_fallback_inputs() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("nsz-rs-compress-test-{unique}"));
    let repo_root = temp_root.join("fake-nsz");
    let input_root = temp_root.join("input");
    let output_root = temp_root.join("output");

    fs::create_dir_all(&repo_root).expect("create fake repo");
    fs::create_dir_all(&input_root).expect("create input dir");
    fs::create_dir_all(&output_root).expect("create output dir");

    let script = r#"#!/usr/bin/env python3
import pathlib
import sys

args = sys.argv[1:]
pathlib.Path("args.txt").write_text(" ".join(args), encoding="utf-8")

pathlib.Path("invoked.txt").write_text("ok", encoding="utf-8")
"#;
    fs::write(repo_root.join("nsz.py"), script).expect("write fake nsz.py");

    let fallback_input = input_root.join("misc.txt");
    fs::write(&fallback_input, b"dummy").expect("write fallback input");

    let report = nsz_rs::compress(&nsz_rs::CompressRequest {
        files: vec![fallback_input.clone()],
        output_dir: Some(output_root),
        python_repo_root: Some(repo_root.clone()),
        ..Default::default()
    })
    .expect("compress should succeed");

    assert!(report.processed_files.is_empty());
    assert!(repo_root.join("invoked.txt").exists());

    let args = fs::read_to_string(repo_root.join("args.txt")).expect("args file");
    assert!(args.contains("-C"), "args did not contain -C: {args}");
    assert!(args.contains("-o"), "args did not contain -o: {args}");
    assert!(
        args.contains(fallback_input.to_string_lossy().as_ref()),
        "args did not include input file path: {args}"
    );

    fs::remove_dir_all(&temp_root).expect("cleanup temp directory");
}
