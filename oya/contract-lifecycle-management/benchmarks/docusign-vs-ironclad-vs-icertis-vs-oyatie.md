---
doc_class: Benchmark
microservice: contract-lifecycle-management
status: wave-4-rolling-remediated
date: 2026-05-21
top_3_counterparts:
  - Ironclad
  - DocuSign CLM
  - Conga CLM
related_adrs: [ADR-0328, ADR-0329, ADR-0330, ADR-0331]
note: |
  Filename retained for legacy reference. Per audit X-D3 resolution, the canonical top-3 counterparts are
  Ironclad / DocuSign CLM / Conga CLM. Icertis is referenced for legacy context but not in the canonical
  benchmark set. Per ADR-0329, workload tables drawn as (deployment_context × tenant_class) instead of
  retired named capability levels tiers.
---

# Benchmarks — CLM Workload Numbers

This document captures quantitative workload benchmarks for Oyatie CLM against the canonical top-3 counterparts (Ironclad, DocuSign CLM, Conga CLM). Per the audit T-017 remediation, the workload tables are drawn as `(deployment_context × tenant_class)` instead of retired named capability levels tiers.

Full benchmarking methodology in `performance-benchmark-numbers-2026-05-20.md`.

## 1. Draft generation throughput

| Configuration | Throughput (drafts/min) | Latency p95 |
|---|---|---|
| Oyatie demo_trial on `oci-guest/always-free` | 12 | 4.2s |
| Oyatie paid + per_seat on `oyatie-public-cloud` standard cell | 240 | 0.4s |
| Oyatie paid + per_seat on `aws-guest` standard cell | 320 | 0.3s |
| Oyatie paid + per_seat + per_usage on `oyatie-public-cloud` sovereign EU cell | 280 | 0.5s |
| Oyatie paid on `on-prem` 8-node cell | 220 | 0.6s |
| Ironclad standard plan | ~180 | 0.6s |
| DocuSign CLM standard plan | ~200 | 0.7s |
| Conga CLM (Salesforce-native) | ~120 | 1.2s |

## 2. AI redlining latency

50-page contract, AI clause suggestion.

| Configuration | p95 latency | Model |
|---|---|---|
| Oyatie demo_trial | N/A (AI gated off) | N/A |
| Oyatie paid on `oyatie-public-cloud` standard | 2.8s | Llama-3.1-8B local |
| Oyatie paid on `oyatie-public-cloud` enterprise | 1.4s | Llama-3.1-70B local |
| Oyatie paid on `oyatie-public-cloud` + cross-emit Claude | 1.8s | Claude 3.7 Sonnet |
| Oyatie paid on `on-prem` with NVIDIA L40S GPU | 1.2s | Llama-3.1-70B local |
| Oyatie paid on sovereign cell with BYOK model | 2.5s | Tenant-fine-tuned 70B |
| Ironclad Jurist | ~2.0s | Proprietary |
| DocuSign Insight | ~3.0s | Proprietary |
| Conga AI | ~3.5s | Proprietary |

## 3. E-signature delivery latency

Single signatory, AES envelope.

| Configuration | p95 latency |
|---|---|
| Oyatie demo_trial AES via platform DocuSign | 4.2s |
| Oyatie paid + per_seat AES via platform DocuSign | 2.4s |
| Oyatie paid + per_seat AES via BYOK DocuSign | 2.6s |
| Oyatie paid + per_seat AES via Adobe Sign | 2.5s |
| Oyatie paid + per_seat native AES envelope | 1.8s |
| Oyatie paid QES via Oyatie-leased Thales Luna 7 | 4.1s |
| Oyatie paid QES via BYOK Thales Luna 7 on-prem | 3.6s |
| Ironclad → DocuSign AES | 3.8s |
| DocuSign CLM → DocuSign AES | 2.2s |
| Conga CLM → DocuSign AES | 4.0s |

## 4. Obligation extraction quality

Test set: 1,000 sample MSAs annotated by paralegal panel; ground-truth obligation count = 8,400.

| Configuration | F1 score | Auto-propose rate | Human review rate |
|---|---|---|---|
| Oyatie paid Llama-3.1-8B | 0.91 | 64% | 36% |
| Oyatie paid Llama-3.1-70B | 0.96 | 84% | 16% |
| Oyatie paid Claude 3.7 Sonnet cross-emit | 0.97 | 88% | 12% |
| Oyatie paid GPT-4o cross-emit | 0.96 | 86% | 14% |
| Ironclad Jurist | 0.93 | (proprietary metric) | (proprietary) |
| DocuSign Insight | 0.91 | (proprietary metric) | (proprietary) |
| Conga AI | 0.89 | (proprietary metric) | (proprietary) |

## 5. TCO at 500 users + 100,000 contracts/year

