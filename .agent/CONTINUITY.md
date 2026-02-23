# CONTINUITY

Facts only. No transcripts. If unknown, write UNCONFIRMED.
Add dated entries with provenance tags per AGENTS.md: [USER], [CODE], [TOOL], [ASSUMPTION].

## Snapshot

Goal: 2026-02-22 [USER] Reimplement Python `nsz` in native safe Rust with total feature parity.
Now: 2026-02-22 [CODE] Task 9 started with `decompress`/`verify` operation wiring and parity test scaffold.
Next: 2026-02-22 [ASSUMPTION] Continue Task 9 by replacing baseline-adapter behavior with native Rust logic and unblocking heavy corpus parity execution.
Open Questions: 2026-02-22 [UNCONFIRMED] Python baseline environment in this machine lacks `pycryptodome` (`Crypto` module), blocking heavy parity runs.

## Done (recent)
- 2026-02-22 [USER] Chose parity harness default mode `fail-fast`.
- 2026-02-22 [CODE] Started Task 9: added `src/ops/decompress.rs`, `src/ops/verify.rs`, and wired `lib.rs` operations.
- 2026-02-22 [CODE] Added real-corpus parity test scaffold `tests/decompress_verify_parity.rs` (runs when `NSZ_RUN_HEAVY_PARITY=1`).
- 2026-02-22 [TOOL] Heavy parity run reproduced dependency blocker (`ModuleNotFoundError: No module named 'Crypto'`) from Python baseline.
- 2026-02-22 [TOOL] Current default suite still passes via `cargo fmt --all && cargo test -q`.
- 2026-02-22 [CODE] Completed foundation Tasks 1-8 (scaffold, defaults/errors, fs/crypto, container/ncz primitives) and committed checkpoints.
- 2026-02-22 [CODE] Updated ExecPlan progress/discoveries with Task 9 status and blocker details.

## Working set
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/CONTINUITY.md
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/execplans/INDEX.md
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/execplans/active/EP-2026-02-22__nsz-rs-parity.md
- /home/matteo/Documents/prog/rust/nsz-rs/src/lib.rs
- /home/matteo/Documents/prog/rust/nsz-rs/src/parity/python_runner.rs
- /home/matteo/Documents/prog/rust/nsz-rs/src/ops/decompress.rs
- /home/matteo/Documents/prog/rust/nsz-rs/src/ops/verify.rs
- /home/matteo/Documents/prog/rust/nsz-rs/src/container/pfs0.rs
- /home/matteo/Documents/prog/rust/nsz-rs/src/ncz/header.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/decompress_verify_parity.rs
- /home/matteo/Documents/prog/rust/nsz-rs/tests/ncz_header_block.rs
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
- 2026-02-22 [TOOL] `ls -la .agent` in `nsz-rs`: continuity/plans/index present.
- 2026-02-22 [TOOL] `rg --files /home/matteo/Documents/prog/python/nsz` used to inventory source tree and assets.
- 2026-02-22 [TOOL] `find ... -name '*.py' | xargs wc -l` used to quantify migration size.
- 2026-02-22 [TOOL] `sed` reads of README and core modules captured feature/format behavior and flags.
- 2026-02-22 [TOOL] `git -C /home/matteo/Documents/prog/python/nsz log --oneline -n 12` captured recent upstream context.
- 2026-02-22 [TOOL] `git -C /home/matteo/Documents/prog/python/nsz rev-list -n 1 4.6.1` resolved canonical baseline commit `d84f7c813c3fe278104ff8877803f22028e57452`.
- 2026-02-22 [TOOL] Web docs checked: `python-zstandard` compressor parameters and multithread behavior (`https://python-zstandard.readthedocs.io/en/latest/compressor.html`, `https://python-zstandard.readthedocs.io/en/0.25.0/multithreaded.html`).
- 2026-02-22 [TOOL] Web docs checked: zstd frame parameter defaults (`contentSize/checksum/dictID`) and threading parameter controls (`https://facebook.github.io/zstd/zstd_manual.html`).
- 2026-02-22 [USER] Corpus root designated for parity harness: `/home/matteo/Documents/switch_games/Bad Cheese [NSP]`; future sample expansion expected.
- 2026-02-22 [CODE] Wrote design and implementation plan docs under `docs/plans/` and canonical ExecPlan under `.agent/execplans/active/`.
- 2026-02-22 [TOOL] Created commit `41f2631` for the design document artifact.
- 2026-02-22 [TOOL] `cargo test public_api_symbols_exist -q` initially failed without `Cargo.toml`, then passed after crate bootstrap.
- 2026-02-22 [TOOL] `cargo test python_baseline_reports_461 -q`, `cargo test compress_defaults_match_python_461 -q`, `cargo test parity_mismatch_error_carries_offsets -q` all pass after corresponding implementations.
- 2026-02-22 [TOOL] `cargo test -q` currently passes all defined tests.
- 2026-02-22 [TOOL] `cargo test file_policy_rejects_duplicate_without_overwrite -q`, `cargo test key_loader_checks_required_entries -q`, `cargo test pfs0_header_roundtrip_is_stable -q`, `cargo test ncz_block_header_binary_layout_matches_python -q` all pass.
- 2026-02-22 [TOOL] `cargo fmt --all && cargo test -q` passes after Task 8 implementation.
- 2026-02-22 [TOOL] `NSZ_RUN_HEAVY_PARITY=1 cargo test decompress_verify_matches_python_for_fixture -- --nocapture` fails in baseline Python with missing `Crypto` module (`pycryptodome` not installed).
- 2026-02-22 [TOOL] `cargo test decompress_verify_matches_python_for_fixture -q` passes in default mode (heavy parity env flag not set).
