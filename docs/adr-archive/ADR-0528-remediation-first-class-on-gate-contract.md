---
id: ADR-0528
title: "Remediation is first-class on the gate contract: remediate() ships WITH every gate (pure, returns described edits, never writes); amends the semver'd oya-ci-gate-contract surface (ADR-0515 WS-D)"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-700]
depends_on: [ADR-0515]
amends: [ADR-0515]
related: [ADR-0515, ADR-0516, ADR-0519, ADR-0529, ADR-0530, ADR-0531, ADR-0534]
related_specs:
  - /specs/masterplan.json
milestone: W2
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0528: Remediation is first-class on the gate contract

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Detail under Component 2 of ADR-0516. **Amends the semver'd `oya-ci-gate-contract` surface**
(ADR-0515 WS-D) — a deliberate one-way change. (Gate-contract crate name RATIFIED at this door:
`oya-ci-gate-contract`; the third-party SDK is `oya-ci-gate-sdk`, ADR-0534.)

## Context

The founder automation-first directive is "automation, auto-fix, auto-generation come first … then
enforcement." Today detection (`evaluate_keyed`) and any fix are separate, so detector and fixer drift,
and "comes-first" is a convention rather than a structural requirement.

## Decision

Extend the published, semver'd gate-trait crate `oya-ci-gate-contract` (the home of `Finding` /
`evaluate_keyed`, per ADR-0515 WS-D) so every gate carries a remediation sibling to detection:

```text
fn remediate(&self, finding: &Finding, face: &Value) -> Remediation
where Remediation = AutoFix(Edit) | AutoGenerate(NewFile) | None
```

`remediate()` is as PURE as `evaluate_keyed` — it returns a *described* edit (path, byte-range,
replacement) or (path, body) and NEVER writes; write capability stays out of the gate entirely
(sandboxed, default-deny, no-I/O). Gate registration gains a structural assertion: every (gate, code)
MUST declare a remediation tier (`auto-fix` | `auto-generate` | `block:<rationale>`); a code with
neither an AutoFix/AutoGenerate nor an explicit `None`-with-rationale FAILS registration. The
disposition table (`gate-disposition.json`, DATA) gains a `remediation` field. The platform default
inverts: a violation produces `Finding` + `remediate()`; if AutoFix/AutoGenerate, the fix is PROPOSED
first; block-and-surface (today's `Finding` + firewall RED, ADR-0515) is the explicit Tier-4 FALLBACK
only for `Remediation::None`. AUTO-FIX is fenced to edits provably behavior-preserving on a falsifiable
signal (manifest fields, declarative config, renames-with-full-reference-update, rustfmt/clippy --fix);
anything that changes runtime SEMANTICS is `None`/advisory and never an applied AutoFix.

## Governed surfaces

The REMED-001 implementation of this gate-contract decision adds the first-party
`oya-ci-gate-contract` crate and its review evidence. The governed, justified surfaces are:

- `libs/oya-ci-gate-contract/BUCK`
- `libs/oya-ci-gate-contract/Cargo.toml`
- `libs/oya-ci-gate-contract/OWNERS`
- `libs/oya-ci-gate-contract/src/lib.rs`
- `evidence/multispectrum/remed-001-gate-contract-20260701-1782944808.json`

## Drivers

- The founder automation-first directive ("comes-first").
- Making remediation a first-class trait member (vs an out-of-band tool) prevents detector/fixer drift
  and makes "comes-first" a structural requirement.
- Keeping `remediate()` pure preserves the small auditable trusted base (the firewall's two pure
  predicates stay literally unedited).

## Alternatives considered

- **(B) remediation as a separate out-of-band tool** — rejected: violates "comes-first" and drifts
  from the detector.
- **(C) bot applies fixes directly to main** — rejected: silent unreviewed mutation, bypasses the
  merge gate.
- **(D) `remediate()` returns *applied* edits** — rejected: puts write capability into the sandboxed
  gate; described edits chosen instead.

## Consequences

Mutates the semver'd `oya-ci-gate-contract` surface (a deliberate one-way change); every gate must
author + test a remediation; the four-tier model (AUTOMATE → AUTO-FIX → AUTO-GENERATE →
BLOCK-AND-SURFACE) becomes the platform's enforced priority order; idempotence is a gate invariant
(re-running `remediate()` on the post-fix face MUST return `None`). The applied-edit delivery is owned
by ADR-0531. **Amends ADR-0515 (WS-D gate contract).** door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source:
AUTOMATED-QUALITY-ENFORCEMENT-AND-AUTOREMEDIATION-ARCHITECTURE.md (RATIFY-TO-ADR). Amends ADR-0515
WS-D. Detail under Component 2 of ADR-0516.*
