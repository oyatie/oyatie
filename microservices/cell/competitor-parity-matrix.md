---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cell-substrate + council-architecture
deciders: axis-cell-substrate, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0130]
related_artifacts:
  - microservices/cell/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-CELL gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (cell µservice)

## Purpose

Quantitative + qualitative parity comparison vs the industry-leading tenant-cell / multi-tenant-Kubernetes products. Drives the `oya-foundry-fitness-hyperscaler-maturity-claims` gate (per ADR-0123 HG-CELL). Re-validated bi-annually.

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Kubernetes Cluster API | Cluster CRDs + Machine + MachineSet | Declarative cluster lifecycle; open standard | `cluster-api.sigs.k8s.io` |
| GKE Autopilot | Fully-managed K8s with opinionated multi-tenancy | Tenant abstraction; node-pool autoscaling | `cloud.google.com/kubernetes-engine/docs/concepts/autopilot-overview` |
| AWS EKS Fargate | Serverless pod-as-cell model | Pod isolation; no node mgmt | `aws.amazon.com/fargate/` |
| AWS App Runner | Managed app runtime | Tenant pinning; managed scaling | `aws.amazon.com/apprunner/` |
| OCI OKE | OCI K8s service | OCI-native cluster lifecycle | `oracle.com/cloud/cloud-native/container-engine-kubernetes/` |
| Crossplane | Cloud-resource composition | CRDs for cross-cloud resources | `crossplane.io` |
| Capsule (Kubernetes Multi-Tenancy SIG) | Tenant abstraction over namespaces | Namespace-as-tenant; quotas + policies | `capsule.clastix.io` |
| Karmada | Multi-cluster orchestration | Cross-cluster scheduling | `karmada.io` |

## Feature Parity Matrix

### Cell + Tenant Lifecycle

| Capability | oyatie | Cluster API | GKE Autopilot | EKS Fargate | App Runner | OKE | Crossplane | Capsule | Karmada |
|---|---|---|---|---|---|---|---|---|---|
| Declarative cell CRDs | ✅ | ✅ | ❌ (proprietary) | ❌ (proprietary) | ❌ | ✅ | ✅ | partial | partial |
| Per-tenant namespace + Postgres + S3 triple | ✅ | manual | ❌ | partial | ❌ | manual | manual | partial | manual |
| Tenant → cell binding (Ontology entity) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | partial | ❌ |
| Cell-affinity scheduling (binpack per tenant) | ✅ | ❌ | proprietary | proprietary | proprietary | ❌ | ❌ | ❌ | partial |
| Tenant migration ≤ 10 min p99 | ✅ | manual | ❌ | ❌ | ❌ | manual | manual | ❌ | ❌ |
| Cell decommission soft-delete (≥30d) | ✅ | manual | ❌ | n/a | n/a | manual | manual | manual | manual |
| Warm host pool per pack | ✅ | manual | proprietary | proprietary | proprietary | partial | manual | ❌ | ❌ |
| Per-cell SPIFFE SVID | ✅ | manual | partial (WorkloadIdentity) | partial | partial | manual | manual | partial | partial |

### Tenant Isolation

| Capability | oyatie | Cluster API | GKE | EKS | App Runner | OKE | Capsule |
|---|---|---|---|---|---|---|---|
| Hard tenant isolation (cell-namespace + Postgres schema + S3 prefix) | ✅ | manual | partial | partial | partial | manual | partial |
| Postgres row-level-security cell-scope | ✅ | manual | manual | manual | manual | manual | partial |
| Cedar policy boundary enforcement | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Cell-boundary CI lane (PR-time refusal) | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Audit-chain Ed25519 on every cell event | ✅ | ❌ | partial (Cloud Audit) | partial (CloudTrail) | partial | partial | ❌ |
| Cross-pack residency forbidden by default | ✅ | manual | partial | manual | manual | manual | manual |

### Cross-pack + multi-region

| Capability | oyatie | Cluster API | GKE | EKS | App Runner | OKE | Karmada |
|---|---|---|---|---|---|---|---|
| 11-pack region pinning | ✅ | manual | ✅ regions | ✅ regions | ✅ regions | ✅ regions | partial |
| DR pair intra-pack | ✅ | manual | proprietary | proprietary | proprietary | manual | partial |
| Cross-pack forbidden by default | ✅ | n/a | ❌ (cross-region allowed) | ❌ | ❌ | n/a | ❌ |
| SCC-exception path for cross-pack | ✅ | n/a | n/a | n/a | n/a | n/a | n/a |
| HIPAA-dedicated pack | ✅ | n/a | ✅ | ✅ | ✅ | ✅ | n/a |

