---
doc_class: Spec
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Define how every oyatie-* crate is versioned: SemVer 2.0.0 mandatory, workspace
  lockstep until W-Foundry-Preview, independent thereafter, with cargo-semver-checks
  enforcement in CI and a breaking-change ADR gate.
planned_enforcement_ref: governance-semver-discipline
related_adrs: [ADR-0041, ADR-0050]
doc_status: published
---

# Crate Versioning Spec — oyatie

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Rule of law

Every public crate (anything publishable to a registry OR depended on across
crate boundaries) MUST follow **SemVer 2.0.0** as defined at semver.org.

Per Rust Cargo SemVer reference: a breaking change is any change that requires
a downstream consumer to alter source code in order to compile against the new
version. cargo-semver-checks enforces this mechanically.

## 2. Workspace inheritance (the two phases)

### 2.1 Phase A — lockstep (now → W-Foundry-Preview)

`Cargo.toml` workspace root:

```toml
[workspace.package]
version = "0.X.Y"   # all oyatie-* crates inherit
```

All crates publish together; the version moves together. This eliminates the
combinatorial skew problem during pre-GA.

### 2.2 Phase B — independent (post W-Foundry-Preview)

Per-crate `version = "X.Y.Z"`. Crates may bump on their own cadence once:
- `governance-semver-discipline` is green for 60 days across the crate.
- A per-crate `CHANGELOG.md` exists.
- The crate has shipped at least one `1.0.0` release.

## 3. Bump rules (per SemVer 2.0.0)

| Change class | Bump | Examples |
|---|---|---|
| Add a `pub` item | MINOR | new function, struct, trait, enum variant |
| Remove a `pub` item | MAJOR | drop a function, rename a struct |
| Change a `pub` signature | MAJOR | change argument type, add required argument |
| Add a method to a `pub trait` (no default) | MAJOR | breaks downstream impls |
| Add a method to a `pub trait` (with default) | MINOR | downstream impls unaffected |
| Bug fix with no API change | PATCH | internal logic only |
| Documentation / comments | PATCH | no code change |

These rows are the contract the `cargo-semver-checks` lint runs against.

## 4. Pre-release labels

Per SemVer §9. Order: `alpha.N` < `beta.N` < `rc.N` < `<release>`.

- `0.X.Y-alpha.N` — actively iterating; no SemVer promise.
- `0.X.Y-beta.N` — feature-complete; bug fixes only; ≤ 1 breaking change per N.
- `0.X.Y-rc.N` — release candidate; no breaking changes from rc.1 forward.
- `0.X.Y-dev-snapshot.YYYYMMDD` — origin/dev branch builds; NOT published.

## 5. `cargo-semver-checks` CI integration

Lane: [`governance-semver-discipline`](enforcement-lanes.md) (BLOCKER).

```bash
# in CI per crate
cargo semver-checks check-release --baseline-rev origin/prod
```

Behavior:
- `deny` level (per cargo-semver-checks): hard error → CI fails → PR blocked.
- Override allowed ONLY via the breaking-change ADR (see §6).
- Runs on every PR that touches `crates/oyatie-*/src/**`.

Per research: accidental SemVer violations happen in ~3% of releases; this
lane catches them mechanically.

## 6. Breaking-change ADR template

If the lane reports a SemVer violation, the PR author has two paths:

1. **Fix the violation**: revert the API change, ship the additive equivalent.
2. **Embrace the major bump**: open an ADR under
   `/templates/ADR-BREAKING-CHANGE.md` with:
   - Frontmatter `breaking_change: true`.
   - Justification (data-shape change, security, regulatory).
   - Migration path for downstream consumers.
   - Supersession note linking to the previous ADR(s).
   - 180-day sunset entry in `docs/release/SUNSET-LEDGER.md`.

The `change-class-reviewer` agent and `api-stability-reviewer` agent must both
approve. Without both, the PR is blocked.

## 7. Linus discipline (reject versioning ceremony)

A version bump must reflect a **real data-shape change**. The following are NOT
acceptable triggers:

- Refactors that don't cross a crate boundary.
- Documentation-only edits.
- Internal renames of non-`pub` items.
- "Vibes" releases without an underlying change.

Per Directive Linus: if it doesn't show up in the SemVer surface,
cargo-semver-checks correctly says nothing happened. Honour that.

## 8. Cross-crate skew constraint

Even in Phase B (independent cadence), the workspace asserts:
- All `platform-*` crates share `major` (compatibility kernel layer).
- All `foundry-*` crates share `major` (control-plane kernel layer).
- All `oyatie-{axis}-*` crates may diverge per-axis (axis-prefix layer).

This is the same shape Kubernetes uses for version skew across kube components
(client ≤ control plane ≤ nodes within one minor).

## 9. Publishing flow

`origin/prod` tag `vX.Y.0` → `release-cherry-pick` agent cuts
`release/X.Y` → crate `Cargo.toml` set to `X.Y.0` → `cargo publish` per crate.

`crates.io` (or internal registry) publish requires the lane to be green AND
the tag to exist.

## 10. Lift target

`oyatie/docs/release/crate-versioning-spec.md` on approval.
