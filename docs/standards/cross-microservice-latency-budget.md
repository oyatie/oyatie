---
contract: cross-microservice-latency-budget
authored: 2026-05-18
canonical_authority: ADR-0067 (perf authority) + ADR-0128 (hyperscaler-architecture-invariants) + ADR-0139 (agentic SLO-gated promotion)
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/agentic-slo-gated-promotion.json
related_adrs:
  - ADR-0067
  - ADR-0128
  - ADR-0139
  - ADR-0141 (retired per ADR-0145)
status: canonical-base
overlay_consumers:
  - microservices/<ms>/slos/<sli>.openslo.yaml
  - microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml
authorities_cited:
  - Google SRE Workbook ch. 5 (multi-window multi-burn-rate)
  - Amazon Builders' Library — "Avoiding overload in distributed systems"
  - Stripe engineering — "Online migrations at scale"
  - Linear engineering — per-service deploy refs
  - Cloudflare engineering — Workers latency budgets
---

# Cross-microservice latency budget allocation (PERF-143-005)

This document establishes **end-to-end p99 latency budgets** for the
highest-traffic cross-microservice flows and decomposes each into
**per-hop budgets** that sum to the end-to-end target. Every µservice
SLO at `microservices/<ms>/slos/*.openslo.yaml` MUST honor the per-hop
ceiling allotted here; ADR-0139 SLO-gated promotion refuses fast-
forwards when the burn-rate against any hop in the chain exceeds its
allocation.

## Why per-hop budgets exist

Without explicit per-hop allocation, a single µservice can silently
consume more than its share of an end-to-end budget and starve
downstream hops. The pattern below mirrors how AWS structures its
Builders' Library budget allocations (per Amazon's "Avoiding overload
in distributed systems" white paper) and how Cloudflare allocates
per-hop budgets in its Workers platform (per the public engineering
blog 2022–2024). Industry convention: allocate budgets that sum to ≤
100% of the end-to-end target; reserve 5–10% for tail-latency
distribution skew (network jitter, GC pauses, sidecar overhead).

## Notation

- `p99` = 99th-percentile latency under steady-state traffic.
- `p95` = 95th-percentile latency (used for async flows where tail is
  bounded by retry rather than user-perceived latency).
- "async" flows have user-visible latency at the **first hop**
  (acknowledgement) and an SLO on the total completion time.
- All budgets are measured at the µservice's ingress, EXCLUDING the
  client→edge RTT (which is owned by the CDN/edge layer separately).

## Flow A: social post → audit-chain → ontology → workflow-engine → notification fan-out

**End-to-end budget: ≤ 1000 ms p99.**

Use case: an authenticated user submits a social post; the chain seals
the post to the immutable audit log, materialises it as an Ontology
entity, fires any matching workflow-engine triggers, and fans out
notifications to followers' inboxes.

| Hop | µservice                  | Operation                                           | Budget (p99) |
|-----|---------------------------|-----------------------------------------------------|--------------|
| 1   | `social`                  | Receive REST POST, validate, persist post row       | 150 ms       |
| 2   | `audit-chain`             | Ed25519-seal and append the post event              | 80 ms        |
| 3   | `ontology`                | Project post as Ontology entity (write path)        | 120 ms       |
| 4   | `workflow-engine`         | Match triggers + enqueue fan-out worker             | 150 ms       |
| 5   | `notification`            | Fan-out to follower inboxes (first-fan emission)    | 400 ms       |
| —   | (jitter / sidecar reserve)|                                                     | 100 ms       |
| **Σ** |                         |                                                     | **1000 ms**  |

Notes:
- Read-side ontology queries from `social` use the direct-path read
  granted by ADR-0141 (Workflow+Ontology read-path-direct); only the
  ontology **write** hop traverses the Workflow orchestration layer.
- The 400 ms notification budget assumes the "first follower's inbox
  receives the row" SLI. Tail fan-out to >10 000 followers is allowed
  to run async beyond the 1 s envelope and is governed by the
  notification µservice's own dedicated `notification_fanout_completion`
  SLO (p95 ≤ 30 s for 99% of fan-outs ≤ 1 M followers).

