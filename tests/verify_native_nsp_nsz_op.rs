use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[test]
fn verify_uses_native_path_for_nsp_inputs() {
    let payload = b"native-verify-nsp-payload";
    let mut nca = vec![0u8; 0x4000];
    nca.extend_from_slice(payload);
    let hash = format!("{:x}", Sha256::digest(&nca));

    let nca_name = format!("{hash}.nca");
    let nsp_bytes = build_pfs0(&[(nca_name.as_str(), &nca), ("dummy.tik", b"tik")]);

    let root =
        std::env::temp_dir().join(format!("nsz-rs-native-verify-nsp-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let input = root.join("fixture.nsp");
    fs::write(&input, nsp_bytes).unwrap();

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
fn verify_uses_native_path_for_nsz_inputs() {
    let payload = b"native-verify-nsz-payload";
    let mut nca = vec![0u8; 0x4000];
    nca.extend_from_slice(payload);
    let hash = format!("{:x}", Sha256::digest(&nca));

    let compressed = zstd::stream::encode_all(&payload[..], 1).unwrap();
    let mut ncz = vec![0u8; 0x4000];
    ncz.extend_from_slice(b"NCZSECTN");
    ncz.extend_from_slice(&(1u64).to_le_bytes());
    ncz.extend_from_slice(&(0x4000u64).to_le_bytes());
    ncz.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    ncz.extend_from_slice(&(0u64).to_le_bytes());
    ncz.extend_from_slice(&0u64.to_le_bytes());
    ncz.extend_from_slice(&[0u8; 16]);
    ncz.extend_from_slice(&[0u8; 16]);
    ncz.extend_from_slice(&compressed);

    let ncz_name = format!("{hash}.ncz");
    let nsz_bytes = build_pfs0(&[(ncz_name.as_str(), &ncz), ("dummy.tik", b"tik")]);

    let root =
        std::env::temp_dir().join(format!("nsz-rs-native-verify-nsz-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let input = root.join("fixture.nsz");
    fs::write(&input, nsz_bytes).unwrap();

    let report = nsz_rs::verify(&nsz_rs::VerifyRequest {
        files: vec![input.clone()],
        fix_padding: false,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    })
    .unwrap();

    assert_eq!(report.verified_files, vec![input]);
    let _ = fs::remove_dir_all(root);
}

fn build_pfs0(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut string_table = Vec::new();
    let mut string_offsets = Vec::with_capacity(entries.len());
    for (name, _) in entries {
        string_offsets.push(string_table.len() as u32);
        string_table.extend_from_slice(name.as_bytes());
        string_table.push(0);
    }

    let header_size = 16 + entries.len() * 24 + string_table.len();
    let mut out = Vec::new();
    out.extend_from_slice(b"PFS0");
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    out.extend_from_slice(&(string_table.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());

    let mut offset = 0u64;
    for ((_, data), string_offset) in entries.iter().zip(string_offsets.iter()) {
        out.extend_from_slice(&offset.to_le_bytes());
        out.extend_from_slice(&(data.len() as u64).to_le_bytes());
        out.extend_from_slice(&string_offset.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        offset += data.len() as u64;
    }

    out.extend_from_slice(&string_table);
    out.resize(header_size, 0);
    for (_, data) in entries {
        out.extend_from_slice(data);
    }

    out
}
