---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-cell-substrate, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0117, ADR-0123, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/cell/threat-model.md
  - microservices/cell/dpia.md
  - microservices/cell/policy/cell-boundary.md
  - microservices/cell/policy/data-residency.md
  - microservices/cell/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (cell µservice)

## Purpose

Canonical control-to-framework mapping. Tells an external auditor (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR) exactly which control implementation satisfies which framework clause, with pointers to evidence.

## Enforced Frameworks

### SOC 2 Type 2 (2017 TSC + 2022 PoF)

| TSC | Control objective | Implementation | Evidence |
|---|---|---|---|
| CC1.1 | COSO Principle 1 | Code-of-conduct + signed-commit policy | `docs/standards/code-review.md` |
| CC1.5 | Accountability for performance | Per-µservice SLOs + on-call | `PRD.md` §Performance + `incident-response.md` |
| CC3.1 | Risk identification | Annual threat-model + DPIA | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to entity objectives | Multi-spectrum review per IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; 2-person rule on cell-decommission | `policy/cell-boundary.md` §"Audit Trail" |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain on every state transition | ADR-0028 |
| CC5.1 | Control activities | LEAN lanes (cell-boundary; cell-no-cross-cell-query; cell-rls-conformance; …) | `microservices/governance/` |
| CC5.2 | Technology controls | Cedar + Postgres RLS + SPIFFE + signed commits | `policy/*.cedar` |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar + JIT via OpenBao | `policy/tenant-scope.cedar`, `auditor-scope.cedar`, `ci-scope.cedar` |
| CC6.2 | Authentication + authorization | OIDC + per-cell API keys + SPIFFE | `policy/cell-boundary.md` §"Cell Identity Model" |
| CC6.3 | Adds/removes access | OpenBao lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Cell-namespace + Postgres RLS + reserved cells | `policy/cell-boundary.md` CB-01..CB-08 |
| CC6.7 | Information transmission + disposal | mTLS + KMS at rest + DSR cascade | `policy/data-residency.md` §"DSR Cascade" |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype + weekly CVE scan | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA Postgres + warm pool + autoscaling | `capacity-model.md` |
| CC7.2 | Monitoring system inputs | Self-observability metrics + OnCall | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + decommission-rate alerts | `incident-response.md` |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + branch protection | `branch-protection.yaml` |
| CC9.1 | Risk mitigation | Multi-region + DR pair + automated rollback | `multi-region.md` + ADR-0139 |
| CC9.2 | Vendor risk management | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` |

### ISO 27001:2022 (Annex A)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Annual threat-model + quarterly review | `threat-model.md` |
| A.5.10 | Acceptable use | Cedar + per-tenant scoping | `policy/*.cedar` |
| A.5.14 | Information transfer | mTLS + cross-pack-forbidden | `policy/data-residency.md` |
| A.5.15 | Access control | OIDC + Cedar + Postgres RLS + cell namespace | `policy/cell-boundary.md` |
| A.5.17 | Authentication information | OpenBao rotation 24h/30d | OpenBao audit |
| A.5.18 | Access rights | RBAC via OpenTofu; UI editing forbidden | `iac/terraform/cluster-api-rbac.tf` |
| A.5.23 | Cloud services security | OCI HIPAA-eligible regions for pack-us-healthcare | `policy/data-residency.md` |
| A.5.24 | Incident management planning | Incident-response playbook | `incident-response.md` |
| A.5.25 | Assessment of events | Severity classification | `incident-response.md` §"Severity" |
| A.5.26 | Response to incidents | Severity-driven runbook | `runbooks/` |
| A.5.27 | Learning from incidents | Postmortem template | `runbooks/postmortem-template.md` |
| A.5.28 | Evidence collection | Audit-chain Ed25519 emission | ADR-0028 |
| A.5.30 | ICT readiness for BC | Multi-region DR + RPO/RTO | `multi-region.md` |
| A.5.31 | Legal + statutory | This document + per-pack overlays | `compliance.md` |
| A.5.33 | Records protection | Audit-chain immutability + retention | `policy/data-residency.md` §"Retention" |
| A.5.34 | Privacy + PII protection | DPIA + DSR cascade + Cedar | `dpia.md` + `policy/*.cedar` |
| A.8.2 | Privileged access rights | JIT via OpenBao; 2-person rule | OpenBao audit |
| A.8.3 | Information access restriction | Cell namespace + Postgres RLS + Cedar | `policy/cell-boundary.md` |
| A.8.4 | Access to source code | CODEOWNERS + branch protection | `branch-protection.yaml` |
| A.8.5 | Secure authentication | SPIFFE SVID + per-cell credentials | `policy/cell-boundary.md` CB-02 |
| A.8.7 | Authentication management | OpenBao rotation | OpenBao audit |
| A.8.11 | Information masking | Hashed tenant-id + cell-id; no PII | `policy/cell-boundary.md` |
| A.8.12 | Data leakage prevention | DP-noise on aggregates; cell-adjacency never exposed | `policy/data-residency.md` |
| A.8.20 | Network controls | NetworkPolicy + mTLS + per-cell namespace | `policy/cell-boundary.md` CB-01 |
| A.8.21 | Network services security | mTLS via SPIFFE on every internal call | `policy/cell-boundary.md` |
| A.8.22 | Segregation of networks | Per-cell namespace + cross-namespace deny | `policy/cell-boundary.md` |
| A.8.23 | Web filtering | n/a (cell substrate is mesh-internal) | – |
| A.8.32 | Change management | PR + LEAN gates | `branch-protection.yaml` |
| A.8.34 | Audit logs protection | Audit-chain Ed25519 + Mimir retention | ADR-0028 |

### GDPR (key articles)

| Article | Requirement | Implementation | Evidence |
|---|---|---|---|
| Art. 5(1)(a) Lawfulness | Tenant DPA + onboarding consent | DPA template at `legal/dpa-template.md` (Slice D) | `dpia.md` §"Purposes" |
| Art. 5(1)(c) Data minimisation | cell binding processes minimal data (hashed tenant-id + metadata) | – | `dpia.md` §"Scope of Processing" |
| Art. 5(1)(f) Integrity + confidentiality | TI-01..TI-07 + cell-boundary | – | `policy/cell-boundary.md` |
| Art. 6 Lawfulness of processing | Per `dpia.md` Step 2.4 | – | `dpia.md` |
| Art. 17 Right to erasure | DSR cascade | – | `policy/data-residency.md` §"DSR Cascade" |
| Art. 25 Privacy by design | Cell-namespace + Cedar + pseudonymisation by design | – | `dpia.md` Step 6 |
| Art. 28 Processor obligations | Per-tenant DPA | DPA template | `dpia.md` |
| Art. 30 Records of processing | This file + audit-chain | – | `compliance.md` |
| Art. 32 Security of processing | All STRIDE mitigations | – | `threat-model.md` |
| Art. 33 Breach notification (DPA) | 72h notification | – | `incident-response.md` §"Regulatory Notifications" |
| Art. 34 Breach notification (data subjects) | Without undue delay | – | `incident-response.md` |
| Art. 35 DPIA | This document + DPIA | – | `dpia.md` |
| Arts. 44–50 Transfers | Cross-pack forbidden; SCC exception | – | `policy/data-residency.md` |

## Per-Pack Overlays

### pack-kr (KR PIPA + ISMS-P + 전자문서법)

| Framework clause | Implementation |
|---|---|
| KR PIPA Art. 3 (principles) | Cell substrate processes minimal data |
| KR PIPA Arts. 15, 17, 18 (collection + use + provision) | Per-tenant DPA discloses cell substrate |
| KR PIPA Art. 22-2 (consent for child data) | n/a (cell substrate processes tenant data, not end-user data) |
| KR PIPA Art. 23 (sensitive data) | `(tenant_id, cell_id)` treated as sensitive; salt rotation; cell-adjacency never exposed |
| KR PIPA Art. 28 (cross-border) | Forbidden; multi-region.md enforces |
| KR PIPA Art. 29 (technical safeguards) | All 12 prescribed safeguards mapped to CB-01..CB-08 |
| KR PIPA Art. 33 (DPIA) | `dpia.md` fulfils |
| KR PIPA Art. 34 (breach notification) | 72h initial + 30d full per `incident-response.md` |
| KR-ISMS-P §2.5 인적보안 | Onboarding + training |
| KR-ISMS-P §2.7 접근통제 | 2-person rule + JIT + SPIFFE |
| KR-ISMS-P §2.10 시스템보안 | NetworkPolicy + per-cell namespace |
| KR 전자문서법 Art. 5 (electronic document integrity) | Audit-chain Ed25519 |
| Audit log retention | ≥ 1y standard; 3y for `tenant_scope: production`; 5y for KR-FSS tenants |

### pack-us-healthcare (HIPAA)

| Framework clause | Implementation |
|---|---|
| §164.308(a)(1)(ii)(A) Risk analysis | `dpia.md` |
| §164.308(a)(4)(ii)(B) Access authorization | Cedar + Postgres RLS |
| §164.310 Physical safeguards | OCI HIPAA-eligible region (covered by OCI BAA + oyatie BAA) |
| §164.312(a)(1) Access control | Per-cell SVID + Cedar + Postgres RLS |
| §164.312(b) Audit controls | Audit-chain Ed25519; retention ≥ 6y per §164.316(b)(2) |
| §164.312(e)(1) Transmission security | mTLS + TLS 1.3 |
| §164.314 BAA | Per Covered Entity tenant; recorded |
| §164.502(b) Minimum necessary | Per-cell scope; workload µservices read only own cell |
| §164.514 De-identification | Hashed tenant-id; no PHI in cell substrate |
| §164.516(b)(2) Audit retention | ≥ 6 years |

### pack-eu (GDPR + EDPB + eIDAS + NIS2)

| Framework clause | Implementation |
|---|---|
| GDPR Arts. 5, 6, 25, 28, 30, 32, 33, 35, 44–50 | Per matrix above + `dpia.md` |
| EDPB Guidelines 4/2019 (Art. 25) | Privacy-by-design measures |
| EDPB Guidelines 9/2022 (breach notification) | `incident-response.md` |
| eIDAS 910/2014 | Ed25519 audit-chain seals satisfy AdES for transaction records |
| NIS2 2022/2555 (when applicable) | 24h initial + 72h detailed + 1mo final per `incident-response.md` |

### Other packs

Per-pack overlays at `regional-packs/<pack>/cell-compliance-overlay.md` for pack-jp (APPI), pack-sg (PDPA), pack-au (Privacy Act + APRA CPS 234), pack-in (DPDPA 2023), pack-br (LGPD), pack-ae (UAE PDPL), pack-ksa (PDPL + SAMA).

## Continuous-Compliance Evidence Emission

Per ADR-0123 (hyperscaler-maturity-claims-gate) + `oya-governance-compliance-evidence-recency` lane.

Cell substrate emits evidence per quarter:

| Evidence | Source | Cadence |
|---|---|---|
| Threat-model review | `threat-model.md` Sign-off block | Quarterly |
| DPIA refresh | `dpia.md` annual review | Annual |
| Cell-boundary lane state | LEAN-lane state in CI dashboard | Continuous |
| Postgres RLS conformance | `oya-cell-rls-conformance` lane | Continuous |
| Cedar fragment coverage | `oya-check-cedar-fragment-coverage` lane | Continuous |
| Pen-test report (annual) | external firm | Annual |
| Chaos drill report (quarterly) | ops-sre-reliability | Quarterly |
| DR-failover drill report | ops-sre-reliability | Quarterly |
| DSR cascade tabletop | council-privacy | Annual |
| Audit-chain seal integrity verification | audit-chain µservice | Continuous |

## Auditor Onboarding

Per `policy/auditor-scope.cedar`:
1. Audit firm signs engagement letter + DPA.
2. ops-security issues OpenBao JIT token: `engagement_id`, `scoped_tenants`, `valid_from/to`, `audit_framework`.
3. Auditor reads scoped tenants' cell evidence via Cedar-bounded `auditor-scope.cedar`.
4. Auditor reads policy artifacts (this document; threat-model; DPIA; etc.) via PolicyArtifact scope.
5. Every read is itself audit-emitted.

## References

- ADR-0028 (Bominal; audit-chain).
- ADR-0117 (residency).
- ADR-0123 (hyperscaler-maturity-claims-gate).
- ADR-0139 (SLO gate).
- ADR-0131 (per-µservice).
- ADR-0140 (Cedar).
- `microservices/cell/threat-model.md`; `dpia.md`; `policy/cell-boundary.md`; `policy/data-residency.md`; `incident-response.md`.
- SOC 2 Trust Services Criteria (AICPA 2017 + 2022 PoF).
- ISO/IEC 27001:2022 + Annex A controls.
- GDPR Regulation (EU) 2016/679.
- KR PIPA + PIPC Enforcement Decree.
- HIPAA 45 CFR §164.
- EDPB Guidelines.
