---
doc_class: FailureModes
template_id: TPL-FAILURE-MODES
microservice: calendar
status: Accepted
date: 2026-05-17
owner_team: axis-calendar + ops-sre-reliability
methodology: STAMP + FMEA + Google SRE
related_adrs: [ADR-0130, ADR-0131]
doc_status: published
---

# Failure Modes — calendar µservice

## Purpose

Enumerate failure modes, blast radius, detection signals, automated recovery, and operational runbooks. Each failure mode has at least one runbook in `runbooks/` and at least one SLO + alert in `dashboards/`.

## Failure-mode catalog

### FM-01 — Recurrence-engine OOM (out-of-memory) on complex RRULE

- **Cause:** RRULE with deeply nested BYSETPOS + BYDAY + BYMONTH + EXDATE on 5y horizon exhausts worker heap.
- **Blast radius:** affected worker pod crash; pending expansions queued; tenant sees `RecurrenceWindowExpanded` events delayed.
- **Detection:** Pod OOMKilled event in K8s; worker queue depth > 60s of cadence; `recurrence_expansion_p99_ms` exceeds 1s.
- **Automated recovery:** Worker restart by K8s; pre-warmed pool absorbs; expansion retry with smaller window batches.
- **Runbook:** `runbooks/recurrence-storm.md`.
- **Mitigation hardening:** RRULE complexity bound at API; worker memory request 2GB + limit 4GB; horizon hard-cap 5y.

### FM-02 — Time-zone DB stale (IANA tzdata > 6h old)

- **Cause:** Hourly tzdata refresh job fails (network / upstream IANA outage / config error).
- **Blast radius:** events created after a DST transition may persist with wrong UTC offset; recurrence expansion past the transition incorrect.
- **Detection:** Metric `calendar_tzdata_freshness_hours` > 6; alert at 6h, page at 24h.
- **Automated recovery:** Refresh job retries every 15min; if upstream IANA unreachable, fall back to mirror; if still failing, hold staging promotion.
- **Runbook:** `runbooks/timezone-db-refresh.md`.
- **Mitigation hardening:** Pre-deploy gate refuses promotion if tzdata > 30d stale; chrono-tz pinned version.

### FM-03 — Availability-resolver cache-miss storm

- **Cause:** Synchronous cache TTL expiry across many tenants creates simultaneous cache-miss; Postgres read storm.
- **Blast radius:** Postgres connection pool exhausted; `cross_tenant_availability_p99_ms` > 500ms; cascading 5xx on availability endpoint.
- **Detection:** Cache hit ratio drops < 30% on `oya:calendar_availability_cache_hit_ratio`; Postgres connection-pool utilisation > 85%.
- **Automated recovery:** Single-flight per `(tenant, attendees, window)`; per-key TTL jitter (± 5s); fallback to "unknown" projection when storm exceeds threshold.
- **Runbook:** `runbooks/availability-cache-rebuild.md`.

### FM-04 — Room-booking race condition double-books a room

- **Cause:** Two concurrent booking writes against the same `(resource_id, time_range)`; missing serialization at DB layer.
- **Blast radius:** A single room booked twice; tenant operational confusion; possible meeting collision in physical space.
- **Detection:** Conflict detected by post-write integrity scan; `calendar_room_double_booking_count > 0` alert.
- **Automated recovery:** Postgres `SELECT … FOR UPDATE` on the resource row before INSERT booking; idempotency key on booking request prevents retry race.
- **Runbook:** `runbooks/room-booking-conflict.md`.

### FM-05 — .ics import parse failure on tenant migration

- **Cause:** Tenant uploads .ics file with non-conforming RFC 5545 sequences (CRLF / unfolding bugs / vendor-specific extensions).
- **Blast radius:** Tenant cannot migrate; specific import job stuck `failed`.
- **Detection:** Import job state machine emits `state=failed` Workflow event; `calendar_ics_import_failure_rate > 0.1%` alert.
- **Automated recovery:** Per-event partial-success import: parseable events imported; unparseable surfaced to tenant via report.
- **Runbook:** `runbooks/ics-import-failure.md`.

### FM-06 — Cross-tenant grant revocation does not invalidate cache

- **Cause:** Tenant-A revokes grant to Tenant-B at `t=0`; cached free/busy projection in Redis still served until TTL expires (up to 60s).
- **Blast radius:** Tenant-B continues to see Tenant-A free/busy for up to 60s post-revocation.
- **Detection:** Cache-invalidation lag metric `calendar_cross_tenant_grant_invalidation_lag_seconds > 5s` alert.
- **Automated recovery:** Grant revocation emits explicit Redis DEL for affected cache keys; if Redis DEL fails, force-rotate the cache prefix.
- **Runbook:** `runbooks/availability-cache-rebuild.md`.
- **Mitigation hardening:** Cache TTL ≤ 60s caps blast radius; revocation event chain includes mandatory cache-purge step.

### FM-07 — Postgres replica lag breaks read-after-write consistency

- **Cause:** Postgres replica lags primary by > 5s; user creates event, then refreshes and sees stale state.
- **Blast radius:** Tenant operational confusion; possible duplicate writes.
- **Detection:** `pg_replication_lag_seconds > 5` alert.
- **Automated recovery:** Read-after-write reads are pinned to primary for 30s post-write; reads beyond 30s use replica.
- **Runbook:** `runbooks/calendar-restore.md`.

