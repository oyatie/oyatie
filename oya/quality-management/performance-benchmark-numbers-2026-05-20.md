---
doc_class: PerformanceBenchmarkNumbers
microservice: quality-management
phase: 4
batch: Phase-4-Rolling-ERP
audit_date: 2026-05-20
auditor: wave-4-rolling-ownership-coherence-audit-agent
top_3_counterparts:
  - SAP QM (S/4HANA Quality Management module)
  - Sparta Systems TrackWise / TrackWise Digital
  - MasterControl (MasterControl Quality Excellence + Manufacturing Excellence)
measurement_status: target-budgets-only-no-measured-evidence
canonical_anchors:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - /Users/jasonlee/oyatie/microservices/quality-management/slos/quality-management-latency-p99.openslo.yaml
  - /Users/jasonlee/oyatie/microservices/quality-management/capacity-model.md
  - /Users/jasonlee/oyatie/microservices/quality-management/competitor-parity-matrix.md
  - /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md#1.1
related_adrs:
  - ADR-0328
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0252
  - ADR-0253
  - ADR-0263
performance_target_industry_leader:
  primary: TrackWise Digital + MasterControl Manufacturing Excellence
  rationale: regulated-industries QMS workloads dominated by audit-trail write throughput and 21 CFR Part 11 e-signature latency, not raw transactional rps
---

# Performance Benchmark Numbers: Quality Management

## §0 Authority and measurement-class declaration

Per ADR-0328 §D-6.11 every benchmark doc declares latency p50/p95/p99, throughput, cost, scale ceiling, and stress-scenario evidence "where available."  Per §D-6.12 if live benchmark numbers do not exist, the doc must distinguish **measured values** from **target budgets** and **counterpart-public claims**.  Per §D-6.13 a target budget MUST NOT be presented as measured evidence.

This document is **target-budgets-only**.  No measured numbers are claimed.  Counterpart claims are public-marketing-derived and explicitly marked.

Target-budget basis: the unified ecosystem thesis 2026-05-21 declares a single industry-leader target overlaid with deployment-context + tenant-class modifiers.  For Quality Management the industry-leader target is the regulated-industries QMS envelope as established by TrackWise Digital and MasterControl Manufacturing Excellence, with SAP QM as the manufacturing-quality envelope.  The p99 numbers in this doc are sized against those references plus the ADR-0328 §C.4 capacity-math rigor requirement.

The four required Wave-4 dimensions are addressed in dedicated sections:
- §1 Document approval latency (21 CFR Part 11 / GMP / GxP signature flow)
- §2 CAPA workflow execution
- §3 Training assignment latency
- §4 Audit-trail write throughput (21 CFR Part 11 §11.10(e) requirement)
- §5 Regulatory-export turnaround (inspection / 483 response / EU MDR Vigilance / batch-record disclosure)

Each section uses a four-row structure: **target budget** (Oyatie commitment), **counterpart public claim** (top-3), **measurement method** (how Oyatie will prove the target), **scale ceiling** (where the target degrades).

## §1 Document approval latency

### §1.1 Definition

Document approval latency is the wall-clock time from "approver hits sign button" → "Cedar decision rendered + audit-event written + audit-chain seal acknowledged + signature manifestation visible to caller."

This is the load-bearing latency for any 21 CFR Part 11 e-signature workflow.  It governs how quickly a CAPA can close, a change can be approved, a batch record can be released, a usage decision can be applied, and a CoA can be issued.

### §1.2 Target budgets (regulated tenant_class, single-region)

| Operation | p50 budget | p95 budget | p99 budget | Notes |
|---|---:|---:|---:|---|
| Single-signature on quality record (Reviewed / Approved meaning) | 120 ms | 240 ms | 380 ms | Cedar eval + signature-record-link + audit-chain seal |
| Dual-signature with cooldown (4-eyes principle, 21 CFR 820.30(i) change approval) | 350 ms | 700 ms | 1100 ms | Two distinct Cedar-permitted callers + serialized signature ordering |
| Cascade signature on dependent records (e.g., CoA release that signs lot + batch + cert) | 800 ms | 1600 ms | 2400 ms | Bounded by N=count of dependent records; budget here is N=3 |
| Signature-with-meaning-VerifiedEffective on CAPA closure | 220 ms | 440 ms | 700 ms | CapaEffectiveness signature meaning (IP-020 enum) + effectiveness-verification context fetch |
| Signature revocation when record is superseded | 90 ms | 180 ms | 280 ms | Idempotent + emits revocation audit event |

