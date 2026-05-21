# Oyatie canonical architecture (2026-05-21)

**Single source of truth (current).** Authoring date 2026-05-21. Maintainer: orchestrator session 8f603fc7. Status: ACTIVE.

**Relationship to existing docs:**
- `/docs/DESIGN.md` — pre-existing 811-line architecture doc from 2026-05-09. Authoritative-deep intent but DRAFT v0.1; pre-dates the 14-ADR keystone bundle (ADR-0242..0255), Wave 15 doctrine (ADR-0329..0336), foundry retirement, cell retirement, shorts retirement. **This file supersedes DESIGN.md for areas where they conflict** (foundry-as-axis-4 is now wrong; intelligence two-layer replaces it).
- `/specs/platform-architecture.json` — 1,598-line machine-readable canonical spec. Per-update reference for structural fields.
- `/docs/architecture/diagrams/*.md` — per-flow diagram docs (ai-substrate-two-layer / cedar-policy-evaluation / cell-routing-shuffle-sharding / audit-chain-emission-pipeline / etc.).

Cross-reference these for depth. This file is the QUICK-REFERENCE + AUTHORITY-CHAIN summary current as of 2026-05-21.

This document codifies the complete Oyatie architecture so future agents (and future-me) can ground reliably without re-deriving from scattered ADRs. Every claim cites authority. Conflicts resolve via §22 authority chain.

---

## §1. Mission + product positioning

Oyatie is a **B2B unified-ecosystem platform** that displaces per-department SaaS through a single integrated suite spanning agentic + dev + business + healthcare + supply-chain + delivery domains. Industry-leader quality bar (Stripe / Palantir / Linear / Salesforce / Snowflake / etc.). Hyperscaler-grade performance + horizontal scalability + multi-context (AWS-guest / OCI-guest / on-prem / colo / Oyatie-as-cloud-provider).

**Workflow Studio** is the n8n-class first hero product. Multi-domain. Both shared substrate (engine) AND end-user product (visual editor).

Authority: `feedback_quality_performance_scalability_bar` + `feedback_workflow_studio_scope` + ADR-0212 (buildability doctrine) + ADR-0213 (ecosystem-as-a-service).

---

## §2. Architecture invariants (keystone decisions)

These are non-negotiable. Every µservice, every ADR, every IP must respect them.

