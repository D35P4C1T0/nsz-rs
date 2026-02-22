#[test]
fn public_api_symbols_exist() {
    let _ = nsz_rs::compress;
    let _ = nsz_rs::decompress;
    let _ = nsz_rs::verify;
    let _ = nsz_rs::extract;
    let _ = nsz_rs::create;
    let _ = nsz_rs::titlekeys;
    let _ = nsz_rs::undupe;
}
