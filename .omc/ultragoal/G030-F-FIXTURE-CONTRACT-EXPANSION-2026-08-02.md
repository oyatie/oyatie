# G030-F fixture contract expansion — 2026-08-02

State: **PLANNING_ONLY — 118/137 MACHINE-CONSUMED; 19 RETAINED, NOT DELETION CANDIDATES**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-E-SPECS-REGISTRY-SEMANTIC-CONSUMER-CENSUS-2026-08-02.md`.  
No fixture, policy, gate, registry, generated face, or cluster state was changed.

## Result

The complete immutable `specs/fixtures/**` tree contains **137** paths in **19** fixture families.

| Class | Paths | Families | Disposition |
|---|---:|---:|---|
| Exact or directory-contract semantic consumer found | **118** | **16** | `GRAPH_WIRED_INPUT` |
| No machine consumer found in the measured scan | **19** | **3** | `POLICY_PROTECTED_MACHINE_ARTIFACT` |
| Deletion candidate | **0** | **0** | none |

The 118 machine-consumed paths are not merely structurally inventoried. Their exact paths and parent fixture contracts are cited by Rust gate tests, gate policies, or contract-slice specifications. The exact-path and parent-directory searches identify the same 118-path union; parent matching is corroborating evidence rather than an extra 118 rows.

This expands G030-E's measured fixture coverage. It does **not** weaken the unit-class rule that classifies all 137 paths as protected registry fixtures.

## Machine-consumed families — 118 paths

| Fixture family | Paths |
|---|---:|
| `cloud-ci-firewall` | 23 |
| `cross-artifact-agreement` | 22 |
| `cloud-ci-run-observability` | 13 |
| `phase0-ci-enforcement-baseline` | 12 |
| `phase0-automation-ratchet` | 10 |
| `owners-schema` | 7 |
| `total-accounting` | 7 |
| `staleness-reaper` | 6 |
| `phase0-claim-ceiling` | 5 |
| `phase0-exit-gate` | 3 |
| `cloud-ci-run-terminal-state` | 2 |
| `compliance-pack` | 2 |
| `platform-delivery-fabric` | 2 |
| `residency` | 2 |
| `cell-002-promotion-automation` | 1 |
| `cell-topology-manifest` | 1 |
| **Total** | **118** |

Representative consumer evidence includes:

- firewall fixtures read by `ci/facade/baseline-ratchet/tests/firewall.rs`;
- cell/compliance/residency fixtures named by contract-slice policies, slice specifications, and Rust tests;
- topology fixtures read by `ci/facade/topology-manifest-contract` tests;
- observability, cross-artifact, total-accounting, owners, staleness, claim-ceiling, and exit-gate families named by their corresponding gate contracts.

A fixture-path citation from a gate policy or test is a semantic edge because the consumer parses or validates the fixture content. A directory contract is accepted only where the gate enumerates or loads the family, not merely because prose names the directory.

## Residual families — 19 paths

### Calendar PRD — 15 paths

- `specs/fixtures/calendar-prd/red-fixtures.json`
- fourteen `specs/fixtures/calendar-prd/replay/**/*.fixture.json` rows covering AC, API-contract, authority, boundary, produced-contract, policy, and UX replay.

Measured evidence:

- colocated `specs/fixtures/calendar-prd/calendar_prd_replay_check.py` loads `red-fixtures.json`, locks `future_replay_root`, iterates every declared replay path, and validates each replay fixture;
- ADR-0599 names all fifteen paths as the commissioned calendar replay corpus;
- no Buck target, cloud-ci gate, governance gate, or workflow invokes the Python checker on the measured tip.

Disposition: `POLICY_PROTECTED_MACHINE_ARTIFACT — LOCAL_REPLAY_CONSUMER_NOT_BUILD_GRAPH_WIRED`.

The artifacts are semantically consumed by a colocated checker, so they are not dark bureaucracy. They do not qualify for `GRAPH_WIRED_INPUT` because the checker itself is not wired to Buck2 or protected CI. The correct follow-up is consumer migration/wiring in the owning calendar lane, not fixture deletion and not synthetic seeding merely to improve this census.

### CRATEADR owner-batch — 3 paths

- `tc-CRATEADR-002A-good-governance-check-gates-owner-batch.json`
- `tc-CRATEADR-002B-good-ci-control-plane-owner-batch.json`
- `tc-CRATEADR-002D-good-billing-metering-reorg-owner-batch.json`

Measured evidence:

- ADR-0515 explicitly registers each path and bounds it as fixture coverage for a named owner-batch scenario;
- no Rust, Buck, gate-policy, workflow, or current script consumer was found outside the fixture tree;
- the ADR citations establish retention authority, not executable consumption.

Disposition: `POLICY_PROTECTED_MACHINE_ARTIFACT — AUTHORITY_RETAINED_CONSUMER_UNRESOLVED`.

These rows are not deletion candidates. The owning crate-ADR coverage lane must either cite the live machine consumer, wire one, migrate the scenarios to an existing native gate corpus, or explicitly declassify them through reviewed authority. G030 must not infer declassification from consumer absence.

### DR RTO/RPO matrix — 1 path

- `specs/fixtures/dr-rto-rpo-matrix/dr-001-dashboard-manifest.fixture.json`

Measured evidence:

- `scripts/tests/dr_001_rto_rpo_matrix_slice_check.py` loads this exact fixture and validates its manifest DR block, compliance-pack floors, effective RTO/RPO, drill-evidence freshness, and dashboard row shape;
- the automation-language policy lists that Python validator as a temporary exception/provenance bridge;
- ADR-0343 calls both script and fixture contract-only local-bridge evidence and names native Rust/Buck2 cloud-ci gates as the successor;
- no Buck or protected workflow invocation was found for the Python bridge.

Disposition: `POLICY_PROTECTED_MACHINE_ARTIFACT — TRANSITIONAL_LOCAL_BRIDGE_NOT_BUILD_GRAPH_WIRED`.

The fixture is live to the temporary bridge and retained by ADR authority. It is not sufficient merge-authority evidence and is not deletable before the native Rust/Buck2 successor absorbs its assertions and the owner retires the bridge atomically.

## Corrected G030-E interpretation

G030-E reported 131 fixture rows in the policy-protected remainder. That number was a conservative first-pass bucket, not the immutable fixture-tree cardinality. This expansion establishes the exact fixture partition:

```
137 total = 118 GRAPH_WIRED_INPUT + 19 POLICY_PROTECTED_MACHINE_ARTIFACT
```

The G030-E top-level 1,176-path partition remains a non-overlapping first-pass census and is not silently rewritten here. A later reconciled semantic census must recompute every row once, from the immutable tree, rather than adding 118 to the prior graph count and double-counting paths already classified through other exact citations.

## Consumer-proof limits

The scan covered exact path literals, fixture-family directory contracts, Rust/Buck gate surfaces, gate policies, workflows, governance surfaces, and known colocated validators. It supports these claims:

1. 118 fixtures have executable/gate semantic edges.
2. 15 calendar and one DR fixture have local Python semantic consumers but no measured Buck/protected-CI edge.
3. Three CRATEADR fixtures have explicit authority retention but no measured machine consumer.

It does not prove that an untracked or external consumer does not exist. It also does not treat historical prose citation as machine execution.

## Next smallest work

1. Reconcile the full G030-E 1,176-row census once, eliminating overlap and carrying these fixture dispositions row-for-row.
2. Queue calendar replay and DR bridge migration to native Rust/Buck2 with their owning lanes; do not create a G030 implementation PR for them.
3. Ask the crate-ADR coverage owner to identify the current consumer or rule on migration/declassification for the three retained owner-batch fixtures.
4. Keep all 19 protected until those owner-lane outcomes are independently reviewed.

## Non-actions and non-claims

- No fixture deletion, move, edit, or generated artifact change.
- No Python validator was promoted as merge authority.
- No synthetic affected-set edge was invented.
- No claim that prose-only authority is executable consumption.
- No independent APPROVE; unavailable review transport is not approval.
