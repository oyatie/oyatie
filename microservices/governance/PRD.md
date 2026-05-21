---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-governance
microservice: governance
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0106
  - ADR-0110
  - ADR-0123
  - ADR-0139
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
related_specs: [/specs/per-microservice-flat-layout.json, /specs/industry-best-practice-conformance.json, /specs/hyperscaler-gates.json]
date: 2026-05-17
owner_team: axis-foundry
doc_status: published
---

# PRD-governance: CI-Fitness Substrate + Industry-Best-Practice Conformance Engine

## Purpose

The `governance` microservice is oyatie's substrate for **CI-fitness checks, policy enforcement, evidence emission, and continuous industry-best-practice conformance auditing**. It bundles all ~50 historical `oya-check-*` crates per ADR-0131 §"governance" + ADR-0132 (product-suite-and-bundle dissolution) into a single µservice with four bounded contexts (`lane-runtime`, `policy-engine`, `evidence-emitter`, `aggregation-indexer`).

This µservice is the **enforcement origin** of the 6-axis program defined by ADR-0133 (industry-best-practice + hyperscaler-grade conformance) and the **execution origin** of every CI fitness lane that gates every other oyatie µservice's pull requests. A compromise here cascades to every µservice; a regression here weakens every other µservice's quality bar.

This µservice is **shared substrate**, not a hero product. It is consumed by every other oyatie µservice (each PR runs through governance lanes before admission to `dev`) and consumed by tenants only indirectly via the conformance posture published on the public-status surface. Its existence is the precondition for oyatie's "hyperscaler-grade in every practice" bar per `feedback_quality_performance_scalability_bar.md`.

This µservice has no Bominal equivalent and originates in oyatie. The historical `oya-governance-*` working name retires here; the canonical name is `governance` per ADR-0131 §"Migration DAG → IP-M01-MIGR-014" + ADR-0132.

## Tenant Value

- **Tenant Outcome 1 — Auditable quality posture.** Every PR run produces signed, replayable evidence; tenants can request their µservice's evidence trail for SOC 2 Type 2 + ISO 27001 + GDPR audits without ad-hoc engineering work.
- **Tenant Outcome 2 — Industry-benchmarked conformance.** Tenants see the per-axis conformance score (pipeline / directory / naming / standards / practices / policies) at the public-status surface; conformance is sourced from named industry baselines (SLSA, NIST SSDF, OWASP ASVS, Google SRE, AWS Well-Architected, Azure WAF), not internal opinion.
- **Internal Outcome 3 — Substrate uniformity.** Every oyatie µservice is gated by the same ~50 fitness lanes; no per-team divergence in what "production-ready" means at the PR level.
- **Internal Outcome 4 — Aggregation-index source-of-truth integrity.** Per-µservice catalog/SLO/spec/PRD files are the canonical source; central indices (`docs/prds/INDEX.md`, `registry/catalog/`, `/specs/microservices/`) regenerate deterministically; hand-edits refused at PR-time.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | PR author (agent or human) | every PR to run the full ~50 fitness lane suite | regressions on any axis are blocked at PR-time | lane-runtime | Must |
| FR-02 | governance lane | to read the canonical industry-baseline pin in `/specs/industry-best-practice-conformance.json` | findings cite a named source per ADR-0133 | policy-engine | Must |
| FR-03 | governance lane | to emit a signed Finding for every violation | each violation is auditable and replayable | evidence-emitter | Must |
| FR-04 | governance lane | to read every µservice's `microservices/<ms>/{PRD.md,catalog/**,slos/**,policy/**,contracts/**,specs/**}` | per-µservice authority is honoured | lane-runtime | Must |
| FR-05 | aggregation indexer | to regenerate `docs/prds/INDEX.md`, `registry/catalog/<crate>.yaml`, `/specs/microservices/<product>.json` from per-µservice sources | central indices are never hand-edited | aggregation-indexer | Must |
| FR-06 | merge queue | to query "is PR #N admissible against `dev`?" against the latest verdict | admission decisions are gate-driven, not human-driven | policy-engine | Must |
| FR-07 | auditor (external; SOC 2 / ISO 27001) | to query findings + replay evidence for a date range scoped to a µservice | audit preparation completes without ad-hoc tooling | evidence-emitter | Must |
| FR-08 | quarterly refresh | to fetch current industry baselines (SLSA, NIST SSDF, OpenSLO, OpenTelemetry, OWASP ASVS) + diff against pinned baselines + open successor-IP PRs | baselines never silently drift | policy-engine | Must |
| FR-09 | per-µservice CODEOWNERS lane | to refuse a PR that adds a new crate without an authoring RACI override | ownership is explicit per ADR-0123 | policy-engine | Must |
| FR-10 | aggregation-index lane | to refuse PRs that hand-edit central indices | per-µservice source-of-truth honoured | aggregation-indexer | Must |
| FR-11 | hyperscaler-maturity-claim-gate | to refuse marketing claims at `oya-check-hyperscaler-maturity-claims` lane unless cited against an industry baseline | sales surface cannot drift from reality | policy-engine | Must |
| FR-12 | retired-vocabulary lane | to refuse PRs reintroducing retired terms (`platform`, `object-graph`, `application` for new crates) | terminology drift refused at PR-time | policy-engine | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Per-PR fitness suite execution (full ~50 lanes, single PR) | ≤15s | ≤60s | ≤90s | parallel-fanout across GitHub Actions matrix |
| Single-lane execution (any individual lane) | ≤3s | ≤10s | ≤30s | matrix entry timeout |
| Aggregation-index regeneration (full repo) | ≤60s | ≤5min | ≤10min | invoked on every PR + scheduled cron |
| Finding emission latency (lane fail → Postgres write → audit-chain seal) | ≤200ms | ≤1s | ≤3s | end-to-end |
| Industry-baseline diff (quarterly refresh) | ≤2min | ≤10min | — | external HTTPS fetch + JSON diff |
| Admission-gate verdict query (merge queue) | ≤50ms | ≤200ms | ≤500ms | Postgres-backed |
| Evidence replay (single PR, full evidence) | ≤500ms | ≤2s | ≤5s | object-storage backed |