3-year TCO including licensing, hosting, integration, training, change management.

| Configuration | 3-year TCO (USD) | Notes |
|---|---|---|
| Oyatie demo_trial on OCI Always Free | $0 | Caps apply; not full production |
| Oyatie paid + per_seat on `oyatie-public-cloud` | $1.2M | Standard cell, multi-region |
| Oyatie paid + per_seat on `aws-guest` (customer-AWS) | $0.9M | Customer pays AWS directly |
| Oyatie paid + per_seat on `oci-guest` | $0.8M | OCI pricing typically lower than AWS |
| Oyatie paid + per_seat on `on-prem` | $1.4M | Customer hardware + Oyatie license |
| Oyatie paid sovereign + HSM BYOK | $1.6M | HSM + cell + license |
| Ironclad enterprise | $1.8-2.4M | Per-seat + add-ons |
| DocuSign CLM enterprise | $1.5-2.1M | Per-seat + envelope |
| Conga CLM Salesforce-native | $2.2-2.8M | Per-seat + Salesforce license dependency |

## 6. Sovereign-pack feature parity

| Capability | Oyatie | Ironclad | DocuSign CLM | Conga CLM |
|---|---|---|---|---|
| EU eIDAS QES native | ✓✓✓ | Via DocuSign | DocuSign EU Trust List | Via DocuSign |
| KR-PIPA sovereign cell | ✓✓✓ | ✗ | ✗ | ✗ |
| HIPAA-BAA pack overlay | ✓✓✓ | ✓ (via add-on) | ✓ (via add-on) | ✓ (via add-on) |
| SOX-404 + WORM | ✓✓✓ | ✓ (via add-on) | ✓ (via add-on) | ✓ (via add-on) |
| SEC 17a-4(f) WORM | ✓✓✓ | ✓ (via add-on) | ✓ (via add-on) | ✓ (via add-on) |
| Air-gap on-prem | ✓✓✓ | ✗ | ✗ | Limited |
| Provider BYOK HSM | ✓✓✓ | ✗ | ✗ | ✗ |
| FIPS 140-3 Level 3 | ✓✓✓ | Limited | ✓ (DocuSign EU) | Limited |

## 7. Multi-region failover

Cross-region failover drill, p95 RTO/RPO.

| Configuration | RTO | RPO |
|---|---|---|
| Oyatie paid on `oyatie-public-cloud` (3 regions) | 38 min | 22s |
| Oyatie paid + sovereign cell (per-region cell) | 45 min | 45s |
| Ironclad multi-region | ~60 min | ~60s |
| DocuSign CLM multi-region | ~45 min | ~30s |
| Conga CLM (Salesforce) | (per Salesforce SLA; ~60 min RTO, ~5 min RPO) | |

## 8. Migration import throughput

Bulk migration from each counterpart (contracts/hour).

| Source | Throughput | Notes |
|---|---|---|
| Ironclad export → Oyatie | 4,800/hr | Per `migration-playbooks/from-ironclad.md` |
| DocuSign CLM export → Oyatie | 6,000/hr | Per `migration-playbooks/from-docusign-clm.md` |
| Conga CLM (Salesforce Bulk API) → Oyatie | 3,200/hr | Per `migration-playbooks/from-conga-clm.md`; Salesforce API rate limits |

## Methodology

All measurements taken on Oyatie reference cells:

- `oci-guest/always-free` cell: 1 OCPU Ampere A1 + 4 GB RAM + 50 GB block.
- `oyatie-public-cloud` standard cell: 8 vCPU + 32 GB RAM + 1 TB NVMe + Llama-3.1-8B local.
- `oyatie-public-cloud` enterprise cell: 16 vCPU + 64 GB RAM + 4 TB NVMe + NVIDIA L40S GPU + Llama-3.1-70B local.
- `on-prem` reference: 8-node Kubernetes cluster + Talos OS + Cloud Hypervisor.
- Sovereign EU cell: Frankfurt + Paris + Dublin replicas; D-Trust QSCD HSM; D-Trust QTSA.
- Sovereign KR cell: Seoul + Busan replicas; KISA TSA; Yessign certificates.

Counterpart figures cited from public benchmarks + analyst reports (Gartner Magic Quadrant for CLM 2024-2025, Forrester Wave Q4 2024); accurate at time of measurement.

## Wave 15A remediation closure

Per audit T-017, the prior tier-stratified rows (retired named capability levels across 6 workload tables ≈ 18-24 row scrubs) have been redrawn as `(deployment_context × tenant_class)` rows. The legally substantive numeric content (Thales Luna 7 A790, NVIDIA L40S, Llama-3.1 / Claude / GPT-4o, KISA TSA, D-Trust TSA, SeaweedFS WORM, Yessign certificate authority, all measured numbers) is preserved.
