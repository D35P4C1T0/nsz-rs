# NSZ Rust Parity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a safe Rust library that reproduces Python `nsz` `4.6.1` output bytes exactly for equivalent operations and options.

**Architecture:** Implement a high-level facade API in `src/lib.rs` with private modules for container I/O, NCZ transforms, crypto/key handling, compression pipelines, verification, fs policies, and parity harnessing. Follow behavior-clone sequencing first, then optimize only behind parity gates.

**Tech Stack:** Rust stable, Cargo, `zstd`, `sha2`, `aes`, `ctr`, `rayon`, `thiserror`, `serde`, `serde_json`, `regex`, `walkdir`, `tempfile`, `insta`/snapshot or binary diff tooling.

---

### Task 1: Workspace Bootstrap and Crate Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/error.rs`
- Create: `src/config.rs`
- Create: `src/ops/mod.rs`
- Create: `tests/smoke_public_api.rs`

**Step 1: Write the failing test**

```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test public_api_symbols_exist -q`
Expected: FAIL because crate and symbols are missing.

**Step 3: Write minimal implementation**

```rust
pub fn compress(_: &config::CompressRequest) -> Result<ops::OperationReport, error::NszError> { todo!() }
```

Add equivalent function signatures for all facade operations.

**Step 4: Run test to verify it passes**

Run: `cargo test public_api_symbols_exist -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml src tests
git commit -m "chore: bootstrap nsz-rs library facade"
```

### Task 2: Baseline Runner for Python 4.6.1

**Files:**
- Create: `src/parity/python_runner.rs`
- Create: `src/parity/mod.rs`
- Create: `tests/parity_python_version.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn python_baseline_reports_461() {
    let version = nsz_rs::parity::python_runner::query_version("/home/matteo/Documents/prog/python/nsz").unwrap();
    assert_eq!(version, "4.6.1");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test python_baseline_reports_461 -q`
Expected: FAIL because runner module is missing.

**Step 3: Write minimal implementation**

Implement a safe process wrapper that executes `python3 nsz.py --help` and parses the version banner.

**Step 4: Run test to verify it passes**

Run: `cargo test python_baseline_reports_461 -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/parity tests/parity_python_version.rs
git commit -m "test: add python baseline runner for parity harness"
```

### Task 3: Deterministic Request Config Mapping

**Files:**
- Modify: `src/config.rs`
- Create: `tests/config_defaults.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn compress_defaults_match_python_461() {
    let req = nsz_rs::config::CompressRequest::default();
    assert_eq!(req.level, 18);
    assert_eq!(req.block_size_exponent, 20);
    assert_eq!(req.multi, 4);
    assert_eq!(req.threads, -1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test compress_defaults_match_python_461 -q`
Expected: FAIL because defaults are unset.

**Step 3: Write minimal implementation**

Define all request structs with explicit defaults matching `ParseArguments.py`.

**Step 4: Run test to verify it passes**

Run: `cargo test compress_defaults_match_python_461 -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/config.rs tests/config_defaults.rs
git commit -m "feat: add python-matching operation config defaults"
```

### Task 4: Core Error Taxonomy

**Files:**
- Modify: `src/error.rs`
- Create: `tests/error_contract.rs`

**Step 1: Write the failing test**

```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test parity_mismatch_error_carries_offsets -q`
Expected: FAIL because variant is missing.

**Step 3: Write minimal implementation**

Define `NszError` using `thiserror` with explicit variants for key/config/container/verify/parity/fs policy failures.

**Step 4: Run test to verify it passes**

Run: `cargo test parity_mismatch_error_carries_offsets -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/error.rs tests/error_contract.rs
git commit -m "feat: define typed nsz error contract"
```

### Task 5: File Discovery and Policy Engine (PathTools + ExistingChecks clone)

**Files:**
- Create: `src/fs_ops/path_tools.rs`
- Create: `src/fs_ops/existing_checks.rs`
- Create: `src/fs_ops/mod.rs`
- Create: `tests/fs_policy.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn file_policy_rejects_duplicate_without_overwrite() {
    // arrange temp output with existing target
    // expect policy returns DenyDuplicate
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test file_policy_rejects_duplicate_without_overwrite -q`
Expected: FAIL because fs policy module is missing.

**Step 3: Write minimal implementation**

Implement path expansion and output-write allowance rules cloned from Python behavior.

**Step 4: Run test to verify it passes**

Run: `cargo test file_policy_rejects_duplicate_without_overwrite -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/fs_ops tests/fs_policy.rs
git commit -m "feat: implement path expansion and output policy rules"
```

### Task 6: Key Loading and Validation

**Files:**
- Create: `src/crypto/keys.rs`
- Create: `src/crypto/mod.rs`
- Create: `tests/keys_loading.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn key_loader_checks_required_entries() {
    let err = nsz_rs::crypto::keys::load_from_str("master_key_00 = 00").unwrap_err();
    assert!(format!("{err}").contains("aes_kek_generation_source"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test key_loader_checks_required_entries -q`
Expected: FAIL because key loader is missing.

**Step 3: Write minimal implementation**

Implement parser + required-key validation + checksum tracking semantics compatible with Python `Keys.py` behavior.

**Step 4: Run test to verify it passes**

Run: `cargo test key_loader_checks_required_entries -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/crypto tests/keys_loading.rs
git commit -m "feat: add key loading and validation semantics"
```

### Task 7: Container Abstractions (NSP/XCI/PFS0/HFS0 Minimal Read/Write)

