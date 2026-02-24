use std::fs;
use std::path::PathBuf;

#[test]
fn decompress_uses_native_path_for_nsz_inputs() {
    let payload = b"native-op-nsz-payload";
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

    let ncz_name = "0123456789abcdef0123456789abcdef.ncz";
    let txt_name = "note.txt";
    let txt_bytes = b"hello";
    let nsz_bytes = build_pfs0(&[(ncz_name, &ncz), (txt_name, txt_bytes)]);

    let root = std::env::temp_dir().join(format!("nsz-rs-native-op-nsz-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let input = root.join("fixture.nsz");
    let out_dir = root.join("out");
    fs::write(&input, nsz_bytes).unwrap();
    fs::create_dir_all(&out_dir).unwrap();

    let report = nsz_rs::decompress(&nsz_rs::DecompressRequest {
        files: vec![input],
        output_dir: Some(out_dir.clone()),
        fix_padding: false,
        python_repo_root: Some(PathBuf::from("/does/not/exist")),
    })
    .unwrap();

    let out_nsp = out_dir.join("fixture.nsp");
    assert_eq!(report.processed_files, vec![out_nsp.clone()]);
    assert!(out_nsp.exists());

    let out_entries = parse_pfs0_entries(&fs::read(&out_nsp).unwrap());
    let out_nca_name = "0123456789abcdef0123456789abcdef.nca";
    let out_nca = out_entries.get(out_nca_name).unwrap();
    assert_eq!(&out_nca[0x4000..], payload);
    assert_eq!(out_entries.get(txt_name).unwrap(), txt_bytes);

    let _ = fs::remove_dir_all(root);
}

fn build_pfs0(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut string_table = Vec::new();
    let mut string_offsets = Vec::with_capacity(entries.len());
    for (name, _) in entries {
        string_offsets.push(u32::try_from(string_table.len()).unwrap());
        string_table.extend_from_slice(name.as_bytes());
        string_table.push(0);
    }

    let header_size = 16 + entries.len() * 24 + string_table.len();
    let mut out = Vec::new();
    out.extend_from_slice(b"PFS0");
    out.extend_from_slice(&u32::try_from(entries.len()).unwrap().to_le_bytes());
    out.extend_from_slice(&u32::try_from(string_table.len()).unwrap().to_le_bytes());
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

fn parse_pfs0_entries(data: &[u8]) -> std::collections::BTreeMap<String, Vec<u8>> {
    assert_eq!(&data[..4], b"PFS0");
    let file_count = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
    let string_table_size = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;

    let mut entries = Vec::with_capacity(file_count);
    let mut cursor = 16usize;
    for _ in 0..file_count {
        let offset = usize::try_from(u64::from_le_bytes(
            data[cursor..cursor + 8].try_into().unwrap(),
        ))
        .unwrap();
        let size = usize::try_from(u64::from_le_bytes(
            data[cursor + 8..cursor + 16].try_into().unwrap(),
        ))
        .unwrap();
        let string_offset =
            u32::from_le_bytes(data[cursor + 16..cursor + 20].try_into().unwrap()) as usize;
        entries.push((offset, size, string_offset));
        cursor += 24;
    }

    let string_table = &data[cursor..cursor + string_table_size];
    let max_end = entries
        .iter()
        .map(|(offset, size, _)| offset + size)
        .max()
        .unwrap_or(0);
    let data_start = data.len() - max_end;

    let mut out = std::collections::BTreeMap::new();
    for (offset, size, string_offset) in entries {
        let end = string_table[string_offset..]
            .iter()
            .position(|b| *b == 0)
            .unwrap()
            + string_offset;
        let name = std::str::from_utf8(&string_table[string_offset..end])
            .unwrap()
            .to_string();
        let start = data_start + offset;
        out.insert(name, data[start..start + size].to_vec());
    }

    out
}
