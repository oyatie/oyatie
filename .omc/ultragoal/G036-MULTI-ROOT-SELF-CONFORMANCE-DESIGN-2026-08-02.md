# G036 multi-root self-conformance design — 2026-08-02

State: **DESIGN ONLY — NOT IMPLEMENTED — NOT REVIEWED — NOT ADMITTED**

Anchor: `origin/dev` `0c1014b87f0d881a821faa6a872b309deba0cfbf` (#1529 merged; ARC request declared `22Gi`, live request still `20Gi`).  
Measurement provenance for the original census remains `b651080…`; tip recompute 2026-08-02 confirmed 56/8/48 unchanged (`G036-EXACT-48-KERNEL-PROTECTED-CONTEXT-GAP-2026-08-02.md`).

Related:
- `.omc/ultragoal/G036-PROTECTED-GRAPH-CENSUS-2026-08-02.md` (56 kernels; 8 policy-selected; 48 bridge-only)
- `.omc/ultragoal/G036-EXACT-48-KERNEL-PROTECTED-CONTEXT-GAP-2026-08-02.md` (exact gap set; no activation)
- `ci/facade/gate-self-conformance/{src/lib.rs,gate-self-conformance-policy.json}`
- `ci/facade/baseline-ratchet/tests/gate_registration.rs`

## Observed contracts (do not redesign from taste)

### Self-conformance collector (`collect_observed_gates`)

Single root today:

```json
{"scan":{"gates_root":"ci/facade","workflow_path":".github/workflows/oya-ci-required.yml"}}
```

Per directory under that root it requires:
- `Cargo.toml` present
- optional prefix filter (`gate_crate_prefix`)
- `non_gate_crates` exclusion set
- BUCK gate target name shape: `ci-{name}-gate` (or `bespoke_buck2_gate_crates`)
- BUCK unittest name shape: `ci-{name}-unittest`
- `workflow_registered` via matrix include OR `//{gates_root}/{name}:` OR recursive `buck2 test //{prefix}...` that covers the root
- hermetic + policy-literal production-source scans

Docstring already admits: `workflow_registered` is descriptive; fan-in reachability is owned by `gate_registration.rs`.

### Registration meta-test (`gate_registration.rs`)

- Universe is the `ci/facade/` fleet directory.
- Registration = fan-in-reachable executable `buck2 test` coverage + real `*_test` rule in BUCK.
- Separate shrink-only freeze: `no_new_gate_crate_is_born_outside_the_registered_gate_fleet` for catalog rows declaring `fitness-*` capabilities outside `ci/facade/`.

### `governance/check/*` reality

Sampled BUCK/Cargo shapes on trunk:

| Kernel | package | library target | unittest target | `ci-*-gate` target |
|---|---|---|---|---|
| `pr-traceability` | `check-pr-traceability` | `check-pr-traceability` | `check-pr-traceability-unittest` | **absent** |
| `data-class` | `check-data-class` | `check-data-class` | `check-data-class-unittest` | **absent** |
| `active-artifact-contract` | same pattern | same | same | **absent** |
| `codeowners-mirror` | same pattern | same | same | **absent** |

So a naive second root entry `governance/check` under today's collector would either:

1. skip every kernel (prefix/name filters / missing `ci-*-gate`), producing a false green; or
2. if naming is loosened carelessly, mark 56 kernels `workflow_unregistered` / missing-gate-target and red the protected graph without a disposition policy.

Neither is acceptable.

## Goal of the next slice

Prove **protected-context reachability** for retained gate kernels across multiple roots, with born-blocking fixtures, without:

- mass-deleting the 48 bridge-only kernels
- exempting them into silence
- greening by comment/mention
- changing `gates_root` from `ci/facade` to `governance/check` (moves the blind spot)

## Design: multi-root scan with per-root contracts

### Policy shape (proposed)

```json
{
  "scan": {
    "workflow_path": ".github/workflows/oya-ci-required.yml",
    "roots": [
      {
        "path": "ci/facade",
        "class": "protected_fleet",
        "gate_crate_prefix": "",
        "non_gate_crates": ["...existing..."],
        "bespoke_buck2_gate_crates": ["affected-target-set", "generated-artifact-freshness"],
        "buck_gate_target": "ci-{name}-gate",
        "buck_unittest_target": "ci-{name}-unittest",
        "registration": "fan_in_reachable_execution_required"
      },
      {
        "path": "governance/check",
        "class": "legacy_check_kernel",
        "gate_crate_prefix": "",
        "non_gate_crates": [],
        "buck_gate_target": null,
        "buck_library_target": "check-{name}",
        "buck_unittest_target": "check-{name}-unittest",
        "registration": "classified_protected_or_bridge_or_retire",
        "classification_authority": "policy.legacy_check_kernel_classifications"
      }
    ]
  },
  "legacy_check_kernel_classifications": {
    "required_keys_complete": true,
    "rows": {
      "pr-traceability": {"class": "PROTECTED_POLICY", "consumer": "affected-set-policy.synthetic_dependencies"},
      "data-class": {"class": "PROTECTED_POLICY", "consumer": "affected-set-policy.synthetic_dependencies"},
      "...48 bridge-only...": {"class": "BRIDGE_ONLY_PENDING_REVIEW", "consumer": "marketplace/facade/dev-cli"}
    }
  }
}
```

Compatibility: if `scan.roots` is absent, read legacy `scan.gates_root` as a single protected_fleet root. No silent behavior change for current fleet.

### Evaluation rules by root class

#### `protected_fleet` (current `ci/facade`)

Keep existing red findings:

- missing `ci-*-gate` / unittest (unless bespoke)
- workflow unregistered under fan-in-reachable execution semantics (prefer tightening self-conformance to call the same helper as `gate_registration`, or keep descriptive + leave binding to registration test — do not weaken registration)
- hermetic / policy-literal / autofix contracts

#### `legacy_check_kernel` (`governance/check`)

New findings (born-blocking):

| Code | RED when |
|---|---|
| `gate_self_conformance_legacy_kernel_unclassified` | kernel dir with Cargo/BUCK exists and has no classification row |
| `gate_self_conformance_legacy_kernel_stale_class` | classification row names a missing kernel |
| `gate_self_conformance_legacy_protected_unregistered` | class=`PROTECTED_*` and no fan-in-reachable `buck2 test` pattern covers `//governance/check/{name}` OR no unittest rule exists |
| `gate_self_conformance_legacy_bridge_only_claimed_protected` | class claims protected but only bridge exposure exists |
| `gate_self_conformance_legacy_delete_without_proof` | class=`RETIRE` without required proof fields (no importers, no protected consumer, replacement/ADR note) |

Non-findings (explicitly not red yet):

- `BRIDGE_ONLY_PENDING_REVIEW` with complete classification row and honest consumer=`dev-cli` — **visible debt**, not silent. Optional advisory count in report, not admission red, until owner disposition. The red is **missing classification**, not the bridge-only state itself.

This matches the census safety ruling: do not delete 48 rows; do not pretend they are binding.

### Collector changes (mechanical)

1. Replace single `gates_root` walk with loop over `roots`.
2. Emit each observation with `root_path` + `root_class`.
3. Parameterize target-name checks by root contract (no hard-coded `ci-` only once multi-root lands).
4. For registration checks on legacy roots, reuse the same recursive-pattern / keep-going exclusion logic already in `workflow_executes_recursive_gates_pattern`, but evaluate package path `governance/check/{name}` not `ci/facade/{name}`.
5. Keep hermetic/policy-literal scans root-agnostic on production Rust sources.

### Registration meta-test relationship

Do **not** expand `every_gate_crate_is_registered_in_oya_ci_required_workflow`'s universe to all of `governance/check` in the same PR as multi-root collection. That would immediately red 48 bridge-only kernels and force false exemptions.

Order:

1. Multi-root self-conformance + classification completeness (this design).
2. Owner dispositions for bridge-only rows (PROTECTED wire / RETIRE with proof / KEEP bridge-local with expiry).
3. Only then extend fan-in registration universe to kernels classified PROTECTED.

The outside-fleet freeze remains for `fitness-*` catalog gates born outside `ci/facade/`; `governance/check` kernels are a distinct legacy class and must not be laundered through that freeze array.

## Born-blocking fixtures (minimum set)

Implement as pure evaluator fixtures first (no repo-wide scan required), then one temp-tree collector fixture.

1. **Single protected root still greens** a well-formed `ci/facade` gate (compat).
2. **Unclassified legacy kernel reds** — temp tree with `governance/check/orphan-kernel/{Cargo.toml,BUCK,src/lib.rs}` and empty classifications ⇒ `legacy_kernel_unclassified`.
3. **Stale classification reds** — row for missing kernel ⇒ `legacy_kernel_stale_class`.
4. **Protected-classified but unexecuted reds** — classification `PROTECTED_POLICY` + workflow without covering `buck2 test //governance/check/...` or per-target ⇒ `legacy_protected_unregistered`.
5. **Bridge-only classified does not red** on registration alone when row is complete.
6. **RETIRE without proof reds**.
7. **Comment-only workflow mention does not register** (existing invariant retained).

Oracle-before-rewrite: land fixtures failing against current evaluator (single-root cannot see legacy kernel), then implement multi-root until fixtures pass without weakening protected_fleet rules.

## Explicit non-goals of the first implementation PR

- No deletion of any `governance/check/*` kernel
- No mass wiring of 48 kernels into `oya-ci-required`
- No renaming `check-*` targets to `ci-*`
- No changing affected-set synthetic_dependencies in the same PR unless a single exemplar needs it
- No live cluster mutation
- No second registry of kernels; classification lives in the existing self-conformance policy (policy-as-data), one file

## Smallest implementation PR split

| PR | Contents | Admission risk |
|---|---|---|
| G036-A | policy schema multi-root + compat path + pure fixtures 1–3 (red on current code) | low if fixtures only / feature-gated? Prefer ship evaluator+compat together so CI stays green |
| G036-B | collector multi-root + classification completeness for all 56 rows (48 as BRIDGE_ONLY_PENDING_REVIEW, 8 as PROTECTED_POLICY with current consumers) + fixtures 4–7 | medium; must not red protected fleet |
| G036-C+ | per-kernel dispositions / wiring / retirement with proof | serial, one concern each |

G036-A+B may be one PR if fixtures and compat are complete; do not combine with G025 moves.

## Worked example: first protected exemplar (later)

`pr-traceability` is already named in workflow prose as retained locally while PR-body admission is retired. Before any PROTECTED wiring PR:

1. Prove current consumer (local retention vs affected-set synthetic vs none).
2. If retained as protected: add fan-in-reachable `buck2 test` pattern covering `//governance/check/pr-traceability` OR move into `ci/facade` via G025 plan with owned codemod — do not hand-move.
3. Only then flip classification from pending to protected-registered and extend registration universe.

## Acceptance for this design artifact

This document is accepted as planning evidence when:

- it cites the real collector/registration contracts (done)
- it forbids mass-delete and mass-false-green (done)
- it names born-blocking fixtures before rewrite (done)
- implementation has not started without independent review of the first code PR

Independent review of this design: **not yet obtained**.
