---
id: ADR-SLIDES-0008
title: Chart-live-link to sheets µservice — eventual consistency + revocation cascade
microservice: slides
status: Accepted
date: 2026-05-17
owner: axis-workspace + sheets-team + ops-security
deciders: council-architecture, axis-workspace, sheets-team, ops-security
supersedes: []
superseded_by: []
related: [ADR-0105, ADR-0135, ADR-0131]
related_specs: []
related_artifacts:
  - microservices/slides/PRD.md (FR-09, AC-11, AC-19)
  - microservices/slides/PHASE-01-SLIDES-FOUNDATION.md (IP-007)
  - microservices/slides/failure-modes.md (FM-12, FM-13)
  - microservices/slides/runbooks/share-acl-drift.md
purpose: Establish the cross-µservice chart-live-link contract between slides and sheets — consistency model, refresh policy, revocation cascade behavior, and audit guarantees.
doc_status: published
---

# ADR-SLIDES-0008: Chart-live-link to sheets — eventual consistency, on-open / scheduled refresh, end-to-end revocation cascade

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

A canonical workspace use case: tenant authors a slide with a chart whose data comes from a sheets cell range. When the underlying sheet data changes, the chart should update in the deck without manual re-author work. This is competitor parity with Google Slides ↔ Sheets and PowerPoint Web ↔ Excel.

The slides + sheets co-authoring (both net-new under ADR-0135) gives oyatie an opportunity to establish a contract that is more rigorous than competitors:

1. **Consistency model**: Google Slides ↔ Sheets is eventual but the refresh is on-deck-open + on-explicit-refresh; PowerPoint Web ↔ Excel is similar. Strong consistency (block deck save until refresh confirmed) is too costly for usual workflows.
2. **Revocation cascade**: when a sheet's ACL is revoked for a viewer, the chart in a deck embedded in slides should reflect the revocation. Google Slides + Sheets has this but the cascade is delayed and inconsistent (sometimes shows stale data for hours).
3. **Pack-residency consistency**: cross-pack chart-live-link is forbidden by `policy/data-residency.md`; sheets and slides must be in the same pack.

PRD Open Question 7 — Chart-live-link consistency model: strong vs eventual; bias: eventual with explicit stale-marker UI.

## Decision

Adopt the following chart-live-link contract:

1. **Bind contract**: tenant binds a chart to `(sheets_deck_id, cell_range, refresh_policy)` via slides REST `/decks/{deck_id}/charts/bind`. Slides invokes sheets SDK to:
   - Verify caller has sheets-side READ permission on the cell range.
   - Verify pack residency match (slides deck pack == sheets deck pack).
   - Issue a chart-binding token (slides-owned) carrying the binding-id and sheets-side context.
2. **Refresh policy enum**: `eventual` (default) | `on_open` | `manual`.
   - `eventual`: sheets emits `SheetsCellRangeUpdated` event to the workflow bus; slides consumes and refreshes within p95 ≤ 200ms (per PRD §"Performance").
   - `on_open`: refresh triggered at deck-open + on-explicit-refresh button only; no event subscription.
   - `manual`: refresh only on tenant click of refresh button.