### Security

- All Finding writes are signed by the lane runner's per-environment Ed25519 signing key (per Bominal ADR-0028 audit-chain posture).
- Lane bypass refused at infrastructure layer: GitHub branch-protection `required_status_checks` enforces; no admin-merge without break-glass procedure logged in `runbooks/lane-bypass-emergency.md`.
- Evidence writes are append-only + content-addressed (SHA256 of canonical-JSON) per ADR-0028.
- Aggregation-index regeneration is hermetic: input = per-µservice files only; no network egress during regen.
- Secrets (GitHub PAT, Postgres password, S3 keys) follow OpenBao SecretReference pattern (per `feedback_openbao_secrets.md` 2026-05-12 directive); raw secrets never enter the repo, chat, or checkpoints.

### Audit + Compliance

- Every `LaneFailed`, `FindingEmitted`, `AuditCompleted`, `BaselinePinUpdated`, `AggregationIndexRegenerated` event emits an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028).
- The `registry/fixuptasks.jsonl` and `evidence/pr-reviews/` paths are append-only; union-merged across concurrent agent commits via the existing `.gitattributes` driver.
- Audit-chain seal latency ≤1s per `(tenant, period)`.
- Per-axis quarterly refresh report at `evidence/audits/industry-best-practice-conformance/<quarter>.json`.

### Availability + SLO

- Availability target: 99.95% monthly for the lane-runtime's per-PR gate decision path (the gate must be available even when the µservice it gates is degraded).
- Aggregation-indexer availability target: 99.9% monthly (lower; non-real-time path).
- RTO: ≤15 min. RPO: ≤60 s (single PR re-run).

### Data residency

- Findings and per-PR metadata inherit the originating PR-author's `jurisdiction_code` per ADR-0117. Per-pack overlays at `iac/kustomize/overlays/pack-<pack>/`.

### DR posture

| Field | Value |
|---|---|
| ADR | ADR-0343 |
| Target | Lane-runtime gate decision path RTO 900 s and RPO 60 s, matching `manifest.json#dr`. |
| Compliance-pack floor | HIPAA floor RTO 3600 s / RPO 300 s, SOC2-T2 floor RTO 14400 s / RPO 900 s, ISO27001 floor RTO 14400 s / RPO 3600 s; governance keeps the stricter 900 s / 60 s target. |
| Failover runbook | `runbooks/evidence-replay.md`, matching `manifest.json#dr.failover_runbook`; `runbooks/cedar-policy-rollback-protocol.md` remains the gate-decision rollback branch. |
| Multi-region active-active | Yes, matching `manifest.json#dr.multi_region_active_active=true`; evidence and lane state remain pack-local under the active-active-multi-AZ-cross-region-warm replication shape. |
| WHY | Governance must fail closed enough to protect the merge queue, but fast enough to restore PR admission and evidence replay before engineers bypass the quality bar. |

### Capacity model

