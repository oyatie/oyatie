---
doc_class: Feature-Parity-Matrix
microservice: contract-lifecycle-management
status: Wave-4-Rolling-Audit-Companion
wave: Wave-4-Rolling-Legal-Complexity-CLM
date: 2026-05-21
auditor_agent_class: codex-ms-audit-contract-lifecycle-management
audit_priority: P0-Legal-Complexity
parity_set: [Ironclad, DocuSign CLM, Conga CLM]
companion_audit_deliverables:
  - microservices/contract-lifecycle-management/coherence-audit-2026-05-20.md
  - microservices/contract-lifecycle-management/performance-benchmark-numbers-2026-05-20.md
union_coverage_bar: Ironclad ∪ DocuSign CLM ∪ Conga CLM
---

CANONICAL ANCHORS

1. /Users/jasonlee/oyatie/docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md §D-15..D-20 substance-bar + batch discipline; legal-complexity µservice variant of Big-8 P0 escalation.
2. Ironclad official product docs — https://docs.ironcladapp.com/ + Workflow Designer + Repository + Jurist AI + Insights + Public API; Ironclad CLAUSE-LIBRARY semantics; Workflow Approval matrix.
3. DocuSign CLM official docs — https://support.docusign.com/s/document-item?bundleId=lqu1644367762589 + SpringCM legacy data model + DocuSign Insight + DocuSign Negotiate + DocuSign Gen + Salesforce CLM integration.
4. Conga CLM (formerly Apttus) official docs — https://documentation.conga.com/clm + Conga Contracts + Conga Composer + Conga Sign + Conga AI + CPQ-CLM bridge.
5. /Users/jasonlee/oyatie/microservices/contract-lifecycle-management/PRD.md §B (five bounded contexts: contract-intake, clause-library, negotiation, obligation, renewal) and IP-001..IP-030 (canonical Oyatie CLM capability inventory; IP-026..IP-030 are legal-domain bespoke; IP-001..IP-025 are structural backbone).
6. /Users/jasonlee/oyatie/microservices/contract-lifecycle-management/capability-tiers/tier-matrix.md §"Vendor displacement table" — current internal vendor comparison (retired named capability levels stratified; this audit deliverable redraws onto post-tier model per coherence-audit T-016).

# Feature Parity Matrix: Contract Lifecycle Management

## §1 Purpose

This matrix maps the canonical capability inventory of the three Legal-Complexity industry-counterparts (Ironclad, DocuSign CLM, Conga CLM) against the present-state Oyatie contract-lifecycle-management capability surface. The UNION-coverage rule means Oyatie CLM must support any capability present in any one of the three counterparts; per-counterpart absence is not a basis to drop a capability.

The matrix is organised by canonical CLM capability family. Each capability row shows:
- Capability — what the capability does at industry-leader granularity.
- C1 has (Ironclad) — Ironclad-specific surface name + presence.
- C2 has (DocuSign CLM) — DocuSign-CLM-specific surface name + presence.
- C3 has (Conga CLM) — Conga-CLM-specific surface name + presence.
- UNION required — true if any of C1/C2/C3 has it (always true for canonical capabilities).
- Oyatie CLM has — what is present in microservices/contract-lifecycle-management/ today, cited file:line where possible.
- Gap classification — one of {PARITY, PARTIAL, MISSING, DELEGATED, NEEDS-DECISION}.

PARITY means the Oyatie surface is functionally equivalent at the substance-bar floor. PARTIAL means structural surface is present but at less than counterpart depth. MISSING means absent. DELEGATED means owned by another Oyatie µservice (with that µservice named). NEEDS-DECISION means Wave 14 ownership question.

The brief calls out specific capability families: contract authoring, template library, clause library, redlining, approval workflows, e-signature, repository, search, AI extraction, contract analytics, integrations with CRM/ERP, mobile, e-discovery, audit trail. The §3 matrix below organises capabilities into seven families that span these surfaces:

A. Contract authoring + template library + clause library (§3.1).
B. Negotiation + redlining + approval workflow (§3.2).
C. E-signature + cryptographic evidence (§3.3).
D. Repository + search + retention + WORM (§3.4).
E. AI extraction + obligation + renewal-risk + analytics (§3.5).
F. Integrations: CRM / ERP / mail / calendar / workflow / marketplace (§3.6).
G. Mobile + e-discovery + audit trail + reporting (§3.7).

## §2 Counterpart reference inventories

This section establishes the per-counterpart reference inventory used to construct the matrix.

### §2.1 Ironclad canonical capability set

I1. Workflow Designer — Ironclad's visual workflow builder where each step has fields, approvals, and conditions. Drag-and-drop authoring.

I2. Workflow Repository — Workflows produced by Designer become repository contracts; each repository row is a Workflow+Document pair with full version history.

I3. Clause Library — Ironclad Clause Library with named clauses, fallback variants, playbook bindings, and per-clause approval routing.

I4. Templates (Word Tags) — Ironclad Templates use Word Tags inside .docx files; field-binding via mail-merge style placeholders.

I5. Smart Import — Ironclad Smart Import AI ingests existing contracts (PDF/DOCX) and extracts metadata + obligations into the repository.

I6. Smart Search — Ironclad Smart Search with semantic search across contract content + metadata + clauses.

I7. Approvals (Conditional Approval) — Ironclad Conditional Approval routes based on contract value, clause deviation, counterparty type.

I8. Email Capture — Ironclad Email Capture inbound: redlines and approvals received via email are auto-routed into the repository.

I9. Editor (Word Online integration) — Ironclad Editor uses Microsoft Word Online for native editing; tracked changes flow into Workflow.

I10. Redlining (CompareDocs) — Ironclad's diff engine compares draft vs counter-redline.

I11. Jurist AI (clause-suggestion + risk-flagging) — Ironclad's AI assistant powered by OpenAI; suggests clauses, flags risk, drafts emails.

I12. Insights (analytics) — Ironclad Insights with dashboards on cycle time, deviation frequency, counterparty patterns.

I13. Repository Search (faceted) — full-text + metadata + tag faceted search across repository.

I14. Public API (REST) — Ironclad Public API v1 with /workflows, /records, /approvals, /documents, /webhooks.

I15. Webhooks — Ironclad Webhooks for workflow-state-change, document-uploaded, approval-decided.

I16. Salesforce integration — Ironclad for Salesforce; bidirectional sync of Account / Opportunity / Contract with Ironclad Workflows.

I17. HubSpot integration — Ironclad for HubSpot.

I18. Microsoft Dynamics integration — Ironclad for Microsoft Dynamics 365.

I19. NetSuite / Procurement integrations — Ironclad NetSuite + procurement system integrations.

I20. SSO / SCIM — Ironclad SAML SSO + SCIM provisioning.

I21. Custom Fields — Ironclad allows custom fields per Workflow.

