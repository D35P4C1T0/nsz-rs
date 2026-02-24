use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[test]
fn verify_uses_native_path_for_ncz_inputs() {
    let payload = b"native-verify-ncz-payload";
    let compressed = zstd::stream::encode_all(&payload[..], 1).unwrap();

    let mut nca = vec![0u8; 0x4000];
    nca.extend_from_slice(payload);
    let hash = Sha256::digest(&nca);
    let stem = format!("{hash:x}");

    let mut fixture = vec![0u8; 0x4000];
    fixture.extend_from_slice(b"NCZSECTN");
    fixture.extend_from_slice(&(1u64).to_le_bytes());
    fixture.extend_from_slice(&(0x4000u64).to_le_bytes());
    fixture.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    fixture.extend_from_slice(&(0u64).to_le_bytes());
    fixture.extend_from_slice(&0u64.to_le_bytes());
    fixture.extend_from_slice(&[0u8; 16]);
    fixture.extend_from_slice(&[0u8; 16]);
    fixture.extend_from_slice(&compressed);

    let root = std::env::temp_dir().join(format!("nsz-rs-native-verify-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let input = root.join(format!("{stem}.ncz"));
    fs::write(&input, fixture).unwrap();

    let report = nsz_rs::verify(&nsz_rs::VerifyRequest {
        files: vec![input.clone()],
        fix_padding: false,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    })
    .unwrap();

    assert_eq!(report.verified_files, vec![input]);

    let _ = fs::remove_dir_all(root);
}
