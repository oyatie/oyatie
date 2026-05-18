---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-015-hg-calendar-registration-and-branch-protection
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + ops-release-management
acceptance_lanes: [branch-protection-validate, oya-governance-hyperscaler-maturity-claims, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-CALENDAR registration + branch-protection

## Intent

Register HG-CALENDAR as a BLOCKER lane in
`.github/branch-protection.yaml`. Calendar promotion past dev requires
HG-CALENDAR green per ADR-0123 + ADR-0139. Wire all 9 OpenSLO manifests
+ all per-BC CI lanes into branch-protection.

## ChangeSet boundary

`.github/branch-protection.yaml` + `/registry/quality/lanes.yaml`
updates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `.github/branch-protection.yaml` | extend | add HG-CALENDAR + per-BC + per-SLO lanes |
| `/registry/quality/lanes.yaml` | extend | per-lane metadata for new HG-CALENDAR lanes |
| `/registry/claim-matrix/ops-portal.json` | extend | claim ownership for calendar lanes |

## New BLOCKER lanes registered

- `oya-governance-rfc-5545-conformance` — per ADR-CAL-0002.
- `oya-governance-rfc-4791-conformance` — per ADR-CAL-0001.
- `oya-governance-caldav-backend-conformance` — per ADR-CAL-0001.
- `oya-governance-tzdb-staleness-bound` — per ADR-CAL-0004.
- `oya-governance-dual-context-correctness` — per PRD AC-07.
- `oya-governance-room-conflict-correctness` — per PRD AC-09.
- HG-CALENDAR — composite gate per ADR-0123.

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate branch-protection-validate
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice calendar
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice calendar
```

## Test Plan

- branch-protection.yaml parses + all referenced lanes exist in
  registry.
- HG-CALENDAR composite is consistent with the SLO files +
  conformance tests.

## Halt Conditions

- Any referenced lane missing from registry — block.
- HG-CALENDAR composite trips on a known-passing setup — block;
  root-cause.

## Phase exit

This IP closes M03-connect-dissolution-phase-01-calendar-foundation
phase. After all 15 IPs land, the calendar µservice is "Phase 1
exit-gate ready" per ADR-0134 phase model (parallel ship; legacy
`oya-connect-calendar-*` still serves traffic; new `oya-calendar-*`
serves canary).

## References

- ADR-0123; ADR-0139; ADR-0131; ADR-0134.
- `.github/branch-protection.yaml`.
- `/registry/quality/lanes.yaml`.
- `microservices/calendar/PHASE-01-CALENDAR-FOUNDATION.md`.
