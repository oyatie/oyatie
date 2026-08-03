# G025 check-like destination audit — 2026-08-02

State: **PLANNING_ONLY — NOT ACTIVATED**
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.
Supplements `G025-LIBS-DISPOSITION-PLAN-COVERAGE-2026-08-02.md`.

## Existing plan fact (do not duplicate)

`specs/reorg/governance-check-move-plan.json` has **56** moves, all `libs/oya-check-*` → `governance/check/<leaf>`. Those 56 destinations **exist** as live `governance/check/*` packages. Live `libs/` no longer holds those 56 old paths (coverage of remaining libs by that plan = **0**). Do not re-author them.

## 16 check-like-unplanned rows

| libs crate | proposed leaf | Live destination on origin/dev |
|---|---|---|
| `oya-check-adr-index` | `adr-index` | **MISSING** |
| `oya-check-adr-placeholders` | `adr-placeholders` | **MISSING** |
| `oya-check-backup-retention-discipline` | `backup-retention-discipline` | **MISSING** |
| `oya-check-brand-residue` | `brand-residue` | **MISSING** |
| `oya-check-claim-ceiling` | `claim-ceiling` | **MISSING** |
| `oya-check-compliance-evidence-coverage` | `compliance-evidence-coverage` | **MISSING** |
| `oya-check-container-base-image` | `container-base-image` | **MISSING** |
| `oya-check-cost-budget` | `cost-budget` | **MISSING** |
| `oya-check-dependency-seam` | `dependency-seam` | **MISSING** |
| `oya-check-doc-axis` | `doc-axis` | **MISSING** |
| `oya-check-i18n-coverage` | `i18n-coverage` | **MISSING** |
| `oya-check-license-policy` | `license-policy` | **EXISTS** `ci/facade/license-policy` |
| `oya-check-realtime-transport-tier` | `realtime-transport-tier` | **MISSING** |
| `oya-check-saga-shape` | `saga-shape` | **MISSING** |
| `oya-check-slo-coverage` | `slo-coverage` | **EXISTS** `ci/facade/slo-coverage` |
| `oya-check-step-up-auth-coverage` | `step-up-auth-coverage` | **MISSING** |

### Counters

| Class | Count |
|---|---:|
| Leaf destination exists (possible MOVE target after importer proof) | **2** |
| Leaf missing (CLASS_KNOWN_LEAF_MISSING) | **14** |

## Special handling for the two EXISTS rows — RESOLVED

See `G025-LICENSE-SLO-KERNEL-FACADE-AUDIT-2026-08-02.md`.

Both EXISTS rows are intentional kernel/facade pairs:

- facade Cargo/BUCK depend on the libs kernel
- CI producers consume the facade packages
- marketplace/dev-cli still consumes the kernels directly

Disposition: **KEEP / REFACTOR_CANDIDATE**, not MOVE and not DELETE. Candidate A is rejected.

## Smallest safe next G025 executable slice (still blocked)

Only after independent APPROVE + #1526 observed green + #1523 promoted green:

- **Not candidate:** 2-row MOVE-into-existing-face or DELETE dual-home.
- **Candidate A' (1 pair REFACTOR):** absorb one pure kernel into its existing `ci/facade/<leaf>` package, rewrite importers, retire CLI dependency, prove zero kernel importers, then delete the libs package.
- **Not candidate:** 14-row mega-plan inventing `governance/check/<leaf>` paths.

Face birth for the 14 missing leaves remains a separate born-blocking package + catalog row problem (same class as G026 tools).

## Non-claims

- No move-plan JSON authored.
- No code moved.
- 16 name-classed rows are not 16 authorized MOVEs.
