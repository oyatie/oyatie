# ITSM µservice (IT Service Management)

Service: `itsm`
Business capability: service-management
Date: 2026-05-21
Doc class: README
Top-3 counterparts: ServiceNow ITSM (Washington DC + Vancouver; ITSM Pro + CMDB + Now Assist) / Atlassian Jira Service Management (Cloud Enterprise; Assets formerly Insight; Atlassian Intelligence) / Freshservice (Enterprise tier; Freddy AI; Orchestration Center)
Second-tier counterparts: BMC Helix ITSM, Ivanti Neurons for ITSM, SolarWinds Service Desk, Zendesk Support, PagerDuty, Opsgenie, FireHydrant
Binding authorities: docs/standards/documentation-rigor.md §1.1, §1.2, §2, §3.2.1, §3.2.3, §3.2.5; ADR-0064; ADR-0105; ADR-0131; ADR-0132; ADR-0145; ADR-0242; ADR-0243; ADR-0244; ADR-0245; ADR-0246; ADR-0247; ADR-0251; ADR-0252; ADR-0253; ADR-0254; ADR-0255; ADR-0263; ADR-0314; ADR-0321; ADR-0328; ADR-0329; ADR-0330; ADR-0331.
Backend code in Rust per `specs/master-plan-sequencing.json#language_policy` and ADR-0328 D-18; frontend native bundles per platform allowlist only (Swift on iOS/macOS, Kotlin on Android, WinUI 3 C#/.NET on Windows).

## Scope

The ITSM µservice owns the operational concern of running an IT service desk: incident management, problem management, change enablement, service request fulfillment, service catalog, CMDB projection (the cmdb µservice owns the data; ITSM owns the operational integration), knowledge base, AI-powered deflection, on-call routing, escalation, status updates, and postmortems.

Per ADR-0131 (flat layout) and ADR-0132 (no-grouping), the µservice does NOT compose a multi-service "suite". It instead composes five separately-buildable bounded-context crates under `crates/`:

| Bounded context | Crate | Counterparts |
|---|---|---|
| `on-call-schedule` | `crates/on-call-schedule` | PagerDuty Schedules, Opsgenie On-Call, FireHydrant Schedules |
| `escalation-policy` | `crates/escalation-policy` | PagerDuty Escalation Policies, Opsgenie Escalations, FireHydrant Notification Policies |
| `incident-room` | `crates/incident-room` | PagerDuty Incident Workflows, FireHydrant Runbooks, ServiceNow Major Incident Management |
| `status-update` | `crates/status-update` | Atlassian Statuspage, FireHydrant Statuspage, Opsgenie Status |
| `postmortem` | `crates/postmortem` | FireHydrant Retro, Jeli, PagerDuty Postmortem |

The legacy aggregate domain (incident-ticket / problem / change / service-request / configuration-item) continues to live in `src/domain/mod.rs` while migration into per-bounded-context crates proceeds.

## What this µservice does NOT own

- The CMDB data store itself — that's the `cmdb` µservice. ITSM integrates via the `cmdb-sync` capability and the IP-027 reconciliation graph.
- The change-management approval state machine across non-ITSM contexts — that's the `change-management` µservice. ITSM consumes its decisions for CAB and emergency change events.
- The workflow runtime — that's the `workflow-engine` µservice. ITSM owns workflow template authoring (workflow-designer capability + IP-044); the engine owns execution.
- The intelligence stack — that's the `intelligence` µservice. ITSM uses it via knowledge-base RAG (IP-034), AI virtual agent (IP-035), and predictive intelligence (IP-038).
- The marketplace settlement rails — that's the `marketplace` µservice. ITSM publishes service-catalog items + workflow templates as listings under ADR-0314 DealSet settlement.
- The identity / tenant graph — that's the `identity` + `tenancy` µservices.

## Per-capability surface (28 ServiceNow ITSM family surfaces)

| Surface | Capability YAML | IP stub | Status |
|---|---|---|---|
| Incident Management | `capabilities/incident-open.yaml` | IP-026 | LANDED |
| Problem Management | `capabilities/problem-link.yaml` | IP-026 | LANDED |
| Change Enablement | `capabilities/change-approve.yaml` | IP-029 | LANDED |
| Service Catalog | `capabilities/service-catalog-publish.yaml` | IP-028 | LANDED |
| CMDB Sync | `capabilities/cmdb-sync.yaml` | IP-027 | LANDED |
| Major Incident Bridge | `capabilities/major-incident-bridge.yaml` | IP-030 | LANDED |
| Self-Service Portal | `capabilities/self-service-portal.yaml` | IP-031 | LANDED (Wave 15A) |
| Mobile ITSM | `capabilities/mobile-itsm.yaml` | IP-032 | LANDED (Wave 15A) |
| Agent Workspace | `capabilities/agent-workspace.yaml` | IP-033 | LANDED (Wave 15A) |
| Knowledge Base | `capabilities/knowledge-base.yaml` | IP-034 | LANDED (Wave 15A) |
| AI Virtual Agent | `capabilities/ai-virtual-agent.yaml` | IP-035 | LANDED (Wave 15A) |
| Discovery | `capabilities/discovery.yaml` | IP-036 | LANDED (Wave 15A) |
| Service Mapping | `capabilities/service-mapping.yaml` | IP-037 | LANDED (Wave 15A) |
| Predictive Intelligence | `capabilities/predictive-intelligence.yaml` | IP-038 | LANDED (Wave 15A) |
| CSAT Survey | `capabilities/csat-survey.yaml` | IP-039 | LANDED (Wave 15A) |
| Walk-Up Experience | `capabilities/walk-up-experience.yaml` | IP-040 | LANDED (Wave 15A) |
| SLA Engine | `capabilities/sla-engine.yaml` | IP-041 | LANDED (Wave 15A) |
| Visual Task Boards | `capabilities/visual-task-boards.yaml` | IP-042 | LANDED (Wave 15A) |
| Performance Analytics | `capabilities/performance-analytics.yaml` | IP-043 | LANDED (Wave 15A) |
| Workflow Designer | `capabilities/workflow-designer.yaml` | IP-044 | LANDED (Wave 15A) |
| On-Call Schedule (PagerDuty-class) | (5 crate bounded context) | (substrate per crate) | LANDED (Wave 15A) |
| Escalation Policy (PagerDuty-class) | (5 crate bounded context) | (substrate per crate) | LANDED (Wave 15A) |
| Incident Room (war-room) | (5 crate bounded context) | (substrate per crate) | LANDED (Wave 15A) |
| Status Update (statuspage) | (5 crate bounded context) | (substrate per crate) | LANDED (Wave 15A) |
| Postmortem | (5 crate bounded context) | (substrate per crate) | LANDED (Wave 15A) |
| ITIL Process Normalizer | (substrate) | IP-026 | LANDED |
| ITSM-to-DevOps bridge | (handed off to `oya git` µservice) | (downstream) | OUT-OF-SCOPE; routed |
| Multi-Instance per-tenant DB | (handed off to `tenancy` + `cloud-cell`) | (downstream) | OUT-OF-SCOPE; routed |

## Performance leadership (preserved per audit §13.2)

| Metric | Oyatie ITSM | ServiceNow ITSM | Advantage |
|---|---|---|---|
| SLA breach detection p99 | 15 s | 120 s | 8× |
| Sustained workflow throughput | 800 workflows/sec | 120 workflows/sec | 7× |
| CMDB 3-hop traversal p99 | 380 ms | 1,400 ms | 3.7× |

Source: `app/itsm/performance-benchmark-numbers-2026-05-20.md` §3 + §8.

## First 30 minutes — running ITSM locally

Pre-reqs: Rust 1.97.1+ (stable), OpenTofu, Docker, Kustomize.

1. Clone repo.
2. From the repo root: `cd microservices/itsm && cargo build`.
3. Verify the scaffold: `cargo test`. Tests in `tests/integration.rs` plus 5 bounded-context crate unit tests should pass.
4. Apply local IaC: `cd iac/on-prem && tofu init && tofu apply` (uses MinIO + Postgres + Valkey + Cedar evaluator in compose).
5. Open `http://localhost:8080/api/v1/itsm/openapi-v1.yaml` to see the surface.
6. POST a sample incident: `curl -X POST http://localhost:8080/api/v1/itsm/tickets -H 'content-type: application/json' -d @samples/incident-create.json`.

## Tenant-class behavior (per ADR-0331)

- `demo_trial`: caps on tickets (500/mo), CIs (200), workflows (1000/mo), agent seats (3), KB articles (50), AI deflection (200/mo), attachments (5 GB), mobile API (5000/mo). Time-gate 60 days. Compliance packs unavailable. SLO best-effort.
- `paid`: no caps. Per-seat + per-usage + revenue-share billing components per ADR-0331. All compliance packs activatable. Contractual SLOs apply.

## Compliance packs

SOC-2, ISO-27001, ITIL, GDPR, KR-PIPA, FedRAMP-High, HIPAA. Each pack is a tenant-activated overlay per ADR-0251. Pack activation modifies permit grants, data-class retention, residency rules, export evidence, and audit retention windows.

## Deployment contexts (per ADR-0328 §D-15)

- `oyatie-public-cloud` (Oyatie SaaS).
- `guest-on-aws` (customer's AWS account; OpenTofu modules under `iac/guest-on-aws/`).
- `guest-on-oci` (customer's OCI account; OpenTofu modules under `iac/guest-on-oci/`; OCI Always Free profile for demo_trial under `iac/guest-on-oci/always-free/`).
- `on-prem` (customer datacenter; uses MinIO + on-prem K8s).
- `colo` (colocation; bare-metal Kata pods).
- `oyatie-as-cloud-provider` (Oyatie's own IaaS substrate; cloud-* µservices).

## Directory index

- `PRD.md` — Product requirements.
- `ARCHITECTURE.md` — Architecture (902 lines).
- `manifest.json` — Machine-readable spec.
- `compliance.md`, `threat-model.md`, `dpia.md`, `capacity-model.md`, `cost-budget.md`, `failure-modes.md`, `incident-response.md`, `multi-region.md`, `sdk-plan.md`, `backfill-replay.md` — Operational evidence.
- `capabilities/` — One YAML per capability primitive (20 entries).
- `IP-001 .. IP-044.md` — Implementation plans (44 plans).
- `contracts/` — OpenAPI + AsyncAPI + proto3.
- `policies/` + `policy/` — Cedar policies (canonical + local-pack overlays).
- `iac/` — OpenTofu modules per deployment context.
- `runbooks/` — 22 runbooks.
- `slos/` — OpenSLO definitions.
- `dashboards/` — Grafana JSON.
- `catalog/` — Layer-by-layer catalog records.
- `crates/` — Five bounded-context crates (per ADR-0131 + Wave 15A remediation).
- `migration-playbooks/` — From ServiceNow, JSM, Freshservice.
- `tutorials/`, `faqs/`, `onboarding/`, `scorecards/`, `reference-implementations/` — Auxiliary surfaces.
- `coherence-audit-2026-05-20.md`, `feature-parity-matrix-2026-05-20.md`, `performance-benchmark-numbers-2026-05-20.md` — Wave 4-rolling audit deliverables.
- `REMEDIATION-NOTES-2026-05-21.md` — Wave 15A remediation log.

## Foundry absorption posture (ADR-0247 + ADR-0255-amendment)

Workflow templates that previously lived in a separate "foundry" runtime are now part of the workflow-engine µservice. ITSM's `workflow-designer` capability authors templates and the workflow-engine executes them. There is no separate foundry path. Agent principals interact with ITSM under the `oyatie.foundry.itsm.*` Cedar role namespace.

## Oyatie-is-a-tenant doctrine (ADR-0242)

Oyatie itself runs ITSM as a reserved-namespace tenant (`oyatie.it-ops.*`) for its own incident response, change approval, CMDB inventory, and postmortems. No carve-outs; the same Cedar gates apply.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
