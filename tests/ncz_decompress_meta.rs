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

#[test]
fn ncz_native_decompress_roundtrip_no_crypto() {
    let payload = b"native-ncz-payload";
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

    let decompressed = nsz_rs::ncz::decompress::decompress_ncz_to_vec(&fixture).unwrap();
    assert_eq!(decompressed.len(), 0x4000 + payload.len());
    assert_eq!(&decompressed[0x4000..], payload);
}

#[test]
fn ncz_native_decompress_roundtrip_block_stream() {
    let payload = b"native-block-stream-payload";
    let compressed_block = zstd::stream::encode_all(&payload[..], 1).unwrap();

    let mut fixture = vec![0u8; 0x4000];
    fixture.extend_from_slice(b"NCZSECTN");
    fixture.extend_from_slice(&(1u64).to_le_bytes());
    fixture.extend_from_slice(&(0x4000u64).to_le_bytes());
    fixture.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    fixture.extend_from_slice(&(0u64).to_le_bytes());
    fixture.extend_from_slice(&0u64.to_le_bytes());
    fixture.extend_from_slice(&[0u8; 16]);
    fixture.extend_from_slice(&[0u8; 16]);

    fixture.extend_from_slice(b"NCZBLOCK");
    fixture.push(2);
    fixture.push(1);
    fixture.push(0);
    fixture.push(20);
    fixture.extend_from_slice(&(1u32).to_le_bytes());
    fixture.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    fixture.extend_from_slice(&u32::try_from(compressed_block.len()).unwrap().to_le_bytes());
    fixture.extend_from_slice(&compressed_block);

    let decompressed = nsz_rs::ncz::decompress::decompress_ncz_to_vec(&fixture).unwrap();
    assert_eq!(&decompressed[0x4000..], payload);
}

#[test]
fn ncz_native_decompress_unknown_crypto_type_is_passthrough() {
    let payload = b"native-unknown-crypto-payload";
    let compressed = zstd::stream::encode_all(&payload[..], 1).unwrap();

    let mut fixture = vec![0u8; 0x4000];
    fixture.extend_from_slice(b"NCZSECTN");
    fixture.extend_from_slice(&(1u64).to_le_bytes());
    fixture.extend_from_slice(&(0x4000u64).to_le_bytes());
    fixture.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    fixture.extend_from_slice(&(2u64).to_le_bytes());
    fixture.extend_from_slice(&0u64.to_le_bytes());
    fixture.extend_from_slice(&[0u8; 16]);
    fixture.extend_from_slice(&[0u8; 16]);
    fixture.extend_from_slice(&compressed);

    let decompressed = nsz_rs::ncz::decompress::decompress_ncz_to_vec(&fixture).unwrap();
    assert_eq!(&decompressed[0x4000..], payload);
}

#[test]
fn ncz_native_decompress_preserves_leading_gap_before_first_section() {
    let leading_gap = vec![0xAB; 0x200];
    let payload = b"native-gap-section-payload";
    let mut stream = leading_gap.clone();
    stream.extend_from_slice(payload);
    let compressed = zstd::stream::encode_all(&stream[..], 1).unwrap();

    let mut fixture = vec![0u8; 0x4000];
    fixture.extend_from_slice(b"NCZSECTN");
    fixture.extend_from_slice(&(1u64).to_le_bytes());
    fixture.extend_from_slice(&(0x4200u64).to_le_bytes());
    fixture.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    fixture.extend_from_slice(&(0u64).to_le_bytes());
    fixture.extend_from_slice(&0u64.to_le_bytes());
    fixture.extend_from_slice(&[0u8; 16]);
    fixture.extend_from_slice(&[0u8; 16]);
    fixture.extend_from_slice(&compressed);

    let decompressed = nsz_rs::ncz::decompress::decompress_ncz_to_vec(&fixture).unwrap();
    assert_eq!(&decompressed[0x4000..0x4200], &leading_gap);
    assert_eq!(&decompressed[0x4200..], payload);
}
