---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-workflow + council-architecture
deciders: axis-workflow, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0131]
related_artifacts:
  - microservices/workflow-engine/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-WF-ENGINE gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (workflow-engine µservice)

## Purpose

Quantitative + qualitative parity comparison vs the industry-leading durable-execution + workflow-engine products. Drives the `oya-foundry-fitness-hyperscaler-maturity-claims` gate (per ADR-0123 HG-WF-ENGINE) and tells gtm-customer-success what to say + what NOT to say in tenant sales conversations.

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Temporal | Temporal Cloud + Temporal SDK (open-source) | Mature durable execution; deterministic replay; multi-language SDKs; multi-week run lifetimes | `docs.temporal.io` |
| Cadence | Uber Cadence | Temporal's predecessor; same semantics; older API surface | `cadenceworkflow.io` |
| Apache Airflow | Airflow 2.x | Open-source DAG scheduler; sensor operators; backfill; widely adopted in data engineering | `airflow.apache.org/docs` |
| Camunda | Camunda Platform 8 (Zeebe engine) | BPMN 2.0 engine; broker-based; cluster-shardable; enterprise workflow focus | `docs.camunda.io` |
| Argo Workflows | Argo Workflows on Kubernetes | Container-native step execution; CRD-driven; DAG + step templates; resource-bounded | `argoproj.github.io/argo-workflows` |
| n8n (engine layer) | n8n engine (separated from Studio editor) | Sub-second event-to-action; node retry; webhook trigger; tight coupling to its own Studio | `docs.n8n.io` |
| AWS Step Functions | AWS Step Functions | Managed state machine; Amazon States Language (ASL) DSL; tight AWS service integration | `docs.aws.amazon.com/step-functions/` |
| Dapr Workflows | Dapr Workflow building block | Actor-based durable execution; multi-language SDK | `docs.dapr.io/developing-applications/building-blocks/workflow` |
| Restate | Restate Server | Modern durable execution; SDK-first; embedded state machine | `docs.restate.dev` |

(Note: this matrix is engine-substrate only. Visual editor competitors — n8n Studio, Zapier, Make, Workato — are in `microservices/workflow-studio/competitor-parity-matrix.md`.)

## Feature Parity Matrix

### Durable execution

| Capability | oyatie | Temporal | Cadence | Airflow | Camunda 8 | Argo WF | n8n | Step Fn | Dapr | Restate |
|---|---|---|---|---|---|---|---|---|---|---|
| Deterministic replay (engine restart resumes run identically) | ✅ | ✅ | ✅ | partial (DAG state only) | ✅ | partial | ❌ (best-effort) | ✅ | ✅ | ✅ |
| Multi-week run lifetimes (paused-in-place) | ✅ 90d | ✅ unbounded | ✅ unbounded | days only | ✅ weeks | ✅ weeks | hours typical | weeks | weeks | weeks |
| Crash-safe step persistence (no duplicated effects on resume) | ✅ | ✅ | ✅ | partial (task-level) | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ |
| Event-sourced state model | ✅ | ✅ | ✅ | ❌ (DAG-state-only) | ✅ | partial | ❌ | partial | ✅ | ✅ |
| Side-effect ledger / activity recording | ✅ | ✅ | ✅ | ❌ | ✅ | partial | ❌ | ✅ | partial | ✅ |
| Workflow signals (async input mid-run) | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | partial (webhook) | partial | ✅ | ✅ |

### Performance + scalability

| Capability | oyatie | Temporal | Camunda 8 | n8n | Step Fn |
|---|---|---|---|---|---|
| Event-to-action latency p99 | ≤ 500ms | ≤ 500ms | ≤ 1s | ≤ 1s | ≤ 2s |
| Step execution start p99 | ≤ 200ms | ≤ 100ms | ≤ 500ms | ≤ 1s | ≤ 1s |
| Concurrent active runs per cell | 10,000 | unbounded (cluster) | 100k+ | unbounded | unbounded (regional) |
| Per-tenant sharding | ✅ Citus | partial (namespace) | ✅ partition | ❌ | partial (account) |
| Per-tenant rate limits + active-run caps | ✅ | partial | ✅ | ❌ | ✅ |

### Substrate + multi-tenancy

| Capability | oyatie | Temporal | Camunda 8 | n8n | Step Fn | Restate |
|---|---|---|---|---|---|---|
| Self-hosted (no vendor lock) | ✅ | partial (Cloud is SaaS; OSS Server available) | ✅ + Cloud | ✅ | ❌ (SaaS only) | ✅ |
| Multi-region data-residency | ✅ 11 packs | regional namespaces | partial (cluster per region) | self-hosted-only | ✅ AWS regions | partial |
| HIPAA BAA | conditional pack-us-hc | ✅ Cloud | partial | DIY | ✅ | partial |
| KR PIPA compliance | conditional pack-kr | partial | partial | DIY | partial | ❌ |
| EU GDPR DPA | ✅ | ✅ | ✅ | DIY | ✅ | partial |
| Per-tenant audit-chain Ed25519 seals | ✅ | ❌ (logs only) | partial | ❌ | partial (CloudTrail) | ❌ |

### Engineering ecosystem

