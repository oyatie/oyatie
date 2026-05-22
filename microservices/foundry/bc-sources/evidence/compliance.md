---
doc_class: ComplianceMatrix
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: council-privacy + axis-foundry-evidence
reviewers: [council-privacy, dpo-eu, dpo-kr, ops-security, legal-counsel]
related_artifacts:
  - microservices/foundry-evidence/PRD.md
  - microservices/foundry-evidence/dpia.md
  - microservices/foundry-evidence/threat-model.md
  - microservices/foundry-evidence/policy/data-residency.md
  - microservices/foundry-evidence/policy/regulator-export-scope.cedar
  - microservices/foundry-evidence/capabilities/regulator-export.yaml
doc_status: published
---

# foundry-evidence — compliance matrix

This µservice is the audit-evidence frontend for the Foundry agent runtime. Its compliance posture is the AI-Act / HIPAA / GDPR / KR PIPA / SOC 2 / ISO 27001 obligation cross-walk for "what the agent did + the evidence to prove it."

## EU AI Act (Regulation (EU) 2024/1689)

| Article | Obligation | foundry-evidence control |
|---|---|---|
| Art. 12 — Recordkeeping (logs) | Maintain machine-readable logs of operation throughout the lifecycle of the high-risk AI system; logs must include period, reference database against which input is checked, identification of natural persons involved | EvidencePack carries `invocation_ts`, `model_version`, `provider`, `agent_id`, `capability_id`, `principal_spiffe_id`; sealed in audit-chain |
| Art. 18 — Technical documentation | Maintain technical documentation demonstrating compliance | regulator-export `eu-ai-act` profile includes Art. 18 fields with citation anchors |
| Art. 26 — Obligations of deployers | Use system in accordance with intended purpose; keep logs ≥ 6 months (or as required by Union/Member State law) | `policy/data-residency.md` retention: 10 y for pack-eu AI-Act high-risk records; T2/T3 autonomy-ceiling decisions tied to deployer obligation |
| Art. 14 — Human oversight | Enable natural persons to oversee the system | autonomy_level T3 + foundry-supervisor escalation records; `governance` µservice review-and-respond loop |
| Art. 50 — Transparency to natural persons | When applicable, inform natural persons interacting with AI system | foundry-evidence captures `transparency_disclosure_evidence_hash` field (when supplied by foundry-runtime); content gated by `payload_data_class` |
| Art. 60-66 — Post-market monitoring | Establish post-market monitoring system | foundry-evidence is the post-market monitoring backbone; evidence-query + dashboards |

## HIPAA (45 CFR §164)

| Section | Obligation | foundry-evidence control |
|---|---|---|
| §164.312(b) Audit controls | Implement hardware, software, and procedural mechanisms that record + examine activity in info systems with ePHI | Every PHI-touching invocation produces an EvidencePack sealed in audit-chain; tamper-evident |
| §164.308(a)(1)(ii)(D) Information system activity review | Procedures to regularly review records of info system activity | dashboards/evidence-query queries; regulator-export `hipaa` profile |
| §164.308(a)(5)(ii)(C) Log-in monitoring | Procedures for monitoring log-in attempts and reporting discrepancies | foundry-evidence captures `principal_spiffe_id` + auth events from foundry-runtime |
| §164.316(b)(1)(i) Documentation | Maintain HIPAA documentation in written or electronic form | All foundry-evidence artifacts are version-controlled in oyatie repo + ADR-0131 compliant |
| §164.316(b)(2) Retention | Retain documentation for 6 years from creation or last effective date | pack-us-healthcare retention 6 y, substrate-cascaded; verified by retention-cascade CI drill |

## GDPR (Regulation (EU) 2016/679)

| Article | Obligation | foundry-evidence control |
|---|---|---|
| Art. 5 Principles | lawfulness, fairness, transparency; purpose limitation; data minimisation; accuracy; storage limitation; integrity + confidentiality; accountability | DPIA Section 3; data-minimisation via `payload_data_class` gating; storage-limitation via retention cascade |
| Art. 22 Automated decision-making | Right not to be subject to a decision based solely on automated processing | EvidencePack carries `autonomy_level_decision` + `autonomy_level_rationale_hash`; T3 carries human-in-the-loop evidence |
| Art. 25 Data protection by design | Implement appropriate technical + organisational measures | Cedar default-deny + SPIFFE + WORM + Merkle seal (substrate) |
| Art. 30 ROPA | Records of processing activities | DPIA Section 7 join + regulator-export `gdpr` profile |
| Art. 32 Security | Pseudonymisation + encryption + ongoing confidentiality, integrity, availability, resilience | TLS 1.3 + mTLS + AES-256-GCM at rest + Ed25519 seal (substrate) |
| Art. 33–34 Breach notification | Notify supervisory authority + data subjects of breach | `incident-response.md` Sev-1 procedure includes regulator + tenant notification |
| Art. 17 Erasure | Right to erasure | DSR cascade via tenancy → audit-chain substrate retention-cascade RPC; foundry-evidence row redacted |

