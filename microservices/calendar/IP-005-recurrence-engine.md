---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-005-recurrence-engine
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-nextest, oya-governance-rfc-5545-conformance, oya-governance-layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: recurrence-engine — kernel + domain + usecase + adapter (`rrule-rs` 0.13.x)

## Intent

Implement the recurrence-engine BC per PRD §"Bounded Contexts" row 2
+ ADR-CAL-0002. Engine: `rrule-rs` 0.13.x LTS. Vendor + wire the
libical + python-dateutil RRULE corpora. Encode the 7 named edge-case
test matrix from ADR-CAL-0002.

## ChangeSet boundary

4 crates: `oya-calendar-recurrence-engine-{kernel,domain,usecase,adapter}`
+ vendored corpora.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/calendar/src/crates/oya-calendar-recurrence-engine-kernel/` | create | RecurrenceExpander port |
| `microservices/calendar/src/crates/oya-calendar-recurrence-engine-domain/` | create | bounded materialisation invariant (5y horizon); DST edge cases |
| `microservices/calendar/src/crates/oya-calendar-recurrence-engine-usecase/` | create | expand-recurrence orchestrator |
| `microservices/calendar/src/crates/oya-calendar-recurrence-engine-adapter/` | create | `rrule-rs` 0.13.x backed adapter |
| `microservices/calendar/tests/corpora/rfc-5545-libical/` | create (vendor) | libical RRULE test corpus + VERSION.txt + DIVERGENCES.md |
| `microservices/calendar/tests/corpora/rfc-5545-python-dateutil/` | create (vendor) | python-dateutil RRULE tests + VERSION.txt |
| `microservices/calendar/tests/rrule_edge_cases.rs` | create | 7 named edge cases per ADR-CAL-0002 |

## Acceptance Gates

```bash
cargo nextest run -p oya-calendar-recurrence-engine-domain -- rfc_5545_libical_corpus
cargo nextest run -p oya-calendar-recurrence-engine-domain -- rfc_5545_python_dateutil_corpus
cargo nextest run -p oya-calendar-recurrence-engine-domain -- rrule_edge_cases
cargo nextest run -p oya-calendar-recurrence-engine-domain -- bound_exceeded
cargo run -p oya-dev-cli -- gate validate rfc-5545-conformance --microservice calendar
```

## Test Plan

- libical corpus 100% pass (≥200 cases).
- python-dateutil corpus 96% pass (4% RFC-divergent cases assert
  RFC-strict).
- 7 named edge cases all pass.
- Bound-exceeded refusal at 5y horizon (PRD AC-10).
- Performance: 1y RRULE expansion p99 ≤ 1s.

## Halt Conditions

- libical corpus regresses to < 100% — block (PRD AC-03).
- 5y bound-exceeded refusal fails — block (PRD AC-10).

## Next IP

[`IP-006-availability-resolver.md`](IP-006-availability-resolver.md)

## References

- ADR-CAL-0002 (engine choice + edge-case matrix).
- RFC 5545 + RFC 5546.
- `rrule-rs` 0.13.x — `crates.io/crates/rrule`.
- libical corpus — `github.com/libical/libical/tree/master/src/test/data`.
- python-dateutil corpus — `github.com/dateutil/dateutil/tree/master/tests`.
