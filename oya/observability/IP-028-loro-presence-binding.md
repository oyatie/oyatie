---
microservice: observability
ip: IP-028
title: Loro presence binding (CRDT awareness protocol → per-cell subscription manager)
status: Drafting
owner: axis-observability
co_owners: [axis-frontend]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0204, ADR-0208]
---

# IP-028 — Loro presence binding

## Purpose

Wire `oya-shared-presence-kernel::LoroPresenceTracker` to the cell-local subscription manager so Workflow Studio canvas + similar surfaces can replicate awareness state across collaborators (per ADR-0145 Loro pin + ADR-0204 canvas integration).

## Acceptance criteria

1. `oya-observability-presence-adapter` crate wires Loro awareness to Valkey pub-sub.
2. Per-tenant + per-room isolation (kernel invariant).
3. Stale-entry pruning at 30s idle.
4. Cursor coordinates validated (kernel rejects NaN / Inf).
5. ≥ 5 integration tests.

## Cross-references

- ADR-0145 — Loro pin.
- ADR-0204 — canvas.
- `oya-shared-presence-kernel`.

## Wave 15 substance conversion

### A. Problem this IP closes

Presence in observability is not collaborative document editing. It is operator awareness for shared dashboards, incident rooms, tail-sampling debug views, and dashboard-share sessions where multiple people inspect the same burn-rate or trace-loss evidence.
The prior IP imported Workflow Studio canvas language without defining the observability-specific presence contract.
This IP closes the presence binding for observability views while leaving CRDT document content ownership to the product surfaces that own those documents.

### B. Approach

Bind `oya-shared-presence-kernel` and Loro awareness to observability session rooms keyed by tenant, cell, channel kind, dashboard/view ID, and incident ID where applicable.
Presence payloads are intentionally small: principal display bucket, role, cursor/view focus, active panel ID, last heartbeat, and redacted status.
No raw trace/log/profile payload is stored in presence state.
Presence is replicated cell-locally through the subscription manager and expires after 30 seconds of idle.

### C. Deliverables

- Add crate or module `oya-observability-presence-adapter`.
- Define room keys for `gate-eligibility`, `operator-burn-rate`, `tail-sampling-debug`, and `incident-dashboard`.
- Add presence message schema for join, leave, heartbeat, cursor/focus update, and prune.
- Add integration tests for per-tenant room isolation, stale prune, invalid coordinates/focus ID, cross-cell route, and redacted display data.
- Update dashboard-share and realtime transport docs/runbooks to mention presence metrics.

### D. Implementation steps

1. Inventory existing dashboards: `gate-eligibility.json`, `operator-burn-rate.json`, `tenant-slo-overview.json`, and tail-sampling/clickhouse dashboards.
2. Define `PresenceRoomId` as tenant + cell + channel + view ID, rejecting unscoped rooms.
3. Define `PresenceUser` with principal ID hash, role, and safe display label; never raw email or tenant customer name.
4. Implement Loro awareness bridge using shared presence kernel types and Valkey pub-sub or equivalent cell-local manager.
5. Prune stale entries after 30 seconds idle and emit prune metrics.
6. Validate cursor/focus coordinates and reject NaN/Inf or unknown panel IDs.
7. Replicate only presence deltas, not dashboard query results.
8. Add cross-cell routing decision: local room by default; explicit cross-cell only for approved incident rooms.
9. Add tests for tenant A not seeing tenant B presence in same dashboard ID.
10. Add runbook evidence for presence leak or stale-session incident.

### E. Acceptance

- Presence room IDs cannot be constructed without tenant and cell scope.
- Stale entries are pruned within 30 seconds.
- Presence payload excludes raw telemetry and raw PII.
- Tests prove cross-tenant and unauthorized cross-cell presence reads fail.
- Observability dashboards can subscribe to presence without depending on Workflow Studio code.

### F. Evidence

- `microservices/observability/dashboards/gate-eligibility.json`.
- `microservices/observability/dashboards/operator-burn-rate.json`.
- `microservices/observability/dashboards/tenant-slo-overview.json`.
- `crates/oya-observability-domain/src/lib.rs` data exposure rules.
- `microservices/observability/runbooks/realtime-transport-connection-leak.md`.

### G. Counterpart closure

| Counterpart | Presence expectation | This IP closure |
|---|---|---|
| Grafana | shared dashboard awareness | room-scoped panel focus and active viewers |
| Datadog | incident collaboration context | incident-dashboard presence without raw telemetry |
| New Relic | team debugging views | scoped viewer/focus state |
| Slack | lightweight presence reference, not chat | presence only; messaging remains out of scope |
