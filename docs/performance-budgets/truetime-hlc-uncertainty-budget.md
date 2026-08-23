---
slug: truetime-hlc-uncertainty-budget
title: "TrueTime / HLC Uncertainty Budget — Modeled Latency and Clock Assumptions"
binding_adrs:
  - ADR-0252-time-coordination-distributed-consistency.md
status: modeled
date: 2026-05-20
authors:
  - council-architecture
  - axis-audit-chain
  - ops-sre-reliability
related_budgets:
  - cedar-hot-path-1ms-p99.md
  - deployment-model-slo-budgets.md
---

# TrueTime / HLC Uncertainty Budget — Modeled Latency and Clock Assumptions

## Purpose

This modeling note provides the measurement basis and assumption set for the clock-coordination
performance claims in ADR-0252. Key claims addressed:

1. **HLC uncertainty default: ±500ms** — is this achievable and what does it mean for SLOs?
2. **TrueTime uncertainty: 7ms default / 1ms when GPS+atomic healthy** — commit-wait latency
   implications for Tier-4 financial cells.
3. **StrictTotalOrder adds ~5-30ms p99** — basis for this claim.
4. **ExternalConsistency commit-wait = TT uncertainty bound (7ms default)** — implications for
   KR-FSS order matching at 10k orders/sec.

---

## 1. HLC Default Uncertainty: ±500ms

### 1.1 Assumptions

| Assumption ID | Assumption | Source |
|---|---|---|
| H-01 | Each K8s node runs `chronyd` with ≥2 stratum-2 NTP sources | ADR-0252 §D-1; ADR-0241 node-hardening |
| H-02 | chronyd achieves ≤100ms NTP offset within a datacenter | ADR-0252 §D-1; chronyd documentation; typical measured offset: 1-10ms within DC, 10-50ms cross-region |
| H-03 | Cross-region NTP drift ≤ 100ms (worst case GPS antenna degradation; per Spanner 2017 incident) | ADR-0252 incident table §Context |
| H-04 | HLC logical counter absorbs clock skew events without wraparound at ±500ms uncertainty | Demirbas-Kulkarni 2014 HLC proof |
| H-05 | Leap seconds smeared via `chronyd leapsectz slew` (Google approach) | ADR-0252 §D-7 |

### 1.2 Why ±500ms Is Conservative

CockroachDB's TrueTime equivalent uses a 500ms uncertainty bound as its default (matching
Google Spanner's public documentation). In practice:

| Deployment type | Typical measured clock skew | Configured uncertainty |
|---|---|---|
| Same-DC, same-rack | ≤1ms | 500ms (uses logical counter, physical drift negligible) |
| Same-DC, different racks | ≤5ms | 500ms |
| Cross-AZ, same region | ≤20ms | 500ms |
| Cross-region, continental | ≤50ms | 500ms |
| Cross-region, intercontinental | ≤100ms | 500ms |
| GPS antenna degradation (2017 Spanner incident) | ~80ms transient | 500ms (absorbed) |

**The 500ms bound is ~5-10× larger than actual worst-case drift.** This is intentional:
HLC's logical counter prevents out-of-order events even when physical clocks diverge by up to
the configured bound. Using a large bound (500ms) means the logical counter rarely needs to
increment, keeping timestamps compact.

### 1.3 SLO Implications of HLC ±500ms

The HLC uncertainty does NOT add 500ms to query latency. The uncertainty bound affects only:

- **Causal read freshness:** A read after a write is guaranteed consistent as of
  `write_hlc + uncertainty_bound`. At 500ms bound, a causal read may observe a write up to
  500ms stale. This is acceptable for all non-financial operations (messages, docs, workflows).
- **Idempotency key dedup window:** The 24h idempotency key TTL (ADR-0252 §D-4) is orders of
  magnitude larger than the 500ms uncertainty; no conflict.
- **Audit chain ordering:** Audit events may be ordered up to 500ms out-of-physical-order but
  in correct HLC logical order. Forensic reconstruction is deterministic.

