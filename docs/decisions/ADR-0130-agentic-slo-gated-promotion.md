---
id: ADR-0130
status: Accepted
deciders: council-architecture, ops-sre-reliability, ops-security, axis-foundry, axis-observability
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: []
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

# ADR-0130: Agentic SLO-gated promotion

## Status

Accepted — 2026-05-17.

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
12. **Promotion-eligibility ledger — Mimir-native (recording rules ARE the ledger)** — the burn-rate evaluator emits eligibility verdicts as Prometheus metrics (`oya_promotion_eligibility_verdict`, `oya_promotion_burn_rate_*`, `oya_promotion_release_pointer_*`) into Mimir; recording rules compute the aggregate "is this SHA eligible across every microservice it touches" view. The ledger is the Mimir time-series store (object-storage-backed, 90d hot + 2y cold). Query API is PromQL via Mimir's HTTP endpoints. **No git-tracked JSONL ledger.** Per-changeset immutable audit row lives at `microservices/<ms>/evidence/multispectrum/<change_id>-<unix_ts>.json` per docs/AGENTS.md §changeset; that carries the burn-rate snapshot at promotion time as the archived view. Industry-canonical: Google Borgmon → Monarch and AWS CloudWatch SLO both store SLO verdicts as time-series, not as git-tracked files. See `/specs/agentic-slo-gated-promotion.json` §"promotion_eligibility_ledger" for the metric and recording-rule schema.
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

This ADR introduces typed events and Ontology writes per `feedback_workflow_objectgraph_adapter_layer.md`:

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
- **Decommission** of the FUTURE-marked stubs in `promote-dev-to-staging.yml` and `promote-staging-to-production.yml` (references to `oya-foundry-fitness-canary-cohort-observability` and `oya-foundry-fitness-full-rollout-observability` lanes are replaced by the concrete Layer-B components).
- **Oya VCS primitives** per ADR-0116: every changeset in this work uses `oya vcs claim/verify/done/promote`, never the retired grit/icm/rtk/vox CLIs.
- **Multispectrum evidence** per docs/AGENTS.md §changeset: each IP merging under this ADR emits `microservices/observability/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0.

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
| 1 | Long-lived evaluator service (preferred, hyperscaler-canonical) vs. scheduled GitHub Action against Prometheus snapshots (bootstrap-cheaper) — final landing decision before IP-008 merges. | axis-observability | ADR-NNNN successor-IP or resolved in IP-008. |
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
