# CONTINUITY

Facts only. No transcripts. If unknown, write UNCONFIRMED.
Add dated entries with provenance tags per AGENTS.md: [USER], [CODE], [TOOL], [ASSUMPTION].

## Snapshot

Goal: 2026-02-22 [USER] Reimplement Python `nsz` in native safe Rust with total feature parity.
Now: 2026-02-24 [CODE] Byte parity remains intact for heavy NSP/XCI compress gates; native NCZ compress/decompress paths now avoid extra per-chunk allocations and benchmark harness reports final release-mode speed snapshot (`compress` ~= `0.992x` Python, `decompress` ~= `3.668x` Python on staged fixture).
Next: 2026-02-24 [ASSUMPTION] Continue parity-safe speed iterations with release-mode Rust-vs-Python benchmark measurements after each optimization step and periodic `/tmp` cleanup.
Open Questions: 2026-02-24 [UNCONFIRMED] None.

## Done (recent)
- 2026-02-24 [USER] Requested periodic `/tmp` artifact cleanup during heavy testing loops and benchmark sampling after each optimization iteration.
- 2026-02-24 [CODE] Optimized native NCZ decompress/compress memory paths: removed per-section decrypt clones in `src/ncz/decompress.rs` and removed per-chunk copy for unencrypted compress segments in `src/ncz/compress.rs`.
- 2026-02-24 [CODE] Added release-mode same-input benchmark harness `tests/perf_compare_python.rs` with Python-vs-Rust timing and byte-equality assertions for `compress` and `decompress`.
- 2026-02-24 [CODE] Added regression coverage for NCZ leading-gap handling: `ncz_native_decompress_preserves_leading_gap_before_first_section`.
- 2026-02-24 [CODE] Refined XCI tail-trim policy in `src/ops/compress.rs` to keep native single-partition XCI correctness while preserving multi-partition byte parity with Python.
- 2026-02-24 [TOOL] Final verification sweep passed: standard gates, heavy XCI parity, and fast heavy NSP compress parity.
- 2026-02-24 [TOOL] Final release benchmark snapshot captured and recorded in receipts.

