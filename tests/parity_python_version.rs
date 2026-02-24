#[test]
fn python_baseline_reports_461() {
    let repo_root = std::env::var("NSZ_PYTHON_REPO")
        .unwrap_or_else(|_| "/home/matteo/Documents/prog/python/nsz".to_string());
    let setup_py = std::path::Path::new(&repo_root).join("setup.py");
    if !setup_py.exists() {
        eprintln!(
            "skipping parity_python_version: missing {}",
            setup_py.display()
        );
        return;
    }

    let version = nsz_rs::parity::python_runner::query_version(&repo_root)
        .unwrap_or_else(|err| panic!("failed to query Python baseline at {repo_root}: {err}"));
    assert_eq!(version, "4.6.1");
}
