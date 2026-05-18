---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-010-sharing-acl-named-range-cedar
status: pending
owner: axis-sheets + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-sheets-range-acl-cedar-required]
depends_on: [IP-007]
---

# IP-010: sharing-acl + named-ranges — full BC crate sets per ADR-SHEETS-0006

## Intent

Author the `sharing-acl` BC (view/comment/edit + per-range named-ACL Cedar policy fragments per ADR-SHEETS-0006) and the `named-ranges` BC (workbook-scope + sheet-scope named-range registry).

## ChangeSet boundary

~13 crates.

## Code Shape

`sharing-acl-domain/src/acl_evaluator.rs` (excerpt):

```rust
pub fn evaluate_range_acl(
    principal: &TenantUser,
    action: AclAction,  // ReadCell, ReadRange, WriteCell, WriteRange, WriteFormula
    resource: &CellOrRange,
) -> AclDecision {
    // Cedar policy evaluation against per-range ACL fragments
    // Default-deny; permit only if resource.range_id in principal.allowed_range_acls
}
```

## Acceptance Gates

```bash
cargo check -p oya-sheets-sharing-acl-kernel ... -p oya-sheets-named-ranges-adapter
cargo nextest run -p oya-sheets-sharing-acl-domain --test test_per_range_acl_hides_pii
cargo run -p oya-dev-cli -- gate validate sheets-range-acl-cedar-required --microservice sheets
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_per_range_acl_hides_pii` | AC-04 invariant per ADR-SHEETS-0006 |
| `test_cedar_fragment_synthesised_from_postgres` | Cedar fragment auto-generated from Postgres ACL rows |
| `test_acl_drift_detection` | quarterly audit catches drift; alert fires |
| `test_named_range_workbook_scope` | named ranges resolve at workbook scope |
| `test_named_range_sheet_scope` | named ranges resolve at sheet scope |

## Halt Conditions

- Cedar fragment generator regression — STOP. T-T-07 + FM-05 critical.
- Per-range ACL not enforced on a read path — STOP.

## Next IP

[`IP-011-ai-formula-smart-fill-foundry-runtime-bridge.md`](IP-011-ai-formula-smart-fill-foundry-runtime-bridge.md)

## References

- PRD FR-11 + FR-18 + AC-04.
- threat-model.md T-T-07.
- ADR-SHEETS-0006 (per-range ACL granularity).
- Cedar v4.2 LTS — `cedarpolicy.com`.
