---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-observability
microservice: observability
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0110
  - ADR-0114
  - ADR-0139
  - ADR-0131
  - ADR-0337
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs: [/specs/agentic-slo-gated-promotion.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
owner_team: axis-observability
doc_status: published
---

# PRD-observability: Agentic SLO + Observability Substrate

## Purpose

The `observability` microservice is oyatie's substrate for SLO authoring, real-time burn-rate evaluation, promotion-eligibility ledger writes, and Layer-A telemetry runtime (self-hosted Grafana Alloy + Prometheus + Mimir + Loki + Tempo + Pyroscope + Grafana + Alertmanager + Grafana OnCall). It is the enforcement origin of ADR-0139's agentic SLO-gated promotion gate.

This µservice is **shared substrate**, not a hero product. It is consumed by every other oyatie µservice (each must author its own OpenSLO manifest before its release pointer can advance past `dev`) and exposed to tenants for tenant-defined SLOs over their own workflows, applications, and integrations. Its existence is the precondition for oyatie's "hyperscaler-grade in every practice" bar per `feedback_quality_performance_scalability_bar.md`.

This µservice has no Bominal equivalent and originates in oyatie.

## Tenant Value

- **Tenant Outcome 1 — SLO authoring without vendor lock.** Tenants define their workloads' SLOs in OpenSLO format inside their own µservice folder; the engine evaluates against the self-hosted Prometheus/Mimir telemetry; no Datadog/Honeycomb/Lightstep contract required.
- **Tenant Outcome 2 — Real-time burn-rate visibility.** Per-microservice Grafana dashboards backed by the Grafana stack; multi-window multi-burn-rate alerts routed through Alertmanager + Grafana OnCall.
- **Tenant Outcome 3 — Promotion safety.** Tenant deployments fast-forward only when SLO evidence is green; rollbacks are automated when production burn-rate exceeds budget; no human-in-loop required to keep tenant tier healthy.
- **Internal Outcome 4 — Substrate uniformity.** Every oyatie µservice and every product is gated by the same SLO model; eliminates per-team divergence in how "production-ready" is defined.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | µservice author | to author an OpenSLO manifest at `microservices/<ms>/slos/<sli>.openslo.yaml` | my microservice's release pointer can advance past `dev` | slo-engine | Must |
| FR-02 | promote-workflow | to read the latest eligibility verdict for `(microservice, source_sha, target_env)` | I can refuse fast-forward when verdict is held / rejected / rollback | slo-engine | Must |
| FR-03 | burn-rate evaluator | to query Prometheus/Mimir over 1h, 6h, 3d, 30d windows for every active SLI | I can compute the canonical Google SRE multi-window multi-burn-rate state | slo-engine | Must |
| FR-04 | eligibility ledger | to append signed JSONL records to `registry/promotion-eligibility.jsonl` | every verdict transition is provably auditable | slo-engine | Must |
| FR-05 | rollback primitive | to fast-forward `release/<microservice>/production` back to its prior pointer when a production-tier fast-burn fires | regressions auto-revert within 1 minute without human escalation | slo-engine | Must |
| FR-06 | tenant operator | to view per-microservice SLO compliance and burn-rate over time | I can plan capacity, validate releases, and meet contractual SLAs | otel-ingest, slo-engine | Must |
| FR-07 | OTel collector | to receive metrics, logs, traces, and profiles from every oyatie µservice via Grafana Alloy | downstream SLO computation has complete signal | otel-ingest | Must |
| FR-08 | incident on-call | to receive a Grafana OnCall page when a fast-burn alert fires | regressions are surfaced operationally, not silently held | slo-engine | Must |
| FR-09 | canary cohort controller | to ramp staging traffic 1 % → 10 % → 50 % → 100 % per microservice per the spec | burn-rate windows accumulate real signal before production promotion | otel-ingest | Must |
| FR-10 | aggregation index | to regenerate `docs/prds/INDEX.md`, `registry/catalog/<crate>.yaml` union, and machine-readable views from per-microservice sources | central indices are never hand-edited; per-microservice folders are source of truth | (cross-cutting) | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Eligibility verdict latency (evaluator → ledger) | ≤500ms | ≤2s | ≤5s | end-to-end from Prometheus query to JSONL append |
| Burn-rate evaluator cadence | — | 60s | — | continuous; one record per `(microservice, source_sha, target_env)` per cycle |
| OpenSLO manifest hot-reload | ≤1s | ≤3s | — | manifest SHA change → evaluator picks up new spec |
| Promote workflow trigger latency (event-driven) | ≤2s | ≤10s | — | from `eligibility-changed` dispatch to workflow start |
| Rollback execution latency | ≤30s | ≤60s | — | fast-burn fire → production ref reverted |
| Grafana dashboard query latency | ≤200ms | ≤1s | ≤3s | per Mimir best-practice tuning |

### Security

- All ledger writes are signed by the evaluator's per-environment signing key (Ed25519 per Bominal ADR-0028 audit-chain posture).
- OpenSLO manifest reads are restricted to the µservice's own folder; cross-µservice manifest reads are explicitly disallowed at the path layer.
- Layer-A endpoints (Prometheus, Mimir, Loki, Tempo, Grafana, Alertmanager, OnCall) are accessible only via the mesh-internal control plane; no public exposure.
- Secrets (Grafana OnCall integration tokens, Mimir multi-tenant API keys) follow the local-OpenBao SecretReference pattern (per user 2026-05-12 directive `feedback_openbao_secrets.md`); raw secrets never enter the repo, chat, or checkpoints.

### Audit + Compliance

- Every `EligibilityChanged`, `PromotionExecuted`, and `RollbackExecuted` event emits an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028).
- `registry/promotion-eligibility.jsonl` is append-only; the file is union-merged across concurrent agent commits via the existing `.gitattributes` driver.
- Audit-chain seal latency ≤1s per `(tenant, period)`.

