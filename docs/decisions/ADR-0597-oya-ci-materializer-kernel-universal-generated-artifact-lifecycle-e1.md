---
id: ADR-0597
title: "oya-ci-materializer-kernel (E1): universal generated-artifact lifecycle — pure planner kernel"
status: Accepted
planning_impact: false
deciders: founder
date: 2026-06-23
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: []
depends_on: [ADR-0595, ADR-0596, ADR-0551, ADR-0552, ADR-0523, ADR-0547]
related: [ADR-0515, ADR-0539, ADR-0540, ADR-0541, ADR-0558]
related_specs:
  - /specs/root-hub-pointers.json
  - /registry/generated-artifact-control-plane.json
milestone: W0
---

# ADR-0597: oya-ci-materializer-kernel (E1) — universal generated-artifact lifecycle, pure planner kernel

## Status

**Proposed — 2026-06-23 (E1 slice of the universal generated-artifact lifecycle determination;
door: one-way once E3 repoints the freshness gate onto this kernel).**

## Context

ADR-0595 de-committed the six pure-derivation cloud-ci faces (derive-on-demand). ADR-0596 forbade
re-committing firewall frozen-reference artifacts. Both ship the policy class; neither provides the
**universal engine** that makes the policy repo-agnostic and hermetic. The determination at
`.omc/ultragoal/universal-generated-artifact-lifecycle-determination.md` (with appended adversarial
critique) specifies E1..E6:

> E1 — author the kernel (pure planner + predicate), no behavior change. New crate
> `oya-ci-materializer-kernel`: `plan()`, `evaluate()`, `materialize_closure()`, plus the
> conformance crate GREEN. At this point nothing consumes the kernel yet — additive, gates unaffected.

This ADR records E1 as shipped.

## Decision

Ship `libs/oya-ci-materializer-kernel` — a pure Rust planner + predicate kernel with zero I/O,
zero clock, zero subprocess, zero git, zero net. Dependencies: `serde` + `serde_json` only.

### Public API

```rust
// Pure analysis phase — no filesystem, clock, buck2, git.
pub fn plan(manifest: &ControlPlane, scope: MaterializeScope) -> Result<MaterializePlan, PlanError>;

// Pure verdict predicate — fed materialized bytes by the executor; never materializes.
pub fn evaluate(
    pass_a:    &[(ArtifactId, Bytes)],
    pass_b:    &[(ArtifactId, Bytes)],   // second canary pass; empty for non-canary
    committed: &[(ArtifactId, Bytes)],   // empty for de-commit class
    manifest:  &ControlPlane,            // MF-1: caller supplies merge-base manifest in E3
) -> Findings;

// Derived consumption-order view — universal CI-ordering source.
pub fn materialize_closure(manifest: &ControlPlane, target_paths: &BTreeSet<String>)
    -> Result<MaterializePlan, PlanError>;
```

### Policy contract v2

`registry/generated-artifact-control-plane.json` `schema_version` advances to 2 (additive):

- `runner_registry` promoted from a gate Rust const (`GENERATOR_RUNNERS`) to manifest data.
  Every runner must be declared; a runner not in the registry causes `plan()` to return `Err`.
  There is NO `shell` runner — `plan()` returns `ShellRunnerForbidden` if one is declared.
- `generator` block promoted from decorative to load-bearing: `plan()` reads `runner`,
  `generator_target`, `operation_id`, `parameters`, `input_contract`, `output_mode` from it.
- `input_contract` strings are the DAG edge source: an artifact whose `input_contract` references
  another artifact's `operation_id` or `artifact_id` is sequenced AFTER it. The topological order
  is fully data-derived — no hand-authored `needs:` or hardcoded target constants.

### Purity guarantee (MF-2)

The kernel source contains ZERO uses of:
- `std::process` (no subprocess spawn)
- `std::time::SystemTime` / `std::time::Instant` (no clock)
- `std::net::` (no network)
- `std::fs::` (no filesystem I/O)
- `std::env::` (no environment reads)
- `rand` (no randomness)

Enforced by the `mf2_no_banned_symbols_in_kernel_source` conformance test (source-grep over the
kernel src at test runtime). ADR-0547's kernel-purity gate bans dep CRATES, not std SYMBOLS; this
test fills that gap.

### No-leak guarantee (MF-3)

The kernel src contains ZERO hardcoded oyatie paths, names, or targets. Specifically:
- No `//cloud/` literals
- No `oya-cloud-ci-` literals
- No `cloud/cloud-ci` literals

Enforced by `cp6_mf3_no_oyatie_literals_in_kernel_source` (source-grep). Everything
oyatie-specific lives in the manifest data.

### MF-1 / merge-base anchoring (E3 precondition)

The de-commit exemption set is derived from the `manifest: &ControlPlane` parameter passed to
`evaluate()`. In E1, callers supply the candidate manifest (additive; no gate behavior change).
In E3 (the keystone repoint), the freshness gate MUST supply the **merge-base manifest** (the
control-plane as materialized at `git merge-base <base_ref> HEAD`) so a candidate PR cannot forge
a new de-commit row to evade byte-parity. The v2 contract is shaped to accept this: `manifest` is
a distinct parameter, not baked into the kernel — no schema break required for E3.

### Conformance certificate (CP-1..CP-6)

28 tests GREEN (14 unit + 14 conformance):

