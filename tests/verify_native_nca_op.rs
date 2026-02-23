use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[test]
fn verify_uses_native_path_for_nca_inputs() {
    let payload = b"native-verify-nca-payload";
    let mut nca = vec![0u8; 0x4000];
    nca.extend_from_slice(payload);
    let hash = format!("{:x}", Sha256::digest(&nca));

    let root =
        std::env::temp_dir().join(format!("nsz-rs-native-verify-nca-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let input = root.join(format!("{hash}.nca"));
    fs::write(&input, nca).unwrap();

    let report = nsz_rs::verify(&nsz_rs::VerifyRequest {
        files: vec![input.clone()],
        fix_padding: false,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    })
    .unwrap();

    assert_eq!(report.verified_files, vec![input]);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn verify_skips_cnmt_nca_hash_check() {
    let payload = b"native-verify-cnmt-nca-payload";
    let mut nca = vec![0u8; 0x4000];
    nca.extend_from_slice(payload);

    let root = std::env::temp_dir().join(format!(
        "nsz-rs-native-verify-cnmt-nca-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let input = root.join("not-a-hash.cnmt.nca");
    fs::write(&input, nca).unwrap();

    let report = nsz_rs::verify(&nsz_rs::VerifyRequest {
        files: vec![input.clone()],
        fix_padding: false,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    })
    .unwrap();

    assert_eq!(report.verified_files, vec![input]);
    let _ = fs::remove_dir_all(root);
}
