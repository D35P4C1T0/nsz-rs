#[test]
fn ncz_decompressed_size_matches_header_sections() {
    let mut fixture = vec![0u8; 0x4000];
    fixture.extend_from_slice(b"NCZSECTN");
    fixture.extend_from_slice(&(1u64).to_le_bytes());
    fixture.extend_from_slice(&(0x4000u64).to_le_bytes());
    fixture.extend_from_slice(&(0x1200u64).to_le_bytes());
    fixture.extend_from_slice(&(3u64).to_le_bytes());
    fixture.extend_from_slice(&0u64.to_le_bytes());
    fixture.extend_from_slice(&[0u8; 16]);
    fixture.extend_from_slice(&[0u8; 16]);

    let size = nsz_rs::ncz::decompress::decompressed_nca_size_from_bytes(&fixture).unwrap();
    assert_eq!(size, 0x4000 + 0x1200);
}