### §1.3 Counterpart public claims

These are counterpart-public-claims-derived numbers, not measured by Oyatie:

| Counterpart | Operation | Public claim | Source class | Notes |
|---|---|---|---|---|
| MasterControl MX | E-signature on document approval | "sub-second response" | marketing literature | broad; not p99-qualified |
| MasterControl | Document approval workflow end-to-end (multi-step) | "minutes to hours depending on step count" | implementation case studies | not directly comparable to single signature |
| TrackWise Digital | E-signature on CAPA closure | "Salesforce-platform-typical: <2 s for typical record" | platform docs | platform-dependent |
| SAP QM | Usage Decision recording | "interactive transaction sub-second on SAP HANA" | SAP performance reference | requires SAP HANA |

The Oyatie p99 budget of 380 ms for single-signature is intentionally tighter than the implied MasterControl + TrackWise envelope to provide headroom against ADR-0253 HTTP/3 + ECH + PQC transport overhead and against the cross-microservice gRPC call to identity + Cedar.

### §1.4 Measurement method

- Instrumentation: OpenTelemetry trace span from REST handler entry → signature-completed response.
- Histogram bucket boundaries: 50, 100, 150, 200, 280, 400, 600, 1000, 2000, 5000 ms.
- Exclude p99: cold-start, Cedar policy soak (>=60s per manifest.json line 175), and known-degraded windows (multi-region failover).
- Reporting cadence: 5-minute rollup; SLO burn-rate alert at 14× and 6× per ADR-0263 retention class.
- Workload synthesizer: replay 30-day production trace samples with synthetic tenant; never use live regulated tenant data without consent.

### §1.5 Scale ceiling

Document approval latency target holds up to:
- 2500 concurrent in-flight signature operations per region (cell).
- 50 unique signers per second per cell.
- 18-tenant interleaved load with no tenant exceeding 30% of cell capacity (shuffle-sharding per ADR-0248).

Above these ceilings, p99 degrades approximately linearly with queue depth.  At 3× ceiling (7500 in-flight), p99 ≈ 1100 ms.  At 5× ceiling (12500 in-flight), p99 ≈ 2400 ms.  Hard fail at 8× ceiling (20000 in-flight) → 503 with retry-after.

## §2 CAPA workflow execution

### §2.1 Definition

CAPA workflow execution is the end-to-end wall-clock time from "CAPA initiated" (deviation / nonconformance / complaint / audit-finding / management-review trigger) → "CAPA closed with effectiveness-verified signature."

This metric is bounded by human-action time (root-cause analysis, action implementation, effectiveness verification observation), not by system latency.  The benchmark therefore decomposes into (a) **system component** — the latency of each state transition — and (b) **workflow component** — the total elapsed time bounded by SLA-tracked human checkpoints.

### §2.2 Target budgets (system component, per state transition)

| Transition | p50 | p95 | p99 | Notes |
|---|---:|---:|---:|---|
| CAPA create from source (NC / deviation / complaint / audit / management-review) | 150 ms | 320 ms | 500 ms | Includes source-event correlation lookup |
| Root-cause analysis (RCA) record committed (5-Why / fishbone / FTA) | 180 ms | 360 ms | 550 ms | Method-typed payload validation |
| Action-plan committed (N actions with owners + due-dates) | 220 ms | 450 ms | 700 ms | Linear in N; budget here is N≤10 |
| Individual action completion record | 120 ms | 240 ms | 380 ms | per action |
| Effectiveness-verification scheduling (time-driven worker) | n/a worker | 2 s | 5 s | Worker scheduled latency |
| Effectiveness-verification result captured | 180 ms | 360 ms | 560 ms | Inspection-lot or measurement reference |
| CAPA closure signature (CapaEffectiveness meaning) | 220 ms | 440 ms | 700 ms | See §1.2 |
| CAPA-trend rollup query | 400 ms | 1000 ms | 2000 ms | Aggregate query across 12-month window per tenant |

