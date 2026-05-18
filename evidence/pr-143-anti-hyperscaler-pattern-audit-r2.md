# PR #143 — anti-hyperscaler pattern audit, round 2 (Fix-L)

**Audit date**: 2026-05-18
**Auditor**: Fix-L (continued anti-hyperscaler pattern hunting)
**Source PR**: #143 (oya-microservice-flat-layout-buildout-2026-05-17)
**Scope**: 30 candidate patterns NOT in the Tier A/B/C briefs (Fix-I, Fix-J, Fix-K queues).
**Goal**: Find genuinely-missed anti-hyperscaler patterns and either (a) cite the ADR that already addresses them or (b) author a new ADR.

## Classification legend

- **HANDLED** — an existing ADR or shipped artifact already addresses the pattern.
- **PARTIAL** — addressed in part; identified gap noted; routed to existing ADR follow-up debt.
- **MISSING** — no ADR or artifact; new ADR authored OR concretely deferred to a tier brief.
- **TIER-A** — immediate / new ADR authored here.
- **TIER-B** — strategic; ADR queued in placeholder-debt.
- **TIER-C** — nice-to-have; documented intent only.

## Summary

| # | Pattern | Status | Action |
|---|---|---|---|
| 1 | Connection pooling discipline (pgbouncer / RDS-Proxy / pgcat) | **MISSING — TIER-A** | **ADR-0179 authored** |
| 2 | Hot/cold storage tiering (beyond recordings) | PARTIAL | Routed to placeholder-debt; per-µservice manifest extension |
| 3 | Quorum reads/writes vs strict-serializable per data class | HANDLED | ADR-0158 multi-region active-active + ADR-0028 |
| 4 | Per-tenant BYOK encryption keys | HANDLED | ADR-0043 OpenBao + HSM per cell + ADR-0045 (DB-tier KMS hook) |
| 5 | Right-to-be-forgotten (GDPR Art. 17) DSR cascade | HANDLED | ADR-0038 trust framework + DSR cascade + proof-of-erasure |
| 6 | Mutating + validating admission webhooks | **MISSING — TIER-B** | Queued in placeholder-debt; governance µservice owns |
| 7 | Pod priority + preemption + over-commit | **MISSING — TIER-B** | Queued; cloud-k8s µservice owns; aligns with Tier A13 |
| 8 | Backup encryption + per-tenant key | PARTIAL | ADR-0043 (key material) + needs backup-µservice ADR (in Tier B queue) |
| 9 | Cluster-level egress allowlist (Cilium ClusterMesh) | HANDLED | ADR-0148-Cilium (multi-cluster ClusterMesh) + ADR-0158 |
| 10 | Container scanning in CI (Trivy / Snyk / Grype) | HANDLED | ADR-0039 supply-chain-security (trivy + cosign + sbom + signed-commits) |
| 11 | License compliance — per-dependency tracking | HANDLED | ADR-0013 product-license-policy + cargo-deny via ADR-0039 |
| 12 | gRPC streaming with flow control | PARTIAL | Contracts allow but no canonical rubric; routed to placeholder-debt |
| 13 | Provisioned concurrency for ML inference cold-start | **MISSING — TIER-B** | Foundry runtime concern; queued; aligns with ADR-0136/0139 |
| 14 | Read-through cache pattern (cache-aside Redis) | PARTIAL | ADR-0002 mentions cache-aside; no canonical Redis-substrate ADR |
| 15 | LWW vs CRDT vs strict-serializable per data class | HANDLED | ADR-0142 CRDT portability trait + ADR-0158 |
| 16 | Multi-tenant sharding key (Citus pgvector) | HANDLED | ADR-0009 cell architecture + ADR-0045 database-tier strategy |
| 17 | Snapshot isolation vs serializable per data class | PARTIAL | ADR-0028 mentions; no per-BC declaration discipline |
| 18 | Idempotent retry with deduplication window | HANDLED | ADR-0169 webhook-dlq-retry + ADR-0153 outbox-pattern; covered by Tier A1 |
| 19 | Two-phase-commit avoidance (saga enforced) | HANDLED | ADR-0145 + workflow-engine ADR-0035 saga pattern |
| 20 | Latency budget propagation across hops | HANDLED | ADR-0093 latency-budget-reporter + ADR-0158 |
| 21 | Service-level objective inheritance (child SLO → parent SLO) | **MISSING — TIER-A** | **ADR-0180 authored** |
| 22 | Auto-rollback on SLO regression | PARTIAL | ADR-0139 + ADR-0160 progressive-delivery-flagger; concrete impl debt remains |
| 23 | Tracing sampling discipline (head vs tail) | **MISSING — TIER-B** | Queued; observability µservice owns |
| 24 | Log retention tiering (hot / warm / cold) | **MISSING — TIER-B** | Queued; observability µservice owns |
| 25 | Per-µservice cargo workspace ownership | HANDLED | ADR-0131 per-microservice flat layout + CODEOWNERS |
| 26 | Cargo-deny + supply-chain policy | HANDLED | ADR-0039 supply-chain-security |
| 27 | Per-µservice rust-toolchain pin | HANDLED-AT-WORKSPACE | Workspace-level pin sufficient per ADR-0131; per-µservice override not required |
| 28 | Secrets scanning in CI (gitleaks / trufflehog) | HANDLED | ADR-0121 §security posture (gitleaks systemd timer) + ADR-0039 |
| 29 | Container image promotion pipeline (dev → staging → prod) | **MISSING — TIER-A** | **ADR-0181 authored** |
| 30 | Blue-green DNS swap | HANDLED | ADR-0040 progressive-delivery + ADR-0160 flagger + ADR-0158 multi-region |

