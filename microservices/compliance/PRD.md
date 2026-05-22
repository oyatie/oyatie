---
microservice: compliance
doc: PRD
status: Drafting
authority_tier: 2
owner: axis-compliance
co_owners: [axis-security, council-architecture]
related_adrs: [ADR-0131, ADR-0145, ADR-0170, ADR-0181, ADR-0183, ADR-0209, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
date: 2026-05-18
---

# Compliance — Product Requirements Document

## Problem statement

Selling to enterprise + regulated industries requires continuous compliance evidence: SOC 2 Type II, GDPR (including Art. 12 DSAR), HIPAA, and (when payments process) PCI-DSS 4.0. Commercial vendors (Drata, Vanta, Tugboat Logic, AuditBoard, ServiceNow GRC) charge $50k-$500k/year to wire continuous evidence collection + auditor portal access — and they sit between us and the auditor, with opaque tamper-evidence chains.

oyatie has the underlying primitives — audit-chain seal (ADR-0145), Cedar policy snapshots (ADR-0183), deploy receipts (ADR-0181), Backstage developer portal (ADR-0170), SeaweedFS storage (ADR-0145). The compliance µservice stitches these primitives into a unified evidence pipeline + auditor-facing read surface, owned end-to-end.

**Differentiation:** in-house build = direct auditor relationship + tamper-evidence verifiable by anyone holding the audit chain + sovereignty preserved (evidence never leaves operator-owned cluster).

## Goals

1. **SOC 2 Type II readiness** — continuous evidence pipeline covering AICPA Trust Services Criteria (security, availability, processing integrity, confidentiality, privacy).
2. **GDPR DSAR automation** — 5-day target SLA against the 30-day statutory limit. Per-subject export / deletion / rectification.
3. **HIPAA continuous compliance** — minimum-necessary access logs + BAA inventory.
4. **PCI-DSS readiness** — out-of-scope unless payments process; substrate ready when `microservices/payments/` lands.
5. **Auditor self-service portal** — read-only Backstage view; per-engagement auditor identity; access expires on engagement close.
6. **Cross-tenant isolation invariant** — DSAR responses never leak cross-tenant data; tamper-evidence per-tenant.

## Non-goals

- Not a vendor wrapper. The µservice does NOT proxy to Drata / Vanta APIs.
- Not a GRC platform. Risk register + control management remain in axis-compliance team workflows (out of scope here).
- Not a privacy-policy-text generator. Legal text authoring is human (lawyer) work.
- Not the audit-chain seal source. The seal is emitted by `oya-shared-audit-chain-client-kernel` (per ADR-0145); the compliance µservice consumes seals.

## Users + primary jobs

| User | Job |
|---|---|
| Security / compliance lead | Run an audit; pull a quarter's worth of access reviews + deploy receipts + vuln scans; satisfy SOC 2 Type II Trust Service Criteria. |
| Privacy officer | Handle GDPR DSAR; export subject data within target SLA; produce statutory-compliance evidence. |
| HIPAA covered-entity privacy officer | Pull minimum-necessary access logs for a subject + a window. |
| External auditor | Read-only access to per-framework artifact inventory; verify audit-chain seal for any artifact. |
| Tenant admin | Configure tenant's enabled compliance frameworks. |

## Success metrics

| Metric | Target |
|---|---|
| DSAR completion p50 | ≤ 5 days |
| DSAR completion p99 | ≤ 30 days (statutory) |
| Cross-tenant DSAR leak count | 0 (any incident is a Sev-1) |
| SOC 2 control coverage | 100% required artifacts emitted per quarter |
| Audit-chain seal verification rate (auditor portal) | 100% of viewed artifacts |
| Evidence storage durability | 99.999% (per SeaweedFS tier in ADR-0184) |
| Auditor portal availability | 99.95% during audit engagements |

## Functional surface

### REST API (per ADR-0182 north-south)

- `POST /api/v1/dsar/export` — accept subject identity + tenant; emit job; return `dsar_request_id`.
- `POST /api/v1/dsar/delete` — Ontology cascade per ADR-0145.
- `POST /api/v1/dsar/rectify` — field-level update.
- `GET  /api/v1/dsar/{request_id}` — request status + elapsed days + SLA.
- `GET  /api/v1/evidence/coverage?framework=...&tenant=...` — coverage report.
- `GET  /api/v1/evidence/artifact/{artifact_id}` — artifact metadata + seal hex.
- `POST /api/v1/evidence/manual-upload` — manual artifact upload (pen-test reports, BAA inventory).

### Backstage auditor portal plugin

- `/auditor/<framework>/` — per-framework artifact inventory.
- `/auditor/seal-verify/<artifact_id>` — verify audit-chain seal via Sigstore / Cosign chain.

### Event surface

- Inbound: deploy events (ADR-0181), Trivy scan events, CI build events, DSAR request events.
- Outbound: `EVT-COMPLIANCE-ARTIFACT-EMITTED`, `EVT-DSAR-REQUEST-OPENED`, `EVT-DSAR-REQUEST-CLOSED`, `EVT-AUDIT-SEAL-VERIFY-FAILED` (Sev-1).

## Architecture summary

The µservice is a thin layer over existing primitives:

- **Kernel:** `oya-shared-compliance-evidence-kernel` (closed framework + artifact-kind enums + coverage matrix + DSAR SLA tracking).
- **Domain:** `oya-compliance-domain` (DSAR aggregation, per-framework rollup, cross-tenant guard).
- **Use-case:** `oya-compliance-usecase` (collector orchestration, DSAR flow, audit-chain seal verify).
- **REST API:** `oya-compliance-api-rest` (per ADR-0182).
- **Auditor portal:** Backstage plugin at `clients/auditor-portal/`.

## SLOs (canonical OpenSLO)

- **DSAR export p99:** 5 days (target); 30 days (statutory cap).
- **Evidence emission lag p99:** 60 seconds (event-driven collectors); ≤ 15 minutes (cron collectors).
- **Auditor portal p99 latency:** 800 ms (per `observability.trace_sampling_recipe.p99_latency_threshold_ms`).
- **Cross-tenant isolation invariant:** 0 violations (any → Sev-1).

## Non-Functional Requirements

### DR posture (ADR-0343)

- Target: RTO ≤1800s and RPO ≤300s for evidence collection state, DSAR queues, auditor engagements, pack-overlay decisions, and audit-seal verification metadata. The current manifest has no D-2 `dr` block, so this is a PRD-level doctrine target pending manifest backfill.
- Compliance-pack floors considered: EU-AI-ACT-2024-HIGH-RISK (1800s/300s, multi-region), HIPAA-2024 (3600s/300s, multi-region), KR-CSAP-v3.1 (3600s/900s, multi-region), SOC2-T2 (14400s/900s), PCI-DSS-L1-v4 (86400s/3600s), ISO27001-2022/SOX-404 (14400s/3600s), and KR-PIPA-2023-amendment (14400s/900s). Effective target is RTO 1800s, RPO 300s, multi-region for regulated packs.
- Failover runbook: `microservices/compliance/runbooks/seaweedfs-evidence-bucket-loss.md`, matching manifest `dr.failover_runbook`; collector and auditor-facing continuity use `microservices/compliance/runbooks/certification-evidence-pipeline-stall.md` and `microservices/compliance/runbooks/regulator-evidence-export-failure.md`.
- Multi-region active-active: yes for collector scheduling, coverage ledgers, and auditor portal metadata; object evidence remains pack-resident and is restored through the evidence bucket runbook if a storage cell is lost.
- WHY: auditors and privacy officers can keep statutory evidence windows open during a region outage without risking cross-tenant DSAR leakage or unverifiable artifacts.

### Capacity model (ADR-0340)

- Per-tenant baseline: 0.16 vCPU, 256 MiB RAM, 12 GiB evidence/export storage allowance, 2 Valkey connections, 3 Postgres connections, and 8 outbound collector/API slots, matching manifest `capacity_model`.
- Scaling dimension: `per_workflow_run`, because manifest doctrine treats evidence exports and compliance collectors as workflow-execution shaped.
- Cell placement class: Tier-2 compliance workflow substrate, matching manifest `capacity_model.cell_placement_class`; runtime placement maps to pod runtime Tier 1 because manifest `pod_runtime_tier=1`.
- Autoscaling boundary: minimum 2 collectors, 1 DSAR worker, 1 coverage evaluator, and 1 auditor portal replica per regulated pack/cell; maximum 20 collectors and 10 DSAR/evidence workers per tenant during audit windows before workload isolation is required.
- WHY: the model supports quarterly SOC2/HIPAA/PCI audit surges and DSAR statutory deadlines without letting one tenant's engagement drain collector capacity for another tenant.

### Sustainability + cost attribution (ADR-0344)

- Every evidence artifact, DSAR state transition, pack subscription, manual upload, auditor access, seal verification, and breach-clock event emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, and `carbon_intensity_source` in its audit row.
- Provider-routing affected by carbon: no for breach notification clocks, DSAR statutory caps, evidence sealing, or pack gates; yes for batch evidence replay, coverage recomputation, and non-deadline report generation when pack policy permits.
- Per-tenant cost surface: the compliance portal shows pack-level evidence costs and links to the tenant FinOps dashboard for the tenant/product/capability/provider/cell/compliance_pack axes.
- WHY: CSRD, SB-253, and SEC climate-disclosure support must be traceable to compliance evidence itself, not reconstructed after the audit.

### API versioning posture (ADR-0342)

- Public API version model: DSAR, evidence coverage, artifact, manual upload, auditor portal, and regulator export APIs use the YYYY-MM-DD carrier triplet: `Oyatie-Version` header, `/v/<YYYY-MM-DD>/...` URL prefix, and `oyatie_version` proto3 field.
- SDK semver model: compliance client SDKs ship as major.minor.patch and map supported date versions explicitly.
- Support window: last 3 public API versions for at least 180 days.
- Per-tenant pinning: yes for auditor engagements, DSAR clients, and regulated-pack integrations.
- Internal-mesh exemption: yes; collector-to-substrate gRPC remains exempt under ADR-0145.

## Cost ceiling

- Steady-state: $1,500/month for a 32-µservice fleet at moderate scale.
- Major driver: SeaweedFS evidence storage (~ 5 TB / quarter).
- Compares vs Drata ~$25k/year baseline + per-employee fees.

## Risk register

1. **Cross-tenant DSAR leak** — Sev-1 incident; mitigated by kernel-level `tenant_id` invariant + integration tests.
2. **Audit-chain seal verification regression** — Sev-1; mitigated by cosign keyless OIDC chain test.
3. **Storage exhaustion** — degrades to read-only; auto-tier to cold storage per ADR-0184.
4. **DSAR backlog during high-traffic events** — auto-scale collector tier; circuit-break new DSAR intake at backlog > 100; manual review.

## Out-of-scope (Phase 1)

- PCI-DSS payments enablement (deferred until `microservices/payments/` lands).
- Multi-jurisdiction tax / VAT compliance (separate µservice).
- Drata / Vanta migration wizard (no in-bound vendor data assumed; greenfield).

## References

- ADR-0131 — per-microservice flat layout.
- ADR-0145 — audit-chain seal substrate.
- ADR-0170 — Backstage developer portal (auditor view).
- ADR-0181 — container image promotion (deploy receipts).
- ADR-0183 — Cedar policy engine.
- ADR-0209 — compliance evidence automation (this µservice's authority).
- `docs/standards/compliance-evidence-automation.md` — canonical standard.
- `oya-shared-compliance-evidence-kernel` — kernel implementation.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `compliance` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `compliance` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 4 context(s).
- Scaling input: `per_workflow_run` with cell placement `Tier-2` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
