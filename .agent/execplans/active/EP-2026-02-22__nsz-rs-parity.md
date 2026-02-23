# NSZ Rust Library 1:1 Parity With Python 4.6.1

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained under `.agent/PLANS.md` requirements and is the canonical execution artifact for this migration.

- Plan ID: EP-2026-02-22__nsz-rs-parity
- Status: ACTIVE
- Created: 2026-02-22
- Last Updated: 2026-02-22
- Owner: UNCONFIRMED

## Purpose / Big Picture

After this change, the repository will provide a native safe Rust library that reproduces Python `nsz` release `4.6.1` behavior and output bytes exactly for equivalent inputs and options. Users will be able to call high-level library operations (`compress`, `decompress`, `verify`, `extract`, `create`, `titlekeys`, `undupe`) and verify correctness through byte-level parity tests against the canonical corpus root `/home/matteo/Documents/switch_games/Bad Cheese [NSP]`.

## Progress

- [x] (2026-02-22T23:10Z) Gathered requirements and approved design sections: architecture, determinism, error model, and test gates.
- [x] (2026-02-22T23:12Z) Wrote design artifact `docs/plans/2026-02-22-nsz-rs-parity-design.md`.
- [x] (2026-02-22T23:20Z) Wrote implementation task plan `docs/plans/2026-02-22-nsz-rs-parity-implementation.md`.
- [x] (2026-02-22T23:34Z) Created Rust crate skeleton with high-level facade API and smoke test (`public_api_symbols_exist`).
- [x] (2026-02-22T23:38Z) Implemented baseline version reader `parity::python_runner::query_version` and passing test (`python_baseline_reports_461`).
- [x] (2026-02-22T23:40Z) Implemented deterministic compress defaults and parity mismatch error contract tests.
- [x] (2026-02-22T23:50Z) Implemented filesystem write policy primitives and key-loading required-key checks with passing tests.
- [x] (2026-02-22T23:56Z) Implemented minimal container and NCZ binary layout primitives with deterministic roundtrip tests.
- [ ] (2026-02-22T23:59Z) Started Task 9: added `ops::decompress`/`ops::verify` path wiring and real-corpus parity test scaffold (`decompress_verify_matches_python_for_fixture`), pending Python baseline dependency readiness for full run.
- [ ] Implement operations in parity-first order: decompress/verify, then compress, then extract/create/titlekeys/undupe.
- [ ] Implement corpus-wide parity harness and docs for adding new samples.
- [ ] Run formatting, lint, tests, and parity gates; fix regressions.
- [ ] Update plan sections and archive flow when complete.

## Surprises & Discoveries

- Observation: Local Python repo tag for the target release is `4.6.1` (without `v` prefix).
  Evidence: `git -C /home/matteo/Documents/prog/python/nsz rev-list -n 1 4.6.1` -> `d84f7c813c3fe278104ff8877803f22028e57452`.
- Observation: Multithreaded zstd behavior can produce differing output characteristics from single-thread mode.
  Evidence: python-zstandard multithread docs and zstd parameter docs captured in continuity receipts.
- Observation: Initial `cargo test` run failed until crate dependency resolution was allowed with network access.
  Evidence: Failure on downloading `index.crates.io/config.json`, then success after approved escalated `cargo test`.
- Observation: Heavy real-corpus parity execution currently fails because Python baseline runtime is missing `Crypto` module (`pycryptodome` dependency).
  Evidence: `NSZ_RUN_HEAVY_PARITY=1 cargo test decompress_verify_matches_python_for_fixture -- --nocapture` failed with `ModuleNotFoundError: No module named 'Crypto'`.
- Observation: After installing Python deps locally, heavy parity execution now fails due missing key files required by baseline NSZ tooling.
  Evidence: Escalated heavy run fails with `Exception: Could not load keys file.` and no `prod.keys`/`keys.txt` found in expected paths.

## Decision Log

- Decision: Build library-only (no GUI), high-level facade public API.
  Rationale: Smaller stable API surface and better long-term optimization freedom with fewer breakage risks.
  Date/Author: 2026-02-22 / [USER]
- Decision: Require strict 1:1 byte parity against Python `4.6.1`.
  Rationale: User requirement prioritizes deterministic compatibility over early refactor flexibility.
  Date/Author: 2026-02-22 / [USER]
- Decision: Use behavior-clone-first migration (Approach A).
  Rationale: Lowest risk path to strict byte parity; optimize only after parity gates are stable.
  Date/Author: 2026-02-22 / [USER]
- Decision: Canonical parity corpus root is `/home/matteo/Documents/switch_games/Bad Cheese [NSP]` and must support future additions.
  Rationale: Real-world fixtures maximize practical compatibility confidence.
  Date/Author: 2026-02-22 / [USER]
- Decision: Parity harness default reporting mode is fail-fast.
  Rationale: User preference for immediate stop on first mismatch to speed triage.
  Date/Author: 2026-02-22 / [USER]

## Outcomes & Retrospective

Current status: implementation foundation slice is complete through binary/header primitives, and Task 9 wiring has started. The library now includes API scaffolding, baseline version probing, defaults/error contracts, filesystem policy checks, key parser validation, deterministic roundtrip tests for PFS0 and NCZ block headers, plus preliminary decompress/verify operation wiring and parity test harnessing. Remaining work is full native operation implementations and corpus-wide parity validation once baseline dependencies are available.

## Context and Orientation

