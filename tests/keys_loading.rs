#[test]
fn key_loader_checks_required_entries() {
    let err = nsz_rs::crypto::keys::load_from_str("master_key_00 = 00").unwrap_err();
    assert!(format!("{err}").contains("aes_kek_generation_source"));
}
