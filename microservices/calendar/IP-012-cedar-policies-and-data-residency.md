---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-012-cedar-policies-and-data-residency
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + council-privacy + ops-security
acceptance_lanes: [cedar-validate, oya-governance-dual-context-correctness, oya-governance-data-residency-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: Cedar v4.2 LTS policies + data-residency conformance

## Intent

Author Cedar v4.2 LTS policies for the calendar µservice's
authorisation surface. Default-deny + defence-in-depth FORBID
(per ADR-0140). Cover all 6 BCs + cross-tenant invite gates + T0/T1/T2
autonomy tier gates.

## ChangeSet boundary

6 Cedar policy files (modify in place; per-pack overlays in subdir)
+ Cedar entity schema + Cedar test suite.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/policy/tenant-scope.cedar` | extend | per-BC permit/forbid rules; T0/T1/T2 admission per tenant tier |
| `microservices/calendar/policy/auditor-scope.cedar` | extend | auditor read access to audit-chain events |
| `microservices/calendar/policy/ci-scope.cedar` | extend | CI bot scope |
| `microservices/calendar/policy/public-read.cedar` | extend | public iCal feed export (RFC 5545 read-only) |
| `microservices/calendar/policy/event-isolation.md` | extend | dual-context Cedar fragment + invariant documentation |
| `microservices/calendar/policy/data-residency.md` | extend | per-pack data residency table |
| `microservices/calendar/policy/cedar-schema.cedar` | create | entity types: Calendar, CalendarEvent, Resource, Booking, Invitation, LegalHold |
| `microservices/calendar/policy/pack-kr/` | create | KR-PIPA + KR-FSS overlays |
| `microservices/calendar/policy/pack-eu/` | create | GDPR + ePrivacy overlays |
| `microservices/calendar/policy/pack-us-healthcare/` | create | HIPAA overlays |
| `microservices/calendar/policy/pack-jp/`, etc. | create | per-pack overlays for remaining 8 packs |
| `microservices/calendar/tests/cedar-policies.rs` | create | Cedar policy permit/forbid coverage |

## Acceptance Gates

```bash
cedar validate --schema microservices/calendar/policy/cedar-schema.cedar --policies microservices/calendar/policy/tenant-scope.cedar
cargo run -p oya-dev-cli -- gate validate dual-context-correctness --microservice calendar
cargo run -p oya-dev-cli -- gate validate data-residency-conformance --microservice calendar
```

## Test Plan

- Cedar policy permit/forbid coverage for every PRD §FR-* requirement.
- Cross-tenant invite path — Cedar admit ONLY when explicit grant
  exists (PRD AC-02).
- Personal ↮ Professional structural isolation — Cedar refuses
  cross-context access at the policy layer (defence-in-depth in
  addition to the kernel-layer refusal).

## Halt Conditions

- Any cross-tenant query that Cedar admits which lacks an explicit
  grant — block.
- Any cross-context attempt that Cedar admits — block.

## Next IP

[`IP-013-workflow-handoff.md`](IP-013-workflow-handoff.md)

## References

- ADR-0140 (Cedar policy enforcement).
- Cedar v4.2 LTS — `docs.cedarpolicy.com`.
- PRD-calendar §FR-* + AC-02 + AC-07.
