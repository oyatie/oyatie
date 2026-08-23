---
purpose: "Index of all performance modeling notes. Every aspirational latency/throughput claim in a keystone ADR must cite either a benchmark commit or a modeling note in this directory."
doc_class: Index
status: published
date: 2026-05-20
authority: documentation-rigor.md §6 anti-patterns (aspirational performance numbers without evidence)
enforced_by: governance-doc-rigor
---

# Performance Budgets — Modeling Notes Index

Every performance claim in a keystone ADR that lacks a benchmark commit citation MUST reference
a modeling note in this directory. Modeling notes lay out assumption sets, per-stage
decompositions, sensitivity analyses, and verification protocols.

This directory closes the **F6 budget-honesty** promotion gate from the keystone-bundle
2026-05-20 multispectrum review (F6-B1, F6-B2, F6-B3 BLOCKERs).

See `docs/standards/documentation-rigor.md` §6 for the anti-pattern definition:
> "Aspirational performance numbers without evidence → Cite the benchmark commit OR a modeling
> note in docs/performance-budgets/."

---

## Modeling Note Index

| Slug | Title | Binding ADR(s) | Key metric(s) | Status |
|---|---|---|---|---|
| [cedar-hot-path-1ms-p99](cedar-hot-path-1ms-p99.md) | Cedar Hot-Path 1ms p99 — Engineered Budget Decomposition | ADR-0243 §D-6, ADR-0246 §D-6 | Hot-path Cedar eval: 385µs modeled p99 [P5..P95: 250µs–750µs]; leaves 615µs headroom against 1ms target | modeled |
| [cedar-hot-reload-propagation-dual-path](cedar-hot-reload-propagation-dual-path.md) | Cedar Fragment Hot-Reload Propagation — Dual-Path Reconciliation | ADR-0243 §D-10, ADR-0248 §D-9 | Path A (EMERGENCY push): 5s p99; Path B (constant-work pull): ≤35s p99 | modeled |
| [edge-first-byte-50ms-p99](edge-first-byte-50ms-p99.md) | Edge First-Byte 50ms p99 — Per-Stage Budget Decomposition | ADR-0253 §Consequences#1, §Consequences#2 | Scenario A (edge cache hit): ~9ms p99; Scenario B (dynamic, same-region): ~51ms p99; Scenario C (cold, cross-region): ~207ms p99 | modeled |
| [deployment-model-slo-budgets](deployment-model-slo-budgets.md) | Deployment Model SLO Budgets — Modeled Latency and Availability | ADR-0254 §D-8 | Shared-cloud read: 91ms modeled p99 [P5..P95: 91ms–180ms] (budget 200ms); write: 201ms modeled p99 [P5..P95: 201ms–450ms] (budget 500ms); availability: 99.95% [P5..P95: 99.85%..99.99%] | modeled |
| [truetime-hlc-uncertainty-budget](truetime-hlc-uncertainty-budget.md) | TrueTime / HLC Uncertainty Budget — Modeled Latency and Clock Assumptions | ADR-0252 §D-1, §D-3, §D-4 | HLC uncertainty: ±500ms [conservative; actual drift ≤100ms cross-region]; TrueTime commit-wait: +1ms p99 (GPS+atomic healthy) / +7ms p99 (GPS degraded); StrictTotalOrder: +5ms–30ms p99 [P5..P95 by region-pair] | modeled |

---

## Metric Quick-Reference

