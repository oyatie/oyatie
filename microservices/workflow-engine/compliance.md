---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-workflow, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0035, ADR-0117, ADR-0123, ADR-0139, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/workflow-engine/threat-model.md
  - microservices/workflow-engine/dpia.md
  - microservices/workflow-engine/policy/data-residency.md
  - microservices/workflow-engine/policy/spec-integrity.md
  - microservices/workflow-engine/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (workflow-engine µservice)

## Purpose

Canonical control-to-framework mapping for workflow-engine. Tells external auditors (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / etc.) which control implementation satisfies which framework clause, with pointers to evidence. Continuous-compliance-evidence emission keeps this matrix machine-verifiable.

## Enforced Frameworks

### SOC 2 Type 2 (2017 Trust Services Criteria + 2022 Points of Focus)

| TSC | Control objective | Implementation | Evidence artifact |
|---|---|---|---|
| CC1.1 | COSO Principle 1: Integrity and ethical values | Code-of-conduct + signed-commit policy; CODEOWNERS reviewed quarterly | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.2 | COSO Principle 2: Board oversight | Council-architecture quarterly review | `docs/teams/council-architecture.md` |
| CC1.3 | Organizational structure | RACI matrix per µservice | `microservices/workflow-engine/CODEOWNERS` |
| CC1.4 | Commitment to competence | Onboarding + training | `docs/standards/onboarding.md` |
| CC1.5 | Accountability | Per-µservice SLO + on-call rotation | `PRD.md` §Performance + `incident-response.md` |
| CC2.1 | Communication of information | Status page + tenant comms | `runbooks/oncall-rotation.md` |
| CC2.2 | Internal communication | Slack + incident channels | `incident-response.md` §"Escalation" |
| CC2.3 | Communication with external parties | DPA + BAA + tenant onboarding | `legal/dpa-template.md` |
| CC3.1 | Risk identification + assessment | Annual threat-model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to entity objectives | Multi-spectrum review per ADR + per IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; 2-person rule for sensitive ops | `policy/spec-integrity.md` |
| CC3.4 | Significant change risk | Change-management via PR review + LEAN lanes | `branch-protection.yaml` |
| CC4.1 | Internal monitoring | LEAN CI lanes + per-µservice SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain emission on every state transition | ADR-0028 + audit-chain µservice |
| CC5.1 | Control activities | LEAN lanes | `microservices/governance/` |
| CC5.2 | Technology controls | Cedar policy + Citus + RLS + signed specs | `policy/*.cedar` + `policy/spec-integrity.md` |
| CC5.3 | Policy and procedure deployment | Per-µservice runbooks + standards | `docs/standards/*.md` + `runbooks/` |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar + JIT via OpenBao | `policy/tenant-scope.cedar`, `policy/auditor-scope.cedar`, `policy/ci-scope.cedar` |
| CC6.2 | Authentication + authorization | OIDC + per-tenant SDK API keys + SPIFFE | `threat-model.md` §"Trust boundaries" |
| CC6.3 | Adds / removes access | OpenBao lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Citus + RLS + reserved CI tenants | `threat-model.md` T-I-01 |
| CC6.7 | Information transmission + disposal | mTLS in transit + KMS at rest + DSR cascade | `policy/data-residency.md` §"DSR Cascade" |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype; weekly CVE scan | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA workers + per-tenant rate limits + autoscaling | `capacity-model.md` |
| CC7.2 | Monitoring system inputs | Self-observability metrics + OnCall alerts | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + run-stuck SLI | `dashboards/*.json` |
| CC7.4 | Incident response | Severity-classified response + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + branch protection | this changeset |
| CC9.1 | Risk mitigation | Multi-region + DR pair + auto-rollback | `multi-region.md` + ADR-0139 |
| CC9.2 | Vendor risk management | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` |

**Privacy Criteria (P1-P8, 2017 TSC, optional):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice + privacy practices | DPA template + tenant onboarding notice |
| P2 | Choice + consent | Tenant onboarding consent (OpenBao tenant-resolver) |
| P3 | Collection | SDK PII redactor + `data_class` annotation |
| P4 | Use, retention, disposal | Retention matrix in `data-residency.md`; DSR cascade |
| P5 | Access | Tenant operators read own; DSR access cascade |
| P6 | Disclosure to third parties | Sub-processor list + transfer register |
| P7 | Quality | Spec signature verification + audit-chain integrity |
| P8 | Monitoring + enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A control families)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Annual threat-model + quarterly review | `threat-model.md` |
| A.5.10 | Acceptable use | Cedar policy + per-tenant scoping | `policy/*.cedar` |
| A.5.14 | Information transfer | mTLS + cross-pack-replication-forbidden | `policy/data-residency.md` §"Cross-Pack Replication" |
| A.5.15 | Access control | OIDC + Cedar + Citus + RLS | `threat-model.md` |
| A.5.17 | Authentication information | OpenBao rotation (30d / 90d) | OpenBao audit log |
| A.5.18 | Access rights | RBAC managed via Terraform | `iac/terraform/engine-rbac.tf` |
| A.5.23 | Cloud services | OCI HIPAA-eligible for pack-us-healthcare | `policy/data-residency.md` |
| A.5.24 | Incident management planning | Incident-response playbook | `incident-response.md` |
| A.5.25 | Assessment of security events | Severity classification | `incident-response.md` §"Severity Definitions" |
| A.5.26 | Response to incidents | Severity-driven runbook | `incident-response.md` + `runbooks/*` |
| A.5.27 | Learning from incidents | Post-incident review template | `runbooks/postmortem-template.md` |
| A.5.28 | Collection of evidence | Audit-chain Ed25519 emission | ADR-0028 |
| A.5.30 | ICT readiness for BC | Multi-region DR + RPO/RTO | `multi-region.md` |
| A.5.31 | Legal, statutory, regulatory | This document + per-pack overlays | `compliance.md` (this file) |
| A.5.32 | Intellectual property rights | License-policy CI lane | `oya-check-license-policy` |
| A.5.33 | Protection of records | Audit-chain immutability + retention | `policy/data-residency.md` |
| A.5.34 | Privacy and protection of PII | DPIA + DSR cascade + Cedar | `dpia.md` + `policy/*.cedar` |
| A.8.2 | Privileged access rights | JIT via OpenBao; 2-person rule | OpenBao audit |
| A.8.3 | Information access restriction | Citus + RLS + Cedar | `threat-model.md` |
| A.8.4 | Access to source code | CODEOWNERS + branch-protection | `branch-protection.yaml` |
| A.8.5 | Secure authentication | mTLS + OIDC + signed SDK API keys | `threat-model.md` §"Trust boundaries" |
| A.8.7 | Protection against malware | Wasmtime sandbox for step bodies | `threat-model.md` T-E-01 |
| A.8.11 | Data masking | SDK redactor for PII in step payloads | `threat-model.md` T-I-02 |
| A.8.12 | Data leakage prevention | OTel redactor + log sampling | `threat-model.md` T-I-02 |
| A.8.15 | Logging | Audit-chain emission | ADR-0028 |
| A.8.16 | Monitoring activities | Engine self-SLI + Grafana | `dashboards/*.json` |
| A.8.20 | Network security | Network policies; engine → Postgres / Redis / ClickHouse only | Kubernetes NetworkPolicy review |
| A.8.21 | Network services security | mTLS internal; TLS public | ingress configuration |
| A.8.23 | Web filtering | n/a (engine is API-only) | — |
| A.8.25 | Secure development life cycle | LEAN gates + multispectrum review | per-IP evidence |
| A.8.26 | Application security requirements | Step-body determinism contract; sandbox; signature verification | `policy/spec-integrity.md` |
| A.8.27 | Secure system architecture | Hexagonal layering per ADR-0103 | `PRD.md` §"Bounded Contexts" |
| A.8.28 | Secure coding | `cargo clippy -- -D warnings`; sanitizers in CI | per-IP acceptance lane |

### GDPR (relevant articles)

| Article | Requirement | Implementation | Evidence |
|---|---|---|---|
| Art. 5 | Principles of processing | Lawful + transparent + minimised + purpose-limited | `dpia.md` §2.4 |
| Art. 6 | Lawful basis | Per purpose in DPIA §2.4 | `dpia.md` |
| Art. 9 | Special categories | pack-us-healthcare BAA; pack-kr explicit consent | `legal/baa-template.md`, `legal/dpa-template.md` |
| Art. 13 + 14 | Information to data subject | Tenant DPA upstream-disclosure clause | `legal/dpa-template.md` |
| Art. 17 | Right to erasure | DSR cascade | `policy/data-residency.md` §"DSR Cascade" |
| Art. 22 | Automated decision-making | Engine is operational, not solely-automated-with-legal-effect | `dpia.md` R-04 |
| Art. 25 | Privacy by design | This µservice's design satisfies; EDPB Guidelines 4/2019 alignment | `dpia.md` §4 |
| Art. 28 | Processor agreement | DPA template | `legal/dpa-template.md` |
| Art. 30 | Records of processing | ROPA | `legal/ropa.md` |
| Art. 32 | Security of processing | Every threat mitigation contributes | `threat-model.md` |
| Art. 33 | Breach notification (DPA) | 72h notification chain | `incident-response.md` §"Regulatory Notifications" |
| Art. 35 | DPIA | This document + `dpia.md` | `dpia.md` |
| Arts. 44-50 | Transfers | SCC-only; default pack-pinning | `policy/data-residency.md` |

## Per-Pack Overlay Sections

### pack-kr (KR-ISMS-P + KR PIPA + KR 전자문서법)

KR PIPA Art. 29 (technical safeguards) — cross-mapped to engine mitigations:

| PIPA safeguard | Engine mitigation |
|---|---|
| Access control | OIDC + Cedar + Citus + RLS |
| Encryption (transit) | mTLS internal; TLS public ingress |
| Encryption (at rest) | KMS-SSE for Postgres + Redis AOF + ClickHouse + object-storage |
| Integrity verification | Spec signature + audit-chain Merkle |
| Audit log retention ≥ 1y | 3y default for KR-FSS sector |
| IDS / IPS | Network policies + WAF |
| Vulnerability management | `cargo deny` + Trivy + Grype |
| Mobile / remote access | mTLS + OIDC + MFA |
| User account management | OpenBao lifecycle + JIT elevation |
| Logging | OTel + audit-chain |
| Patch management | Helm + ArgoCD declarative |
| Incident response | Severity-classified runbooks |

KR 전자문서법 (Electronic Document Act):
- Art. 5 (integrity preservation): Ed25519 audit-chain seal satisfies.
- Art. 6 (long-term preservation): Audit-chain immutability + retention windows satisfy.
- Art. 7 (admissibility): Audit-chain Merkle proof is admissible evidence.

### pack-us-healthcare (HIPAA)

| HIPAA section | Requirement | Engine implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) | Risk analysis | This document + `dpia.md` + `threat-model.md` |
| §164.308(a)(3)(ii)(C) | Termination procedure | OpenBao lifecycle revokes access on offboarding |
| §164.308(a)(4)(ii)(B) | Access authorization | Cedar policy + 2-person rule |
| §164.310(a)(1) | Facility access controls | Inherited from cloud-k8s + OCI HIPAA-eligible |
| §164.310(c) | Workstation security | Inherited from corporate IT |
| §164.312(a)(1) | Access control (unique user ID) | OIDC + per-user identity |
| §164.312(a)(2)(i) | Emergency access | JIT elevation procedure |
| §164.312(a)(2)(ii) | Automatic logoff | Session timeout |
| §164.312(b) | Audit controls | Audit-chain emission |
| §164.312(c)(1) | Integrity | Audit-chain Merkle + spec signature |
| §164.312(d) | Person or entity authentication | OIDC + MFA + SPIFFE |
| §164.312(e)(1) | Transmission security | mTLS + TLS |
| §164.316(b)(1) | Documentation | This document + sibling artifacts |
| §164.316(b)(2) | Documentation retention (6y) | Repo retention + audit-chain |

Business Associate Agreement at `microservices/workflow-engine/legal/baa-template.md`.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS + DORA)

- GDPR Art. 32 cross-mapped (above).
- EDPB Guidelines 4/2019 (Art. 25): every mitigation in `threat-model.md` maps to a TOM.
- NIS2 (2022/2555): when oyatie crosses Annex I/II thresholds (likely on platform-wide tenant count), the 24h + 72h + 1mo reporting timelines apply; `incident-response.md` reflects.
- eIDAS 910/2014 Art. 26 (AdES): Ed25519 audit-chain seals satisfy AdES; QualifiedES requires HSM-backed signing (available where OCI offers KMS-HSM).
- DORA (Digital Operational Resilience Act 2022/2554): for EU financial-services tenants, the BCDR posture in `multi-region.md` aligns with DORA Article 11 (ICT continuity); annual DORA testing at `multi-region.md` §"BCDR Exercise Cadence".

### pack-jp (APPI)

| APPI Article | Requirement | Engine implementation |
|---|---|---|
| Art. 17 | Purpose of use declaration | DPA + tenant onboarding |
| Art. 18 | Consent for sensitive data | Tenant DPA |
| Art. 20 | Security control measures | Threat-model mitigations |
| Art. 21 | Cross-border transfer | pack-jp residency |
| Art. 23 | Joint use disclosure | Tenant DPA upstream clause |
| Art. 24 | Provision to third party | Sub-processor list |
| Art. 26-2 | Breach notification | `incident-response.md` |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/workflow-engine-overlay.md` with full citations.

## Continuous-Compliance Evidence

The `oya-governance-compliance-evidence-recency` lane validates that each control's evidence artifact:
- Exists at its declared path.
- Was last modified within the freshness window (annual default; quarterly for sensitive controls).
- Has a valid Ed25519 signature where required.

Annual external audit reads this file + the evidence artifacts cited above.

## Re-review Triggers

- Annually (Q2 each year).
- On any pack activation.
- On any enforced-framework version update.
- On any sub-processor change.
- Post-incident (Sev-1 or Sev-2).
- On any change to the engine's processing scope (new data class, new BC, new actor type).

## References

- ADR-0028 (Bominal): Audit chain.
- ADR-0035 (Bominal): Workflow engine.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0123: Hyperscaler maturity claim gate.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- `microservices/workflow-engine/threat-model.md`.
- `microservices/workflow-engine/dpia.md`.
- `microservices/workflow-engine/policy/data-residency.md`.
- `microservices/workflow-engine/policy/spec-integrity.md`.
- `microservices/workflow-engine/incident-response.md`.
- SOC 2 Type 2 (2017 TSC + 2022 PoF).
- ISO 27001:2022 Annex A.
- GDPR EUR-Lex 2016/679.
- KR PIPA + 전자문서법 + KR-ISMS-P.
- HIPAA 45 CFR Parts 160 + 164.
- APPI 2003 (改正 2022).
- LGPD 2018.
- DPDPA 2023.
- DORA 2022/2554.
- NIS2 2022/2555.
- eIDAS 910/2014.
