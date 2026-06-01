---
id: ADR-0488
title: "Linker layer — mold (cargo-era gnu-host interim), lld/rust-lld (buck2 + musl self-contained), wild + ld-prime (hermetic-cell destination)"
status: Accepted
date: 2026-05-31
owner: council-architecture
planning_impact: true
supersedes: []
superseded_by: []
related:
  - ADR-0083
  - ADR-0514
---

# ADR-0488 — Linker layer: mold / lld / wild + ld-prime

## Status

Accepted — 2026-05-31.

Records the **in-use, per-context linker decisions** across the cargo-era, buck2, musl-static, and
hermetic-toolchain-cell surfaces. This ADR resolves the phantom citation in
`docs/research/mold-linker-impl-2026-05-28.md` and `.cargo/config.toml`, and is cited by ADR-0514.

---

## Context

Linking is not a single choice: the correct linker depends on the **build system**, **host triple**,
**target triple**, and **phase of the toolchain roadmap**. A single "pick mold everywhere" answer is
wrong; the decision is layered.

### Layers in scope

| Layer | Build system | Host | Target | Current state |
|---|---|---|---|---|
| cargo-era gnu-host | cargo | x86_64/aarch64-linux-gnu | same | **mold** (interim, via `.cargo/config.toml`) |
| cargo macOS host | cargo | aarch64/x86_64-darwin | same | rustc default (lld via clang driver since 1.74) — **no override** |
| buck2 C/C++ link | buck2 | any | linux | **lld** (LLVM; bundled with the buck2 toolchain cell) |
| buck2 Rust link | buck2 | any | linux | **rust-lld** (shipped with the Rust toolchain) |
| musl static target | cargo or buck2 | any | aarch64/x86_64-unknown-linux-musl | **rust-lld self-contained** (`-C link-self-contained=yes -C linker=rust-lld`) |
| hermetic cell (destination) | buck2 | aarch64-darwin | darwin | **ld-prime** (Apple's linker, hermetic cell) — per issue #83 |
| hermetic cell (destination) | buck2 | aarch64-linux | linux | **wild** (bespoke Rust linker, hermetic cell) — per issue #83 |

### Why mold for the cargo-era gnu-host?

The `.cargo/config.toml` change (cross-ref: `docs/research/mold-linker-impl-2026-05-28.md`) sets:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

Rationale: during the cargo-era (pre-buck2 full adoption) Linux CI agents link with cargo. mold is
the fastest available system linker for gnu-linux hosts (5–20% warm-incremental improvement per
upstream benchmarks; large multi-crate workspaces see the largest gains). It is open-source, actively
maintained (≥ 2.0 required), and available in Debian/Alpine without non-standard repositories.

macOS is intentionally untouched: rustc has used lld via the clang driver as its default since 1.74;
no override is needed or desired.

### Why lld / rust-lld for buck2?

Buck2 ships its own LLVM-based toolchain cell. C/C++ targets in the prelude use lld by default;
Rust targets use rust-lld (shipped with the Rust toolchain sysroot). Both are hermetic — no host
linker is required or consulted. This is the correct choice for all buck2-built targets on Linux.

### Why rust-lld self-contained for the musl static target?

The `aarch64-unknown-linux-musl` (and `x86_64-unknown-linux-musl`) targets produce fully static
binaries. mold does not support musl linking without additional glue. The canonical flags are:

```
RUSTFLAGS="-C link-self-contained=yes -C linker=rust-lld"
```

`link-self-contained=yes` instructs rustc to use the compiler-rt CRT objects from the sysroot,
and `linker=rust-lld` selects the bundled lld. No host linker toolchain dependency is introduced.
mold MUST NOT be used for the musl target — the `.cargo/config.toml` gnu-host blocks do not match
the musl triple and therefore do not interfere.

### Why wild + ld-prime as the hermetic destination?

The build-platform-optimization doctrine (hyperscaler-lens filter applied to the linker layer)
requires:

- **Active upstream** — both wild (bespoke Rust linker) and ld-prime (Apple's shipped linker) satisfy
  this.
- **Self-hostable** — wild is pure-Rust, buildable from source in the buck2 toolchain cell; ld-prime
  ships with Xcode/Command Line Tools.
- **Hermetic** — the toolchain cell owns the linker binary; no host linker is consulted at build time.

Issue #83 tracks the toolchain-cell migration that replaces system lld/mold with wild (Linux) and
ld-prime (macOS) as the hermetic linkers baked into the cell. This is the **destination**, not the
current state.

---

## Decision

The linker choice is **per-context**:

1. **cargo-era gnu-host (x86_64-linux-gnu, aarch64-linux-gnu):** mold ≥ 2.0 via clang driver, as
   pinned in `.cargo/config.toml`. This is the **interim** choice for the cargo build surface. It is
   explicitly NOT the long-term destination.

2. **cargo macOS host (aarch64-darwin, x86_64-darwin):** no override; rustc's default lld-via-clang
   is correct and sufficient. No `.cargo/config.toml` block for darwin targets.

3. **buck2 C/C++ links (Linux):** lld, provided by the buck2 toolchain cell. No action required.

4. **buck2 Rust links (Linux):** rust-lld, shipped with the Rust toolchain sysroot. No action
   required.

5. **musl static target (aarch64/x86_64-unknown-linux-musl):** rust-lld self-contained
   (`-C link-self-contained=yes -C linker=rust-lld`). mold is NOT used for this target.

6. **Hermetic toolchain-cell destination (Linux):** wild — tracked under issue #83; not yet active.

7. **Hermetic toolchain-cell destination (macOS):** ld-prime — tracked under issue #83; not yet
   active.

mold is classified as **transitory cargo-host infrastructure**. It will be retired for Linux gnu-host
builds when the hermetic cell (wild) is activated under issue #83. The `.cargo/config.toml` blocks
will be removed at that point.

---

## Consequences

### Positive

- Fast incremental links on Linux cargo-era CI agents with zero changes to CI agent provisioning
  beyond `apt install mold` / `apk add mold`.
- musl static builds remain self-contained and reproducible; no system linker dependency.
- buck2 toolchain-cell builds are fully hermetic today via lld/rust-lld.
- The wild + ld-prime destination is clearly documented and tracked, avoiding future confusion about
  whether mold is the intended end-state.

### Negative / Risks

- CI agents must have mold ≥ 2.0 installed; missing package → link failure on gnu-host cargo builds.
  Mitigation: provisioning scripts check `mold --version` at bootstrap.
- mold does not support LTO as well as lld; if full LTO is enabled for release binaries on gnu-host
  (cargo-era), mold may need to be bypassed. Mitigation: release profiles can override linker flags.
- wild is not yet production-validated at this project's scale; the hermetic-cell migration (issue
  #83) must include link-correctness and performance benchmarks before activation.

### Neutral

- This ADR records a decision already implemented (`.cargo/config.toml` landed with the mold
  research). No code change is introduced by this ADR itself.

---

## Verification

After this ADR is merged, `oya gate validate adr-citation` resolves the ADR-0488 citation in
`docs/research/mold-linker-impl-2026-05-28.md` and `.cargo/config.toml` (the comment on line 1 of
that file references `ADR-0488`). The gate previously failed with a missing-ADR finding; it now
passes.

---

## References

- `docs/research/mold-linker-impl-2026-05-28.md` — implementation notes for the mold `.cargo/config.toml` change
- `.cargo/config.toml` — active linker configuration
- ADR-0083 — Rust error-handling tier (companion governance pattern; not a linker dependency)
- ADR-0514 — cites this ADR
- Issue #83 — hermetic toolchain-cell linker migration (wild + ld-prime destination)
- [mold upstream](https://github.com/rui314/mold) — open-source high-speed linker
- [wild upstream](https://github.com/davidlattimore/wild) — bespoke Rust linker
