---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-014-hg-calendar-authority-cohesion
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + council-architecture
acceptance_lanes: [oya-governance-hyperscaler-maturity-claims, oya-governance-authority-cohesion]
---

# IP-014: HG-CALENDAR authority cohesion — hyperscaler-maturity claim wiring

## Intent

Register the calendar µservice as a hyperscaler-maturity-claim
participant per ADR-0123 (HG-CALENDAR gate). Wire the authority-
cohesion check between PRD ↔ specs/microservices/calendar/calendar.json ↔
catalog records ↔ SLO authoring.

## ChangeSet boundary

Spec promotion + authority-cohesion test wiring; no new crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `/specs/microservices/calendar/calendar.json` | create (promote) | promote from `/specs/microservices/calendar.json` per ADR-0134 Phase 5 |
| `/specs/microservices/calendar.json` | mark deprecated | set `deprecated: true`; `replacement_path: /specs/microservices/calendar/calendar.json` |
| `/specs/hyperscaler-gates.json` | extend | register HG-CALENDAR with SLO references |
| `microservices/calendar/tests/authority-cohesion.rs` | create | per ADR-0123 cohesion checks |

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice calendar
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice calendar
```

## Test Plan

- PRD claims must trace to spec entries + catalog rows + SLO files.
- All 9 OpenSLO manifests register in HG-CALENDAR's `slo_files` array.
- All 11-pack overlays cited in PRD §Pack scope are present.

## Halt Conditions

- Any PRD claim without a corresponding spec/catalog/SLO trace — block.

## Next IP

[`IP-015-hg-calendar-registration-and-branch-protection.md`](IP-015-hg-calendar-registration-and-branch-protection.md)

## References

- ADR-0123 (HG gate); ADR-0131; ADR-0134.
- `/specs/hyperscaler-gates.json`.