### Availability + SLO

- Availability target: 99.95 % monthly for the evaluator's `eligibility-changed` event emission path (the gate decision must be available even when the µservice it gates is degraded).
- Layer-A telemetry availability target: 99.9 % monthly per the Grafana stack's published SLO posture.
- RTO: ≤15 min. RPO: ≤60 s (one evaluator cycle).

### Data residency

- SLO manifests, ledger records, and per-tenant dashboards inherit the tenant's `jurisdiction_code` per ADR-0117. Mimir multi-tenancy enforces per-tenant data isolation.

### DR posture

| Field | Value |
|---|---|
| ADR | ADR-0343 |
| Target | RTO 900 s and RPO 300 s for evaluator, eligibility-ledger, and ClickHouse telemetry rollup state, matching `manifest.json#dr`. |
| Compliance-pack floor | HIPAA floor RTO 3600 s / RPO 300 s, SOC2-T2 floor RTO 14400 s / RPO 900 s, ISO27001 floor RTO 14400 s / RPO 3600 s; observability keeps the stricter 900 s / 300 s manifest target. |
| Failover runbook | `runbooks/held-promotion-recovery.md`, `runbooks/rollback.md`, and `runbooks/clickhouse-restore.md`. |
| Multi-region active-active | Active-active per pack for ingest and SLO evaluation when the pack has local telemetry stores; ClickHouse cold restore remains runbook-driven and pack-local. |
| WHY | Promotion gates and incident response depend on fresh SLO evidence, so DR must restore eligibility verdicts before deployment safety degrades. |

### Capacity model

| Field | Value |
|---|---|
| ADR | ADR-0340, with pod runtime tier declared by ADR-0338. |
| Per-tenant baseline | `manifest.json#capacity_model`: 0.26 vCPU, 768 MiB RAM, 20 GB storage, and connections `{valkey: 3, postgres: 3, outbound_http: 8}` per tenant/query source. `capacity-model.md` also parameterizes active series, samples/sec, log bytes/sec, trace spans/sec, and profile rate. |
| Scaling dimension | `per_query`, matching `manifest.json#capacity_model.scaling_dimension`; ingest and SLO evaluation capacity are attached to the query/telemetry pressure they create. |
| Cell placement class | Tier-1 per `manifest.json#capacity_model.cell_placement_class`; service criticality remains `criticality_tier=T0`, and runtime tier is ADR-0338 Tier-1 because `manifest.json#pod_runtime_tier=1`. |
| Autoscaling boundaries | Mimir distributor 4 at XS through 1300 at L; Mimir ingester 12 at XS through 600 at L; SLO engine worker/rest/app min 2. ClickHouse observability ceiling: 500K rows/sec steady, 2M hard; 100 TB hot-tier sustained triggers in-house OLAP review. |
| WHY | The model serves high-cardinality telemetry and promotion evidence without letting one tenant's log or trace burst erase fleet-wide SLO visibility. |

