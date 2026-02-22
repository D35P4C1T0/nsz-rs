#[test]
fn pfs0_header_roundtrip_is_stable() {
    let fixture: Vec<u8> = vec![
        b'P', b'F', b'S', b'0', 1, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0x20, 0, 0, 0, 0, 0, 0, 0,
        0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, b'f', b'i', b'l', b'e', 0, 0, 0, 0,
    ];

    let header = nsz_rs::container::pfs0::Pfs0Header::from_bytes(&fixture).unwrap();
    let encoded = header.to_bytes();

    assert_eq!(encoded, fixture);
}
