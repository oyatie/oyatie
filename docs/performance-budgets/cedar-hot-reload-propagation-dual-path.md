---
slug: cedar-hot-reload-propagation-dual-path
title: "Cedar Fragment Hot-Reload Propagation — Dual-Path Reconciliation"
binding_adrs:
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0248-amazon-shape-cellular-architecture.md
status: modeled
date: 2026-05-20
authors:
  - council-architecture
  - axis-policy-engine
  - ops-sre-reliability
related_budgets:
  - cedar-hot-path-1ms-p99.md
  - cedar-cold-path-50ms-p99.md
---

# Cedar Fragment Hot-Reload Propagation — Dual-Path Reconciliation

## Purpose

This modeling note reconciles the apparent contradiction between two performance claims in the
keystone bundle:

- **ADR-0243 §D-10:** Cedar fragment hot-reload propagates to all evaluator replicas in a cell
  within **5s p99**.
- **ADR-0248 §D-9:** Tier 2 policy-engine publishes a versioned snapshot every **30 seconds**;
  Tier 3 cells pull the snapshot.

The F6-B2 BLOCKER finding (keystone-bundle-2026-05-20-F6-performance-r1.json) identified these
as contradictory. This note declares the canonical **dual-path model** that makes both claims
simultaneously true and operationally correct.

---

## 1. The Two Distribution Paths

The Cedar fragment distribution system uses two distinct paths with different purposes,
latency characteristics, and failure modes. They are not alternatives — they are complementary.

### Path A: Push-based emergency notification (Kafka pub-sub)

**Purpose:** Emergency Cedar permits, incident-response fragments, security-critical policy
changes that MUST propagate within seconds.

**Mechanism:**
1. `policy-engine` Tier 2 publishes a `FragmentPublished` event to Kafka topic
   `policy-engine.fragment.published` (per ADR-0050 Kafka substrate) immediately on
   `ActivateFragment` API call.
2. Every Tier 3 cell's `policy-engine-evaluator` DaemonSet subscribes to this topic via
   consumer group `fragment-reload-<cell-id>`.
3. On receiving `FragmentPublished`, the evaluator fetches the specific fragment from the
   fragment registry, recompiles the affected bundle subset, and atomic-swaps it into the
   active evaluator — all within the DaemonSet replica.
4. HPA replica coordination: after the first replica completes reload, it publishes a
   `FragmentReloaded` event to a per-cell internal topic; other replicas listen and execute
   reload in parallel.

**Latency model:**
| Stage | p50 | p99 | p999 |
|---|---|---|---|
| Kafka `FragmentPublished` publish latency (Tier 2 → broker) | 5ms | 20ms | 50ms |
| Kafka consumer poll interval (Tier 3 evaluator DaemonSet) | 100ms | 500ms | 1s |
| Fragment fetch from registry (Postgres, fragment ~4KB) | 5ms | 20ms | 50ms |
| Cedar AST recompile for affected bundle subset | 100ms | 500ms | 1s |
| Atomic swap into active evaluator (all DaemonSet replicas, parallel) | 50ms | 2s | 4s |
| **Total end-to-end (Tier 2 publish → all Tier 3 replicas reloaded)** | **260ms** | **~3s** | **~6s** |

**Declared p99:** < 5s (the 5s p99 claim in ADR-0243 §D-10 is achievable on Path A).

**When used:**
- Emergency Cedar permit fragments (ADR-0243 Appendix B incident-response-emergency-key flow)
- Security-critical fragment activation (immediate propagation required)
- Any `ActivateFragment` call carrying `priority: EMERGENCY` context attribute

### Path B: Constant-work periodic snapshot pull (per Brooker 2020)

**Purpose:** Steady-state distribution of all Cedar fragments for static stability. Every Tier
3 cell maintains a consistent, complete view of the full fragment set even if Tier 2 becomes
temporarily unreachable (ADR-0248 §D-8: 24-hour isolation tolerance).

