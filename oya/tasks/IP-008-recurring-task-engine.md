---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-008-recurring-task-engine
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-test, rrule-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: recurring-task engine — RFC 5545 RRULE subset (rrule-rs 0.13)

## Intent

Ship the `recurrence` BC. RFC 5545 RRULE subset aligned with calendar
ADR-CAL-0002 — same `rrule-rs 0.13` LTS pin, same conformance corpus.
Supported features: `FREQ`, `INTERVAL`, `COUNT`, `UNTIL`, `BYDAY`,
`BYMONTHDAY`, `BYMONTH`, `BYWEEKNO`, `BYHOUR`, `BYMINUTE`, `BYSETPOS`,
`WKST`, `EXDATE`. Bounded materialisation horizon = 5 years (per
PRD §FR-05 + ADR-TASKS-0003).

The recurrence worker (background; cron-class) materialises upcoming
task instances on a 24-hour rolling window. Materialised tasks emit
`TaskRecurrenceMaterialised` events for observability.

## ChangeSet boundary

7 recurrence crates (kernel/domain/usecase/api/adapter/worker/app).
Domain layer wraps `rrule-rs 0.13` behind a bounded-horizon façade.

## Crate Naming

`oya-tasks-recurrence-*` per ADR-0056 v4.1.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/oya-tasks-recurrence-{kernel,domain,usecase,api,adapter,worker,app}/src/lib.rs` | created/replaced | 7-crate stack |
| `microservices/tasks/src/oya-tasks-recurrence-domain/tests/rrule_corpus.rs` | created | RFC 5545 conformance corpus |
| `microservices/tasks/catalog/oya-tasks-recurrence-*.yaml` | created | catalog entries |

## Acceptance Gates

```bash
cargo test -p oya-tasks-recurrence-domain
cargo test -p oya-tasks-recurrence-usecase rrule_corpus
cargo run -p oya-dev-cli -- gate validate rrule-conformance --microservice tasks
```

## Test Plan

- RFC 5545 corpus (libical-derived) — 100% pass (AC-03).
- 5-year-horizon enforcement: rule yielding 6-year span refuses to
  materialise beyond 5y; emits truncated marker for the caller.
- DST boundary correctness: a daily-9am-Asia/Seoul rule honours local
  9am across spring-forward + fall-back transitions where IANA tzdb
  is current.
- EXDATE removes the right occurrence (ADR-TASKS-0003 + ADR-CAL-0002
  shared edge-case coverage).

## Halt Conditions

- Any RFC 5545 corpus regression — refuse to ship.
- Horizon enforcement bypassable — refuse.

## Next IP

[`IP-009-state-workflow-engine-cross-link.md`](IP-009-state-workflow-engine-cross-link.md)

## References

- ADR-TASKS-0003 (RRULE engine alignment with calendar).
- ADR-CAL-0002 (RFC 5545 conformance).
- rrule-rs 0.13 — `github.com/fmeringdal/rust-rrule`.
- RFC 5545 — `tools.ietf.org/html/rfc5545`.