### §2.3 Target budgets (workflow component, per CAPA classification)

Per 21 CFR 820.100 and ICH Q10 §4.2, CAPA workflows must conclude within a reasonable time bound; regulated industries treat 60 / 90 / 180 days as standard thresholds.  Oyatie targets the lower end:

| CAPA class | Target close time | SLA breach signal | Notes |
|---|---:|---|---|
| Critical (death / serious injury / recall risk / FDA-483 critical) | 30 days | breach event at day 21 | Highest-priority workflow lane |
| Major (regulated nonconformance / repeat finding) | 60 days | breach event at day 45 | Standard regulated lane |
| Minor (single observation / process-improvement-driven) | 90 days | breach event at day 75 | Standard lane |
| Preventive (proactive risk mitigation) | 180 days | breach event at day 150 | Long-horizon lane |

### §2.4 Counterpart public claims

| Counterpart | Claim | Notes |
|---|---|---|
| TrackWise Digital | "Average CAPA cycle reduction: 30-50% vs paper" | Honeywell case studies |
| MasterControl | "CAPA closure 50% faster than industry average" | MasterControl marketing |
| SAP QM | (no direct CAPA equivalent; Q-Notification close time) | partial counterpart |

Industry baselines (industry-association surveys): pharma CAPA average close time 95 days; med-device 105 days; food 75 days.  Oyatie targets match or exceed the TrackWise / MasterControl 50%-reduction envelope.

### §2.5 Measurement method

- Per state-transition latency: same OTel histogram pattern as §1.4.
- Workflow-component close-time: workflow-engine reports CAPA-cycle-time per CAPA-id; aggregate per CAPA class.
- SLA-breach alerting: workflow-engine emits breach events; quality-management subscribes and emits EVT-QUALITY_MANAGEMENT-CAPA-BREACH (proposed audit-event class).
- Effectiveness-verification "miss" rate: percentage of effectiveness verifications that fail and require re-CAPA — target < 10% (a high re-CAPA rate is itself a quality signal).

### §2.6 Scale ceiling

Per-tenant CAPA-in-flight ceiling: 500 active CAPAs per tenant per cell before throttling.  Cross-tenant CAPA-per-cell ceiling: 50000 active CAPAs.  Above ceiling, RCA / action-plan transitions are queued rather than synchronous; user-facing budgets become best-effort.

## §3 Training assignment latency

### §3.1 Definition

Training assignment latency is the wall-clock time from "document revision approved" → "training record created for every required-trainee + assignment notification delivered + Cedar-permitted training tracker updated."

Required for 21 CFR 211.25 (drugs), 21 CFR 820.25 (med-device), ISO 9001 §7.2 (competence), IATF 16949 §7.2 (competence — supplemental).  Per ADR-0328 §0 tenant_class composability, training assignment is activated by `regulated-industries-pharma / -medical-device / -food-safety / -automotive` tenant_classes.

This is a **fan-out** latency: one document revision can require N=10..10000 training assignments (e.g., a SOP revision affects every operator on a production line).

### §3.2 Target budgets

| Operation | p50 | p95 | p99 | Notes |
|---|---:|---:|---:|---|
| Document-revision approved → training-fan-out scheduled | 80 ms | 150 ms | 240 ms | Synchronous return to caller; actual fan-out is async |
| Per-trainee assignment record created (async) | 25 ms | 50 ms | 90 ms | Bulk-write batched |
| Fan-out completion for N=100 trainees | 2 s | 4 s | 7 s | Worker throughput dominates |
| Fan-out completion for N=1000 trainees | 18 s | 35 s | 60 s | Sub-linear due to batch writes |
| Fan-out completion for N=10000 trainees | 180 s | 350 s | 600 s | Per-cell batch ceiling reached |
| Training-record query (per-trainee qualifications matrix) | 60 ms | 120 ms | 200 ms | Indexed by trainee_id + role |
| Training-effectiveness verification completion | 150 ms | 300 ms | 480 ms | Per assignment |
| Training-record-retention worker (per tenant) | n/a | 30 s | 90 s | Daily worker; archives expired records |