**Mechanism:**
1. Tier 2 `policy-engine` publishes a versioned snapshot of ALL active Cedar fragments every
   **30 seconds** (configurable per cell; default 30s) to an object-store snapshot path
   (e.g., `s3://policy-snapshots/<cell-id>/latest.bundle`).
2. Tier 3 cells' `policy-engine-evaluator` DaemonSet polls the snapshot path on a 30-second
   jittered cadence (jitter: ±5s to avoid thundering-herd across cells).
3. If the snapshot version matches the cached version, no recompile occurs. If it differs,
   the evaluator downloads and recompiles the full bundle, then atomic-swaps.

**Latency model:**
| Stage | p50 | p99 | p999 |
|---|---|---|---|
| Time since last snapshot publish | 0–30s (uniform) | 29s | 30s |
| Snapshot download (full bundle, ~200 fragments × 4KB = 800KB compressed) | 200ms | 800ms | 2s |
| Cedar AST full recompile (all 200 fragments) | 500ms | 2s | 4s |
| Atomic swap into active evaluator | 50ms | 500ms | 1s |
| **Total worst-case (fragment activated → Tier 3 sees it)** | **~15s** | **~32s** | **~37s** |

**Declared p99:** ≤ 35s for a fragment published via Tier 2 snapshot path alone (30s pull
cadence + up to 5s recompile).

**When used:**
- All standard `ActivateFragment` calls without `priority: EMERGENCY`
- Catch-up path after Tier 2 partition recovery
- Static stability assurance (24-hour isolation tolerance; fragment TTL in the snapshot is
  24 hours, per ADR-0248 §D-8)

---

## 2. The Canonical Dual-Path Policy

```
For every Cedar fragment activation:

  IF priority == EMERGENCY:
    → Path A (Kafka push, 5s p99 propagation)
    → Fragment is also included in next Path B snapshot (idempotent)

  ELSE:
    → Path B only (30s pull cadence, ≤35s p99 propagation)
    → Path A NOT triggered (constant-work discipline: no per-change push for routine updates)
```

**Why not always use Path A?** The constant-work doctrine (ADR-0248 §D-9, Brooker 2020)
explicitly rejects per-change push semantics for standard updates: work scales as
`O(change_rate × fleet_size)` which grows unboundedly. Path A is reserved for the rare
emergency case (estimated < 1% of fragment activations).

**Why not always use Path B?** Emergency fragments (incident-response-emergency-key activations
per ADR-0243 Appendix B) MUST propagate within seconds. A 35s propagation delay during an
active security incident is unacceptable.

---

## 3. Incident Response Runbook Update

The emergency permit scenario in ADR-0243 Appendix B states a 5s propagation assumption. This
is correct for Path A (EMERGENCY priority activation). The runbook at
`docs/runbooks/cedar-hsm-root-key-ceremony.md` and ADR-0243 Appendix B MUST be updated to
explicitly state:

> **Emergency Cedar fragments activated via `ActivateFragment` with `priority: EMERGENCY`
> propagate via Kafka pub-sub (Path A) and reach all cell evaluator replicas within 5s p99.**
> Standard fragment activations propagate via the 30-second snapshot pull (Path B) and reach
> all cells within 35s p99.

---

## 4. Sensitivity Analysis — What Would Shift This Answer

