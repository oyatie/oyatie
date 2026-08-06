---
id: ADR-0534
title: "Gate/pipeline-step SDK + gate-artifact marketplace: trait Gate extraction, runtime GateRegistry, three binding kinds (producer-face | external-artifact | wasm-component), higher CI-gate trust bar; reuses docs/standards/plugin-authoring.md"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-0700]
depends_on: [ADR-0516, ADR-0528, ADR-0532]
amends: []
related: [ADR-0516, ADR-0519, ADR-0528, ADR-0529, ADR-0531, ADR-0532, ADR-0533]
related_specs:
  - /specs/bespoke-cloud-toolchain-services.json
  - /specs/masterplan.json
milestone: W3
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0534: Gate/pipeline-step SDK + gate-artifact marketplace

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Detail under Component 3 of ADR-0516. Reuses `docs/standards/plugin-authoring.md`. The third-party SDK
crate is `oya-ci-gate-sdk` (ratified in ADR-0532); the in-tree semver'd contract is
`oya-ci-gate-contract` (ADR-0528).

## Context

Make the pipeline extensible by THIRD PARTIES without forking the producer. Today there is NO
`trait Gate` anywhere; the producer hardcodes a compile-time `match`, and `struct Finding` is COPIED
across the gate crates. A gate is more powerful than a runtime plugin — it blocks/allows merges to a
consumer's main — so the trust bar must be higher.

## Decision

(1) **SDK EXTRACTION (highest-leverage):** extract ONE versioned crate (`oya-ci-gate-sdk`) holding the
gate contract — `trait Gate { gate_id; codes; evaluate_keyed(&FaceValue) -> BTreeSet<Finding> [SSOT];
evaluate() -> Report [provided] }` plus the `Finding`/`Report`/`Verdict` types — replacing the copied
`struct Finding` definitions. Preserve the pure/I-O-free/panic-free shape (`#![forbid(unsafe_code)]`,
no-panic); add a SemVer'd `SDK_ABI_VERSION`. (The in-tree semver'd contract surface stays
`oya-ci-gate-contract`, ADR-0528.)

(2) **RUNTIME GateRegistry:** replace the compile-time `match` in the producer with `gate_id ->
Box<dyn Gate>` (in-tree) / `gate_id -> ResolvedArtifact` (external/wasm), collapsing per-gate
`GateInputs` to a generic `BTreeMap<FaceId, Value>`; the firewall RATCHET needs ZERO change.

(3) **THREE BINDING KINDS:** keep `producer-face` (in-tree Rust, static dispatch) for first-party; add
`external-artifact` (published gate binary speaking JSON face↔findings over a process boundary) and
`wasm-component` (wasm32-wasi-p2 implementing a gate WIT interface in the existing Wasmtime sandbox);
relax the closed schema at ONE point — `[[gates.enabled]]` gains `source='marketplace://<ns>/<gate>@<ver>'`,
a string `face_id`, and an OPEN `[gates.enabled.params]` table validated against the gate's PUBLISHED
param schema.

(4) **MARKETPLACE = REUSE `docs/standards/plugin-authoring.md`** (manifest id + version + trust_tier +
capabilities allowlist + resource_caps + cosign-keyless + Rekor + SBOM + Wasmtime/WASI-P2 sandbox) for
a NEW "gate" artifact class, adding sdk_abi_version/codes[]/face_schema/binding_kind/default_disposition.

## Drivers

- A gate is MORE powerful than a runtime plugin (it blocks/allows merges to a consumer's main).
- The verified copied-`Finding` duplication and the producer's hard wall (the anti-goal).
- The AUTO/ADVISE/GATE safety governor (tier is DATA per finding-code; meta-gate rejects untagged
  codes — ADR-0519/0529).

## Alternatives considered

- **(a) keep editing the producer per gate** — rejected (the verified anti-goal).
- **(b) native-binary-only gates** — rejected (un-sandboxed arbitrary code in the merge path).
- **(c) invent a new marketplace** — rejected (`plugin-authoring.md` already exists; do not reinvent).

## Consequences

HIGHER TRUST BAR: the gate marketplace DEFAULTS higher than runtime plugins — verified-isv only by
default; community gates run ADVISORY-MODE-ONLY until the consumer explicitly promotes them to
baseline-block-on-new; `wasm-component` is the PREFERRED third-party binding (capability-deny
clock/network/random); `external-artifact` reserved for trusted first-party; a gate-conformance harness
(run-twice-diff for determinism + codes-match-manifest + pure/panic-free) gates the verified-isv tier;
cosign + Rekor + SBOM verified before enable. OQ-2 (Rust-engine-with-any-language-input vs first-class
non-Rust gate authoring) carried to founder (ADR-0521). door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: PLATFORM-PRODUCTIZATION-ARCHITECTURE.md
(RATIFY-TO-ADR). Reuses plugin-authoring.md. SDK crate = oya-ci-gate-sdk (ratified). Detail under
Component 3 of ADR-0516.*