3. **Stale marker UI**: when refresh is in-flight OR sheets µservice unavailable, chart shows a stale-data overlay with timestamp of last successful refresh. Tenant-visible, never silently stale.
4. **Revocation cascade contract** — load-bearing (AC-19): when sheets emits `SheetsAclRevoked` for a cell range that has a slides chart binding, slides processes the event and:
   - Within p95 ≤ 5s, marks the chart as "access revoked"; renders a revocation marker (last successful render becomes the cached value, but with a "data access revoked" visual overlay and a tenant-visible audit log entry).
   - The data displayed in the cached chart is NOT removed (tenant has already seen it; we don't time-travel revoke); but no future refresh can occur for the revoked principal.
   - Audit row emitted (`ChartLinkRevoked`).
   - If the deck author then attempts to refresh, refresh is refused with a Cedar-grade error message.
5. **Pack-residency**: slides verifies pack match at bind time; cross-pack chart binding refused.
6. **Chart-binding lifecycle**: binding survives sheet rename + cell-range shift via sheets-owned binding stability (sheets tracks bindings and updates the cell-range pointer; slides receives `SheetsCellRangeUpdated` with new range).
7. **Sheets-side ACL is the authority**: slides does NOT cache sheet ACL state; every sheets-side ACL change is reflected via the event bus. The chart's read access is sheets-side authoritative.
8. **Public-share-link interaction**: public-share-link viewer can see a chart's last cached render IF the chart was rendered while the public-share-link existed AND the sheets binding allows public read. If sheets binding does not allow public read, the chart shows "data access restricted" overlay for public-share-link viewers.
9. **Chart-binding revocation by deck author**: explicitly unbind via REST DELETE; the cached render is purged from S3.

## Alternatives Considered

### A — Strong consistency: block deck save until chart refresh confirmed

- **Pros**: Tenant never sees stale data.
- **Cons**: Couples slides save latency to sheets refresh availability (sheets down → slides can't save). Save round-trip p95 ≤ 100ms invariant unattainable when sheets is sluggish. Tenant frustration.
- **Rejected reason**: latency + availability coupling.

### B — Snapshot-only: chart is a one-time copy of cell range; no live-link

- **Pros**: Trivial implementation.
- **Cons**: Competitor parity lost (Google Slides + PowerPoint Web have live-link).
- **Rejected reason**: parity.

### C — Embed sheets URL as image (no native chart)

- **Pros**: Generic embed pattern; works for many cross-µservice cases.
- **Cons**: Chart-as-image loses tenant authoring UX (chart type change, color, axis labels); also loses revocation cascade.
- **Rejected reason**: UX + revocation.

### D — Slides-side data caching with periodic poll

- **Pros**: No event bus dependency.
- **Cons**: Periodic poll adds latency to refresh; revocation cascade slower; load on sheets µservice.
- **Rejected reason**: event-driven is cleaner.

### E — Cross-µservice direct DB query (slides reads sheets Postgres)

- **Pros**: Lowest latency.
- **Cons**: Violates LEAN-A2 cross-product-refusal lane; violates per-µservice DB ownership invariant.
- **Rejected reason**: architectural invariant.

### F — Slides-side ACL cache (cache sheets ACL state)

- **Pros**: Cached evaluations save SDK calls.
- **Cons**: Cache drift → stale ACL → potential cross-tenant leak (per `threat-model.md` T-I-06). Source-of-truth divergence.
- **Rejected reason**: drift risk too high.

## Consequences

### Architectural

- `chart` BC crates: `oya-slides-chart-{kernel, domain, usecase, api, adapter, sdk}`.
- Kernel port `ChartLiveLinkProvider` invokes sheets SDK; sheets SDK is the only consumer.
- Chart binding persisted in Postgres (Citus by tenant_id) with `(binding_id, deck_id, slide_id, sheets_deck_id, cell_range, refresh_policy, last_refresh_at, access_status)`.
- Cached render stored in S3 (per-tenant prefix); served via CDN per-tenant cache key.
- Workflow-bus subscription for `SheetsCellRangeUpdated` + `SheetsAclRevoked` events.
- Revocation cascade has dedicated SLI (≤ 5s p95).
- Cross-pack binding refused at bind-time.

### Downstream impact on other µservices and IPs

1. **IP-007 (chart embed-bridge to sheets)** — authors the contract.
2. **sheets µservice** — emits `SheetsCellRangeUpdated` + `SheetsAclRevoked` events; provides SDK for bind + verify + refresh.
3. **audit-chain µservice** — Ed25519-sealed `ChartLinkBound` + `ChartLinkRevoked` events.
4. **observability µservice** — slides-specific chart SLIs (refresh latency, stale rate, revocation-cascade latency).
5. **embed-bridge BC (cross-µservice)** — chart is the canonical example of the embed-bridge pattern; docs (quotes) + forms (polls) follow.

### SLOs gaining new dimensions

- `slides.chart_refresh_p95_seconds` — target ≤ 0.2s.
- `slides.chart_stale_rate` — target ≤ 0.01 (1% of charts in stale state at any moment).
- `slides.chart_revocation_cascade_p95_seconds` — AC-19 invariant; target ≤ 5s.

### CI lanes added

- `oya-governance-chart-revocation-cascade-bounded` — runs the revocation cascade chaos test with sheets ACL revoke trigger; asserts cascade ≤ 5s.
- `oya-governance-chart-pack-residency` — asserts bind refused on cross-pack.

### Risk register

- **Risk**: Workflow bus event lost / delayed → revocation cascade exceeds 5s. **Mitigation**: periodic reconciliation cron (every 60s) + dead-letter alarm.
- **Risk**: Sheets µservice down → no refreshes possible. **Mitigation**: tenant-visible stale marker; chart shows last cached render with timestamp.
- **Risk**: Sheets binding cell range shifts on sheet edit; chart binding invalid. **Mitigation**: sheets-side binding stability (sheets emits `SheetsCellRangeUpdated` with new range); slides updates binding without tenant action.
- **Risk**: Cached chart render preserves data after sheet deletion. **Mitigation**: sheet deletion emits `SheetsDeleted`; slides cascades to chart binding deletion + cached render purge.
- **Risk**: Cross-tenant leak via shared chart-render S3 prefix. **Mitigation**: per-tenant prefix; per-tenant CDN cache key; verified by lane.
- **Risk**: Public-share-link viewer accesses chart whose sheet ACL was later revoked. **Mitigation**: cached render shown WITH "data access restricted" overlay; per `policy/public-read.cedar`.

## References

- PRD `microservices/slides/PRD.md` FR-09, AC-11, AC-19.
- `microservices/slides/policy/tenant-scope.cedar`.
- `microservices/slides/policy/public-read.cedar`.
- `microservices/slides/failure-modes.md` FM-12, FM-13.
- `microservices/slides/threat-model.md` T-I-06.
- ADR-0140 (Cedar policy enforcement).
- ADR-0105 (backend-qualified adapters).
- ADR-0131 (per-microservice flat layout).
- sheets µservice PRD + ADR family.
- Google Slides ↔ Sheets chart-live-link reference — `support.google.com/docs/answer/9050447`.
