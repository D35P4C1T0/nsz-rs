use std::fs;
use std::path::PathBuf;

#[test]
fn decompress_uses_native_path_for_ncz_inputs() {
    let payload = b"native-op-ncz-payload";
    let compressed = zstd::stream::encode_all(&payload[..], 1).unwrap();

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

    let root = std::env::temp_dir().join(format!("nsz-rs-native-op-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let input = root.join("fixture.ncz");
    let out_dir = root.join("out");
    fs::write(&input, fixture).unwrap();
    fs::create_dir_all(&out_dir).unwrap();

    let report = nsz_rs::decompress(&nsz_rs::DecompressRequest {
        files: vec![input.clone()],
        output_dir: Some(out_dir.clone()),
        fix_padding: false,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    })
    .unwrap();

    let out_nca = out_dir.join("fixture.nca");
    assert!(out_nca.exists());
    let bytes = fs::read(&out_nca).unwrap();
    assert_eq!(&bytes[0x4000..], payload);
    assert_eq!(report.processed_files, vec![out_nca]);

    let _ = fs::remove_dir_all(root);
}