### Sustainability + cost attribution

| Field | Value |
|---|---|
| ADR | ADR-0344 |
| Per-call emission claim | Every eligibility, SLO-evaluate, OpenSLO-validate, ClickHouse DDL, bridge, promotion, and rollback audit row must emit `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region`. |
| Carbon-aware routing | Yes for ClickHouse rollups, cold-tier compaction, dashboard backfills, and long-window replay. No for SLO breach evaluation, fast-burn alerts, incident pages, or production promotion gates. |
| Tenant transparency surface | Tenant observability dashboards and the FinOps portal expose telemetry ingestion, retention, query, and SLO-evaluation cost by tenant, provider, cell, signal type, and compliance pack. |
| WHY | CSRD, SB-253, and SEC climate-disclosure posture require telemetry cost transparency, but safety gates and incident response have to prefer freshness over carbon placement. |

### API versioning posture

| Field | Value |
|---|---|
| ADR | ADR-0342 |
| Public API version model | Date carrier triplet: `Oyatie-Version: YYYY-MM-DD`, `/v/YYYY-MM-DD/...` for public REST/SSE/WebSocket surfaces, and proto3 `oyatie_version`. |
| SDK semver model | Observability SDKs use `major.minor.patch`; telemetry schema versions remain explicit signal metadata. |
| Support window | Last N=3 public versions supported for >=180 days. |
| Per-tenant pinning | Yes for dashboard/query APIs, SLO authoring, and SDK consumers; emergency alert schemas may receive safety patches without tenant pin delay. |
| Internal-mesh exemption | Yes. ADR-0145 direct gRPC remains exempt from public URL date prefixes while proto3 metadata preserves compatibility. |

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), layers used by this µservice are: `kernel`, `domain`, `usecase`, `api` (protocol-neutral typed contracts), `adapter`, `rest`, `worker`, `sdk` (client library — closes hyperscaler SDK gap), `app` (composition root). The slo-engine BC includes one optional `-adapter-mimir` backend-qualified adapter per ADR-0105 §"Amendment 3" (`*-adapter-<backend>` pattern).

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `slo-engine` | `oya-observability-slo-engine-{kernel,domain,usecase,api,adapter,adapter-mimir,rest,worker,sdk,app}` | OpenSLO manifest read; burn-rate computation; eligibility verdict emission to Mimir; rollback primitive; tenant-facing SLO query API + client SDK | `SloTarget`, `BurnRateWindow`, `EligibilityVerdict`, `ReleasePointer`, `MimirTenant` |
| `otel-ingest` | `oya-observability-otel-ingest-{kernel,usecase,api,adapter,app}` | OpenTelemetry collector configuration; per-microservice signal annotation; mesh integration. Most ingest is Grafana Alloy (upstream OSS) — oyatie code is configuration + per-µservice signal-routing rules. | `Sample`, `LogRecord`, `Trace`, `Profile`, `ServiceMeshTag` |
| `eligibility-ledger-writer` | (subsumed under `slo-engine`'s `-adapter-mimir` crate) | Emits eligibility verdicts as Prometheus metrics into Mimir per ADR-0139's Mimir-native ledger model | — |

Naming justification — `slo-engine`:

```
NAME: oya-observability-slo-engine-<layer>
JUSTIFICATION:
- microservice = observability: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder. No shared|vertical bisection.
- bc-tokens = slo-engine: primary BC for SLO evaluation, eligibility verdict emission,
  rollback primitive. ADR-0056 v4.1 BC-optionality rule honoured (sibling BC otel-ingest
  exists, justifying explicit BC token).
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + sealed-trait + entity types (SloTarget, BurnRateWindow,
    EligibilityVerdict, ReleasePointer, MimirTenant). Zero I/O. Carries data_class
    annotations on every field (per Bominal ADR-0028 + oya-check-data-class lane).
  - domain: pure burn-rate math, window arithmetic, error-budget computation.
  - usecase (per ADR-0106; replaces legacy 'application'): orchestrators reading
    OpenSLO + Prometheus, computing verdicts, writing ledger metrics via ports.
  - api: protocol-neutral typed I/O contracts (request/response types + error variants).
    Consumed by rest/sdk; depends on kernel only.
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-mimir: backend-qualified adapter (per ADR-0105 Amendment 3
    `*-adapter-<backend>` pattern); implements PrometheusClient + MetricEmitter
    against Mimir multi-tenant HTTP endpoints.
  - rest: HTTP handler/route layer; consumes -api types.
  - worker: long-lived continuous evaluator binary (60s cadence; emits metrics + events).
  - sdk: client library (Rust; future TS/Python via bindings) for tenant-side
    OpenSLO authoring + verdict subscription. Closes the industry-standard
    observability SDK gap.
  - app: composition root binary; wires worker + rest + adapter clients.
- exemptions claimed: none. -adapter-mimir uses the canonical *-adapter-<backend>
  pattern; no exception required.
```

Naming justification — `otel-ingest`:

```
NAME: oya-observability-otel-ingest-<layer>
JUSTIFICATION:
- microservice = observability.
- bc-tokens = otel-ingest: sibling BC for OpenTelemetry ingest pipeline.
- layer = <layer>: trimmed crate set because Layer-A self-hosted OSS adoption
  (Grafana Alloy is upstream OTel collector) means oyatie ships configuration +
  per-µservice routing, not a custom collector implementation.
  - kernel: port-trait + entity types (Sample, LogRecord, Trace, Profile,
    ServiceMeshTag). Zero I/O.
  - usecase: per-µservice signal annotation + routing logic.
  - api: typed contracts for upstream consumers (Alloy receiver configuration).
  - adapter: Alloy configuration emission; ServiceMonitor / PodMonitor CRD
    generation for in-cluster discovery.
  - app: composition root binary (generator + validator entry point).
- exemptions claimed: none.
```

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-mimir | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|
| `slo-engine` | `oya-observability-slo-engine-kernel` | `oya-observability-slo-engine-domain` | `oya-observability-slo-engine-usecase` | `oya-observability-slo-engine-api` | `oya-observability-slo-engine-adapter` | `oya-observability-slo-engine-adapter-mimir` | `oya-observability-slo-engine-rest` | `oya-observability-slo-engine-worker` | `oya-observability-slo-engine-sdk` | `oya-observability-slo-engine-app` |
| `otel-ingest` | `oya-observability-otel-ingest-kernel` | — | `oya-observability-otel-ingest-usecase` | `oya-observability-otel-ingest-api` | `oya-observability-otel-ingest-adapter` | — | — | — | — | `oya-observability-otel-ingest-app` |

Total crates introduced by this µservice: **15** (10 in slo-engine + 5 in otel-ingest).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `SloTargetRepository` | `oya-observability-slo-engine-kernel` | `-adapter` (OpenSLO YAML reader) | `INTERNAL_ONLY` (manifest content) |
| `PrometheusClient` | `oya-observability-slo-engine-kernel` | `-adapter-mimir` (Mimir HTTP client, per-tenant scoped) | `BEHAVIORAL_TENANT_PRODUCT` (per-tenant query results) |
| `BurnRateEvaluator` | `oya-observability-slo-engine-kernel` | `-usecase` (orchestrator; pure logic via domain math) | `INTERNAL_ONLY` (verdict computation; per-tenant SLI inputs flow through but no PII at this layer) |
| `EligibilityVerdictEmitter` | `oya-observability-slo-engine-kernel` | `-adapter-mimir` (emits Prometheus gauges; Mimir tenant-header scoped) | `BEHAVIORAL_TENANT_PRODUCT` |
| `ReleasePointerStore` | `oya-observability-slo-engine-kernel` | `-adapter` (Git refs HTTP API) | `AUDIT` (audit-chain record per advance / rollback) |
| `MimirTenantResolver` | `oya-observability-slo-engine-kernel` | `-adapter` (resolves caller → Mimir X-Scope-OrgID) | `SENSITIVE_PIPA_ART23` (tenant identifier mapping) |
| `OtelSampleReceiver` | `oya-observability-otel-ingest-kernel` | `-adapter` (Alloy configuration emission) | varies per-µservice signal; declared at consume site |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane (now under `microservices/governance/`) refuses unannotated fields at PR-time per `feedback_clean_architecture_requirements.md`.

Cross-product rule: `observability` MUST NOT import any other product µservice crate at any layer. All cross-product flows go through Workflow (events) or Ontology (entity reads/writes). LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice observability` — dependency-direction
- `oya gate validate lean-a2 --microservice observability` — cross-product-refusal
- `oya gate validate port-location --microservice observability` — ports in kernel
- `oya gate validate layer-correctness --microservice observability` — layer enum match
- `oya gate validate per-microservice-layout --microservice observability` — ADR-0131 conformance
- `oya gate validate statelessness --microservice observability`
- `oya gate validate shardability --microservice observability`

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `EligibilityChanged` | burn-rate evaluator verdict transition | `promote-dev-to-staging.yml`, `promote-staging-to-production.yml`, `grafana-oncall` | promotion-state-machine per `/specs/agentic-slo-gated-promotion.json` |
| `PromotionExecuted` | promote workflow advances `release/<ms>/<env>` | `audit-chain`, per-microservice release-pointer ledger | — |
| `RollbackExecuted` | rollback primitive reverts `release/<ms>/production` | `audit-chain`, incident response, `grafana-oncall` | — |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `MicroserviceRegistered` | `tenancy` (when a new µservice scaffolds) | `slo-engine` | discover the new µservice; expect an OpenSLO manifest within bootstrap window; emit `eligible=held` until manifest arrives |
| `OpenSLOManifestUpdated` | `oya-governance-aggregation-index-generation` (on git push) | `slo-engine` | hot-reload the manifest; trigger a re-evaluation cycle |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `SLOTarget{microservice, sli, target, window, error_budget}` | `targets→Microservice` | `slo-engine` | Ed25519 |
| `EligibilityVerdict{microservice, sha, environment, verdict, snapshot}` | `verdict_for→ReleasePointer` | `slo-engine` | Ed25519 |
| `ReleasePointer{microservice, environment, current_sha, prior_sha}` | `pointer_for→Microservice` | `slo-engine` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Microservice` (catalog) | `slo-engine` | `filter(tenant_id?).where(active=true)` to enumerate µservices requiring SLO coverage |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Grafana Labs | Grafana Cloud SLO product | SLO authoring, burn-rate alerts, dashboard generation | `grafana.com/products/cloud/slo` |
| Datadog | Datadog SLO product | SLO definition, 4-window burn-rate, target tracking | `docs.datadoghq.com/service_management/service_level_objectives/` |
| Nobl9 | OpenSLO-native SLO platform | OpenSLO manifest authoring, multi-source SLI computation | `docs.nobl9.com` |
| Sloth | Prometheus + OpenSLO + Sloth alert rules | OpenSLO → PromQL alert rule generator | `github.com/slok/sloth` |
| Google Cloud Monitoring | Service monitoring SLOs | Per-service SLO targets, burn-rate alerts | `cloud.google.com/monitoring/slo` |

Key parity gaps to close (ordered by priority):

1. **Promotion-gate integration** — none of the competitors gate VCS fast-forwards on SLO compliance; this is oyatie's differentiator. Target: ledger-backed, signed, per-microservice, automated rollback.
2. **OpenSLO native** — Sloth and Nobl9 are OpenSLO-native; Datadog and Grafana have OpenSLO converters. Target: native authoring with no proprietary alternative.
3. **Self-hosted, no vendor coupling** — Datadog/Grafana Cloud/Nobl9 are SaaS. Target: every component self-hosted under the Grafana OSS stack.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Eligibility verdict latency | ≤500ms | ≤2s | ≤5s | end-to-end evaluator → ledger |
| Burn-rate query throughput | — | 10k+ queries/min/cluster | — | Mimir-backed |
| Promote workflow event lag | ≤2s | ≤10s | — | dispatch → workflow start |
| Rollback execution | ≤30s | ≤60s | — | breach → ref reverted |
| OTel collector throughput | — | 100k spans/s per Alloy replica | — | per Grafana published benchmarks |

Error budget:
- Monthly error budget for evaluator: 0.05 % (≈22 min/month).
- Burn-rate alarm on the evaluator itself: 14.4× burn rate over 1 h triggers page (self-observability).
- Error budget policy: `microservices/observability/runbooks/error-budget-policy.md` (Turn 3).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `stateless | postgres | object-storage | persistent-volume | mixed` → **`mixed`**. Rationale: evaluator workers are stateless (re-derivable from Prometheus + ledger); Mimir uses object storage for long-term metrics; Loki uses object storage for logs; Tempo uses object storage for traces; Grafana uses Postgres for dashboard metadata.

**Active-active compatibility**: `stateless-compatible` for evaluator workers; Mimir and Loki are horizontally shardable; Tempo is partition-by-trace; Grafana dashboards replicate.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Max evaluator throughput | 100 microservices | 1000 microservices | evaluator queue depth > 60s of cadence |
| Mimir samples/s ingest | 1M | 10M | Mimir block-ingester CPU > 70 % |
| Loki ingest | 100 GB/day | 1 TB/day | Loki ingester CPU > 70 % |
| Tempo trace ingest | 10k traces/s | 100k traces/s | Tempo ingester memory > 80 % |

Scale-out policy:
- Kubernetes HPA: evaluator workers scale on CPU `>70%`; min 2 replicas, max 50 replicas.
- Mimir / Loki / Tempo: horizontal scale via Grafana-published Helm chart `replicas` parameter; storage backed by S3-compatible object storage.
- Pre-warmed pool: 2 standby evaluator pods; cold-start budget ≤500 ms.

Cross-region story:
- M01 launch: single KR region (OCI ap-seoul-1); per-tenant residency locked per ADR-0117.
- Post-M01 expansion: federated Mimir + replicated Loki/Tempo per region; ADR successor-IP.

Sharding:
- Eligibility ledger partitions by `microservice`; evaluator shards by µservice or by SLO target without coordination.
- `oya-check-shardability-cli` CI lane verifies partition key presence.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | OpenSLO manifest at `microservices/<ms>/slos/<sli>.openslo.yaml` validates against OpenSLO v1.0 schema | `cargo run -p oya-observability-slo-engine-rest -- validate <path>` exit 0 |
| AC-02 | Eligibility verdict for known-good SHA on green burn-rate window is `eligible` | end-to-end test under `microservices/observability/tests/e2e/eligibility-happy-path.rs` |
| AC-03 | Eligibility verdict transitions `eligible → held` within ≤60s of fast-burn alert firing | timed e2e drill |
| AC-04 | `oya-vcs-promotion-readiness` CI lane refuses fast-forward when verdict is `held` | branch-protection emulation test |
| AC-05 | Rollback primitive reverts `release/<ms>/production` within ≤60s of post-promotion fast-burn fire | timed e2e rollback drill |
| AC-06 | Grafana OnCall raises incident on `held → page` transition | webhook integration test |
| AC-07 | Canary cohort ramp follows 1 % → 10 % → 50 % → 100 % schedule with abort-on-burn | service-mesh integration test |
| AC-08 | All Layer-A Helm charts deploy clean against a kind cluster | CI lane `oya-observability-iac-smoke` |
| AC-09 | `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice observability` exit 0 | ADR-0131 lane |
| AC-10 | `cargo run -p oya-dev-cli -- gate validate authority-cohesion` exit 0 | ADR-0123 lane; HG-OBS registered |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Evaluator long-lived service vs. scheduled GitHub Action — final landing decision | axis-observability | resolved in IP-008 |
| 2 | Layer-A cluster co-located with workload cluster vs. dedicated observability cluster | ops-sre-reliability | resolved in IP-001 |
| 3 | Tenant-defined SLOs: same OpenSLO file shape, or a tenant-scoped variant? | council-architecture | ADR-#### successor-IP |
| 4 | Self-observability: does the SLO engine evaluate its own SLOs (bootstrap paradox)? Default: yes, with a synthetic-probe fallback during cold-start. | axis-observability | resolved in IP-002 |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0110 | ChangeSet state machine | each IP is one ChangeSet |
| ADR-0114 | Canary observability rollback | precedent — ADR-0139 implements |
| ADR-0139 | Agentic SLO-gated promotion | the design this PRD scaffolds |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it |
| ADR-0123 | Hyperscaler maturity claim gate | HG-OBS registers here |
| ADR-0116 | Retire external agent-coordination tooling | oya vcs primitives used throughout |

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `observability` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `observability` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 4 context(s).
- Scaling input: `per_query` with cell placement `Tier-1` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