### Operations + integrations

| Capability | oyatie | Cluster API | GKE | EKS | OKE | Capsule |
|---|---|---|---|---|---|---|
| Tenant migration as first-class capability | ✅ | ❌ | proprietary | ❌ | ❌ | ❌ |
| Live migration ≤ 10min p99 | ✅ | n/a | proprietary | ❌ | ❌ | ❌ |
| Multi-language SDK | M01: Rust; M02: TS/Py/Go; M03: JVM | n/a | ✅ | ✅ | ✅ | partial |
| GitOps cell lifecycle | ✅ | ✅ | partial | partial | partial | ✅ |
| Per-cell observability SLO labels | ✅ | manual | partial | partial | manual | partial |

## Quantitative Performance Parity

(All numbers reference 30-day rolling-window evaluations on equivalent workloads where competitor data available; otherwise marked "n/a — no published benchmark".)

| Metric | oyatie target | GKE Autopilot ref | EKS Fargate ref | OKE ref | Notes |
|---|---|---|---|---|---|
| Cell-assignment lookup p99 | ≤ 50 ms | n/a (different model) | n/a | n/a | oyatie unique |
| Cell-create end-to-end (warm hit) | ≤ 5 min p99 | ≤ 3 min (node provisioned) | ≤ 60 s (pod) | ≤ 5 min | parity with GKE |
| Cell-create end-to-end (cold) | ≤ 15 min p99 | ≤ 5 min | ≤ 60 s (pod) | ≤ 15 min | EKS Fargate faster for ephemeral pod model |
| Tenant migration p99 | ≤ 10 min | n/a | ❌ | n/a | oyatie unique |
| Scheduler placement decision p99 | ≤ 500 ms | proprietary | proprietary | n/a | parity |

## Key Parity Gaps to Close

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | Cross-pack tenant rehome with operator-friendly UX (SCC + 2-person rule has rough edges) | axis-cell-substrate | M02 |
| 2 | Multi-language SDK breadth (Py / Go / JVM) | axis-cell-substrate | M02–M03 |
| 3 | Cell auto-tiering (move tenant between cell_scopes shared → dedicated automatically based on usage) | axis-cell-substrate | M03 |
| 4 | Cross-cluster federation (Karmada-style) for very-large-tenant cells | axis-cell-substrate + cloud-k8s | M04 |

## Key oyatie Differentiators

1. **Cell-boundary as first-class invariant**: Postgres RLS + Cedar + LEAN lane + per-cell SVID enforce boundaries at four layers; no competitor enforces this combination.
2. **Tenant migration as standard capability**: ≤ 10 min p99 cutover with checkpoints; GKE/EKS treat migration as ad-hoc.
3. **Cross-pack residency forbidden by default**: explicit SCC + 2-person rule for any exception; no competitor enforces this strictness.
4. **Cell-boundary lane at PR time**: catches cross-cell coupling before merge.
5. **Cryptographic audit-chain over cell lifecycle**: Ed25519 seals on every state transition.
6. **Per-pack regulatory overlays**: cell substrate is regulation-aware (KR PIPA + HIPAA + GDPR + …) by design.

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "Cell-boundary CI lane (PR-time refusal) is unique to oyatie among production-deployed solutions" (true as of 2026-05-17; review bi-annually).
- ✅ "Tenant migration ≤ 10 min p99 as a standard capability" (no competitor publishes this; oyatie's distinction).
- ✅ "11-pack region pinning with cross-pack-forbidden default" (Cluster API doesn't pin; hyperscalers allow cross-region by default).

Sales claims FORBIDDEN (per ADR-0123 hyperscaler-maturity-claim-gate):
- ❌ "Cell substrate is faster than GKE Autopilot" (no published apples-to-apples benchmark; would be unsourced).
- ❌ "HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation; do not claim universal).
- ❌ "We beat Cluster API on operational ergonomics" (depends on workflow; do not claim universal).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-cell-substrate |
| 3. Re-run quantitative benchmarks (load tests) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `microservices/cell/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-CELL gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- Bominal ADR-0009; ADR-0019.
- Competitor docs as cited inline above.
- Kubernetes Multi-Tenancy SIG comparison — `github.com/kubernetes-sigs/multi-tenancy`.