## Flow B: messenger DM → audit-chain → ontology mention → notification

**End-to-end budget: ≤ 500 ms p99.**

Use case: a user sends a direct message; the chain seals it, materialises
the @-mention as an Ontology mention edge, and notifies the recipient.

| Hop | µservice         | Operation                                 | Budget (p99) |
|-----|------------------|-------------------------------------------|--------------|
| 1   | `messenger`      | Receive WS frame, validate, persist row   | 100 ms       |
| 2   | `audit-chain`    | Ed25519-seal + append message event       | 60 ms        |
| 3   | `ontology`       | Persist @-mention edge                    | 80 ms        |
| 4   | `notification`   | Push frame to recipient WS + APNs/FCM     | 200 ms       |
| —   | (jitter reserve) |                                           | 60 ms        |
| **Σ** |                |                                           | **500 ms**   |

Notes:
- Messenger is the hot-path µservice for the PERF-143-002 metric
  emission integration described in `microservices/messenger/IP-NEW-
  hyperscaler-metric-emission.md`. Per-hop p99 violations of this
  budget MUST trigger circuit-breaker `record_capability_circuit_state`
  emission per the canonical metric naming convention.
- The 60 ms ontology budget is half the social post's 120 ms budget
  because mention edges are simpler than entity materialisation
  (single insert vs. multi-column projection).

## Flow C: tasks create → workflow-engine trigger → calendar event → notification

**End-to-end budget: ≤ 1500 ms p99.**

Use case: a user creates a task with a due date; workflow-engine
matches the "task created" trigger, and if a due-date is set, a
calendar event is auto-created and a confirmation notification is
sent to the assignee.

| Hop | µservice           | Operation                                  | Budget (p99) |
|-----|--------------------|--------------------------------------------|--------------|
| 1   | `tasks`            | Receive REST POST, validate, persist task  | 200 ms       |
| 2   | `audit-chain`      | Seal TaskCreated event                     | 80 ms        |
| 3   | `workflow-engine`  | Match trigger + invoke calendar capability | 250 ms       |
| 4   | `calendar`         | Create event from task due-date            | 350 ms       |
| 5   | `audit-chain`      | Seal CalendarEventCreated event            | 80 ms        |
| 6   | `notification`     | Notify assignee                            | 400 ms       |
| —   | (jitter reserve)   |                                            | 140 ms       |
| **Σ** |                  |                                            | **1500 ms**  |

Notes:
- The calendar hop is the longest synchronous hop in this chain
  because of tzdb resolution and recurrence-rule expansion (per
  `microservices/tasks/capabilities/T2-auto.yaml#T2-task-auto-recurring-materialise`).
- If `tasks.due_date` is null, hops 3-5 are skipped and the chain
  collapses to a 700 ms p99 budget (hops 1, 2, 6, + reserve).

## Flow D: meet end → recordings ingest → transcript → notification (async)

**End-to-end budget: ≤ 90 s p99 (async).**

Use case: a video meeting ends; the recording is uploaded, transcoded,
transcribed via Whisper, and a "transcript ready" notification is
dispatched to participants.

User-visible synchronous latency: only the "meeting ended" ack at hop
1 (≤ 200 ms p99). The remaining hops run async with their own SLO.

| Hop | µservice          | Operation                                         | Budget (p99) |
|-----|-------------------|---------------------------------------------------|--------------|
| 1   | `meet`            | Finalise session state, return ack to client      | 200 ms (sync) |
| 2   | `recordings`      | Ingest raw recording blob, virus-scan, persist    | 8 000 ms     |
| 3   | `recordings`      | Transcode to streaming-ready format               | 25 000 ms    |
| 4   | `recordings`      | Whisper transcription (90 min meet @ Whisper-large) | 45 000 ms  |
| 5   | `audit-chain`     | Seal TranscriptReady event                        | 80 ms        |
| 6   | `notification`    | Notify each participant                           | 1 200 ms     |
| —   | (queue + jitter)  |                                                   | 10 520 ms    |
| **Σ** | (async portion) |                                                   | **90 000 ms** |

