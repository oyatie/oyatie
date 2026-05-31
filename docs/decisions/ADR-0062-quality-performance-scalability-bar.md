---
id: ADR-0062
status: Accepted
doc_status: published
---

# ADR-0062: Quality/Performance/Scalability bar — industry leaders + hyperscaler scale, mandatory from day one

> **Status:** Accepted
> **Owner:** `council-architecture`
> **Date:** 2026-05-13
> **Related:** ADR-0001, ADR-0009, ADR-0056, ADR-0058, ADR-0059, ADR-0060

---

## Context

Oyatie ships a complete product, not a prototype. The quality bar is set by industry leaders (competitive-benchmarked) and hyperscalers (100M+ user scale). Horizontal scalability is mandatory from day one. No single-instance-only designs. No "prototype-quality" first releases — feature-complete or not shipped.

User instruction 2026-05-13: "Our quality bar is industry leaders (with existing research benchmarked against competitors) and hyperscalers. Our quality and performance bar is high and must be horizontally scalable."

This aligns with Bominal master-plan: "Complete product, not prototype", "Modern architecture", "100M+ users from day one."

**Naming justification:** "Quality/Performance/Scalability" are the three orthogonal dimensions of the bar. Each has concrete targets and CI enforcement per this ADR.

---

## Decision

### Quality bar — Industry Leaders

Every µservice must benchmark against the industry leader for its domain before graduating from Proof-Ladder L4 → L5:

| Dimension | Reference standard |
|---|---|
| API design | Stripe (REST/gRPC contracts, idempotency, pagination, error model) |
| Data layer | Palantir Ontology (typed entities + provenance + audit) |
| UI/UX craft | Linear / Stripe / Superhuman (flat dual-mode surfaces) |
| Operational telemetry | Palantir Foundry-grade observability + on-call runbooks |
| Auth + identity | Auth0 / Okta capability parity + own-rails (Bominal ADR-0123 inherited) |
| Eventing | Confluent Kafka (KRaft) + Apache schema registry parity |
| Search | OpenSearch / Algolia parity; oyatie uses pgroonga + Tantivy |
| Document gen | Typst (best-of-class) |

**Every µservice PRD MUST include a `## Competitive Benchmark` section** naming the industry leader(s) the µservice targets parity with, listing the specific features/quality dimensions benchmarked, and citing primary-source research.

### Performance bar — Hyperscaler targets

| Dimension | Target | Source |
|---|---|---|
| API p99 latency (read-only Ontology Functions) | ≤50ms | Bominal ADR-0107 §"Threat model" |
| API p99 latency (Action Types / mutations) | ≤200ms | Bominal ADR-0107 |
| Throughput per cell | 10k+ req/sec baseline; 100k+ aggregate via cell sharding | `[[feedback-quality-performance-scalability-bar]]` |
| Event lag (outbox → consumer) | sub-second | Bominal ADR-0116 LISTEN/NOTIFY model |
| Audit chain segment-seal latency | <1s per (tenant, period) | Bominal ADR-0028 |
| Cell failover RTO | ≤30s | Bominal ADR-0009 cell architecture |
| Cell failover RPO | ≤5s | Bominal ADR-0009 + outbox + cross-region replication |
| Cold start (serverless) | ≤500ms | Bominal ADR-0020 multi-runtime |
| Tenant onboarding (self-serve SaaS) | ≤5min | Bominal ADR-0118 |

**Every µservice PRD MUST include a `## Performance Targets` section** listing concrete p50/p99/p999 latency targets, throughput targets, error-budget allocation, SLO burn-rate alarms. Targets must be testable via load tests (k6 / locust / vegeta).

**Every implementation plan MUST include a `## Load test` section** running the perf targets against the implementation before merging to main.

### Horizontal scalability bar — Mandatory from day one

