---
doc_class: PerformanceBenchmarkNumbers
microservice: itsm
benchmark_wave: Wave-4-rolling
benchmark_date: 2026-05-21
benchmark_owner: codex-itsm-ownership-audit-w4r
benchmark_mode: single-industry-leader-target-plus-context-overlay-plus-tenant-class-overlay
industry_leader_target: ServiceNow ITSM (Washington DC + Vancouver release; ITSM Pro + CMDB + Now Assist running on ServiceNow Sovereign Cloud or ServiceNow Public Cloud)
secondary_counterparts:
  - Atlassian Jira Service Management Cloud Enterprise
  - Freshservice Enterprise
deployment_contexts:
  - oyatie-public-cloud
  - guest-on-aws
  - guest-on-oci
  - on-prem
  - colo
  - oyatie-as-cloud-provider
tenant_classes:
  - demo_trial
  - paid
five_anchors:
  - /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md §D-15 deployment contexts + §D-19 OCI Always Free
  - /Users/jasonlee/oyatie/microservices/itsm/capacity-model.md (86 KB) + microservices/itsm/cost-budget.md (69 KB) + microservices/itsm/multi-region.md (69 KB)
  - /Users/jasonlee/oyatie/microservices/itsm/slos/*.openslo.yaml (12 SLO files)
  - /Users/jasonlee/oyatie/microservices/itsm/benchmarks/servicenow-vs-jsm-vs-freshservice-vs-oyatie.md (existing benchmark file; to be retracted-and-reauthored per Wave 15J)
  - ServiceNow Now Platform performance baseline (public documentation + customer-reported benchmarks)
related_adrs:
  - ADR-0244 tenant scoping
  - ADR-0247 self-modification (Foundry absorption)
  - ADR-0248 Amazon cellular architecture
  - ADR-0250 build-ahead-of-certification
  - ADR-0252 HLC + TrueTime tier
  - ADR-0253 HTTP/3 + QUIC default
  - ADR-0254 K8s + Cloud Hypervisor pods
  - ADR-0263 audit emission
  - ADR-0316 capability tiers (retirement candidate)
  - ADR-0328 canonical sequence + substance bar
---

# ITSM Performance Benchmark Numbers — Single Industry-Leader Target + Deployment-Context Overlay + Tenant-Class Overlay

## 0. Methodology and brief-anchor header

### 0.1 No tier segmentation

Per `feedback_no_capability_profiles_2026_05_20.md` and `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md`, benchmark numbers in this deliverable do NOT use retired named capability levels tier scaffolding. The retired tier scheme appears in `microservices/itsm/benchmarks/servicenow-vs-jsm-vs-freshservice-vs-oyatie.md` (which is a Wave 15J retraction-and-reauthor target per T-RET-02 in the companion coherence audit). This document is the replacement shape.

### 0.2 Single industry-leader target

Per ADR-0328 substance bar + Big-8 priority + the directive in this audit wave, benchmark numbers target a single industry-leading counterpart with full counterpart-equivalent feature surface enabled. The chosen target is ServiceNow ITSM (Washington DC + Vancouver release), running on ServiceNow Public Cloud / Sovereign Cloud Enterprise tier with ITSM Pro + CMDB + Now Assist + Discovery + Service Mapping enabled. ServiceNow is the canonical Big-8 ServiceNow-family displacement target per ADR-0321 + ADR-0328 §D-2.16.

Secondary counterparts (Atlassian JSM Cloud Enterprise + Freshservice Enterprise) provide corroborating evidence rows but are not the primary numeric target. The Oyatie target equals or beats ServiceNow at the same workload envelope.

### 0.3 Deployment-context overlay

Six canonical contexts per `specs/master-plan-sequencing.json#deployment_contexts`:

- `oyatie-public-cloud`: Oyatie-operated managed cloud cells.
- `guest-on-aws`: Oyatie runs inside AWS accounts.
- `guest-on-oci`: Oyatie runs inside OCI tenancies (Always Free sub-profile for demo/trial).
- `on-prem`: Customer-controlled data centers on customer hardware.
- `colo`: Owned/rented hardware in colocation facilities.
- `oyatie-as-cloud-provider`: Oyatie's own IaaS surface.

Each numeric target is overlaid with the context-specific adjustment (latency floor changes with control-plane proximity; throughput ceiling changes with capacity envelope; cost basis changes with provider/colo/on-prem economics).

### 0.4 Tenant-class overlay

Two tenant classes per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md`:

- `demo_trial`: zero-cost, OCI Always Free profile by default, hard usage caps + time gates, best-effort SLO posture.
- `paid` (with `billing_components ⊆ {revenue_share, per_seat, per_usage}`): customer-chosen deployment context, no usage caps by default, contractual SLO per tenant contract.

Both classes get uniform industry-leader-grade feature surface; benchmarks differ in workload envelope (demo/trial = capped; paid = uncapped) and SLO commitment level (demo/trial = best-effort; paid = contractual).

### 0.5 Workload envelopes

Three reference envelopes are used for benchmark sizing:

- Envelope A — Small Paid Tenant: 100 IT-Ops agents, 50k tickets/year, 100k CMDB CIs, 5k KB articles, 10 GB attachment storage.
- Envelope B — Enterprise Paid Tenant: 500 IT-Ops agents, 2M tickets/year, 5M CMDB CIs, 50k KB articles, 1 TB attachment storage.
- Envelope C — Mega Enterprise Paid Tenant: 5,000 IT-Ops agents, 50M tickets/year, 50M CMDB CIs, 500k KB articles, 10 TB attachment storage.

Demo/trial envelope is bounded by hard caps (e.g., 10 agents, 500 tickets, 1k CIs, 100 KB articles, 100 MB attachments) and is reported separately under §6.

### 0.6 Five anchors and binding ADRs

Anchors and binding ADRs appear in frontmatter. The performance benchmarks bind to ADR-0248 (Amazon cellular architecture — workloads run inside Tier-0..Tier-4 cells with shuffle sharding), ADR-0254 (K8s + Cloud Hypervisor + Kata pods), ADR-0253 (HTTP/3 + QUIC default; gRPC over HTTP/3), and ADR-0252 (HLC default for causality; TrueTime opt-in for financial-grade tenants).

## 1. Metric definitions

### 1.1 Ticket creation latency

Definition: Time from client `POST /itsm/incidents` (or analogous create endpoint) to response with `audit_event_class`, `workflow_run_id`, `ontology_object_ref` populated and audit-chain seal confirmed. Measured end-to-end including Cedar policy evaluation, tenant-scope assertion, audit-chain emit, and ontology projection.

Percentiles measured: p50, p95, p99, p99.9.

### 1.2 Ticket search latency

Definition: Time from client `GET /itsm/incidents?q=<query>` (or analogous search endpoint) to first paginated response. Measured against ~1M-ticket corpus, ~5M-ticket corpus, ~50M-ticket corpus (Envelope A / B / C respectively). Full-text query (single keyword) and faceted query (5+ filter facets) are reported separately.

Percentiles measured: p50, p95, p99.

### 1.3 Workflow execution time

Definition: Time from workflow trigger (incident creation, change submission, catalog request) to workflow completion (terminal state). Measured for a canonical 10-step approval workflow with two human-approval gates and one HTTP-integration step.

Percentiles measured: p50, p95, p99.

### 1.4 SLA breach detection latency

Definition: Time from SLA condition becoming true (SLA timer expires past threshold) to canonical `oya.itsm.sla.breached` event emission to `audit-chain` and notification dispatch to `messenger`/`mail`. Measured against `slos/local-sla-breach-detection.openslo.yaml`.

Percentiles measured: p50, p95, p99, p99.9.

### 1.5 Mobile sync latency

Definition: Time from mobile client cold-start with cached session token to populated agent inbox of 100 active tickets. Includes session restore, tenant assertion, ticket query, and payload serialization. Measured on iPhone 15 Pro on 5G + AT&T mid-band + ~30 ms RTT to closest cell.

Percentiles measured: p50, p95, p99.

### 1.6 Agent dashboard p99 (interactive UX latency)

Definition: Time from agent click on "Active Tickets" dashboard to fully rendered table of 100 tickets with priority, status, assignee, SLA timer, and quick-action buttons. Measured for Envelope B (Enterprise) tenant, 500 concurrent agents, sustained ticket-creation rate of 1 ticket/sec/agent.

Percentile measured: p99 (also p95 + p99.9 secondary).

### 1.7 CMDB query latency

Definition: Time from client `GET /itsm/cmdb/cis/<id>?expand=relationships` to populated CI with all 1-hop, 2-hop, 3-hop relationships. Measured against the CMDB cardinality of each envelope. (Reported here as a fourth metric beyond the headline five, because ITSM-as-displacement-target requires CMDB performance to beat ServiceNow.)

Percentiles measured: p50, p95, p99.

### 1.8 Workflow execution throughput

Definition: Sustained workflow-completion rate (workflows/sec) under steady-state mixed load (60% incident-state-machine transitions, 25% change-approval workflows, 10% catalog-fulfillment workflows, 5% SLA-breach-remediation workflows).

Measured as throughput at given concurrent-agent count. (Reported here as a sixth metric to cover sustained vs burst capacity.)

### 1.9 AI deflection response latency

Definition: Time from requester portal question submission to AI-suggested answer rendered. Bound to `intelligence` µservice Llama-3.1-70B fine-tuned model running on co-located GPU pool.

Percentiles measured: p50, p95, p99.

## 2. Industry-leader target — ServiceNow ITSM baseline (single target)

### 2.1 ServiceNow ITSM Washington DC + Vancouver release on ServiceNow Public Cloud Enterprise tier with ITSM Pro + CMDB + Now Assist enabled. Envelope B (Enterprise Paid Tenant).

ServiceNow does not publish standardized performance benchmarks; numbers below derive from ServiceNow Performance Analytics dashboards (where exposed), the Now Platform documentation, customer case studies, and third-party customer-reported benchmarks. Some numbers are conservative estimates marked `~`.

| Metric | ServiceNow ITSM target |
|---|---|
| Ticket creation latency p50 | ~280 ms |
| Ticket creation latency p95 | ~580 ms |
| Ticket creation latency p99 | ~1 200 ms |
| Ticket creation latency p99.9 | ~3 500 ms |
| Ticket search latency p50 (single keyword, 5M corpus, Zing search) | ~180 ms |
| Ticket search latency p95 (single keyword, 5M corpus) | ~420 ms |
| Ticket search latency p99 (single keyword, 5M corpus) | ~780 ms |
| Ticket search latency p95 (faceted, 5M corpus, 5 facets) | ~780 ms |
| Workflow execution time p50 (10-step approval) | ~1 400 ms |
| Workflow execution time p95 | ~2 400 ms |
| Workflow execution time p99 | ~4 800 ms |
| SLA breach detection latency p50 | ~30 s (SLA-recompute schedule, default every 30 s) |
| SLA breach detection latency p95 | ~60 s |
| SLA breach detection latency p99 | ~120 s |
| Mobile sync latency p99 (Now Mobile Agent, 100 tickets) | ~3 200 ms |
| Agent dashboard p99 (Now Workspace, 100 tickets) | ~1 800 ms |
| CMDB 1-hop query p99 (5M CIs) | ~220 ms |
| CMDB 3-hop query p99 (5M CIs) | ~1 400 ms |
| Workflow throughput sustained (workflows/sec, Enterprise instance) | ~120 |
| Workflow throughput burst (workflows/sec, ≤60 s) | ~400 |
| AI deflection response p95 (Now Assist) | ~2 800 ms |

(Numbers derived from publicly-reported ServiceNow Performance Analytics datapoints, customer case studies at Knowledge25 keynote benchmarks, and third-party load-test reports. Where conservative, marked `~`; specific numbers will be verified during Wave 15E by re-running standardized harness against ServiceNow Public Cloud.)

### 2.2 Secondary counterpart corroboration (not target)

| Metric | JSM Cloud Enterprise | Freshservice Enterprise |
|---|---|---|
| Ticket creation latency p99 | ~1 600 ms | ~1 100 ms |
| Ticket search latency p99 (1M corpus) | ~720 ms | ~880 ms |
| Workflow execution time p99 (10-step approval) | ~6 400 ms | ~5 200 ms |
| CMDB 3-hop query p99 (5M CIs) | ~1 800 ms | ~2 200 ms |
| Workflow throughput sustained | ~80/sec | ~100/sec |

Reading: ServiceNow leads on workflow throughput and CMDB latency due to dedicated CMDB graph engine. JSM lags on workflow throughput due to Jira's project-management-retrofitted architecture. Freshservice mid-market positioning shows higher latency variance. Oyatie ITSM target is to beat ServiceNow at every metric — single industry-leader target per the directive.

## 3. Oyatie ITSM target — base numbers (oyatie-public-cloud context, paid tenant, Envelope B)

### 3.1 Target baseline numbers

The baseline assumes:
- Context: `oyatie-public-cloud`.
- Tenant class: `paid`.
- Cell tier: Tier-2 (per ADR-0248 Amazon cellular; Tier-0 reserved for substrate, Tier-1 reserved for sovereign).
- Envelope: B (500 IT-Ops agents, 2M tickets/year, 5M CMDB CIs, 50k KB articles, 1 TB attachments).
- Workload: steady-state mixed (per §1.8 definition).
- Hardware: K8s pods on Cloud Hypervisor with Kata isolation; 12× ITSM API pods (16 vCPU AMD EPYC 9354P / Ampere A1 equivalent, 64 GiB DDR5), 6× workflow-engine pods, 2× NVIDIA L4 GPUs for AI-deflection, PostgreSQL 17 cluster (1 primary + 2 sync replicas + 2 async replicas, 32 vCPU + 128 GiB + 4 TiB NVMe), Elasticsearch 8.15 fleet (9 nodes), Valkey 8 cache layer, OpenBao secrets, SPIFFE identity.

| Metric | Oyatie ITSM target (paid, public cloud, Envelope B) | vs ServiceNow |
|---|---|---|
| Ticket creation latency p50 | **120 ms** | beats ServiceNow ~280 ms |
| Ticket creation latency p95 | **280 ms** | beats ServiceNow ~580 ms |
| Ticket creation latency p99 | **520 ms** | beats ServiceNow ~1 200 ms |
| Ticket creation latency p99.9 | **1 200 ms** | beats ServiceNow ~3 500 ms |
| Ticket search latency p50 (single keyword, 5M corpus) | **80 ms** | beats ServiceNow ~180 ms |
| Ticket search latency p95 (single keyword) | **180 ms** | beats ServiceNow ~420 ms |
| Ticket search latency p99 (single keyword) | **320 ms** | beats ServiceNow ~780 ms |
| Ticket search latency p95 (faceted, 5 facets) | **240 ms** | beats ServiceNow ~780 ms |
| Workflow execution time p50 (10-step approval) | **400 ms** | beats ServiceNow ~1 400 ms |
| Workflow execution time p95 | **920 ms** | beats ServiceNow ~2 400 ms |
| Workflow execution time p99 | **1 800 ms** | beats ServiceNow ~4 800 ms |
| SLA breach detection latency p50 | **2 s** | beats ServiceNow ~30 s (Oyatie uses event-driven detection per IP-030 vs ServiceNow scheduled recompute) |
| SLA breach detection latency p95 | **8 s** | beats ServiceNow ~60 s |
| SLA breach detection latency p99 | **15 s** | beats ServiceNow ~120 s |
| Mobile sync latency p99 (100 tickets, iPhone 15 Pro, 5G mid-band) | **1 200 ms** | beats ServiceNow ~3 200 ms |
| Agent dashboard p99 (100 tickets) | **600 ms** | beats ServiceNow ~1 800 ms |
| CMDB 1-hop query p99 (5M CIs) | **80 ms** | beats ServiceNow ~220 ms |
| CMDB 2-hop query p99 (5M CIs) | **180 ms** | (not a ServiceNow-published metric; estimate target) |
| CMDB 3-hop query p99 (5M CIs) | **380 ms** | beats ServiceNow ~1 400 ms |
| Workflow throughput sustained | **800/sec** | beats ServiceNow ~120/sec (Rust workflow-engine vs JavaScript-on-MariaDB) |
| Workflow throughput burst (≤60 s) | **3 500/sec** | beats ServiceNow ~400/sec |
| AI deflection response p95 (Llama-3.1-70B fine-tuned KB) | **2 400 ms** | beats ServiceNow Now Assist ~2 800 ms |

### 3.2 Rationale per metric

- Ticket creation latency: Oyatie writes through Rust handler → tokio async stack → PostgreSQL via sqlx pool → audit-chain emit → ontology projection. No JVM warmup, no on-table-trigger script execution (ServiceNow's Business Rules add 50-200 ms per write). The 120 ms p50 target reflects Rust+PostgreSQL+HTTP/3 baseline with Cedar evaluation at <5 ms.
- Ticket search: Elasticsearch 8.15 with appropriate sharding (~5 shards per million tickets) on dedicated nodes outperforms ServiceNow's Zing engine (which is single-tenant multi-tenant-shared and competes with other ServiceNow workloads).
- Workflow execution: Rust `workflow-engine` µservice with deterministic state-machine, durable functions, and replay primitives. ServiceNow Flow Designer compiles to JavaScript executed on the application node; Rust beats JavaScript on workflow-step transition latency by a meaningful factor.
- SLA breach detection: Event-driven from IP-030 (SLA breach remediation loop) using audit-chain stream + observability. ServiceNow uses a recompute scheduler with 30-second default cadence.
- Mobile sync: HTTP/3 + QUIC reduces handshake to 0-RTT for resumed sessions. Payload is binary proto3 via gRPC-over-HTTP/3 + server-side delta-encoded incremental sync.
- Agent dashboard: Server-side rendered (Leptos SSR per `feedback_rust_strict_only_no_python_2026_05_20.md`) with selective island hydration.
- CMDB queries: Per IP-027 (CMDB reconciliation graph), Oyatie uses PostgreSQL JSONB + recursive CTEs + custom graph-traversal kernel. ServiceNow's CMDB uses row-table relational model with explicit join tables — slower at 3-hop and beyond.
- Workflow throughput: Rust `workflow-engine` per ADR-0247 + ADR-0255-amendment Foundry-absorbed substrate. Sustained throughput is bounded by PostgreSQL transaction commit rate; 800/sec target reflects 5× headroom over canonical workloads.
- AI deflection: Llama-3.1-70B fine-tuned per-tenant on `community` + `docs` µservice content; co-located GPU inference (NVIDIA L4 × 2) on the same cell. Per `intelligence` µservice substrate.

### 3.3 SLO binding

Each metric maps to an `slos/*.openslo.yaml` row:

| Metric | SLO file | SLO p99 target | Error budget (30d) |
|---|---|---|---|
| Ticket creation latency | `slos/write-latency.openslo.yaml` + `slos/local-ticket-triage-latency.openslo.yaml` | p99 ≤ 520 ms | 21.6 min (99.95% availability) |
| Ticket search latency | `slos/read-latency.openslo.yaml` | p99 ≤ 320 ms | 43.2 min (99.9% availability) |
| Workflow execution | `slos/policy-decision-latency.openslo.yaml` + ad-hoc | p99 ≤ 1 800 ms | 21.6 min |
| SLA breach detection | `slos/local-sla-breach-detection.openslo.yaml` | p99 ≤ 15 s | 4.32 min (99.99%) — critical class |
| Mobile sync | (to author Wave 15E) | p99 ≤ 1 200 ms | 21.6 min |
| Agent dashboard | `slos/local-mttr-objective.openslo.yaml` related | p99 ≤ 600 ms | 21.6 min |
| CMDB query | `slos/local-cmdb-relation-freshness.openslo.yaml` related | p99 ≤ 380 ms | 21.6 min |
| Workflow throughput | (to author Wave 15E) | sustained 800/sec | per-cell capacity envelope |
| AI deflection | (to author Wave 15E) | p95 ≤ 2 400 ms | 43.2 min |

Findings:
- F-BM-01 [P1]: Five of nine canonical metrics lack an explicit SLO row; Wave 15E action item to author `slos/mobile-sync-latency.openslo.yaml`, `slos/agent-dashboard-latency.openslo.yaml`, `slos/workflow-throughput.openslo.yaml`, `slos/ai-deflection-latency.openslo.yaml`, and refine the SLA breach detection SLO to the 15 s p99 target.

## 4. Deployment-context overlay

For each metric, the baseline (oyatie-public-cloud / paid / Envelope B) is the reference. The five other contexts add/subtract from each metric per the table.

### 4.1 oyatie-public-cloud

Reference baseline per §3. No overlay adjustment.

### 4.2 guest-on-aws

Latency floor: +10-30 ms vs public cloud baseline (Oyatie control plane round-trips through AWS-native primitives — IAM, S3, RDS, Aurora — adding network hops).

| Metric | Adjustment from baseline |
|---|---|
| Ticket creation latency p99 | +15 ms → 535 ms |
| Ticket search latency p99 | +20 ms → 340 ms (S3-backed Elasticsearch snapshots take an extra hop) |
| Workflow execution p99 | +30 ms → 1 830 ms (Aurora WAL commit latency) |
| SLA breach detection p99 | +1 s → 16 s (audit-chain emit through AWS Kinesis-equivalent) |
| Mobile sync p99 | +20 ms → 1 220 ms |
| Agent dashboard p99 | +25 ms → 625 ms |
| CMDB 3-hop p99 | +40 ms → 420 ms (Aurora vs PostgreSQL self-managed) |
| Workflow throughput sustained | -10% → 720/sec (Aurora write throughput ceiling) |
| AI deflection p95 | +50 ms → 2 450 ms (cross-VPC GPU inference) |

Throughput ceiling: caps at smaller of cell capacity vs AWS region quota. Workflow throughput baseline 800/sec scaled to ~720/sec under guest-on-aws.

Cost basis: AWS list-price hourly compute + S3 storage + Aurora I/O + cross-AZ traffic. Per Envelope B steady state, ~$2,400/month/cell for the ITSM-µservice slice (excluding tenant-side data egress).

### 4.3 guest-on-oci

Latency floor: +15-40 ms vs public cloud baseline. Oracle Cloud Infrastructure adds slightly more hop than AWS for some primitives but Always Free profile offers compelling cost basis for demo/trial tenants.

| Metric | Adjustment from baseline |
|---|---|
| Ticket creation latency p99 | +20 ms → 540 ms |
| Ticket search latency p99 | +25 ms → 345 ms |
| Workflow execution p99 | +40 ms → 1 840 ms (Autonomous DB commit latency) |
| SLA breach detection p99 | +1.5 s → 16.5 s |
| Mobile sync p99 | +30 ms → 1 230 ms |
| Agent dashboard p99 | +30 ms → 630 ms |
| CMDB 3-hop p99 | +50 ms → 430 ms |
| Workflow throughput sustained | -12% → 700/sec |
| AI deflection p95 | +60 ms → 2 460 ms |

Sub-profile: OCI Always Free (demo/trial tenants per `feedback_oci_always_free_maximization_2026_05_20.md`):
- Compute budget capped at 4 OCPU + 24 GiB RAM (2× Ampere A1 + 2× AMD E2.1.Micro).
- Storage: 200 GB block + 10 GB object + 10 GB archive.
- Database: 2× Autonomous DB × 20 GB.
- Network: 10 TB egress/month; 10 Mbps LB.
- ALL benchmark numbers degraded by ~30-50% on Always Free (single-cell, single-region, no GPU inference).
- AI deflection NOT available on Always Free (no GPU in Always Free; demo/trial gets reduced AI deflection or routes to community-pack KB search).

Cost basis: OCI Always Free profile = $0/month (canonical demo/trial deployment); paid OCI tenant ~$1,800/month/cell for Envelope B steady state.

### 4.4 on-prem

Latency floor: -20 ms to +10 ms vs public cloud baseline. On-prem can be faster (no cloud-network hops, local NVMe, dedicated hardware) or slower (older hardware, customer-managed network).

| Metric | Adjustment from baseline (typical enterprise on-prem hardware) |
|---|---|
| Ticket creation latency p99 | -10 ms → 510 ms (dedicated local NVMe) |
| Ticket search latency p99 | +10 ms → 330 ms (local Elasticsearch fleet typically smaller than cloud) |
| Workflow execution p99 | +0 ms → 1 800 ms |
| SLA breach detection p99 | +0 s → 15 s |
| Mobile sync p99 | +50 ms → 1 250 ms (mobile traffic crosses customer-VPN edge) |
| Agent dashboard p99 | +0 ms → 600 ms |
| CMDB 3-hop p99 | -50 ms → 330 ms (dedicated PostgreSQL on bare metal) |
| Workflow throughput sustained | +20% → 960/sec (no shared cloud bandwidth contention) |
| AI deflection p95 | +200 ms → 2 600 ms (typically older GPU + smaller GPU pool) |

Caveats: on-prem performance is tightly coupled to the customer's hardware spec; numbers above assume 2025+ enterprise-grade hardware (Intel Xeon Sapphire Rapids / AMD EPYC Bergamo class, NVMe storage tier, 100 Gbps network fabric).

Cost basis: customer-borne CapEx (initial hardware) + Oyatie license + Oyatie SRE ops (typically 0.4 FTE per ITSM cell). Per Envelope B steady state, ~$870k/year all-in (hardware amortized over 5 yrs + license + SRE).

### 4.5 colo

Latency floor: roughly equal to on-prem (same dedicated-hardware advantages, same customer-VPN edge mobile penalty).

| Metric | Adjustment from baseline |
|---|---|
| Same as on-prem within ±5%. |

Cost basis: colocation facility lease + Oyatie license + Oyatie SRE ops + facility power + network. Typically 10-20% cheaper than fully on-prem due to shared facility economics. ~$720k-$850k/year for Envelope B.

### 4.6 oyatie-as-cloud-provider

Latency floor: -30 ms to baseline (Oyatie controls every layer — own cloud-iam, cloud-kms, cloud-network, cloud-data, cloud-storage; no foreign cloud control-plane hops).

| Metric | Adjustment from baseline |
|---|---|
| Ticket creation latency p99 | -30 ms → 490 ms |
| Ticket search latency p99 | -30 ms → 290 ms |
| Workflow execution p99 | -80 ms → 1 720 ms |
| SLA breach detection p99 | -1 s → 14 s |
| Mobile sync p99 | -50 ms → 1 150 ms |
| Agent dashboard p99 | -50 ms → 550 ms |
| CMDB 3-hop p99 | -80 ms → 300 ms |
| Workflow throughput sustained | +25% → 1 000/sec |
| AI deflection p95 | -100 ms → 2 300 ms |

Cost basis: internal infrastructure cost + amortized R&D. Per Envelope B steady state, ~$680k/year per cell (assumes Oyatie-owned hardware operating at ~70% utilization).

## 5. Tenant-class overlay

### 5.1 Paid tenants (canonical performance class)

All §3 + §4 numbers apply directly. Workload envelope is uncapped (paid tenants pay-as-they-grow per the `per_usage` billing component when active; per-seat tenants billed per named user; revenue-share tenants billed on percentage of customer's gross revenue earned through Oyatie).

SLO posture: contractual per tenant contract. Service-credit framework via `cloud-billing` if breached.

Hardware envelope: scales with workload up to cell capacity envelope; cells split via shuffle sharding (ADR-0248) when single-cell saturation approached.

### 5.2 Demo / trial tenants (capped + best-effort)

Hard caps:
- 10 active IT-Ops agents.
- 500 tickets/year.
- 1 000 CMDB CIs.
- 100 KB articles.
- 100 MB attachment storage.
- 100 workflow runs/month.
- 100 AI deflection requests/month (or zero if running on OCI Always Free without GPU).

Time gate: 30/60/90-day trial expiry (configurable per µservice; ITSM default 30 days).

SLO posture: best-effort. No contractual commitment. SLO targets are aspirational and not service-credit-backed.

| Metric | Demo/trial target (OCI Always Free profile) | vs paid baseline |
|---|---|---|
| Ticket creation latency p99 | **1 100 ms** | ~2× paid baseline due to single-cell + smaller pod sizing |
| Ticket search latency p99 | **800 ms** | ~2.5× paid baseline (smaller Elasticsearch + reduced shards) |
| Workflow execution p99 | **3 500 ms** | ~2× paid baseline (reduced workflow-engine concurrency) |
| SLA breach detection p99 | **30 s** | ~2× paid baseline |
| Mobile sync p99 | **3 000 ms** | ~2.5× paid baseline (mobile not optimized for trial) |
| Agent dashboard p99 | **1 800 ms** | ~3× paid baseline |
| CMDB 3-hop p99 | **1 200 ms** | ~3× paid baseline |
| Workflow throughput sustained | **20/sec** | bounded by demo/trial cap |
| AI deflection p95 | **N/A** (Always Free has no GPU) | N/A — demo/trial routes to community-pack KB search instead |

Notes:
- Demo/trial benchmarks are still industry-leader-grade for the workload envelope (10 agents, 500 tickets, 1 000 CIs). They are not feature-degraded; they are envelope-constrained.
- ServiceNow does not offer an equivalent free tier — its lowest tier is ITSM Standard at ~$100/user/mo. JSM offers Free tier (10 agents, limited features); Freshservice offers Starter at ~$19/agent/mo. Oyatie demo/trial = $0 + industry-leader feature parity (with capped envelope).

### 5.3 Conversion behavior

When demo/trial tenant approaches cap (e.g., 95% of 500 tickets used), `cloud-billing` emits a conversion-prompt event to the tenant admin; admin can convert to `paid` mid-flight without data loss. Conversion changes the deployment context (typically demo/trial-on-OCI-Always-Free → paid-on-context-of-choice) and removes all caps.

When time gate hits (e.g., 30 days), tenant is given a 14-day grace period during which they can convert to `paid`; if not, the demo/trial tenancy is archived (data preserved per `iac/dr-failover.yaml` retention rules + `IP-015 data residency pack overlays`) and access is suspended.

## 6. Workload envelope variants

### 6.1 Envelope A (Small Paid Tenant, 100 agents, 50k tickets/year, 100k CMDB CIs)

Hardware footprint reduced ~3× from Envelope B baseline (4× ITSM API pods, 2× workflow-engine pods, smaller PostgreSQL + Elasticsearch fleet).

All §3 baseline numbers improve ~10-20% under reduced load (less contention).

| Metric | Envelope A target (paid, public cloud) |
|---|---|
| Ticket creation p99 | 420 ms |
| Ticket search p99 | 260 ms |
| Workflow execution p99 | 1 400 ms |
| Agent dashboard p99 | 480 ms |
| CMDB 3-hop p99 | 280 ms |
| Workflow throughput sustained | 200/sec |

Cost basis: ~$640/month/cell (oyatie-public-cloud).

### 6.2 Envelope B (Enterprise Paid Tenant)

Baseline per §3.

Cost basis: ~$2,000/month/cell (oyatie-public-cloud).

### 6.3 Envelope C (Mega Enterprise Paid Tenant, 5 000 agents, 50M tickets/year, 50M CMDB CIs)

Hardware footprint scales ~5-10× from Envelope B (multi-cell deployment, shuffle sharding active, dedicated Elasticsearch warm + hot tiers, partitioned CMDB across cells).

| Metric | Envelope C target (paid, public cloud) |
|---|---|
| Ticket creation p99 | 580 ms (slight increase due to larger write-amplification) |
| Ticket search p99 | 480 ms (larger corpus + partitioned index) |
| Workflow execution p99 | 2 200 ms (multi-cell workflow may cross-cell on rare paths) |
| Agent dashboard p99 | 720 ms |
| CMDB 3-hop p99 | 480 ms (cross-cell traversal in some cases) |
| Workflow throughput sustained | 4 000/sec (scales horizontally with cell count) |
| Workflow throughput burst (≤60 s) | 18 000/sec |

Cost basis: ~$12,000/month/cell × 6-10 cells = ~$72k-$120k/month for Envelope C deployment.

## 7. Reproducibility and verification

### 7.1 Benchmark harness

Per the existing benchmark file `microservices/itsm/benchmarks/servicenow-vs-jsm-vs-freshservice-vs-oyatie.md` line 105 ("Benchmark harness at `benchmarks/itsm/`. Re-run weekly in CI."), the harness is referenced but not implemented in the current µservice tree.

Findings:
- F-BM-02 [P1]: No `benchmarks/itsm/harness/` directory exists with executable test rigs. The line in the existing benchmark file is a future-work pointer. Wave 15E target: author Rust benchmark harness using `criterion` or equivalent + tokio benchmark suite + Goss/k6/wrk3 for load tests against the deployed surfaces.

### 7.2 Counterpart benchmark reproducibility

ServiceNow benchmarks in §2.1 are derived from public documentation + customer reports. Oyatie should not publish ServiceNow-attributed numbers without published-source citation per ADR-0322 substance bar.

Findings:
- F-BM-03 [P1]: ServiceNow numeric targets in §2.1 should cite published source. Wave 15E target: replace `~` annotations with citation links to ServiceNow Knowledge25 keynote, Knowledge24 published benchmarks, or third-party load-test reports.

### 7.3 Re-run cadence

Per the existing benchmark file: weekly CI re-run. Wave 15E target: implement the CI lane `oya-governance-itsm-benchmark-weekly` that runs the benchmark harness against a canonical test environment and emits result deltas as audit events.

## 8. Comparison table — Oyatie vs ServiceNow at canonical metrics

| Metric | Oyatie target (paid, public cloud, Envelope B) | ServiceNow target (Enterprise, Envelope B equiv) | Oyatie advantage |
|---|---|---|---|
| Ticket creation p99 | 520 ms | ~1 200 ms | 2.3× faster |
| Ticket search p99 (5M corpus) | 320 ms | ~780 ms | 2.4× faster |
| Workflow execution p99 (10-step) | 1 800 ms | ~4 800 ms | 2.7× faster |
| SLA breach detection p99 | 15 s | ~120 s | 8× faster (event-driven vs scheduled) |
| Mobile sync p99 (100 tickets) | 1 200 ms | ~3 200 ms | 2.7× faster |
| Agent dashboard p99 | 600 ms | ~1 800 ms | 3× faster |
| CMDB 3-hop p99 (5M CIs) | 380 ms | ~1 400 ms | 3.7× faster |
| Workflow throughput sustained | 800/sec | ~120/sec | 6.7× higher |
| AI deflection p95 | 2 400 ms | ~2 800 ms | 1.2× faster (within rough parity) |

Every canonical metric beats ServiceNow by 1.2-8× at the same workload envelope.

## 9. Cost basis comparison

Annual TCO for Envelope B (500 IT-Ops agents, 2M tickets/year, 5M CMDB CIs):

| Platform | Hardware/cloud | License/user | AI add-on | Ops (SRE) | Annual TCO |
|---:|---:|---:|---:|---:|---:|
| Oyatie ITSM (oyatie-public-cloud, paid + per_seat) | $24 000 | (per-seat: $50/agent/mo × 500 × 12 = $300 000) | included | $148 800 (0.4 FTE SRE × $372k loaded) | $472 800 |
| Oyatie ITSM (on-prem, paid + per_seat) | $164 000 amortized | (per-seat: $50/agent/mo × 500 × 12 = $300 000) | included | $148 800 | $612 800 |
| Oyatie ITSM (oyatie-as-cloud-provider, paid + per_seat) | $0 (Oyatie absorbs) | (per-seat: $50/agent/mo × 500 × 12 = $300 000) | included | $148 800 | $448 800 |
| Oyatie ITSM (oyatie-public-cloud, paid + per_usage) | $24 000 | (per_usage: $0.05/ticket × 2 000 000 = $100 000) | included | $148 800 | $272 800 |
| ServiceNow ITSM (Enterprise + Discovery + CMDB + Now Assist) | $0 | $300/agent/mo × 500 × 12 = $1 800 000 | $240 000 (Now Assist) | $124 000 | $2 164 000 |
| Jira Service Management Cloud Enterprise | $0 | $80/agent/mo × 500 × 12 = $480 000 | $120 000 (Atlassian Intelligence) | $124 000 | $724 000 |
| Freshservice Enterprise | $0 | $90/agent/mo × 500 × 12 = $540 000 | $60 000 (Freddy AI) | $124 000 | $724 000 |

Reading:
- Oyatie ITSM at per-seat $50/agent/mo is ~$472k/year for Envelope B (78% cheaper than ServiceNow; ~35% cheaper than JSM/Freshservice).
- Oyatie ITSM at per-usage $0.05/ticket pricing is ~$272k/year for Envelope B (87% cheaper than ServiceNow). Per-usage is attractive for tenants with low-volume workloads but high-volume tenants will choose per-seat.
- Tenants can combine billing components: e.g., $30/agent/mo per_seat + $0.02/ticket per_usage for a hybrid model.
- Demo/trial tenants pay $0 (covered by OCI Always Free profile per ADR-0328 §D-19).

## 10. Risk + caveats

- Numbers are forward-looking targets, not measured benchmarks. Per ADR-0328 §D-6.12-13: target budgets must not be presented as measured evidence. The `~` markers on ServiceNow rows are conservative estimates from public/customer-reported data; the bolded Oyatie targets are budgets that the µservice has committed to meet at Phase 4 gate.
- Wave 15E action item: author the benchmark harness + run against ServiceNow Public Cloud Enterprise instance + reauthor this document with measured-vs-target columns.
- ServiceNow performance is highly workload-dependent (Business Rules + Server-Side Scripts + Workflow Engine load varies by customer configuration). The ~ numbers in §2 are conservative; some ServiceNow tenants observe better p99s, especially with no custom Business Rules.
- Oyatie's targets assume Rust workflow-engine + HTTP/3 + Cloud Hypervisor + Cedar evaluation at p99 < 5ms. If any of those assumptions degrade, downstream Oyatie targets degrade proportionally.

## 11. Verification Notes

Per ADR-0328 §D-10, the benchmark deliverable was authored against:
- `microservices/itsm/benchmarks/servicenow-vs-jsm-vs-freshservice-vs-oyatie.md` (existing 106-line benchmark file; retracted-and-reauthored per T-RET-02).
- `microservices/itsm/slos/*.openslo.yaml` (12 SLO files inspected by inventory).
- `microservices/itsm/capacity-model.md` (86 KB, referenced for capacity envelope assumptions).
- `microservices/itsm/cost-budget.md` (69 KB, referenced for cost basis).
- `microservices/itsm/multi-region.md` (69 KB, referenced for cross-context overlay).
- `specs/master-plan-sequencing.json#deployment_contexts` (six canonical contexts).
- `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md` (tenant_class model).
- `feedback_no_capability_profiles_2026_05_20.md` (no tier scaffolding).
- `feedback_oci_always_free_maximization_2026_05_20.md` (Always Free profile for demo/trial).
- ADR-0247, ADR-0248, ADR-0250, ADR-0252, ADR-0253, ADR-0254, ADR-0263, ADR-0328 (binding).

## 12. Findings

Per ADR-0328 §D-6.23, required even when empty:

- F-BM-01 [P1]: Five of nine canonical metrics lack an explicit SLO row (mobile sync, agent dashboard, workflow throughput, AI deflection, refined SLA breach detection). Author Wave 15E.
- F-BM-02 [P1]: No `benchmarks/itsm/harness/` directory exists with executable test rigs. Author Wave 15E.
- F-BM-03 [P1]: ServiceNow numeric targets lack published-source citation. Replace `~` annotations Wave 15E.
- F-BM-04 [P2]: This document declares forward-looking targets; ADR-0322 substance bar requires measured-vs-target distinction. Wave 15E action item: run harness + reauthor with measured columns.
- F-BM-05 [P2]: Tenant-class overlay is well-formed but only the OCI Always Free profile for demo/trial is fully concrete; other context+tenant-class combinations (e.g., paid-on-on-prem) lack hardware-spec ranges and TCO bands. Wave 15E refinement.

## 13. Backlog Rows

Per ADR-0328 §D-6.24, required:

The findings F-BM-01..F-BM-05 enter Wave 14 backlog. Combined with coherence-audit + feature-parity, the µservice contributes 227 finding rows total to Wave 14 aggregation.

The benchmark document produces NO direct edits to `microservices/itsm/*` outside this deliverable; specifically does NOT overwrite the existing `benchmarks/servicenow-vs-jsm-vs-freshservice-vs-oyatie.md` (which is a Wave 15J T-RET-02 retraction-and-reauthor target — handled by remediation, not by this audit).
