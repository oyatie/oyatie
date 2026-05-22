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
acceptance_lanes: [cargo-nextest, rfc-5545-conformance, recurrence-storm-test]
---

# IP-005: Recurrence engine

## A. Problem
Recurring meetings are a core counterpart feature, and bad recurrence expansion can shift events, leak busy windows, or create recurrence storms during import and free/busy queries.

## B. Approach
Implement the manifest-named `oya-calendar-recurrence-engine-kernel` and `oya-calendar-recurrence-engine-adapter` with bounded RFC 5545 RRULE/EXDATE/RDATE expansion, explicit time-zone handling, and pre-expansion complexity checks.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-calendar-recurrence-engine-kernel.yaml` | Kernel catalog anchor. |
| `catalog/oya-calendar-recurrence-engine-adapter.yaml` | Adapter catalog anchor. |
| `src/crates/oya-calendar-recurrence-engine-kernel/` | Planned trait and value-object crate named by manifest/catalog. |
| `src/crates/oya-calendar-recurrence-engine-adapter/` | Planned RFC 5545 adapter path named by manifest/catalog. |
| `decisions/ADR-CAL-0002-recurrence-engine-rfc-conformance.md` | Decision source for conformance strategy. |

## D. Ordered implementation steps
1. Define recurrence rule, occurrence window, exclusion, inclusion, and expansion-limit types in the kernel.
2. Implement adapter expansion with the selected RFC 5545 library and a deterministic horizon cap.
3. Reject recurrence bombs before allocating unbounded occurrence sets.
4. Preserve floating-time and IANA time-zone semantics explicitly.
5. Add libical-style conformance fixtures and DST edge cases.
6. Add import-path tests that prove recurrence-bomb rejection before ICS persistence.
7. Wire SLO probes for expansion latency and recurrence-storm runbook triggers.

## E. Acceptance
- `cargo nextest run -p oya-calendar-recurrence-engine-kernel` passes.
- `cargo nextest run -p oya-calendar-recurrence-engine-adapter` passes.
- `cargo run -p oya-dev-cli -- gate validate rfc-5545-conformance --microservice calendar` passes.
- Recurrence-storm tests align with `runbooks/recurrence-storm.md`.
- Performance targets align with `performance-benchmark-numbers-2026-05-20.md`.

## F. Evidence
- PRD FR-02, FR-11, recurrence performance targets: `microservices/calendar/PRD.md`.
- Decision: `decisions/ADR-CAL-0002-recurrence-engine-rfc-conformance.md`.
- SLO: `slos/freebusy-query-latency.openslo.yaml`.
- Runbook: `runbooks/recurrence-storm.md`.
- Counterpart benchmark: `benchmarks/gcal-outlook-calendly-vs-oyatie.md`.

## G. Counterpart comparison
Google, Outlook, Apple, Fastmail, and Proton all support recurring events; Cal.com and Calendly support only scheduling-specific recurrence subsets. Oyatie must meet the full-calendar peers on RFC behavior while exceeding point schedulers through bounded expansion, recurrence-bomb refusal, and reproducible time-zone evidence.

## H. Foundation delivery expansion
- Deliverable detail: kernel defines RRULE, RDATE, EXDATE, occurrence window, horizon, and expansion-limit value objects.
- Deliverable detail: adapter wraps the selected RFC 5545 parser behind a deterministic port.
- Deliverable detail: recurrence-bomb detection runs before allocating occurrence vectors.
- Deliverable detail: floating-time behavior is represented explicitly and cannot silently become UTC.
- Deliverable detail: DST fixtures cover spring-forward, fall-back, and historical tzdb revisions.
- Deliverable detail: import adapter errors distinguish unsupported rule, unsafe rule, and malformed input.
- Deliverable detail: SLO instrumentation records expansion count, duration, and refusal reason.
- Deliverable detail: Slack recurring huddle expectations are collaboration pressure but not an RFC substitute.

## I. Acceptance expansion
- Acceptance detail: RFC fixture tests must include weekly, monthly-byday, yearly, exclusion, and inclusion cases.
- Acceptance detail: recurrence-bomb tests must fail fast under the configured horizon cap.
- Acceptance detail: DST tests must preserve local wall-clock intent where RFC semantics require it.
- Acceptance detail: import-path tests must reject unsafe recurrence before event persistence.
- Acceptance detail: performance tests must compare bounded expansion against PRD targets.
- Acceptance detail: runbook links must trigger on recurrence-storm alert names.
- Acceptance detail: adapter crate must not own event-store persistence.
- Acceptance detail: Slack, Google, and Outlook comparisons must be about recurrence UX and meeting cadence parity.

## J. Evidence expansion
- Evidence detail: capture nextest output for recurrence kernel and adapter crates.
- Evidence detail: capture RFC conformance gate output.
- Evidence detail: capture recurrence-storm fixture names and refusal counts.
- Evidence detail: cite `ADR-CAL-0002` for parser/conformance strategy.
- Evidence detail: cite `runbooks/recurrence-storm.md` for operator response.
- Evidence detail: cite `performance-benchmark-numbers-2026-05-20.md` for expansion targets.
- Evidence detail: cite Slack as recurring collaboration pressure alongside Google and Outlook calendar behavior.
