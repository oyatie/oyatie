---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-012-per-microservice-layout-lane
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, per-microservice-layout]
---

# IP-012: oya-check-per-microservice-layout lane (BLOCKER on dev)

## Intent

Author the new BLOCKER lane that enforces ADR-0131 per-microservice flat layout. Refuses:
- Out-of-layout artifacts (PRD / phase / IP / catalog row / runbook / threat-model / OpenSLO at any location other than the owning µservice's folder).
- Crate creation outside `microservices/<ms>/src/crates/`.
- Hand-edits to central aggregation indices.

## ChangeSet boundary

New crate + activation in branch-protection.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-check-per-microservice-layout/Cargo.toml` | create |
| `…/src/main.rs` | create — CLI |
| `…/src/checks/{prd,phase,ip,catalog,runbook,threat_model,openslo,crates}.rs` | create — one per artifact class |
| `.github/branch-protection.yaml` | edit — add to `required_status_checks` |
| `microservices/governance/catalog/oya-check-per-microservice-layout.yaml` | create |

## Code Shape

```rust
// src/checks/prd.rs
pub fn check_prd_locations(repo_root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    // PRDs MUST live at microservices/<ms>/PRD.md
    // Any other PRD location is a violation (e.g., docs/prds/, docs/products/<p>/PRD.md)
    for entry in walk_repo(repo_root) {
        if entry.file_name() == "PRD.md" {
            let parent = entry.parent();
            if !matches_pattern(parent, "microservices/<ms>/") {
                findings.push(Finding::new(
                    "per-microservice-layout.prd-out-of-place",
                    BLOCKER,
                    "ADR-0131 §canonical folder shape",
                ).with_file_line(entry));
            }
        }
    }
    findings
}
```

## Acceptance Gates

```bash
cargo check -p oya-check-per-microservice-layout
cargo nextest run -p oya-check-per-microservice-layout
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice governance
# Self-application: governance µservice's own files all live within microservices/governance/
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice observability
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_prd_in_correct_location` | PRD.md at µservice root |
| `test_crate_in_correct_location` | crates under src/crates/ |
| `test_catalog_row_per_crate` | one yaml per crate |
| `test_self_application` | governance itself passes |
| `test_refuses_legacy_paths` | docs/prds/* refused (after migration) |

## Halt Conditions

- Self-application fails → fix governance layout in same PR.
- Legacy paths still present (pre-migration) → lane emits `legacy-grandfathered` findings; not BLOCKER until migration complete per ADR-0131 §"Strangler pattern".

## Next IP

[`IP-013-aggregation-index-generation-lane.md`](IP-013-aggregation-index-generation-lane.md)

## References

- ADR-0131 (per-microservice flat layout).
- `microservices/governance/runbooks/aggregation-rebuild.md`.
