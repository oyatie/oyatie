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
related_adrs: [ADR-0056, ADR-0105, ADR-0110, ADR-0114, ADR-0139, ADR-0131]
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
| 3 | Tenant-defined SLOs: same OpenSLO file shape, or a tenant-scoped variant? | council-architecture | ADR-NNNN successor-IP |
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