Notes:
- Hop 4 dominates and is the lever that the recordings µservice SLO
  must protect. If Whisper-large p99 exceeds 45 s per 90 min input
  for sustained periods, the recordings SLO burns budget and ADR-0139
  refuses the next production fast-forward of the recordings release
  pointer.
- The notification on hop 6 is "fan-out to participants" (typically
  ≤ 50 participants) — substantially smaller than Flow A's 400 ms
  follower fan-out, so 1 200 ms p99 is generous.
- Per ADR-0143 (foundry per-bc release pointer), `recordings` releases
  independently of `meet`; tail-latency regressions in transcription
  do not block meeting-state changes.

## Flow E: drive upload → virus scan → ontology entity → preview generation (async)

**End-to-end budget: ≤ 5 s p95 (async).**

Use case: a user uploads a file to Drive; the file is virus-scanned,
projected as an Ontology entity, and a preview thumbnail is generated.
User-visible latency: ack at hop 1 within 300 ms p99; the chain
completes within 5 s p95 with status visible at the file's detail
endpoint.

| Hop | µservice          | Operation                                      | Budget (p95) |
|-----|-------------------|------------------------------------------------|--------------|
| 1   | `drive`           | Persist upload, ack to client                  | 300 ms (sync) |
| 2   | `drive`           | Virus scan (ClamAV or vendor adapter)          | 1 500 ms     |
| 3   | `audit-chain`     | Seal FileUploaded event                        | 80 ms        |
| 4   | `ontology`        | Project file row as Ontology entity            | 250 ms       |
| 5   | `drive`           | Preview generation (libvips for images, headless Chromium for docs) | 2 500 ms |
| 6   | `notification`    | Notify uploader (if preview-ready notification opted in) | 250 ms |
| —   | (jitter reserve)  |                                                | 120 ms       |
| **Σ** | (async portion) |                                                | **5 000 ms** |

Notes:
- The p95 envelope is used here (not p99) because preview generation
  for outlier formats (large CAD files, multi-tab spreadsheets) can
  reach 10s of seconds and would otherwise dominate the p99 tail. The
  outliers are governed by a separate `drive_preview_outlier_p99` SLO
  with a 30 s target.
- Virus scan (hop 2) is the canonical insertion point for
  `record_responses_429` if the per-tenant scan quota is hit — this
  is one of the call sites that the Messenger IP-NEW
  hyperscaler-metric-emission pattern generalises across µservices.

## Sources

- Google SRE Workbook ch. 5: *Alerting on SLOs* — multi-window
  multi-burn-rate.
- Amazon Builders' Library: *Avoiding overload in distributed systems*
  (s2n) and *Timeouts, retries, and backoff with jitter*.
- Stripe Engineering: *Online migrations at scale* — per-hop budget
  allocation for write-amplification chains.
- Linear Engineering: *Building a fast and reliable real-time sync
  engine* — per-service deploy refs (precedent for ADR-0143).
- Cloudflare Engineering: *Performance at the edge* — Workers latency
  budgets.
- ADR-0067 — perf authority (SSR p99 ≤ 500 ms, SSE p99 ≤ 2 s).
- ADR-0128 — hyperscaler architecture invariants.
- ADR-0139 — agentic SLO-gated promotion.
- ADR-0141 — workflow-ontology read-path-direct (mitigates Workflow
  SLO-ceiling concern).

## CI enforcement

A future `check-cross-microservice-latency-budget` lane will assert
that every per-hop budget in this document is reflected in the matching
`microservices/<ms>/slos/<sli>.openslo.yaml#objective.target`. Until
that lane lands, the budget is enforced by code review against this
document + the per-µservice SLO authoring contract in
`microservices/observability/IP-014-observability-slo-authoring.md`.
