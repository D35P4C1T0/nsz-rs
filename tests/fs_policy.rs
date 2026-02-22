use std::fs;

#[test]
fn file_policy_rejects_duplicate_without_overwrite() {
    let root = std::env::temp_dir().join(format!("nsz-rs-fs-policy-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let source = root.join("Example.nsp");
    fs::write(&source, b"dummy").unwrap();

    let existing_target = root.join("Example.nsz");
    fs::write(&existing_target, b"existing").unwrap();

    let decision =
        nsz_rs::fs_ops::existing_checks::allow_write_outfile(&source, ".nsz", &root, false)
            .unwrap();

    assert_eq!(
        decision,
        nsz_rs::fs_ops::existing_checks::WriteDecision::DenyDuplicate
    );

    let _ = fs::remove_file(source);
    let _ = fs::remove_file(existing_target);
    let _ = fs::remove_dir(root);
}