| Field | Value |
|---|---|
| ADR | ADR-0340, with pod runtime tier declared by ADR-0338. |
| Per-tenant baseline | `manifest.json#capacity_model`: 0.20 vCPU, 384 MiB RAM, 6 GB storage, and connections `{valkey: 2, postgres: 4, outbound_http: 10}` per tenant/workflow source. Median PR load from `capacity-model.md`: about 50 lanes per PR, 100 Postgres inserts per PR, 0-2 findings, and 4 KB evidence per finding. |
| Scaling dimension | `per_workflow_run`, matching `manifest.json#capacity_model.scaling_dimension`; admission verdict and replay queries remain request-shaped inside that workflow envelope. |
| Cell placement class | Tier-1 per `manifest.json#capacity_model.cell_placement_class`; runtime tier is ADR-0338 Tier-1 because `manifest.json#pod_runtime_tier=1`, with runner sandboxes upgraded to Tier-0 if a lane executes tenant-supplied test fixtures. |
| Autoscaling boundaries | ARC runner pool min 8, max 200; lane-runtime worker min 2, max 10; policy-engine worker min 2, max 10; evidence-emitter worker min 2, max 10; aggregation-indexer worker min 2, max 5. |
| WHY | The model absorbs PR bursts and evidence replay without letting one service's PR storm starve the gate decisions for the rest of the fleet. |

### Sustainability + cost attribution

| Field | Value |
|---|---|
| ADR | ADR-0344 |
| Per-call emission claim | Every `LaneFailed`, `FindingEmitted`, `AuditCompleted`, `BaselinePinUpdated`, and `AggregationIndexRegenerated` audit row emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region`. |
| Carbon-aware routing | No for live merge-gate verdicts and security-blocking findings. Yes for quarterly baseline refresh, replay backfills, and aggregation-index regeneration when the merge queue is not waiting. |
| Tenant transparency surface | The public conformance/status surface shows evidence volume and lane usage; the FinOps portal allocates governance cost by tenant, microservice, lane, cell, and compliance pack. |
| WHY | CSRD, SB-253, and SEC climate-disclosure reporting need evidence-generation cost and emissions to be attributable, while live security gates must not wait for low-carbon capacity. |

### API versioning posture

| Field | Value |
|---|---|
| ADR | ADR-0342 |
| Public API version model | Date carrier triplet: `Oyatie-Version: YYYY-MM-DD`, `/v/YYYY-MM-DD/...` for public REST/status endpoints, and proto3 `oyatie_version`. |
| SDK semver model | Governance SDKs use `major.minor.patch`; rule-pack and lane behavior pins remain date-versioned. |
| Support window | Last N=3 public versions supported for >=180 days. |
| Per-tenant pinning | Yes for evidence-query, conformance-posture, and auditor replay APIs; merge-queue internals are pinned by lane registry version instead. |
| Internal-mesh exemption | Yes. ADR-0145 direct gRPC between governance, observability, and audit-chain remains exempt from public URL date prefixes. |

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), layers used by this µservice are: `kernel`, `domain`, `usecase`, `api`, `adapter`, `rest`, `worker`, `sdk`, `app`. Plus the ~50 historical `oya-check-*` crates which migrate into `microservices/governance/src/crates/` under their existing names during M01 per ADR-0131 §"Migration DAG → IP-M01-MIGR-014".

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `lane-runtime` | `oya-governance-lane-runtime-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Executes any fitness lane against a target ref/PR/µservice. Hosts the matrix-fanout, lane registry, and timeout/retry policy. | `LaneId`, `LaneRun`, `LaneRequest`, `LaneVerdict`, `RunnerProfile` |
| `policy-engine` | `oya-governance-policy-engine-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Owns the ~50 check rule-sets (data-class, license, supply-chain, glossary, perf-budget, etc.) + the industry-baseline pin. Pure decision logic. | `Rule`, `RulePack`, `Severity`, `BaselineCitation`, `Verdict` |
| `evidence-emitter` | `oya-governance-evidence-emitter-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Signs + persists Findings; writes audit-chain records; serves replay queries. | `Finding`, `EvidenceRecord`, `AuditSeal`, `ReplayCursor` |
| `aggregation-indexer` | `oya-governance-aggregation-indexer-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}` | Reads per-µservice sources + regenerates central indices (`docs/prds/INDEX.md`, `registry/catalog/`, `/specs/microservices/`). Refuses hand-edits. | `IndexEntry`, `Aggregation`, `DivergenceReport` |

Plus the **~50 historical `oya-check-*` crates** (bundled per ADR-0131 IP-M01-MIGR-014); each is a single-purpose validator that registers itself with `lane-runtime` and emits `Finding`s through `evidence-emitter`. The full list (current as of 2026-05-17):

