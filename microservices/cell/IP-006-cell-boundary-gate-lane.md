---
doc_class: ImplementationPlan
template_id: TPL-IMPL
microservice: cell
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-006-cell-boundary-gate-lane
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
co_owners: [axis-security, axis-data]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0183, ADR-0117]
acceptance_lanes: [cargo-check, cargo-nextest, oya-cell-boundary, oya-vcs-promotion-readiness]
depends_on: [IP-005]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006 — `oya-cell-boundary` BLOCKER CI lane

## Goal

Author the BLOCKER lane `oya-cell-boundary` registered on `dev` + `staging` branch protection. The lane statically validates four cross-cutting cell isolation invariants per workload µservice repository tree:

1. SQL queries never `JOIN` or `UNION ALL` across cell prefixes.
2. Kubernetes manifests never cross-reference `cell-*` namespaces.
3. Cedar policy fragments never permit cross-pack writes (per ADR-0183).
4. Postgres migrations include RLS clauses on every cell-scoped table.

## Files to create or modify

| Path | Action | Line range (approx) |
|---|---|---|
| `crates/oya-check-cell-boundary/Cargo.toml` + `src/lib.rs` | create | ~280 LoC; primary entrypoint + 4 sub-scanners |
| `crates/oya-check-cell-boundary/src/sql_scan.rs` | create | ~140 LoC; SQL parser via `sqlparser` crate; cross-cell JOIN/UNION detector |
| `crates/oya-check-cell-boundary/src/k8s_scan.rs` | create | ~120 LoC; serde-yaml + namespace reference graph |
| `crates/oya-check-cell-boundary/src/cedar_scan.rs` | create | ~140 LoC; uses `cedar-policy` crate AST to detect cross-pack writes |
| `crates/oya-check-cell-boundary/src/postgres_migration_scan.rs` | create | ~120 LoC; migration parser; asserts RLS clause present per cell-scoped table |
| `crates/oya-check-cell-boundary/tests/cell_boundary_lane.rs` | create | ~240 LoC; 8 fixture-based tests |
| `crates/oya-check-cell-boundary/tests/fixtures/violating/` | create | curated synthetic violations (sql/k8s/cedar/migration) |
| `crates/oya-check-cell-boundary/tests/fixtures/clean/` | create | curated clean fixtures |
| `crates/oya-dev-cli/src/governance_gates.rs` | edit | register `cell-boundary` gate |
| `registry/quality/lanes.yaml` | edit | declare `oya-cell-boundary` as BLOCKER on `dev` + `staging` |
| `microservices/cell/specs/cell-boundary-lane.json` | create | ~80 LoC; lane spec doc |
| `.github/branch-protection.yaml` | edit | add `oya-cell-boundary` to required status checks |
| `microservices/cell/runbooks/cell-boundary-violation.md` | create | ~100 LoC operator playbook |
| `microservices/cell/decisions/ADR-0145.md` | append §"Lane shipped" | +6 LoC |

## Code shape

`crates/oya-check-cell-boundary/src/lib.rs` (excerpt):

```rust
pub struct CellBoundaryCheck { config: CellBoundaryConfig }

impl CellBoundaryCheck {
    pub fn run(&self, manifest_root: &Path) -> Result<CheckReport, CheckError> {
        let mut violations = Vec::new();
        violations.extend(sql_scan::scan(manifest_root, &self.config)?);
        violations.extend(k8s_scan::scan(manifest_root, &self.config)?);
        violations.extend(cedar_scan::scan(manifest_root, &self.config)?);
        violations.extend(postgres_migration_scan::scan(manifest_root, &self.config)?);
        Ok(CheckReport { violations, ms_scanned: enumerate_ms(manifest_root)?.len() })
    }
}
```

## Tests to write (acceptance)

| Test name | File | Asserts |
|---|---|---|
| `sql_cross_cell_join_detected` | tests/cell_boundary_lane.rs | `JOIN cell_b.events` from cell_a context → violation |
| `sql_union_all_across_cells_detected` | tests/cell_boundary_lane.rs | `UNION ALL` across cell schemas → violation |
| `sql_clean_intra_cell_join_passes` | tests/cell_boundary_lane.rs | `JOIN cell_a.users` from cell_a → clean |
| `k8s_cross_namespace_reference_detected` | tests/cell_boundary_lane.rs | ServiceMonitor in cell-a referencing cell-b service → violation |
| `cedar_cross_pack_write_detected` | tests/cell_boundary_lane.rs | `permit(principal in eu, action == ::write, resource in us_healthcare)` → violation |
| `postgres_migration_missing_rls_detected` | tests/cell_boundary_lane.rs | CREATE TABLE with cell-scoped columns but no `ENABLE ROW LEVEL SECURITY` → violation |
| `postgres_migration_with_rls_passes` | tests/cell_boundary_lane.rs | Same table with RLS enabled + policy → clean |
| `lane_completes_under_60s_on_full_repo` | tests/cell_boundary_lane.rs | Full repo scan < 60s wall clock |

Minimum 5 required; 8 specified.

## Evidence to emit

- `evidence/microservices/cell/cell-boundary-scan-{date}.json` — per-µservice scan result + violation list
- `evidence/microservices/cell/cell-boundary-perf-{date}.json` — scan wall-clock per µservice
- Audit-chain seal: `oya audit-chain seal --kind cell-boundary-scan --window 7d`
- Metrics: `oya_cell_boundary_violations_total{kind,microservice}`, `oya_cell_boundary_scan_duration_seconds`

## Rollback procedure

1. Revert ChangeSet for the crate + spec + branch-protection edit.
2. Remove `oya-cell-boundary` from `registry/quality/lanes.yaml` (move to `lanes_quarantined` with reason).
3. Existing detected violations remain visible in evidence ledger (no destructive op).
4. Emit rollback evidence JSON.

## Blocking dependencies

- IP-005 — cell substrate scaffolded (so cell prefixes exist to scan).
- ADR-0145 — cell isolation canonical.
- ADR-0183 — Cedar canonical.
- ADR-0117 — residency / per-pack policy.

## Acceptance gates

```bash
cargo nextest run -p oya-check-cell-boundary
cargo run -p oya-dev-cli -- gate validate cell-boundary --manifest-root .
cargo run -p oya-dev-cli -- gate validate oya-vcs-promotion-readiness --microservice cell
```

## Halt conditions

- False-negative (lane misses a real violation): STOP — lane cannot be declared BLOCKER until false-negative-free on the curated fixture set.
- Lane runtime > 60s on full repo: STOP — optimize before promoting to BLOCKER.
- Cedar AST parse fails on a real policy fragment: STOP — file parser-defect IP.

## Exit criteria

1. All 8 tests green.
2. `cell-boundary` lane declared BLOCKER in `registry/quality/lanes.yaml` and `.github/branch-protection.yaml`.
3. `cargo-check`, `cargo-nextest`, `oya-cell-boundary`, `oya-vcs-promotion-readiness` lanes green.
4. Evidence ledger sealed.
5. Runbook published.
6. ADR-0145 status updated.

## Next IP

[`IP-007-scheduler-binpack.md`](IP-007-scheduler-binpack.md)

## References

- ADR-0145 — cell isolation canonical.
- ADR-0183 — Cedar canonical.
- ADR-0117 — residency / per-pack policy.
- `microservices/cell/policy/cell-boundary.md`.
- `registry/quality/lanes.yaml` lane registration schema.
- `sqlparser-rs` — `https://github.com/sqlparser-rs/sqlparser-rs`.
- `cedar-policy` upstream crate.