| Test | Property |
|---|---|
| `cp1_plan_determinism` | CP-1: `plan()` byte-identical across calls |
| `cp1_plan_determinism_synthetic` | CP-1: deterministic on synthetic fixture |
| `cp2_topological_order_from_input_contract` | CP-2: topo order from `input_contract` only |
| `cp2_topological_order_node_codegen_synthetic` | CP-2: node-codegen ordering in synthetic fixture |
| `cp3_canary_catches_nondeterminism` | CP-3: nondeterministic bytes → RED |
| `cp4_single_build_invariant` | CP-4: `multiplicity=2` is a structural plan property |
| `cp5_anti_forgery_full_path_keying` | CP-5: full-path keying defeats basename collision |
| `cp5_unregistered_runner_is_err` | CP-5: unregistered runner → `Err` |
| `cp5_non_canonical_target_is_err` | CP-5: non-canonical target → `Err` |
| `cp5_shell_runner_forbidden` | CP-5: `shell` runner → `ShellRunnerForbidden` |
| `cp6_repo_agnosticism_synthetic_fixture` | CP-6: engine plans a TypeScript repo fixture with zero engine changes |
| `cp6_mf3_no_oyatie_literals_in_kernel_source` | MF-3: source-grep universality certificate |
| `mf2_no_banned_symbols_in_kernel_source` | MF-2: banned std-symbol purity |
| `mf1_evaluate_accepts_separate_manifest_parameter` | MF-1: E3 can pass merge-base manifest without schema break |

The synthetic fixture (`tests/fixtures/synthetic-repo/control-plane.json`) contains a TypeScript
repo with `buck2` + `node-codegen` runners, proving the engine is repo-agnostic with ZERO engine
code changes — only a runner binding + manifest rows.

### Ownership + justification manifest (ADR-0555 D2 / total-accounting onboarding)

Owner: `libs/oya-ci-materializer-kernel/OWNERS` = `cloud-ci-platform` — the build/ meta-home owner
(ADR-0562 capability-registry membership_lint_coverage: `libs/oya-ci-materializer-kernel` maps to
the `build/` meta directory, the off-runtime-ladder CI engines + buck2/workspace/manifest tooling,
the same team that owns the cloud-ci gate fleet and the `tools/oya-*-app` build tooling). The
crate's sources are reachable via the `libs/oya-*` cargo-members glob (ADR-0538); the
member-dir prefix covers the whole crate directory, not only Rust files, so the `BUCK`, `Cargo.toml`,
`OWNERS`, and `tests/fixtures/*.json` non-Rust files are reachable too. No catalog record is minted:
the pure `libs/*-kernel` siblings (`libs/oya-ci-config`, `libs/oya-crate-registrar-kernel`) carry
none (the gate-tool default for a build/-home kernel). Files commissioned by this decision:

`libs/oya-ci-materializer-kernel/BUCK`,
`libs/oya-ci-materializer-kernel/Cargo.toml`,
`libs/oya-ci-materializer-kernel/OWNERS`,
`libs/oya-ci-materializer-kernel/src/evaluate.rs`,
`libs/oya-ci-materializer-kernel/src/lib.rs`,
`libs/oya-ci-materializer-kernel/src/model.rs`,
`libs/oya-ci-materializer-kernel/src/plan.rs`,
`libs/oya-ci-materializer-kernel/tests/conformance.rs`,
`libs/oya-ci-materializer-kernel/tests/fixtures/synthetic-repo/control-plane.json`,
`libs/oya-ci-materializer-kernel/tests/fixtures/synthetic-repo/transitive-chain.json`.

## Consequences

### Positive
- The pure kernel is additive: gates are unaffected by E1. Nothing is deleted or repointed.
- The conformance certificate (CP-1..CP-6) is the productization gate: any future engine change
  that breaks a CP test is a regression.
- The `manifest: &ControlPlane` parameter shape in `evaluate()` enables E3's merge-base anchoring
  without a schema break — MF-1 is an E3 acceptance criterion, not a deferred concern.
- All impurity (buck2 bootstrap, git, clock, pid temp paths) is reserved for E2
  (`oya-ci-materializer-app`), which is the sole ADR-0523 irreducible-glue surface.

### Negative / risks
- R1 (MF-1): The de-commit exemption is still candidate-manifest-trusted in E1 (no gate change,
  so no regression vs. today). E3 MUST supply the merge-base manifest to close the forgery hole
  identified by the adversarial critique. E3 is a hard acceptance criterion.
- R2: The runner pluggability surface (`runner_registry`) is per-adopter data. Supply-chain
  hardening (pinned-digest binding per runner) is future work, recorded in §3.5 of the
  determination.

## Supersedes / feeds

- **Implements** the E1 slice of the determination at
  `.omc/ultragoal/universal-generated-artifact-lifecycle-determination.md`.
- **Completes** the historical E1 TODO in ADR-0595 that called for Rust-native
  materialization — partially. E2..E5 complete the remaining controller lifecycle TODOs.
- **Feeds** E2 (`oya-ci-materializer-app`): the impure executor that byte-parity-proves parity
  for the retired shell bridge and the Rust/Buck2 materializer path.
- **Feeds** E3 (gate repoint + merge-base anchoring): the keystone that makes the gate a pure
  predicate fed by the executor.