| Input | Current assumption | 10× shift scenario | Impact |
|---|---|---|---|
| **Kafka consumer poll interval** | 100ms default, 500ms p99 (Path A) | 5s poll interval (misconfigured consumer) | Path A end-to-end p99 grows from ~3s to ~8s — FAILS the 5s p99 guarantee. Critical: poll interval MUST be ≤ 500ms for EMERGENCY consumers. |
| **Number of active fragments** | 200 fragments, 800KB bundle (Path B) | 2000 fragments, 8MB bundle | Full recompile time on Path B grows ~10× to ~20s; total Path B p99 grows to ~52s. Fragment cap enforcement is the mitigation. |
| **Snapshot interval** | 30s (Path B) | 300s (misconfigured; 5-minute interval) | Path B worst-case p99 grows to ~305s — fragment changes visible to Tier 3 after 5+ minutes. This is acceptable for static stability but not for any latency-sensitive deployment. The 30s default MUST be treated as a floor for production cells. |
| **Cell count** | ≤ 100 Tier 3 cells (ADR-0248 §D-7) | 1000 cells (scale-out) | Path A: Kafka fan-out to 1000 consumer groups; at ≤100ms poll interval, this is Kafka 1-to-1000 fan-out — well within Kafka's design parameters. Path B: each cell pulls independently; no fan-out amplification. Both paths scale. |
| **Tier 2 → Tier 3 isolation (static stability)** | Up to 24h isolation (ADR-0248 §D-8) | Permanent partition | Path B self-contained: Tier 3 continues serving from last-good snapshot with 24h TTL. Path A fails immediately (no Kafka path). This is the intended behaviour: EMERGENCY fragments cannot be activated during Tier 2 partition — they require Tier 2 reachability. |

---

## 5. Verification Protocol

An intern can verify the dual-path behaviour on a deployed cell:

```bash
# Verify Path A (EMERGENCY push propagation)
# 1. Activate a test fragment with EMERGENCY priority
oya policy activate-fragment \
  --fragment-id test-emergency-probe \
  --priority EMERGENCY \
  --cell <cell-id>

# 2. Watch evaluator reload events (should appear within 5s)
oya metrics query \
  'cedar_fragment_reload_duration_seconds{cell="<cell-id>",path="push"}' \
  --window 10s
# Expected: p99 < 5s

# Verify Path B (snapshot pull propagation)
# 1. Activate a standard fragment (no EMERGENCY priority)
oya policy activate-fragment \
  --fragment-id test-standard-probe \
  --cell <cell-id>

# 2. Watch snapshot pull cycle (should complete within 35s)
oya metrics query \
  'cedar_snapshot_pull_age_seconds{cell="<cell-id>"}' --window 60s
# Expected: max < 35s

# Verify Path A consumer poll interval (must be ≤500ms)
kubectl -n policy-engine get configmap fragment-reload-consumer-config \
  -o jsonpath='{.data.poll_interval_ms}'
# Expected: <= 500
```

---

## 6. Evidence Status

| Evidence type | Status | Path |
|---|---|---|
| Dual-path model declaration (this document) | COMPLETE | `docs/performance-budgets/cedar-hot-reload-propagation-dual-path.md` |
| Path A implementation (Kafka consumer in evaluator DaemonSet) | IMPLEMENTATION REQUIRED | `microservices/policy-engine/src/fragment_reload_consumer.rs` |
| Path B implementation (snapshot pull loop) | PARTIALLY IMPLEMENTED — see ADR-0248 §D-9 | `microservices/policy-engine/src/snapshot_pull.rs` |
| OpenSLO for Path A p99 | PENDING | `microservices/policy-engine/slos/fragment-reload-push-propagation.openslo.yaml` |
| OpenSLO for Path B p99 | PENDING | `microservices/policy-engine/slos/fragment-reload-pull-propagation.openslo.yaml` |

---

## 7. Cross-References

- ADR-0243 §D-10 — hot-reload propagation claim (5s p99 via Path A)
- ADR-0248 §D-9 — constant-work snapshot pull model (30s cadence via Path B)
- ADR-0243 Appendix B — emergency Cedar permit scenario (relies on Path A)
- ADR-0280 §D-6 — SLO composition methodology
- `docs/performance-budgets/cedar-hot-path-1ms-p99.md` — hot-path evaluation budget
- `docs/runbooks/cedar-fragment-emergency-activation.md` — emergency activation runbook
