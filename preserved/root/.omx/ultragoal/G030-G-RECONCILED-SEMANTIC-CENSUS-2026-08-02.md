# G030-G reconciled specs/registry semantic census — 2026-08-02

State: **PLANNING_ONLY — NON-OVERLAPPING RECONCILIATION; 130 PROTECTED ROWS REMAIN**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supersedes only the **counts** in the first-pass partition of `G030-E-SPECS-REGISTRY-SEMANTIC-CONSUMER-CENSUS-2026-08-02.md`; preserves its evidence and non-deletion posture. Incorporates `G030-F-FIXTURE-CONTRACT-EXPANSION-2026-08-02.md`.  
No spec, registry, fixture, policy, gate, generated face, or cluster state was changed.

## Corrected one-pass partition

Every one of the 1,176 immutable focus paths is assigned exactly once, under this precedence:

1. `MACHINE_SSOT` — direct authority/exact semantic evidence from the first pass;
2. `GRAPH_WIRED_INPUT` — executable gate, policy, Buck, or validated fixture-family semantic edge;
3. `POLICY_PROTECTED_MACHINE_ARTIFACT` — protected by unit class/TTL but without a proven build-graph semantic edge.

| Class | G030-E first pass | Fixture reconciliation | Corrected |
|---|---:|---:|---:|
| `MACHINE_SSOT` | 152 | 0 | **152** |
| `GRAPH_WIRED_INPUT` | 782 | +112 newly proven fixture rows | **894** |
| `POLICY_PROTECTED_MACHINE_ARTIFACT` | 242 | −112 newly proven fixture rows | **130** |
| **Total** | **1,176** | **0 net** | **1,176** |

Anti-double-count proof:

- fixture tree: 137 paths;
- six fixtures were already in G030-E `GRAPH_WIRED_INPUT` through exact direct citations;
- 112 were in the protected bucket and G030-F proved executable/family consumers;
- 19 remain protected-only under the residual calendar/CRATEADR/DR dispositions;
- `6 + 112 + 19 = 137`;
- all three corrected class sets are pairwise disjoint and their union cardinality is 1,176.

The corrected invariant is:

```
152 + 894 + 130 = 1,176
```

## Why the correction is legitimate

G030-E deliberately used exact-path and known directory-contract evidence conservatively. G030-F then expanded the complete fixture tree and separated:

- 118 fixture paths with semantic consumers;
- 19 retained rows with only local/transitional/authority evidence.

Promoting the 112 newly proven rows changes evidence classification, not retention policy. It does not create synthetic edges, infer execution from prose, or weaken `protected: true`.

## Remaining protected-only queue — 130

### Fixture residual — 19

- calendar PRD replay: 15 — colocated Python semantic consumer, not Buck/protected-CI wired;
- CRATEADR owner-batch: 3 — ADR-retained, no measured machine consumer;
- DR RTO/RPO: 1 — transitional Python bridge, not Buck/protected-CI wired.

These remain exactly as ruled in G030-F.

### Non-fixture remainder — 111

| Family | Rows |
|---|---:|
| root-level `specs/*` and nested schema/contract families | 36 |
| `specs/design-system/*` | 17 |
| `registry/check-empirical-evidence/*` | 14 |
| `specs/reorg/*` move/graph plans | 10 |
| `registry/accounts/*` | 5 |
| `registry/vcs/*` | 5 |
| `registry/foundation-bypasses/*` | 4 |
| `registry/capability-templates/*` | 3 |
| `registry/loop-recovery-patterns/*` | 3 |
| `registry/capabilities/*` | 2 |
| `registry/release/*` | 2 |
| singleton registry families | 10 |
| **Total non-fixture** | **111** |

The singleton registry rows are ADR inheritance, retry budget, claim matrix, dependency allowlist, architecture graph, hyperscaler scorecard index, two merge-queue logs, microservices registry, and mistakes ledger.

## Investigation rule for the remaining 130

This queue is **not** a deletion queue. Each row keeps policy protection until one of these outcomes is proven:

1. `MACHINE_SSOT` — direct authority or semantic reader;
2. `GRAPH_WIRED_INPUT` — executable build/gate/consumer edge;
3. `POLICY_PROTECTED_MACHINE_ARTIFACT` — retained pending owner or consumer migration;
4. only after explicit owner declassification **and** negative consumer proof may a separate dual-proof deletion inquiry begin.

No row may jump from protected-only to `DARK_BUREAUCRACY` because a literal grep is empty. Directory enumeration, producer globs, runtime loaders, historical authority, and owner retention must be checked first.

## Next measured slices

Use bounded families rather than a 130-row mega-audit:

1. ten `specs/reorg/*` plans — prove active consumer, spent-plan history, or compatibility retention row-by-row;
2. three `registry/loop-recovery-patterns/*` plus their empirical scorecards — prove detector/fixture pairing;
3. five `registry/accounts/*` — distinguish executable account-pool configuration schema/examples from reference-only material;
4. seventeen design-system JSON rows — map to console/design-system consumers without inventing app ownership;
5. remaining root specs and registries by explicit owner/controller.

Start with the ten reorg plans because they are structurally bounded and have an established owned codemod consumer contract. Do not edit or delete them in the census lane.

## Verification and evidence boundary

The reconciliation was computed from the immutable 1,176-path class lists produced for G030-E plus the immutable 137-path fixture census from G030-F. It verifies:

- corrected class cardinalities 152 / 894 / 130;
- zero pairwise overlap;
- union cardinality 1,176;
- fixture accounting 6 already graph + 112 promoted + 19 residual = 137;
- remaining protected-only queue 19 fixture + 111 non-fixture = 130.

The exact row lists remain temporary analysis material rather than a new repository registry. This report records counts, evidence rules, and bounded next work; it does not create another machine authority.

## Non-actions and non-claims

- No deletion, declassification, policy edit, synthetic seed, or generated artifact edit.
- No claim that all 894 graph-wired rows are global authorities.
- No claim that the 130 protected rows are unused.
- No new multispectrum evidence surface.
- No independent APPROVE; review transport failure remains non-approval.
