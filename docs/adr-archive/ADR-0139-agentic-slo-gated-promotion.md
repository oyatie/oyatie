---
id: ADR-0139
status: Superseded
deciders: council-architecture, ops-sre-reliability, ops-security, axis-foundry, axis-observability
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: [ADR-0700]
related: [ADR-0041, ADR-0056, ADR-0105, ADR-0110, ADR-0111, ADR-0112, ADR-0113, ADR-0114, ADR-0116, ADR-0123, ADR-0131]
related_specs: [/specs/agentic-slo-gated-promotion.json, /specs/hyperscaler-gates.json, /specs/masterplan.json, /specs/master-plan-sequencing.json]
bominal_source: |
  Verification result (2026-05-17 audit): no Bominal ADR governs agentic SLO-gated promotion.
  Bominal carries observability-substrate ADRs (e.g., ADR-0009 cell observability; ADR-0019 runtime catalog)
  but neither addresses gate-integrated SLO authority. This ADR is oyatie originating.
competitor_parity_reference: |
  microservices/observability/competitor-parity-matrix.md §"Gate integration (the differentiator)" —
  no competitor (Grafana Cloud SLO / Datadog SLO / Nobl9 / Sloth / GCP SLO / New Relic / Honeycomb)
  enforces SLO-gated promotion at the VCS layer. This ADR is the unique oyatie differentiator.
purpose: Make every fast-forward of `staging` and `production` strictly conditional on hyperscaler-grade SLO evidence, by adopting the OSS observability leaders (Grafana Alloy + Prometheus + Mimir + Loki + Tempo + Pyroscope + Grafana + Alertmanager + Grafana OnCall) as the substrate and owning the differentiator (OpenSLO-driven SLO engine + per-component release pointers + agentic promotion-eligibility ledger + event-driven promote workflows + automated rollback + canary cohort weighting).
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0139: Agentic SLO-gated promotion

## Status

Accepted — 2026-05-17.

**Amended — 2026-06-29 (review evidence retirement):** ADR-0515 and `docs/AGENTS.md` retire the multispectrum review/evidence convention. References in this ADR to `microservices/**/evidence/multispectrum/**` and `/specs/multispectrum-review.json` are historical only; current promotion evidence is Mimir metrics/recording rules plus typed cloud-ci/oya-ci quality-gate packets and audit-chain events.

**Amended — 2026-07-02 (DATA-005 static data-substrate skeleton):** DATA-005/G003 may introduce a contract-only, evidence-required data substrate ops skeleton without claiming runtime/controller readiness, measured SLO windows, Argo CD live sync, tenant traffic, or database migrations. This amendment justifies the additive static review surface at `data/ops/README.md`, `data/ops/operator/kustomization.yaml`, `data/ops/operator/namespace.yaml`, `data/ops/operator/service-account.yaml`, `data/ops/operator/role.yaml`, `data/ops/operator/role-binding.yaml`, `data/ops/operator/configmap.yaml`, `data/ops/operator/deployment.yaml`, `data/ops/gitops/kustomization.yaml`, `data/ops/gitops/data-substrate-operator.application.yaml`, and `data/observability/slos/data-substrate/operator-reconciliation-latency.openslo.yaml`. Review/fix evidence for this slice belongs in PR Code Review / typed quality-gate artifacts, not in standalone `evidence/multispectrum/*.json` files. Promotion remains blocked until future DATA-005 successors attach live controller, CRD, measured SLO, burn-rate, audit-chain, and GitOps sync evidence.

## Context

oyatie's current branch pipeline (per `feedback_branch_pipeline_implemented.md`, 2026-05-16) fast-forwards `staging` from `dev` on push + 30-minute cron, and fast-forwards `production` from `staging` on push + hourly cron. Both promotions are **unconditional**. The canary-cohort-observability and full-rollout-observability lanes referenced in `promote-dev-to-staging.yml` and `promote-staging-to-production.yml` are FUTURE stubs; no gate exists today between a `dev` merge and a production deployment within ~1 hour.

This contradicts the hyperscaler-grade bar (`feedback_quality_performance_scalability_bar.md`) and the user-mandated 2026-05-17 directive: "any of our practices should be hyperscaler grade, industry leading" plus "nothing scheduled-for-distinct-tracked-work until later."

Three reinforcing user directives narrowed the design space:

1. **Deterministic gate, no LLM.** Promotion decisions consume policy-checked SLO/observability thresholds; no LLM reasoning in the gate path.
2. **Adopt OSS leaders, own the differentiator.** The observability runtime is the canonical OSS Grafana stack (used internally by AWS Managed Prometheus, GCP Managed Service for Prometheus, Grafana Cloud, Cloudflare, Shopify). oyatie's IP lives in the SLO model, the eligibility ledger, the per-component release pointers, and the agentic gate workflow — not in re-implementing Prometheus or Grafana.
3. **Industry-standard SLO model.** Google SRE Workbook ch. 5 multi-window multi-burn-rate alert pattern; OpenSLO manifest format (same shape used by Sloth, Nobl9, Datadog OpenSLO converter); per-component release pointers (precedent: Linear per-service, Stripe per-service, Google per-binary).

This ADR has no Bominal equivalent; promotion gating is an oyatie originating substrate decision.

This ADR depends on ADR-0131 (per-microservice flat layout) for the location of every artifact it produces. The observability µservice ships natively under that convention.

## Decision

oyatie adopts a two-layer design: **adopted OSS observability runtime (Layer A)** plus **oyatie owned agentic-gate differentiator (Layer B)**. Both layers ship together as one M01 phase; neither is scheduled-for-distinct-tracked-work. The deployment substrate is the canonical Grafana stack, self-hosted; the gate logic is a new oyatie µservice `observability` with the BNF v4.1 crate family `oya-observability-<bc>-<layer>` (see §Naming Justification).

### Layer A — adopted OSS runtime (commodity; deployed, not written)

All Apache-2.0; deployed alongside oyatie services via Helm/Kustomize manifests in `microservices/observability/iac/helm/`:

1. **Grafana Alloy** — unified OpenTelemetry collector (metrics + logs + traces + profiles); replaces what would have been a custom collector.
2. **Prometheus + Grafana Mimir** — TSDB + horizontally-scalable long-term metrics storage; PromQL is the canonical query language.
3. **Grafana Loki** — log aggregation; LogQL.
4. **Grafana Tempo** — distributed trace storage; TraceQL.
5. **Grafana Pyroscope** — continuous profiling.
6. **Grafana** — visualization, dashboards, ad-hoc query UI.
7. **Prometheus Alertmanager** — alert routing, deduplication, silencing.
8. **Grafana OnCall** (Apache-2.0) — incident paging integration; replaces any PagerDuty/Opsgenie dependency in the gate path.

Industry precedent for Layer A: AWS Managed Prometheus + Managed Grafana (Grafana stack), GCP Managed Service for Prometheus (Prometheus + Grafana), Cloudflare (Prometheus + Grafana per public engineering blog 2019–2024), Shopify (Datadog plus open-source Prometheus per Engineering blog 2020+), Grafana Labs Cloud (the stack, multi-tenant). Self-hosting this stack inside oyatie's perimeter *is* the hyperscaler practice; rebuilding equivalent substrate is a 50-person-year detour not warranted by oyatie's scale or product surface.

### Layer B — oyatie owned agentic gate (the differentiator)

The new µservice `observability` introduces the following BCs and substrate. Every artifact ships under `microservices/observability/` per ADR-0131.

9. **OpenSLO manifests per component** at `microservices/<ms>/slos/<sli>.openslo.yaml` — industry-standard OpenSLO spec (same shape as Sloth, Nobl9). Each manifest carries SLI definition, 30-day SLO target, error-budget windows, and burn-rate alert thresholds.
10. **`oya-observability-slo-engine-{kernel,domain,application,adapter,rest,worker,app}`** — new crate family at `microservices/observability/crates/`. Reads OpenSLO manifests; queries Prometheus/Mimir via PromQL; computes multi-window multi-burn-rate per the Google SRE Workbook ch. 5 canonical pattern (2 % budget / 1 h fast burn, 5 % budget / 6 h slow burn); emits per-component eligibility verdicts to the ledger.
11. **Per-component release pointers** — ref naming convention `release/<microservice>/<environment>` (e.g. `release/observability/staging`, `release/workflow/production`). The single tree-wide `staging` and `production` refs are deprecated; each µservice fast-forwards independently. Precedent: Linear per-service deploy refs (public 2023 engineering blog), Stripe service-per-pointer, Google per-binary release. Branch-protection rule patterns matched against `release/*/*`.
12. **Promotion-eligibility ledger — Mimir-native (recording rules ARE the ledger)** — the burn-rate evaluator emits eligibility verdicts as Prometheus metrics (`oya_promotion_eligibility_verdict`, `oya_promotion_burn_rate_*`, `oya_promotion_release_pointer_*`) into Mimir; recording rules compute the aggregate "is this SHA eligible across every microservice it touches" view. The ledger is the Mimir time-series store (object-storage-backed, 90d hot + 2y cold). Query API is PromQL via Mimir's HTTP endpoints. **No git-tracked JSONL ledger.** The archived per-evaluation view is an audit-chain event and typed quality-gate packet keyed by change id, release pointer, environment, correlation id, and evaluation timestamp; historical multispectrum paths are not current coverage claims. Industry-canonical: Google Borgmon → Monarch and AWS CloudWatch SLO both store SLO verdicts as time-series, not as git-tracked files. See `/specs/agentic-slo-gated-promotion.json` §"promotion_eligibility_ledger" for the metric and recording-rule schema.
13. **`oya-vcs-promotion-readiness` CI lane** — added to `.github/branch-protection.yaml` required-status-checks on `dev` and `staging`. Lane reads the ledger; refuses the fast-forward unless the latest record for every component touched by the source SHA is `eligible` for the target environment.
14. **Continuous burn-rate evaluator** — runs inside the `oya-observability-slo-engine-worker` crate; 1-minute cadence; idempotent ledger writes. The evaluator is the event source.
15. **Event-driven promote workflows** — `.github/workflows/promote-dev-to-staging.yml` and `.github/workflows/promote-staging-to-production.yml` rewritten to fire on `repository_dispatch` event `eligibility-changed`. The existing 30-minute and hourly crons remain only as reconciliation heartbeat, not as primary trigger.
16. **Automated rollback primitive** — production-tier burn-rate breach triggers `release/<microservice>/production` to fast-forward back to the prior ledger pointer for that microservice. Signed, linear, recorded as a `rollback` verdict.
17. **Canary cohort weighting** — staging traffic ramps progressively (1 % → 10 % → 50 % → 100 %) per microservice via Layer-A service-mesh traffic-split. Without this, the staging ref advancing produces no observable traffic and the gate would have no signal.