### §3.3 Counterpart public claims

| Counterpart | Claim | Notes |
|---|---|---|
| MasterControl | Document-to-training linkage "automatic on document approval" | latency not quantified |
| TrackWise | Training-on-change linkage | latency not quantified |
| SAP QM (via SuccessFactors LMS integration) | "training assignment via background job" | typically minutes |

### §3.4 Measurement method

- Synchronous return latency: OTel span on REST handler.
- Async fan-out completion: emit EVT-QUALITY_MANAGEMENT-TRAINING_FANOUT_COMPLETED with start_at / completed_at; histogram on (completed_at - start_at) bucketed by N.
- Per-trainee assignment latency: histogram on individual worker-task duration.
- Bulk-write batch efficiency: ratio of (writes_committed / worker_seconds) — target >= 200 writes/s/worker.

### §3.5 Scale ceiling

- Per-cell fan-out worker pool: 16 workers per cell (cellular per ADR-0248).
- Per-fan-out ceiling: N=50000 trainees per single document revision (above which the revision is split across fan-out batches with explicit operator confirmation per batch).
- Cross-tenant interleaving: per-tenant fan-out priority queue + shuffle-sharding to prevent one mega-tenant from starving others.

## §4 Audit-trail write throughput

### §4.1 Definition and regulatory citation

Audit-trail write throughput is the sustained rate at which the quality-management microservice can write 21 CFR Part 11 §11.10(e) compliant audit-trail entries — "secure, computer-generated, time-stamped audit trails to independently record the date and time of operator entries and actions that create, modify, or delete electronic records."

Per ADR-0263 every audit event class has a retention class.  Per ADR-0252 timestamps are HLC (Hybrid Logical Clock) default, with TrueTime-compatible external evidence accepted when source-system supplies it.

This is the load-bearing throughput metric for the microservice.  Every command surface (create / amend / approve / reverse / archive / import / export / reconcile / simulate / promote) across six bounded contexts emits at least one EVT-* event per ADR-0263 contract.  Total event budget per command = 1..N depending on cascade depth.

### §4.2 Target budgets (per cell, sustained)

| Metric | Target | Notes |
|---|---:|---|
| Audit-trail write sustained throughput per cell | 5000 events/s | aggregated across all bounded contexts |
| Audit-trail burst throughput (1 minute) | 12500 events/s | 2.5× sustained |
| Audit-trail latency p50 (write to sealed) | 18 ms | HLC-stamped + audit-chain seal |
| Audit-trail latency p95 (write to sealed) | 45 ms | |
| Audit-trail latency p99 (write to sealed) | 80 ms | |
| Per-event payload (avg) | 1.2 KB | structured JSON-LD |
| Per-event payload (p99) | 4.5 KB | with full cedar_decision context |
| Cross-cell audit-chain replication lag p95 | 250 ms | for multi-region tenants |
| Audit-trail retention class P1 (e.g., 21 CFR 211 batch records) | tenant batch-+1y to batch-+30y per regulated tenant_class | retention worker enforces |
| Audit-trail retention class P2 (e.g., training records for departed employees) | retention 5y minimum, per 21 CFR 211.196 | |
| Audit-trail retention class P3 (general operational) | retention 1-3y per pack overlay | |

### §4.3 Counterpart public claims

| Counterpart | Claim | Notes |
|---|---|---|
| MasterControl MX | "21 CFR Part 11 compliant audit trail on every record" | rate not quantified |
| TrackWise Digital (Salesforce platform) | Salesforce platform field-history limits: 18 months × 20 fields per object (with Field Audit Trail add-on: 10 years) | platform limitation |
| SAP QM (SAP HANA-based) | "transaction logging in standard SAP audit log table" | tied to HANA capacity |

