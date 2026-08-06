---
id: ADR-0605
title: "Supply-chain audit gate (owned RustSec advisory scan over a vendored mirror)"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-06-28
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
amended_by: []
depends_on: [ADR-0083, ADR-0510, ADR-0515, ADR-0535, ADR-0547, ADR-0548, ADR-0566]
amends: []
related: [ADR-0535, ADR-0547, ADR-0548, ADR-0566]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0605: Supply-chain audit gate (owned RustSec advisory scan over a vendored mirror)

## Status

**Proposed - 2026-06-28 (authored for founder sign-off; door: one-way).**

## Context

PR #974 added a born-blocking supply-chain vulnerability gate by shelling out to `cargo-audit` /
`cargo-deny`. That is shell plus a network-fetching RustSec advisory index — both forbidden by the
no-shell doctrine, the hermetic-gate / pipeline-as-product bar (ADR-0548: every gate must be a pure,
deterministic, buck2-cacheable predicate), and rust-purity (zero non-Rust). #977 reverted #974,
keeping only the quinn-proto CVE fix (`quinn-proto 0.11.15`) and restoring the pre-#974 `deny.toml`.

The supply-chain coverage gap is real and must be re-closed — owned. The naive owned path (the
`rustsec` + `cargo-lock` crates) pulls `git2` → `libgit2-sys` (a C dependency, a rust-purity strike)
and a network index (defeats hermeticity), so it is disqualified for the gate (reserved only as a
behind-a-port adapter if advisory-format fidelity ever proves insufficient). The RustSec advisory
format is a small, stable TOML-front-matter `.md` (`rustsec/advisory-db` `EXAMPLE_ADVISORY.md` /
`README.md`), so an owned `toml`-only parser over a VENDORED snapshot is sufficient — and `semver`
(the only requirement-matching dependency) is already locked transitively, so promoting it to a
direct dep adds ZERO new crate to `Cargo.lock`.

## Decision

Ship a **self-contained cloud-ci gate**, `cloud-ci-supply-chain-audit`
(`ci/facade/supply-chain-audit`), mirroring the kernel-purity (ADR-0547)
/ authz-coverage (ADR-0566) registration footprint: own crate, own policy JSON, one appended matrix
line in `.github/workflows/oya-ci-required.yml`, no `libs/oya-ci-config` edit, no producer-face
binding. The advisory parsing/normalization lives in a reusable pure kernel,
`libs/oya-advisory-mirror-kernel`.

The gate's neutral Rust engine lives in
`ci/facade/supply-chain-audit/src/lib.rs` (the I/O collector
`collect(root, policy)` + the pure `evaluate_keyed(policy, observed)`), the binary in `src/main.rs`,
the live-corpus + RED/GREEN self-test in `tests/supply_chain_audit.rs`, the buck2 wiring in `BUCK`,
the manifest in `Cargo.toml`, the vendored snapshot in `advisory-mirror/{advisories.json,
mirror-manifest.json}`, and ALL repo-specifics as DATA in `supply-chain-audit-policy.json`.

### D1 — Owned parser, not the rustsec/cargo-lock crates

`libs/oya-advisory-mirror-kernel` is a pure, I/O-free distiller: `distill(&[String]) -> Vec<Advisory>`
parses each advisory's fenced TOML front matter (`id`, `package`, `[versions] patched`/`unaffected`,
`informational`) into a normalized record, dropping WITHDRAWN (retracted) advisories. `canonical_hash`
is a deterministic, order-independent content hash of the distilled set. Dependencies: `serde` +
`serde_json` + `toml` only — no `rustsec`/`git2`/`libgit2`. The matching dependency `semver` is
promoted from a transitive-only lock entry (already `1.0.28` via cedar-policy/rustc_version/wasmparser)
to a direct workspace dep via one `reindeer buckify`, emitting the `third-party//:semver` buck alias
with ZERO new crate entering `Cargo.lock`.

### D2 — Vendored, content-addressed mirror

