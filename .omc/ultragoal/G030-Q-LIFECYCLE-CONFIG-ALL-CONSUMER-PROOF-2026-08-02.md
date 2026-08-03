# G030-Q lifecycle-config residual all-config consumer proof — 2026-08-02

State: **PLANNING_ONLY — EIGHT RESIDUAL LIFECYCLE CONFIGS GRAPH-WIRED; NO CONFIG/POLICY EDIT**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-P-SINGLETON-REGISTRY-RESIDUAL-CONSUMER-PROOF-2026-08-02.md`.  
No lifecycle config, gate policy, baseline, PR, GitOps declaration, or cluster state was changed.

## Result

G030-G's 36-row residual contains eight `specs/lifecycle-configs/*.json` rows not classified by G030-E's exact-literal probe. All eight are now proven executable graph inputs through a stronger directory contract: the Buck2-native `ci/facade/lifecycle-status` gate reads **every JSON config** under the policy-selected directory, loads and evaluates each, and fails closed when a discovered config is not evaluated.

| Residual path | Gate treatment at tip | Disposition |
|---|---|---|
| `specs/lifecycle-configs/adr-status-lifecycle.json` | enumerated; live corpus; frozen shrink-only baseline | `GRAPH_WIRED_INPUT — ALL-CONFIG LIFECYCLE GATE` |
| `specs/lifecycle-configs/api-stability-tier-lifecycle.json` | enumerated; explicitly known-broken pending re-root-or-delete | `GRAPH_WIRED_INPUT — ALL-CONFIG LIFECYCLE GATE` |
| `specs/lifecycle-configs/capability-status-lifecycle.json` | enumerated; explicitly known-broken zero-observation lane | `GRAPH_WIRED_INPUT — ALL-CONFIG LIFECYCLE GATE` |
| `specs/lifecycle-configs/crate-status-lifecycle.json` | enumerated; explicitly known-broken after reorg moved its corpus | `GRAPH_WIRED_INPUT — ALL-CONFIG LIFECYCLE GATE` |
| `specs/lifecycle-configs/dependency-status-lifecycle.json` | enumerated; explicitly known-broken pending re-root-or-delete | `GRAPH_WIRED_INPUT — ALL-CONFIG LIFECYCLE GATE` |
| `specs/lifecycle-configs/doc-status-lifecycle.json` | enumerated; live corpus; frozen shrink-only baseline | `GRAPH_WIRED_INPUT — ALL-CONFIG LIFECYCLE GATE` |
| `specs/lifecycle-configs/migration-status-lifecycle.json` | enumerated; explicitly known-broken because source root is ignored runtime state | `GRAPH_WIRED_INPUT — ALL-CONFIG LIFECYCLE GATE` |
| `specs/lifecycle-configs/plan-status-lifecycle.json` | enumerated; explicitly known-broken because source root is ignored runtime state | `GRAPH_WIRED_INPUT — ALL-CONFIG LIFECYCLE GATE` |

This promotes eight residual rows. Reconciled totals become **152 `MACHINE_SSOT` + 956 `GRAPH_WIRED_INPUT` + 68 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. Remaining protected queue: 19 fixture + 49 non-fixture. Delete candidates remain 0.

`feature-flag-status-lifecycle.json` is intentionally absent from this slice: G030-E already classified it through an exact execution citation, so re-counting it would violate the non-overlap invariant.

## Directory-contract proof

`ci/facade/lifecycle-status/lifecycle-status-policy.json`:

1. sets `configs_dir` to exactly `specs/lifecycle-configs`;
2. carries frozen shrink-only baselines for live lanes;
3. carries an explicit `known_broken_lanes` ledger for seven measured blind/vacuous lanes;
4. requires every broken row to state a concrete re-root-or-delete resolution.

`ci/facade/lifecycle-status/tests/lifecycle_status.rs`:

1. joins the policy-selected directory under the repo root;
2. `read_dir`s it and filters every `.json` file;
3. sorts the complete config set and derives lane IDs from file stems;
4. loads every config through `oya_governance_lifecycle_kernel::discovery::load_config`;
5. attempts discovery/evaluation for every lane and records discovery failure rather than swallowing it;
6. passes the complete discovered-lane list and observations to `compare`;
7. asserts the findings list is empty.

`ci/facade/lifecycle-status/src/lib.rs` fails closed on:

- config discovered but not evaluated;
- discovery failure or zero observations for an unlisted lane;
- known-broken lane becoming live without deleting its exception row;
- known-broken or baseline row naming a missing config;
- unbaselined violations, regression growth, or stale shrinkable baselines.

`ci/facade/lifecycle-status/BUCK` defines `ci-lifecycle-status-gate` over the integration test and kernel. Its comments explicitly bind the completeness invariant to **every config on disk**. This is stronger than eight independent hard-coded path literals.

## Broken-lane boundary

Graph wiring does not mean the underlying lifecycle is effective:

- live and baselined: `adr-status-lifecycle`, `doc-status-lifecycle`;
- known broken: API stability, capability, crate, dependency, migration, plan;
- `feature-flag-status-lifecycle` is also known broken but was already graph-wired before this residual slice.

The required lane intentionally makes those defects visible instead of certifying vacuous green. A known-broken config is therefore wired policy debt, not an unused artifact and not a delete candidate. Re-root-or-delete remains an owner decision in G011/G012 governance work.

## Anti-vacuity and semantic boundary

Proven:

- immutable tip contains nine lifecycle JSON configs;
- one (`feature-flag-status-lifecycle.json`) was already classified by G030-E;
- the exact residual set is the other eight;
- the gate enumerates all nine and compares the full discovered set;
- the eight residual rows are new promotions, with no double count;
- six of the eight are explicitly known broken and two carry live frozen baselines.

Not proven:

- successful protected-context execution at current tip;
- remediation of any known-broken lane;
- correctness or completeness of every lifecycle stage model;
- permission to delete rather than re-root a broken config.

These are governance lifecycle decisions, not G030 delete authority.

## Verification boundary

Evidence came from immutable tip tree membership, gate policy, Rust all-config enumeration/evaluation, fail-closed comparison logic, and Buck target wiring at `b651080374113aeb57500eecbd9d1326f0404e48`. No local CLI execution is used as merge authority.

An independent lifecycle-consumer verifier failed with the same encrypted-content transport error. It remains **FAILED_TRANSPORT_NOT_APPROVE**; this mechanical proof is not independent approval.

## Non-actions and non-claims

- No lifecycle config, known-broken entry, or frozen baseline edited.
- No claim that known-broken means acceptable or enforceable.
- No deletion/re-root ruling made.
- No move-plan JSON, generated face, or multispectrum evidence surface added.
- No independent APPROVE inferred from transport failure.
- G028 remains local-only unpushed at `051bc7ec6`; no cluster mutation.
