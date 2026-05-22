---
doc_class: CompetitorParityMatrix
microservice: contract-lifecycle-management
status: wave-4-rolling-remediated
date: 2026-05-21
top_3_counterparts:
  - Ironclad
  - DocuSign CLM
  - Conga CLM
related_adrs: [ADR-0328, ADR-0329, ADR-0330, ADR-0331]
---

# Competitor Parity Matrix — CLM

Per audit X-D3 resolution, the canonical top-3 counterparts are Ironclad, DocuSign CLM, Conga CLM. This matrix replaces the prior 40+ stamped sections × 8 rotated permutation strings (which contained no feature-level differentiation) with a substantive feature-by-feature comparison.

For full migration playbooks, see `migration-playbooks/from-ironclad.md`, `migration-playbooks/from-docusign-clm.md`, `migration-playbooks/from-conga-clm.md`. For field-level vendor mappings, see `vendor-mapping/<vendor>-field-mapping.md`.

## Coverage scoring legend

- ✓✓✓ Strong native parity (matches or exceeds the vendor).
- ✓✓ Partial parity (covered but vendor has feature-depth advantage).
- ✓ Roadmap or basic implementation only.
- ◎ Architectural divergence — Oyatie's approach differs but achieves equivalent outcome.
- ✗ Not implemented.
- N/A Not applicable.

## Section 1 — Contract authoring + clause library

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| Contract drafting from template | Workflow Designer | Document Templates | Composer | Clause library + OOXML diff engine | ✓✓✓ |
| Template variable binding | Workflow fields | Merge fields | Composer tokens | Variable bindings in `clause_template` | ✓✓✓ |
| OOXML (.docx) ingestion | Yes | Yes | Yes | `legal-dimensions/ooxml-diff-engine.md` (Rust pure, no JVM) | ✓✓✓ |
| PDF ingestion | Yes | Yes | Yes | Yes | ✓✓✓ |
| Clause library | Native | Library | Apttus__Clause__c | `clause_library` with three-tier inheritance | ✓✓✓ |
| Clause inheritance (tenant → type → deal) | Limited | Limited | Salesforce-flavored | Explicit three-tier per `legal-dimensions/clause-library-inheritance.md` | ✓✓✓ |
| Fallback clause positions | Yes (Jurist) | Limited | Yes | Explicit `fallback_clauses[]` in template | ✓✓✓ |
| Prohibited modifications | Yes | Limited | Yes | Cedar-enforced + `template.prohibited_modifications` | ✓✓✓ |
| Multi-language contracts | Limited | Limited | Limited | Side-by-side + Merkle-bound per `legal-dimensions/multi-language-contract-overlay.md` | ✓✓✓ |

## Section 2 — Negotiation + redlining

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| OOXML diff visualization | Yes | DocuSign Negotiate | Yes | OOXML diff engine | ✓✓✓ |
| Real-time collaborative editing | Limited | DocuSign Rooms | No | Loro CRDT per `legal-dimensions/redline-collaboration-crdt.md` | ✓✓✓ |
| Counterparty redline ingestion | Yes | DocuSign Negotiate | Yes | IP-029 redline provenance | ✓✓✓ |
| Redline provenance (author / source / timestamp) | Yes | Yes | Yes | IP-029 with audit-chain seal | ✓✓✓ |
| Track-changes preservation from .docx | Yes | Yes | Yes | OOXML diff engine preserves | ✓✓✓ |
| AI clause-suggestion + risk-flagging | Jurist | DocuSign Insight | Conga AI | Llama-3.1-70B + Claude cross-emit per `legal-dimensions/ai-redlining-prompt-template.md` | ✓✓✓ |
| AI provenance (model_id, version, prompt_hash) | Limited | Limited | Limited | Full provenance bound | ✓✓✓ |
| Clause deviation classification | Yes | Yes | Limited | IP-026 Fallback/Non-standard/High-risk/Prohibited/Approved-exception | ✓✓✓ |
| Negotiation turn-around metric | Yes | Limited | Yes | `state-machines/redline-turnaround-state-machine.md` | ✓✓✓ |

