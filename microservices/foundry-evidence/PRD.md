---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-foundry-evidence
microservice: foundry-evidence
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: [ADR-0028, ADR-0003, ADR-0024]
related_adrs: [ADR-0003, ADR-0024, ADR-0028, ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0130, ADR-0131, ADR-0132, ADR-0133]
related_specs:
  - /specs/foundry-evidence.json
  - /specs/per-microservice-flat-layout.json
  - /specs/foundry-agent-runtime.json
date: 2026-05-17
owner_team: axis-foundry-evidence
doc_status: published
---

# PRD-foundry-evidence: Foundry Agent-Runtime Evidence Frontend

## Purpose

The `foundry-evidence` µservice is the **Foundry-specific evidence frontend** that records every capability-invocation made by a Foundry agent (model call, tool call, autonomy-tier decision, guardrail decision, eval outcome) into a structured **evidence pack** and emits that pack to the global `audit-chain` substrate for cryptographic sealing.

Per ADR-0131 (Foundry split): `foundry-evidence` owns audit-evidence emission for the Foundry agent runtime. It is **not** an audit substrate itself — that role belongs to the `audit-chain` µservice. `foundry-evidence` is the **assembly + frontend** that aggregates the heterogeneous runtime signals into a uniform evidence-pack schema, hands it to `audit-chain` for Merkle-sealing, and exposes regulator-grade query + export surfaces over the indexed evidence.

This split is deliberate:

| Concern | Owned by | Why |
|---|---|---|
| Per-event Ed25519 + Merkle sealing | `audit-chain` | Domain-uniform cryptographic substrate (Bominal ADR-0028) |
| Per-event WORM raw blob | `audit-chain` | Substrate substrate; WORM lock under `audit-chain` Object Lock |
| Aggregation of foundry-runtime + foundry-eval + foundry-guardrails signals into a per-invocation evidence pack | `foundry-evidence` | Foundry-specific schema; not a substrate concern |
| Regulator-grade evidence export filtered to AI-traceability frameworks (EU AI Act Art. 12 + Art. 26 logs) | `foundry-evidence` | Specific to agent-runtime semantics; substrate doesn't know about "agent invocation" |
| Per-capability-invocation indexing + query | `foundry-evidence` | Foundry-specific query surface; substrate query is per-event not per-invocation |

`foundry-evidence` inherits Bominal ADR-0003 (emission contract) + ADR-0024 (eval-evidence integration) + ADR-0028 (audit chain Merkle/Ed25519) 1:1 via the `audit-chain` substrate; oyatie-specific decisions overlay only where AI-Act traceability or per-vertical pack constraints diverge.

## Tenant Value