| Capability | oyatie | Temporal | Cadence | Airflow | Camunda 8 | Argo WF | Restate |
|---|---|---|---|---|---|---|---|
| Multi-language SDK | M02b: Rust; M03: TS/Py/Go; M04: JVM | ✅ broad (Go, Java, TS, Python, .NET, PHP) | ✅ (Go, Java, Python) | ❌ (Python only) | ✅ (Java, Node) | ❌ (YAML/CRD only) | ✅ (TS, Java, Python) |
| gRPC + REST API | ✅ both | ✅ both | ✅ gRPC | partial | ✅ both | ❌ (CRD) | ✅ both |
| Plugin substrate (WASM) | ✅ ADR-0037 | ❌ | ❌ | partial | partial | ❌ | partial |
| Workflow event registry (typed) | ✅ | partial | partial | ❌ | ✅ | ❌ | partial |
| Cross-product orchestration adapter (this engine = the bus) | ✅ unique | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

## Quantitative Performance Parity

(All numbers reference 30-day rolling-window evaluations on equivalent workloads.)

| Metric | oyatie target | Temporal reference | Camunda 8 reference | Notes |
|---|---|---|---|---|
| Step execution latency p99 | ≤ 200ms | ≤ 100ms | ≤ 500ms | Temporal advantage (mature optimizer); parity within 100ms is acceptable for M02b |
| Event-to-action latency p99 | ≤ 500ms | ≤ 500ms | ≤ 1s | parity |
| Replay throughput p99 | ≥ 1000 steps/s/worker | ≥ 2000 steps/s/worker | n/a (BPMN model is different) | Temporal advantage; gap-closure target M03 |
| Concurrent active runs per cell | 10,000 | 100,000+ | 100,000+ | gap-closure target M03 via Citus shard addition |
| Audit chain seal latency p99 | ≤ 1s | n/a (no equivalent) | n/a | oyatie unique |
| 30-day retention storage cost per 100k runs (estimated) | ~$50/mo cold-tier @ OCI | Temporal Cloud: ~$300+ | Camunda Cloud: ~$200+ | oyatie advantage via self-host |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | Multi-language SDK breadth (TS, Py, Go, JVM) | axis-workflow | M03 |
| 2 | Step execution latency parity with Temporal | axis-workflow | M03 (engine optimizer tuning) |
| 3 | Replay throughput parity (1000 → 2000 steps/s/worker) | axis-workflow | M03 |
| 4 | Workflow versioning patterns (replay against old spec, run against new spec) | council-architecture | M03 ADR |
| 5 | Workflow saga / compensation pattern primitives (built-in) | axis-workflow | M04 |
| 6 | Visual debugger UX parity with Camunda Cockpit (Studio responsibility, but engine ships backend) | axis-workflow | M03 (backend); Studio M04 |

## Key oyatie Differentiators (NOT in any competitor)

1. **Cross-product orchestration adapter**: the engine IS the cross-µservice event bus by design; direct product-to-product calls forbidden (LEAN-A2). No competitor positions the engine this way.
2. **Per-tenant audit-chain Ed25519 seals**: every run + transition cryptographically sealed; Merkle-verified. Competitors offer logs, not cryptographic audit.
3. **Multi-pack residency by design**: 11 region-pinned packs with explicit cross-pack forbidden + SCC exception path; exceeds Temporal Cloud's region offering (5).
4. **SLO-gated promotion of workflow-engine itself**: engine SLO must be green before workflow-engine deployment advances; recursive observability gate (this µservice gated by `observability/PHASE-01`).
5. **WASM plugin substrate**: tenant-authored custom step types as sandboxed WASM modules (ADR-0037); only Camunda has a partial equivalent (Java extension points).

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "workflow-engine is the cross-µservice orchestration adapter; all events route through it" (true and unique).
- ✅ "Per-tenant audit-chain Ed25519 seals exceed Temporal's log-based audit" (true).
- ✅ "Multi-pack residency (11 packs) exceeds Temporal Cloud's regional offering" (Temporal Cloud has ~5 regions).
- ✅ "Deterministic replay invariant verified per release via CI lane" (true; PRD AC-02).

Sales claims FORBIDDEN (per ADR-0123 hyperscaler-maturity-claim-gate):
- ❌ "oyatie is faster than Temporal" (no published benchmark; Temporal is ahead today on step latency).
- ❌ "oyatie scales to unlimited runs per cell" (current target is 10k; gap-closure target 100k+ at M03; do not claim unlimited).
- ❌ "We beat Camunda 8 on cost" (depends on workload shape; do not claim universal).
- ❌ "Our SDK breadth matches Temporal" (current breadth is Rust + TS; Temporal has 6+ languages; M03 narrows but not closes).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes (new features / pricing / claims) | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-workflow |
| 3. Re-run quantitative benchmarks (load tests in staging cluster) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary rule updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `microservices/workflow-engine/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-WF-ENGINE gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0131 (per-microservice flat layout + workflow unbundle).
- Competitor docs cited inline.
- Temporal vs Cadence migration guide — `docs.temporal.io/dev-guide/migrate-from-cadence`.
- Camunda 8 architecture — `docs.camunda.io/docs/components/concepts/architecture-cluster/`.