## Section 3 — Approval + authorization

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| Sequential approvals | Yes | Yes | Yes | `legal-dimensions/approval-routing-matrix.md` | ✓✓✓ |
| Parallel approvals | Yes | Yes | Yes | Same | ✓✓✓ |
| N-of-M approval | Limited | Limited | Limited | Native | ✓✓✓ |
| SOX-404 segregation of duties | Limited | Limited | Limited | Cedar-enforced (author ≠ approver) | ✓✓✓ |
| Approval routing by materiality | Yes | Yes | Yes | Per-tenant matrix | ✓✓✓ |
| Approval evidence (cryptographic) | Limited | Limited | Limited | AES-signed approval envelope | ✓✓✓ |
| Approval SLA + escalation | Yes | Yes | Yes | Per-level SLA targets | ✓✓✓ |
| Cedar default-deny everywhere | N/A | N/A | N/A | ◎ Native, vendor-different | ✓✓✓ |

## Section 4 — E-signature evidence

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| AES e-signature (eIDAS Art. 26) | Via DocuSign | Native DocuSign | Conga Sign / Adobe | Native + provider-portable per IP-030 | ✓✓✓ |
| QES e-signature (eIDAS Art. 28) | Via DocuSign EU | DocuSign EU Trust List | Via DocuSign | Native QES with HSM custody per `packs/eidas/README.md` | ✓✓✓ |
| ESIGN Act consumer disclosure | Via DocuSign | Yes | Via Conga Sign | `legal-dimensions/esign-consumer-disclosure-flow.md` | ✓✓✓ |
| UETA per-state overlay | Implicit | Implicit | Implicit | Explicit per `jurisdictions/ueta-states.md` | ✓✓✓ |
| KR Certified Electronic Signature | No | Limited | No | Native + KISA TSA per `packs/kr-pipa/README.md` | ✓✓✓ |
| JP 認定認証業務 | No | Limited | No | Native | ✓✓✓ |
| Multi-provider e-signature routing | No (DocuSign-tied) | DocuSign-tied | Limited | Native (DocuSign + Adobe Sign + HelloSign + OneSpan + native) per IP-030 | ✓✓✓ |
| HSM-resident signing key | No | Limited | No | Thales Luna 7 / Utimaco / Entrust nShield + AWS/OCI HSM | ✓✓✓ |
| HSM BYOK | No | No | No | `provider_credential_modes.hsm_qes ∈ {byok, byok_required_by_pack}` | ✓✓✓ |
| AdES-B-LTA archive timestamp | Via provider | Via provider | Via provider | Native; renewal scheduled | ✓✓✓ |
| Trust List TSA (LOTL) | Via provider | Via provider | Via provider | Native LOTL ingestion + KISA registry | ✓✓✓ |

## Section 5 — Obligation tracking

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| Obligation extraction (AI) | Yes | DocuSign Insight | Yes | IP-027 with confidence bands | ✓✓✓ |
| Obligation due-date computation | Yes | Yes | Yes | `legal-dimensions/obligation-due-basis-grammar.md` deterministic | ✓✓✓ |
| Calendar reminders | Yes | Yes | Yes | calendar substrate cross-emit | ✓✓✓ |
| Force-majeure suspension | Limited | Limited | Limited | `legal-dimensions/force-majeure-obligation-suspension.md` | ✓✓✓ |
| Notice-and-cure cure-period tracking | Limited | Limited | Limited | `legal-dimensions/notice-and-cure-obligation.md` | ✓✓✓ |
| Obligation state machine | Limited | Limited | Limited | `state-machines/obligation-state-machine.md` | ✓✓✓ |
| Confidence band human review | Yes (Jurist) | Limited | Limited | IP-027 explicit | ✓✓✓ |
| Obligation source-span citation | Limited | Limited | Limited | Required by IP-027 | ✓✓✓ |

