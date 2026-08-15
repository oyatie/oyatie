---
doc_status: published
id: ADR-0704
title: "Live Kubernetes Go→Rust port engine and owned-kernel interfaces"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-0066, ADR-0093, ADR-0134, ADR-0142, ADR-0198, ADR-0208, ADR-0222, ADR-0241, ADR-0276, ADR-0394, ADR-0569, ADR-0611, ADR-0632, ADR-0637, ADR-0638]
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: []
milestone: W0
deliverables:
  - id: ADR-0704-D1
    description: "Live apex source-of-truth for topic k8s_port: Live Kubernetes Go→Rust port engine and owned-kernel interfaces."
    exit_criteria: "docs/decisions/ADR-0704-k8s-port-live-apex.md is Accepted with planning_impact true; member ADRs listed in supersedes are archived under docs/adr-archive/."
    verified_by: "oya-ci-required"
---
# ADR-0704: Live Kubernetes Go→Rust port engine and owned-kernel interfaces

## Status

**Accepted** — live consolidated source-of-truth entry for topic `k8s_port` (E5 2026-08-06).

## Context

Oyatie ADR corpus cleanup: agents must not treat every historical Accepted file as equal live law.
This apex consolidates **15** Accepted ADRs in the `k8s_port` topic. Member files are
**Superseded** by this apex and then archived; full text remains in git history.

Live resolution: prefer this apex; follow `supersedes` for provenance.

## Decision

### 2026-08-15 host-substrate boundary amendment

The port engine remains a neutral mechanism, not a reason to retain speculative output. Its
current producer set does not include an OS or kernel producer. Hand-maintained Asterinas and
Talos-derived repository trees are retired; Kubernetes remains the differentiated port target.
Host kernel and OS artifacts are consumed and pinned outside the first-party source tree. Any
future generated host region must land atomically with its registered producer, rule pack,
source/toolchain pins, receipts, and failure tests. Earlier Asterinas and owned-host-kernel gists
in this consolidated record are provenance only where they conflict with this amendment.

1. **This ADR is the live reading entry** for topic `k8s_port` under the end-state ADR policy.
2. **Member ADRs listed in `supersedes`** are historical; normative gist is preserved below.
3. **Contradictions** among members are resolved by later higher-number members and by
   ADR-0515 / ADR-0363 / ADR-0562 / ADR-0615 / ADR-0635 / ADR-0637–0639 when applicable.
4. **Activation-sensitive** items (warm CAS, RE workers) remain fail-closed until explicit go-gate.

## Preserved member gists