**Files:**
- Create: `src/container/mod.rs`
- Create: `src/container/pfs0.rs`
- Create: `src/container/hfs0.rs`
- Create: `src/container/nsp.rs`
- Create: `src/container/xci.rs`
- Create: `tests/container_roundtrip.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn pfs0_header_roundtrip_is_stable() {
    // parse header bytes, serialize again, assert exact bytes match
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test pfs0_header_roundtrip_is_stable -q`
Expected: FAIL because container modules are missing.

**Step 3: Write minimal implementation**

Implement parsing/serialization primitives required for deterministic header reproduction.

**Step 4: Run test to verify it passes**

Run: `cargo test pfs0_header_roundtrip_is_stable -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/container tests/container_roundtrip.rs
git commit -m "feat: add deterministic container primitives"
```

### Task 8: NCZ Header and Block Stream Support

**Files:**
- Create: `src/ncz/mod.rs`
- Create: `src/ncz/header.rs`
- Create: `src/ncz/block_reader.rs`
- Create: `tests/ncz_header_block.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn ncz_block_header_binary_layout_matches_python() {
    // assert packed bytes exactly match baseline fixture
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test ncz_block_header_binary_layout_matches_python -q`
Expected: FAIL because NCZ module is missing.

**Step 3: Write minimal implementation**

Implement NCZ section/block structures and binary read/write logic.

**Step 4: Run test to verify it passes**

Run: `cargo test ncz_block_header_binary_layout_matches_python -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/ncz tests/ncz_header_block.rs
git commit -m "feat: implement ncz header and block layout support"
```

### Task 9: Decompress + Verify Path

**Files:**
- Create: `src/ops/decompress.rs`
- Create: `src/ops/verify.rs`
- Modify: `src/ops/mod.rs`
- Create: `tests/decompress_verify_parity.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn decompress_verify_matches_python_for_fixture() {
    // run python baseline and rust for one corpus file and compare outputs
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test decompress_verify_matches_python_for_fixture -q`
Expected: FAIL because ops are not implemented.

**Step 3: Write minimal implementation**

Implement `decompress` and `verify` behavior clone and wire to facade.

**Step 4: Run test to verify it passes**

Run: `cargo test decompress_verify_matches_python_for_fixture -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/ops tests/decompress_verify_parity.rs
git commit -m "feat: implement decompress and verify parity path"
```

### Task 10: Solid and Block Compression Path

**Files:**
- Create: `src/compress/mod.rs`
- Create: `src/compress/solid.rs`
- Create: `src/compress/block.rs`
- Create: `src/ops/compress.rs`
- Modify: `src/ops/mod.rs`
- Create: `tests/compress_parity.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn compress_output_is_byte_identical_to_python() {
    // run matched options and compare produced .nsz/.xcz bytes
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test compress_output_is_byte_identical_to_python -q`
Expected: FAIL because compression ops are missing.

**Step 3: Write minimal implementation**

Implement solid and block compression with Python-equivalent sequencing and zstd parameters.

**Step 4: Run test to verify it passes**

Run: `cargo test compress_output_is_byte_identical_to_python -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/compress src/ops/compress.rs tests/compress_parity.rs
git commit -m "feat: implement solid and block compression parity path"
```

### Task 11: Extract, Create, Titlekeys, Undupe Operations

**Files:**
- Create: `src/ops/extract.rs`
- Create: `src/ops/create.rs`
- Create: `src/ops/titlekeys.rs`
- Create: `src/ops/undupe.rs`
- Modify: `src/ops/mod.rs`
- Create: `tests/ops_misc_parity.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn misc_ops_match_python_behavior_on_fixture_set() {
    // assert file sets and output bytes for extract/create/titlekeys/undupe
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test misc_ops_match_python_behavior_on_fixture_set -q`
Expected: FAIL because modules are missing.

**Step 3: Write minimal implementation**

Implement remaining operations with cloned semantics.

**Step 4: Run test to verify it passes**

Run: `cargo test misc_ops_match_python_behavior_on_fixture_set -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/ops tests/ops_misc_parity.rs
git commit -m "feat: implement extract create titlekeys and undupe parity"
```

### Task 12: Corpus-Wide Parity Harness + Documentation

**Files:**
- Modify: `src/parity/mod.rs`
- Create: `src/parity/diff.rs`
- Create: `tests/corpus_parity.rs`
- Create: `README.md`
- Create: `docs/parity.md`

**Step 1: Write the failing test**

```rust
#[test]
fn corpus_parity_runner_detects_no_mismatches() {
    let report = nsz_rs::parity::run_default_corpus().unwrap();
    assert_eq!(report.mismatches.len(), 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test corpus_parity_runner_detects_no_mismatches -q`
Expected: FAIL until all parity behaviors are wired.

**Step 3: Write minimal implementation**

Implement corpus discovery under `/home/matteo/Documents/switch_games/Bad Cheese [NSP]`, baseline invocation, and diff reports. Update docs to describe running parity checks and adding corpus samples.

**Step 4: Run test to verify it passes**

Run: `cargo test -- --nocapture`
Expected: PASS with no mismatches for enabled corpus.

**Step 5: Commit**

```bash
git add src/parity tests/corpus_parity.rs README.md docs/parity.md
git commit -m "feat: add corpus-wide parity harness and usage docs"
```

### Task 13: Final Verification Gate

**Files:**
- Modify as needed based on verification findings.

**Step 1: Run full verification**

Run: `cargo fmt -- --check`
Expected: PASS.

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS.

Run: `cargo test --all-targets --all-features`
Expected: PASS.

**Step 2: Run parity gate command**

Run: `cargo test corpus_parity_runner_detects_no_mismatches -- --nocapture`
Expected: PASS with zero mismatches.

**Step 3: Commit stabilization**

```bash
git add -A
git commit -m "chore: finalize parity gate and documentation"
```
