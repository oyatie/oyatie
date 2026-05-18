---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-observability + council-architecture
deciders: axis-observability, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0139]
related_artifacts:
  - microservices/observability/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-OBS gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (observability µservice)

## Purpose

Quantitative + qualitative parity comparison vs the industry-leading SLO-centric observability products. Drives the `oya-foundry-fitness-hyperscaler-maturity-claims` gate (per ADR-0123 HG-OBS) and tells gtm-customer-success what to say + what NOT to say in tenant sales conversations. Re-validated bi-annually because the competitor landscape moves.

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Grafana Labs | Grafana Cloud SLO | OpenSLO-aligned; on-call integrated | `grafana.com/products/cloud/slo/` |
| Datadog | Datadog SLOs | Mature 4-window burn-rate UX; broad integration | `docs.datadoghq.com/service_management/service_level_objectives/` |
| Nobl9 | Nobl9 SLO platform | OpenSLO-native vendor; multi-source SLI | `docs.nobl9.com` |
| Sloth | Open-source OpenSLO→Prometheus rule generator | OpenSLO authoring tool | `github.com/slok/sloth` |
| Google Cloud Monitoring | Service Monitoring SLOs | Per-service SLO UI; integrated with Cloud Run / GKE | `cloud.google.com/monitoring/slo` |
| New Relic | New Relic SLOs | Distributed-tracing-led SLI computation | `docs.newrelic.com/docs/service-level-management/` |
| Honeycomb | Honeycomb SLO | Wide-event-shape SLI; BubbleUp anomaly detection | `docs.honeycomb.io/working-with-your-data/slos/` |

## Feature Parity Matrix

### SLI authoring

| Capability | oyatie | Grafana | Datadog | Nobl9 | Sloth | GCP | New Relic | Honeycomb |
|---|---|---|---|---|---|---|---|---|
| OpenSLO v1.0 native authoring | ✅ | ✅ | converter | ✅ | ✅ | ❌ | ❌ | ❌ |
| GitOps SLO manifest (PR-reviewed) | ✅ | partial | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ |
| Multi-window multi-burn-rate (Google SRE) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Availability + Latency + Correctness + Freshness SLI types | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ | ✅ |
| Tenant-defined SLOs (multi-tenant SaaS shape) | ✅ | per-org | per-org | per-org | n/a | per-project | per-account | per-team |
| Schema validation API (anonymous) | ✅ | ❌ | ❌ | ❌ | CLI-only | ❌ | ❌ | ❌ |

### Gate integration (the differentiator)

| Capability | oyatie | Grafana | Datadog | Nobl9 | Sloth | GCP | New Relic | Honeycomb |
|---|---|---|---|---|---|---|---|---|
| Promotion gate driven by SLO verdict | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Per-component release pointer | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Automated rollback on production burn-rate breach | ✅ | ❌ | partial (anomaly alert) | ❌ | ❌ | partial (Cloud Deploy + Cloud Monitoring) | partial (alert→webhook) | partial (alert→webhook) |
| Canary cohort weighting via SLO signal | ✅ | manual | manual | manual | manual | partial (Cloud Deploy) | manual | manual |
| Git-tracked promotion audit (cryptographic) | ✅ Ed25519 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Multispectrum changeset evidence | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

### Substrate (Layer-A)

| Capability | oyatie | Grafana | Datadog | Nobl9 | Sloth | GCP | New Relic | Honeycomb |
|---|---|---|---|---|---|---|---|---|
| Self-hosted (no vendor lock) | ✅ (Grafana stack OSS) | partial (Cloud is SaaS; OSS available) | ❌ (SaaS only) | ❌ (SaaS only) | ✅ | ❌ (SaaS only) | ❌ (SaaS only) | ❌ (SaaS only) |
| Multi-region data-residency | ✅ (11 packs) | ✅ | ✅ | partial | n/a | ✅ | ✅ | ✅ |
| HIPAA BAA | conditional | ✅ | ✅ | ✅ | n/a | ✅ | ✅ | ✅ |
| KR PIPA compliance | conditional | partial | partial | ❌ | n/a | partial | partial | ❌ |
| EU GDPR DPA | ✅ | ✅ | ✅ | ✅ | n/a | ✅ | ✅ | ✅ |
| PromQL + LogQL + TraceQL + Pyroscope | ✅ | ✅ | proprietary | partial | n/a | proprietary | proprietary | proprietary |

