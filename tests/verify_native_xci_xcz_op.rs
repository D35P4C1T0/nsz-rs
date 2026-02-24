use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[test]
fn verify_uses_native_path_for_xci_inputs() {
    let payload = b"native-verify-xci-payload";
    let mut nca = vec![0u8; 0x4000];
    nca.extend_from_slice(payload);
    let hash = format!("{:x}", Sha256::digest(&nca));

    let secure_hfs0 = build_hfs0(&[
        (format!("{hash}.nca"), nca),
        ("dummy.tik".to_string(), b"tik".to_vec()),
    ]);
    let root_hfs0 = build_hfs0(&[("secure".to_string(), secure_hfs0)]);
    let xci = build_xci_like(&root_hfs0);

    let root =
        std::env::temp_dir().join(format!("nsz-rs-native-verify-xci-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let input = root.join("fixture.xci");
    fs::write(&input, xci).unwrap();

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
fn verify_uses_native_path_for_xcz_inputs() {
    let payload = b"native-verify-xcz-payload";
    let mut nca = vec![0u8; 0x4000];
    nca.extend_from_slice(payload);
    let hash = format!("{:x}", Sha256::digest(&nca));

    let compressed = zstd::stream::encode_all(&payload[..], 1).unwrap();
    let mut ncz_blob = vec![0u8; 0x4000];
    ncz_blob.extend_from_slice(b"NCZSECTN");
    ncz_blob.extend_from_slice(&(1u64).to_le_bytes());
    ncz_blob.extend_from_slice(&(0x4000u64).to_le_bytes());
    ncz_blob.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    ncz_blob.extend_from_slice(&(0u64).to_le_bytes());
    ncz_blob.extend_from_slice(&0u64.to_le_bytes());
    ncz_blob.extend_from_slice(&[0u8; 16]);
    ncz_blob.extend_from_slice(&[0u8; 16]);
    ncz_blob.extend_from_slice(&compressed);

    let secure_hfs0 = build_hfs0(&[
        (format!("{hash}.ncz"), ncz_blob),
        ("dummy.tik".to_string(), b"tik".to_vec()),
    ]);
    let root_hfs0 = build_hfs0(&[("secure".to_string(), secure_hfs0)]);
    let xcz = build_xci_like(&root_hfs0);

    let root =
        std::env::temp_dir().join(format!("nsz-rs-native-verify-xcz-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let input = root.join("fixture.xcz");
    fs::write(&input, xcz).unwrap();

    let report = nsz_rs::verify(&nsz_rs::VerifyRequest {
        files: vec![input.clone()],
        fix_padding: false,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    })
    .unwrap();

    assert_eq!(report.verified_files, vec![input]);
    let _ = fs::remove_dir_all(root);
}

fn build_hfs0(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut string_table = Vec::new();
    let mut string_offsets = Vec::with_capacity(entries.len());
    for (name, _) in entries {
        string_offsets.push(u32::try_from(string_table.len()).unwrap());
        string_table.extend_from_slice(name.as_bytes());
        string_table.push(0);
    }

    let header_size = 16 + entries.len() * 0x40 + string_table.len();
    let mut out = Vec::new();
    out.extend_from_slice(b"HFS0");
    out.extend_from_slice(&u32::try_from(entries.len()).unwrap().to_le_bytes());
    out.extend_from_slice(&u32::try_from(string_table.len()).unwrap().to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());

    let mut offset = 0u64;
    for ((_, data), string_offset) in entries.iter().zip(string_offsets.iter()) {
        out.extend_from_slice(&offset.to_le_bytes());
        out.extend_from_slice(&(data.len() as u64).to_le_bytes());
        out.extend_from_slice(&string_offset.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes());
        out.extend_from_slice(&[0u8; 32]);
        offset += data.len() as u64;
    }

    out.extend_from_slice(&string_table);
    out.resize(header_size, 0);
    for (_, data) in entries {
        out.extend_from_slice(data);
    }
    out
}

fn build_xci_like(root_hfs0: &[u8]) -> Vec<u8> {
    let hfs0_offset = 0xF000u64;
    let mut out = vec![0u8; 0x200];
    out[0x100..0x104].copy_from_slice(b"HEAD");
    out[0x130..0x138].copy_from_slice(&hfs0_offset.to_le_bytes());
    out[0x138..0x140].copy_from_slice(&(root_hfs0.len() as u64).to_le_bytes());
    out.resize(usize::try_from(hfs0_offset).unwrap(), 0);
    out.extend_from_slice(root_hfs0);
    out
}
