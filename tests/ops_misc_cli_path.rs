use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn make_temp_root(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("nsz-rs-{prefix}-{unique}"))
}

fn write_fake_repo(repo_root: &std::path::Path) {
    fs::create_dir_all(repo_root).expect("create fake repo");
    let script = r#"#!/usr/bin/env python3
import pathlib
import sys

args = sys.argv[1:]
pathlib.Path("args.txt").write_text("\n".join(args), encoding="utf-8")
pathlib.Path("invoked.txt").write_text("ok", encoding="utf-8")
if "-c" in args:
    index = args.index("-c")
    if index + 1 < len(args):
        output = pathlib.Path(args[index + 1])
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(b"dummy")
"#;
    fs::write(repo_root.join("nsz.py"), script).expect("write fake nsz.py");
}

#[test]
fn extract_invokes_cli_and_reports_output_dirs() {
    let temp_root = make_temp_root("extract");
    let repo_root = temp_root.join("fake-nsz");
    let input_root = temp_root.join("input");
    let output_root = temp_root.join("output");
    write_fake_repo(&repo_root);
    fs::create_dir_all(&input_root).expect("create input dir");
    fs::create_dir_all(&output_root).expect("create output dir");

    let input = input_root.join("sample.nsz");
    fs::write(&input, b"dummy").expect("write input");

    let report = nsz_rs::extract(&nsz_rs::ExtractRequest {
        files: vec![input.clone()],
        output_dir: Some(output_root.clone()),
        extract_regex: Some("^.*\\.nca$".to_string()),
        python_repo_root: Some(repo_root.clone()),
    })
    .expect("extract should succeed");

    assert_eq!(
        report.processed_files,
        vec![output_root.join(input.file_stem().expect("stem"))]
    );
    assert!(repo_root.join("invoked.txt").exists());

    let args = fs::read_to_string(repo_root.join("args.txt")).expect("args file");
    assert!(args.contains("-x"), "args missing -x: {args}");
    assert!(args.contains("-o"), "args missing -o: {args}");
    assert!(
        args.contains(output_root.to_string_lossy().as_ref()),
        "args missing output path: {args}"
    );
    assert!(
        args.contains("--extractregex") && args.contains("^.*\\.nca$"),
        "args missing regex options: {args}"
    );
    assert!(
        args.contains(input.to_string_lossy().as_ref()),
        "args missing input path: {args}"
    );

    fs::remove_dir_all(&temp_root).expect("cleanup temp directory");
}

#[test]
fn create_invokes_cli_and_reports_output_file() {
    let temp_root = make_temp_root("create");
    let repo_root = temp_root.join("fake-nsz");
    let input_root = temp_root.join("input");
    write_fake_repo(&repo_root);
    fs::create_dir_all(&input_root).expect("create input dir");

    let source_a = input_root.join("a");
    let source_b = input_root.join("b");
    fs::write(&source_a, b"a").expect("source a");
    fs::write(&source_b, b"b").expect("source b");
    let output_file = temp_root.join("out").join("packed.nsp");

    let report = nsz_rs::create(&nsz_rs::CreateRequest {
        output_file: Some(output_file.clone()),
        sources: vec![source_a.clone(), source_b.clone()],
        fix_padding: true,
        python_repo_root: Some(repo_root.clone()),
    })
    .expect("create should succeed");

    assert_eq!(report.processed_files, vec![output_file.clone()]);
    assert!(repo_root.join("invoked.txt").exists());

    let args = fs::read_to_string(repo_root.join("args.txt")).expect("args file");
    assert!(args.contains("-c"), "args missing -c: {args}");
    assert!(args.contains("-F"), "args missing -F: {args}");
    assert!(
        args.contains(output_file.to_string_lossy().as_ref()),
        "args missing output path: {args}"
    );
    assert!(
        args.contains(source_a.to_string_lossy().as_ref())
            && args.contains(source_b.to_string_lossy().as_ref()),
        "args missing source paths: {args}"
    );

    fs::remove_dir_all(&temp_root).expect("cleanup temp directory");
}