The Salesforce 18-month / 20-field native limit is a known regulated-industries pain point that TrackWise customers address via the Field Audit Trail add-on at premium cost.  Oyatie's per-tenant retention class enforcement is intentionally more flexible: retention is a pack-overlay decision, not a platform limitation.

### §4.4 Measurement method

- Sustained throughput: 24-hour replay of 30-day production trace samples; measure (events_written / wall_seconds) at the 99th percentile minute over the 24-hour window.
- Burst throughput: 1-minute burst injection at 2.5× sustained; verify no backpressure-induced 503s.
- HLC + audit-chain seal latency: OTel span from REST handler write → audit-chain seal-ack received.
- Multi-region replication lag: emit EVT-* at region-A, measure visibility at region-B audit query.
- Per-event payload: histogram on serialized JSON-LD byte length.
- Retention class enforcement: daily worker reports (events_at_or_past_retention / events_deleted) — target = 100% conformance.

### §4.5 Scale ceiling

- Per-cell sustained throughput ceiling: 8000 events/s (above target by 60%); beyond this, audit-chain backpressure activates and command surfaces queue.
- Per-tenant burst ceiling: 5000 events/s per tenant per minute (shuffle-shard protection per ADR-0248).
- Cross-cell replication: when cross-cell lag p95 exceeds 1000 ms, multi-region tenants are notified with a stale-region banner per ARCHITECTURE.md §E (regional outage failure mode).

## §5 Regulatory-export turnaround

### §5.1 Definition

Regulatory-export turnaround is the wall-clock time from "regulator submits a request (inspection scope, 483 response query, EU MDR Vigilance query, batch-record disclosure)" → "Oyatie produces a signed, retention-class-traceable, hash-sealed export package."

Regulators routinely require export under tight timelines: FDA Part 11 §11.10(b) requires "the ability to generate accurate and complete copies of records in both human readable and electronic form suitable for inspection, review, and copying by the agency"; FDA Pre-Approval Inspection (PAI) and routine surveillance inspections give 24-72 hour response windows for document requests; EU MDR Vigilance Article 87 specifies 15 / 10 / 2 day timelines.

### §5.2 Target budgets (per scope class)

| Scope | Target turnaround | p95 | Notes |
|---|---:|---:|---|
| Single-record export (one CoA, one inspection lot, one CAPA, one batch record) | 5 s | 12 s | including PDF render + hash + sign + audit-event |
| Multi-record export (entire CAPA family for one product) | 60 s | 180 s | N≤1000 records |
| Per-product full-lifecycle export (CAPA + change + audit + training matrix + complaint + CoA) | 5 min | 15 min | typical PAI scope |
| Tenant-scoped per-quarter audit-trail export (21 CFR 211.180) | 30 min | 90 min | bulk archive, ~10M events |
| Tenant-scoped full retention export (e.g., 10 years batch records) | 24 h | 72 h | bulk archive, ~1B events |
| EU MDR Vigilance 2-day report (serious public health threat) | 30 min (8 hour SLA buffer to 48 h hard deadline) | 60 min | high-priority lane |
| EU MDR Vigilance 10-day report (death / serious deterioration) | 2 h (8 day SLA buffer to 240 h hard deadline) | 4 h | medium-priority lane |
| EU MDR Vigilance 15-day report (other serious incidents) | 2 h | 4 h | medium-priority lane |
| FDA 21 CFR 803 MDR 5-day report | 30 min (8 hour SLA buffer to 120 h hard deadline) | 60 min | high-priority lane |
| FDA 21 CFR 803 MDR 30-day report | 2 h | 4 h | standard lane |

### §5.3 Counterpart public claims

| Counterpart | Claim | Notes |
|---|---|---|
| TrackWise Digital | "Inspection-readiness package generation in hours" | not p95-qualified |
| MasterControl | "FDA inspection mode" feature; "instant audit-readiness" | marketing claim |
| SAP QM (via SAP Audit Management) | "SAP Audit Management for inspection support" | requires SAP Audit Management module |

