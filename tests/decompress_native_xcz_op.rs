use std::fs;
use std::path::PathBuf;

#[test]
fn decompress_uses_native_path_for_xcz_inputs() {
    let payload = b"native-op-xcz-payload";
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

    let secure_hfs0 = build_hfs0(&[
        (
            "0123456789abcdef0123456789abcdef.ncz".to_string(),
            ncz.clone(),
        ),
        ("dummy.tik".to_string(), b"tik".to_vec()),
    ]);
    let root_hfs0 = build_hfs0(&[("secure".to_string(), secure_hfs0)]);
    let xcz = build_xci_like(root_hfs0);

    let root = std::env::temp_dir().join(format!("nsz-rs-native-op-xcz-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let input = root.join("fixture.xcz");
    let out_dir = root.join("out");
    fs::write(&input, xcz).unwrap();
    fs::create_dir_all(&out_dir).unwrap();

    let report = nsz_rs::decompress(&nsz_rs::DecompressRequest {
        files: vec![input],
        output_dir: Some(out_dir.clone()),
        fix_padding: false,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    })
    .unwrap();

    let out_xci = out_dir.join("fixture.xci");
    assert_eq!(report.processed_files, vec![out_xci.clone()]);
    let out_bytes = fs::read(&out_xci).unwrap();

    let xci = nsz_rs::container::xci::XciArchive::from_bytes(&out_bytes).unwrap();
    let root_bytes = xci.root_hfs0_bytes(&out_bytes).unwrap();
    let root_hfs0 = xci.root_hfs0_archive(&out_bytes).unwrap();
    let secure_entry = root_hfs0
        .entries()
        .iter()
        .find(|entry| entry.name == "secure")
        .unwrap();
    let secure_bytes = root_hfs0.entry_bytes(root_bytes, secure_entry);
    let secure_hfs0 = nsz_rs::container::hfs0::Hfs0Archive::from_bytes(secure_bytes).unwrap();

    let nca_entry = secure_hfs0
        .entries()
        .iter()
        .find(|entry| entry.name.ends_with(".nca"))
        .unwrap();
    let nca_bytes = secure_hfs0.entry_bytes(secure_bytes, nca_entry);
    assert_eq!(&nca_bytes[0x4000..], payload);

    let _ = fs::remove_dir_all(root);
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
