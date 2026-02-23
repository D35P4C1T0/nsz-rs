# CONTINUITY

Facts only. No transcripts. If unknown, write UNCONFIRMED.
Add dated entries with provenance tags per AGENTS.md: [USER], [CODE], [TOOL], [ASSUMPTION].

## Snapshot

Goal: 2026-02-22 [USER] Reimplement Python `nsz` in native safe Rust with total feature parity.
Now: 2026-02-22 [CODE] Task 9 operation wiring is in place with fail-fast parity test scaffold and local Python baseline venv support.
Next: 2026-02-22 [ASSUMPTION] Continue Task 9 by implementing native Rust decompress/verify internals and removing baseline adapter dependency.
Open Questions: 2026-02-22 [UNCONFIRMED] Baseline parity execution needs user key files (`prod.keys`/`keys.txt`) to exist in expected locations.

## Done (recent)
- 2026-02-22 [USER] Chose parity harness default mode `fail-fast`.
- 2026-02-22 [CODE] Started Task 9: added `src/ops/decompress.rs`, `src/ops/verify.rs`, and wired `lib.rs` operations.
- 2026-02-22 [CODE] Added real-corpus parity test scaffold `tests/decompress_verify_parity.rs` with `NSZ_RUN_HEAVY_PARITY=1` gating and key-file precheck.
- 2026-02-22 [TOOL] Created project-local baseline venv `.venv-nsz-baseline` and installed Python `nsz` requirements (`pycryptodome`, `zstandard`, `enlighten`).
- 2026-02-22 [TOOL] Heavy parity run now reaches baseline key-loading stage and fails due missing key files (`Could not load keys file`).
- 2026-02-22 [TOOL] Default suite remains green (`cargo fmt --all && cargo test -q`).
- 2026-02-22 [CODE] Foundation Tasks 1-8 completed and committed; ExecPlan updated through Task 9 bootstrap status.

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
- 2026-02-22 [TOOL] Baseline inventory and reference pin completed: Python repo surveyed; baseline fixed to `4.6.1` commit `d84f7c813c3fe278104ff8877803f22028e57452`; corpus root set to `/home/matteo/Documents/switch_games/Bad Cheese [NSP]`.
- 2026-02-22 [TOOL] Foundations validated with passing targeted tests (`public_api_symbols_exist`, `python_baseline_reports_461`, defaults/error, fs/keys, PFS0/NCZ binary layout).
- 2026-02-22 [TOOL] `cargo fmt --all && cargo test -q` passes after Task 9 scaffolding changes.
- 2026-02-22 [TOOL] Local baseline venv created and requirements installed: `.venv-nsz-baseline/bin/pip install -r /home/matteo/Documents/prog/python/nsz/requirements.txt`.
- 2026-02-22 [TOOL] Heavy parity run in sandbox failed on multiprocessing manager permissions (`PermissionError`/`EOFError`); escalated rerun bypassed sandbox restriction.
- 2026-02-22 [TOOL] Heavy parity run outside sandbox currently fails at baseline key loading (`Exception: Could not load keys file`).
- 2026-02-22 [TOOL] `ls -la ~/.switch` and key path checks found no available `prod.keys`/`keys.txt` for baseline execution.