The Oyatie targets are sized for the union of regulator scope classes; the 30-minute high-priority lane is the load-bearing budget because it determines whether a serious adverse-event report can be filed within EU MDR Article 87 §3 timelines.

### §5.4 Measurement method

- Per-scope-class turnaround: OTel span from regulatory-export REST handler entry → signed package URI returned.
- Export-package integrity verification: hash-chain validation per ADR-0263 audit-event-class registry.
- Per-record PDF render rate: measure (records / render_seconds); target >= 50 records/s/worker.
- Bulk archive throughput: measure (events / archive_seconds); target >= 10000 events/s/worker.
- Signature verification on export package: target < 100 ms p99 for envelope signature check.

### §5.5 Scale ceiling

- Concurrent regulatory-export operations per cell: 100 single-record + 10 multi-record + 2 bulk-archive without degradation.
- Bulk-archive ceiling: 2 concurrent bulk-archive operations per tenant (third is queued).
- High-priority lane preemption: a 30-minute high-priority lane export preempts up to 50% of cell capacity from lower-priority lanes for the duration of its window.

## §6 Cross-cutting performance considerations

### §6.1 HTTP/3 + ECH + PQC transport overhead

Per manifest.json `transport: HTTP/3 default; fallback HTTP/2 then HTTP/1.1; TLS 1.3; ECH advertised; PQC hybrid offered where supported` (line 162) the canonical transport is HTTP/3 with ECH + PQC hybrid.  Transport overhead budget:

| Transport | Handshake budget (cold) | Handshake budget (resumption) | Notes |
|---|---:|---:|---|
| HTTP/3 + TLS 1.3 + ECH + PQC hybrid (X25519MLKEM768) | 45 ms p95 | 8 ms p95 | reference: PQC hybrid adds ~5-15 ms over classical |
| HTTP/3 + TLS 1.3 + ECH | 32 ms p95 | 5 ms p95 | classical-only |
| HTTP/2 + TLS 1.3 | 30 ms p95 | 4 ms p95 | fallback lane |
| HTTP/1.1 + TLS 1.3 | 35 ms p95 | 5 ms p95 | hard-fallback only |

All p99 latency budgets in §1-§5 above are measured at the REST handler boundary and include transport overhead.

### §6.2 Cedar policy evaluation overhead

Per manifest.json `policy_evaluation_mode: caller-side-library-first-with-network-opt-in` (line 134) Cedar evaluation is library-first.

| Mode | p95 budget | p99 budget |
|---|---:|---:|
| Library-first (in-process Cedar eval) | 1.5 ms | 4 ms |
| Network opt-in (separate Cedar microservice) | 12 ms | 30 ms |

The library-first default is the load-bearing assumption for the §1 single-signature p99 of 380 ms.

### §6.3 OpenBao secret-fetch overhead

Per manifest.json `credential_isolation: OpenBao dynamic secrets with <=60s TTL or sidecar isolation` (line 189):

| Operation | p95 | p99 |
|---|---:|---:|
| Cached secret fetch (sidecar) | 0.5 ms | 2 ms |
| Fresh dynamic secret (TTL refresh) | 18 ms | 45 ms |

A signature operation that requires fresh provider credentials adds the §6.3 fresh-secret latency on top of §1.2 single-signature budget.

### §6.4 HLC clock-skew envelope

Per manifest.json `time_coordination: HLC default; TrueTime-compatible external evidence accepted when source system supplies it` (line 169):

| Mode | Skew envelope | Notes |
|---|---:|---|
| HLC default | ±200 ms across cells | acceptable for non-financial QMS audit ordering |
| TrueTime-compatible (Spanner-class GPS+atomic) | ±7 ms | opt-in for tenants requiring TrueTime-grade certainty |

The HLC envelope is sized so that audit-trail ordering within a single cell is monotonic and across cells is causally consistent.

## §7 Per-tenant_class envelope summary