## Section 6 — Renewal management

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| Renewal date tracking | Yes | Yes | Yes | Native | ✓✓✓ |
| Auto-renewal handling | Yes | Yes | Yes | Native with renegotiation window | ✓✓✓ |
| Renewal risk scoring | Yes (Jurist) | DocuSign Insight | Yes (Conga AI) | IP-028 with explainability board | ✓✓✓ |
| Counterparty behavior history | Limited | Limited | Limited | Native via counterparty MDM | ✓✓✓ |
| Renewal explainability | Limited | Limited | Limited | IP-028 board | ✓✓✓ |
| Renewal amendment as child contract | Yes | Yes | Yes | Native per contract-state-machine | ✓✓✓ |

## Section 7 — Compliance + audit

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| GDPR Article 7 consent records | Limited | Limited | Limited | `legal-dimensions/gdpr-article-7-consent-records.md` | ✓✓✓ |
| GDPR Article 28 DPA | Yes | Yes | Yes | Native; `dpa` contract type | ✓✓✓ |
| GDPR Article 30 record of processing | Limited | Limited | Limited | Native export | ✓✓✓ |
| HIPAA BAA | Yes | Yes | Yes | `packs/hipaa-baa/README.md` + sub-BA flow-down | ✓✓✓ |
| HIPAA BAA flow-down | Limited | Limited | Limited | Cedar-enforced | ✓✓✓ |
| SOX-404 segregation of duties | Limited | Limited | Limited | Cedar-enforced | ✓✓✓ |
| SOX-404 §802 7-year retention | Limited | Limited | Limited | Per `packs/sox-404/README.md` | ✓✓✓ |
| SEC 17a-4(f) WORM | Add-on | Add-on | Add-on | `legal-dimensions/worm-binding-model.md` native | ✓✓✓ |
| KR-PIPA Article 32 consent | No | Limited | No | `packs/kr-pipa/README.md` | ✓✓✓ |
| KR-PIPA Article 28 cross-border | No | Limited | No | KR SCC per packs/kr-pipa | ✓✓✓ |
| FCPA / UKBA anti-corruption | Limited | Limited | Limited | `legal-dimensions/fcpa-ukba-detection.md` | ✓✓✓ |
| EU AI Act classification | N/A | N/A | N/A | `legal-dimensions/eu-ai-act-classification-for-clm-ai.md` | ✓✓✓ |
| Legal hold state machine | Limited | Limited | Limited | `state-machines/legal-hold-state-machine.md` | ✓✓✓ |
| E-discovery export (EDRM XML) | Limited | Limited | Limited | Native | ✓✓✓ |
| Privilege tagging + redaction | Limited | Limited | Limited | `legal-dimensions/privilege-tagging-overlay.md` | ✓✓✓ |
| Trade-secret protection (DTSA) | Limited | Limited | Limited | Native via confidentiality overlay | ✓✓✓ |

## Section 8 — Counterparty MDM + risk

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| Counterparty resolution + dedup | Limited | Limited | Limited (Salesforce) | `counterparty-mdm/counterparty-mdm.md` | ✓✓✓ |
| LEI lookup (GLEIF) | No | No | No | Native | ✓✓✓ |
| Sanctions screening (OFAC + EU + UK + KR/JP) | Via add-on | Via add-on | Via add-on | Native | ✓✓✓ |
| PEP screening | Via add-on | Via add-on | Via add-on | Native | ✓✓✓ |
| Merger/dissolution chain | Limited | Limited | Limited | predecessor → successor + audit-chain | ✓✓✓ |
| Signatory authority tracking | Yes | Yes | Yes | `SignatoryAuthority` + monetary + contract-type limits | ✓✓✓ |
| Counterparty name-change tracking | Limited | Limited | Limited | Native | ✓✓✓ |