| Crate (migrates to `microservices/governance/src/crates/`) | Lane name | Severity | Migration tier |
|---|---|---|---|
| `oya-check-active-artifact-contract` | active-artifact-contract | BLOCKER | M01-A |
| `oya-check-adr-citation` | adr-citation | BLOCKER | M01-A |
| `oya-check-adr-index` | adr-index | BLOCKER | M01-A |
| `oya-check-authority-cohesion` | authority-cohesion | BLOCKER | M01-A |
| `oya-check-benchmark` | benchmark | WARN | M01-B |
| `oya-check-brand-residue` | brand-residue | BLOCKER | M01-A |
| `oya-check-cedar-fragment-coverage` | cedar-fragment-coverage | BLOCKER | M01-B |
| `oya-check-claim-ceiling` | claim-ceiling | BLOCKER | M01-A |
| `oya-check-codeowners-mirror` | codeowners-mirror | BLOCKER | M01-A |
| `oya-check-cohesion` | cohesion | BLOCKER | M01-A |
| `oya-check-cost-budget` | cost-budget | BLOCKER | M01-B |
| `oya-check-data-class` | data-class | BLOCKER | M01-A |
| `oya-check-doc-catalog` | doc-catalog | BLOCKER | M01-A |
| `oya-check-documentation-system` | documentation-system | BLOCKER | M01-A |
| `oya-check-glossary-coverage` | glossary-coverage | BLOCKER | M01-A |
| `oya-check-glossary-vocabulary` | glossary-vocabulary | BLOCKER | M01-A |
| `oya-check-license-policy` | license-policy | BLOCKER | M01-A |
| `oya-check-mobile-native` | mobile-native | WARN | M01-C |
| `oya-check-openapi-rest-route-parity` | openapi-rest-route-parity | BLOCKER | M01-B |
| `oya-check-perf-budget` | perf-budget | BLOCKER | M01-B |
| `oya-check-placeholder-debt` | placeholder-debt | BLOCKER | M01-A |
| `oya-check-pr-traceability` | pr-traceability | BLOCKER | M01-A |
| `oya-check-pre-push` | pre-push | BLOCKER | M01-A |
| `oya-check-protection-context-match` | protection-context-match | BLOCKER | M01-A |
| `oya-check-quality-lane` | quality-lane | BLOCKER | M01-A |
| `oya-check-raci-coverage` | raci-coverage | BLOCKER | M01-B |
| `oya-check-readme-coverage` | readme-coverage | BLOCKER | M01-B |
| `oya-check-release-pack` | release-pack | BLOCKER | M01-B |
| `oya-check-retired-vocabulary` | retired-vocabulary | BLOCKER | M01-A |
| `oya-check-runbook-freshness` | runbook-freshness | BLOCKER | M01-B |
| `oya-check-runbook-index` | runbook-index | BLOCKER | M01-B |
| `oya-check-shardability` | shardability | BLOCKER | M01-A |
| `oya-check-slo-coverage` | slo-coverage | BLOCKER | M01-A |
| `oya-check-statelessness` | statelessness | BLOCKER | M01-A |
| `oya-check-supply-chain` | supply-chain | BLOCKER | M01-A |
| `oya-check-typescript-workspace` | typescript-workspace | WARN | M01-C |
| `oya-check-vendor-recency` | vendor-recency | WARN | M01-C |
| `oya-check-hyperscaler-maturity-claims` (planned per ADR-0123) | hyperscaler-maturity-claims | BLOCKER | M01-A |
| `oya-check-industry-best-practice-conformance` (planned per ADR-0133) | industry-best-practice-conformance | BLOCKER | M01-A |
| `oya-check-per-microservice-layout` (planned per ADR-0131) | per-microservice-layout | BLOCKER | M01-A |
| `oya-check-aggregation-index-generation` (planned per ADR-0131) | aggregation-index-generation | BLOCKER | M01-A |
| `oya-check-cross-ref-validity` (planned per ADR-0117) | cross-ref-validity | BLOCKER | M01-A |
| `oya-check-lean-a1` (dependency-direction) | lean-a1 | BLOCKER | M01-A |
| `oya-check-lean-a2` (cross-product-refusal) | lean-a2 | BLOCKER | M01-A |
| `oya-check-port-location` | port-location | BLOCKER | M01-A |
| `oya-check-layer-correctness` | layer-correctness | BLOCKER | M01-A |
| `oya-check-composition-root-only` | composition-root-only | BLOCKER | M01-A |
| `oya-check-sdk-kernel-only` | sdk-kernel-only | BLOCKER | M01-A |
| `oya-check-naming-bnf-v41` | naming-bnf-v41 | BLOCKER | M01-A |
| `oya-check-no-suite` (per ADR-0132) | no-suite | BLOCKER | M01-A |

