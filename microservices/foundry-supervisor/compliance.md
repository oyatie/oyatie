---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping (foundry-supervisor)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-foundry-control-plane, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0117, ADR-0123, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/foundry-supervisor/threat-model.md
  - microservices/foundry-supervisor/dpia.md
  - microservices/foundry-supervisor/policy/supervisor-isolation.md
  - microservices/foundry-supervisor/policy/data-residency.md
  - microservices/foundry-supervisor/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (foundry-supervisor µservice)

## Purpose

Canonical control-to-framework mapping for the foundry-supervisor µservice. Tells an external auditor (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / EU AI Act notified body / etc.) exactly which control implementation satisfies which framework clause, with pointers to evidence. Continuous-compliance lane `oya-governance-compliance-evidence-recency` enforces freshness.

## Enforced Frameworks (every µservice; every pack)

### SOC 2 Type 2 (2017 TSC + 2022 Points of Focus)

| TSC | Control objective | Implementation | Evidence |
|---|---|---|---|
| CC1.1 | Integrity + ethical values | Code-of-conduct + signed commits | `docs/standards/code-review.md` |
| CC1.3 | Org structure | RACI matrix per µservice | `microservices/foundry-supervisor/CODEOWNERS` (Slice C) |
| CC1.5 | Accountability for performance | Per-µservice SLOs + on-call | `PRD.md` §Performance + `incident-response.md` |
| CC3.1 | Risk identification | Annual threat-model + DPIA + risk register | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to entity objectives | Multispectrum review per IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | Audit-chain Ed25519 seals; 2-person rule | `policy/supervisor-isolation.md` |
| CC3.4 | Significant change risk | PR review + LEAN lanes | branch-protection.yaml |
| CC4.1 | Internal monitoring | LEAN CI lanes + self-SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | Audit-chain on every state transition | ADR-0028 |
| CC5.1 | Control activities | LEAN lanes | `microservices/governance/` |
| CC5.2 | Technology controls | Cedar + Postgres RLS + signed commits | `policy/*.cedar` + `policy/supervisor-isolation.md` |
| CC6.1 | Logical + physical access | OIDC + MFA + Cedar + JIT via OpenBao | `policy/tenant-scope.cedar` etc. |
| CC6.2 | Authentication + authorization | OIDC + per-tenant SPIFFE | `policy/supervisor-isolation.md` §"Tenant Identity Model" |
| CC6.3 | Adds/removes access | OpenBao lifecycle | OpenBao audit log |
| CC6.6 | Logical access control | Postgres RLS + Redis ACL + Cedar | `policy/supervisor-isolation.md` TI-* |
| CC6.7 | Transmission + disposal | mTLS in transit + KMS at rest + DSR cascade | `policy/data-residency.md` §"DSR" |
| CC6.8 | Vulnerability management | `cargo deny` + Trivy + Grype | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA Postgres + Redis Cluster + autoscaling | `capacity-model.md` |
| CC7.2 | Monitoring inputs | Self-observability + OnCall | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Burn-rate alerts + autonomy-violation rate | `/specs/foundry-supervisor-control-plane.json` |
| CC7.4 | Incident response | Severity-classified + escalation | `incident-response.md` |
| CC8.1 | Change management | PR review + LEAN gates + branch protection | this changeset |
| CC9.1 | Risk mitigation | Multi-region + DR + automated rollback | `multi-region.md` + ADR-0130 |
| CC9.2 | Vendor risk | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` (Slice D) |

### ISO 27001:2022 (Annex A control families)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Annual threat-model + quarterly review | `threat-model.md` |
| A.5.10 | Acceptable use | Cedar policy + per-tenant scoping | `policy/*.cedar` |
| A.5.14 | Information transfer | mTLS + cross-pack-forbidden | `policy/data-residency.md` |
| A.5.15 | Access control | OIDC + Cedar + Postgres RLS | `policy/supervisor-isolation.md` |
| A.5.17 | Authentication info | OpenBao rotation (30d / 90d cadences) | OpenBao audit |
| A.5.18 | Access rights | RBAC via Terraform; UI forbidden | IaC |
| A.5.23 | Cloud services security | OCI HIPAA-eligible regions for pack-us-healthcare | `policy/data-residency.md` |
| A.5.24 | Incident management planning | Playbook | `incident-response.md` |
| A.5.25 | Assessment + decision | Severity classification | `incident-response.md` |
| A.5.26 | Response | Severity-driven runbook | `incident-response.md` + `runbooks/*` |
| A.5.27 | Learning from incidents | Postmortem template | `runbooks/postmortem-template.md` |
| A.5.28 | Evidence collection | Audit-chain Ed25519 | ADR-0028 |
| A.5.30 | ICT readiness for BC | Multi-region DR + RPO/RTO | `multi-region.md` |
| A.5.31 | Legal, statutory | This document + per-pack overlays | this file |
| A.5.33 | Records protection | Audit-chain immutability + retention | `policy/data-residency.md` |
| A.5.34 | Privacy + PII protection | DPIA + DSR + Cedar | `dpia.md` |
| A.8.2 | Privileged access | JIT via OpenBao; 2-person rule | OpenBao audit |
| A.8.3 | Info access restriction | Postgres RLS + Redis ACL + Cedar | `policy/supervisor-isolation.md` |
| A.8.4 | Source code access | CODEOWNERS + branch-protection | branch-protection.yaml |
| A.8.5 | Secure authentication | OIDC + MFA + SPIFFE | `policy/supervisor-isolation.md` |
| A.8.7 | Malware protection | Trivy + Grype + signed images (Cosign) | `.github/workflows/cosign.yml` |
| A.8.11 | Data masking | PII redactor at OTel SDK + tenant-id pseudonymisation | `policy/supervisor-isolation.md` |
| A.8.12 | Data leakage prevention | Cross-tenant query refusal + DP aggregation | `policy/supervisor-isolation.md` TI-* |
| A.8.14 | Redundancy | Postgres replica + Redis Cluster + Operator HA | `multi-region.md` |
| A.8.15 | Logging | Audit-chain + structured logs | ADR-0028 |
| A.8.16 | Monitoring activities | Self-observability + OnCall | `failure-modes.md` |
| A.8.20 | Network security | Network policies + Istio mTLS | `cloud-k8s` µservice |
| A.8.21 | Network services security | same | same |
| A.8.23 | Web filtering | WAF + OWASP CRS | `cloud-iac` µservice |
| A.8.24 | Cryptography | TLS 1.3 + Ed25519 + AES-256-GCM | ADR-0028 |
| A.8.25 | Secure development | LEAN lanes + PR review | `docs/standards/*` |
| A.8.26 | Application security requirements | OpenAPI + Cedar + LEAN | `contracts/openapi/*.yaml` |
| A.8.27 | Secure system architecture | Clean architecture (ADR-0056 + ADR-0105) | ADR-0056 + ADR-0105 |
| A.8.28 | Secure coding | Cedar fuzz + `cargo clippy` + `cargo deny` | LEAN lanes |
| A.8.32 | Change management | PR review + LEAN gates | branch-protection.yaml |
| A.8.34 | Protection during audit testing | Auditor JIT + scoped reads | `policy/auditor-scope.cedar` |

### GDPR (Arts. 5/6/9/13/14/17/22/25/28/30/32/33/35/44–50)

| Art. | Requirement | Implementation | Evidence |
|---|---|---|---|
| 5(1)(a-f) | Lawful + fair + transparent + minimisation + accuracy + storage + integrity-confidentiality | DPIA §2.4 + retention matrix + Cedar | DPIA + `policy/*` |
| 5(2) | Accountability | This document + DPIA + ROPA | `legal/ropa.md` (Slice D) |
| 6 | Lawful basis | Art. 6(1)(b)(c)(f) per purpose | DPIA §2.4 |
| 9 | Special category | Art. 9(2)(h) PHI; explicit consent KR sensitive | DPIA §4 |
| 13+14 | Information to data subjects | Tenant notice + joint-controllership cascade | DPA template |
| 17 | Right to erasure | DSR cascade per `policy/data-residency.md` | DSR runner |
| 22 | Automated decision-making | Operational decision carve-out (autonomy precondition is per-invocation safety, not solely-automated legal-effect-producing) | DPIA §6 R-05 |
| 25 | Privacy by design + default | Pseudonymisation + Cedar default-deny + Postgres RLS default | `policy/supervisor-isolation.md` + `policy/data-residency.md` |
| 28 | Processor terms | DPA template | `legal/dpa-template.md` (Slice D) |
| 30 | Records of processing | ROPA register | `legal/ropa.md` (Slice D) |
| 32 | Security of processing | Threat-model + supervisor-isolation + Cedar | `threat-model.md` |
| 33 | Breach notification (72h) | Incident response | `incident-response.md` |
| 35 | DPIA | This DPIA | `dpia.md` |
| 36 | Prior consultation | NOT triggered (residual ≤ Medium) | DPIA §7 |
| 44–46 | Cross-border transfers | SCC-only; Schrems-II supplementary | `legal/transfer-register.md` |

### EU AI Act (2024/1689) — high-risk system requirements

This µservice is a control plane for high-risk AI systems (when tenant capabilities fall in Annex III §1–8). The following articles are engaged:

| Art. | Requirement | Implementation |
|---|---|---|
| Art. 9 (risk management) | Continuous risk-management process | This `compliance.md` + `threat-model.md` + `dpia.md` + post-market monitoring |
| Art. 10 (data + data governance) | Training-data integrity (capability YAMLs are not training-data per se; but capability-definition integrity is critical) | Capability YAML LEAN schema + PR review + signed commits |
| Art. 12 (record-keeping) | Automatic logging of operations | Supervision events Ed25519-sealed; audit-chain Merkle; retention per pack |
| Art. 13 (transparency + information to deployers) | Deployers (tenants) receive operational info | Tenant-facing dashboards + capability registry + DPA cascade |
| Art. 14 (human oversight) | Effective human oversight measures | 2-person rule on fleet-wide kill-switch + tenant DPO can disengage own scope + readable audit-chain |
| Art. 15 (accuracy, robustness, cybersecurity) | Accuracy claims + robustness + cybersecurity | Threat-model mitigations + chaos drills + supply-chain (cargo deny / Trivy / Grype / Cosign) |
| Art. 17 (quality management system) | QMS per provider | LEAN lanes + ADR-0123 hyperscaler maturity claim gate + ADR-0133 industry-best-practice conformance |
| Art. 26 (deployer obligations) | Deployer (tenant) responsibilities | Tenant DPA enumerates: tenant must register capability in Annex III sub-domain at admit-time |
| Art. 27 (FRIA) | Fundamental Rights Impact Assessment | Per-tenant overlay required at first high-risk Annex III capability admit |
| Art. 60 (post-market monitoring) | Continuous monitoring + incident reporting | observability + supervisor self-SLOs + Sev-1 reporting chain via `incident-response.md` |
| Art. 73 (incident reporting) | Serious incident reporting to relevant authority | EU AI Office reporting timeline integrated in `incident-response.md` |

**EU AI Act notified-body engagement:** triggered at first high-risk Annex III tenant capability admit. Provider-side technical documentation per Art. 18 + Annex IV maintained at `microservices/foundry-supervisor/legal/ai-act-technical-documentation.md` (Slice D).

## Suggested Frameworks (per-pack activation)

### pack-kr (KR-ISMS-P + KR PIPA + 전자문서법)

| Section | Requirement | Implementation |
|---|---|---|
| KR-ISMS-P §2.1–2.12 | Policy / risk / HR / physical / access / crypto / system / ops / incident / PII / sub-processor / violation | Cross-mapped above |
| KR PIPA Art. 3, 15, 17, 18, 22-2, 23, 23-2, 24, 25, 28, 29, 29-2, 33, 33-2, 34 | Collection minimisation, consent, use limitation, sensitive data, cross-border, retention, technical safeguards, encryption, DPIA, DPO, breach notification | Cross-mapped in `threat-model.md` per-pack-kr overlay |
| KR 전자문서법 Art. 5, 6, 7 | Electronic document integrity, storage, verification | Ed25519 audit-chain |

### pack-us-healthcare (HIPAA)

| 45 CFR Part 164 | Requirement | Implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) | Risk analysis | `threat-model.md` + `dpia.md` |
| §164.308(a)(4)(ii)(B) | Access authorization | Cedar + Postgres RLS |
| §164.310(a) | Facility access | OCI HIPAA-eligible attestation |
| §164.312(a)(1) | Access control | Per-tenant scope + RLS + Cedar |
| §164.312(b) | Audit controls | Audit-chain emission |
| §164.312(c)(1) | Integrity | Ed25519 + Merkle |
| §164.312(d) | Person/entity authentication | OIDC + MFA + SPIFFE |
| §164.312(e)(1) | Transmission security | TLS 1.3 |
| §164.314(a)(1) | Business associate contracts | BAA template (Slice D) |
| §164.316(a)+(b)(2) | Policies + procedures + 6y retention | This file + retention matrix |
| §164.502(a) | Permitted uses (TPO) | DPIA §2.4 |
| §164.502(b) | Minimum necessary | Per-tenant scope + OTel redactor |
| §164.404/§164.406/§164.408 | Notifications | `incident-response.md` |

### pack-eu (GDPR + EDPB + eIDAS + NIS2 + EU AI Act)

- GDPR Arts. + EDPB Guidelines: as cross-mapped above.
- eIDAS 910/2014 Art. 26 (AdES): Ed25519 audit-chain seals satisfy when sealing EU-tenant supervisor records.
- NIS2 (2022/2555): incident reporting 24h / 72h / 1mo timelines per `incident-response.md` §"NIS2".
- **EU AI Act**: as enumerated in §"EU AI Act" above; engaged on first high-risk Annex III tenant.

### pack-jp (APPI)

| Art. | Requirement | Implementation |
|---|---|---|
| APPI Art. 17 | Purpose of use | DPIA §2.4 |
| APPI Art. 18 | Purpose limitation | DPIA §2.4 |
| APPI Art. 20 | Security control measures | `policy/supervisor-isolation.md` + `threat-model.md` |
| APPI Art. 21 | Supervision of employees + entrustees | `legal/sub-processors.md` |
| APPI Art. 23 | Third-party provision | DPA + cross-border SCCs |
| APPI Art. 24 | Cross-border restrictions | `policy/data-residency.md` JP-pack pinning |
| APPI Art. 26-2 | Data breach reporting | `incident-response.md` |
| APPI Art. 27 | Sensitive data consent | Tenant DPA |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/foundry-supervisor-compliance-overlay.md`.

## Continuous Compliance Evidence

### Lane: `oya-governance-compliance-evidence-recency`

Refuses merges if any evidence artifact > 90d stale.

### Evidence emission

- `evidence/compliance/<framework>/<control>/<date>.json` — control evidence
- `microservices/foundry-supervisor/evidence/multispectrum/<change_id>-<unix_ts>.json` — per-changeset evidence

Per-framework cadence:
- Daily: SOC 2 CC4/CC7; ISO 27001 A.8.15/A.8.16; EU AI Act Art. 12 audit-chain emission verification.
- Weekly: CC8; A.5.27; EU AI Act post-market monitoring snapshot.
- Monthly: CC3; A.5.7.
- Quarterly: this entire matrix re-validated.
- Annually: full external auditor re-attestation; EU AI Act notified-body re-engagement (if Annex III tenants present).

## Audit Evidence Delivery

External auditors receive a frozen evidence pack per `docs/templates/evidence-pack-template.md`; JIT token (per `policy/auditor-scope.cedar`) scopes their read; every read audit-chain-emitted.

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance-evidence-recency --microservice foundry-supervisor` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0.
- Annual SOC 2 Type 2 + ISO 27001:2022 + EU AI Act notified-body (when applicable) audits.

## References

- `microservices/foundry-supervisor/threat-model.md`.
- `microservices/foundry-supervisor/dpia.md`.
- `microservices/foundry-supervisor/policy/{supervisor-isolation, data-residency}.md`.
- `microservices/foundry-supervisor/policy/*.cedar`.
- `microservices/foundry-supervisor/incident-response.md`.
- ADR-0028, ADR-0117, ADR-0123, ADR-0130, ADR-0131, ADR-0140.
- SOC 2: `aicpa.org/topic/audit-assurance/audit-and-assurance-greater-than-soc-2`.
- ISO 27001:2022: `iso.org/standard/27001`.
- GDPR + EDPB: `gdpr-info.eu`, `edpb.europa.eu`.
- EU AI Act 2024/1689: `eur-lex.europa.eu/eli/reg/2024/1689`.
- HIPAA: `hhs.gov/hipaa`.
- KR PIPA + ISMS-P: `pipc.go.kr`, `kisa.or.kr`.
- APPI: `ppc.go.jp`.
- PDPA (SG): `pdpc.gov.sg`; MAS: `mas.gov.sg`.
- Privacy Act 1988 (AU): `oaic.gov.au`; APRA: `apra.gov.au`.
- DPDPA 2023 (IN): `meity.gov.in`.
- LGPD (BR): `gov.br/anpd`.
- UAE PDPL: `mohre.gov.ae`.
- KSA PDPL: `sdaia.gov.sa`; SAMA: `sama.gov.sa`.
