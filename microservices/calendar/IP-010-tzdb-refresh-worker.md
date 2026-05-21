---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-010-tzdb-refresh-worker
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + ops-sre-reliability
acceptance_lanes: [cargo-nextest, cronjob-dry-run, tzdb-staleness-slo]
---

# IP-010: TZDB refresh worker

## A. Problem
Calendar cannot provide reproducible historical events or regulated scheduling if time-zone database changes silently rewrite past occurrences.

## B. Approach
Implement `oya-calendar-tzdb-refresh-worker` as the manifest/catalog-named worker and bind it to the existing Helm CronJob. The worker fetches, verifies, stages, soaks, promotes, and can roll back tzdb versions while preserving per-occurrence `tzdb_version` evidence.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-calendar-tzdb-refresh-worker.yaml` | Existing worker catalog anchor. |
| `src/crates/oya-calendar-tzdb-refresh-worker/` | Planned worker path named by manifest/catalog. |
| `iac/helm/templates/cronjob.yaml` | Runtime binding. |
| `slos/tzdb-staleness-bound.openslo.yaml` | Staleness promotion SLO. |
| `runbooks/timezone-db-refresh.md` and `runbooks/tzdb-rollback.md` | Operator closure. |

## D. Ordered implementation steps
1. Implement source fetch and signature/hash verification.
2. Stage candidate tzdb versions without changing active scheduling decisions.
3. Run regression checks over recurrence and existing occurrence fixtures.
4. Promote only after soak and SLO checks pass.
5. Preserve historical occurrence version pins.
6. Add rollback command and operator evidence emission.
7. Wire CronJob status to observability and alert rules.

## E. Acceptance
- `cargo nextest run -p oya-calendar-tzdb-refresh-worker` passes.
- `kubectl --dry-run=client apply -f microservices/calendar/iac/helm/templates/cronjob.yaml` or chart-rendered equivalent passes.
- `slos/tzdb-staleness-bound.openslo.yaml` resolves.
- ADR-CAL-0004 checks pass.
- Rollback runbook validates previous-version restoration.

## F. Evidence
- Decision: `decisions/ADR-CAL-0004-tzdb-refresh-and-pinning-policy.md`.
- PRD timezone and DST requirements: `microservices/calendar/PRD.md`.
- Runbooks: `runbooks/timezone-db-refresh.md`, `runbooks/tzdb-rollback.md`.
- Benchmark: `performance-benchmark-numbers-2026-05-20.md`.

## G. Counterpart comparison
Google and Outlook update time zones for users but offer limited tenant-visible pinning; Calendly and Cal.com expose even less control. Oyatie's counterpart advantage is reproducible historical occurrence behavior with staleness SLOs and rollback evidence.

## H. Foundation delivery expansion
- Deliverable detail: worker records candidate, active, previous, and rollback tzdb versions.
- Deliverable detail: source fetch includes signature or hash verification before staging.
- Deliverable detail: soak checks run recurrence and occurrence fixtures before promotion.
- Deliverable detail: promotion writes audit evidence and leaves historical occurrence pins unchanged.
- Deliverable detail: rollback restores the previous active version and records affected tenants.
- Deliverable detail: CronJob status emits last success, last failure, and active version metrics.
- Deliverable detail: alerting distinguishes stale source, failed verification, failed soak, and failed promotion.
- Deliverable detail: Slack scheduled reminders create collaboration pressure for accurate time-zone refreshes.

## I. Acceptance expansion
- Acceptance detail: signature/hash tests must reject tampered tzdb payloads.
- Acceptance detail: soak tests must compare recurrence output before and after candidate staging.
- Acceptance detail: promotion tests must preserve historical occurrence version pins.
- Acceptance detail: rollback tests must restore previous active version and audit correlation.
- Acceptance detail: CronJob dry-run must prove image pinning and SecretReference wiring.
- Acceptance detail: SLO checks must alert when active tzdb exceeds staleness bound.
- Acceptance detail: runbooks must include operator commands and evidence artifacts.
- Acceptance detail: Slack/Google/Outlook comparisons must focus on time correctness for distributed scheduling.

## J. Evidence expansion
- Evidence detail: capture nextest output for `oya-calendar-tzdb-refresh-worker`.
- Evidence detail: capture chart-rendered CronJob dry-run output.
- Evidence detail: capture `tzdb-staleness-bound` SLO resolution.
- Evidence detail: cite `ADR-CAL-0004` for pinning policy.
- Evidence detail: cite `runbooks/timezone-db-refresh.md` for promotion.
- Evidence detail: cite `runbooks/tzdb-rollback.md` for rollback.
- Evidence detail: cite Slack as collaboration scheduling pressure where stale tzdb causes visible meeting drift.
