# CONTINUITY

Facts only. No transcripts. If unknown, write UNCONFIRMED.
Add dated entries with provenance tags per AGENTS.md: [USER], [CODE], [TOOL], [ASSUMPTION].

## Snapshot

Goal: 2026-02-22 [USER] Reimplement Python `nsz` in native safe Rust with total feature parity.
Now: 2026-02-23 [CODE] Native paths now cover `.ncz` + `.nsz` decompression and `.ncz`/`.nsp`/`.nsz` verify without Python fallback; expanded heavy corpus parity passes.
Next: 2026-02-23 [ASSUMPTION] Continue Task 9 by reducing fallback for XCI/XCZ-related paths and filling remaining operation parity gaps.
Open Questions: 2026-02-23 [UNCONFIRMED] Expanded heavy parity runtime is high (~18 minutes in debug); decision pending on whether to split fast-vs-full corpus gates.

## Done (recent)
- 2026-02-23 [CODE] Added native PFS0/NSP parser-writer (`container::nsp`) with bounds checks and deterministic re-encoding.
- 2026-02-23 [CODE] Added native `.nsz -> .nsp` decompression path in `ops::decompress` (including `.ncz -> .nca` entry rewrite).
- 2026-02-23 [CODE] Added native `.nsp`/`.nsz` verify path in `ops::verify` plus native tests for `.nsz` decompress and `.nsp`/`.nsz` verify without Python.
- 2026-02-23 [CODE] Matched Python crypto semantics by treating non-`3/4` NCZ crypto types as passthrough; added regression test.
- 2026-02-23 [CODE] Expanded heavy corpus parity harness to iterate discovered `.nsz` and `.nsp` fixtures under canonical corpus root.
- 2026-02-23 [TOOL] Validation gates pass: `cargo fmt --all && cargo test -q && cargo clippy --all-targets --all-features -- -D warnings`.
- 2026-02-23 [TOOL] Expanded heavy parity passes: `NSZ_RUN_HEAVY_PARITY=1 cargo test decompress_verify_matches_python_for_fixture -- --nocapture` (escalated; ~1093.75s).

## Working set
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/CONTINUITY.md
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/execplans/INDEX.md
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/execplans/active/EP-2026-02-22__nsz-rs-parity.md
- /home/matteo/Documents/prog/rust/nsz-rs/src/container/nsp.rs
- /home/matteo/Documents/prog/rust/nsz-rs/src/ops/decompress.rs
- /home/matteo/Documents/prog/rust/nsz-rs/src/ops/verify.rs
- /home/matteo/Documents/prog/rust/nsz-rs/src/ncz/decompress.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/decompress_verify_parity.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/ncz_decompress_meta.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/decompress_native_nsz_op.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/verify_native_nsp_nsz_op.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/verify_native_ncz_op.rs
- /home/matteo/Documents/switch_games/Bad Cheese [NSP]

## Decisions
- D001 ACTIVE: 2026-02-22 [USER] Target outcome is total feature parity with Python `nsz`; implementation language constraints are native safe Rust.
- D002 ACTIVE: 2026-02-22 [USER] Deliverable is strictly a Rust library; no GUI implementation.
- D003 ACTIVE: 2026-02-22 [USER] Public API will prioritize a high-level facade (operation functions + config structs) rather than low-level-first API.
- D004 ACTIVE: 2026-02-22 [USER] Output compatibility bar is strict 1:1 byte identity with Python `nsz` outputs, not only functional parity.
- D005 ACTIVE: 2026-02-22 [USER] Canonical reference implementation is Python tag `4.6.1` (commit `d84f7c813c3fe278104ff8877803f22028e57452`).
- D006 ACTIVE: 2026-02-22 [USER] Chosen migration strategy is Approach A: behavior clone and byte-parity lock before optimization/refactor.
- D007 ACTIVE: 2026-02-22 [USER] Parity validation corpus source is user-provided real-world NSP/XCI/NSZ/XCZ samples.
- D008 ACTIVE: 2026-02-22 [USER] Canonical corpus root is `/home/matteo/Documents/switch_games/Bad Cheese [NSP]`, and parity dataset must support incremental additions over time.
- D009 ACTIVE: 2026-02-22 [USER] Architecture section approved: one library crate with high-level public operations and private parity-first internal modules.
- D010 ACTIVE: 2026-02-22 [USER] Determinism section approved: Python-order behavior clone, explicit byte-affecting config surface, incremental corpus parity harness with byte-level diff reporting.
- D011 ACTIVE: 2026-02-22 [USER] Error/API section approved: typed `NszError`, structured results, and explicit parity mismatch reporting.
- D012 ACTIVE: 2026-02-22 [USER] Testing section approved: layered tests plus Python-vs-Rust byte-parity gates on canonical corpus.
- D013 ACTIVE: 2026-02-22 [USER] Parity harness default mode is fail-fast.