### Naming justification

```
NAME: oya-observability-slo-engine-<layer>
JUSTIFICATION:
- microservice = observability: matches the new microservices/observability/ folder name; this
  µservice owns SLO evaluation, OpenSLO manifest authoring discipline, eligibility ledger
  writes, burn-rate alerting. ADR-0056 v4.1 flat BNF; no shared|vertical bisection.
- bc-tokens = slo-engine: one BC of the µservice; sibling BCs may be added (e.g. otel-ingest,
  burn-rate-evaluator) but slo-engine is the primary read-path differentiator. ADR-0056 v4.1
  BC-optionality rule honoured.
- layer = <layer>: one crate per layer per ADR-0105 13-value enum:
  - kernel: port traits + entity types (SLOTarget, BurnRateWindow, EligibilityVerdict)
  - domain: SLO computation invariants; burn-rate math; window arithmetic
  - application: orchestrators reading OpenSLO + Prometheus, writing ledger
  - adapter: Prometheus/Mimir HTTP client; OpenSLO YAML reader; ledger writer
  - rest: optional REST surface for human-readable SLO/eligibility query
  - worker: continuous evaluator binary (long-lived service)
  - app: composition root binary
- exemptions claimed: none.
```

Layer mapping respects clean-arch dependency direction per `feedback_clean_architecture_requirements.md`: `kernel ← domain ← application ← adapter ← {rest, worker} ← app`.

## Rejected alternatives