| Invariant | Authority |
|---|---|
| oyatie-is-a-tenant (no carve-outs; `oyatie` is a reserved-namespace tenant) | ADR-0242 |
| Cedar universal gate (every policy decision = Cedar eval; no policy in code) | ADR-0243 |
| Tenant scoping primitive (every row/audit/cost carries tenant context) | ADR-0244 |
| Substrate vs product layering (substrate µservices serve all products; no duplication) | ADR-0245 |
| MLS RFC 9420 E2EE for messenger | ADR keystone #5 |
| Self-modification doctrine (Foundry→Intelligence runs as oyatie.foundry.* under Cedar) | ADR-0247 |
| Amazon-shape cellular architecture (Tiers 0-4 + shuffle sharding + Cloud Hypervisor + Kata) | ADR-0248 + ADR-0254 |
| Multi-category marketplace (plugins / apps / workflows / agents / models / datasets) | ADR-0249 |
| Build-ahead-of-certification (certified shape day-one; no retrofit) | ADR-0250 |
| Compliance packs (HIPAA/GDPR/SOC2/CSAP/PCI/EU-AI-Act per tenant + per cell) | ADR-0251 |
| HLC default; TrueTime opt-in for fin-grade | ADR-0252 |
| HTTP/3 + QUIC default everywhere; gRPC over HTTP/3 | ADR-0253 |
| K8s + Cloud Hypervisor + Kata pods (except edge → Talos minimal-K8s) | ADR-0254 |
| Intelligence two-layer substrate (AI Substrate + Consumer Brand Surface; absorbs Foundry) | ADR-0255 |
| Per-µservice flat layout (`microservices/<ms>/src/`) | ADR-0131 |
| No-suite policy (one concern per µservice) | ADR-0132 |
| 13-layer canonical enum (kernel/domain/usecase/adapter/api/app/infrastructure/cli/rest/grpc/graphql/sdk/worker) | ADR-0105 |
| Tier system retired (replaced by tenant_class) | ADR-0329 |
| Tenant class = {demo_trial, paid} + composable billing_components (revenue_share, per_seat, per_usage) | ADR-0330 |
| Cross-µservice tenant_class adoption template | ADR-0331 |
| Healthcare domain decomposition (7 µservices: EMR/diagnostics/emergency/pharmacy/patient-monitoring/imaging + healthcare-integration broker) | ADR-0332 |
| cell µservice RETIRED (cellular is a pattern; absorbed into tenancy + cloud-iac + observability + oya-shuffle-sharding crate) | ADR-0333 |
| shorts µservice RETIRED (merged into social) | ADR-0334 |
| foundry µservice RETIRED (absorbed by intelligence per ADR-0255 KS#14) | ADR-0335 |
| Valkey not Redis (LF BSD fork; Redis Inc. relicensed to SSPL/RSALv2 March 2024) | ADR-0336 |
| BYOK opt-in for LLM/provider credentials (NOT encryption — that's ADR-0251 §D-10) | ADR-0255 §D-4 |

---

## §3. Languages

| Concern | Language | Authority |
|---|---|---|
| Backend / µservice / scripting | **Rust (strict)** — no exceptions without per-µservice ADR | `feedback_rust_strict_only_no_python_2026_05_20` + ADR-0211 |
| iOS / macOS frontend | **Swift** | feedback memory |
| Android frontend | **Kotlin** | feedback memory |
| Windows frontend | **WinUI 3 (C#/.NET)** | feedback memory |
| Web frontend | **Leptos (Rust → WASM)**, SSR + selective island hydration | `feedback_cell_standalone_network_merges_community_2026_05_21` |
| IaC | **OpenTofu HCL** (NOT Terraform; Linux Foundation BSL-replacement) | `feedback_zero_handroll_opentofu_only_2026_05_20` |
| Policy | **Cedar** (AWS Permissions Language; Apache-2.0) | ADR-0150 + ADR-0243 |
| Schemas | **OpenAPI 3.2.0** + **AsyncAPI 3.1.0** + **proto3** | canonical-primitives.md |
| SLO authoring | **OpenSLO 1.0 YAML** | ADR-0130 + ADR-0245 |
| SQL migrations | PostgreSQL 17 dialect | ADR-0028 |
| SDK generation targets | TS / Python / Go / Java / Kotlin / Swift / Rust / .NET-C# / **C** / **C++** (Ruby/PHP/Elixir DROPPED 2026-05-20; C/C++ added for embedded/IoT/HFT/games/system programming) | `feedback_developer_sdk_stainless_generator_2026_05_20` |
| **FORBIDDEN backend** | Python · JS-app-logic · Ruby · Perl · PHP · Java · Scala · Groovy · Go · F# | same |

---

## §4. Substrate (Class C in-house wherever possible)

| Concern | Choice | Version | Authority |
|---|---|---|---|
| In-memory KV / cache / pubsub / streams | **Valkey** (LF BSD fork — NOT Redis) | 8.x | **ADR-0336** |
| Relational DB | **PostgreSQL** | 17 LTS | ADR-0028 |
| Postgres connection pool | **pgcat** | 1.1+ | ADR-0179 |
| Postgres sharding | **Citus** for horizontal scale | latest | ADR-0028 |
| OLAP / lakehouse | **Apache Iceberg** + Delta + Hudi + ClickHouse + Photon-class engine | latest | data-warehouse µservice IPs |
| Object storage | **S3 protocol** (provider-agnostic; AWS S3 / OCI Object / MinIO on-prem) | latest | multi-context |
| Vector DB | **Milvus** | latest | ADR-0192 |
| Streaming bus | **Kafka** (canonical) + **NATS JetStream** | latest | ADR-0050 |
| Search | **OpenSearch** (Apache 2.0) | latest | dependency-policy.md |
| Secrets | **OpenBao** (LF fork of Vault BSL-1.1 — Class C, BSD-licensed) | latest | ADR-0211 + cloud-secrets µservice |
| HSM / KMS | tenant BYOK to KMS (AWS KMS / OCI Vault / on-prem); provider-BYOK opt-in | — | ADR-0255 §D-4 + ADR-0251 §D-10 |
| WASM sandbox | **Wasmtime** + Component Model | latest | ADR-0200 |

**FORBIDDEN substrate:** Redis (relicensed 2024-03); Consul (HashiCorp BSL); Terraform (HashiCorp BSL); Vault (HashiCorp BSL); DragonflyDB (BSL-1.1).

---

## §5. Service mesh + network (CANONICAL — DO NOT DRIFT)

**Layered Istio Ambient + Envoy Gateway + Cilium.** This is the single canonical mesh stack.

```
NORTH-SOUTH (edge)
    ↓
Envoy Gateway — edge tier (NO identity decisions)
  Concerns: IP / rate / WAF / bot / DDoS / geo / ASN
  Same Envoy data plane as Istio Ambient (uniform config + observability)
  Authority: ADR-0044 + ADR-0182 + ADR-0157 + ADR-0191 edge tier
    ↓
EAST-WEST (origin, per-namespace waypoint)
    ↓
Istio Ambient waypoint — origin tier (identity-context only)
  Cedar PDP via ext_authz filter
  Concerns: principal / action / resource / tenant / data-class / residency / step-up / ACR
  mTLS terminate
  Carries cell_id header (ADR-0009)
  Authority: ADR-0044 + ADR-0148 + ADR-0183 + ADR-0191 origin tier
    ↓
EAST-WEST (per-node data plane)
    ↓
ztunnel (per-node, Ambient mode — NO sidecar)
  Authority: ADR-0044 §Istio Ambient mode
    ↓
NETWORK / CNI
    ↓
Cilium — L3/L4 + ClusterMesh + L7 egress
  Concerns: NetworkPolicy / ClusterMesh per-cell trust bundle / L7 egress (air-gap) / Hubble flow obs
  Authority: ADR-0148
```

### §5.1 Critical mesh invariants

1. **Istio is Ambient mode, NOT sidecar.** Sidecar mode forbidden per ADR-0044.
2. **Envoy is gateway-class only at the edge.** Same data plane as Istio Ambient = uniform config.
3. **Cilium is CNI + L3/L4 + ClusterMesh.** NOT a replacement for Istio. They LAYER, not substitute.
4. **No concern enforced at both edge AND origin tiers.** Disjoint per ADR-0191.
5. **Cedar PDP lives at Istio Ambient waypoint's ext_authz filter.** NOT at edge. NOT in code. Per ADR-0183 + ADR-0243.
6. **mTLS everywhere by default.** Per-traffic-type opt-out only via ADR. SPIFFE per-pod identity.
7. **Per-cell namespace.** One Istio namespace per cell. Cross-cell traffic explicit + Cedar-policied + audit-chained.

### §5.2 Observability for the mesh

- **Hubble** (Cilium L3/L4 flow observability) → ships via OTel → ADR-0186
- **ztunnel + waypoint** (Istio Ambient telemetry) → ships via OTel → ADR-0186
- Edge Envoy access logs → ships via OTel logs receiver → ADR-0186

### §5.3 Forbidden mesh choices

- ❌ Istio sidecar mode (Ambient is canonical)
- ❌ Linkerd (single-stack only; not CNI-layered)
- ❌ Consul Connect (HashiCorp BSL)
- ❌ Sidecar-based Envoy (sidecar bad)
- ❌ Calico without Cilium
- ❌ AWS App Mesh / Azure Service Fabric Mesh (not provider-agnostic)
- ❌ Kuma (incomplete OSS coverage)

Authority memory: [[service-mesh-istio-ambient-envoy-cilium-2026-05-21]]

---

## §6. Transport + RPC

| Concern | Choice | Authority |
|---|---|---|
| Default protocol | **HTTP/3 + QUIC** | ADR-0253 |
| Inter-µservice RPC | **gRPC over HTTP/3** (NO forced workflow/ontology adapter) | ADR-0145 |
| Auth-N transport | **mTLS** with SPIFFE per-pod identity | ADR-0028 |
| Webhook ingress | webhook-driven receiver (foundry pipeline legacy → intelligence post-ADR-0335) | ADR-0112 |
| Time | **HLC** default; **TrueTime opt-in** for fin-grade | ADR-0252 |
| Causality | HLC per call; CRDT where needed | same |

---

## §7. Compute + orchestration

| Concern | Choice | Authority |
|---|---|---|
| Container runtime | **Cloud Hypervisor + Kata** | ADR-0254 |
| Orchestration baseline | **Kubernetes** = **kubeadm + containerd** on enterprise Linux distros (Oracle Linux / RHEL / SUSE / Ubuntu LTS / Debian / Rocky / AlmaLinux / CentOS Stream / Amazon Linux / Flatcar / Photon) | ADR-0121 + ADR-0254 |
| Edge profile only | **Talos** minimal-K8s (NOT the only on-prem option — just one minimal-K8s edge variant) | OS matrix |
| Dev / laptop | **macOS Apple Silicon M5+** (kind / k3d / minikube for local) | OS matrix |
| Cell topology | **AWS-shape cellular** Tiers 0-4 + **shuffle sharding** | ADR-0248 + `oya-shuffle-sharding` crate (ADR-0333) |
| Service mesh | Istio Ambient + Envoy + Cilium (see §5) | ADR-0044 + ADR-0148 |
| Admission policy | **Kyverno** | ADR-0117 + ADR-0183 |

---

## §8. Identity + Auth

| Concern | Choice | Authority |
|---|---|---|
| AuthN protocol | **WebAuthn / Passkeys** | ADR-0188 |
| AuthZ engine | **Cedar** universal gate | ADR-0243 |
| AuthZ enforcement point | Istio Ambient waypoint `ext_authz` filter | ADR-0183 |
| Tenant scoping | tenant_id on every row + audit + cost | ADR-0244 |
| Tenant class | `{demo_trial, paid}` | ADR-0329 + ADR-0330 |
| Tenant billing components | subset of `{revenue_share, per_seat, per_usage}` | ADR-0330 |
| Self-modification | Foundry → Intelligence runs under `oyatie.foundry.*` Cedar principal | ADR-0247 |
| Step-up authn | Cedar context: ACR (Authentication Context Class) | ADR-0191 |

---

## §9. AI substrate — Intelligence (two-layer)

Per ADR-0255 §D-1 (keystone #14 of 14). The `microservices/intelligence/` µservice is restructured into two layers:

### §9.1 Layer A — AI Substrate (9 BCs, audience-neutral)

| BC | Crate | Responsibility |
|---|---|---|
| `transport` | `oya-intelligence-transport` | Provider-agnostic LLM + multi-modal call dispatch |
| `credential-resolver` | `oya-intelligence-credential-resolver` | SecretReference at call time; never materializes credentials at rest |
| `policy-engine-client` | `oya-intelligence-policy-engine-client` | Wraps `oya-shared-policy-engine-client`; builds Cedar request per call |
| `guardrails` | `oya-intelligence-guardrails` | Pre + post-call: prompt-injection detection / PII redaction / toxic refusal / jailbreak success detect |
| `audit-emit` | `oya-intelligence-audit-emit` | Per-call audit rows (IntelligenceDispatch / ToolCall / GuardrailFired / etc.) |
| `tool-registry` | `oya-intelligence-tool-registry` | Discovery surface for tools (MCP / Ontology-defined Functions / internal capabilities) |
| `audience-policy-router` | `oya-intelligence-audience-policy-router` | Routes audience tag through call; selects guardrail bundle + audit stream + cost center |
| `cost-attribution` | `oya-intelligence-cost-attribution` | Per-call cost computation + attribution to cost center |
| `eval` (absorbed from Foundry) | `oya-intelligence-eval` | Golden-set evaluations + multispectrum review v2.4.0 fan-out |

### §9.2 Layer B — Consumer Brand Surface (6 BCs, consumer-only)

| BC | Crate | Responsibility |
|---|---|---|
| `prompt-history` | `oya-intelligence-prompt-history` | Per-user persistent prompt history; DSAR subject |
| `consent-cascade` | `oya-intelligence-consent-cascade` | Per-tenant + per-user consent state |
| `dsar-cascade` | `oya-intelligence-dsar-cascade` | Article 17 / KR PIPA Article 36 erasure-request handling for AI memory |
| `eu-ai-act-tier-ui` | `oya-intelligence-eu-ai-act-tier-ui` | UI for EU AI Act tier classification (Art. 6/6(2)/50/minimal-risk) |
| `tenant-admin-console-controls` | `oya-intelligence-tenant-admin-console-controls` | Tenant admin controls for AI assist |
| `brand-ux-surface` | `oya-intelligence-brand-ux-surface` | Sparkle icons / streaming-text / model-thinking UX / citation rendering |

Layer B is invoked when `audience ∈ {b2b-tenant-product, b2c-consumer}`. Layer A is always invoked.

### §9.3 Provider adapters (10+)

`transport` BC owns per-provider adapters absorbed from Foundry per ADR-0255 §D-16:

- Anthropic (API + Subscription)
- OpenAI (API + Subscription)
- Google Vertex AI / Generative AI API
- AWS Bedrock
- Azure OpenAI
- vLLM (self-hosted)
- SGLang (self-hosted)
- TensorRT-LLM (self-hosted)
- Apple Foundation Models
- OpenRouter
- Together
- Groq

### §9.4 Embeddings + Fine-tuning

Separate substrate µservices:
- `microservices/embeddings/` per ADR-0255 §D-8
- `microservices/fine-tuning/` per ADR-0255 §D-9

### §9.5 Model serving — external + own-hosted

Both supported. Own-hosted via vLLM / SGLang / TensorRT-LLM with NVIDIA GPU pools or Apple Silicon Foundation Models adapter. Per ADR-0255 §D-10.

### §9.6 RAG — caller-side only

NO Intelligence-side retrieval. Per ADR-0255 §D-7.

### §9.7 Tool calling — MCP

Anthropic Model Context Protocol 2024. Tool-call ingress lives in Ontology's `tool-call-ingress` BC; Intelligence's `tool-registry` is the discovery surface. Per ADR-0255 §D-12.

### §9.8 BYOK credentials

Opt-in provider-BYOK per ADR-0255 §D-4:
- `owner_kind = 'oyatie-subscription'` → oyatie's ToS applies (default for B2C fallback)
- `owner_kind = 'tenant-subscription'` → tenant's ToS applies
- `owner_kind = 'tenant-byok'` → tenant's ToS applies + tenant attests provider-BYOK ToS clearance

NOTE: Provider-BYOK is distinct from encryption-BYOK (tenant KMS/HSM root per ADR-0251 §D-10).

---

## §10. Messenger (E2EE)

| Concern | Choice | Authority |
|---|---|---|
| Group messaging E2EE | **MLS (RFC 9420)** | keystone #5 |
| 1:1 fallback | MLS pairwise group of 2 | same |
| Forward secrecy | MLS continuous-group-key-agreement (CGKA) | RFC 9420 |
| Tenant pack toggles | Per-tenant E2EE policy (e.g. compliance-required key escrow) | ADR-0251 |

---

## §11. Frontend stack + mobile bundle

| Platform | Stack | Authority |
|---|---|---|
| iOS / macOS | Swift native (Apple Silicon M5+ only) | `feedback_rust_strict_only_no_python_2026_05_20` |
| Android | Kotlin native | same |
| Windows | WinUI 3 (C#/.NET) | same |
| Web | **Leptos** (Rust→WASM) SSR + selective island hydration | `feedback_cell_standalone_network_merges_community_2026_05_21` |

**Mobile app bundle**: ONE mobile app per platform = messages (messenger) + email (mail) + social + community. Backend µservices stay DISTINCT.

---

## §12. Deployment contexts (5)

Every µservice ships per-context iac module: `iac/<context>/*.tf`.

| Context | Description | Always Free path |
|---|---|---|
| `aws-guest` | Oyatie hosted on AWS (EKS / RDS / ElastiCache for Valkey / S3 / KMS / IAM) | n/a |
| `oci-guest` | Oyatie hosted on OCI (OKE / Autonomous DB / OCI Cache with Valkey / Object Storage / Vault) + **Always Free maximization** | `iac/oci-guest/always-free/` |
| `on-prem` | Customer-controlled K8s baseline (**kubeadm + containerd**) on **enterprise Linux**: Oracle Linux / RHEL / SUSE / Ubuntu LTS / Debian / Rocky / AlmaLinux / CentOS Stream / Amazon Linux (+ Talos / Flatcar / Photon for minimal-K8s edge variants only). PostgreSQL + Valkey + Milvus + Cilium + Istio Ambient + Envoy. macOS Apple Silicon M5+ for dev/laptop. | n/a |
| `colo` | Same engine as on-prem; customer colocation | n/a |
| `oyatie-as-cloud-provider` | Oyatie's own IaaS µservices (cloud-iac/iam/kms/storage/network/billing) | n/a |

### §12.1 OCI Always Free spec (CORRECTED 2026-05-21)

- **1× Ampere A1 ARM** allocation totalling **4 OCPUs + 24 GB RAM** — splittable into up to 4 VM instances
- **2× AMD VM.Standard.E2.1.Micro** Always Free instances (1 OCPU + 1 GB RAM each)
- 200 GB block volume total
- 10 GB Object Storage standard + 10 GB Archive + 5 GB Storage Gateway
- 2× Autonomous DB × 20 GB (ATP + ADW)
- 1 VCN + 1 Always Free Load Balancer (10 Mbps)
- **10 TB egress per month**
- OCI Vault (3 vaults + 20 keys / region)
- Logging 10 GB / month + Monitoring + Notifications 1M / month
- Streaming 1 partition + 1 GB + 50 GB ingress + 50 GB egress
- Email Delivery 100 / month
- Functions 2M invocations / month
- API Gateway 100M calls / month
- Resource Manager (OpenTofu state backend)

Authority memory: [[oci-always-free-maximization-2026-05-20]] (corrected 2026-05-21).

---

## §13. OS support matrix

**Approved**: Talos · RHEL · Oracle Linux · SUSE · Ubuntu LTS · Debian · Rocky · AlmaLinux · CentOS Stream · Amazon Linux · Flatcar · Photon · macOS **Apple Silicon M5+** (no Intel macOS, no pre-M5).

**Arch matrix**: linux/amd64 + linux/arm64 + darwin/arm64 + Tier-2 ppc64le/s390x.

**Per-OS package format**: RPM / DEB / container / pkg / Homebrew.

**Per-µservice `supported_oses` manifest** + per-OS CI lane mandatory.

Authority: `feedback_os_support_matrix_2026_05_20`.

---

## §14. CI/CD + quality gates

| Gate | Authority |
|---|---|
| **Multispectrum review v2.4.0** (F1-F13 facets + M1+M2 meta + A1-A7 adherence; per-facet subagent) | `feedback_multispectrum_review_v22` + `feedback_multispectrum_adherence_facets` |
| **Reviewer-agent APPROVE + CI green** before auto-merge | `feedback_self_merge_via_contract_path` |
| **lean-a5-doc-coverage** lane (full doc suite per µservice) | ADR-0063 |
| **lean-a10-no-silent-regression** (public contracts protected) | `feedback_no_silent_regression` |
| **Canonical-base neutrality** CI-enforced (per-pack overlays) | ADR-0064 |
| **Anti-template-stamping** | ADR-0324 |
| **No mock-DB in integration tests** (mock/prod divergence caused past incident) | `feedback_no_silent_regression` |
| **Cosign signature verification** on container images + IaC modules | ADR-0181 |
| **Kyverno admission policy** (image signing + Pod Security + label/annotation) | ADR-0183 |

---

## §15. Compliance packs

Per ADR-0251 (KS#8). Per-tenant + per-cell pack toggles:

- HIPAA (PHI + 6-year retention)
- GDPR (Article 17 DSR + Article 30 retention)
- SOC2 Type II
- CSAP (Korea)
- PCI-DSS
- EU AI Act (Article 6 prohibited / Article 6(2) high-risk / Article 50 limited-risk / minimal-risk)
- SOX 404
- ISO 27001
- KR PIPA (Article 36 erasure)

**Build-ahead-of-certification**: certified shape day-one. Never retrofit. Per ADR-0250.

---

## §16. VCS + agent ops

| Concern | Choice | Authority |
|---|---|---|
| Canonical git invocation | **`oya git <subcommand>`** (drop-in for raw git + ledger layer) | canonical-primitives.md + `feedback_oya_git_canonical_2026_05_18` |
| Coordination ratchet | `oya vcs claim/work/verify/done/promote` | same |
| Retired tooling | grit · rtk · icm · vox (per ADR-0116) | same |
| Codex dispatch flags | `codex exec -c model_reasoning_effort=xhigh --skip-git-repo-check --sandbox danger-full-access --dangerously-bypass-approvals-and-sandbox` | **`feedback_codex_dispatch_canonical_2026_05_21`** |
| Branch pipeline | dev (default) → staging → production; auto-promotion 30min/hourly cadence | `project_branch_pipeline_implemented` |
| Agent operating contract | `docs/AGENTS.md` (until PHASE-5 promotes `/specs/agent-operating-contract.json`) | root CLAUDE.md |
| Inherited agent-skills | `tools/agent-skills/` (MIT — Addy Osmani) — lifecycle skills + personas + intent→skill mapping; Oyatie governance OVERLAYS and WINS on conflict | root CLAUDE.md |

---

## §17. Marketplace

Multi-category per ADR-0249 (KS#11):
- plugins
- apps
- workflows
- agents
- models
- datasets

---

## §18. Active µservices (77, post-retirements)

```
analytics, api-gateway, application, audit-chain, calendar, cloud-billing, cloud-billing-tax,
cloud-data, cloud-iac, cloud-iam, cloud-k8s, cloud-kms, cloud-network, cloud-network-dns,
cloud-secrets, cloud-storage, comms-email, community, compliance, connect, consent-graph,
contact-center, contract-lifecycle-management, crm, data-pipeline, data-warehouse,
design-collaboration, detection, developer-sdk, diagnostics, docs, drive, emergency, emr,
feature-flags, financial-planning, finops-portal, forms, global-trade, governance,
healthcare-integration, identity, imaging, incident-management, intelligence, itsm,
learning-management, mail, marketing-automation, marketplace, meet, messenger, notes,
observability, ontology, ops-dashboard-control-center, patient-monitoring, payments,
performance-management, pharmacy, plant-maintenance, plugin-app-store, production-planning,
quality-management, real-estate, recordings, sheets, sites, slides, social,
supply-chain-planning, tasks, tenancy, translate, treasury, warehouse, whiteboard,
workflow-engine, workflow-studio, workplace-integration
```

### §18.1 Healthcare cluster (per ADR-0332)

7 µservices: `emr` + `diagnostics` + `emergency` + `pharmacy` + `patient-monitoring` + `imaging` + `healthcare-integration` (broker for FHIR R5 / HL7v2 / DICOM external interop).

### §18.2 Substrate cluster

Per ADR-0245 substrate-vs-product. Substrate µservices serve all products.
- `cloud-iac` · `cloud-iam` · `cloud-kms` · `cloud-secrets` · `cloud-storage` · `cloud-data` · `cloud-network` · `cloud-network-dns` · `cloud-k8s` · `cloud-billing` · `cloud-billing-tax` · `audit-chain` · `consent-graph` · `compliance` · `governance` · `observability` · `ontology` · `policy` (note: `policy` not currently a µservice; Cedar lives in shared crates) · `tenancy` · `identity` · `intelligence` · `embeddings` (separate per ADR-0255 §D-8) · `fine-tuning` (separate per ADR-0255 §D-9)

### §18.3 Mobile app bundle (one app per platform)

Backend µservices `messenger` + `mail` + `social` + `community` → ONE mobile app per platform (iOS Swift / Android Kotlin / Windows WinUI / Web Leptos). Backend stays DISTINCT.

### §18.4 Community + Social cluster

- **Community** = 4-pillar text-first: Reddit + Teamblind + Handshake + LinkedIn-jobs+profile+InMail (NO LinkedIn feed)
- **Social** = Instagram visual + TikTok short-video flavor (NOT engagement-feed; merged from shorts per ADR-0334)
- Authority: `feedback_cell_standalone_network_merges_community_2026_05_21`

### §18.5 Top-3 counterpart anchors per cluster (Big-8 reference)

| Cluster | Top-3 counterparts |
|---|---|
| CRM | Salesforce / HubSpot / Microsoft Dynamics 365 |
| ITSM | ServiceNow / Jira Service Management / BMC Helix |
| Marketing automation | HubSpot / Marketo / Salesforce Marketing Cloud |
| Contract lifecycle mgmt | DocuSign CLM / Ironclad / Conga |
| Performance management | Workday / Lattice / SAP SuccessFactors |
| Cloud billing | Stripe / Recurly / Zuora / Chargebee |
| Data warehouse | Snowflake / Databricks / BigQuery / Redshift |
| Workflow engine | Temporal / n8n / Airflow / Cadence |
| Developer SDK gen | Stainless / Speakeasy / Fern |
| Messenger | Discord / Slack / Teams |
| Community | Reddit / Teamblind / Handshake |
| Social | TikTok / Instagram / Snapchat |

---

## §19. Retired µservices (4)

| µservice | Retired by | Absorbed by |
|---|---|---|
| `foundry` | ADR-0335 (2026-05-21) | intelligence (§D-16 BC absorption: providers + guardrails + eval) |
| `network` | Wave 15K | community |
| `cell` | ADR-0333 | tenancy (assignment) + cloud-iac (provisioning + registry) + observability (health + blast-radius) + `oya-shuffle-sharding` crate (algorithm) + api-gateway (routing) + audit-chain (cell-scoped audit) |
| `shorts` | ADR-0334 (2026-05-21) | social |

Foundry retirement detail: 125 `oya-foundry-*` crates retained as transition debt per ADR-0335 D-43 (sequenced as separate cleanup wave; cargo workspace stays green). Hermes terminology dropped corpus-wide.

---

## §20. Wave 15 doctrine + landed ADRs (this session)

| ADR | Title | Status |
|---|---|---|
| ADR-0329 | Tier system retired (supersedes ADR-0316) | LANDED 2026-05-21 |
| ADR-0330 | Tenant class = {demo_trial, paid} + composable billing_components | LANDED 2026-05-21 |
| ADR-0331 | Cross-µservice tenant_class adoption template | LANDED 2026-05-21 |
| ADR-0332 | Healthcare domain decomposition (7 µservices) | LANDED 2026-05-21 |
| ADR-0333 | Cell µservice RETIRED (pattern, not service) | LANDED 2026-05-21 |
| ADR-0334 | Shorts µservice RETIRED (merged into social) | LANDED 2026-05-21 |
| ADR-0335 | Foundry µservice RETIRED (absorbed by intelligence) | LANDED 2026-05-21 (774 lines) |
| ADR-0336 | Valkey not Redis substrate | LANDED 2026-05-21 (731 lines) |

---

## §21. Keystone ADR bundle (2026-05-20 foundational doctrine)

| ADR | Keystone # | Purpose |
|---|---|---|
| ADR-0242 | KS#1 | oyatie-is-a-tenant doctrine |
| ADR-0243 | KS#2 | Cedar universal gate |
| ADR-0244 | KS#3 | Tenant as universal scoping primitive |
| ADR-0245 | KS#4 | Substrate vs product layering |
| ADR-0246 | KS#5 | MLS RFC 9420 E2EE messenger |
| ADR-0247 | KS#6 | Self-modification doctrine |
| ADR-0248 | KS#7 | Amazon-shape cellular architecture |
| ADR-0249 | KS#11 | Multi-category marketplace |
| ADR-0250 | KS#9 | Build-ahead-of-certification |
| ADR-0251 | KS#8 | Compliance pack primitive |
| ADR-0252 | KS#12 | HLC default; TrueTime opt-in |
| ADR-0253 | KS#10 | HTTP/3 + QUIC default |
| ADR-0254 | KS#13 | K8s + Cloud Hypervisor + Kata |
| ADR-0255 | KS#14 | Intelligence two-layer substrate |

---

## §22. Authority chain (precedence; highest wins)

```
1. CURRENT-SESSION USER DIRECTIVES IN MEMORY
   (~/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_*.md)
        ↓
2. KEYSTONE ADRs (2026-05-20 + Wave 15 doctrine)
   ADR-0242..0255 + ADR-0329..0336
        ↓
3. OTHER ADRs (chronological; newer wins unless explicit supersession)
        ↓
4. CANONICAL SPECS
   specs/master-plan-sequencing.json
   specs/microservices/<ms>.json
   specs/root-hub-pointers.json
   specs/manifests-index.json
        ↓
5. PER-µSERVICE MANIFEST + PRD + ARCHITECTURE
        ↓
6. IMPLEMENTATION (src/ + crates/)
        ↓
7. HISTORICAL DOCS + RETIRED ARTIFACTS
   RETIRED.md markers, archive dirs, old ADRs with status: Retired/Substantially-Rewritten
```

Special precedence:
- **Cedar policy** wins over code claims for authorization decisions (per ADR-0243)
- **OpenSLO YAML** wins over markdown SLO prose (per ADR-0245)
- **manifest.json** wins over README markdown for `substrate_dependencies` + `supported_oses`

---

## §23. Key memory files (durable cross-session knowledge)

| Memory | Topic |
|---|---|
| `feedback_service_mesh_istio_ambient_envoy_cilium_2026_05_21` | Canonical mesh stack (§5) |
| `feedback_codex_dispatch_canonical_2026_05_21` | `-c model_reasoning_effort=xhigh` mandatory |
| `feedback_valkey_not_redis_2026_05_21` | Substrate doctrine §4 |
| `feedback_cell_standalone_network_merges_community_2026_05_21` | Community/social/mobile-bundle (§11/§18.3/§18.4) |
| `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20` | Tenant model (§8) |
| `feedback_no_capability_tiers_2026_05_20` | Tier retirement (§2) |
| `feedback_developer_sdk_stainless_generator_2026_05_20` | SDK generator (§3) |
| `feedback_rust_strict_only_no_python_2026_05_20` | Language policy (§3) |
| `feedback_os_support_matrix_2026_05_20` | OS matrix (§13) |
| `feedback_zero_handroll_opentofu_only_2026_05_20` | IaC (§3) |
| `feedback_oci_always_free_maximization_2026_05_20` | OCI Always Free (§12.1) |
| `feedback_multi_context_provider_agnostic_2026_05_20` | Deployment contexts (§12) |
| `feedback_microservice_ownership_coherence_2026_05_20` | Audit doctrine prerequisite |
| `feedback_verify_deliverables_not_just_line_count_2026_05_20` | Audit substance bar |
| `feedback_docs_substance_not_scaffold_2026_05_20` | Anti-stamping (§14) |
| `feedback_layer_enum_adr_0105_13_canonical` | 13-layer enum (§2) |
| `feedback_clean_architecture_requirements` | Clean arch invariants |
| `feedback_quality_performance_scalability_bar` | Hyperscaler bar |
| `feedback_workflow_studio_scope` | Workflow Studio = first hero product (§1) |
| `feedback_no_silent_regression` | Public contract protection (§14) |
| `feedback_oya_git_canonical_2026_05_18` | VCS canonical (§16) |
| `feedback_bominal_inheritance_precedence` | Inherit Bominal ADRs 1:1 unless overridden |
| `feedback_realignment_review_findings_2026_05_21` | Wave 15 progress summary |
| `project_branch_pipeline_implemented` | dev/staging/production pipeline |

---

## §24. State files (project-state, not personal memory)

| File | Content |
|---|---|
| `.omc/state/audit-doctrine-2026-05-21.md` | Canonical audit doctrine (every audit subagent reads once) |
| `.omc/state/wave-14-aggregation.md` | Canonical Wave 14 findings rollup (411 lines) |
| `.omc/state/wave-15-progress-2026-05-21.md` | Pre-compact Wave 15 snapshot |
| `.omc/state/wave-15-ca-verify-2026-05-21.md` | ADR-0105 13-layer audit (22 flat-layout µservices) |
| `.omc/state/wave-15-ca-verify-workspace-2026-05-21.md` | ADR-0105 audit for 19 workspace-crate-layout µservices |
| `.omc/state/realignment-review-2026-05-21.md` | Orchestrator mid-stream analysis |
| `.omc/state/wave-findings-aggregation-2026-05-21.md` | Per-µservice findings tally |
| `.omc/state/oyatie-architecture-2026-05-21.md` | **THIS FILE — single architecture source of truth** |

---

## §25. When this document drifts

- If a NEW user directive supersedes any section: update the relevant section IN PLACE + add a `**Changed 2026-MM-DD:**` annotation
- If a NEW ADR lands: append to §20 with a one-line summary + update affected sections
- If an ADR is RETIRED or substantially-rewritten: annotate the affected sections
- Keep MEMORY.md index lean (≤200 lines truncated; use this file as the source-of-truth instead)

---

**End of architecture canonical. Read this before any task that touches Oyatie structure.**
