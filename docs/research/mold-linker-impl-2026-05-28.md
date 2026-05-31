# mold linker implementation — 2026-05-28

Cross-ref: ADR-0392 (Proposed; build-graph/linker-parity input)

## Implementation

Added `.cargo/config.toml` with `[target.x86_64-unknown-linux-gnu]` and
`[target.aarch64-unknown-linux-gnu]` blocks that set `linker = "clang"` and
`rustflags = ["-C", "link-arg=-fuse-ld=mold"]`.

## macOS

No-op. lld has been the rustc default linker driver on macOS since 1.74; no
override is needed or added.

## Expected Linux link-time improvement

5–20% on warm incremental rebuilds per upstream mold benchmarks. Large
workspaces with many crates see the largest gains because mold parallelises the
link step across all available cores.

## Toolchain requirement

Linux CI agents must have `mold` installed:

```
# Debian/Ubuntu
apt install mold

# Alpine
apk add mold
```

Verify with `mold --version` (require ≥ 2.0).

## Cross-reference

- ADR-0392 (Proposed) records that Buck2 and cargo must preserve Rust
  toolchain/linker parity with `.cargo/config.toml`, including `mold + clang`
  where configured.

## ADR status handling

ADR records are interpreted by their frontmatter status and supersession fields,
not by calendar age. The previous pointer named a missing `docs/decisions/` record, so this note
now cites the existing status-bearing record that covers the linker-parity
decision.