- **Tenant Outcome 1 — EU AI Act Art. 12 + Art. 26 traceability out-of-the-box.** Every Foundry agent invocation produces a regulator-grade evidence pack with model version, prompt hash, output hash, autonomy-tier decision, guardrail decisions, eval outcomes, and Merkle-sealed audit link. Tenants do not write their own logging glue.
- **Tenant Outcome 2 — Per-invocation forensic answer in ≤ 100 ms p99.** "What did the agent see, decide, and do during invocation X?" returns a full evidence pack in one query; the auditor or incident responder is not joining 5 systems.
- **Tenant Outcome 3 — Regulator-ready export by (tenant, framework, window).** Tenant raises an AI-Act, HIPAA §164.312(b), GDPR Art. 30, KR PIPA Art. 29 audit request; oyatie exports a signed, Merkle-linked, time-bounded evidence-pack bundle scoped to that framework's required fields; regulator independently verifies via the substrate chain.
- **Tenant Outcome 4 — Eval-evidence join.** Every agent invocation links to the eval set that gated its release and to the eval verdict at the invocation moment, per ADR-0024.
- **Internal Outcome 5 — Single source of truth for "what did the agent do".** Other µservices (governance, observability, billing, support) consume evidence packs rather than re-walking foundry-runtime logs.
- **Internal Outcome 6 — Honest non-fabrication frontend.** Per ADR-0133, oyatie does not claim hyperscaler-grade evidence depth that it cannot reproduce; this µservice's claim matrix is CI-enforced via `hyperscaler-maturity-claims` gate.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | foundry-runtime worker | to call `record_invocation(invocation_envelope)` and receive `{pack_id, audit_event_id}` synchronously within ≤500 ms p99 | the capability call can complete without waiting on regulator-grade pack assembly | capability-invocation-recorder | Must |
| FR-02 | evidence-pack-builder | to aggregate signals from foundry-runtime (provider response, prompt, output), foundry-eval (eval verdict at invocation time), foundry-guardrails (guardrail decisions), foundry-supervisor (autonomy-tier decision) into a single evidence pack per `(invocation_id)` | downstream regulator export and forensic query operate on a single uniform shape | evidence-pack-builder | Must |
| FR-03 | evidence-pack-builder | to emit the assembled pack to `audit-chain` with `event_class=foundry.invocation.evidence.v1` and receive the audit `event_id` + `period_id` back | Merkle-sealing is delegated to the substrate; foundry-evidence never invents its own sealing | evidence-pack-builder | Must |
| FR-04 | eval-evidence-aggregator | to join the eval-set verdict at invocation-time (from foundry-eval) to the invocation envelope before pack emission | per ADR-0024 every invocation carries the eval-verdict that was current at the moment of execution | eval-evidence-aggregator | Must |
| FR-05 | evidence-query API | to read `evidence_packs(tenant, time_range, invocation_id?, agent_id?, capability?, autonomy_tier?, framework_filter?)` with pagination | tenants and internal forensic users get per-invocation answers | evidence-query | Must |
| FR-06 | regulator-export | to produce a signed, Merkle-linked, framework-filtered evidence-pack bundle for `(tenant, framework, time_range)` where `framework ∈ {eu-ai-act, hipaa, gdpr, kr-pipa, soc2, iso-27001}` | regulator engagement honours each framework's specific evidence requirements | regulator-export | Must |
| FR-07 | evidence-query API | every read is itself audit-emitted via `audit-chain` per Bominal ADR-0028 §"Self-observability" (audit-of-audits) | tenants and regulators can see who read what evidence and when | evidence-query | Must |
| FR-08 | every Foundry µservice integration | to consume a stable `oya-foundry-evidence-sdk` client (Rust + future TS/Python bindings) | uniform integration across foundry-runtime, foundry-guardrails, foundry-supervisor, foundry-eval | capability-invocation-recorder | Must |
| FR-09 | autonomy-tier overlay | to attach the autonomy-tier decision (T0..T3) that was active for the invocation per ADR-0024 + ADR-0130 | regulator-grade T2/T3 evidence requirements are satisfied by construction | evidence-pack-builder | Must |
| FR-10 | evidence-archive cascade | to drive cold-tier archival of evidence-pack blobs at per-pack retention boundaries while preserving the audit-chain Merkle proof | regulatory retention obligations honoured without operator action; chain integrity preserved | regulator-export | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| `record_invocation` synchronous receipt | ≤100 ms | ≤500 ms | ≤1.5 s | Task brief; covers durable WAL + pack-builder enqueue + audit-chain emit |
| Evidence pack full assembly (signals → audit-chain seal) | ≤500 ms | ≤2 s | ≤5 s | end-to-end async (foundry-runtime/eval/guardrails signal collection + audit-chain seal mint) |
| `evidence_query` p99 | ≤50 ms | ≤100 ms | ≤300 ms | Task brief; Postgres-indexed per-invocation lookup |
| Regulator-export bundle assembly | — | ≤30 s per 10k packs | — | streamed; pagination + Merkle-proof chunking |
| Sustained `record_invocation` throughput per cluster | — | ≥20 k invocations/s | — | horizontally shardable per tenant_partition |
| Pack assembly success rate | — | ≥99.99% | — | failed packs go to dead-letter; never lost |

