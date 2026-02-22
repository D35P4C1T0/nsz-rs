#[test]
fn ncz_block_header_binary_layout_matches_python() {
    let fixture: Vec<u8> = vec![
        b'N', b'C', b'Z', b'B', b'L', b'O', b'C', b'K', 2, 1, 0, 20, 2, 0, 0, 0, 0x00, 0x20, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00,
    ];

    let parsed = nsz_rs::ncz::header::BlockHeader::from_bytes(&fixture).unwrap();
    let encoded = parsed.to_bytes();

    assert_eq!(encoded, fixture);
}