## Receipts
- 2026-02-22 [TOOL] Baseline inventory and reference pin completed: Python repo surveyed; baseline fixed to `4.6.1` commit `d84f7c813c3fe278104ff8877803f22028e57452`; corpus root set to `/home/matteo/Documents/switch_games/Bad Cheese [NSP]`.
- 2026-02-22 [TOOL] Foundations validated with passing targeted tests (`public_api_symbols_exist`, `python_baseline_reports_461`, defaults/error, fs/keys, PFS0/NCZ binary layout).
- 2026-02-22 [TOOL] `cargo fmt --all && cargo test -q` passes after Task 9 scaffolding changes.
- 2026-02-22 [TOOL] Local baseline venv created and requirements installed: `.venv-nsz-baseline/bin/pip install -r /home/matteo/Documents/prog/python/nsz/requirements.txt`.
- 2026-02-22 [TOOL] Heavy parity run in sandbox failed on multiprocessing manager permissions (`PermissionError`/`EOFError`); escalated rerun bypassed sandbox restriction.
- 2026-02-22 [TOOL] Heavy parity run outside sandbox currently fails at baseline key loading (`Exception: Could not load keys file`).
- 2026-02-22 [TOOL] `ls -la ~/.switch` and key path checks found no available `prod.keys`/`keys.txt` for baseline execution.
- 2026-02-22 [USER] Added `keys.txt` in workspace root (`/home/matteo/Documents/prog/rust/nsz-rs/keys.txt`).
- 2026-02-22 [TOOL] Heavy parity rerun passes after key-home provisioning: `NSZ_RUN_HEAVY_PARITY=1 cargo test decompress_verify_matches_python_for_fixture -- --nocapture`.
- 2026-02-22 [TOOL] Native NCZ metadata test passes: `cargo test ncz_decompressed_size_matches_header_sections -q`.
- 2026-02-22 [TOOL] Native no-crypto NCZ roundtrip test passes: `cargo test ncz_native_decompress_roundtrip_no_crypto -q`.
- 2026-02-22 [TOOL] Native `.ncz` op-path test passes: `cargo test decompress_uses_native_path_for_ncz_inputs -q`.
- 2026-02-22 [TOOL] Native crypto NCZ roundtrip test passes: `cargo test ncz_native_decompress_roundtrip_crypto_type3 -q`.
- 2026-02-22 [TOOL] Heavy fail-fast parity rerun passes after native crypto update: `NSZ_RUN_HEAVY_PARITY=1 cargo test decompress_verify_matches_python_for_fixture -- --nocapture`.
- 2026-02-22 [TOOL] Native `.ncz` verify-path test passes: `cargo test verify_uses_native_path_for_ncz_inputs -q`.
- 2026-02-22 [TOOL] Heavy fail-fast parity rerun passes after native verify update: `NSZ_RUN_HEAVY_PARITY=1 cargo test decompress_verify_matches_python_for_fixture -- --nocapture`.
- 2026-02-23 [TOOL] Added and passed block-stream roundtrip coverage: `cargo test ncz_native_decompress_roundtrip_block_stream -q`.
- 2026-02-23 [TOOL] Validation gates pass after `NCZBLOCK` decode addition: `cargo fmt --all && cargo test -q` and `cargo clippy --all-targets --all-features -- -D warnings`.
- 2026-02-23 [TOOL] Heavy parity in sandbox still fails with Python multiprocessing permission; escalated rerun passes with no parity mismatch.
- 2026-02-23 [TOOL] New red-green coverage landed for native non-`.ncz` paths: `decompress_uses_native_path_for_nsz_inputs`, `verify_uses_native_path_for_nsp_inputs`, `verify_uses_native_path_for_nsz_inputs`.
- 2026-02-23 [TOOL] Real-corpus heavy parity initially failed on `UnsupportedFeature` for NCZ crypto type != 0/3/4; fixed by passthrough semantics and regression test `ncz_native_decompress_unknown_crypto_type_is_passthrough`.
- 2026-02-23 [TOOL] Expanded heavy parity passes after fixes: `NSZ_RUN_HEAVY_PARITY=1 cargo test decompress_verify_matches_python_for_fixture -- --nocapture` (escalated), duration `1093.75s`.