| Requirement | Target | Enforcement |
|---|---|---|
| Stateless services | `application` + `rest` + `grpc` + `graphql` + `worker` layer crates have ZERO module-level mutable state | `oya-check-statelessness` CI lane |
| Sharded state | Postgres + Citus per Bominal ADR-0117 stage 2; ClickHouse + replicas; Valkey/Redis cluster. No single-DB-only designs | `oya-check-shardability` CI lane |
| Event-driven | Outbox → Kafka KRaft. Direct synchronous cross-service calls require ADR justification | `oya-check-architecture` (LEAN-A2) |
| Cell architecture | All tenant-bound state partitioned per (cell, region); per ADR-0009 + Bominal ADR-0009 | `oya-check-shardability` |
| Active-active capable | All `worker` + `adapter` layers declare `active_active_compatibility: stateless-compatible` OR `single-writer-compatible` in `[package.metadata.oya]` | CI verifies field presence |
| Cross-region replication | Required for high-consequence domains (medical, payments, connect-pro mail) | Per ADR-0049; `oya-check-shardability` |
| Auto-scale ready | Kubernetes HPA + VPA hooks; pre-warmed pools; no startup-blocking dependencies | Deployment manifests reviewed at PR |

**Every µservice PRD MUST include a `## Horizontal Scalability` section** declaring:
- State strategy (stateless / postgres / object-storage / persistent-volume / mixed)
- Active-active compatibility flag
- Per-cell capacity envelope (max QPS, max storage, max concurrent users)
- Scale-out trigger metrics + auto-scale policy
- Cross-region story (single-region launch acceptable if documented; M03 KR-only acceptable)

### CI enforcement — complete 14-lane matrix

The four new quality/perf/scale check crates plus the pre-existing LEAN architecture lanes form the full 14-lane enforcement matrix. Quality/scale lanes authored in M02 substrate phase; all start `--report-only`; flip to BLOCKER at M02 exit gate (LEAN-A3/A4 BLOCKER day-1).

**Four new check crates (this ADR):**

```
oya-check-statelessness    — verifies application/rest/grpc/graphql/worker layer crates
                             have no module-level mutable state (proc-macro scan)
oya-check-shardability     — verifies DB designs declare tenant_id partition key + RLS;
                             no unbounded single-DB designs
oya-check-perf-budget      — verifies impl plans include load-test results meeting
                             declared perf targets before merge
oya-check-benchmark        — verifies PRDs include competitive-benchmark section
                             before µservice graduates from Proof-Ladder L4 → L5
```

**Naming justification (BNF v4.1, ADR-0056):**
- `oya-check-statelessness`: check namespace (BNF-exempt); rule-name = `statelessness`
- `oya-check-shardability`: check namespace; rule-name = `shardability`
- `oya-check-perf-budget`: check namespace; rule-name = `perf-budget` (multi-token)
- `oya-check-benchmark`: check namespace; rule-name = `benchmark`

**Full 14-lane matrix (clean-arch + quality/perf/scale):**

| Lane | Enforces | BLOCKER from |
|---|---|---|
| `oya-shared-architecture-check-cli -- dependency-direction` | Inward-only 12-layer flow | M02 exit |
| `oya-shared-architecture-check-cli -- layer-correctness` | Declared layer matches code shape | M02 exit |
| `oya-shared-architecture-check-cli -- lib-name-parity` | `[lib] name` = snake_case(`[package] name`) | M02 exit |
| `oya-shared-architecture-check-cli -- port-location` | Port traits in `kernel`; impls in `adapter` | M02 exit |
| `oya-shared-architecture-check-cli -- cross-product-refusal` | No direct cross-microservice imports (LEAN-A2) | M02 exit |
| `oya-shared-architecture-check-cli -- composition-root-only` | Only `app` layer has unrestricted inward deps | M02 exit |
| `oya-shared-architecture-check-cli -- sdk-kernel-only` | `sdk` depends only on `kernel` | M02 exit |
| `oya-shared-bounded-contexts-check-cli` (LEAN-A2) | BC registration + cross-product-refusal | M02 exit |
| `oya-shared-supply-chain-check-cli` (LEAN-A3) | `cargo-deny` bans + SBOM | Day-1 |
| `oya-shared-semver-check-cli` (LEAN-A4) | API stability per ADR-0037 | Day-1 |
| `oya-check-statelessness-cli` | No mutable state in presentation/application/worker | M02 exit |
| `oya-check-shardability-cli` | DB: `tenant_id` partition key + RLS | M02 exit |
| `oya-check-perf-budget-cli` | Impl plans include load-test results | M02 exit |
| `oya-check-benchmark-cli` | PRDs include Competitive Benchmark before L4→L5 | pre-M03 |

