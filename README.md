# nsz-rs

`nsz-rs` is a native, safe Rust implementation of `nsz` behavior, pinned to Python `nsz` `4.6.1` parity.

The project has two deliverables:

- a Rust library (`nsz_rs`) for direct native integration
- a Python package (`nsz`) backed by the Rust core for compatibility-oriented migration

No GUI is included.

## Why this project exists

The main goal is strict behavior and byte-output parity with upstream `nsz 4.6.1`, with Rust safety and maintainability.

Performance-wise, compression improvements are often modest because both implementations are mostly bounded by `zstd` work in high-compression modes. In practice, this project still provides a cleaner native core and was also a way to explore and harden a Rust implementation while keeping parity constraints intact.

## Current support

Core native paths:

- Compression: `.nsp -> .nsz`, `.xci -> .xcz`, `.nca -> .ncz`
- Decompression: `.nsz -> .nsp`, `.xcz -> .xci`, `.ncz -> .nca`
- Verification: `.nsp`, `.nsz`, `.xci`, `.xcz`, `.nca`, `.ncz`
- Additional ops: `extract`, `create`, `titlekeys`, `undupe` (compatibility surfaces are exposed)

Parity target:

- Upstream reference: Python `nsz` release `4.6.1`
- Requirement: 1:1 byte output parity for covered flows

## Performance note

On the same inputs with high compression settings, observed behavior is:

- Compression: usually close to Python baseline (often near parity, occasionally small wins/losses)
- Decompression: typically materially faster in Rust due to native container processing and reduced Python overhead

Example high-level benchmark snapshot (level 20, same corpus/input):

- Compress: Python `87903 ms`, Rust `91994 ms` (`0.956x`)
- Decompress: Python `4463 ms`, Rust `1169 ms` (`3.816x`)

## Rust usage

Add as a library and call high-level operations via request structs.

```rust
use std::path::PathBuf;
use nsz_rs::{compress, CompressRequest};

let request = CompressRequest {
    files: vec![PathBuf::from("/path/to/game.nsp")],
    output_dir: Some(PathBuf::from("/tmp/out")),
    ..Default::default()
};

let report = compress(&request)?;
println!("processed: {}", report.processed_files.len());
```

## Python compatibility layer

This repository ships a Python package named `nsz` backed by the Rust core.

### Build and install (editable)

```bash
pip install maturin
maturin develop --features python
```

### Use as module

```python
import nsz

nsz.main(["-C", "-o", "/tmp/out", "/path/to/game.nsp"])
```

### Use as script

```bash
nsz -C -o /tmp/out /path/to/game.nsp
python nsz.py -C -o /tmp/out /path/to/game.nsp
```

Compatibility API surface currently provided:

- `compress`
- `decompress`
- `verify`
- `extract`
- `create`
- `titlekeys`
- `undupe_files`
- `main`

This compatibility layer is intended for high-level drop-in use (CLI + common module calls). It is not a full clone of every internal upstream Python module.

## Keys and environment

For encrypted content workflows, provide valid keys (for example `keys.txt`/`prod.keys`) in expected locations used by your flow.

If keys are missing or incomplete, operations involving encrypted NCAs/XCI partitions can fail.

## Verification commands

```bash
cargo fmt --all
cargo test -q
cargo clippy --all-targets -- -D warnings
python3 -m py_compile python/nsz/__init__.py python/nsz/__main__.py python/nsz/ParseArguments.py nsz.py
```

## CI policy

GitHub Actions runs formatting, linting, build checks, Python-feature compilation, and a fast Rust test suite.

Game-fixture parity and benchmark tests are intentionally excluded from CI and remain opt-in via environment flags:

- `NSZ_RUN_HEAVY_PARITY=1`
- `NSZ_RUN_HEAVY_COMPRESS_PARITY=1`
- `NSZ_RUN_HEAVY_XCI_COMPRESS_PARITY=1`
- `NSZ_RUN_HEAVY_MISC_PARITY=1`
- `NSZ_RUN_HEAVY_SPEED_BENCH=1`
