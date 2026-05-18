---
microservice: compliance
doc: PRD
status: Drafting
authority_tier: 2
owner: axis-compliance
co_owners: [axis-security, council-architecture]
related_adrs: [ADR-0131, ADR-0145, ADR-0170, ADR-0181, ADR-0183, ADR-0209]
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