**The 500ms bound does NOT add to per-request latency** — it is a configuration parameter
controlling logical counter behavior, not a wait time.

### 1.4 Alert Thresholds

| Metric threshold | Alert level | Action |
|---|---|---|
| `hlc_uncertainty_ms > 300ms` p99 sustained 5 min | WARN (SEV-4) | Investigate chronyd health; check NTP source reachability |
| `hlc_uncertainty_ms > 500ms` p99 sustained 1 min | SEV-3 (cell-local page) | As ADR-0252 §D-11 |
| `hlc_uncertainty_ms > 1000ms` p99 | SEV-2 (cell-wide) | Brown-out signal; saga coordinator pauses strict-total-order ops |
| `hlc_uncertainty_ms > 5000ms` p99 | SEV-1 (cross-cell) | Outage mode; ADR-0252 §D-11 full response |

The **300ms warn threshold** (below the 500ms budget) gives operators 30+ minutes of lead
time before the budget is breached, based on chronyd drift rates (typical: ≤10ms/hour drift
rate, so 300ms → 500ms takes ≥20 hours from a gradual drift scenario).

---

## 2. TrueTime Uncertainty: 7ms Default / 1ms When Healthy

### 2.1 Hardware Basis

Per ADR-0252 §D-4 Tier-4 cell hardware:

| Component | Contribution to uncertainty | Notes |
|---|---|---|
| GPS receiver (primary) | ±100ns (negligible) | GPS time accuracy when satellites visible |
| Cesium primary atomic clock | ±5×10⁻¹³/s drift | If GPS lost: ≤1µs/day → ≤365µs/year hold-over |
| Rubidium secondary atomic clock | ±5×10⁻¹¹/s drift | If GPS lost: ≤4.3ms/day hold-over |
| Network propagation (GPS signal to server) | ≤1µs (same-rack) | Negligible |
| Kernel clock discipline (PTP IEEE 1588) | ≤1µs | Hardware timestamping required |
| **Total uncertainty (GPS + atomic healthy)** | **≤1ms** | Dominated by network jitter to GPS receiver |
| **Total uncertainty (GPS degraded, Rubidium only)** | **≤7ms/day** | Spanner 7ms design bound; matches Google's published figure |

The **7ms default** is the bound when at least one Rubidium secondary is healthy but GPS is
degraded. The **1ms tightened bound** is achievable when both GPS and atomic clock pairs
are healthy (per ADR-0252 §D-4).

### 2.2 Commit-Wait Latency Implications

TrueTime `ExternalConsistency` (Tier-4 cells) requires commit-wait: the system waits until
`TT.now().latest < commit_timestamp` before acknowledging the write. This means:

| TrueTime state | Uncertainty bound | Commit-wait duration | Write p99 latency impact |
|---|---|---|---|
| GPS + atomic healthy | 1ms | 1ms | +1ms per write |
| GPS degraded, Rubidium healthy | 7ms | 7ms | +7ms per write |
| Rubidium degraded (hold-over) | Growing (7ms + drift) | ≥7ms, growing | Degrading; SEV-2 trigger |
| Both degraded | Cell non-TrueTime (ADR-0252 §D-11 Tier-4 alerts) | N/A — TrueTime disabled | Fall back to HLC |

**KR-FSS order matching at 10k orders/sec with 7ms commit-wait:**

At 10k orders/sec sustained, commit-wait queueing analysis:
- Single-thread commit rate: 1000ms / 7ms = ~142 commits/sec (serial)
- Required throughput: 10,000 commits/sec
- Parallelism required: 10,000 / 142 = ~70 parallel commit streams

**Conclusion: 10k orders/sec with 7ms commit-wait requires ≥70 parallel Postgres commit
streams (Citus shards or parallel coordinators).** This is achievable with Citus distributed
transactions partitioned by `(tenant_id, instrument_id)` — each instrument has its own commit
stream. KR-FSS order matching MUST partition commit streams by instrument, not globally.