The Rust repo initially contains no source files. The Python reference implementation lives at `/home/matteo/Documents/prog/python/nsz` and target semantics are pinned to commit `d84f7c813c3fe278104ff8877803f22028e57452` (release `4.6.1`).

Key Python modules that define behavior:

- `nsz/__init__.py`: orchestration and operation flow.
- `nsz/ParseArguments.py`: defaults and option meanings.
- `nsz/BlockCompressor.py`, `nsz/SolidCompressor.py`: compression behavior.
- `nsz/NszDecompressor.py`: decompression and verification behavior.
- `nsz/FileExistingChecks.py`, `nsz/PathTools.py`, `nsz/undupe.py`: filesystem and duplicate handling semantics.
- `nsz/nut/Keys.py`: key loading and validation logic.

Canonical planning artifacts in this repo:

- `docs/plans/2026-02-22-nsz-rs-parity-design.md`
- `docs/plans/2026-02-22-nsz-rs-parity-implementation.md`
- `.agent/CONTINUITY.md`

## Plan of Work

Start by creating a minimal compileable Rust crate with explicit facade signatures and typed configuration/error contracts. Add a parity baseline runner that executes Python `4.6.1` for comparison. Implement deterministic and byte-sensitive foundations first (path policies, key loading, container binary formats, NCZ header/block structures). Build operation paths in risk order: `decompress`/`verify` first, then `compress`, then remaining operations (`extract`, `create`, `titlekeys`, `undupe`). Add a corpus parity harness that discovers fixtures under the canonical root and compares Rust and Python outputs byte-for-byte with actionable diff metadata. Gate all optimization/refactor work behind passing parity tests.

## Concrete Steps

Working directory: `/home/matteo/Documents/prog/rust/nsz-rs`

1. Initialize and validate crate skeleton.

    cargo test public_api_symbols_exist -q

2. Validate Python baseline binding.

    cargo test python_baseline_reports_461 -q

3. Implement deterministic defaults and error model.

    cargo test compress_defaults_match_python_461 -q
    cargo test parity_mismatch_error_carries_offsets -q

4. Implement foundational modules and their tests.

    cargo test file_policy_rejects_duplicate_without_overwrite -q
    cargo test key_loader_checks_required_entries -q
    cargo test pfs0_header_roundtrip_is_stable -q
    cargo test ncz_block_header_binary_layout_matches_python -q

5. Implement operations and parity checks.

    cargo test decompress_verify_matches_python_for_fixture -q
    cargo test compress_output_is_byte_identical_to_python -q
    cargo test misc_ops_match_python_behavior_on_fixture_set -q

6. Run full gate.

    cargo fmt -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all-targets --all-features
    cargo test corpus_parity_runner_detects_no_mismatches -- --nocapture

Expected result at completion: all tests pass and parity mismatch count is zero for enabled corpus samples.

## Validation and Acceptance

Acceptance is behavioral and byte-level:

- For each operation, Rust output files must match Python baseline output bytes exactly for same inputs and options.
- Operation reports must classify processed/skipped/warned files consistently with parity expectations.
- Full test suite (`fmt`, `clippy`, `cargo test`, parity gate) must pass.
- Adding new files under corpus root must include them in parity discovery without code changes.

## Idempotence and Recovery

Commands are rerunnable. If a parity test fails, preserve both output directories, inspect diff metadata, and fix the corresponding module before rerunning only the failing test. If baseline tool invocation fails, verify Python repo path and keys availability first, then rerun targeted parity tests. No destructive global operations are required.

## Artifacts and Notes

Important artifacts created during planning:

- `docs/plans/2026-02-22-nsz-rs-parity-design.md`
- `docs/plans/2026-02-22-nsz-rs-parity-implementation.md`
- `.agent/execplans/active/EP-2026-02-22__nsz-rs-parity.md`

Baseline reference command transcript:

    git -C /home/matteo/Documents/prog/python/nsz rev-list -n 1 4.6.1
    d84f7c813c3fe278104ff8877803f22028e57452

## Interfaces and Dependencies

Public interface (stable):

- `nsz_rs::compress(&CompressRequest) -> Result<OperationReport, NszError>`
- `nsz_rs::decompress(&DecompressRequest) -> Result<OperationReport, NszError>`
- `nsz_rs::verify(&VerifyRequest) -> Result<VerifyReport, NszError>`
- `nsz_rs::extract(&ExtractRequest) -> Result<OperationReport, NszError>`
- `nsz_rs::create(&CreateRequest) -> Result<OperationReport, NszError>`
- `nsz_rs::titlekeys(&TitleKeysRequest) -> Result<OperationReport, NszError>`
- `nsz_rs::undupe(&UndupeRequest) -> Result<OperationReport, NszError>`

Core dependencies to introduce:

- `zstd` for compression/decompression behavior matching.
- `sha2` for SHA-256 parity and verification paths.
- `aes` + `ctr` for AES-CTR/ECB flows currently done in Python.
- `thiserror` for typed error contract.
- `serde`/`serde_json` for structured reports and optional harness metadata.
- `regex`, `walkdir`, `tempfile` for filesystem behavior and harness runs.

## Plan Revision Notes (bottom-of-file change notes)

- (2026-02-22) Initial plan created from approved brainstorming design and repository continuity constraints.
- (2026-02-22) Updated progress after completing implementation Tasks 1-4 and recording dependency-resolution discovery.
- (2026-02-22) Updated progress after completing Tasks 5-8 (fs policy, key loading, container and NCZ header primitives).
- (2026-02-22) Updated progress after starting Task 9 and recording baseline blockers (dependency install then key-file absence).