A data-gen producer (`oya-advisory-mirror-producer`, a `rust_binary` in the kernel crate) reads a
PINNED `rustsec/advisory-db` checkout and writes the vendored snapshot. The producer is pure file
I/O: the pinned commit and last-sync date are passed as CLI args (no subprocess, no clock, no
network in Rust). `mirror-manifest.json` records `{ schema, source:{repo,commit,last_sync},
content_hash, advisory_count }`. The first snapshot pins commit
`e69927bf37afb7707575bc95aff6a9ef3f2534fb` (last_sync 2026-06-26), 1106 advisories.

### D3 — Matching (the verdict)

For each advisory whose `package` matches a locked crate `name`, the locked `semver::Version` is
AFFECTED iff it satisfies NO `patched` and NO `unaffected` `semver::VersionReq`. An affected advisory
blocks unless its id is in `policy.ignore[]`:
- a security vulnerability (no `informational`) → `SCA-VULN`.
- an `informational = "unmaintained"` advisory → `SCA-UNMAINTAINED`, gated by
  `policy.unmaintained_policy == "all"`.
- `informational = "unsound" | "notice"` are tracked but OUT of this gate's blocking scope (a
  documented policy extension point).

### D4 — Time-boxed ignore[] (moved from deny.toml), shrink-only

The reverted #974 `deny.toml` carried three time-boxed unmaintained ignores
(`RUSTSEC-2025-0052` async-std, `RUSTSEC-2024-0436` paste, `RUSTSEC-2026-0173` proc-macro-error2,
remove-by 2026-12-31) and the `unmaintained = "all"` posture. Both move into the gate's
`policy.json` (the new blocking home): each ignore carries a `reason`, `pull_chain`, and `remove_by`.
The ignore list is **shrink-only**: `SCA-STALE-IGNORE` flags an ignore that suppresses no live
affected advisory, and the gate binary `--write` drops stale entries — it NEVER adds an ignore (a
new affected advisory must be FIXED, not auto-baselined). The CI matrix runs the gate's `rust_test`
legs (a verdict assertion), not the binary with `--write`, so no automation can grow the ignore list.

### D5 — Mirror integrity, fail-closed

`collect` is read-only and no-walk. For the structured multi-lock policy it first consumes the
materialized `oya-ci/scm-facts/v2` tracked-path snapshot (the pipeline's single out-of-graph SCM
boundary) and independently projects every workspace-owned lock: a tracked `Cargo.lock` is owned
exactly when its tracked sibling `Cargo.toml` declares top-level `[workspace]`. The sorted projection
must equal `policy.lockfile_corpus`. A newly tracked fifth workspace lock therefore fails until the
policy explicitly declares it; tracked package/member-local locks and orphan locks without a
tracked sibling manifest do not expand the scan. The legacy one-lock `lockfile_path` form remains
supported without this multi-workspace projection.

After totality is proven, `collect` parses every declared `Cargo.lock` (TOML) and the vendored
`advisory-mirror/{advisories.json,mirror-manifest.json}` (JSON) — never shells out, walks the runtime
filesystem, or touches the network/clock/git. Every configured lock must contain at least one
`[[package]]` row, and every row must be a table with non-empty string `name` and `version`; malformed
rows fail collection rather than disappearing from the advisory scan. To preserve legacy consumers,
the gate still emits a compact `{name, version}` `locked` projection, and adds a deterministic
`locked_by_source` projection (`{name, version, lockfile}`) so every finding can be attributed to the
exact workspace lockfile and version.
The mirror is
integrity-checked against a vacuously-green truncated snapshot:
`SCA-MIRROR-MALFORMED` fires when the manifest `content_hash` ≠ the recomputed `canonical_hash` of
`advisories.json`, OR the manifest `advisory_count` ≠ the actual record count, OR the payload is
missing/non-array; `SCA-MIRROR-UNDERFLOW` fires below the `min_advisories` floor. Fail-closed
matching: an unparseable locked version or advisory `VersionReq` is treated as affected.
`SCA-POLICY-GATE-ID-MISMATCH` / `SCA-POLICY-MALFORMED` fail closed on a corrupt policy.
`#![forbid(unsafe_code)]`; deterministic sorted output. The content hash is a SHA-256 collision-resistant integrity anchor that detects accidental
corruption, a desynced regeneration, and provides a tamper-evident content seal verified
fail-closed by the gate at every run.