- **ADR-66** (ADR-0066-live-code-introspection-docs-portal): ### 1. Extractor-as-canonical-source Every fact in the docs portal MUST originate from a canonical extractor over one of these sources. Hand-authored markdown is allowed only for prose narrative (rationale, decision context, deliberate-mode pre-mortem); structured facts (lane lists, lane commands, µservice counts, endpoint inventories, dep graphs) 
- **ADR-93** (ADR-0093-latency-budget-reporter-rename): - Rename type `DeadlineMiddleware` → `LatencyBudgetReporter` in `oya-http-deadline-middleware-domain` (also renamed; see below). - Rename crate `oya-http-deadline-middleware-domain` → `oya-http-latency-budget-middleware-infrastructure` (also picks up the middleware-infrastructure layer rename from ADR-0092 D3). - Rename associated identifiers: - `D
- **ADR-134** (ADR-0134-portfolio-hyperscaler-pattern-remediation-backlog): Adopt the following remediation items as a **proposed remediation backlog**, organized into two lanes: **(A) product-SLO** (the original five — LLM circuit breaker, per-tenant rate limit, provider-degraded shed, Workflow Studio golden signals, error-budget burn rate) and **(B) build/CI/CD pipeline** (the P0 + LATER items, per ADR-0514). Each item m
- **ADR-142** (ADR-0142-crdt-portability-trait): oyatie owns a CRDT portability trait kernel; Loro is the primary adapter; Yjs and Automerge are maintained as **INV-PORTABILITY-CI- COMPILE** alternates — they compile in CI on every change, exercising the trait surface, but are NOT deployed. ### Layer A — trait kernel (this ADR's primary artifact) A new crate `crates/oya-shared-crdt-portability-ke
- **ADR-198** (ADR-0198-k8s-node-autoscaling-karpenter): ### D-1. Karpenter 1.11 is the canonical node autoscaler - **License:** Apache 2.0. - **Source:** kubernetes-sigs/karpenter (CNCF; vendor-neutral core) + cloud-provider plugins. - **Cluster Autoscaler is removed** from the substrate. There is no fallback; if Karpenter fails, the manually-fixed nodepool (per NodePool CRD) survives and absorbs steady
- **ADR-208** (ADR-0208-realtime-transport-tier): ### Three-tier transport model | Tier | Use | When | |---|---|---| | **SSE (Server-Sent Events)** | One-way server → client streams | Log tail, metric tail, AI streaming responses, status feed, deploy progress | | **WebSocket** | Bidirectional client-facing product surfaces | Workflow Studio canvas collab (Loro CRDT sync), shared cursors, chat | | 
- **ADR-222** (ADR-0222-saga-compensation-portfolio-policy): ### D-1. Saga shape Every cross-µservice write is a saga consisting of an ordered list of steps. Each step declares: ```rust pub struct SagaStep { pub step_id: StepId, // unique within the saga pub target_microservice: MicroserviceId, pub forward_action: ActionRef, // capability + input pub compensation_action: CompensationRef, // Cancel | Refund |
- **ADR-241** (ADR-0241-dr-business-continuity-portfolio-policy): ### D-1. Four DR tiers | Tier | RTO | RPO | Replication shape | Drill cadence | Typical µservices | | --- | --- | --- | --- | --- | --- | | **T1** | < 5 min | 0 (zero data loss) | Active-active multi-AZ + cross-region warm standby | Quarterly + ad-hoc on every release | Intelligence runtime (capability invocation), audit chain, observability, identity k
- **ADR-276** (ADR-0276-backup-portability-format-gdpr-article-20): ### D-1: Format — JSON-LD 1.1 with per-µservice schemas referenced by URI The canonical wire format is **JSON-LD 1.1** per the W3C Recommendation of 2020-07-16. Every exported document is a JSON-LD node with a mandatory `@context` field referencing the per-µservice schema URI: ```json { "@context": "https://contracts.oyatie.dev/portability/v1/mail/
- **ADR-394** (First-party Rust internal developer platform (Leptos portal + ops BFF)): ### 1. One first-party portal Oyatie builds and operates a **first-party Rust internal developer platform** consisting of: 1. a Leptos SSR + hydration portal shell; 2. an owned Rust operations BFF that composes capability APIs without becoming a domain owner; 3. catalog, documentation, scorecard, SLO, runbook, release, incident, cost, and provision
- **ADR-569** (Commission the oya-data outbox CDC change-stream Postgres adapter (oya-data-outb): Commission **`libs/oya-data-outbox-adapter-postgres`** — the ADR-0510 transitional Postgres (via sqlx) realization of `oya-data-outbox-kernel::ChangeStreamSource`. It absorbs ALL engine impedance behind the unchanged port; only this adapter is replaced by the engine-native changefeed at W5. ### D1 — `SqlxChangeStreamSource { pool: PgPool }`, async-
- **ADR-611** (Land the Asterinas real-boot foundation harness under kernel/ (kuberos Wave-1 sh): Land, under the sanctioned `kernel/` nested workspace, one owned-Rust deliverable: 1. **`kernel/core/asterinas-boundary`** — a zero-dependency compile-time boundary crate: the black-box pin of the upstream release ISO (`asterinas-release-v0.17.2.json`, digest-embedded via `include_str!` so pin and manifest cannot drift) and the **closed** boot-read
- **ADR-632** (Public product protocols, internal RPC, transport security, serialization, telem): ### D1 — Public product contract: REST, versioned webhooks, events, and deliberate streaming Public synchronous APIs are HTTPS REST documented by OpenAPI 3.2.0. Public asynchronous delivery uses versioned, authenticated, signed, idempotent, replay-protected webhooks described with AsyncAPI 3.1.0; CloudEvents 1.0.2 is the event envelope where its st
- **ADR-637** (Owned deterministic Go-to-Rust port engine): ### D1 — reusable engine, explicit home, and neutral policy split The program SHALL build `oya-port`, an owned, Kubernetes-agnostic deterministic Go-to-Rust port engine at `build/port-engine/*`. The one root Cargo workspace members-line amendment needed to admit that root is authorized by this ADR as the ADR-0538 exception described above. It MUST 
- **ADR-638** (Mechanically maintained Kubernetes Rust port): ### D1 — destination, scope, and doctrine-first divergence Generated Kubernetes Rust output SHALL live under `k8s/`. `os/`, `cloud/cloud-k8s`, and managed-Kubernetes facades MUST consume it only through approved `k8s/ports/**` seams; they MUST NOT become alternate homes for generated upstream code. The program adopts full A-prime scope from the sta

## Consequences

- Agent default read path: `docs/decisions/ADR-0xxx` apex files + this topic.
- Citations to member numbers remain valid via `docs/decisions/_disposition/adr-redirect.v1.json`.
- Further body merge refinements may land as amendments to this apex only.

### ADR-638 residual

**Mechanically maintained Kubernetes Rust port** — ### D1 — destination, scope, and doctrine-first divergence Generated Kubernetes Rust output SHALL live under `k8s/`. `os/`, `cloud/cloud-k8s`, and managed-Kubernetes facades MUST consume it only through approved `k8s/ports/**` seams; they MUST NOT become alternate homes for generated upstream code. The program adopts full A-prime scope from the start: apimachinery, API types, client machinery, com

### ADR-611 residual

**Land the Asterinas real-boot foundation harness under kernel/ (kuberos Wave-1 shard-1)** — Land, under the sanctioned `kernel/` nested workspace, one owned-Rust deliverable: 1. **`kernel/core/asterinas-boundary`** — a zero-dependency compile-time boundary crate: the black-box pin of the upstream release ISO (`asterinas-release-v0.17.2.json`, digest-embedded via `include_str!` so pin and manifest cannot drift) and the **closed** boot-ready marker set (login / shell / `Welcome to NixOS` /

### ADR-222 residual

**ADR-0222-saga-compensation-portfolio-policy** — ### D-1. Saga shape Every cross-µservice write is a saga consisting of an ordered list of steps. Each step declares: ```rust pub struct SagaStep { pub step_id: StepId, // unique within the saga pub target_microservice: MicroserviceId, pub forward_action: ActionRef, // capability + input pub compensation_action: CompensationRef, // Cancel | Refund | Retry | Noop-with-evidence pub idempotency_key_st

### ADR-241 residual

**ADR-0241-dr-business-continuity-portfolio-policy** — ### D-1. Four DR tiers | Tier | RTO | RPO | Replication shape | Drill cadence | Typical µservices | | --- | --- | --- | --- | --- | --- | | **T1** | < 5 min | 0 (zero data loss) | Active-active multi-AZ + cross-region warm standby | Quarterly + ad-hoc on every release | Intelligence runtime (capability invocation), audit chain, observability, identity kernel, payment, ops-portal | | **T2** | < 1 h | <

### ADR-93 residual

**ADR-0093-latency-budget-reporter-rename** — - Rename type `DeadlineMiddleware` → `LatencyBudgetReporter` in `oya-http-deadline-middleware-domain` (also renamed; see below). - Rename crate `oya-http-deadline-middleware-domain` → `oya-http-latency-budget-middleware-infrastructure` (also picks up the middleware-infrastructure layer rename from ADR-0092 D3). - Rename associated identifiers: - `DEADLINE_EXCEEDED_BODY_PREFIX` → `LATENCY_BUDGET_EX

### ADR-632 residual

**Public product protocols, internal RPC, transport security, serialization, telemetry, and provider-owned fabric posture** — ### D1 — Public product contract: REST, versioned webhooks, events, and deliberate streaming Public synchronous APIs are HTTPS REST documented by OpenAPI 3.2.0. Public asynchronous delivery uses versioned, authenticated, signed, idempotent, replay-protected webhooks described with AsyncAPI 3.1.0; CloudEvents 1.0.2 is the event envelope where its stable HTTP binding applies. SSE is the default for

### ADR-569 residual

**Commission the oya-data outbox CDC change-stream Postgres adapter (oya-data-outbox-adapter-postgres) behind the ChangeSt** — Commission **`libs/oya-data-outbox-adapter-postgres`** — the ADR-0510 transitional Postgres (via sqlx) realization of `oya-data-outbox-kernel::ChangeStreamSource`. It absorbs ALL engine impedance behind the unchanged port; only this adapter is replaced by the engine-native changefeed at W5. ### D1 — `SqlxChangeStreamSource { pool: PgPool }`, async-over-sync-kernel split The adapter's async `poll_c

### ADR-637 residual

**Owned deterministic Go-to-Rust port engine** — ### D1 — reusable engine, explicit home, and neutral policy split The program SHALL build `oya-port`, an owned, Kubernetes-agnostic deterministic Go-to-Rust port engine at `build/port-engine/*`. The one root Cargo workspace members-line amendment needed to admit that root is authorized by this ADR as the ADR-0538 exception described above. It MUST be reviewed as a root-membership change and MUST N

### ADR-208 residual

**ADR-0208-realtime-transport-tier** — ### Three-tier transport model | Tier | Use | When | |---|---|---| | **SSE (Server-Sent Events)** | One-way server → client streams | Log tail, metric tail, AI streaming responses, status feed, deploy progress | | **WebSocket** | Bidirectional client-facing product surfaces | Workflow Studio canvas collab (Loro CRDT sync), shared cursors, chat | | **gRPC streaming** | Service-to-service streams |

### ADR-276 residual

**ADR-0276-backup-portability-format-gdpr-article-20** — ### D-1: Format — JSON-LD 1.1 with per-µservice schemas referenced by URI The canonical wire format is **JSON-LD 1.1** per the W3C Recommendation of 2020-07-16. Every exported document is a JSON-LD node with a mandatory `@context` field referencing the per-µservice schema URI: ```json { "@context": "https://contracts.oyatie.dev/portability/v1/mail/message.jsonld", "@type": "MailMessage", "@id": "u

### ADR-198 residual

**ADR-0198-k8s-node-autoscaling-karpenter** — ### D-1. Karpenter 1.11 is the canonical node autoscaler - **License:** Apache 2.0. - **Source:** kubernetes-sigs/karpenter (CNCF; vendor-neutral core) + cloud-provider plugins. - **Cluster Autoscaler is removed** from the substrate. There is no fallback; if Karpenter fails, the manually-fixed nodepool (per NodePool CRD) survives and absorbs steady-state load. - **Deployment:** Helm chart at `micr

### ADR-394 residual

**First-party Rust internal developer platform (Leptos portal + ops BFF)** — ### 1. One first-party portal Oyatie builds and operates a **first-party Rust internal developer platform** consisting of: 1. a Leptos SSR + hydration portal shell; 2. an owned Rust operations BFF that composes capability APIs without becoming a domain owner; 3. catalog, documentation, scorecard, SLO, runbook, release, incident, cost, and provisioning modules over existing canonical sources; 4. st

### ADR-66 residual

**ADR-0066-live-code-introspection-docs-portal** — ### 1. Extractor-as-canonical-source Every fact in the docs portal MUST originate from a canonical extractor over one of these sources. Hand-authored markdown is allowed only for prose narrative (rationale, decision context, deliberate-mode pre-mortem); structured facts (lane lists, lane commands, µservice counts, endpoint inventories, dep graphs) are extracted, never typed. | Source | Extractor |

### ADR-142 residual

**ADR-0142-crdt-portability-trait** — oyatie owns a CRDT portability trait kernel; Loro is the primary adapter; Yjs and Automerge are maintained as **INV-PORTABILITY-CI- COMPILE** alternates — they compile in CI on every change, exercising the trait surface, but are NOT deployed. ### Layer A — trait kernel (this ADR's primary artifact) A new crate `crates/oya-shared-crdt-portability-kernel/`: - `pub trait CrdtDoc` — the shared surface

### ADR-134 residual

**ADR-0134-portfolio-hyperscaler-pattern-remediation-backlog** — Adopt the following remediation items as a **proposed remediation backlog**, organized into two lanes: **(A) product-SLO** (the original five — LLM circuit breaker, per-tenant rate limit, provider-degraded shed, Workflow Studio golden signals, error-budget burn rate) and **(B) build/CI/CD pipeline** (the P0 + LATER items, per ADR-0514). Each item may become binding only in the PR that ships its va