**Tally:** HANDLED = 15 · PARTIAL = 6 · MISSING = 9 · NEW ADRs authored = 3 (ADR-0179, ADR-0180, ADR-0181).

## Detailed findings

### 1. Connection pooling discipline — **MISSING → ADR-0179**

- **Hyperscaler precedent.** AWS RDS Proxy (Aurora / RDS Postgres in front of every fleet service); Stripe internal pgbouncer-fork "pgcat-class" pooler; Linear pgbouncer transaction mode at the µservice tier; Notion uses pgcat.
- **Why it matters.** Postgres connection limits become the platform SLO ceiling under burst (each connection ≈ 5-10MB RAM in the backend; max_connections ≈ 200-500). Without a pooler, every µservice replica × pod consumes a backend slot; at 33 µservices × N pods this exhausts the DB before workload SLO bites.
- **Current oyatie state.** ADR-0045 database-tier-strategy mentions connection management but does not pin a pooler choice or topology. No per-µservice `db_connection_pool_max` manifest field. `microservices/<ms>/iac/helm/<ms>/templates/deployment.yaml` does not declare a pooler sidecar.
- **Adoption.** ADR-0179 authored: pgcat (Rust pgbouncer-class) as canonical; per-µservice `manifest.json#postgres.pool_max` declaration; transaction-mode pooling; per-µservice pooler sidecar OR per-cell pooler service (decided per workload).
- **Tier.** A (immediate).

### 2. Hot/cold storage tiering (beyond recordings) — PARTIAL

