# CONTINUITY

Facts only. No transcripts. If unknown, write UNCONFIRMED.
Add dated entries with provenance tags per AGENTS.md: [USER], [CODE], [TOOL], [ASSUMPTION].

## Snapshot

Goal: 2026-02-22 [USER] Reimplement Python `nsz` in native safe Rust with total feature parity.
Now: 2026-02-22 [CODE] Design and implementation planning artifacts created; ExecPlan activated and indexed.
Next: 2026-02-22 [ASSUMPTION] Begin implementation from plan Task 1 with TDD and parity-first sequencing.
Open Questions: 2026-02-22 [UNCONFIRMED] Parity harness mode default (fail-fast vs collect-all mismatch report).

## Done (recent)
- 2026-02-22 [CODE] Added design document: `docs/plans/2026-02-22-nsz-rs-parity-design.md`.
- 2026-02-22 [TOOL] Committed design doc as `41f2631` with message `docs: add nsz rust parity design`.
- 2026-02-22 [CODE] Added implementation task plan: `docs/plans/2026-02-22-nsz-rs-parity-implementation.md`.
- 2026-02-22 [CODE] Added canonical ExecPlan: `.agent/execplans/active/EP-2026-02-22__nsz-rs-parity.md`.
- 2026-02-22 [CODE] Updated ExecPlan index with active entry for `EP-2026-02-22__nsz-rs-parity`.
- 2026-02-22 [USER] Approved all four design sections and finalized parity constraints/baseline/corpus path.
- 2026-02-22 [TOOL] Completed Python baseline analysis and pinned release commit for parity reference.

## Working set
- /home/matteo/Documents/prog/rust/nsz-rs/AGENTS.md
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/CONTINUITY.md
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/PLANS.md
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/execplans/INDEX.md
- /home/matteo/Documents/prog/rust/nsz-rs/.agent/execplans/active/EP-2026-02-22__nsz-rs-parity.md
- /home/matteo/Documents/prog/rust/nsz-rs/docs/plans/2026-02-22-nsz-rs-parity-design.md
- /home/matteo/Documents/prog/rust/nsz-rs/docs/plans/2026-02-22-nsz-rs-parity-implementation.md
- /home/matteo/Documents/switch_games/Bad Cheese [NSP]
- /home/matteo/Documents/prog/python/nsz/nsz/__init__.py
- /home/matteo/Documents/prog/python/nsz/nsz/ParseArguments.py
- /home/matteo/Documents/prog/python/nsz/nsz/NszDecompressor.py
- /home/matteo/Documents/prog/python/nsz/nsz/nut/Keys.py

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