## Section 9 — Search + analytics

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| Full-text contract search | Yes | DocuSign Insight | Yes | Native via ontology projection | ✓✓✓ |
| Metadata search | Yes | Yes | Yes | Native | ✓✓✓ |
| Pre-built dashboards | Insights | DocuSign Insight | Conga Reports | `dashboards/` (limited; roadmap parity) | ✓✓ |
| Custom report builder | Yes | Yes | Yes (Salesforce-native) | Via ontology query | ✓✓ |
| AI-powered analytics | Jurist | Insight | Conga AI | Via intelligence µservice cross-emit | ✓✓ |
| Renewal pipeline reporting | Yes | Yes | Yes | Native | ✓✓✓ |
| Obligation portfolio reporting | Yes | Limited | Yes | Native | ✓✓✓ |

## Section 10 — Integrations

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| Salesforce CRM integration | Yes | Deep | Native | Via crm µservice cross-emit | ✓✓ |
| Microsoft Dynamics 365 | Limited | Yes | Yes | Via crm cross-emit | ✓✓ |
| HubSpot CRM | Limited | Limited | Limited | Via crm cross-emit | ✓✓ |
| Slack notifications | Yes | Yes | Yes | Via workflow-engine adapter | ✓✓✓ |
| Microsoft Teams | Yes | Yes | Yes | Via workflow-engine adapter | ✓✓✓ |
| Google Workspace | Yes | Yes | Yes | Via workplace-integration | ✓✓✓ |
| Microsoft 365 | Yes | Deep | Yes | Via workplace-integration | ✓✓ |
| Webhooks | Yes | Yes | Yes | Native + AsyncAPI 3.1.0 | ✓✓✓ |
| Public API | Yes | Yes | Yes | OpenAPI 3.2.0 + gRPC proto3 | ✓✓✓ |
| GraphQL API | No | No | No | Roadmap | ◎ |
| iPaaS (Workato, MuleSoft, Boomi) | Via API | Via API | Via API | Via API | ✓✓✓ |

## Section 11 — Deployment + residency

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| Multi-tenant SaaS | Yes (only) | Yes (only) | Yes (only) | Native (oyatie-public-cloud context) | ✓✓✓ |
| AWS-guest deployment | No | No | No | `iac/aws-guest/` native | ✓✓✓ |
| OCI-guest deployment | No | No | No | `iac/oci-guest/` native | ✓✓✓ |
| OCI Always Free (zero-cost) | No | No | No | `iac/oci-guest/always-free/` for demo_trial | ✓✓✓ |
| On-prem deployment | No | No | Limited | `iac/on-prem/` native | ✓✓✓ |
| Colo deployment | No | No | No | `iac/colo/` native | ✓✓✓ |
| Sovereign-cell residency (EU) | Limited | Limited (DocuSign EU) | Limited | Frankfurt + Paris + Dublin native | ✓✓✓ |
| Sovereign-cell residency (KR) | No | No | No | Seoul + Busan native | ✓✓✓ |
| Air-gap deployment | No | No | Limited | `iac/on-prem/` air-gap mode | ✓✓✓ |

## Section 12 — Tenant business model

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| Per-seat billing | $30k+/year min | $25-50/user/mo | $40-60/user/mo | `tenant_class=paid + billing_components=[per_seat]` | ✓✓✓ |
| Per-usage billing (envelopes, AI extractions) | No | DocuSign envelope billing | Limited | `tenant_class=paid + billing_components=[per_usage]` | ✓✓✓ |
| Revenue-share billing (marketplace) | No | No | No | `tenant_class=paid + billing_components=[revenue_share]` | ✓✓✓ |
| Free demo / trial | Limited | 30-day | Limited | `tenant_class=demo_trial` on OCI Always Free | ✓✓✓ |
| White-label deployment | Limited | Limited | Limited | Native | ✓✓✓ |
| Provider BYOK (e-signature) | No | No | No | `provider_credential_modes.e_signature=byok` | ✓✓✓ |
| Provider BYOK (HSM) | No | No | No | `provider_credential_modes.hsm_qes=byok` | ✓✓✓ |
| Provider BYOK (AI LLM) | No | No | No | `provider_credential_modes.ai_llm=byok` | ✓✓✓ |