Total bundled lane crates at the M01 launch tier: **~50** (37 existing in `crates/oya-check-*` + ~13 planned per related ADRs).

Naming justification — `lane-runtime`:

```
NAME: oya-governance-lane-runtime-<layer>
JUSTIFICATION:
- microservice = governance: ADR-0131 per-microservice folder; bundles ~50 oya-check-* crates per IP-M01-MIGR-014.
- bc-tokens = lane-runtime: primary BC for fitness-lane execution. ADR-0056 v4.1 BC-optionality
  rule honoured (sibling BCs policy-engine + evidence-emitter + aggregation-indexer exist,
  justifying explicit BC token).
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + entity types (LaneId, LaneRun, RunnerProfile). Zero I/O.
    Carries data_class annotations per Bominal ADR-0028 + oya-check-data-class lane.
  - domain: pure scheduling math, retry-budget arithmetic, matrix-fanout calculator.
  - usecase (per ADR-0106): orchestrators dispatching matrix jobs, collecting verdicts.
  - api: protocol-neutral typed I/O contracts.
  - adapter: protocol-neutral implementations of kernel ports.
  - rest: HTTP handler/route layer.
  - worker: long-lived continuous lane scheduler binary.
  - sdk: client library for tenant-side lane-status subscription.
  - app: composition root binary; wires worker + rest + adapter clients.
- exemptions claimed: none.
```

Naming justification — `policy-engine`:

```
NAME: oya-governance-policy-engine-<layer>
JUSTIFICATION:
- microservice = governance.
- bc-tokens = policy-engine: sibling BC for decision logic (read rule-set, decide verdict).
- layer = <layer>: 13-value canonical enum.
  - kernel: Rule, RulePack, Severity entities; zero I/O.
  - domain: rule-evaluation algebra (Cedar-style allow/forbid composition).
  - usecase: rule-pack loading + per-axis evaluator (the 6-axis ADR-0133 program).
  - api / adapter / rest / worker / sdk / app: standard layer split.
- exemptions claimed: none.
```

Naming justification — `evidence-emitter`:

```
NAME: oya-governance-evidence-emitter-<layer>
JUSTIFICATION:
- microservice = governance.
- bc-tokens = evidence-emitter: sibling BC for Finding persistence, audit-chain seal,
  replay-query serving.
- layer = <layer>: 13-value canonical enum.
  - kernel: Finding, EvidenceRecord, AuditSeal entities.
  - domain: canonical-JSON serialisation; Merkle-tree composition.
  - usecase: per-Finding emit + replay orchestrators.
  - api / adapter / rest / worker / sdk / app: standard.
- exemptions claimed: none.
```

Naming justification — `aggregation-indexer`:

```
NAME: oya-governance-aggregation-indexer-<layer>
JUSTIFICATION:
- microservice = governance.
- bc-tokens = aggregation-indexer: sibling BC for central-index regeneration.
- layer = <layer>: 13-value canonical enum.
  - kernel: IndexEntry, Aggregation, DivergenceReport entities.
  - domain: per-axis aggregation algebra; deterministic ordering rules.
  - usecase: PRD index + catalog + spec aggregations.
  - api / adapter / rest / worker / sdk / app: standard.
- exemptions claimed: none.
```

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|
| `lane-runtime` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `policy-engine` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `evidence-emitter` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `aggregation-indexer` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

