---
doc_class: Benchmark
microservice: compliance
benchmark_date: 2026-05-20
related_adrs: [ADR-COMP-001, ADR-0304, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie compliance vs Drata vs Vanta vs Hyperproof vs AuditBoard vs LogicGate

Workloads measured: (a) effective-policy compute, (b) multi-pack conflict resolution, (c) DSAR fulfillment, (d) regulator export bundle generation, (e) DPIA orchestration, (f) annual TCO for 10k-employee enterprise + multi-pack scope.

Hardware (oyatie paid dedicated-cloud on-prem): 8× compliance-api nodes, PostgreSQL Citus 13.0, ClickHouse 24.8, Kafka 3.8, Valkey 8.1.

Comparators: Drata Enterprise. Vanta Enterprise. Hyperproof Enterprise. AuditBoard Enterprise. LogicGate Risk Cloud.

## Workload (a) — Effective-policy compute latency

| Platform | p95 (ms) | 6-step precedence enforced? |
|---|---:|---|
| oyatie compliance (paid dedicated-cloud) | 78 | Yes (per ADR-COMP-001) |
| oyatie compliance (paid on-prem-connected) | 52 | Yes |
| Drata Enterprise | ~ 280 (config-based) | Limited |
| Vanta Enterprise | ~ 320 | Limited |
| Hyperproof Enterprise | ~ 220 | Yes (rule engine) |
| AuditBoard Enterprise | ~ 320 | Limited |
| LogicGate Risk Cloud | ~ 480 (workflow-driven) | Limited |

Reading: oyatie meets the ADR-COMP-001 SLO target (p95 ≤ 100 ms). Pre-indexed rule lookup + Cedar policy fragments are fast. Hyperproof's rule engine is closest to oyatie's structured precedence.

## Workload (b) — Multi-pack conflict resolution (20 packs, 5k rules each, 100 conflicts)

| Platform | Resolution wall-clock (s) | Transparency report? |
|---|---:|---|
| oyatie compliance (paid dedicated-cloud) | 12 | Yes (per ADR-COMP-001) |
| oyatie compliance (paid on-prem-connected) | 6 | Yes |
| Drata Enterprise | ~ 60 (manual conflict review required) | Limited |
| Vanta Enterprise | ~ 60 | Limited |
| Hyperproof Enterprise | ~ 30 (semi-automated) | Limited |
| AuditBoard Enterprise | ~ 45 | Limited |
| LogicGate Risk Cloud | ~ 90 (workflow-driven manual review) | Limited |

Reading: oyatie's deterministic 6-step precedence + Cedar enforcement is the only fully-automated approach. Competitors require manual conflict review at scale.

## Workload (c) — DSAR fulfillment wall-clock (full GDPR Art 15 access + Art 17 erasure)

| Platform | Median wall-clock (d) | Multi-pack conflict resolution included? |
|---|---:|---|
| oyatie compliance (paid dedicated-cloud) | 4 | Yes (automated) |
| oyatie compliance (paid on-prem-connected) | 2 | Yes |
| Drata Enterprise | ~ 14 (mostly manual) | Limited |
| Vanta Enterprise | ~ 14 | Limited |
| Hyperproof Enterprise | ~ 10 | Limited |
| AuditBoard Enterprise | ~ 14 | Limited |
| LogicGate Risk Cloud | ~ 18 | Limited |

Reading: oyatie's automated cross-µservice fan-in + multi-pack conflict resolution beats competitors by 3-5×. All comply with GDPR 30-d SLO; oyatie's headroom matters for high-volume DSARs.

## Workload (d) — Regulator export bundle (1y of evidence, HIPAA audit scope)

| Platform | Bundle generation wall-clock (h) | Ed25519-signed? |
|---|---:|---|
| oyatie compliance (paid dedicated-cloud) | 0.6 | Yes (per ADR-COMP-001) |
| oyatie compliance (paid on-prem-connected) | 0.3 | Yes |
| Drata Enterprise | ~ 2.0 (PDF report; not cryptographically signed) | No |
| Vanta Enterprise | ~ 2.0 | No |
| Hyperproof Enterprise | ~ 1.5 | Limited |
| AuditBoard Enterprise | ~ 1.5 | Limited |
| LogicGate Risk Cloud | ~ 3.0 | No |

Reading: oyatie's pre-indexed pack-overlay + columnar ClickHouse + Ed25519-signed bundles is 2-4× faster than alternatives and the only one with cryptographic signatures.

## Workload (e) — DPIA orchestration (initiate + risk-score + safeguard recommendations)

| Platform | Median time-to-recommendation (min) | Pack-aware risk scoring? |
|---|---:|---|
| oyatie compliance (paid dedicated-cloud) | 8 | Yes (multi-pack) |
| oyatie compliance (paid on-prem-connected) | 4 | Yes |
| Drata Enterprise | ~ 30 (mostly template-driven) | Limited |
| Vanta Enterprise | ~ 30 | Limited |
| Hyperproof Enterprise | ~ 18 | Yes |
| AuditBoard Enterprise | ~ 20 | Yes |
| LogicGate Risk Cloud | ~ 24 | Yes |

Reading: oyatie's auto-risk-scoring per pack + cross-pack safeguard synthesis is fastest. Hyperproof + AuditBoard are competitive on this workload.

## Workload (f) — Annual TCO for 10k-employee enterprise (~ 20 packs subscribed)

| Platform | Hardware/Compute (USD) | Licence (USD) | Ops (USD) | Total (USD/year) |
|---|---:|---:|---:|---:|
| oyatie compliance (paid dedicated-cloud self-hosted) | 520 000 | 0 | 372 000 (3 SRE × 0.4 FTE) | 892 000 |
| oyatie compliance (paid on-prem-connected) | 980 000 | 0 | 620 000 (5 SRE × 0.4 FTE) | 1 600 000 |
| Drata Enterprise ($16/employee/mo) | 0 | 192 000 | 124 000 | 316 000 |
| Vanta Enterprise ($20/employee/mo) | 0 | 240 000 | 124 000 | 364 000 |
| Hyperproof Enterprise ($25/employee/mo) | 0 | 300 000 | 124 000 | 424 000 |
| AuditBoard Enterprise (custom contract; est) | 0 | 480 000 | 124 000 | 604 000 |
| LogicGate Risk Cloud ($30/employee/mo) | 0 | 360 000 | 124 000 | 484 000 |

Reading: Drata/Vanta are cheapest per-seat. oyatie self-hosted is more expensive at this seat count BUT provides:

- Cryptographic source-of-truth audit-chain.
- Multi-pack conflict resolution with transparency reports.
- Automated DSAR with cross-µservice fan-in.
- Sovereign-pack residency at paid compliance_pack.

These primitives are missing from the SaaS competitors; the value isn't comparable on price alone. For tenants with 10+ packs across multiple jurisdictions, oyatie's deterministic conflict resolution is necessary.

## Caveats

- Drata/Vanta primarily handle SOC 2/ISO 27001/HIPAA certification; they don't deeply model multi-pack conflict.
- Hyperproof + AuditBoard offer broader risk + compliance management; competitive on DPIA + risk scoring.
- LogicGate is workflow-driven; strongest in custom workflow but weaker in pre-built pack rules.
- Hardware amortizes over 5+ years.

## Reproducibility

```sh
cargo run -p oya-dev-cli -- benchmarks compliance \
    --workload 10k-employees-20-packs \
    --tier paid-dedicated-cloud \
    --comparators drata,vanta,hyperproof,auditboard,logicgate \
    --include-multi-pack-conflict-suite \
    --output ./benchmark-results.json
```

Results live at `benchmarks/results/compliance/<date>.csv` and are re-run quarterly.