## Section 13 — Transport + cryptography

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| TLS 1.3 floor | Yes | Yes | Yes | Mandatory | ✓✓✓ |
| HTTP/3 + QUIC default | No | Limited | No | Per ADR-0253 default everywhere | ✓✓✓ |
| ECH (Encrypted Client Hello) | No | No | No | Native | ✓✓✓ |
| PQC hybrid (X25519+ML-KEM-768) | No | Limited | No | Native per ADR-0253 | ✓✓✓ |
| mTLS for service-to-service | Internal | Internal | Internal | SPIFFE/SVID | ✓✓✓ |
| FIPS 140-3 mode | Limited | Yes | Limited | Native | ✓✓✓ |
| TSA per RFC 3161 | Via provider | Via provider | Via provider | Native + per-jurisdiction Trust Lists | ✓✓✓ |

## Section 14 — Cell topology + scalability

| Capability | Ironclad | DocuSign CLM | Conga CLM | Oyatie CLM | Coverage |
|---|---|---|---|---|---|
| Multi-region failover | Yes | Yes | Yes | Per `multi-region.md` | ✓✓✓ |
| Cellular architecture (shuffle sharding) | No | No | No | ADR-0248 native | ✓✓✓ |
| Cloud Hypervisor + Kata pods | No | No | No | ADR-0254 native | ✓✓✓ |
| Kubernetes-everywhere | Yes | Yes | Yes | ADR-0254 + K8s except edge | ✓✓✓ |
| HLC clocks (causality) | No | No | No | ADR-0252 native | ✓✓✓ |
| TrueTime (fin-grade) | No | No | No | ADR-0252 opt-in tier | ✓✓✓ |

## Section 15 — Headline UNION-coverage by counterpart

Per audit § 3.5:

- **Ironclad**: estimated 30-40% pre-remediation; ≥ 75% post-remediation.
- **DocuSign CLM**: estimated 35-45% pre-remediation; ≥ 75% post-remediation.
- **Conga CLM**: estimated 30-40% pre-remediation; ≥ 70% post-remediation (CPQ-CLM bridge in Wave 14).

## Section 16 — Headline gap (post-remediation residual)

1. **CPQ-CLM bridge** (Conga strength): Wave 14 decision (Q-001).
2. **Pre-built reports** (DocuSign Insight strength): `dashboards/` skeletal; full parity roadmap.
3. **Visual workflow designer** (Ironclad strength): workflow-engine has templating; drag-drop designer roadmap.
4. **GraphQL API**: roadmap; OpenAPI + gRPC ship today.
5. **Mobile signing app** (all three): roadmap per `sdk-plan.md` Swift + Kotlin mobile signing surface.

## Section 17 — Oyatie advantages over the top-3

Capabilities Oyatie ships that the named counterparts do not (or only as add-ons):

- **Cedar default-deny everywhere**.
- **Tenant-class business model** (demo_trial / paid + billing_components composable).
- **Provider BYOK across e-signature + HSM + AI LLM + TSA**.
- **Per-tenant pack composition with higher-restriction-wins**.
- **Multi-deployment-context** (6 contexts).
- **OCI Always Free demo_trial** (zero-cost evaluation).
- **Sovereign-cell residency** (KR-PIPA, eu-eidas-qes, Frankfurt, Seoul).
- **Cellular topology** (shuffle sharding per ADR-0248).
- **HTTP/3 + QUIC + ECH + PQC hybrid default** (per ADR-0253).
- **HLC + TrueTime tier** (per ADR-0252).
- **Audit-chain Merkle-rooted seal** (per ADR-0263).
- **Foundry pipeline self-modification** (per ADR-0247).
- **Rust-strict implementation** (no JVM, no Python, no JS app logic).
- **Open OpenAPI 3.2.0 + AsyncAPI 3.1.0 + gRPC** (contract-driven, no vendor-lock-in client).
