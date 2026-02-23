use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn compress_invokes_cli_and_reports_outputs() {
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

out_dir = None
files = []
i = 0
while i < len(args):
    arg = args[i]
    if arg == "-o":
        out_dir = pathlib.Path(args[i + 1])
        i += 2
        continue
    if arg in ("-l", "-s", "-m", "-t"):
        i += 2
        continue
    if arg.startswith("-"):
        i += 1
        continue
    files.append(pathlib.Path(arg))
    i += 1

for file in files:
    suffix = file.suffix.lower()
    if suffix == ".nsp":
        target = (out_dir if out_dir is not None else file.parent) / (file.stem + ".nsz")
        target.write_bytes(b"nsz")
    elif suffix == ".xci":
        target = (out_dir if out_dir is not None else file.parent) / (file.stem + ".xcz")
        target.write_bytes(b"xcz")
"#;
    fs::write(repo_root.join("nsz.py"), script).expect("write fake nsz.py");

    let nsp_input = input_root.join("game.nsp");
    let xci_input = input_root.join("cart.xci");
    fs::write(&nsp_input, b"dummy").expect("write nsp input");
    fs::write(&xci_input, b"dummy").expect("write xci input");

    let report = nsz_rs::compress(&nsz_rs::CompressRequest {
        files: vec![nsp_input.clone(), xci_input.clone()],
        output_dir: Some(output_root.clone()),
        python_repo_root: Some(repo_root.clone()),
        ..Default::default()
    })
    .expect("compress should succeed");

    let expected_nsz = output_root.join("game.nsz");
    let expected_xcz = output_root.join("cart.xcz");
    assert!(expected_nsz.exists(), "expected output {:?}", expected_nsz);
    assert!(expected_xcz.exists(), "expected output {:?}", expected_xcz);

    let mut processed = report
        .processed_files
        .iter()
        .map(PathBuf::as_path)
        .collect::<Vec<_>>();
    processed.sort_unstable();

    let mut expected = vec![expected_nsz.as_path(), expected_xcz.as_path()];
    expected.sort_unstable();
    assert_eq!(processed, expected);

    let args = fs::read_to_string(repo_root.join("args.txt")).expect("args file");
    assert!(args.contains("-C"), "args did not contain -C: {args}");
    assert!(args.contains("-o"), "args did not contain -o: {args}");

    fs::remove_dir_all(&temp_root).expect("cleanup temp directory");
}
