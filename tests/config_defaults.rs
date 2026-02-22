#[test]
fn compress_defaults_match_python_461() {
    let req = nsz_rs::config::CompressRequest::default();
    assert_eq!(req.level, 18);
    assert_eq!(req.block_size_exponent, 20);
    assert_eq!(req.multi, 4);
    assert_eq!(req.threads, -1);
}