### FM-08 — Mail µservice outage blocks invitation fanout

- **Cause:** mail µservice unavailable; invitations queued but not delivered.
- **Blast radius:** Tenant attendees do not receive invitations; events created but invitations pending.
- **Detection:** `calendar_invitation_dispatch_queue_depth > 1000` alert + `mail_microservice_availability` from observability.
- **Automated recovery:** Invitations queued with exponential backoff; surfaced to user as "delivery pending"; once mail recovers, fanout resumes.
- **Runbook:** `runbooks/invitation-dispatch-recovery.md` (referenced; see also `mail/runbooks/`).

### FM-09 — Tenant-DEK rotation failure leaves events unreadable

- **Cause:** Tenant-DEK rotation event partially applied; some rows re-encrypted, others not; lookup fails.
- **Blast radius:** Tenant cannot read recent events until rotation completes / rolls back.
- **Detection:** `calendar_dek_rotation_in_flight=true` + reads failing with `dek_mismatch` error.
- **Automated recovery:** Rotation is transactional + idempotent; partial-rotation state recovers on next worker run; old DEK retained read-only until rotation acks complete.
- **Runbook:** `runbooks/dek-rotation-recovery.md`.

### FM-10 — Audit-chain emission failure (silent)

- **Cause:** audit-chain µservice ingest endpoint returns 5xx; calendar's emission ack times out.
- **Blast radius:** Audit-chain seal missing for events created during outage; SOC 2 / ISO 27001 audit-coverage gap.
- **Detection:** `calendar_audit_emission_ack_lag_seconds > 30` alert.
- **Automated recovery:** Calendar event write blocks (fail-closed) when emission ack > 30s; user sees "operation pending due to audit-chain unavailable"; ack-or-fail.
- **Runbook:** `runbooks/audit-chain-emission-recovery.md`.
- **Mitigation hardening:** Fail-closed prevents missing-seal gap.

### FM-11 — CalDAV PROPFIND on huge collection times out

- **Cause:** Tenant has 100k events; CalDAV PROPFIND returns multi-status with all of them; response time > 30s.
- **Blast radius:** Tenant client (Apple Calendar / Thunderbird) times out; client gives up sync.
- **Detection:** `caldav_propfind_p99_seconds > 5` alert.
- **Automated recovery:** Pagination via `<C:limit>` element (RFC 4791 extension); default limit 1000 events; client iterates.
- **Runbook:** `runbooks/caldav-pagination.md`.

### FM-12 — Postgres connection pool exhaustion

- **Cause:** Burst of events + availability lookups + CalDAV reads exhausts the per-pool max-connections.
- **Blast radius:** New requests queued; some time out; cascading 5xx.
- **Detection:** `pg_connection_pool_utilisation > 85%` alert.
- **Automated recovery:** HPA scales rest pods (more pool workers); short-term: rate-limit at REST layer.
- **Runbook:** `runbooks/postgres-connection-storm.md`.

### FM-13 — Cross-pack mesh partition during cross-tenant availability lookup

- **Cause:** Mesh partition between pack-kr and pack-eu; cross-pack availability query times out.
- **Blast radius:** Cross-pack scheduling between KR + EU tenants degrades.
- **Detection:** `calendar_cross_pack_availability_timeout_rate > 5%` alert.
- **Automated recovery:** Cross-pack timeout = 2s; on timeout, return `unknown — please contact attendee` projection; tenant sees degraded experience, not full block.
- **Runbook:** `runbooks/cross-pack-mesh-degradation.md`.

### FM-14 — Workflow event loss (calendar → workflow-engine)

- **Cause:** Workflow event bus (Kafka / NATS) loses an `EventCreated` event; downstream workflow trigger never fires.
- **Blast radius:** Tenant's workflow automation does not run; tenant operational impact.
- **Detection:** Event-bus dropped-msg counter; per-event delivery-ack mismatch.
- **Automated recovery:** Outbox pattern: events written to Postgres `outbox` table + relayed by sidecar; relay retries until ack received.
- **Runbook:** `runbooks/workflow-event-recovery.md`.

### FM-15 — Recurring event update creates exponential expansion explosion

- **Cause:** Update on a recurring event with 10k occurrences triggers re-expansion of all occurrences; CPU spike.
- **Blast radius:** Worker CPU exhaustion; pending events delayed.
- **Detection:** Worker CPU > 90% for > 5min.
- **Automated recovery:** Recurring-event updates expand incrementally (per-occurrence diff); full re-expansion only on RRULE itself changing.
- **Runbook:** `runbooks/recurrence-storm.md`.

## Failure-mode aggregation gates

- `oya gate validate failure-mode-coverage --microservice calendar`: refuses build if any new code path lacks at least one failure-mode entry.
- Quarterly failure-mode review.
- Annual game-day: simulate FM-01, FM-03, FM-10, FM-13 end-to-end.

## References

- ADR-0130: SLO-gated promotion.
- ADR-0131: per-microservice layout.
- `runbooks/*.md` (one per failure mode).
- Google SRE Workbook ch. 6 (managing risk) + ch. 11 (managing incidents).
- NASA-STD-8729.1 (System Theoretic Accident Model and Processes, STAMP).
