---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-006-availability-resolver
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-nextest, oya-governance-layer-correctness, oya-governance-dual-context-correctness]
---

# IP-006: Availability resolver

## A. Problem
Cross-tenant scheduling is only defensible if free/busy answers never expose event titles, attendees, locations, or personal/professional context details.

## B. Approach
Implement the manifest-named availability resolver kernel plus Valkey adapter as a minimum-necessary projection service. Cedar evaluates the caller and grant before cache or repository access; cache keys include tenant, context, attendee bucket, window, and policy version.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-calendar-availability-resolver-kernel.yaml` | Kernel catalog anchor. |
| `catalog/oya-calendar-availability-resolver-adapter-valkey.yaml` | Valkey adapter catalog anchor. |
| `src/crates/oya-calendar-availability-resolver-kernel/` | Planned port/value crate named by manifest/catalog. |
| `src/crates/oya-calendar-availability-resolver-adapter-valkey/` | Planned cache adapter path named by manifest/catalog. |
| `slos/freebusy-query-latency.openslo.yaml` | Latency and correctness promotion evidence. |

## D. Ordered implementation steps
1. Define `FreeBusyProjection` and `CrossTenantInviteGrant` types with no raw-event fields.
2. Implement policy-first query orchestration through tenant and context claims.
3. Add cache read/write with policy-versioned keys and bounded TTL.
4. Add remote-tenant outage behavior returning explicit unknown status, not leaked metadata.
5. Add tests for personal-to-professional isolation and cross-tenant minimum-necessary disclosure.
6. Add latency tests for 10, 50, and 100-attendee windows.
7. Wire runbook hooks for cache rebuild and permission drift.

## E. Acceptance
- `cargo nextest run -p oya-calendar-availability-resolver-kernel` passes.
- `cargo nextest run -p oya-calendar-availability-resolver-adapter-valkey` passes.
- `buck2 build //:quality-lane-registry-authority-check # lane=dual-context-correctness --microservice calendar` passes.
- SLO check resolves `slos/freebusy-query-latency.openslo.yaml`.
- Runbook closure uses `runbooks/availability-cache-rebuild.md` and `runbooks/shared-cal-permission-drift.md`.

## F. Evidence
- PRD FR-03 and FR-10: `microservices/calendar/PRD.md`.
- Policy: `policy/event-isolation.md`, `policy/public-read.cedar`, `policy/tenant-scope.cedar`.
- Tutorial: `tutorials/configure-freebusy-acl-cross-tenant-interview.md`.
- Counterpart matrix: `feature-parity-matrix-2026-05-20.md`.

## G. Counterpart comparison
Google freebusy and Microsoft getSchedule set the baseline. Cal.com and Calendly cover booking availability but not tenant-policy projection. Oyatie's required counterpart advantage is Cedar-gated free/busy with explicit unknown/degraded states and no cross-context raw-event leakage.

## H. Foundation delivery expansion
- Deliverable detail: define free/busy projection records that omit title, location, attendee list, and description.
- Deliverable detail: model `Unknown`, `Unavailable`, `Busy`, and `Tentative` states separately.
- Deliverable detail: cache keys include tenant, context, subject bucket, query window, and policy version.
- Deliverable detail: resolver evaluates Cedar before cache lookup to avoid policy-bypass hits.
- Deliverable detail: remote tenant outages return unknown/degraded status, not inferred private availability.
- Deliverable detail: latency probes cover small, medium, and 100-attendee windows.
- Deliverable detail: cache rebuild logic emits permission-drift evidence.
- Deliverable detail: Slack shared availability and channel scheduling are explicit interop pressure for this projection.

## I. Acceptance expansion
- Acceptance detail: free/busy fixtures must prove raw event fields never appear in projections.
- Acceptance detail: cross-tenant tests must pass only with a valid invite grant.
- Acceptance detail: cache tests must invalidate on policy-version changes.
- Acceptance detail: outage tests must prove remote failures do not leak whether a hidden event exists.
- Acceptance detail: dual-context tests must isolate personal and work calendars.
- Acceptance detail: SLO tests must include latency and correctness dimensions.
- Acceptance detail: runbooks must cover cache rebuild and permission drift separately.
- Acceptance detail: Slack/Google/Microsoft comparisons must be backed by projection privacy evidence.

## J. Evidence expansion
- Evidence detail: capture nextest output for resolver kernel and Valkey adapter.
- Evidence detail: capture dual-context gate output for calendar.
- Evidence detail: capture SLO resolution for `freebusy-query-latency`.
- Evidence detail: cite `tutorials/configure-freebusy-acl-cross-tenant-interview.md` for user-visible setup.
- Evidence detail: cite `runbooks/availability-cache-rebuild.md` for cache repair.
- Evidence detail: cite `runbooks/shared-cal-permission-drift.md` for access drift.
- Evidence detail: cite Slack as collaboration-calendar pressure that justifies explicit minimum-necessary projection.
