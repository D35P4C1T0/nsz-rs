#[test]
fn parity_mismatch_error_carries_offsets() {
    let err = nsz_rs::error::NszError::ParityMismatch {
        operation: "compress".into(),
        expected_sha256: "a".repeat(64),
        actual_sha256: "b".repeat(64),
        first_diff_offset: 42,
    };
    let msg = err.to_string();
    assert!(msg.contains("42"));
}