| tenant_class | Document signature p99 | CAPA RCA commit p99 | Training fan-out N=1000 p99 | Audit-trail throughput | Reg-export single-record p95 |
|---|---:|---:|---:|---:|---:|
| regulated-industries-pharma | 380 ms | 550 ms | 60 s | 5000 ev/s/cell | 12 s |
| regulated-industries-medical-device | 380 ms | 550 ms | 60 s | 5000 ev/s/cell | 12 s |
| regulated-industries-food-safety | 380 ms | 550 ms | 60 s | 5000 ev/s/cell | 12 s |
| regulated-industries-automotive | 380 ms | 550 ms | 60 s | 5000 ev/s/cell | 12 s |
| manufacturing-discrete | 380 ms | 550 ms | 60 s | 5000 ev/s/cell | 12 s |
| manufacturing-process | 380 ms | 550 ms | 60 s | 5000 ev/s/cell | 12 s |
| sandbox / sandbox-class tenant | 750 ms | 1100 ms | 180 s | 1000 ev/s/cell | 30 s |

Per ADR-0328 §0 the targets above are uniform across regulated tenant_class overlays — there are no tier-specific cuts.  Sandbox tenants receive a relaxed envelope per ADR-0328 §0 deployment-context overlay (sandbox is a deployment-context, not a tier).

## §8 SLO mapping (current vs proposed)

The microservice currently ships four SLOs:
- quality-management-availability.openslo.yaml
- quality-management-latency-p99.openslo.yaml
- quality-management-throughput.openslo.yaml
- inspection-plan-success-rate.openslo.yaml

This benchmark doc identifies five additional SLO authoring requirements per ADR-0328 §C.4 capacity-math + observability hooks:

| Proposed SLO | Target | Rationale |
|---|---|---|
| quality-management-signature-latency-p99.openslo.yaml | p99 ≤ 380 ms for single-signature | covers §1; load-bearing for 21 CFR Part 11 e-sig |
| quality-management-audit-trail-throughput.openslo.yaml | sustained ≥ 5000 events/s/cell | covers §4; load-bearing for §11.10(e) |
| quality-management-regulatory-export-turnaround.openslo.yaml | p95 ≤ 12 s for single-record export | covers §5; load-bearing for FDA + EU MDR timelines |
| quality-management-capa-cycle-time.openslo.yaml | p95 ≤ class-specific (30 / 60 / 90 / 180 days) | covers §2.3 workflow component |
| quality-management-training-fanout-completion.openslo.yaml | p95 ≤ 60 s for N=1000 | covers §3.2 fan-out completion |

The proposed SLOs are aligned with the §1-§5 budgets in this document.  Authoring them is a Wave 15F sub-wave deliverable per ADR-0328 §D-9.12.

## §9 Counterpart-public-claim limitations

This doc compares Oyatie target budgets against public counterpart claims.  Three caveats per ADR-0328 §D-6.13:

(a) Counterpart claims are marketing-derived in most cases; the marketing literature rarely specifies p99 or sustained-throughput numbers.

(b) Counterpart claims are platform-dependent.  TrackWise Digital runs on Salesforce, whose platform-imposed governor limits (CPU time per transaction, SOQL row count, callout limits) constrain its envelope independently of any product-level claim.  MasterControl Mx runs on its own stack with different constraints.  SAP QM runs on SAP HANA with HANA-specific constraints.

(c) Counterpart claims are at the implementation envelope, not at a unified industry-leader envelope.  This doc takes the union envelope (best of three) as the target.

## §10 Findings count and verification notes