### Operations + integrations

| Capability | oyatie | Grafana | Datadog | Nobl9 | Sloth | GCP | New Relic | Honeycomb |
|---|---|---|---|---|---|---|---|---|
| On-call paging | Grafana OnCall (OSS) | ✅ | ✅ | partial | n/a | partial | ✅ | partial |
| Multi-language SDK | M01: Rust; M01+1: TS; M02: Py/Go; M03: JVM | ✅ | ✅ | ✅ | n/a | ✅ | ✅ | ✅ |
| Cedar / Rego / OPA policy integration | ✅ Cedar | ❌ | ❌ | ❌ | ❌ | partial (IAM) | ❌ | ❌ |
| Tenant isolation (multi-tenant Mimir) | ✅ | ✅ | per-org | per-org | n/a | per-project | per-account | per-team |

## Quantitative Performance Parity

(All numbers reference 30-day rolling-window evaluations on equivalent workloads.)

| Metric | oyatie target | Grafana Cloud reference | Datadog reference | Notes |
|---|---|---|---|---|
| Eligibility verdict latency p99 (eval→ledger) | ≤ 2s | n/a (no gate) | n/a (no gate) | oyatie unique |
| Burn-rate alert fire latency p99 (signal→alert) | ≤ 60s | ≤ 30s | ≤ 30s | parity within ~30s |
| OpenSLO manifest reload | ≤ 3s | n/a | n/a | oyatie unique (Grafana doesn't hot-reload) |
| Mimir ingest p99 | per Mimir bench: 1ms/sample (single-pod throughput) | Grafana Cloud: ~1ms | proprietary | parity (same Mimir) |
| Dashboard query p99 | ≤ 1s (1h range) | Grafana: ≤ 1s | Datadog: ≤ 2s | parity |
| 30-day retention storage cost per 1B samples (estimated) | ~$170/mo cold-tier @ OCI | Grafana Cloud: ~$300+ | Datadog: ~$500+ | oyatie advantage via self-host |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | Multi-language SDK breadth (Py / Go / JVM) | axis-observability | M02–M03 |
| 2 | Service-mesh native traffic split (Istio + Linkerd; we have Istio only today) | ops-sre-reliability | M02 |
| 3 | Honeycomb-style wide-event SLI shape support (we have ratio/threshold only) | axis-observability | M03 |
| 4 | Mature mobile-app on-call (we have web Grafana OnCall) | ops-sre-reliability | M03 |
| 5 | AI-assisted anomaly detection (Datadog Watchdog / Honeycomb BubbleUp peers) | axis-observability | M04 |

## Key oyatie Differentiators (NOT in any competitor)

1. **Gate-integrated SLO**: verdict ↔ promotion is a first-class invariant; no competitor enforces this.
2. **Per-component release pointers**: independent `release/<ms>/<env>` ref per µservice; competitors deploy at coarser granularity.
3. **Cryptographic audit-chain over promotion**: Ed25519 + Merkle seals on every verdict + promotion + rollback event.
4. **OpenSLO native + GitOps-first**: schema validated at PR time + hot-reloaded by engine + tenant-readable as plain YAML files in git.
5. **Multi-pack residency by design**: 11 region-pinned packs with explicit cross-pack forbidden + SCC exception path.

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "SLO-gated promotion is unique to oyatie among production-deployed solutions" (true as of 2026-05-17; review bi-annually).
- ✅ "Multi-pack residency exceeds Datadog's region offering" (Datadog has 5 regions; oyatie has 11 active+conditional).
- ✅ "PromQL + LogQL + TraceQL all OSS; no proprietary query language" (Datadog + New Relic use proprietary; competitive advantage).

Sales claims FORBIDDEN (per ADR-0123 hyperscaler-maturity-claim-gate):
- ❌ "oyatie is faster than Datadog on burn-rate detection" (no published benchmark; would be unsourced superiority).
- ❌ "oyatie is HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation; do not claim universal).
- ❌ "We beat Grafana Cloud on cost" (depends on workload shape; do not claim universal).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes (new features / pricing / claims) | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-observability |
| 3. Re-run quantitative benchmarks (load tests in staging cluster) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary rule updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `microservices/observability/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-OBS gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0139 (agentic SLO-gated promotion).
- Competitor docs as cited inline above.