Total **new** crates introduced by this µservice's umbrella BCs: **36** (4 BCs × 9 layers). Plus the **~50 historical `oya-check-*` crates** that migrate in (single-crate validators; do not subdivide into layers; ADR-0131 §"Migration DAG → IP-M01-MIGR-014" treats them as atomic per-lane crates kept under their existing names).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `LaneRegistry` | `oya-governance-lane-runtime-kernel` | `-adapter` (Cargo workspace introspection) | `INTERNAL_ONLY` |
| `LaneDispatcher` | `oya-governance-lane-runtime-kernel` | `-adapter` (GitHub Actions matrix invocation) | `INTERNAL_ONLY` |
| `RunnerProfileStore` | `oya-governance-lane-runtime-kernel` | `-adapter` (Postgres CRUD) | `INTERNAL_ONLY` |
| `RulePackRepository` | `oya-governance-policy-engine-kernel` | `-adapter` (TOML/YAML reader; baseline pin reader) | `INTERNAL_ONLY` |
| `BaselineDiffClient` | `oya-governance-policy-engine-kernel` | `-adapter` (HTTPS fetch + JSON diff against external industry sources) | `INTERNAL_ONLY` |
| `FindingPersistence` | `oya-governance-evidence-emitter-kernel` | `-adapter` (Postgres CRUD + S3 evidence write) | `AUDIT` |
| `AuditChainSealer` | `oya-governance-evidence-emitter-kernel` | `-adapter` (Ed25519 signer; OpenBao key lookup) | `AUDIT` |
| `ReplayQuery` | `oya-governance-evidence-emitter-kernel` | `-adapter` (S3 read + Postgres query) | `AUDIT` |
| `PerMicroserviceSourceReader` | `oya-governance-aggregation-indexer-kernel` | `-adapter` (filesystem walker; YAML/Markdown frontmatter parser) | `INTERNAL_ONLY` |
| `CentralIndexWriter` | `oya-governance-aggregation-indexer-kernel` | `-adapter` (Git-tracked file writer; refuses out-of-band hand-edits) | `INTERNAL_ONLY` |
| `DivergenceReporter` | `oya-governance-aggregation-indexer-kernel` | `-adapter` (diff renderer) | `INTERNAL_ONLY` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane (now under `microservices/governance/`) refuses unannotated fields at PR-time per `feedback_clean_architecture_requirements.md`.

Cross-product rule: `governance` MUST NOT import any product µservice crate at any layer (it reads file artifacts only). All cross-product flows go through Workflow (events) or Ontology (entity reads/writes). LEAN-A2 CI lane enforces — and `governance` is the lane that **runs** LEAN-A2 against every other µservice.

CI lanes that `governance` must run against itself (the self-application rule; the lane that gates governance is run from governance, so a bootstrap-paradox synthetic-probe fallback exists per ADR-0133 §"Operational"):

- `oya gate validate lean-a1 --microservice governance` — dependency-direction
- `oya gate validate lean-a2 --microservice governance` — cross-product-refusal
- `oya gate validate port-location --microservice governance` — ports in kernel
- `oya gate validate layer-correctness --microservice governance` — layer enum match
- `oya gate validate per-microservice-layout --microservice governance` — ADR-0131 conformance
- `oya gate validate statelessness --microservice governance`
- `oya gate validate shardability --microservice governance`
- `oya gate validate industry-best-practice-conformance --microservice governance`
- `oya gate validate authority-cohesion` — HG-GOV registers here

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `LaneFailed` | any lane verdict transitions to fail | `pr-review.yml`, audit-chain | admission-gate state machine |
| `FindingEmitted` | new Finding signed + persisted | audit-chain, `grafana-oncall` (if severity=BLOCKER) | — |
| `AuditCompleted` | full ~50-lane suite finishes for a PR | merge-queue admission gate | — |
| `BaselinePinUpdated` | quarterly refresh promotes new baseline pin | per-axis remediation IP generator | — |
| `AggregationIndexRegenerated` | aggregation-indexer rewrites central indices | downstream doc-publish pipeline | — |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `PullRequestOpened` | GitHub webhook → `tenancy` event bus | `lane-runtime` | enqueue full ~50-lane suite |
| `PushedToDefaultBranch` | GitHub webhook | `aggregation-indexer` | re-run aggregation; refuse if hand-edits detected |
| `MicroserviceRegistered` | `tenancy` (when a new µservice scaffolds) | `lane-runtime` | register µservice for per-PR fitness-lane execution |
| `OpenSLOManifestUpdated` | `observability` (manifest hot-reload) | `policy-engine` | trigger `slo-coverage` lane re-run |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Finding{microservice, sha, lane, severity, citation, evidence_uri}` | `finding_for→Microservice` | `evidence-emitter` | Ed25519 |
| `LaneRun{lane_id, sha, verdict, duration_ms, runner_profile}` | `run_of→Lane` | `lane-runtime` | Ed25519 |
| `BaselinePin{axis, source_url, pinned_sha, effective_at}` | `pin_for→Axis` | `policy-engine` | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Microservice` (catalog) | `lane-runtime`, `aggregation-indexer` | `filter(active=true)` to enumerate µservices requiring lane coverage |
| `SLOTarget` (per-µservice) | `policy-engine` | for `slo-coverage` lane to assert each µservice has ≥1 SLO |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| SonarQube | Code-quality + security gate | rule-pack management, per-PR gate, severity classification, finding replay | `docs.sonarsource.com/sonarqube/` |
| GitHub Advanced Security (CodeQL + Dependabot + Secret Scanning) | Built-in PR gate on security vulnerabilities | per-PR gate, finding emission, dismissal workflow | `docs.github.com/en/code-security/` |
| Snyk | Vulnerability + license scanning | per-PR gate, supply-chain, license-policy | `docs.snyk.io` |
| Polyspace (MathWorks) | Formal-methods static analysis | rule-pack, finding emission, severity | `mathworks.com/products/polyspace.html` |
| CodeClimate | Maintainability + test-coverage gate | per-PR gate, finding emission, dashboard | `docs.codeclimate.com` |
| Trivy (Aqua Security) | Container + IaC vulnerability scanner | supply-chain, IaC misconfig, SBOM | `aquasecurity.github.io/trivy/` |
| Open Policy Agent + Conftest | Policy-as-code on configs | rule evaluation, OPA Rego, IaC policy | `openpolicyagent.org` |
| Renovate | Dependency-recency policy | vendor-recency, baseline-pin refresh | `docs.renovatebot.com` |
| Backstage TechDocs | Per-service doc system | doc-coverage, runbook-index, readme-coverage | `backstage.io/docs/features/techdocs/` |