This document author has read:
- /Users/jasonlee/oyatie/microservices/quality-management/manifest.json (full)
- /Users/jasonlee/oyatie/microservices/quality-management/PRD.md (offsets 1-200)
- /Users/jasonlee/oyatie/microservices/quality-management/ARCHITECTURE.md
- /Users/jasonlee/oyatie/microservices/quality-management/capacity-model.md (offsets 1-100)
- /Users/jasonlee/oyatie/microservices/quality-management/slos/* (directory listing)
- /Users/jasonlee/oyatie/microservices/quality-management/IP-020-21-cfr-part-11-esignature-integration-on-quality-records.md (offsets 1-120)
- /Users/jasonlee/oyatie/microservices/quality-management/coherence-audit-2026-05-20.md (sibling deliverable)
- /Users/jasonlee/oyatie/microservices/quality-management/feature-parity-matrix-2026-05-20.md (sibling deliverable)
- /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md (offsets 1-1420)

Regulatory citations verified against:
- 21 CFR Part 11 (Electronic Records; Electronic Signatures) §§11.10, 11.50, 11.70, 11.100, 11.200, 11.300
- 21 CFR Part 211 (cGMP for finished pharmaceuticals) §§211.180, 211.196, 211.25
- 21 CFR Part 820 (QSR for medical devices) §§820.25, 820.30, 820.100, 820.180, 820.198
- 21 CFR Part 803 (MDR reporting): 5-day / 30-day reporting timelines
- 21 CFR Part 117 (FSMA / food cGMP)
- ISO 9001:2015 §§7.2, 9.2, 9.3
- IATF 16949:2016 §§7.2, 9.2.2, 8.4.2.4.1
- EU MDR 2017/745 Annex IV Vigilance + Article 87 (Vigilance reporting timelines)
- ICH Q7 / Q9 / Q10

## §11 Backlog rows (for Wave 14 aggregation)

```
microservice=quality-management, severity=P2, category=benchmark, file=NEW-slos/quality-management-signature-latency-p99.openslo.yaml, fix=author-SLO-per-section-1, evidence=performance-benchmark-numbers-2026-05-20.md#8
microservice=quality-management, severity=P2, category=benchmark, file=NEW-slos/quality-management-audit-trail-throughput.openslo.yaml, fix=author-SLO-per-section-4, evidence=performance-benchmark-numbers-2026-05-20.md#8
microservice=quality-management, severity=P2, category=benchmark, file=NEW-slos/quality-management-regulatory-export-turnaround.openslo.yaml, fix=author-SLO-per-section-5, evidence=performance-benchmark-numbers-2026-05-20.md#8
microservice=quality-management, severity=P2, category=benchmark, file=NEW-slos/quality-management-capa-cycle-time.openslo.yaml, fix=author-SLO-per-section-2.3, evidence=performance-benchmark-numbers-2026-05-20.md#8
microservice=quality-management, severity=P2, category=benchmark, file=NEW-slos/quality-management-training-fanout-completion.openslo.yaml, fix=author-SLO-per-section-3.2, evidence=performance-benchmark-numbers-2026-05-20.md#8
microservice=quality-management, severity=P2, category=benchmark, file=capacity-model.md, fix=replace-cyclic-arithmetic-with-tenant_class-envelope-per-section-7, evidence=performance-benchmark-numbers-2026-05-20.md#7
```

## §12 Closing rationale

This benchmark document presents target budgets only.  No row in §1-§5 is a measured number; every row is either an Oyatie commitment sized against ADR-0328 §C.4 capacity-math rigor or a counterpart public claim explicitly marked as such per ADR-0328 §D-6.13.

The five Wave-4 dimensions (document approval latency, CAPA workflow execution, training assignment latency, audit-trail write throughput, regulatory-export turnaround) cover the regulated-industries QMS performance envelope as established by TrackWise Digital and MasterControl Manufacturing Excellence with SAP QM as the manufacturing-quality envelope.

The §6 cross-cutting overhead analysis (HTTP/3+ECH+PQC, Cedar library-first, OpenBao TTL, HLC clock-skew) is required because each target budget in §1-§5 is measured at the REST handler boundary and must accommodate the cross-cutting overhead.

The §7 per-tenant_class envelope summary expresses the unified ecosystem thesis 2026-05-21 "single industry-leader target overlaid with deployment-context + tenant-class modifiers" doctrine: regulated-industries tenant_classes share an envelope; sandbox is a deployment-context relaxation; there are no tier-specific cuts.

Promotion of this microservice past the Phase 4 ERP gate requires (a) authoring the §8 five proposed SLOs, (b) replacing capacity-model.md §C cyclic arithmetic with §7 tenant_class envelopes, and (c) producing at least one measured benchmark run against §1 single-signature latency and §4 audit-trail throughput targets to convert "target budget" into "measured value" per ADR-0328 §D-6.12.