## Working set
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/CONTINUITY.md
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/execplans/active/EP-2026-02-22__nsz-rs-parity.md
- /home/matteo/Documents/prog/rust/nsz-rs/src/ops/compress.rs
- /home/matteo/Documents/prog/rust/nsz-rs/src/ncz/compress.rs
- /home/matteo/Documents/prog/rust/nsz-rs/src/ncz/decompress.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/perf_compare_python.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/ncz_decompress_meta.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/compress_xci_parity.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/decompress_verify_parity.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/compress_native_nsz_xcz_op.rs
- /home/matteo/Documents/prog/rust/nsz-rs/.venv-nsz-baseline/bin/python3
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
- D014 ACTIVE: 2026-02-24 [USER] During testing, periodically clean compressed parity artifacts in `/tmp` to prevent space-related interruptions.
- D015 ACTIVE: 2026-02-24 [USER] During performance tuning, run same-input benchmarks for both Rust and Python implementations after each optimization iteration.
- D016 ACTIVE: 2026-02-24 [USER] If verification reaches a stable state, create a commit for the completed optimization slice.

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
- 2026-02-23 [TOOL] Added passing native `.nca` verify coverage: `cargo test verify_uses_native_path_for_nca_inputs -- --nocapture` and `cargo test verify_skips_cnmt_nca_hash_check -- --nocapture`.
- 2026-02-23 [TOOL] Added optional heavy parity fixture limiter controls (`NSZ_HEAVY_PARITY_MODE`, `NSZ_HEAVY_PARITY_MAX_FILES`) and validated compile/test gates.
- 2026-02-23 [TOOL] Escalated fast heavy parity run was interrupted by user before completion; no post-abort parity process remained active.
- 2026-02-23 [TOOL] Fast heavy parity completed successfully after rerun: `NSZ_RUN_HEAVY_PARITY=1 NSZ_HEAVY_PARITY_MODE=fast cargo test decompress_verify_matches_python_for_fixture -- --nocapture` (escalated), duration `507.12s`.
- 2026-02-23 [TOOL] Added and passed native XCI/XCZ verify coverage: `cargo test verify_uses_native_path_for_xci_inputs -- --nocapture` and `cargo test verify_uses_native_path_for_xcz_inputs -- --nocapture`.
- 2026-02-23 [CODE] Added repository `.gitignore` to ignore local build artifacts, local venvs, and key files (`target/`, `.venv*`, `keys.txt`, `prod.keys`).
- 2026-02-23 [TOOL] Added and passed native XCZ decompression coverage: `cargo test decompress_uses_native_path_for_xcz_inputs -- --nocapture`.
- 2026-02-23 [TOOL] Full validation remains green after native XCZ decompression update: `cargo fmt --all && cargo test -q && cargo clippy --all-targets --all-features -- -D warnings`.
- 2026-02-23 [USER] Provided real XCI fixture path for parity reference: `/home/matteo/Documents/switch_games/xci_test/HEART of CROWN.xci`.
- 2026-02-23 [TOOL] Added red/green coverage for compress op wiring: `cargo test compress_invokes_cli_and_reports_outputs -- --nocapture` fails before implementation, passes after.
- 2026-02-23 [TOOL] Full validation remains green after compress wiring update: `cargo fmt --all && cargo test -q && cargo clippy --all-targets --all-features -- -D warnings`.
- 2026-02-23 [TOOL] Added red/green coverage for native compress routing: `cargo test compress_uses_native_path_for_nsp_inputs -- --nocapture` fails before native path, passes after.
- 2026-02-23 [TOOL] Added passing native XCI compress coverage: `cargo test compress_uses_native_path_for_xci_inputs -- --nocapture`.
- 2026-02-23 [TOOL] Full validation remains green after native compress update: `cargo fmt --all && cargo test -q && cargo clippy --all-targets --all-features -- -D warnings`.
- 2026-02-23 [TOOL] Added heavy compress parity harness and split test entrypoint: `compress_matches_python_for_fixture` (gated by `NSZ_RUN_HEAVY_COMPRESS_PARITY=1`).
- 2026-02-23 [TOOL] Staged fast compress parity mismatch on `/home/matteo/Documents/switch_games/Bad Cheese [NSP]/Bad Cheese [0100BAE021208800][v327680].nsp`: entry names aligned after heuristic fix, but bytes still mismatch (`first_diff_offset=48`, baseline size `128302175`, rust size `159383620`).
- 2026-02-23 [TOOL] Fast staged compress parity runtime observed around `92s`; pre-staging runs could hang/timeout (>300s) due larger fixture selection and optional XCI work.
- 2026-02-23 [CODE] Implemented native NCZ compression planner/encoder parity layer (`src/container/nca.rs`, `src/ncz/compress.rs`, `src/ops/compress.rs`) with key/ticket derivation, BKTR section synthesis, and Python-equivalent partition streaming.
- 2026-02-23 [TOOL] Native compress parity debugging isolated and fixed three blockers: XTS tweak endianness, BKTR crypto type normalization, and `sectionStart` subtraction mismatch versus Python section object behavior.
- 2026-02-23 [TOOL] Fast heavy compress parity now passes byte-identical for sampled NSP fixture after native planner fixes: `NSZ_RUN_HEAVY_COMPRESS_PARITY=1 NSZ_HEAVY_PARITY_MODE=fast cargo test compress_matches_python_for_fixture -- --nocapture` (escalated), duration `75.29s`.
- 2026-02-23 [TOOL] Full-mode NSP compress parity passes byte-identical after native planner fixes: `NSZ_RUN_HEAVY_COMPRESS_PARITY=1 NSZ_HEAVY_PARITY_MODE=full cargo test compress_matches_python_for_fixture -- --nocapture` (escalated), duration `262.35s`.
- 2026-02-23 [TOOL] XCI-inclusive parity blocked by baseline key availability on `/home/matteo/Documents/switch_games/xci_test/HEART of CROWN.xci`: Python `nsz` fails with `master_key_13 missing ... keys.txt`.
- 2026-02-23 [CODE] Added misc operation parity wrappers and request surfaces (`extract/create/titlekeys/undupe`) plus CLI-path test coverage in `tests/ops_misc_cli_path.rs`; full fmt/test/clippy gate remains green.
- 2026-02-23 [TOOL] Added XCI-only heavy parity harness (`tests/compress_xci_parity.rs`) and verified pass: `NSZ_RUN_HEAVY_XCI_COMPRESS_PARITY=1 cargo test --test compress_xci_parity -- --nocapture` (duration ~1101s).
- 2026-02-23 [CODE] Resolved final XCI `+302` mismatch by trimming final trailing alignment bytes in native output (`src/ops/compress.rs`) to mirror Python `4.6.1`.
- 2026-02-23 [TOOL] Combined heavy fast compress parity pass for NSP+XCI: `NSZ_RUN_HEAVY_COMPRESS_PARITY=1 NSZ_HEAVY_PARITY_MODE=fast NSZ_HEAVY_COMPRESS_INCLUDE_XCI=1 cargo test compress_matches_python_for_fixture -- --nocapture` (duration ~983s).
- 2026-02-24T01:20Z [TOOL] `/tmp` capacity check healthy: `df -h /tmp` reports `16G` available and `0` active `/tmp/nsz-rs-*` directories.
- 2026-02-24T01:21Z [TOOL] Ran periodic cleanup pass for test artifacts: `find /tmp -maxdepth 1 -type d -name 'nsz-rs-*' -exec rm -rf {} +`; post-clean check remains `16G` free in `/tmp`.
- 2026-02-24T02:10Z [TOOL] Iteration benchmark (release mode) after allocation optimizations: `NSZ_RUN_HEAVY_SPEED_BENCH=1 cargo test --release --test perf_compare_python -- --nocapture` reported `compress speedup=0.939x`, `decompress speedup=3.581x`.
- 2026-02-24T02:13Z [TOOL] Iteration benchmark rerun after NCZ block-path tuning attempt: `compress speedup=0.991x`, `decompress speedup=3.654x`; follow-up validation exposed XCI structural regression so tuning was not kept.
- 2026-02-24T02:31Z [TOOL] Heavy XCI parity failed after interim trim-rule change with exact `+302` tail delta (`first_diff_offset = baseline_size`), then passed after final trim-rule refinement.
- 2026-02-24T02:46Z [TOOL] Heavy XCI parity pass restored: `NSZ_RUN_HEAVY_XCI_COMPRESS_PARITY=1 cargo test --test compress_xci_parity -- --nocapture` (duration `894.10s`).
- 2026-02-24T02:48Z [TOOL] Fast heavy NSP compress parity remained byte-identical: `NSZ_RUN_HEAVY_COMPRESS_PARITY=1 NSZ_HEAVY_PARITY_MODE=fast cargo test compress_matches_python_for_fixture -- --nocapture` (duration `68.59s`).
- 2026-02-24T02:49Z [TOOL] Final release benchmark snapshot: `compress speedup=0.992x`, `decompress speedup=3.668x` on `/home/matteo/Documents/switch_games/Bad Cheese [NSP]/Bad Cheese [0100BAE021208000][v0]`.
