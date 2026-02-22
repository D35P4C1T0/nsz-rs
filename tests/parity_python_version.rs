#[test]
fn python_baseline_reports_461() {
    let version =
        nsz_rs::parity::python_runner::query_version("/home/matteo/Documents/prog/python/nsz")
            .unwrap();
    assert_eq!(version, "4.6.1");
}
