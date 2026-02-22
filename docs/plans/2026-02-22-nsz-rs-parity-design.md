# NSZ Rust Rewrite Parity Design

## Goal

Reimplement Python `nsz` release `4.6.1` (`d84f7c813c3fe278104ff8877803f22028e57452`) as a native safe Rust **library** with strict byte-for-byte output parity for matching inputs and options.

## Scope and Constraints

- Deliverable is a Rust library only. No GUI.
- Public API is a high-level facade for operations:
  - `compress`
  - `decompress`
  - `verify`
  - `extract`
  - `create`
  - `titlekeys`
  - `undupe`
- Byte-level parity requirement is strict 1:1 with Python `4.6.1` behavior.
- Migration strategy is behavior clone first (Approach A), then optimization.
- Canonical parity corpus root is:
  - `/home/matteo/Documents/switch_games/Bad Cheese [NSP]`
- Corpus is extensible; newly added examples must be included by discovery-based parity runs.

## Architecture

The crate exposes a stable high-level facade in `src/lib.rs` and keeps implementation details internal.

Proposed internal module layout:

- `container`: NSP/XCI/PFS0/HFS0 parsing and writing.
- `ncz`: NCZ section/block header handling and stream transforms.
- `crypto`: key loading/validation and AES flow used during pack/depack.
- `compress`: solid and block compression pipelines.
- `verify`: hash and structural verification.
- `fs_ops`: path expansion, overwrite/removal/duplicate semantics, and undupe logic.
- `parity`: Python-vs-Rust baseline runner and byte-diff reporting.

The architecture intentionally prioritizes parity over cleanup so Python quirks that affect bytes are preserved until parity is fully locked.

## Determinism and Byte Parity Controls

Every option that can affect output bytes is explicit in Rust config structs and uses Python-matching defaults:

- compression level
- long distance mode
- solid vs block mode
- block size exponent
- thread counts
- verify flags
- keep/fix-padding behavior
- parseCnmt/alwaysParseCnmt behavior
- overwrite/rm-old-version and target selection behavior

Execution order, partition traversal, header construction, and file write ordering will follow Python `4.6.1` behavior exactly.

## Error Model and Public Contract

Public operations return typed results:

- `Result<T, NszError>` for failures
- structured operation outcomes for processed/skipped files and warnings

`NszError` includes distinct categories for:

- key loading and key validation
- malformed/unsupported containers
- crypto/compression/decompression failures
- verification mismatch
- filesystem policy conflicts
- parity mismatch

Parity mismatch errors include operation context, compared files, hashes, and first differing byte offset.

## Testing and Acceptance Gates

Three test layers are required:

1. Unit tests for parsers, binary encoding/decoding, key derivation, and deterministic option mapping.
2. Integration tests for each operation path.
3. Parity tests that execute Python `4.6.1` and Rust against the canonical corpus and compare outputs byte-for-byte.

Acceptance for each implementation milestone:

- `cargo test` passes
- parity suite passes with zero mismatches on enabled corpus set
- no optimization/refactor merged if parity regresses

## Notes

The parity harness should support both fail-fast and collect-all mismatch modes. Default mode remains to be finalized during implementation planning.