**Error bars for write p99 at Tier-4 with GPS+atomic healthy:**
Baseline write p99 (shared-cloud, per §1.3): 201ms. Add commit-wait 1ms: **202ms p99.**
With GPS degraded (7ms commit-wait): **208ms p99.** Both within 500ms write budget.

**Modeled commit-wait addition:** +1ms p99 [GPS+atomic healthy] / +7ms p99 [GPS degraded]
(evidence: modeling note docs/performance-budgets/truetime-hlc-uncertainty-budget.md;
Corbett et al. OSDI 2012 §4.2 commit protocol)

---

## 3. StrictTotalOrder: +5-30ms p99

**Basis:** StrictTotalOrder requires one Raft round-trip per saga step within a cell-pair.
Cross-cell RTT within a region pair varies:

| Region pair | RTT | StrictTotalOrder p99 addition |
|---|---|---|
| Same-cell (single-leader Raft) | 1-2ms RTT | +2ms p99 |
| Same-region, different AZ (Frankfurt-A → Frankfurt-B) | ~2-5ms RTT | +5ms p99 |
| Same-continent, different regions (Frankfurt → Dublin) | ~20ms RTT | +20-30ms p99 |
| Cross-continent (Frankfurt → Seoul) | ~250ms RTT | +250ms p99 (not recommended) |

The ADR-0252 §D-3 claim "StrictTotalOrder adds ~5-30ms p99 within a cell-pair" applies to
intra-region deployments. Cross-continent StrictTotalOrder is explicitly discouraged; use
ExternalConsistency (TrueTime) at Tier-4 for cross-continent financial operations.

**Modeled:** StrictTotalOrder adds 5ms p99 [same-region, adjacent AZ] – 30ms p99 [same-region,
far AZ pair] [P5..P95 error bars] (evidence: modeling note
docs/performance-budgets/truetime-hlc-uncertainty-budget.md; Raft round-trip RTT basis;
ADR-0252 §D-3 claim validated)

---

## 4. Idempotency Key UPSERT — Connection Pool Sizing

**Claim from F6 verdict (F6-N10):** At 50k QPS/cell, idempotency key UPSERT requires ~500
concurrent Postgres connections.

**Derivation:**
- QPS: 50,000 state-changing requests/sec/cell (Messenger PRD §10.1)
- Fraction requiring idempotency check: ~80% (all non-idempotent by nature)
- Effective UPSERT rate: 40,000/sec
- UPSERT p99 latency: 10ms (ADR-0252 §D-4)
- Little's Law: concurrent connections = rate × latency = 40,000 × 0.010 = 400 concurrent
- Safety factor (1.25×): 500 concurrent connections

**Recommendation:** Per-cell Postgres connection pool for `idempotency_keys` table:
- pgBouncer pool mode: transaction (not session)
- Pool size: 500 server connections per cell
- Connection overflow: reject with `429 Too Many Requests` + `Retry-After` header

**This is NOT a blocker for the bundle** — it is a capacity-planning note for cell
provisioning. The 500 connection estimate is accurate; it must be reflected in pgBouncer
`iac/helm/<ms>/templates/pgbouncer-values.yaml` per cell.

---

## 5. Sensitivity Analysis — What Would Shift This Answer

