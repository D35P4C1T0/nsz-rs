use std::borrow::Cow;

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

#[test]
fn pfs0_hfs0_encode_accepts_borrowed_payloads() {
    let payload_a = b"alpha".to_vec();
    let payload_b = b"beta".to_vec();
    let pfs_entries = vec![
        ("a.bin".to_string(), Cow::Borrowed(payload_a.as_slice())),
        ("b.bin".to_string(), Cow::Borrowed(payload_b.as_slice())),
    ];
    let pfs = nsz_rs::container::nsp::encode_pfs0(&pfs_entries, 0x80, 0).unwrap();
    let pfs_archive = nsz_rs::container::nsp::NspArchive::from_bytes(&pfs).unwrap();
    assert_eq!(pfs_archive.entries().len(), 2);
    assert_eq!(
        pfs_archive.entry_bytes(&pfs, &pfs_archive.entries()[0]),
        payload_a
    );
    assert_eq!(
        pfs_archive.entry_bytes(&pfs, &pfs_archive.entries()[1]),
        payload_b
    );

    let hfs_entries = vec![
        ("c.bin".to_string(), Cow::Borrowed(payload_a.as_slice())),
        ("d.bin".to_string(), Cow::Borrowed(payload_b.as_slice())),
    ];
    let hfs = nsz_rs::container::hfs0::encode_hfs0(&hfs_entries, 0x100, 0).unwrap();
    let hfs_archive = nsz_rs::container::hfs0::Hfs0Archive::from_bytes(&hfs).unwrap();
    assert_eq!(hfs_archive.entries().len(), 2);
    assert_eq!(
        hfs_archive.entry_bytes(&hfs, &hfs_archive.entries()[0]),
        payload_a
    );
    assert_eq!(
        hfs_archive.entry_bytes(&hfs, &hfs_archive.entries()[1]),
        payload_b
    );
}