Key parity gaps to close (ordered by priority):

1. **6-axis continuous conformance audit** — none of the competitors offer a 6-axis (pipeline/directory/naming/standards/practices/policies) program with named industry baselines per ADR-0133. Target: continuous, baseline-cited, audit-replayable.
2. **Per-µservice flat-layout enforcement** — none enforce a per-µservice folder convention at PR-time. Target: refuse out-of-layout artifacts at PR-time.
3. **Aggregation-index source-of-truth** — none make central indices generated-only. Target: hand-edits refused; per-µservice files canonical.
4. **Agentic-dev-team optimisation** — competitors assume human-on-keyboard; the 8 ADR-0133 agentic principles are oyatie-original. Target: principles 1-8 enforced at scaffold time.
5. **Industry-baseline pin diff** — Renovate refreshes dependencies; none refresh policy baselines (SLSA, NIST SSDF, OpenSLO). Target: quarterly auto-PR with diff against pinned baselines.

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Per-PR fitness suite (full ~50 lanes) | ≤15s | ≤60s | ≤90s | parallel matrix |
| Single-lane execution | ≤3s | ≤10s | ≤30s | matrix entry |
| Aggregation-index regen (full repo) | ≤60s | ≤5min | ≤10min | scheduled + per-PR |
| Finding emission latency | ≤200ms | ≤1s | ≤3s | end-to-end seal |
| Admission-gate verdict query | ≤50ms | ≤200ms | ≤500ms | Postgres-backed |
| Evidence replay (single PR) | ≤500ms | ≤2s | ≤5s | S3-backed |
| Industry-baseline quarterly diff | ≤2min | ≤10min | — | HTTPS fetch |

Error budget:
- Monthly error budget for lane-runtime gate availability: 0.05% (≈22 min/month).
- Burn-rate alarm on the gate itself: 14.4× burn over 1 h triggers page.
- Error budget policy: `microservices/governance/runbooks/error-budget-policy.md` (Slice-B successor-IP).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `stateless | postgres | object-storage | persistent-volume | mixed` → **`mixed`**. Rationale: lane runners are stateless (re-derivable from a PR ref); Postgres carries lane-state + Finding metadata; S3 carries evidence blobs.

**Active-active compatibility**: `stateless-compatible` for lane runners; Postgres uses logical replication for read replicas; S3 is multi-AZ by construction.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Concurrent PR runs | 50 | 500 | runner-pool saturation > 80% |
| Lanes per PR run | ~50 (current); ~70 forecast at M03 | 120 | matrix limit |
| Findings/day | 10k | 1M | Postgres write-IO > 70% |
| Evidence blob size (median) | 4 KB | 1 MB | object-count > 100M per bucket |
| Aggregation regen cadence | per-PR + 15min cron | per-PR + 5min cron | divergence-detection lag > 5min |

Scale-out policy:
- Kubernetes HPA: lane-runner pods scale on CPU `>70%`; min 4 replicas, max 200 replicas.
- GitHub Actions runner pool: per-µservice matrix entry size cap; ARC autoscaling per `iac/helm/lane-runner-pool/values.yaml`.
- Postgres: read-replicas for `ReplayQuery`; primary-only writes; per-Bominal-ADR-0019 vertical-then-horizontal posture.
- Pre-warmed pool: 8 standby lane-runner pods; cold-start budget ≤500 ms.