- **Build a parallel proprietary observability stack** — 8–12 net-new crates duplicating TSDB / log pipeline / trace pipeline / query engine / dashboards / alertmanager. Rejected: industry-leading practice IS to adopt the OSS leaders. The differentiator is the SLO engine + agentic gate, not the TSDB. A 50-person-year detour unless observability becomes oyatie's primary product (it isn't; per 2026-05-17 user statement "observability isn't a hero product. it is just for our internal use and for our tenants").
- **Vendor-managed observability (Datadog / GCP Cloud Operations / Honeycomb / Lightstep)** — rejected by the adopt-OSS-leaders choice; would couple the gate to an external read endpoint and violate the deterministic-gate-no-LLM requirement (managed-vendor SLO APIs vary in availability and rate-limiting).
- **LLM-in-loop reasoning gate** — rejected. Deterministic thresholds with an open evidence trail are the industry-standard answer (Google SRE Workbook; AWS CloudWatch SLO; Datadog SLO product). An LLM in the gate path adds opaque non-determinism for no measurable benefit on threshold-style decisions.
- **Single-error-budget gate** — strict subset of the multi-window burn-rate model; would be deferral by another name and fails to distinguish fast burns (cheap rollback, page on-call) from slow burns (ticket, planned remediation).
- **Monorepo-wide single release ref** — coarse for a flat µservice catalog; per-component is the industry-leading shape (Linear, Stripe, Google per-binary, AWS service-team deploys). One bad µservice would hold healthy ones.
- **Rollback-first as the primary gate** — rejected. Rollback is a co-delivered safety net, not the gate. Permitting bad ships and reverting them is below the hyperscaler bar per `feedback_no_silent_regression.md`.
- **Synthetic-probe-only signal** — rejected. Synthetic probes are useful for bootstrap but cannot substitute for real-traffic SLOs at hyperscaler grade. The design uses real telemetry from real workloads via Grafana Alloy.
- **Cron-only promotion (no event-driven trigger)** — rejected. Cron is reconciliation heartbeat; the primary trigger is the evaluator's `eligibility-changed` event so promotion responds to signal in ≤1 minute, not ≤30 minutes.

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | BNF v4.1 name | Layer |
|---|---|---|---|
| `microservices/observability/` | create | — | (folder) |
| `microservices/observability/PRD.md` | create | — | — |
| `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md` | create | — | — |
| `microservices/observability/IP-001-*.md` through `IP-015-*.md` | create | — | — |
| `microservices/observability/crates/oya-observability-slo-engine-kernel/` | create | `oya-observability-slo-engine-kernel` | kernel |
| `microservices/observability/crates/oya-observability-slo-engine-domain/` | create | `oya-observability-slo-engine-domain` | domain |
| `microservices/observability/crates/oya-observability-slo-engine-application/` | create | `oya-observability-slo-engine-application` | application |
| `microservices/observability/crates/oya-observability-slo-engine-adapter/` | create | `oya-observability-slo-engine-adapter` | adapter |
| `microservices/observability/crates/oya-observability-slo-engine-rest/` | create | `oya-observability-slo-engine-rest` | rest |
| `microservices/observability/crates/oya-observability-slo-engine-worker/` | create | `oya-observability-slo-engine-worker` | worker |
| `microservices/observability/crates/oya-observability-slo-engine-app/` | create | `oya-observability-slo-engine-app` | app |
| `microservices/observability/catalog/oya-observability-slo-engine-<layer>.yaml` | create | per-crate | — |
| `microservices/observability/slos/<sli>.openslo.yaml` | create | — | — |
| `microservices/observability/iac/helm/{alloy,prometheus,mimir,loki,tempo,pyroscope,grafana,alertmanager,oncall}/` | create | — | — |
| `microservices/observability/contracts/openapi/slo-engine.yaml` | create | — | — |
| `microservices/observability/runbooks/{rollback,held-promotion-recovery,canary-graduation}.md` | create | — | — |
| `microservices/observability/threat-model.md` | create | — | — |
| `registry/promotion-eligibility.jsonl` | create | — | (append-only ledger) |
| `/specs/agentic-slo-gated-promotion.json` | create | — | — |
| `.github/branch-protection.yaml` | update | add `oya-vcs-promotion-readiness` to required_status_checks on `dev` and `staging` | — |
| `.github/workflows/promote-dev-to-staging.yml` | update | switch primary trigger to `repository_dispatch: eligibility-changed`; retain cron as heartbeat; remove FUTURE-marked canary stub references | — |
| `.github/workflows/promote-staging-to-production.yml` | update | as above for the production layer | — |
| `docs/standards/observability-slo.md` | create | cross-cutting authoring rules (per ADR-0131 §"central"): OpenSLO manifest discipline, SLI catalog (availability/latency/correctness/freshness), burn-rate threshold convention | — |
| `Cargo.toml` (workspace) | update | add the new 7 crates to `[workspace.members]` under `microservices/observability/crates/*` | — |

### Integration via Workflow + Ontology