### D6 — Network/clock split (deferred reconciler)

The network/clock-bearing half — pinning a fresh advisory-db commit, distilling, opening a GitOps PR,
and a staleness SLO + `ignore[].remove_by` expiry detector — lives in a SEPARATE owned reconciler
(CronJob/operator CRD, NOT shell cron), deferred to a follow-on. Keeping it OUT of the gate is what
makes the gate hermetic and buck2-cacheable. The follow-on also retires
`libs/oya-check-dependency-seam`'s `Command`-spawned `cargo-audit` and demotes the `deny.toml`
`[advisories]` section from blocking.

Implementation guardrail (2026-06-28): the supply-chain-audit gate and its advisory-mirror kernel
are born with the following tracked surfaces —
`ci/facade/supply-chain-audit/BUCK`,
`ci/facade/supply-chain-audit/Cargo.toml`,
`ci/facade/supply-chain-audit/OWNERS`,
`ci/facade/supply-chain-audit/src/lib.rs`,
`ci/facade/supply-chain-audit/src/main.rs`,
`ci/facade/supply-chain-audit/supply-chain-audit-policy.json`,
`ci/facade/supply-chain-audit/tests/supply_chain_audit.rs`,
`ci/facade/supply-chain-audit/advisory-mirror/advisories.json`,
`ci/facade/supply-chain-audit/advisory-mirror/mirror-manifest.json`,
`libs/oya-advisory-mirror-kernel/BUCK`,
`libs/oya-advisory-mirror-kernel/Cargo.toml`,
`libs/oya-advisory-mirror-kernel/OWNERS`,
`libs/oya-advisory-mirror-kernel/src/lib.rs`,
`libs/oya-advisory-mirror-kernel/src/main.rs`, and
`libs/oya-advisory-mirror-kernel/tests/distill_red_fixture.rs`.
All are owned by `cloud-ci-platform` (OWNERS files in each crate root) and reachable via
`cargo-members` (workspace globs `cloud/cloud-ci/gates/*` and `libs/oya-*`).

## Consequences

- A NEW locked crate affected by a security advisory fails the `cloud-ci-supply-chain-audit` lane as
  `SCA-VULN`; a new unmaintained dep fails as `SCA-UNMAINTAINED` (policy=all). The gate lands
  born-blocking GREEN today: quinn-proto is on the patched `0.11.15`, and the only other live
  affected advisories are the three unmaintained ids in `policy.ignore[]`.
- The gate is hermetic: `serde_json` + `toml` + `semver` only, no `rustsec`/`git2`/`libgit2`, no
  shell/network/clock or runtime tree walk. The materialized SCM-facts snapshot supplies the
  deterministic tracked-file boundary; exact workspace-lock projection and strict package-row
  parsing prevent corpus omission and silent dependency disappearance. The only `Cargo.lock`
  additions are the two new owned crates plus the zero-cost `semver` promotion.
- A truncated or desynced mirror cannot make the gate vacuously green (integrity + floor fail closed).
- The advisory snapshot must be periodically revendored; until the Slice-D reconciler ships, that is
  a manual producer run against a freshly-pinned advisory-db commit (the `remove_by` dates bound the
  manual cadence).

## Alternatives considered

- **The `rustsec` / `cargo-lock` crates**: rejected for the gate — they pull `git2` →
  `libgit2-sys` (a C dependency) and a network-fetching index, defeating rust-purity + hermeticity.
  Reserved only as a behind-a-JSON-schema-port adapter if the owned TOML parser's advisory-format
  fidelity ever proves insufficient.
- **Keep the shell `cargo-audit`/`cargo-deny` gate**: rejected — that is exactly what #977 reverted;
  shell + network defeats the no-shell + hermetic-gate doctrine.
- **A producer-face binding** (emit findings into the accounting registry): rejected — the
  accounting-registry producer is oyatie-specific; binding a face would kill R0 portability (same
  rationale as ADR-0547 D1 / ADR-0566).