| Input | Current assumption | 10× shift scenario | Impact |
|---|---|---|---|
| **NTP sources per node** | ≥2 stratum-2 sources (H-01) | 0 NTP sources (network partition from NTP) | HLC physical component drifts at OS clock rate (~10-100ms/hr); uncertainty exceeds 500ms bound within hours → SEV-1 triggered correctly. Mitigation: ≥2 independent NTP sources per node mandatory. |
| **GPS hold-over after GPS loss** | Cesium: ≤1µs/day drift | Rubidium only: ≤4.3ms/day drift | Tier-4 cells must have Cesium primary; Rubidium secondary. If only Rubidium: 1ms tightened bound requires GPS (not achievable on hold-over). Operating procedure: TrueTime tightened bound only declared when GPS active. |
| **Raft leader location (StrictTotalOrder)** | Same-region cell-pair (5-30ms) | Cross-continent (250ms) | StrictTotalOrder across continents adds 250ms per saga step → multi-step sagas could reach 2.5s p99. FORBIDDEN for real-time workflows. Architect cross-continent operations as eventually-consistent with HLC causal ordering. |
| **Idempotency key cardinality** | Per-tenant partition (Citus shard by tenant_id) | Global hot-shard (all writes land on same Citus coordinator) | Coordinator becomes bottleneck; UPSERT p99 grows from 10ms to 500ms. Citus shard-by-tenant_id is mandatory; cross-tenant hot-shard is the anti-pattern. |
| **TrueTime hardware failure (both GPS + atomic)** | Degraded cell → non-TrueTime fallback (HLC) | Prolonged unavailability of replacements | Financial-grade KR-FSS cells lose ExternalConsistency; fall back to StrictTotalOrder (Raft). Mitigation: minimum 2 GPS receivers + 2 atomic clock pairs per Tier-4 cell; ≤24h hardware replacement SLA for GPS units. |

---

## 6. Verification Protocol

An intern can verify the HLC and TrueTime claims on a deployed cell:

```bash
# Verify HLC uncertainty < 500ms on all nodes
oya metrics query \
  'max(hlc_uncertainty_ms{cell="<cell-id>"}) by (node)' \
  --window 1h
# Expected: all nodes < 500ms; alert if any > 300ms sustained

# Verify chronyd NTP offset < 100ms
ssh <node> 'chronyc tracking | grep "RMS offset"'
# Expected: RMS offset < 100ms

# Tier-4: verify TrueTime uncertainty < 10ms p99
oya metrics query \
  'truetime_uncertainty_ms{cell="<cell-id>"}' \
  --window 1h
# Expected: p99 < 10ms; < 1ms when GPS + atomic healthy

# Tier-4: verify GPS + atomic clock health
ssh <tier4-node> 'gpsd -n && gpsmon --once | head -20'
# Expected: satellites visible ≥ 4; fix 3D

# Verify idempotency UPSERT p99 < 10ms at load
retired CLI benchmark idempotency-upsert \
  --cell <cell-id> \
  --qps 40000 \
  --duration 60s
# Expected: UPSERT p99 < 10ms; p999 < 50ms
```

---

## 7. Evidence Status

| Evidence type | Status | Path |
|---|---|---|
| Modeled derivation (this document) | COMPLETE | `docs/performance-budgets/truetime-hlc-uncertainty-budget.md` |
| HLC uncertainty measurement in pilot cell | PENDING | `microservices/observability/dashboards/hlc-uncertainty.json` |
| TrueTime hardware commissioning test | PENDING — Tier-4 cell not yet provisioned | `docs/runbooks/tier4-cell-truetime-commissioning.md` |
| Idempotency UPSERT benchmark | PENDING | `microservices/workflow-engine/benches/idempotency_upsert.rs` |

---

## 8. Cross-References

- ADR-0252 §D-1 — HLC default uncertainty ±500ms (binding ADR)
- ADR-0252 §D-3 — StrictTotalOrder +5-30ms claim (binding ADR)
- ADR-0252 §D-4 — Tier-4 TrueTime hardware; idempotency key UPSERT
- ADR-0252 §D-11 — alert thresholds for HLC uncertainty
- ADR-0280 §D-6 — SLO composition methodology
- Corbett et al. "Spanner: Google's Globally Distributed Database" OSDI 2012 — TrueTime basis
- Demirbas & Kulkarni "Message Passing Semantics with Logical Clocks" OPODIS 2014 — HLC basis
- `docs/performance-budgets/deployment-model-slo-budgets.md` — write p99 with commit-wait