- **Hyperscaler precedent.** S3 Intelligent-Tiering; GCS Coldline; Azure Blob Cool tier. Per-object lifecycle policies based on access age.
- **Current oyatie state.** `recordings` µservice declares hot/cold (per ADR-0146 distroless + recordings-specific PRD). Drive (file storage) and audit-chain (append-only Merkle leaves) do not declare tiering.
- **Adoption.** Routed to `registry/placeholder-debt/adr-follow-ups.yaml` as `per-microservice-storage-tiering-manifest`. Schema extension to `manifest.json#storage.tiering` for any µservice with object-store ownership.
- **Tier.** B (strategic; not blocking PR #143).

### 3. Quorum reads/writes vs strict-serializable per data class — HANDLED

- **Hyperscaler precedent.** Spanner TrueTime serializable; DynamoDB tunable consistency; Cassandra QUORUM.
- **Current state.** ADR-0158 multi-region active-active declares per-data-class consistency rubric (linearizable for billing, eventual for chat presence). ADR-0028 cloud-microservice-architecture pins isolation levels.
- **Status.** HANDLED.

### 4. Per-tenant BYOK encryption — HANDLED

- **Hyperscaler precedent.** AWS KMS BYOK / Imported Key Material; GCP Cloud KMS EKM; Azure Key Vault Managed HSM.
- **Current state.** ADR-0043 OpenBao + per-cell HSM declares per-tenant key isolation and BYOK hook. ADR-0045 database tier strategy declares envelope encryption at the DB tier.
- **Status.** HANDLED.

### 5. Right-to-be-forgotten DSR cascade — HANDLED

- **Hyperscaler precedent.** GDPR Art. 17; CCPA §1798.105.
- **Current state.** ADR-0038 trust framework + DSR cascade + proof-of-erasure is the canonical pattern. Workflow µservice provides the saga; every entity-owning µservice integrates the erasure proof.
- **Status.** HANDLED.

### 6. Mutating + validating admission webhooks — **MISSING (TIER-B)**

- **Hyperscaler precedent.** Kyverno (already adopted per ADR-0117 §Kyverno consolidation); OPA Gatekeeper.
- **Current state.** ADR-0117 consolidates Kyverno policy authoring; no per-µservice admission-webhook contract documenting which µservice owns which validation. Governance µservice has Cedar fragments but no mutating-webhook surface.
- **Adoption.** Queued in placeholder-debt as `governance-admission-webhook-contract`. Governance µservice owns the Kyverno ClusterPolicy catalog + a per-µservice mutating-webhook surface (e.g., default-deny pod security, default-add observability labels).
- **Tier.** B.

### 7. Pod priority + preemption + over-commit — **MISSING (TIER-B)**

- **Hyperscaler precedent.** Google Borg priority bands; Kubernetes PriorityClass + PreemptionPolicy.
- **Current state.** Tier-A13 brief mentions; no concrete artifact. No `iac/helm/<ms>/templates/priorityclass.yaml` shipped.
- **Adoption.** Queued in placeholder-debt as `pod-priority-preemption-canonical`. Three-band model (cluster-critical-1000, tenant-paid-100, tenant-free-10) with per-µservice manifest `pod_priority_class` declaration.
- **Tier.** B.

### 8. Backup encryption + per-tenant key — PARTIAL

- **Hyperscaler precedent.** AWS Backup with KMS-CMK per backup vault; GCP Cloud Backup with CMEK.
- **Current state.** ADR-0043 provides key material; ADR-0152 RPO/RTO declares backup cadence; no explicit declaration that backups use per-tenant key envelopes (vs per-cell key).
- **Adoption.** Queued in placeholder-debt as `backup-substrate-canonical` (Tier B).

### 9. Cluster-level egress allowlist — HANDLED

- **Current state.** ADR-0148-Cilium covers ClusterMesh multi-cluster topology; ADR-0158 active-active topology. Per-µservice CiliumNetworkPolicy already declared in the migration guide.
- **Status.** HANDLED.

### 10. Container scanning in CI — HANDLED

- **Current state.** ADR-0039 supply-chain-security canonicalizes trivy + cosign + sbom + signed-commits. ADR-0121 §security posture wires trivy + debsecan + cargo-audit on systemd timer.
- **Status.** HANDLED.

### 11. License compliance — HANDLED

- **Current state.** ADR-0013 product-license-policy declares the policy; ADR-0039 wires cargo-deny / sbom for per-dependency tracking.
- **Status.** HANDLED.

### 12. gRPC streaming with flow control — PARTIAL

- **Hyperscaler precedent.** Google internal Stubby streaming; gRPC HTTP/2 backpressure; Stripe Sigma streaming for query results.
- **Current state.** µservice contracts (proto IDL) may declare `stream` but no canonical rubric for when to use streaming vs unary RPC. No per-µservice flow-control declaration.
- **Adoption.** Queued in placeholder-debt as `grpc-streaming-rubric`. Tier C (nice-to-have).

### 13. Provisioned concurrency for ML inference cold-start — **MISSING (TIER-B)**

- **Hyperscaler precedent.** AWS Lambda Provisioned Concurrency; SageMaker Endpoint Auto-Scaling with warm instance floor.
- **Current state.** Foundry runtime (per ADR-0136 + ADR-0139) declares SLO-gated promotion but does not declare a warm-instance floor for Whisper / capability execution cold-start.
- **Adoption.** Queued as `foundry-provisioned-concurrency-floor` in placeholder-debt. Tier B.

### 14. Read-through cache pattern (cache-aside Redis) — PARTIAL

- **Hyperscaler precedent.** Facebook TAO; Twitter Cache Aside; Stripe Redis cache-aside for permission checks.
- **Current state.** ADR-0002 tenant-identity mentions cache-aside in passing. No canonical Redis-substrate ADR. ADR-0137 foundry-bounded-contexts mentions caching at a BC level.
- **Adoption.** Queued as `redis-cache-substrate-canonical` in placeholder-debt. Tier C.

### 15. LWW vs CRDT vs strict-serializable per data class — HANDLED

- **Current state.** ADR-0142 CRDT portability trait + ADR-0158 multi-region active-active.
- **Status.** HANDLED.

### 16. Multi-tenant sharding key — HANDLED

- **Current state.** ADR-0009 cell-architecture per-tenant-per-region + ADR-0045 database-tier-strategy + ADR-0030 search-microservice-architecture.
- **Status.** HANDLED.

### 17. Snapshot isolation vs serializable per data class — PARTIAL

- **Hyperscaler precedent.** Spanner SERIALIZABLE; Postgres `serializable` mode (default `read committed`); CockroachDB serializable.
- **Current state.** ADR-0028 mentions isolation levels; no per-BC declaration. Foundry's per-tenant-fleet isolation is documented; transactional µservices' isolation is not.
- **Adoption.** Queued as `per-bc-postgres-isolation-declaration` in placeholder-debt. Tier B.

### 18. Idempotent retry with deduplication window — HANDLED

- **Current state.** ADR-0169 webhook-dlq-retry declares deduplication window; ADR-0153 outbox-pattern provides idempotency. Tier-A1 brief (Idempotency-Key) covered.
- **Status.** HANDLED.

### 19. Two-phase-commit avoidance — HANDLED

- **Current state.** ADR-0145 saga pattern (workflow-engine); ADR-0035 workflow-engine-state-machine-and-dag-hybrid. No 2PC across µservices.
- **Status.** HANDLED.

### 20. Latency budget propagation across hops — HANDLED

- **Current state.** ADR-0093 latency-budget-reporter-rename; ADR-0158 declares cross-region budget; ADR-0157 api-gateway-tier sets ingress budget.
- **Status.** HANDLED.

### 21. SLO inheritance (child SLO → parent SLO) — **MISSING → ADR-0180**

- **Hyperscaler precedent.** Google SRE Workbook "Embedded SLO Hierarchy"; AWS Service Quotas hierarchical limits; Datadog SLO composition.
- **Why it matters.** Without SLO inheritance, a single µservice's SLO drift silently degrades a parent product's SLO. The composition needs to be declared so the gate can detect impossible aggregates (e.g., parent claims 99.95%, child claims 99.5% — the parent is unreachable).
- **Current oyatie state.** ADR-0139 agentic-SLO-gated-promotion declares per-µservice SLOs but does not declare composition arithmetic.
- **Adoption.** ADR-0180 authored: SLO composition arithmetic rules; `slos/composition.openslo.yaml` per parent product; oya-dev-cli gate validates composition feasibility.
- **Tier.** A.

### 22. Auto-rollback on SLO regression — PARTIAL

- **Current state.** ADR-0139 principles; ADR-0160 progressive-delivery-flagger wires Flagger which natively supports SLO-driven rollback. Concrete per-µservice rollback runbooks are debt.
- **Adoption.** Queued in placeholder-debt as `per-microservice-auto-rollback-runbook`. Tier B.

### 23. Tracing sampling discipline — **MISSING (TIER-B)**

- **Hyperscaler precedent.** Honeycomb tail-based sampling; Google Dapper head-based 1/N + adaptive; Datadog APM hybrid.
- **Current state.** ADR-0042 observability mentions OTel; no sampling rule. Cilium Hubble + OTel exporter default sampling not declared.
- **Adoption.** Queued as `otel-sampling-canonical` in placeholder-debt. Tier B.

### 24. Log retention tiering — **MISSING (TIER-B)**

- **Hyperscaler precedent.** CloudWatch Logs tiers (Standard / IA); Loki retention buckets; Datadog log archive tier.
- **Current state.** Observability µservice exists; per-µservice log retention budget not declared. No hot/warm/cold log tiering policy.
- **Adoption.** Queued as `log-retention-tiering-canonical` in placeholder-debt. Tier B.

### 25. Per-µservice cargo workspace ownership — HANDLED

- **Current state.** ADR-0131 per-microservice flat layout + CODEOWNERS files.
- **Status.** HANDLED.

### 26. Cargo-deny + supply-chain policy — HANDLED

- **Current state.** ADR-0039 supply-chain-security.
- **Status.** HANDLED.

### 27. Per-µservice rust-toolchain pin — HANDLED-AT-WORKSPACE

- **Current state.** Workspace-level `rust-toolchain.toml` is the canonical pin (per ADR-0131). Per-µservice overrides not required; would fragment the toolchain matrix.
- **Status.** HANDLED-AT-WORKSPACE (no per-µservice ADR needed).

### 28. Secrets scanning in CI — HANDLED

- **Current state.** ADR-0121 §security posture wires gitleaks + trivy on systemd timer; ADR-0039 covers CI-side. ADR-0043 OpenBao enforces "no raw secrets" discipline.
- **Status.** HANDLED.

### 29. Container image promotion pipeline (dev → staging → prod) — **MISSING → ADR-0181**

- **Hyperscaler precedent.** AWS ECR cross-account image promotion via Cosign signatures; GCP Artifact Registry promotion ladder; Google Borg promotion graph; Stripe-internal canary-image promotion via signed-tag swap.
- **Why it matters.** Without an explicit promotion pipeline, "dev" container images can ship to "prod" via human error or rogue automation. The promotion path needs Cosign signature on each tier and policy that production only pulls promoted-images.
- **Current oyatie state.** ADR-0039 supply-chain-security + ADR-0041 gitops-trunk-based + ADR-0146 distroless-nonroot all touch the surface but do not declare the dev → staging → prod promotion ladder. ADR-0124 own-merge-queue handles code promotion; image promotion is undeclared.
- **Adoption.** ADR-0181 authored: dev → staging → prod image promotion ladder with cosign signature per tier; cluster pull policy restricts each environment to its tier's signed images.
- **Tier.** A.

### 30. Blue-green DNS swap — HANDLED

- **Current state.** ADR-0040 progressive-delivery-canary-blue-green-metric-gated-rollback + ADR-0160 flagger + ADR-0158 multi-region active-active.
- **Status.** HANDLED.

## New ADRs authored in this round

| ADR | Title | Tier |
|---|---|---|
| ADR-0179 | Postgres connection-pooling canonical (pgcat) | A |
| ADR-0180 | SLO composition + inheritance arithmetic | A |
| ADR-0181 | Container image promotion pipeline (dev → staging → prod with cosign) | A |

## Placeholder-debt additions queued

(Routed to `registry/placeholder-debt/adr-follow-ups.yaml` for future authorship — Tier B / C):

1. `per-microservice-storage-tiering-manifest` (B)
2. `governance-admission-webhook-contract` (B)
3. `pod-priority-preemption-canonical` (B)
4. `backup-substrate-canonical` (B)
5. `grpc-streaming-rubric` (C)
6. `foundry-provisioned-concurrency-floor` (B)
7. `redis-cache-substrate-canonical` (C)
8. `per-bc-postgres-isolation-declaration` (B)
9. `per-microservice-auto-rollback-runbook` (B)
10. `otel-sampling-canonical` (B)
11. `log-retention-tiering-canonical` (B)

## References

- ADR-0148-service-mesh-cilium.md (this round's ADR-0148 override).
- evidence/pr-143-review-r2-idea-refine-doubt-driven.json (parent review evidence).
- specs/hyperscaler-architecture-invariants.json.
- ADR-0128 hyperscaler-architecture-invariants.
- Google SRE Workbook (SLO composition references).
- AWS Builders Library (shuffle-sharding, isolation references).
- Stripe + Linear + Notion public engineering blog (connection pool, idempotency references).