### Security

- Every `record_invocation` is authenticated via the caller's SPIFFE identity; SPIFFE → `(microservice=foundry-*, tenant_id)` binding enforced via `policy/tenant-scope.cedar`.
- Evidence pack blob storage is **WORM** via the underlying `audit-chain` Object Lock (Compliance mode). `foundry-evidence` never holds its own WORM; pack-builder writes the canonical blob through the `audit-chain` substrate sealing path.
- Cedar v4 default-deny per ADR-0056; six policy fragments cover tenant scope / CI scope / public-read / auditor scope / regulator-export / data-residency.
- Postgres index is read-replicated; primary writes are append-only at the SQL level (no UPDATE, no DELETE except via retention cascade RPC).
- No raw prompt / output text leaves the substrate boundary; payload references via content-addressable `payload_sha`; tenant policy gates plaintext reads.
- Per ADR-0133 hyperscaler-maturity-claims gate, every NFR row above is CI-asserted; no aspirational numbers.

### Audit + Compliance

- Every evidence-pack emit is itself audit-emitted via `audit-chain` (recursive; substrate-bootstrapped).
- Retention defaults per pack (carried by `audit-chain` retention cascade; `foundry-evidence` carries no retention authority of its own — see ADR-0131 substrate split):
  - pack-us-healthcare: 6 y per HIPAA §164.316(b)(2) + §164.312(b) audit controls.
  - pack-eu: 10 y for EU AI Act high-risk-system technical documentation per Art. 18; 2 y default otherwise.
  - pack-kr: 3 y per KR PIPA Art. 29.
  - pack-us: 7 y SOC 2 evidence retention; per-tenant DPA may extend.
  - pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa: per pack local-law minimum.
- Framework-filtered export profiles (`regulator-export` capability):
  - `eu-ai-act` — Art. 12 logs + Art. 26 user obligations + Art. 18 technical documentation links.
  - `hipaa` — §164.312(b) audit controls + §164.308(a)(1)(ii)(D) information system activity review.
  - `gdpr` — Art. 30 ROPA join + Art. 22 automated-decision evidence.
  - `kr-pipa` — Art. 29 safety measures + Art. 23 sensitive-info handling logs.
  - `soc2` / `iso-27001` — system activity + change-management evidence.

### Availability + SLO

- Availability target: 99.99 % monthly for `record_invocation` (writes MUST never silently fail; WAL + dead-letter on substrate unavailability; degraded-mode emission still returns receipt + later-sealed marker).
- Availability target: 99.95 % monthly for `evidence_query` and `regulator-export`.
- RTO: ≤ 15 min. RPO: ≤ 1 s (period-aligned via audit-chain substrate).
- Self-observability: foundry-evidence emits SLIs for `pack_assembly_latency_seconds`, `audit_chain_emit_latency_seconds`, `evidence_query_latency_seconds`, `regulator_export_latency_seconds`, `pack_assembly_failure_rate`, `audit_chain_backlog_depth`; the SLO engine (`observability` µservice) gates this µservice's own promotion per ADR-0130.

### Data residency

- Evidence packs inherit the source tenant's `jurisdiction_code` per ADR-0117 and `policy/data-residency.md`. Pack data **strictly stays in the source pack**; cross-pack replication is **forbidden** for chain continuity (each pack has its own `audit-chain` chain).
- Cross-pack regulator export to a tenant-controlled archive is permitted only via the tenant-initiated DPA-recorded export RPC + receiving-tenant SCC + Cedar `regulator-export-scope` permit.

## Bounded Contexts