#[test]
fn create_requires_output_file() {
    let err = nsz_rs::create(&nsz_rs::CreateRequest {
        output_file: None,
        sources: vec![],
        fix_padding: false,
        python_repo_root: None,
    })
    .expect_err("create should fail without output path");

    assert!(
        matches!(err, nsz_rs::NszError::ContainerFormat { .. }),
        "unexpected error: {err}"
    );
}

#[test]
fn titlekeys_invokes_cli_and_reports_inputs() {
    let temp_root = make_temp_root("titlekeys");
    let repo_root = temp_root.join("fake-nsz");
    let input_root = temp_root.join("input");
    write_fake_repo(&repo_root);
    fs::create_dir_all(&input_root).expect("create input dir");

    let input = input_root.join("sample.nsp");
    fs::write(&input, b"dummy").expect("write input");

    let report = nsz_rs::titlekeys(&nsz_rs::TitleKeysRequest {
        files: vec![input.clone()],
        python_repo_root: Some(repo_root.clone()),
    })
    .expect("titlekeys should succeed");

    assert_eq!(report.processed_files, vec![input.clone()]);

    let args = fs::read_to_string(repo_root.join("args.txt")).expect("args file");
    assert!(
        args.contains("--titlekeys") && args.contains(input.to_string_lossy().as_ref()),
        "args missing titlekeys invocation: {args}"
    );

    fs::remove_dir_all(&temp_root).expect("cleanup temp directory");
}

#[test]
fn undupe_invokes_cli_with_options() {
    let temp_root = make_temp_root("undupe");
    let repo_root = temp_root.join("fake-nsz");
    let input_root = temp_root.join("input");
    let output_root = temp_root.join("output");
    write_fake_repo(&repo_root);
    fs::create_dir_all(&input_root).expect("create input dir");
    fs::create_dir_all(&output_root).expect("create output dir");

    let input = input_root.join("sample.nsz");
    fs::write(&input, b"dummy").expect("write input");

    let report = nsz_rs::undupe(&nsz_rs::UndupeRequest {
        files: vec![input.clone()],
        output_dir: Some(output_root.clone()),
        dry_run: true,
        rename: true,
        hardlink: true,
        priority_list: Some("^.*\\.nsp$".to_string()),
        whitelist: Some("^.*\\.nsz$".to_string()),
        blacklist: Some("^.*\\.xci$".to_string()),
        old_versions: true,
        python_repo_root: Some(repo_root.clone()),
    })
    .expect("undupe should succeed");

    assert_eq!(report.processed_files, vec![input.clone()]);

    let args = fs::read_to_string(repo_root.join("args.txt")).expect("args file");
    assert!(
        args.contains("--undupe-dryrun"),
        "args missing dryrun: {args}"
    );
    assert!(
        args.contains("--undupe-rename"),
        "args missing rename: {args}"
    );
    assert!(
        args.contains("--undupe-hardlink"),
        "args missing hardlink: {args}"
    );
    assert!(
        args.contains("--undupe-prioritylist") && args.contains("^.*\\.nsp$"),
        "args missing prioritylist: {args}"
    );
    assert!(
        args.contains("--undupe-whitelist") && args.contains("^.*\\.nsz$"),
        "args missing whitelist: {args}"
    );
    assert!(
        args.contains("--undupe-blacklist") && args.contains("^.*\\.xci$"),
        "args missing blacklist: {args}"
    );
    assert!(
        args.contains("--undupe-old-versions"),
        "args missing old-versions: {args}"
    );
    assert!(
        args.contains("-o") && args.contains(output_root.to_string_lossy().as_ref()),
        "args missing output option: {args}"
    );

    fs::remove_dir_all(&temp_root).expect("cleanup temp directory");
}