This ADR introduces typed events and Ontology writes per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`:

- **Workflow events produced**: `EligibilityChanged{microservice, source_sha, target_env, verdict, evaluated_at}` — consumed by the promote-workflow trigger and the OnCall paging path.
- **Workflow events produced**: `PromotionExecuted{microservice, target_env, sha, executed_at}` — consumed by audit-chain and the per-component release-pointer ledger.
- **Workflow events produced**: `RollbackExecuted{microservice, target_env, from_sha, to_sha, reason, executed_at}` — consumed by incident response and audit.
- **Ontology writes**: Object Type `SLOTarget{microservice, sli, target, window, error_budget}`; Object Type `EligibilityVerdict{microservice, sha, environment, verdict, snapshot}`; Object Type `ReleasePointer{microservice, environment, current_sha, prior_sha}`.
- **Ontology reads**: Object Type `Microservice` (catalog) — for enumerating components requiring SLO coverage.

### Positive

- Every fast-forward to `staging` and `production` is conditional on hyperscaler-grade SLO evidence. No code reaches production within the ≤1 h auto-promotion window without surviving a multi-window burn-rate check.
- The observability substrate is industry-conventional and Apache-2.0; no vendor lock; full local control of the gate's read endpoint.
- Per-component release pointers decouple healthy µservices from a regressing neighbour. One bad ship holds itself, not the tree.
- Event-driven promotion responds to signal in ≤1 minute (the evaluator's cadence) rather than ≤30 minutes (cron cadence).
- The eligibility ledger is the agentic-system's source of truth for "did we ship?" — append-only, signed, queryable by both humans and downstream agents.
- Automated rollback on production breach removes the human-in-loop dependency that current pipelines rely on.

### Negative

- Operational ownership of the Grafana stack: oyatie now runs Prometheus, Mimir, Loki, Tempo, Pyroscope, Grafana, Alertmanager, Alloy, and Grafana OnCall. Capacity planning, upgrade discipline, and storage retention are now in scope. Mitigated by the maturity of the OSS components (all decade-old or built by Grafana Labs ~500-engineer team).
- Canary cohort weighting requires every deployable µservice to expose a traffic-split capability. The first migration touches every service-mesh integration; PRDs that lack a traffic-split capability block this gate's coverage for that component.
- SLO targets per µservice must be authored before the gate covers that µservice. Bootstrap order: as each µservice migrates to `microservices/<ms>/` per ADR-0131, it brings its OpenSLO manifest. Until a µservice has an OpenSLO manifest, the gate treats its eligibility as `held` (fail-closed) and the µservice cannot promote past `dev`. This is *intentional*: it forces the SLO-authoring discipline.
- Per-component release pointer proliferation interacts with GitHub branch-protection's per-rule cap on matched refs. Mitigation: pattern-based protection rules (`release/*/staging`, `release/*/production`). Validated in IP-006.
- One-time cost: every existing promotion flow that relies on the tree-wide `staging` and `production` refs updates to consume the per-component pointers. Owned by IP-011.

### Operational

- **New CI lane**: `oya-vcs-promotion-readiness` — added to `.github/branch-protection.yaml` required_status_checks on `dev` and `staging`. Reads `registry/promotion-eligibility.jsonl`; refuses fast-forward unless the latest record for every component touched by the source SHA is `eligible` for the target environment.
- **Decommission** of the FUTURE-marked stubs in `promote-dev-to-staging.yml` and `promote-staging-to-production.yml` (references to `oya-governance-canary-cohort-observability` and `oya-governance-full-rollout-observability` lanes are replaced by the concrete Layer-B components).
- **Oya VCS primitives** per ADR-0116: every changeset in this work uses `oya vcs claim/verify/done/promote`, never the retired grit/icm/rtk/vox CLIs.
- **Promotion review evidence**: IPs merging under this ADR attach typed cloud-ci/oya-ci quality-gate packets and PR Code Review reviewer-agent verdicts; existing `microservices/observability/evidence/multispectrum/**` and `/specs/multispectrum-review.json` references are historical only and must not be used as current coverage or merge evidence.

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Affected | 7 new crates added under `microservices/observability/crates/`; layer declared per BNF v4.1 §"Naming justification" above. |
| `cross-product-refusal` (LEAN-A2) | Not affected | observability is a substrate µservice; it is consumed by other µservices via Workflow events / Ontology reads, never imported directly. |
| `port-location` | Affected | New port traits in `oya-observability-slo-engine-kernel`: `SloTargetRepository`, `BurnRateEvaluator`, `EligibilityLedgerWriter`, `PrometheusClient` (port — adapter impl in `-adapter`). |
| `layer-correctness` | Affected | 7 new layers asserted per ADR-0105 13-value enum (kernel, domain, application, adapter, rest, worker, app). |
| `composition-root-only` | Affected | New composition-root binary in `-app` layer wires the worker, the REST surface, and the adapter clients. |
| `per-microservice-layout` (per ADR-0131) | Affected | New µservice `microservices/observability/`; layout enforced by the lane introduced in ADR-0131. |
| `statelessness` | Affected | The worker carries no persistent in-process state; it reads from Prometheus and writes to the JSONL ledger. Shardability is via per-component partitioning. |
| `shardability` | Affected | Ledger partitions by `microservice`; evaluator can shard by microservice or by SLO target without coordination. |

Port traits introduced (live in `oya-observability-slo-engine-kernel`; implementations in `-adapter`):

```rust
// microservices/observability/crates/oya-observability-slo-engine-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait SloTargetRepository: Send + Sync + sealed::Sealed {
    async fn load_for_microservice(&self, ms: &str) -> Result<Vec<SloTarget>, BoxError>;
}

#[async_trait::async_trait]
pub trait PrometheusClient: Send + Sync + sealed::Sealed {
    async fn instant_query(&self, promql: &str) -> Result<InstantVector, BoxError>;
    async fn range_query(&self, promql: &str, window: Window) -> Result<RangeVector, BoxError>;
}

#[async_trait::async_trait]
pub trait BurnRateEvaluator: Send + Sync + sealed::Sealed {
    async fn evaluate(&self, target: &SloTarget, env: Environment, sha: &Sha) -> Result<EligibilityVerdict, BoxError>;
}

#[async_trait::async_trait]
pub trait EligibilityLedgerWriter: Send + Sync + sealed::Sealed {
    async fn append(&self, record: EligibilityRecord) -> Result<(), BoxError>;
}
```

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Long-lived evaluator service (preferred, hyperscaler-canonical) vs. scheduled GitHub Action against Prometheus snapshots (bootstrap-cheaper) — final landing decision before IP-008 merges. | axis-observability | ADR-#### successor-IP or resolved in IP-008. |
| 2 | Where does the Layer-A cluster run (single Kubernetes cluster alongside staging/prod workloads, or dedicated observability cluster)? | ops-sre-reliability | resolved in IP-001 IaC choice. |
| 3 | Bootstrap order: gate lights up incrementally (each microservice brings its OpenSLO at migration time) vs. gate lands last and retroactively gates. (a) is the hyperscaler answer; (b) is the simpler integration. | council-architecture | resolved in PHASE-01 spec. |

## Verification

- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout` — exit 0; `microservices/observability/` conforms.
- `cargo run -p oya-dev-cli -- gate validate vcs-promotion-readiness --sha <test-sha> --env staging` — exit 0 when eligibility ledger has `eligible` for all components touched by the SHA.
- `cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice observability` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate port-location --microservice observability` — exit 0.
- `cargo nextest run -p oya-observability-slo-engine-domain` — exit 0.
- `cargo nextest run -p oya-observability-slo-engine-application` — exit 0; burn-rate math verified against Google SRE Workbook reference values.
- E2E rollback drill: induce a synthetic burn-rate breach on a canary cohort; verify production ref reverts within 1 minute; verify `rollback` record appended to ledger; verify Grafana OnCall incident raised.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0; this ADR registered in the hyperscaler-gates registry under HG-OBS.

## Amendment — SLO home convention under capability-first reorg (2026-06-17, doctrine-fix)

The capability-first repo reorganization (ADR-0562) re-homes crates from `{cloud,oya}/...` to
`<capability>/{core,ports,adapters,facade}/...`. Promotion-gating SLOs are NOT crates; this
amendment records WHERE they live post-reorg so a capability move never strands them at a dead
stem (one of the two HIGH cloud-native audit findings the doctrine-fix closes).

**Convention (decided):** an SLO's home is **capability-rooted** at
`<capability>/observability/slos/*.openslo.yaml`. Rationale: an SLO is per-SERVICE /
per-promotion-unit, and post-reorg the **capability** is the ADR-0139 "component" /
release-pointer unit. `<capability>/observability/` is a named, OWNERS-scoped sub-tree that is
deliberately NOT under a `core/ports/adapters/facade` face fold — it carries non-crate DATA, not
a crate. A multi-facade capability MAY later refine to
`<capability>/observability/slos/<facade>/` without changing the discovery root.

**Discovery root (forward-looking, not-yet-implemented):** the `SloTargetRepository` adapter
named in this ADR is NOT yet implemented — only the kernel domain types exist (in the
observability core-domain crate's `slo.rs` module). When it lands, its discovery root MUST derive
from the capability registry's `absorbs_current_dirs` facet (the closed `capability-registry.json`
spec is the single source of truth mapping each capability to the legacy `{cloud,oya}/...` dirs it
absorbed), so the resolver enumerates `<capability>/observability/slos/` per registered capability
rather than hard-coding paths. Until then this convention is uncommitted-in-code and cheap to set
now.

**Mechanical co-move (implemented in PR-A, engine only):** the reorg codemod
(`tools/oya-reorg-codemod-app/`) gained an `ArtifactMove { old_path, new_path }` capability — a
content-preserving wholesale `git mv` of NON-crate capability artifacts (SLO dirs/files, catalog
records) that travel WITH a capability move but carry no cargo/buck/rust identifiers to rewrite
(ADR-0563 §C2 content-preserving mover). It is INERT for existing plans: a `MovePlan` with no
`artifacts` behaves byte-identically to before, and the move-manifest is byte-identical when no
artifacts are present. The actual SLO backfill (relocating the orphaned stems to
`<capability>/observability/slos/`) and the catalog re-key are a SEPARATE follow-up (PR-B); this
amendment + PR-A only establish the convention and the engine capability — NO SLOs are moved here.

## Amendment — SLO backfill executed for the moved capabilities (2026-06-18, doctrine-fix PR-B)

PR-B executes the backfill the PR-A amendment deferred: it relocates the orphaned
`slos/*.openslo.yaml` of the already-moved capabilities from their dead legacy
`{cloud,oya}/<service>/slos/` stems to the capability-rooted home
`<capability>/observability/slos/` established above. The relocation is performed by the PR-A
codemod `ArtifactMove` co-move (content-preserving wholesale `git mv`, no in-file rewrite), driven
by a single committed artifact-only move plan.

**Born record (verbatim path, ADR-0562 §10.x justification):** the committed plan is
`specs/reorg/slo-catalog-backfill-move-plan.json`. It carries `moves: []` (zero crate moves) and
`artifacts: [...]` (the SLO old→new path pairs). The move-manifest it regenerates
(`specs/reorg/move-manifest.generated.json`) carries the file pairs so the ADR-0563 path-keyed
relabel + the total-accounting baseline follow the relocated SLOs old→new (a baselined SLO row
relabels to its new path, it is not read as new debt).

**Engine note:** `MovePlan::validate` returned `EmptyPlan` when `moves` alone was empty, which
blocked the artifact-only plan shape this PR needs; it now fails-closed only when BOTH `moves` AND
`artifacts` are empty (the genuine no-op). An artifact-only plan is the canonical PR-B backfill
shape.

**Workspace note:** each `<capability>/observability` SLO-data subtree is added to the root
`[workspace].exclude` (Cargo.toml) — the `<capability>/*/*` member glob would otherwise match
`<capability>/observability/slos` (a non-crate dir with no `Cargo.toml`) and make cargo error, the
same class as the existing buck2-only-gate exclude. OWNERS coverage is breadth-unlimited
(ADR-0555), so the relocated SLOs stay owner-covered.

**Relocated capability SLO homes (this PR; verbatim discovery roots):**
`iac/observability/slos/`, `observability/observability/slos/`, `storage/observability/slos/`,
`cell/observability/slos/`, `gateway/observability/slos/`, `flags/observability/slos/`. `compute`
and `messaging` had zero orphaned SLOs; `marketplace` is excluded (its crate move has not landed
in the workspace globs). Cross-service SLO-name collisions (only `autosharding-events.openslo.yaml`,
which differs per service) were resolved by source-service prefix; `MovePlan::validate`
dup-`new_path` fail-closed is the backstop. The catalog-record re-key remains a sequenced
follow-up (the consolidated capabilities are a many-to-fewer mapping with no authoritative in-tree
old→new manifest).

## Amendment — catalog-liveness gate enforces live-OR-explicitly-marked (2026-06-18, doctrine-fix PR-C3)

PR-C3 makes the founder **live-OR-explicitly-marked** catalog policy mechanically enforced, the
capstone of the doctrine-fix phase. Every `registry/catalog/<stem>.yaml` record is admissible iff
its stem is a LIVE workspace crate-id OR it carries an explicit non-live marker
(`status: retired-compatibility-row-no-crate` / `designed-ahead-row-no-crate` / `planned` /
`aspirational`, or a `non_claims` entry stating no matching crate exists). PR-C1+PR-C2 brought the
silently-stale set to zero; PR-C3 marks the last residual record
(`registry/catalog/oya-cloud-dcops-domain.yaml`, whose crate consolidated into the de-branded
`compute-dcops`) so the gate is born-blocking with an EMPTY frozen baseline — zero accepted debt.

**New gate (born-blocking, pure evaluator):** `ci/facade/service-catalog-parity/`
mirrors the `oya-cloud-ci-slo-coverage-app` pattern — the producer
(`oya-cloud-ci-accounting-registry-app`) owns all I/O, resolves the LIVE workspace crate-id
universe IN-PROCESS via `libs/oya-workspace-members-kernel` (NEVER a `cargo metadata` / `buck2`
shell-out — all-CLI-retirement + hermetic) by reading each resolved member's `[package].name`, and
emits `{"rows":[{"crate_id","source_path","is_live","marker"}]}`; the gate's pure `evaluate_keyed`
applies only the boolean live-OR-marked policy. The de-brand path-as-namespace means the catalog
crate_id matches the crate's `[package].name` (e.g. `compute/core/domain` → `compute-domain`), not
the directory basename. Registered as `[[gates.enabled]] id="cloud-ci-catalog-liveness"` in
`oya-ci.toml` with a `[catalog_liveness]` policy block reusing `catalog_record_globs` (born
pack-shaped, policy-as-data); disposition codes are `frozen_empty` so any future silently-stale
record is NEW debt the firewall blocks (it cannot be laundered into the baseline by regeneration).

**Born record (verbatim paths, ADR-0555 total-accounting justification — same path-mention pattern
ADR-0527 used for the slo-coverage gate so the new gate's source files are born JUSTIFIED, not new
unjustified debt):** the new gate crate is
`ci/facade/service-catalog-parity/Cargo.toml`,
`ci/facade/service-catalog-parity/BUCK`,
`ci/facade/service-catalog-parity/OWNERS`,
`ci/facade/service-catalog-parity/src/lib.rs`, and
`ci/facade/service-catalog-parity/tests/catalog_liveness.rs`.

**SLO-coverage composition:** `oya-cloud-ci-slo-coverage-app` is tightened to additionally require
each row's crate_id be live-OR-marked (`slo_row_no_live_crate_unmarked`, also `frozen_empty`),
closing the same false-green at the SLO surface — a valid `slo:` no longer excuses a stale record.
The SLO home convention remains `<capability>/observability/slos/` per the PR-A/PR-B amendments
above; the catalog-liveness predicate is the catalog→live half of the truth-down (the inverse
live→catalog completeness gap is sequenced backlog).

## Amendment — first doctrine-clean in-move SLO co-move (2026-06-18, ADR-0562 §10.14 compliance move)

The `compliance` capability strangler move (ADR-0562 §10.14) is the FIRST capability move to co-move
its promotion-gating SLOs IN the same move rather than deferring them to a PR-B-style backfill. The
upgraded move protocol folds the SLO co-move into the crate move: the move-plan
`specs/reorg/compliance-move-plan.json` carries one `ArtifactMove`
(`oya/compliance/slos` → `compliance/observability/slos`) alongside the seven crate moves, and the
thirteen `*.openslo.yaml` are relocated content-preserving (wholesale `git mv`, no in-file rewrite) to
the capability-rooted home `compliance/observability/slos/` established by the PR-A convention. The
`compliance/observability` SLO-data subtree is added to the root `[workspace].exclude` (the
`compliance/*/*` member glob would otherwise match the non-crate `compliance/observability/slos` dir);
OWNERS coverage is breadth-unlimited (ADR-0555), so the relocated SLOs are owner-covered by
`compliance/OWNERS`. This makes the convention's intended steady state — SLOs home WITH their
capability — the per-move default; the PR-B backfill remains the one-time catch-up for the nine
capabilities homed before the convention existed.

## References

- ADR-0041: GitOps trunk-based + release-branch cut at tag (precedes; this ADR extends with per-component pointers).
- ADR-0056: BNF v4.1 (naming authority for the 7 new crates).
- ADR-0105: 13-layer enum (layer authority).
- ADR-0110: ChangeSet state machine (each IP is one ChangeSet).
- ADR-0111: Merge-queue projected state fix-at-any-stage.
- ADR-0112: Webhook-driven Foundry agent invocation (precedent for event-driven workflows).
- ADR-0113: VCS orchestrator end-to-end.
- ADR-0114: Canary observability rollback (precedent; this ADR is its concrete implementation).
- ADR-0116: Retire external agent-coordination tooling (oya vcs primitives used throughout).
- ADR-0123: Hyperscaler maturity claim gate (HG-OBS gate registers here).
- ADR-0131: Per-microservice flat layout (this ADR's artifacts ship under that convention).
- ADR-0562: Capability-first repo organization + closed capability registry (the reorg the SLO-home convention amendment serves; the registry's `absorbs_current_dirs` facet is the discovery-root source of truth).
- ADR-0563: Rename-aware path-keyed CI baseline relabel (§C2 content-preserving mover; the reorg codemod's `ArtifactMove` co-move is content-preserving in that sense).
- Google SRE Workbook, ch. 5 "Alerting on SLOs" — multi-window multi-burn-rate canonical model. Betsy Beyer et al., O'Reilly 2018.
- OpenSLO spec — `openslo.com`, version 1.0.
- OpenTelemetry semantic conventions — `opentelemetry.io`.
- Grafana Labs LGTM stack documentation — `grafana.com`.
- AWS Managed Service for Prometheus and Managed Grafana — `aws.amazon.com/prometheus`, `aws.amazon.com/grafana`.
- GCP Managed Service for Prometheus — `cloud.google.com/managed-prometheus`.
- `feedback_quality_performance_scalability_bar.md` — hyperscaler bar.
- `feedback_no_silent_regression.md` — Linus-style regression discipline.
- `feedback_branch_pipeline_implemented.md` — current pipeline baseline.
- Issues: scaffold branch `oya-microservice-flat-layout-buildout-2026-05-17` (PR opened against `dev` per CLAUDE.md Wave-B bootstrap; ADR-0116 explains the temporary seam).