| BC | Purpose | Inbound | Outbound | Owner |
|---|---|---|---|---|
| `evidence-pack-builder` | Aggregate runtime/eval/guardrail signals into a per-invocation pack and emit to audit-chain | foundry-runtime, foundry-eval, foundry-guardrails, foundry-supervisor (via Workflow events) | audit-chain emit | axis-foundry-evidence |
| `capability-invocation-recorder` | Sync receipt API for foundry-runtime workers; WAL + dead-letter | foundry-runtime worker SDK | pack-builder enqueue | axis-foundry-evidence |
| `eval-evidence-aggregator` | Join eval-set verdict (foundry-eval) to invocation envelope at invocation moment | foundry-eval result topic | pack-builder | axis-foundry-evidence + axis-foundry-eval |
| `evidence-query` | Per-invocation read API + dashboards data path | tenant-portal, governance, internal forensic | Postgres + audit-chain query (audit-of-audits) | axis-foundry-evidence |
| `regulator-export` | Framework-filtered, signed evidence bundle assembly | regulator engagement workflows | audit-chain query + S3 export bucket | axis-foundry-evidence + council-privacy |

## Substrate dependencies

| Substrate | Used for | ADR ref |
|---|---|---|
| `audit-chain` µservice | Merkle sealing of every evidence pack; WORM blob storage; chain query | ADR-0028, ADR-0131 |
| `observability` µservice | SLO ingestion + alerting + gate-of-promotion | ADR-0130 |
| `tenancy` µservice | Tenant identity + DSR cascade entry-points | ADR-0131 |
| `governance` µservice | Cedar policy evaluation; autonomy-tier authority resolution | ADR-0056 + ADR-0131 |
| `foundry-runtime` µservice | Invocation envelope source | ADR-0131 |
| `foundry-eval` µservice | Eval-verdict source | ADR-0024, ADR-0131 |
| `foundry-guardrails` µservice | Guardrail decision source | ADR-0131 |
| `foundry-supervisor` µservice | Autonomy-tier decision source | ADR-0024, ADR-0131 |

## Cross-µservice contracts

| Direction | Contract | Shape |
|---|---|---|
| inbound (sync REST) | `record_invocation` | `oya-foundry-evidence-capability-invocation-recorder-rest` POST `/v1/invocations` |
| inbound (events) | `foundry.runtime.invocation.completed.v1`, `foundry.eval.verdict.published.v1`, `foundry.guardrails.decision.emitted.v1`, `foundry.supervisor.autonomy.decided.v1` | Workflow event subscriptions; AsyncAPI in `contracts/asyncapi/foundry-evidence-events.yaml` |
| outbound (sync) | `audit-chain.emit` per Bominal ADR-0003 | via `oya-audit-chain-emission-sdk` |
| outbound (events) | `foundry.evidence.pack.assembled.v1`, `foundry.evidence.pack.assembly_failed.v1`, `foundry.evidence.regulator_export.requested.v1`, `foundry.evidence.regulator_export.completed.v1` | Workflow events |

## Competitor parity scope

See `competitor-parity-matrix.md`. Targets: AWS CloudTrail with Audit Lake, Google Cloud Audit Logs Premium, Azure Sentinel, Splunk, LogicMonitor. Per ADR-0133 the matrix declares **honest gaps** as well as parity.

## Cost target

See `cost-budget.md`. Per-invocation evidence cost target ≤ $0.0001 fully-loaded at sustained 20 k inv/s; archival amortised across pack retention window.

## Failure modes + recovery

See `failure-modes.md` for the FM-01..FM-12 catalogue and `runbooks/` for Sev-1/Sev-2 procedures.

## Capacity model

See `capacity-model.md` for the sizing math, headroom plan, and burst capacity contract.

## Multi-region posture

See `multi-region.md`. Pack-local; no cross-pack data plane; control-plane manifests replicated per ADR-0117.

## Sunset + change-management

This µservice is **stable from M01**. Schema evolution (`oya-foundry-evidence-evidence-pack`) follows the no-silent-regression policy: every change is an ADR + version bump + sunset window. Per ADR-0132 there is no product-bundle to dissolve.