Cross-region story:
- M01 launch: single KR region (OCI ap-seoul-1); per-tenant residency locked per ADR-0117.
- Post-M01 expansion: read-replica Postgres per region; S3 cross-region replication; `multi-region.md` successor-IP.

Sharding:
- Finding table partitions by `microservice` + `month`; lane-run table partitions by `month`.
- `oya-check-shardability` CI lane verifies partition key presence on the governance µservice itself (self-application).

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | All ~50 historical `oya-check-*` crates migrated into `microservices/governance/src/crates/` (atomic per IP-M01-MIGR-014) | `find microservices/governance/src/crates/ -name 'oya-check-*' -type d \| wc -l` ≥ 37 |
| AC-02 | Full ~50-lane fitness suite passes on `dev` HEAD | `cargo run -p oya-dev-cli -- gate run --all` exit 0 |
| AC-03 | Single-lane execution p99 ≤ 10s on representative PR | timed e2e test under `microservices/governance/tests/perf/single-lane.rs` |
| AC-04 | Aggregation-indexer regenerates indices deterministically (idempotent across 3 runs) | `microservices/governance/tests/e2e/aggregation-determinism.rs` |
| AC-05 | Hand-edit of central index (`registry/catalog/<crate>.yaml`) refused at PR-time | branch-protection emulation test |
| AC-06 | New crate without owning µservice folder refused at PR-time | per-microservice-layout lane test |
| AC-07 | Industry-baseline quarterly refresh PR opens automatically | `cron` test against the refresh workflow |
| AC-08 | Finding replay returns canonical evidence for a date range in ≤2s | `microservices/governance/tests/perf/replay.rs` |
| AC-09 | `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice governance` exit 0 | ADR-0131 lane |
| AC-10 | `cargo run -p oya-dev-cli -- gate validate authority-cohesion` exit 0; HG-GOV registered | ADR-0123 lane |
| AC-11 | `cargo run -p oya-dev-cli -- gate validate industry-best-practice-conformance` exit 0 on governance itself | ADR-0133 self-application |
| AC-12 | Lane bypass via admin-merge requires break-glass record at `runbooks/lane-bypass-emergency.md` | branch-protection audit-log assertion |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Migration sequencing of the ~50 oya-check-* crates: atomic single-ChangeSet vs. tier-A/tier-B/tier-C waves? | council-architecture | resolved in IP-001..IP-015 (tier-A in IP-001..IP-010; tier-B in IP-011..IP-013; tier-C in IP-014..IP-015) |
| 2 | Should the historical `oya-check-*` crates rename to `oya-governance-check-*-{kernel,...}` during migration, or retain flat names? | council-architecture | retain flat names for M01 (per ADR-0131 §"Crate naming inside each `microservices/<ms>/crates/` subtree is unchanged"); rename ADR successor-IP subsequent-to-M01-completion |
| 3 | Bootstrap paradox: governance gates governance. Synthetic-probe fallback during cold-start? | axis-foundry | resolved per ADR-0133 §"Operational"; mirrors observability self-SLO fallback in microservices/observability/PRD.md Open Q4 |
| 4 | Per-µservice lane-subset selection (run only relevant lanes per PR) vs. full ~50 every time? | axis-foundry | full ~50 for M01 (deterministic posture); subset-selection ADR successor-IP subsequent-to-M01-completion |
| 5 | Quarterly refresh: PR-bot author identity (council-architecture vs. ops-finops vs. axis-foundry)? | ops-sre-reliability | resolved as `axis-foundry-bot` per `runbooks/industry-baseline-refresh.md` |
| 6 | Finding severity escalation policy: BLOCKER vs WARN vs INFO; does WARN-stacking promote to BLOCKER? | ops-security | M01 launch: strict severity (no escalation); ADR-#### successor-IP if signal-overload observed |
| 7 | External-auditor JIT scope: read-only Postgres replica vs. evidence-export tool? | ops-security | evidence-export tool (per `runbooks/evidence-replay.md`); read-only replica is overscoped |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application → usecase rename | layer-name authority |
| ADR-0110 | ChangeSet state machine | each IP is one ChangeSet |
| ADR-0123 | Hyperscaler maturity claim gate | HG-GOV registers here |
| ADR-0139 | Agentic SLO-gated promotion | governance lanes gate the SLO gate |
| ADR-0131 | Per-microservice flat layout | this PRD authored natively under it; IP-M01-MIGR-014 governs the ~50 crate migration |
| ADR-0132 | Product-suite-and-bundle dissolution | governance bundle decision |
| ADR-0133 | Industry-best-practice + hyperscaler-grade conformance | this µservice IMPLEMENTS the 6-axis program |

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
