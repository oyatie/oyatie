---
id: ADR-0524
title: "Kernel buckification + additive-first verify parity; pinned QEMU/musl toolchain; tracked out/*.elf blob retirement; extends ADR-0392 to the kernel it never scoped"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0392, ADR-0522, ADR-0523]
amends: [ADR-0392]
related: [ADR-0392, ADR-0515, ADR-0516, ADR-0522, ADR-0523]
related_specs:
  - /specs/cloud-toolchain-target.json
  - /specs/masterplan.json
  - /.omc/specs/deep-interview-agentic-delivery-fabric.md
milestone: W1
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0524: Kernel buckification + additive-first verify parity

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

**Extends ADR-0392** to the kernel side it never scoped (ADR-0392 scoped only the cloud build graph).
Detail under Component 1 of ADR-0516. The founder may DEFER this ADR pending the reproducibility proof
(OQ-10, ADR-0521) — the apex + lifecycle ADRs are independently ratifiable.

## Context

The cloud is ~70% buck2-native (ADR-0392) but the kernel port is ~5% (100% cargo + shell, today zero
non-`buck-out` BUCK targets). The kernel is therefore the largest single net-new buckification lift,
and it carries a HIGH-severity hermeticity violation: git-TRACKED `out/*.elf` carrier blobs. ADR-0392
never scoped the kernel; ADR-0522's one-graph posture requires it to be brought in.

## Decision

Bring the kernel port (`stack/kernel`) into the one buck2 graph (ADR-0522) in dependency order, with
verify ADDITIVE-FIRST so the currently-green cargo + QEMU goal-ladder floor is never at risk.

(a) **CARRIERS become buck2 outputs** — the per-crate `.cargo/config.toml` rustflags map to
`rustc_flags` + a linker-script `srcs` dep; the `build.sh` tmpdir-escape hack (which exists only
because cargo concatenates parent `.cargo` rustflags) EVAPORATES under buck2 (a case where buck2 is
strictly cleaner than cargo+bash). This retires `build.sh` × N AND the git-tracked `out/*.elf` carrier
blobs (they become build outputs never committed).

(b) **The KERNEL IMAGE becomes a buck2 target** embedding carrier outputs via
`$(location //...:carrier)` (not `include_bytes!` of tracked blobs).

(c) **QEMU BOOT + the verify gates** (run-loom, assert-smp-boot, diff-oracle, check-tcb,
conformance-probe) become `rust_test` targets over a PINNED QEMU toolchain, replacing the bash
`run-qemu-*.sh` runner wiring; oracle / trace-extraction `awk`/`grep` pipelines become Rust harness
code.

**GATING PREREQ:** pin QEMU + musl-std as sha256 `download_file` toolchain artifacts FIRST and PROVE
boot-verdict reproducibility across dev (macOS) and CI (ubuntu) host classes BEFORE retiring any bash
verify gate — the single biggest kernel-side technical risk (fallback = a minimal Nix flake for
QEMU/talosctl only, per ADR-0523).

## Drivers

- The asymmetry headline: cloud is ~70% buck2-native, the kernel ~5%, so kernel buckification is the
  largest single net-new lift.
- The hermeticity-doctrine HIGH-severity violation of tracked `out/*.elf` blobs.

## Alternatives considered

- **Leave the kernel on cargo+bash permanently** (the steelman antithesis — already substantially
  hermetic via `rust-toolchain.toml`-pinned nightly + reproducible Stage A/B carriers) —
  DEFERRED-NOT-ADOPTED: viable only if the founder wants "hermetic by any means" rather than the
  one-substrate posture of ADR-0522. This is the OQ-10 defer path.
- **Big-bang rip-and-replace of the verify gates** — rejected: breaks the verified floor mid-flight
  while P4·SMP work is in a worktree.

## Consequences

The buck2 kernel targets run ALONGSIDE the cargo + bash gates (exactly as the cloud buck2 lane runs
alongside the cargo lanes today) with byte/verdict-parity proof; the bash gates retire ONLY after the
buck2 equivalents are proven green over the SAME verdicts (never retire a bridge before its replacement
is proven). The prebuilt external Alpine kernel + `.ko` blobs pulled by `fetch-kernel.sh` are flagged
as a SEPARATE HIGH-severity external-blob debt requiring its own build-from-source-vs-vendored-pinned-
hash decision (out of this ADR's scope, recorded as debt). **Extends ADR-0392** to the kernel.
door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source:
LIFECYCLE-HERMETICITY-ZERO-SHELL-ARCHITECTURE.md (RATIFY-TO-ADR). Extends ADR-0392 to the kernel;
may be DEFERRED per OQ-10 (ADR-0521).*
