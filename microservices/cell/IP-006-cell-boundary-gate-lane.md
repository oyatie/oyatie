---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-cell-substrate
impl_plan_id: IP-006-cell-boundary-gate-lane
status: pending
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-nextest, oya-cell-boundary]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: `oya-cell-boundary` BLOCKER CI lane

## Intent

Author a new BLOCKER LEAN lane `oya-cell-boundary` that:
1. Greps every workload µservice's SQL queries for any cross-cell DB reference (`JOIN` across cell prefixes, `UNION ALL` across cell schemas).
2. Validates K8s manifests for cross-namespace references between `cell-*` namespaces.
3. Validates Cedar policy fragments don't accidentally permit cross-pack writes.
4. Validates Postgres migration files include RLS clauses on every cell-scoped table.

Lane is registered in `.github/branch-protection.yaml` as a required status check on `dev` + `staging`.

## Concrete File Targets

| Path | Action |
|---|---|
| `crates/oya-check-cell-boundary/Cargo.toml + src/lib.rs` | create |
| `crates/oya-check-cell-boundary/tests/cell_boundary_lane.rs` | create |
| `crates/oya-dev-cli/src/governance_gates.rs` | update (register cell-boundary gate) |
| `/specs/quality/lanes.yaml` | update (declare oya-cell-boundary BLOCKER) |
| `microservices/cell/specs/cell-boundary-lane.json` | create (lane spec) |

## Code Shape

```rust
// crates/oya-check-cell-boundary/src/lib.rs
pub struct CellBoundaryCheck { config: CellBoundaryConfig }

impl CellBoundaryCheck {
    pub fn run(&self, manifest_root: &Path) -> Result<CheckReport, CheckError> {
        let mut violations = vec![];

        // 1. Grep SQL files for cross-cell JOINs
        violations.extend(self.scan_sql_files(manifest_root)?);

        // 2. Inspect K8s manifests for cross-namespace refs
        violations.extend(self.scan_k8s_manifests(manifest_root)?);

        // 3. Validate Cedar fragments
        violations.extend(self.validate_cedar_fragments(manifest_root)?);

        // 4. Validate Postgres migrations include RLS
        violations.extend(self.validate_postgres_migrations(manifest_root)?);

        Ok(CheckReport { violations })
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-check-cell-boundary
cargo run -p oya-dev-cli -- gate validate cell-boundary --microservice <ms>
```

## Test Plan

- Fixture-based tests: 5 synthetic workload µservice trees with deliberate cross-cell violations; lane catches each.
- Cedar fuzz: random Cedar fragments tested for accidental cross-pack permits.
- Postgres migration test: migration missing RLS → lane fails.

## Halt Conditions

- Lane false-negatives (missed a real violation) — must catch before declaring lane "BLOCKER".
- Lane runtime > 60s — optimize.

## Next IP

[`IP-007-scheduler-binpack.md`](IP-007-scheduler-binpack.md)

## References

- `microservices/cell/policy/cell-boundary.md`.
- `/specs/quality/lanes.yaml`.
- ADR-0140 (Cedar).