I22. Bulk Send — Ironclad Bulk Send for sending the same contract to multiple counterparties.

I23. Sign with Ironclad — Ironclad Sign (Ironclad's own e-signature, supplanting DocuSign for some customers; Ironclad-native + DocuSign integration both supported).

24. Notification routing — Ironclad email notifications + Slack notifications.

I25. Audit Trail — Ironclad Audit Trail recording every change.

### §2.2 DocuSign CLM canonical capability set

D1. Contract Repository (SpringCM legacy) — DocuSign CLM Repository with folder hierarchy + document version history.

D2. Document Generation (DocuSign Gen) — Contract document generation from Salesforce/Dynamics/SAP data + templates.

D3. Clause Library — DocuSign CLM Clause Library with named clauses + variants + per-clause routing.

D4. Workflow Designer — DocuSign CLM Workflow Designer (XML/JSON workflows); approval chains; conditional logic.

D5. Tag-Based Document Properties — SpringCM legacy tag system; each document has tag-value metadata.

D6. DocuSign eSignature integration — DocuSign CLM is natively integrated with DocuSign eSignature (the dominant e-signature SaaS).

D7. DocuSign Negotiate — Native counterparty negotiation tool; redlines via DocuSign Negotiate (not Word).

D8. DocuSign Insight — AI-powered contract analytics across repository; reads contract content; surfaces risk + key terms.

D9. DocuSign Identify — Identity verification (governmental ID + selfie) for QES + KYC signatures.

D10. AI Tagging — DocuSign CLM AI Tagging auto-suggests metadata tags during ingestion.

D11. Salesforce-native — DocuSign CLM has Salesforce-native deployment (DocuSign for Salesforce).

D12. Microsoft 365 integration — Word add-in for DocuSign CLM clause library access during Word editing.

D13. SAP integration — SAP CPI connector for DocuSign CLM.

D14. Repository Search — Federated search across contract content + metadata.

D15. Approval Process — DocuSign CLM approval process with sequential + parallel + conditional routing.

D16. Reminders — Auto-reminders to approvers + signers.

D17. Bulk Operations — Bulk update of metadata + bulk export.

D18. Mass Migration Tooling — DocuSign CLM has reference migration tooling for SharePoint, network shares, legacy CLM products.

D19. Audit Log — DocuSign CLM audit log captures all user activity + system events.

D20. eIDAS QES via DocuSign EU Trust List — DocuSign EU Advanced Signature + QES through DocuSign EU Trust List provider.

D21. UELMA / UCC / ESIGN compliance — DocuSign CLM is ESIGN Act compliant by default; UCC + UELMA evidence packets available.

D22. SEC 17a-4(f) WORM — DocuSign CLM WORM storage add-on for broker-dealer compliance.

D23. HIPAA-eligible deployment — DocuSign CLM HIPAA-eligible service with BAA available.

D24. SOC-2 + ISO-27001 + FedRAMP — DocuSign CLM holds SOC-2 Type II, ISO-27001, FedRAMP Moderate (DocuSign Federal).

D25. Mobile — DocuSign CLM mobile app (iOS + Android).

D26. DocuSign Notary — DocuSign Notary integration for notarised electronic signatures.

D27. Smart Sections — DocuSign CLM dynamic document assembly with conditional sections.

D28. Salesforce Quote-to-Contract — DocuSign CLM for Salesforce CPQ-to-Contract handoff.

### §2.3 Conga CLM canonical capability set

G1. Conga Contracts (Agreement entity on Salesforce) — Conga CLM is Salesforce-native; Agreement entity inherits Salesforce platform features.

G2. Conga Composer — Document generation from Salesforce data + Microsoft Word/Excel/PowerPoint templates.

G3. Conga Sign — Conga's native e-signature; AES under eIDAS.

G4. Conga AI — Conga's AI for clause-extraction, risk-scoring, obligation-extraction.

G5. Conga Approvals — Approval routing engine on Salesforce.

G6. Conga Contracts Workflow — Workflow engine on Salesforce.

G7. Clause Library (Conga Contracts Clause Library) — Named clauses + variants + per-clause approval matrix.

G8. CPQ-CLM Bridge — Conga CPQ to Conga CLM native integration; quotes generate Agreements.

G9. Conga Contracts for Microsoft Word — Word add-in with clause library access + redline tracking.

G10. X-Author for Word — Conga's Word add-in for advanced template editing.

G11. Conga Grid — Excel-like grid for bulk contract editing.

G12. Conga Sign Negotiate — Conga's redline tool (formerly Negotiation Cloud).

G13. Salesforce-native everything — Conga inherits Salesforce platform features (Reports, Dashboards, Chatter, Email, Workflow).

G14. Conga Contracts for Salesforce CPQ — Direct CPQ-to-Contract pipeline.

G15. Conga Document Generation Cloud — Cloud-native document gen platform.

G16. Conga Sign Cloud — Cloud-native e-signature.

G17. SAP integration — Conga Contracts for SAP.

G18. Coupa integration — Conga Contracts for Coupa (procurement).

G19. Workday integration — Conga Contracts for Workday (HR contracts).

G20. Multi-language contract support — Conga supports multi-language documents.

G21. Conga Contract Intelligence — Smart obligation extraction during ingestion.

G22. Conga Renewal Management — Automated renewal alerts + risk scoring.

G23. SLA / Obligation Management — Conga's obligation tracker for post-signing commitments.

G24. Audit Trail — Conga full audit trail on Salesforce.

G25. Mobile (Salesforce Mobile) — Conga CLM accessible via Salesforce Mobile + Conga Mobile-specific app.

G26. eIDAS / ESIGN compliance — Conga Sign is eIDAS AES + ESIGN compliant.

G27. SOC-2 + GDPR — Conga holds SOC-2 + GDPR compliance.

G28. Per-tenant data isolation (Salesforce org) — Each Conga customer is a Salesforce org with full tenant isolation.

## §3 UNION-coverage matrix (capability × C1 × C2 × C3 × UNION × Oyatie × Gap)

### §3.1 Family A: Contract authoring + template library + clause library

| Capability | C1 (Ironclad) | C2 (DocuSign CLM) | C3 (Conga CLM) | UNION | Oyatie CLM | Gap class |
|---|---|---|---|---|---|---|
| Visual workflow designer for contract authoring | YES (Workflow Designer) | YES (Workflow Designer) | YES (Conga Contracts Workflow) | YES | DELEGATED — workflow-engine µservice owns; CLM ARCHITECTURE §D names workflow-engine as integration; the per-flow contract is not authored | DELEGATED |
| DOCX template library | YES (Templates + Word Tags) | YES (DocuSign Gen templates) | YES (Conga Composer + X-Author) | YES | MISSING — no per-tenant template library primitive in PRD §B | MISSING |
| Clause library with named clauses | YES (Clause Library) | YES (Clause Library) | YES (Clause Library) | YES | PARTIAL — clause-library bounded context (PRD §B); ARCHITECTURE §C invariants present; no taxonomy + inheritance + per-clause approval matrix | PARTIAL |
| Clause variants / fallback clauses | YES (Fallback clauses on each clause) | YES (Clause variants) | YES (Clause alternatives) | YES | PARTIAL — IP-026 ClauseDeviation classification (fallback / non-standard / high-risk / prohibited / approved-exception); no library-side fallback authoring UX | PARTIAL |
| Per-clause approval routing | YES (Conditional Approval per clause) | YES (Per-clause routing) | YES (Per-clause approval matrix) | YES | PARTIAL — IP-026 §"Approvals" mentions approval requirement; matrix not enumerated | PARTIAL |
| Template field binding (data merge) | YES (Word Tags) | YES (Smart Sections + Tag-based) | YES (Composer field syntax) | YES | MISSING — no field-binding primitive | MISSING |
| Smart Sections / Conditional document assembly | PARTIAL (Conditional clauses) | YES (Smart Sections) | YES (Conditional logic in Composer) | YES | MISSING | MISSING |
| Multi-language template support | PARTIAL (Workflow per language) | PARTIAL (Multi-language) | YES (Native multi-language) | YES | MISSING — coherence audit L-019 notes the gap | MISSING |
| Contract type taxonomy (MSA/SOW/NDA/DPA/BAA/...) | YES (Workflow per type) | YES (Document Type) | YES (Agreement Type) | YES | MISSING — coherence audit S-005 notes the gap | MISSING |

Family A summary: 0 PARITY, 4 PARTIAL, 4 MISSING, 1 DELEGATED, 0 NEEDS-DECISION. Active gap = 8/9 = 89%.

### §3.2 Family B: Negotiation + redlining + approval workflow

| Capability | C1 (Ironclad) | C2 (DocuSign CLM) | C3 (Conga CLM) | UNION | Oyatie CLM | Gap class |
|---|---|---|---|---|---|---|
| Word Online native editing | YES (Ironclad Editor uses Word Online) | YES (Word add-in) | YES (X-Author + Word add-in) | YES | DELEGATED — drive µservice document storage; no Word Online integration declared | DELEGATED |
| Redline tracking with actor / source / timestamp | YES (CompareDocs + tracked changes) | YES (DocuSign Negotiate) | YES (Conga Sign Negotiate) | YES | PARTIAL — IP-029 counterparty-redline-provenance; RedlineEvent in ADR-CLM-001; actor/source/timestamp specified | PARTIAL |
| Counterparty redline upload | YES (Email Capture inbound) | YES (DocuSign Negotiate) | YES (Conga Sign Negotiate) | YES | PARTIAL — IP-026 redlineThreadOpen.create capability; ingress flow not enumerated | PARTIAL |
| Diff engine (compare draft vs counter-redline) | YES (CompareDocs) | YES (Native diff) | YES (Word-based diff) | YES | MISSING — diff-engine choice (docx4j? Aspose? Python-docx?) not declared in PRD | MISSING |
| Real-time collaborative editing | PARTIAL (Word Online co-edit) | PARTIAL (Word add-in co-edit) | PARTIAL (Word add-in co-edit) | YES | PARTIAL — capability-tiers/tier-matrix.md mentions Loro CRDT at retired-advanced tier; not in PRD | PARTIAL |
| AI clause suggestion / draft assistance | YES (Jurist AI via OpenAI) | YES (Insight + Word add-in) | YES (Conga AI) | YES | PARTIAL — capability-tiers/tier-matrix.md mentions Llama-3.1 + Claude cross-emit; coherence audit S-013 notes prompt template missing | PARTIAL |
| AI risk-flagging (clause deviation risk) | YES (Jurist AI risk flagging) | YES (Insight) | YES (Conga AI risk) | YES | PARTIAL — IP-026 ClauseDeviation classification names high-risk + prohibited categories; AI scoring of risk not specified | PARTIAL |
| Approval matrix (sequential / parallel / conditional) | YES (Conditional Approval) | YES (Approval Process) | YES (Conga Approvals) | YES | DELEGATED — workflow-engine µservice; approval-route capability surfaces in CLM but matrix not declared | DELEGATED (per Q-005-like decision) |
| Approval-on-clause-deviation gate | YES (Conditional Approval) | YES (Conditional Approval) | YES (Approval Matrix) | YES | PARTIAL — IP-026 approval requirement on clause-deviation; routing matrix not declared | PARTIAL |
| Notification routing (email / Slack / Teams) | YES (Email + Slack) | YES (Email) | YES (Salesforce notifications + email) | YES | DELEGATED — mail + connect µservices; not declared in ARCHITECTURE §D | DELEGATED |

Family B summary: 0 PARITY, 6 PARTIAL, 1 MISSING, 3 DELEGATED, 0 NEEDS-DECISION. Active gap = 10/10 = 100% (10/10 with active-gap counting DELEGATED-without-declared-boundary as active gap).

### §3.3 Family C: E-signature + cryptographic evidence

| Capability | C1 (Ironclad) | C2 (DocuSign CLM) | C3 (Conga CLM) | UNION | Oyatie CLM | Gap class |
|---|---|---|---|---|---|---|
| SES (Simple Electronic Signature) | YES (Ironclad Sign) | YES (DocuSign eSignature default) | YES (Conga Sign) | YES | PARTIAL — IP-030 e-signature-provider-portability; SES envelope shape not declared | PARTIAL |
| AES (Advanced Electronic Signature) under eIDAS Art. 26 | YES (Ironclad Sign + DocuSign integration) | YES (DocuSign EU Advanced) | YES (Conga Sign AES) | YES | PARTIAL — capability-tiers/tier-matrix.md mentions AES at retired-basic+; AES evidence model (PKCS#7/CMS/CAdES/PAdES) not declared; coherence audit L-002 | PARTIAL |
| QES (Qualified Electronic Signature) under eIDAS Art. 28 | PARTIAL (via DocuSign EU Trust List integration) | YES (DocuSign EU Trust List) | PARTIAL (via partners) | YES | PARTIAL — capability-tiers/tier-matrix.md mentions QES at retired-standard+ (Cosign/Cryptomathic) and retired-sovereign (in-pack HSM); QES Trust List binding not declared; coherence audit L-003 | PARTIAL |
| ESIGN Act 15 USC § 7001 consumer disclosure | YES (Ironclad Sign) | YES (DocuSign default) | YES (Conga Sign) | YES | MISSING — coherence audit L-004; no consumer-disclosure flow in PRD | MISSING |
| UETA per-state evidence | YES (DocuSign integration) | YES (DocuSign default) | YES (Conga Sign) | YES | MISSING — coherence audit L-005 | MISSING |
| Time-Stamp Authority (TSA) integration | PARTIAL (via signature provider) | YES (DocuSign TSA) | PARTIAL (via signature provider) | YES | PARTIAL — capability-tiers/tier-matrix.md mentions KISA / DSS / RFC 3161 TSAs at retired-advanced/retired-sovereign; binding not in PRD; coherence audit S-007 | NEEDS-DECISION (TSA owner: CLM vs kms) |
| Multi-provider e-signature routing | YES (Ironclad Sign + DocuSign + Adobe) | PARTIAL (DocuSign-native) | PARTIAL (Conga Sign primary, partner-providers) | YES | PARTIAL — IP-030 e-signature-provider-portability; multi-provider routing logic not enumerated | PARTIAL |
| BYOK e-signature provider credentials | YES (customer-DocuSign account integration) | NO (DocuSign-native always) | YES (customer e-sign provider) | YES | MISSING — coherence audit C-011 notes provider_credential_mode missing from manifest | MISSING |
| HSM-resident signing keys (BYOK) | PARTIAL (via DocuSign EU Trust List provider HSM) | YES (DocuSign EU Trust List HSM) | PARTIAL (via partner HSM) | YES | PARTIAL — capability-tiers/tier-matrix.md retired-sovereign mentions Thales Luna 7 A790; coherence audit C-012 notes byok_hsm_mode missing | NEEDS-DECISION (HSM owner: CLM vs kms) |
| Identity verification (ID + selfie) at signing | NO | YES (DocuSign Identify) | PARTIAL (via DocuSign Identify integration) | YES | MISSING | DELEGATED (identity µservice expected) |
| Notarised signature (online notary) | NO | YES (DocuSign Notary) | PARTIAL (via partner) | YES | MISSING | NEEDS-DECISION |
| Bulk send (one contract → many counterparties) | YES (Bulk Send) | YES (Bulk Send) | YES (Conga Grid + bulk send) | YES | MISSING | MISSING |
| Signature envelope cryptographic evidence packet (CAdES/PAdES) | PARTIAL (via provider) | YES (PAdES via DocuSign) | PARTIAL (via Conga Sign or partner) | YES | PARTIAL — ADR-CLM-001 mentions evidence packet ref; per-envelope CAdES/PAdES choice not declared | PARTIAL |

Family C summary: 0 PARITY, 5 PARTIAL, 4 MISSING, 1 DELEGATED, 3 NEEDS-DECISION. Active gap = 13/13 = 100%. CLM has the deepest e-signature complexity in the µservice ecosystem; the gap is structural rather than depth-only.

### §3.4 Family D: Repository + search + retention + WORM

| Capability | C1 (Ironclad) | C2 (DocuSign CLM) | C3 (Conga CLM) | UNION | Oyatie CLM | Gap class |
|---|---|---|---|---|---|---|
| Contract repository (long-term storage) | YES (Repository) | YES (SpringCM Repository) | YES (Salesforce Files + custom storage) | YES | NEEDS-DECISION — drive µservice owns object storage; CLM ownership unclear (coherence audit Q-003) | NEEDS-DECISION |
| Folder hierarchy | PARTIAL (flat with tags) | YES (Folder hierarchy) | YES (Salesforce Folders) | YES | MISSING — flat aggregate model | MISSING |
| Tag / metadata model | YES (Custom Fields per Workflow) | YES (Tag-based) | YES (Custom Fields) | YES | PARTIAL — contract_intake_document keyed by document id + source-system id + status + data class + region + pack + workflow run; tag taxonomy not declared | PARTIAL |
| Version history (document + metadata) | YES (Workflow version history) | YES (Document version history) | YES (Field history) | YES | PARTIAL — ADR-CLM-001 §"contract versions cannot be destructively corrected"; version history is implied by append-only ledger | PARTIAL |
| Full-text search across content | YES (Smart Search) | YES (Federated Search) | PARTIAL (Salesforce search) | YES | DELEGATED — search µservice owns; CLM ↔ search contract not authored | DELEGATED |
| Semantic search (AI-powered) | YES (Smart Search) | YES (Insight Search) | PARTIAL (Conga AI) | YES | DELEGATED — search + intelligence µservices | DELEGATED |
| Faceted search (filter by status / type / counterparty) | YES (Repository Search) | YES (Repository Search) | YES (Salesforce search) | YES | DELEGATED — search µservice | DELEGATED |
| Retention policy per contract type | YES (Configurable per Workflow) | YES (Retention via DocuSign Gen) | YES (Salesforce + Conga retention) | YES | PARTIAL — manifest.json compliance_packs include SOX-404 (7-year retention) + KR-PIPA; per-contract-type retention overlay missing; coherence audit L-006 + L-008 | PARTIAL |
| WORM (Write-Once-Read-Many) storage | PARTIAL (Add-on) | YES (DocuSign CLM WORM) | PARTIAL (Add-on) | YES | MISSING — capability-tiers/tier-matrix.md mentions SeaweedFS Compliance at retired-standard+; not bound in PRD; coherence audit L-016 | MISSING |
| Legal hold (preserve under litigation) | YES (Legal Hold flag) | YES (Legal Hold) | YES (Legal Hold) | YES | PARTIAL — runbook legal-hold-activation.md exists; state machine not declared; coherence audit L-011 | PARTIAL |
| Data residency per pack | YES (Region selection) | YES (DocuSign Federal + GovCloud) | YES (Salesforce data residency) | YES | PARTIAL — manifest.json packs include KR-PIPA; sovereign-pack × deployment-context matrix not authored; coherence audit D-010 | PARTIAL |
| Bulk export (with audit references) | YES (Public API export) | YES (Bulk Operations) | YES (Conga Grid export) | YES | PARTIAL — ADR-CLM-001 §"exported contract packets must include audit references and redaction evidence"; flow not declared | PARTIAL |

Family D summary: 0 PARITY, 6 PARTIAL, 2 MISSING, 3 DELEGATED, 1 NEEDS-DECISION. Active gap = 12/12 = 100% (1 NEEDS-DECISION is the canonical repository owner).

### §3.5 Family E: AI extraction + obligation + renewal-risk + analytics

| Capability | C1 (Ironclad) | C2 (DocuSign CLM) | C3 (Conga CLM) | UNION | Oyatie CLM | Gap class |
|---|---|---|---|---|---|---|
| AI obligation extraction at ingestion | YES (Smart Import) | YES (AI Tagging + Insight) | YES (Conga Contract Intelligence) | YES | PARTIAL — IP-027 obligation-extraction-confidence-review; confidence thresholds (0.85 / 0.95 / 0.70) declared; prompt template missing; coherence audit S-013 | PARTIAL |
| Manual obligation entry | YES | YES | YES | YES | PARTIAL — obligation bounded context; manual-entry flow not explicit | PARTIAL |
| Obligation due-date computation | YES (Smart Import calculates) | YES (DocuSign Insight) | YES (Conga AI) | YES | MISSING — due-basis computation grammar not declared; coherence audit S-014 | MISSING |
| Obligation tracker (post-signing) | YES (Repository obligation view) | YES (Obligation Management) | YES (SLA / Obligation Management) | YES | PARTIAL — obligation bounded context + capability obligation-track | PARTIAL |
| Obligation notifications (reminders) | YES (Auto-reminders) | YES (Reminders) | YES (Salesforce reminders) | YES | DELEGATED — calendar µservice expected; not declared in ARCHITECTURE §D | DELEGATED |
| Renewal risk scoring | PARTIAL (Insights renewal view) | YES (DocuSign Insight) | YES (Conga Renewal Management) | YES | PARTIAL — IP-028 renewal-risk-explainability-board; renewal bounded context | PARTIAL |
| Renewal calendar / forecast | YES (Insights) | YES (Reports) | YES (Conga Renewal Management) | YES | DELEGATED — calendar µservice | DELEGATED |
| Contract analytics dashboard (cycle time, deviation rate) | YES (Insights) | YES (DocuSign Insight Analytics) | YES (Salesforce Reports + Dashboards) | YES | PARTIAL — dashboards/ has Grafana skeleton; customer-facing analytics not declared | PARTIAL |
| Counterparty insights (renewal history) | YES (Insights) | YES (DocuSign Insight) | YES (Conga Renewal + Salesforce Reports) | YES | MISSING | MISSING |
| Clause deviation analytics | YES (Insights) | YES (Insight) | YES (Conga AI) | YES | PARTIAL — IP-026 ClauseDeviation aggregate; per-tenant rollup analytics missing | PARTIAL |
| Forecast pipeline value at risk | PARTIAL (Insights) | PARTIAL (Insight) | YES (Conga + Salesforce Forecast) | YES | DELEGATED — financial-planning µservice expected; not declared | DELEGATED |
| AI risk flagging (red-flag clauses) | YES (Jurist AI) | YES (Insight) | YES (Conga AI) | YES | PARTIAL — IP-026 ClauseDeviation high-risk + prohibited categories; AI scoring not specified | PARTIAL |
| Privilege detection (attorney-client privileged content) | NO | NO | NO | partial (gap that none of the three closes) | MISSING — coherence audit L-013 | MISSING (oyatie-additive opportunity) |

Family E summary: 0 PARITY, 6 PARTIAL, 3 MISSING, 3 DELEGATED, 0 NEEDS-DECISION. Active gap = 12/13 (1 row is an oyatie-additive opportunity rather than counterpart-parity).

### §3.6 Family F: Integrations — CRM / ERP / mail / calendar / workflow / marketplace

| Capability | C1 (Ironclad) | C2 (DocuSign CLM) | C3 (Conga CLM) | UNION | Oyatie CLM | Gap class |
|---|---|---|---|---|---|---|
| Salesforce CRM integration | YES (Ironclad for Salesforce) | YES (DocuSign for Salesforce) | YES (Salesforce-native) | YES | NEEDS-DECISION — crm µservice exists; crm ↔ contract-lifecycle-management contract not authored; coherence audit Q-001 | NEEDS-DECISION |
| HubSpot CRM integration | YES (Ironclad for HubSpot) | PARTIAL (via DocuSign for HubSpot) | PARTIAL (via partner) | YES | MISSING | DELEGATED (workplace-integration?) |
| Microsoft Dynamics 365 integration | YES (Ironclad for Dynamics) | YES (DocuSign for Dynamics) | YES (Dynamics integration via Conga partner) | YES | MISSING | DELEGATED |
| SAP integration | YES (Custom integration) | YES (SAP CPI connector) | YES (Conga for SAP) | YES | MISSING | DELEGATED |
| NetSuite integration | YES | YES | PARTIAL | YES | MISSING | DELEGATED |
| Coupa procurement integration | YES (Ironclad for Coupa) | YES | YES (Conga for Coupa) | YES | MISSING | DELEGATED |
| Workday HR integration | PARTIAL (custom) | PARTIAL | YES (Conga for Workday) | YES | MISSING | DELEGATED (workplace-integration) |
| Microsoft 365 (Word/Outlook) integration | YES (Ironclad Editor) | YES (Word add-in + Outlook add-in) | YES (X-Author + Word add-in) | YES | MISSING — drive µservice owns documents; Word/Outlook plug-in not declared | DELEGATED (workplace-integration + drive) |
| Microsoft Teams integration | YES (Slack equivalent) | YES (Microsoft Teams app) | YES (Salesforce-Teams) | YES | MISSING | DELEGATED (workplace-integration) |
| Slack integration | YES (Slack app) | PARTIAL | PARTIAL (via Salesforce) | YES | MISSING | DELEGATED (workplace-integration) |
| Mail / email-to-CLM ingest | YES (Email Capture) | YES (Email-to-CLM) | YES (Salesforce email) | YES | DELEGATED — mail µservice expected | DELEGATED |
| Calendar / reminder integration | YES (Auto-reminders via email) | YES (Reminders) | YES (Salesforce calendar) | YES | DELEGATED — calendar µservice; not declared | DELEGATED |
| Workflow engine (general) | YES (Workflow Designer = Ironclad-native) | YES (Workflow Designer = DocuSign-native) | YES (Salesforce Workflow + Process Builder) | YES | DELEGATED — workflow-engine µservice owns | DELEGATED |
| Marketplace / DealSet settlement | NO | NO | NO | YES (oyatie-additive per ADR-0314) | PARTIAL — manifest.json substrate_dependencies includes marketplace; dealset-contract-bind capability declared; contract not enumerated | PARTIAL |
| Payments integration (commercial obligation settlement) | NO | NO | NO | YES (oyatie-additive) | PARTIAL — manifest substrate_dependencies includes payments; flow not enumerated | PARTIAL |
| Webhooks (workflow + document events) | YES (Webhooks) | YES (Webhooks) | YES (Salesforce platform events) | YES | PARTIAL — AsyncAPI 3.1.0 contract present; per-event surface not enumerated | PARTIAL |
| SSO (SAML / OIDC) + SCIM | YES (SAML + SCIM) | YES (SAML + SCIM) | YES (Salesforce SSO + SCIM) | YES | DELEGATED — identity µservice owns | DELEGATED |
| Public API (REST) | YES (Public API v1) | YES (REST + SOAP) | YES (Salesforce + Conga API) | YES | PARTIAL — OpenAPI 3.2.0 surface present but only 2 endpoints declared; coherence audit Defect I-D7 | PARTIAL |

Family F summary: 0 PARITY, 4 PARTIAL, 0 MISSING explicitly, 11 DELEGATED, 1 NEEDS-DECISION. Active-gap by delegation-coverage = boundary explicit declaration missing in ARCHITECTURE §D; coherence audit Defect X-D5 covers this.

### §3.7 Family G: Mobile + e-discovery + audit trail + reporting

| Capability | C1 (Ironclad) | C2 (DocuSign CLM) | C3 (Conga CLM) | UNION | Oyatie CLM | Gap class |
|---|---|---|---|---|---|---|
| Mobile native iOS app | PARTIAL (web responsive) | YES (DocuSign CLM iOS) | YES (Conga Mobile + Salesforce Mobile) | YES | MISSING — sdk-plan.md silent; coherence audit R-010 | MISSING |
| Mobile native Android app | PARTIAL (web responsive) | YES (DocuSign CLM Android) | YES (Conga Mobile + Salesforce Mobile) | YES | MISSING | MISSING |
| Mobile signing (counterparty signs on mobile) | YES (via DocuSign integration) | YES (DocuSign eSignature mobile) | YES (Conga Sign mobile) | YES | MISSING | MISSING |
| Mobile approval (approver approves on mobile) | YES (web responsive) | YES (DocuSign CLM mobile) | YES (Salesforce Mobile + Conga Mobile) | YES | MISSING | MISSING |
| E-discovery export (legal hold + complete chain of custody) | YES (Repository export with audit) | YES (Bulk export + Audit Log) | YES (Salesforce + Conga export) | YES | PARTIAL — ADR-CLM-001 §"exported contract packets must include audit references and redaction evidence"; e-discovery FRCP-37(e) workflow not declared | PARTIAL |
| Audit trail (every action) | YES (Audit Trail) | YES (DocuSign CLM audit log) | YES (Field History Tracking on Salesforce) | YES | PARTIAL — manifest.json + ADR-CLM-001 mention audit-chain integration; per-aggregate event taxonomy implied; per-field audit trail not declared | PARTIAL |
| Setup audit trail (who changed configuration) | YES | YES (Setup Audit Log) | YES (Salesforce Setup Audit) | YES | MISSING — no setup audit trail primitive | MISSING |
| Field history tracking (per-field changes) | YES (Custom Field history) | YES (Tag history) | YES (Field History Tracking) | YES | PARTIAL — append-only ledger implies field history at row level; per-field not specified | PARTIAL |
| Reports (customer-facing) | YES (Insights Reports) | YES (Reports) | YES (Salesforce Reports) | YES | MISSING — dashboards/ has operational dashboards only | MISSING |
| Dashboards (customer-facing) | YES (Insights Dashboards) | YES (DocuSign Insight Dashboards) | YES (Salesforce Dashboards + Conga Dashboards) | YES | MISSING — coherence audit notes operational vs customer-facing | MISSING |
| Scheduled report delivery | YES (Insights subscriptions) | YES (Reports subscriptions) | YES (Salesforce subscriptions) | YES | MISSING | MISSING |
| Real-time alerts (contract milestone) | YES (Alert + Webhook) | YES (Alerts) | YES (Salesforce alerts) | YES | DELEGATED — calendar + mail | DELEGATED |
| Notifications (email + Slack + Teams) | YES (Email + Slack) | YES (Email) | YES (Salesforce + Conga email + Slack) | YES | DELEGATED — mail + connect | DELEGATED |
| Per-contract activity feed | YES (Activity tab) | YES (Activity tab) | YES (Salesforce Chatter on Agreement) | YES | MISSING | MISSING |
| Per-clause activity feed | YES (Workflow step audit) | YES (Document property history) | YES (Field History) | YES | PARTIAL — RedlineEvent in ADR-CLM-001 is per-clause; surface not declared | PARTIAL |
| SOC-2 Type II compliance | YES (SOC-2 Type II) | YES (SOC-2 Type II) | YES (SOC-2 Type II) | YES | PARTIAL — manifest.json pack includes SOC-2; CLM-side compliance evidence not enumerated | PARTIAL |
| ISO-27001 compliance | YES | YES | YES | YES | PARTIAL — manifest.json pack ISO-27001 | PARTIAL |
| FedRAMP Moderate | NO | YES (DocuSign Federal) | NO | YES | MISSING | MISSING (oyatie-future) |
| HIPAA-eligible deployment | NO | YES (HIPAA BAA) | NO | YES | PARTIAL — manifest.json pack hipaa included; HIPAA-specific BAA contract type missing per L-007 | PARTIAL |
| SEC 17a-4(f) WORM | PARTIAL | YES (WORM Add-On) | PARTIAL | YES | MISSING — coherence audit L-016 | MISSING |

Family G summary: 0 PARITY, 8 PARTIAL, 8 MISSING, 2 DELEGATED, 0 NEEDS-DECISION. Active gap = 18/20 = 90%.

## §4 Roll-up counter-row counts and family summary

| Family | PARITY | PARTIAL | MISSING | DELEGATED | NEEDS-DECISION | Total | Active gap % |
|---|---|---|---|---|---|---|---|
| A. Authoring + template + clause library | 0 | 4 | 4 | 1 | 0 | 9 | 89% |
| B. Negotiation + redlining + approval | 0 | 6 | 1 | 3 | 0 | 10 | 100% |
| C. E-signature + cryptographic evidence | 0 | 5 | 4 | 1 | 3 | 13 | 100% |
| D. Repository + search + retention + WORM | 0 | 6 | 2 | 3 | 1 | 12 | 100% |
| E. AI + obligation + renewal + analytics | 0 | 6 | 3 | 3 | 0 | 12 | 100% |
| F. Integrations | 0 | 4 | 0 | 11 | 1 | 16 | 100% (delegation-boundary missing) |
| G. Mobile + e-discovery + audit + reporting | 0 | 8 | 8 | 2 | 0 | 18 | 100% |
| Aggregate | 0 | 39 | 22 | 24 | 5 | 90 | ~95% |

Counter notes:
- PARITY count: 0 (zero capabilities at functional-equivalence floor).
- PARTIAL count: 39 (capabilities with structural surface present at sub-counterpart depth).
- MISSING count: 22 (capabilities not present in CLM tree at all).
- DELEGATED count: 24 (capabilities owned by another Oyatie µservice — boundary needs declaration).
- NEEDS-DECISION count: 5 (capabilities awaiting Wave 14 ownership decision: TSA owner, HSM owner, online notary, Salesforce-CRM-integration owner, contract repository owner).

UNION-coverage rendering: the audit estimates UNION-coverage at 30-40% across Ironclad + DocuSign CLM + Conga CLM. The remaining 60-70% is the headline gap addressable in Wave 14-15 remediation. The CLM µservice has structural backbone but lacks legal-domain depth (clause taxonomy, contract type taxonomy, signature evidence model, TSA binding, counterparty MDM) and counterpart-specific features (Smart Sections, Word Online co-edit, Bulk Send, mobile native).

## §5 Headline gap analysis (Top-25 priority gaps for Wave 14-15 remediation)

These gaps are ordered by P0 LEGAL-COMPLEXITY severity plus active-gap weight plus union-coverage criticality.

G-001 (P0 LEGAL-COMPLEXITY): **Canonical clause taxonomy + library inheritance**. All three counterparts have clause library as primary differentiator. CLM has clause-library bounded context but no taxonomy. Resolution: author clause-family-taxonomy.md covering Term & Termination, Indemnification, Limitation of Liability, Confidentiality, Data Protection, SLA, Payment Terms, Assignment, Governing Law, Dispute Resolution, Insurance, Force Majeure, IP Ownership, Warranty, Audit Rights, Survival, MFN, FCPA/UKBA anti-corruption.

G-002 (P0 LEGAL-COMPLEXITY): **Canonical contract type taxonomy**. All three counterparts have contract type. CLM has none. Resolution: author contract-type-taxonomy.md covering MSA, SOW, NDA (uni/mutual/perpetual), DPA, BAA (HIPAA), SaaS Subscription, Reseller, License, Settlement, Employment, IP Assignment, M&A SPA, Real Estate Lease, Government, Procurement PO, Vendor.

G-003 (P0 LEGAL-COMPLEXITY): **Signature envelope canonical model**. All three counterparts have detailed e-signature surfaces. CLM has IP-030 e-signature-provider-portability but no envelope model. Resolution: author signature-envelope-canonical.md covering SES/AES/QES distinction + PKCS#7/CMS/CAdES/PAdES/XAdES envelope choice per jurisdiction + hash algorithm choice (SHA-256/SHA-3/BLAKE3) + signer certificate path + timestamp inclusion.

G-004 (P0 LEGAL-COMPLEXITY): **eIDAS QES Trust List binding**. DocuSign CLM is the only counterpart with native QES. CLM mentions QES at capability-tiers/tier-matrix.md but no binding. Resolution: author qes-trust-list-binding.md covering LOTL / TSL ingestion, certificate-path validation, qualified-status check at signing time, per-EU-state TSP integration (e.g., D-Trust Germany, GlobalSign EU, Trustpro Italy, SwissSign Switzerland).

G-005 (P0 LEGAL-COMPLEXITY): **Counterparty Master Data Management**. All three counterparts treat counterparty as first-class. CLM has no MDM. Resolution: author counterparty-mdm.md covering legal-entity resolution (parent/subsidiary/merger-acquired/dissolved/name-changed), counterparty-as-aggregate or counterparty-as-projection-of-crm-account decision (NEEDS-DECISION Q-008).

G-006 (P0 LEGAL-COMPLEXITY): **CPQ-CLM bridge with crm + cloud-billing**. Conga has the strongest CPQ-CLM. CLM has no quote-to-contract bridge. Resolution: declare crm.quote → contract-lifecycle-management.contract-intake contract; reuse CRM audit Q-003 CPQ ownership decision.

G-007 (P0 LEGAL-COMPLEXITY): **AI clause-suggestion + redlining prompt template**. All three counterparts have AI; CLM has tier-named LLM mentions (Llama-3.1 / Claude cross-emit). Resolution: author AI prompt templates for clause-suggestion + risk-flagging + obligation-extraction; bind to intelligence µservice handoff.

G-008 (P0 LEGAL-COMPLEXITY): **Smart Sections / conditional document assembly**. DocuSign CLM and Conga have it. CLM has none. Resolution: author conditional-document-assembly.md.

G-009 (P0 LEGAL-COMPLEXITY): **Bulk Send (one contract → many counterparties)**. All three counterparts have it. CLM has none. Resolution: add bulk-send capability.

G-010 (P0 LEGAL-COMPLEXITY): **Mobile native (Swift iOS + Kotlin Android)**. DocuSign CLM and Conga have native mobile apps. CLM sdk-plan.md silent. Resolution: author mobile signing + approval app spec.

G-011 (P0 LEGAL-COMPLEXITY): **Customer-facing analytics dashboard + reports**. All three counterparts have customer-facing Reports + Dashboards. CLM has only operational Grafana dashboards. Resolution: author customer-facing analytics primitive.

G-012 (P0 LEGAL-COMPLEXITY): **Ironclad SObject-equivalent migration mapping**. Migration playbook from-ironclad.md is missing. Resolution: author from-ironclad.md with Workflow/Document/Approval/Field/Schema/Record/Repository → Oyatie aggregate mapping.

G-013 (P0 LEGAL-COMPLEXITY): **Conga CLM migration mapping**. Migration playbook from-conga-clm.md is missing. Resolution: author from-conga-clm.md with Agreement/Clause/MSA/OrderForm/Schedule → Oyatie aggregate mapping.

G-014 (P0 LEGAL-COMPLEXITY): **Notice-and-cure clause + obligation suspension**. Standard commercial contract pattern. CLM has IP-027 obligation extraction but no notice-and-cure model. Resolution: include in obligation taxonomy.

G-015 (P0 LEGAL-COMPLEXITY): **Legal hold state machine + Cedar gate**. All three counterparts have Legal Hold flag. CLM has runbook but no state model. Resolution: author legal-hold state machine in PRD §B + per-aggregate Cedar policy "if legal_hold_active AND action = delete then deny".

G-016 (P0 LEGAL-COMPLEXITY): **WORM storage binding**. DocuSign CLM has WORM as primary feature for broker-dealer. CLM mentions SeaweedFS Compliance at retired-standard tier. Resolution: bind WORM binding model in PRD §H or §L with deployment-context overlay (S3 Object Lock Compliance for aws-guest; OCI Vault Compliance for oci-guest; SeaweedFS Compliance for on-prem; OCI Object Storage Compliance for oci demo_trial).

G-017 (P0 LEGAL-COMPLEXITY): **Field history tracking (per-field changes)**. All three counterparts have it. CLM has row-level audit-chain but not per-field. Resolution: author per-field history in ADR-CLM-001 v2.

G-018 (P0 LEGAL-COMPLEXITY): **Word Online native editing**. All three counterparts have it. CLM has none. Resolution: declare drive µservice + workplace-integration µservice handoff for Word Online integration.

G-019 (P0 LEGAL-COMPLEXITY): **Approval matrix declarative model**. All three counterparts have it. CLM delegates to workflow-engine but the declarative model is not authored. Resolution: author approval-routing-matrix.md.

G-020 (P0 LEGAL-COMPLEXITY): **Multi-language contract overlay**. Conga has multi-language; CLM has none. Resolution: author multi-language-contract-overlay.md.

G-021 (P0 LEGAL-COMPLEXITY): **HIPAA BAA contract type**. DocuSign CLM has HIPAA-eligible deployment with BAA. CLM manifest.json includes hipaa pack but no BAA contract type. Resolution: author baa-contract-type-overlay.md.

G-022 (P0 LEGAL-COMPLEXITY): **GDPR Article 7 consent records**. None of the three counterparts has explicit Article-7 evidence; CLM should be additive. Resolution: author IP-031-gdpr-article-7-consent-records.md.

G-023 (P0 LEGAL-COMPLEXITY): **ESIGN consumer disclosure flow**. All three counterparts comply with ESIGN by default but the flow is not explicit. CLM has none. Resolution: author esign-consumer-disclosure-flow.md.

G-024 (P0 LEGAL-COMPLEXITY): **Privilege tagging (attorney-client privilege)**. None of the three counterparts has it explicitly; CLM should be additive. Resolution: author privilege-tagging-overlay.md.

G-025 (P0 LEGAL-COMPLEXITY): **Per-deployment-context sovereign-pack matrix**. CLM-specific gap covering KR-PIPA (Seoul-only), EU eIDAS QES (Frankfurt/Paris/Dublin), HIPAA (US-only), CSAP (Korean public sector), SEC 17a-4(f) (WORM-on-broker-dealer). Resolution: author deployment_context × sovereign_pack matrix.

## §6 Additive surface (capabilities Oyatie CLM has that counterparts lack)

A-001: **Marketplace DealSet settlement binding** (ADR-0314). None of Ironclad / DocuSign CLM / Conga CLM has first-class marketplace settlement integration. Oyatie's dealset-contract-bind capability is additive.

A-002: **Append-only clause/redline/obligation ledger** (ADR-CLM-001 ContractObligationLedger v1). Counterparts have version history but not first-class append-only ledger with compensating-event correction model. Oyatie's ledger is additive on legal-evidence rigor.

A-003: **Cedar default-deny policy per aggregate** (ADR-0243). All three counterparts use platform-native authorization (Salesforce Sharing Rules, DocuSign permissions, Ironclad roles). Oyatie's Cedar-as-universal-gate is more rigorous.

A-004: **Ontology projection per aggregate** via Oyatie ontology µservice. Counterparts have no first-class ontology. Oyatie's tenant-scoped ontology projection is additive for legal-entity / clause-network / obligation-graph queries.

A-005: **HTTP/3 + QUIC default transport** per ADR-0253. Counterparts default to HTTP/1.1 or HTTP/2.

A-006: **Post-quantum cryptography hybrid negotiation** (X25519MLKEM768) per CLM OpenAPI x-transport. Counterparts have classical TLS 1.3 only.

A-007: **Compliance pack overlay model** (SOC-2, ISO-27001, GDPR, SOX-404, eIDAS, ESIGN, KR-PIPA, HIPAA per manifest.json compliance_packs). Counterparts have license-tier compliance add-ons. Oyatie's per-tenant pack composition is additive.

A-008: **Tenant-class binary** (demo_trial + paid) with paid billing_components {revenue_share, per_seat, per_usage} — PENDING ADOPTION per coherence audit C-001..C-012 but the model is additive vs counterparts' tier-based licensing.

A-009: **Workflow-engine + Ontology as cross-µservice adapter** per ADR-0145 direct gRPC. Counterparts use platform-native automation only (Ironclad Workflow Designer, DocuSign Workflow Designer, Salesforce Workflow).

A-010: **OCI Always Free demo_trial** at $0 cost. Counterparts have time-limited free trials only.

A-011: **Privilege tagging** (proposed). None of the three counterparts has explicit attorney-client privilege tagging. Oyatie can lead with this.

A-012: **OpenTofu zero-handroll IaC** for tenant onboarding. Counterparts have SaaS-only delivery; Oyatie's `tofu apply -var tenant_id=acme-legal -var jurisdiction_pack=eu-eidas-qes` is unique.

A-013: **Sovereign-pack residency with in-pack QES HSM** (KR-PIPA + CSAP + EU eIDAS QES + HIPAA-Provider). Per capability-tiers/tier-matrix.md retired-sovereign — this is post-tier-retirement renamed to "paid + sovereign-pack overlay" but the substantive capability is unique. Counterparts have limited sovereign-residency (Icertis ICI on-prem; DocuSign Federal for FedRAMP); none cover the full sovereign-pack × QES × HSM combination.

A-014: **Multi-category marketplace template distribution** per ADR-0249. None of the counterparts has a peer-tenant marketplace for contract templates.

## §7 Wave-14 aggregation prompts

This matrix should be aggregated with the other Legal-Complexity µservice parity matrices (healthcare-integration for HIPAA-grade legal documents; governance for cross-tenant policy; identity for KYC-grade signing) to produce a unified Legal-Complexity capability registry. Aggregation questions:

W-001: What capabilities are universal across Legal-Complexity (e.g., audit-trail-with-cryptographic-evidence, retention-per-pack, legal-hold)? These should be substrate-level primitives shared with audit-chain µservice rather than duplicated per Legal-Complexity µservice.

W-002: What capabilities are CLM-distinctive (e.g., clause-library, redline-tracking, signature-envelope)? These stay in CLM.

W-003: What capabilities cross multiple Legal-Complexity + B2B-leader µservices (e.g., CPQ-to-Contract crosses crm + cloud-billing-tax + payments + contract-lifecycle-management)? These need cross-µservice journey docs at the Wave-14 aggregation layer.

W-004: What is the canonical Legal-Complexity comparator-set registry? CLM has Ironclad/DocuSign CLM/Conga; healthcare-integration has Epic/Cerner/Allscripts; governance has OneTrust/TrustArc/BigID; identity has Okta/Auth0/Microsoft Entra. Wave 14 should produce one registry.

W-005: How is the OCI Always Free profile decomposed across the Legal-Complexity µservices so demo_trial tenants can run CLM + governance + identity together within 4 OCPU + 24 GB? CLM should target ~1 OCPU + 4 GB + 50 GB block + 1× 20 GB Autonomous DB + 25 GB egress (the same envelope as in the performance-benchmark-numbers companion deliverable).

W-006: What is the canonical e-signature provider boundary? CLM owns the contract evidence; the e-signature provider integration could be CLM-internal (current path) or a separate e-signature-provider µservice. Per ADR-0132 no-grouping policy, if e-signature merits a single-concern µservice, Wave 14 should split it out.

W-007: What is the canonical HSM/QES key custody boundary? CLM mentions Thales Luna 7 A790; kms µservice is the natural HSM owner. Wave 14 must declare CLM ↔ kms contract for QES HSM.

W-008: What is the canonical TSA (Time-Stamp Authority) integration boundary? Per RFC 3161 + ETSI EN 319 422. CLM mentions KISA / DSS Trust List TSAs. Wave 14 must declare CLM ↔ kms or CLM ↔ tsa contract.
