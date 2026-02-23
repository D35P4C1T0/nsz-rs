use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};

#[test]
fn ncz_native_decompress_roundtrip_crypto_type3() {
    type AesCtr = ctr::Ctr128BE<Aes128>;

    let payload = b"native-ncz-crypto-payload".to_vec();
    let key = [0x11u8; 16];
    let counter = [0x22u8; 16];

    let mut encrypted = payload.clone();
    let mut cipher = AesCtr::new(&key.into(), &counter.into());
    cipher.seek(0x4000u128);
    cipher.apply_keystream(&mut encrypted);

    let compressed = zstd::stream::encode_all(&encrypted[..], 1).unwrap();

    let mut fixture = vec![0u8; 0x4000];
    fixture.extend_from_slice(b"NCZSECTN");
    fixture.extend_from_slice(&(1u64).to_le_bytes());
    fixture.extend_from_slice(&(0x4000u64).to_le_bytes());
    fixture.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    fixture.extend_from_slice(&(3u64).to_le_bytes());
    fixture.extend_from_slice(&0u64.to_le_bytes());
    fixture.extend_from_slice(&key);
    fixture.extend_from_slice(&counter);
    fixture.extend_from_slice(&compressed);

    let decompressed = nsz_rs::ncz::decompress::decompress_ncz_to_vec(&fixture).unwrap();
    assert_eq!(&decompressed[0x4000..], &payload);
}
