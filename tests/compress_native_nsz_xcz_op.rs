use std::fs;
use std::path::PathBuf;

#[test]
fn compress_uses_native_path_for_nsp_inputs() {
    let nca_name = "0123456789abcdef0123456789abcdef.nca";
    let nca_payload = build_nca_payload(b"native-compress-nsp");
    let nsp_bytes = build_pfs0(&[(nca_name, nca_payload.as_slice()), ("note.txt", b"hi")]);

    let root =
        std::env::temp_dir().join(format!("nsz-rs-native-compress-nsp-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let input = root.join("fixture.nsp");
    let out_dir = root.join("out");
    fs::write(&input, nsp_bytes).unwrap();
    fs::create_dir_all(&out_dir).unwrap();

    let report = nsz_rs::compress(&nsz_rs::CompressRequest {
        files: vec![input],
        output_dir: Some(out_dir.clone()),
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
        ..Default::default()
    })
    .unwrap();

    let out_nsz = out_dir.join("fixture.nsz");
    assert_eq!(report.processed_files, vec![out_nsz.clone()]);

    let out_archive =
        nsz_rs::container::nsp::NspArchive::from_bytes(&fs::read(&out_nsz).unwrap()).unwrap();
    let ncz_entry = out_archive
        .entries()
        .iter()
        .find(|entry| entry.name == "0123456789abcdef0123456789abcdef.ncz")
        .unwrap();
    let out_bytes = fs::read(&out_nsz).unwrap();
    let ncz_bytes = out_archive.entry_bytes(&out_bytes, ncz_entry);
    let roundtrip = nsz_rs::ncz::decompress::decompress_ncz_to_vec(ncz_bytes).unwrap();
    assert_eq!(roundtrip, nca_payload);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn compress_uses_native_path_for_xci_inputs() {
    let nca_name = "fedcba9876543210fedcba9876543210.nca".to_string();
    let nca_payload = build_nca_payload(b"native-compress-xci");
    let secure_hfs0 = build_hfs0(&[(nca_name, nca_payload.clone())]);
    let root_hfs0 = build_hfs0(&[("secure".to_string(), secure_hfs0)]);
    let xci_bytes = build_xci_like(root_hfs0);

    let root =
        std::env::temp_dir().join(format!("nsz-rs-native-compress-xci-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let input = root.join("fixture.xci");
    let out_dir = root.join("out");
    fs::write(&input, xci_bytes).unwrap();
    fs::create_dir_all(&out_dir).unwrap();

    let report = nsz_rs::compress(&nsz_rs::CompressRequest {
        files: vec![input],
        output_dir: Some(out_dir.clone()),
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
        ..Default::default()
    })
    .unwrap();

    let out_xcz = out_dir.join("fixture.xcz");
    assert_eq!(report.processed_files, vec![out_xcz.clone()]);

    let out_xcz_bytes = fs::read(&out_xcz).unwrap();
    let xci = nsz_rs::container::xci::XciArchive::from_bytes(&out_xcz_bytes).unwrap();
    let root_bytes = xci.root_hfs0_bytes(&out_xcz_bytes).unwrap();
    let root_hfs0 = xci.root_hfs0_archive(&out_xcz_bytes).unwrap();
    let secure_entry = root_hfs0
        .entries()
        .iter()
        .find(|entry| entry.name == "secure")
        .unwrap();
    let secure_bytes = root_hfs0.entry_bytes(root_bytes, secure_entry);
    let secure_hfs0 = nsz_rs::container::hfs0::Hfs0Archive::from_bytes(secure_bytes).unwrap();
    let ncz_entry = secure_hfs0
        .entries()
        .iter()
        .find(|entry| entry.name.ends_with(".ncz"))
        .unwrap();
    let roundtrip = nsz_rs::ncz::decompress::decompress_ncz_to_vec(
        secure_hfs0.entry_bytes(secure_bytes, ncz_entry),
    )
    .unwrap();
    assert_eq!(roundtrip, nca_payload);

    let _ = fs::remove_dir_all(root);
}

fn build_nca_payload(payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 0x4000];
    out.extend_from_slice(payload);
    out
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

fn build_hfs0(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut string_table = Vec::new();
    let mut string_offsets = Vec::with_capacity(entries.len());
    for (name, _) in entries {
        string_offsets.push(string_table.len() as u32);
        string_table.extend_from_slice(name.as_bytes());
        string_table.push(0);
    }

    let header_size = 16 + entries.len() * 0x40 + string_table.len();
    let mut out = Vec::new();
    out.extend_from_slice(b"HFS0");
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    out.extend_from_slice(&(string_table.len() as u32).to_le_bytes());
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

fn build_xci_like(root_hfs0: Vec<u8>) -> Vec<u8> {
    let hfs0_offset = 0xF000u64;
    let mut out = vec![0u8; 0x200];
    out[0x100..0x104].copy_from_slice(b"HEAD");
    out[0x130..0x138].copy_from_slice(&hfs0_offset.to_le_bytes());
    out[0x138..0x140].copy_from_slice(&(root_hfs0.len() as u64).to_le_bytes());
    out.resize(hfs0_offset as usize, 0);
    out.extend_from_slice(&root_hfs0);
    out
}