## KR PIPA (Personal Information Protection Act, 개인정보 보호법)

| Article | Obligation | foundry-evidence control |
|---|---|---|
| Art. 23 — Sensitive information | Special handling for ideology, political views, health, sexual orientation, etc. | `payload_data_class=SENSITIVE_PIPA_ART23`; explicit consent entitlement gates plaintext reads |
| Art. 28 — Cross-border transfer | Restriction on transferring personal info outside Korea | `policy/data-residency.md` DR-02 pack-kr resident; substrate WORM in KR region only |
| Art. 29 — Safety measures | Technical + administrative + physical safeguards | Substrate Ed25519 + WORM + Cedar + SPIFFE + audit-of-audits |
| Art. 35 — Right to access | Data subject access | tenancy DSR cascade entry + evidence-query subject_hash filter |
| Art. 36 — Right to deletion | Data subject deletion | substrate retention-cascade with Merkle proof of redaction |

## KR 전자문서법 (Framework Act on Electronic Documents and Transactions)

| Article | Obligation | foundry-evidence control |
|---|---|---|
| Arts. 5–7 — Evidentiary force of electronic documents | Recognise electronic documents as legal evidence under specified conditions | Each EvidencePack sealed with Ed25519 (substrate); satisfies "advanced electronic signature" + eIDAS Art. 26 equivalence |

## SOC 2 (Trust Services Criteria 2017, rev. 2022)

| Criterion | Control |
|---|---|
| CC4.1 — Monitoring activities | Continuous SLI + alert + dashboards; evidence-query + dashboards |
| CC4.2 — Communication of internal control deficiencies | failure-modes.md + incident-response.md procedures |
| CC6.1 — Logical access security | Cedar + SPIFFE + mTLS |
| CC7.1 — System monitoring | observability µservice ingests foundry-evidence SLI |
| CC7.2 — Change management | ADR + LEAN gates + no-silent-regression lane |
| CC7.3 — Security event management | regulator-export-reissue + audit-chain-backlog + pack-assembly-fail runbooks |
| CC7.4 — Incident response | incident-response.md |

## ISO/IEC 27001:2022 + Annex A

| Control | foundry-evidence |
|---|---|
| A.5.28 Collection of evidence | EvidencePack + substrate Merkle seal |
| A.8.15 Logging | Substrate emit of every evidence-pack + audit-of-audits on every read |
| A.5.17 Authentication information | SPIFFE rotation via cloud-secrets |
| A.8.24 Cryptography | substrate Ed25519 + HSM-backed signing |
| A.5.34 Privacy + PII protection | Cedar `tenant-scope.cedar` + DPIA |
| A.8.34 Protection of information systems during audit | auditor-scope.cedar Cedar permits |

## eIDAS (Regulation (EU) No 910/2014)

| Article | Obligation | foundry-evidence control |
|---|---|---|
| Art. 25–26 — Electronic signatures | Advanced electronic signature definition; trust effects | Substrate Ed25519 signatures on Merkle roots + bundle signatures qualify as advanced electronic signatures |

## Cross-framework regulator-export coverage

| Framework | Bundle profile | Field-completeness CI lane |
|---|---|---|
| eu-ai-act | required: Art. 12 logs + Art. 18 tech doc + Art. 26 deployer fields | regulator-profile-drill |
| hipaa | required: §164.312(b) audit-control fields | regulator-profile-drill |
| gdpr | required: Art. 30 ROPA + Art. 22 automated-decision evidence | regulator-profile-drill |
| kr-pipa | required: Art. 29 safety + Art. 23 sensitive-info handling | regulator-profile-drill |
| soc2 | required: CC4.1 + CC7.2 evidence fields | regulator-profile-drill |
| iso-27001 | required: A.5.28 + A.8.15 evidence fields | regulator-profile-drill |

## Compliance gates (CI-enforced)

```bash
oya gate validate compliance-claims --microservice foundry-evidence
oya gate validate regulator-profile-drill --microservice foundry-evidence
oya gate validate cross-pack-replication-forbidden --microservice foundry-evidence
oya gate validate retention-cascade-on-cadence --microservice foundry-evidence
oya gate validate hyperscaler-maturity-claims
```

## Review cadence

- Annual full compliance review by council-privacy + DPO of each pack + legal-counsel.
- Out-of-cycle on:
  - New framework added (e.g., subsequent-to-M01-completion vertical overlay like FDA 21 CFR Part 11 for medical-device tenants on pack-us-healthcare).
  - Substantive amendment to any cited framework (e.g., EU AI Act amendment).
  - Substrate (audit-chain) compliance posture change.
- Sign-off: council-privacy chair + legal-counsel + DPO of any affected pack.

## ADR-0133 honesty annotation

Every framework row above ties a control to a CI lane or runbook. No "we'll get to it" entries. The `hyperscaler-maturity-claims` lane refuses commit-claims of compliance coverage that cannot be CI-asserted.