Inherited clean-arch foundations per ADR-0060: Bominal ADR-0100 (hexagonal reference impl), ADR-0101 (hexagonal microservice standard), ADR-0102 (hexagonal migration plan), ADR-0103 (workflow hexagonal), ADR-0105 (clean-architecture layering), ADR-0125 (domain naming canon).

### No exceptions for internal µservices

Foundry (internal engine) must be scalable + performant — it is used by the whole agent fleet. `oya-check-statelessness` and `oya-check-shardability` apply to `oya-foundry-*` crates equally.

---

## Consequences

### Concrete crate layout (check crates, BNF v4.1)

```
oya-check-statelessness/     — Rust proc-macro scanner for mutable static state
oya-check-shardability/      — Postgres schema analyzer for tenant_id + RLS
oya-check-perf-budget/       — impl plan load-test result parser + gate
oya-check-benchmark/         — PRD benchmark section presence validator
```

All four registered in `[workspace.members]` and run as part of the LEAN check suite alongside existing `oya-check-architecture`.

### PRD template amendments

Every PRD file for a new µservice (created after this ADR) MUST contain:

```markdown
## Competitive Benchmark
<!-- Industry leader(s) + specific features/quality dimensions + citations -->

## Performance Targets
<!-- p50/p99/p999 latency, throughput, error-budget, SLO burn-rate alarms -->

## Horizontal Scalability
<!-- State strategy, active-active flag, per-cell capacity, scale-out policy, cross-region -->
```

`oya-check-benchmark` validates presence of these sections at L4→L5 graduation.

### Impl plan template amendments

Every implementation plan file MUST contain:

```markdown
## Load test
<!-- k6 / locust / vegeta script reference + results meeting declared perf targets -->
```

`oya-check-perf-budget` validates presence + result pass before merge.

### Positive

- Quality is CI-enforced, not slide-deck aspirational.
- Perf targets are testable and gated; no "we'll optimize later" drift.
- Horizontal scalability is structurally enforced from the first crate.

### Negative

- Initial authoring overhead: every PRD needs three new sections.
- `oya-check-statelessness` proc-macro scanner is non-trivial to build; M02 investment required.
- Load-test requirement adds CI time; mitigated by affected-graph testing (ADR-0050).

---

## Related

- ADR-0001 (cohesion — quality bar applies to all µservices)
- ADR-0009 (cell architecture — horizontal scalability baseline)
- ADR-0056 (BNF v4.1 — check crates in `oya-check-*` namespace)
- ADR-0058 (Flat microservice catalog — every µservice in catalog meets this bar)
- ADR-0059 (Workflow + Ontology — p99 targets declared here apply to both)
- ADR-0060 (Bominal-inheritance — Bominal ADR-0009, ADR-0019, ADR-0028, ADR-0049, ADR-0107, ADR-0117 perf targets inherited)
- `[[feedback-quality-performance-scalability-bar]]` — session decision 2026-05-13
- Bominal ADR-0009 (cell architecture)
- Bominal ADR-0019 (runtime catalog + active-active flag)
- Bominal ADR-0028 (audit-chain perf <1s segment-seal)
- Bominal ADR-0049 (cross-region replication)
- Bominal ADR-0107 (Ontology agent gateway p99 target)
- Bominal ADR-0117 (cloud-native infrastructure scaling stages)
