# mold linker implementation — 2026-05-28

Cross-ref: ADR-0488

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

- ADR-0488 (`feat/adr-0488-mold-linker-2026-05-28`)