| Metric | Modeled value | Error bars (P5..P95) | Budget | ADR | Modeling note |
|---|---|---|---|---|---|
| Cedar hot-path eval p99 | 385µs | 250µs – 750µs | < 1ms | ADR-0243 §D-6 | cedar-hot-path-1ms-p99.md |
| Cedar cold-path eval p99 | — | — | < 50ms | ADR-0243 §D-6 | (inherent in Postgres ~10ms + compile ~20ms; no separate note needed) |
| Cedar hot-reload (Path A, EMERGENCY push) | ~3s | 1s – 5s | < 5s | ADR-0243 §D-10 | cedar-hot-reload-propagation-dual-path.md |
| Cedar hot-reload (Path B, snapshot pull) | ~15s | 10s – 35s | ≤ 35s | ADR-0248 §D-9 | cedar-hot-reload-propagation-dual-path.md |
| Edge first-byte (Scenario A, cache hit) | ~9ms | 5ms – 15ms | ≤ 50ms | ADR-0253 §C#1 | edge-first-byte-50ms-p99.md |
| Edge first-byte (Scenario B, dynamic, same-region) | ~51ms | 40ms – 75ms | ≤ 60ms | ADR-0253 §C#1 | edge-first-byte-50ms-p99.md |
| Edge Cedar eval penalty | ~5ms | 2ms – 8ms | ~5-10ms admitted | ADR-0253 §C#2 | edge-first-byte-50ms-p99.md |
| Shared-cloud read p99 | 91ms | 91ms – 180ms | < 200ms | ADR-0254 §D-8 | deployment-model-slo-budgets.md |
| Shared-cloud write p99 | 201ms | 201ms – 450ms | < 500ms | ADR-0254 §D-8 | deployment-model-slo-budgets.md |
| Hybrid/BYO-cloud read p99 | 151ms | 151ms – 240ms | < 250ms | ADR-0254 §D-8 | deployment-model-slo-budgets.md |
| Hybrid/BYO-cloud write p99 | 261ms | 261ms – 590ms | < 600ms | ADR-0254 §D-8 | deployment-model-slo-budgets.md |
| HLC uncertainty (default) | ≤ 100ms actual | ≤ 100ms – ≤ 500ms budget | ± 500ms | ADR-0252 §D-1 | truetime-hlc-uncertainty-budget.md |
| TrueTime commit-wait (GPS+atomic healthy) | 1ms | 0.5ms – 2ms | = TT uncertainty (1ms) | ADR-0252 §D-4 | truetime-hlc-uncertainty-budget.md |
| TrueTime commit-wait (GPS degraded) | 7ms | 5ms – 10ms | = TT uncertainty (7ms) | ADR-0252 §D-4 | truetime-hlc-uncertainty-budget.md |
| StrictTotalOrder overhead (same-region AZ pair) | 5ms | 2ms – 30ms | ~5-30ms | ADR-0252 §D-3 | truetime-hlc-uncertainty-budget.md |
| Shared-cloud availability | 99.95% | 99.85% – 99.99% | 99.95% | ADR-0254 §D-8 | deployment-model-slo-budgets.md |

---

## Conventions

1. **Modeled notes are stubs until promoted.** A modeling note at `status: modeled` is
   sufficient for `Proposed` ADR status. Before an ADR promotes to `Accepted`, the modeling
   note must be upgraded with a benchmark commit reference or replaced by a field measurement.

2. **Error bars are P5..P95 ranges**, not standard deviations. They represent the range of
   p99 values observed across different load conditions, hardware configurations, and
   deployment scenarios described in each note.

3. **Budget vs. measured:** The "budget" column is the claim in the binding ADR. The "modeled
   value" is what this note's analysis predicts. Budget > modeled value = headroom exists.

4. **Sensitivity dominance:** Each modeling note includes a sensitivity analysis identifying
   the single input that would cause a 10× budget overshoot. The dominant factor is noted
   in §5 of each modeling note.

5. **Benchmark commits:** When a benchmark lands, update the modeling note's
   `evidence_status` table and add the commit SHA. The ADR performance claim in the ADR body
   should also be updated to cite the commit directly.

---

## Adding a New Modeling Note

When a new ADR introduces a performance claim without a benchmark commit:

1. Create `docs/performance-budgets/<slug>.md` using the template shape from any existing
   note in this directory.
2. Required sections: Purpose, Topology Assumptions, Per-Stage Budget Decomposition,
   Sensitivity Analysis (§ "What Would Shift This Answer"), Verification Protocol,
   Evidence Status, Cross-References.
3. Add a row to the index table above (this file).
4. Add a `budget_evidence` citation in the ADR body pointing to this file:
   ```
   (modeled; assumptions in docs/performance-budgets/<slug>.md)
   ```
5. CI lane `governance-doc-rigor` will verify the citation is present and the
   modeling note exists.

---

## Related

- `docs/standards/documentation-rigor.md` §6 — anti-pattern definition
- ADR-0280 §D-6 — SLO composition Markov formula (the gold-standard model)
- `microservices/*/slos/*.openslo.yaml` — per-µservice OpenSLO declarations
