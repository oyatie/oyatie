---
doc_class: DPIA
template_id: TPL-DPIA
microservice: docs
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-docs
methodology: ICO DPIA + CNIL DPIA + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145), ADR-DOCS-0001, ADR-DOCS-0003, ADR-DOCS-0004, ADR-DOCS-0005, ADR-DOCS-0006]
related_artifacts:
  - microservices/docs/threat-model.md
  - microservices/docs/policy/document-isolation.md (rendered name; see policy/editor-isolation.md)
  - microservices/docs/policy/data-residency.md
  - microservices/docs/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (AI writing-assist + auto-summary profile authoring patterns)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES conditional (PHI in clinical-notes via pack-us-healthcare; sensitive PIPA Art. 23 via legal/clinical document content)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
doc_status: published
---

# Data Protection Impact Assessment: docs µservice

## Step 1 — Need for a DPIA

Docs processes document content (text + rich structure), author + commenter identities, suggestion authorship + state, share recipients, and cross-µservice embeds. Two of three Art. 35(3) automatic triggers engaged:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a) Systematic + extensive evaluation | YES | AI writing-assist + auto-summary + grammar-check continuously profiles author content; cross-tenant search would be quasi-profiling. |
| Art. 35(3)(b) Large-scale special-category | YES (conditional) | Clinical-notes (pack-us-healthcare) carry PHI; legal documents may contain PIPA Art. 23 categories. |
| Art. 35(3)(c) Public-area monitoring | NO | — |

Also: PIPC Notice 2020-7 (KR) mandates DPIA for sensitive personal information at scale → engaged for pack-kr; APPI voluntary scheme followed for pack-jp.

DPIA mandatory pre-deployment.

## Step 2 — Describe the processing

### 2.1 Nature

**What:** Document authoring, real-time multi-user collaboration via CRDT, commenting + suggestions, version history, sharing + per-block ACL, document import/export (DOCX/Markdown/HTML/PDF/EPUB/LaTeX), cross-µservice embedding, attachment storage, AI writing-assist (T1/T2 capabilities).

**How:** REST + WebSocket ingress → Postgres metadata store (per-tenant RLS + tenant-DEK envelope) → S3 content blobs (per-tenant prefix; Object Lock for legal-hold) → Valkey collab-presence + CRDT op spool → gVisor sandbox for export pipeline → Workflow events to mail (share-via-email) + audit-chain (seal emission) + messenger (mention) + observability (telemetry).

**Where:** Per-pack region-pinned Postgres + S3 + Valkey (pack-kr → KR; pack-eu → EU; pack-us → US; pack-us-healthcare → BAA-eligible US; pack-jp → JP; etc.). Residency enforced via ADR-0117 + ADR-0140.

**When:** Continuous; on-demand for user actions; recurring background sweeps for retention + version compaction + embed-refresh.

**Who:** Per the actor table in `threat-model.md` §"Actors".

### 2.2 Scope

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `PROFESSIONAL_DOC_CONTENT` | strategy docs, design docs, contracts | Art. 6(1)(b) contract + 6(1)(f) legitimate interest | 10⁵ docs/day per medium tenant |
| `PERSONAL_DOC_CONTENT` | personal notes, drafts | Art. 6(1)(a) consent + 6(1)(b) | 10⁴/day per active user |
| `PII_IDENTIFYING` | author / commenter / suggestion identities | Art. 6(1)(b) | 1 per document + 5 per comment thread |
| `PII_QUASI_IDENTIFIER` | document titles, attachment file names | Art. 6(1)(f) | varies |
| `SENSITIVE_PIPA_ART23` | clinical/legal/political content when tagged sensitive | KR PIPA Art. 23(2) explicit consent | per flagged doc |
| `PHI` (pack-us-healthcare only) | clinical-notes content under BAA | HIPAA §164.502(a) Permitted Uses | per BAA tenant |
| `AUDIT` | document lifecycle records | Art. 6(1)(c) legal obligation | 1 per mutation |
| `SECRET` | tenant-DEK, share-link tokens | not personal data | managed via OpenBao |

**Geographical scope:** per pack.

**Cross-border transfer:** forbidden by default; allowed with tenant-executed SCCs per Arts. 44–46 per `multi-region.md`.

### 2.3 Context

- **Data subjects:** end-users (the tenant's employees, contractors, collaborators); tenant operators; external share-link recipients; mentioned principals; oyatie operators (internal).
- **Relationship:** joint controllership with tenant (GDPR Art. 26) for end-user document data; oyatie sole processor for operational metadata.
- **Reasonable expectations:** authors expect operational doc storage; co-editors expect collab + comment per tenant privacy notice; share-link recipients expect bounded access per token scope.
- **Previous experience:** Bominal Connect Docs inheritance per ADR-0208; no DPA-triggered complaints in inheritance period.

### 2.4 Purposes

| Purpose | Necessity | Lawful basis |
|---|---|---|
| Document authoring + storage | Contracted | Art. 6(1)(b) |
| Real-time collaboration | Contracted | Art. 6(1)(b) |
| Commenting + suggestion | Contracted | Art. 6(1)(b) |
| Version history + revert | Operational + Art. 30 records | Art. 6(1)(c) |
| Sharing + per-block ACL | Contracted | Art. 6(1)(b) |
| Export (DOCX / PDF / etc.) | Portability Art. 20 + contracted | Art. 6(1)(b) + 6(1)(c) |
| Import (DOCX / Markdown) | Migration / portability | Art. 6(1)(b) |
| Cross-µservice embed (workflow-studio / sheets / slides) | Composition | Art. 6(1)(b) |
| AI writing-assist (T1) | User-invoked | Art. 6(1)(a) explicit consent at invocation |
| Auto-summary / auto-translate (T2) | Tenant-policy-bound | Art. 6(1)(f) + tenant-class admission + per-user opt-in |
| Audit-chain emission | Records of processing (Art. 30) | Art. 6(1)(c) |
| Legal-hold preservation | Legal obligation | Art. 6(1)(c) |
| Marketing / unrelated commercial use | NOT a purpose | N/A |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | Sign-off pending; see §7 |
| Sample of prospective tenants | Scheduled pre-GA | Feedback folded into §6 |
| End-users (indirect via tenant) | Joint-controllership clause | Tenant disclosure obligation |
| Supervisory authority (DPA / PIPC) | Art. 36 NOT triggered (no H residual after mitigations) | — |
| Information security (ops-security) | YES | Shared residual catalog with threat-model |
| Engineering (axis-docs + consuming µservices) | YES | LEAN gates enforced |
| External auditor | At first audit cycle | Cross-references DPIA |

## Step 4 — Necessity and proportionality

| Question | Assessment |
|---|---|
| Necessary? | YES — first-party docs cannot occur without content. |
| Less-intrusive alternative? | Considered: links-only (no in-platform content). Rejected: defeats integration + collab value. |
| Proportionate? | YES — per-block ACL minimises exposure at the field level; cross-tenant embed limited to source-side ACL passthrough; data-class annotations enforce minimum-necessary. |
| Anonymisation possible? | Partial — aggregate statistics (doc count / edit count) anonymised; full anonymisation incompatible with the doc purpose. |
| Lawful basis | Per §2.4 |
| Special-category (Art. 9) | pack-us-healthcare: Art. 9(2)(h) (health care provision) + HIPAA BAA. pack-kr Art. 23: PIPA Art. 23(2) explicit consent at flagged-doc level. |
| Transfer basis | SCCs only; default residency by pack. |
| Retention | per doc/jurisdiction; HIPAA pack ≥ 6y; default 24mo + per-tenant policy override. |
| Subject rights | Art. 15/16/17/18/20/21/22 honoured per §6. |

## Step 5 — Risks to data subjects

| ID | Risk | L | S | Score |
|---|---|---|---|---|
| R-01 | Personal-context doc content leaks into Professional-context query | M-H | H | **H** |
| R-02 | Per-block ACL bypass leaks private block to non-authorized principal | M | H | **H** |
| R-03 | Share-link enumeration / impersonation | M | M | **M** |
| R-04 | Export pipeline leaks plaintext via temp-file / stderr | L | H | **M** |
| R-05 | Attachment upload contains malware / steganography | M | M | **M** |
| R-06 | Embed-resolver leaks source content past source-side ACL | M | M | **M** |
| R-07 | Long retention enables surveillance pattern across years | M | M-H | **M-H** |
| R-08 | DSR right-to-erasure incomplete due to cross-doc references + legal-hold overlap | M | M | **M** |
| R-09 | Joint-controllership: tenant doesn't disclose oyatie's processing to end-users | M-H | M | **M-H** |
| R-10 | PHI processed without BAA (pack-us-healthcare tenant doesn't sign BAA but ships clinical notes) | M | H | **H** |
| R-11 | Sub-processor breach (Postgres cluster operator / cloud provider) | L | H | **M** |
| R-12 | Cross-border transfer of EU-resident doc data via mis-routed embed | L | H | **M** |
| R-13 | Children's-data authoring (school / family tenant) without parental consent | L | H | **M-H** |
| R-14 | Tenant-DEK leaked via log → mass decryption | L | H | **M** |
| R-15 | Auditor mis-pivot across tenants | L | H | **M** |
| R-16 | AI writing-assist (T1/T2) prompts leak document content into model provider | M | H | **H** |
| R-17 | AI writing-assist applied to HR-context document without conformity assessment (EU AI Act) | L-M | H | **M-H** |
| R-18 | OOXML import preserves embedded macros that exfiltrate on export | L | H | **M** |

Cross-reference: every risk has at least one corresponding STRIDE / LINDDUN threat in `threat-model.md`.

## Step 6 — Risk-reducing measures

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 | Rust type-level Personal vs Professional separation; Cedar `document-isolation.cedar`; LEAN check `oya-check-context-isolation` | L | axis-docs |
| R-02 | Cedar `per-block-acl.cedar` enforced at every read; Postgres block-level RLS; annual pen-test | L | ops-security |
| R-03 | Constant-time share-link verify + rate limit + Ed25519-signed token (per ADR-DOCS-0004) | M (timing baseline) | ops-security |
| R-04 | gVisor sandbox with tmpfs only; stderr scrubbed; per-job ephemeral pod | L | ops-security |
| R-05 | ClamAV (default) / OPSWAT (us-healthcare) scan; per-extension allowlist; image re-encode strips EXIF + LSB | M (steganography) | axis-docs + ops-security |
| R-06 | Embed-resolver re-evaluates source-side ACL at every fetch; LEAN check `oya-check-embed-resolver-acl-passthrough` | L | axis-docs |
| R-07 | Aggressive retention defaults; DSR cascade; cold-tier per-doc access requires admin JIT | L-M | council-privacy |
| R-08 | DSR cascade with cross-doc identifier search + legal-hold overlap policy: erasure honoured except where hold; partial-erasure where compliant | M (hold-vs-erasure tension accepted) | council-privacy |
| R-09 | Tenant DPA mandates upstream disclosure; tenant-onboarding checklist verifies | L-M | council-privacy |
| R-10 | pack-us-healthcare onboarding requires BAA pre-ingest; non-signed tenants pre-flighted to non-PHI pack | L | council-privacy |
| R-11 | Sub-processor list at `legal/sub-processors.md`; DPA + SCCs per sub-processor; quarterly review | M (sub-processor risk irreducible) | council-privacy |
| R-12 | Pack-pinning at ingress; embed-resolver refuses cross-pack source; LEAN check | L | axis-docs |
| R-13 | Tenant DPA includes child-data clause; tenant affirms parental-consent process; docs does not collect age | L | council-privacy |
| R-14 | Secret-scanner CI lane; `Secret<T>` type strips Debug; 90d rotation; rotation event re-encrypts | M (human-error baseline) | ops-security |
| R-15 | Auditor JIT tokens tenant-scoped at row level; pen-test annually | L | ops-security |
| R-16 | Tenant-DEK-wrapped prompts; no cross-tenant training; per ADR-DOCS-0005 prompt-isolation requirements | M (model-provider trust baseline) | foundry-runtime + council-privacy |
| R-17 | EU AI Act Annex III §3 conformity assessment per ADR-DOCS-0005; HR-context REFUSED at Cedar layer in pack-eu until per-tenant assessment evidence on file | L-M | council-privacy + axis-docs |
| R-18 | OOXML import refuses VBA + ActiveX macros at parser; per ADR-DOCS-0006 | L | axis-docs |

## Step 7 — Sign-off

| Sign-off | Status |
|---|---|
| DPO (council-privacy) | `pending` |
| Information Security Officer (ops-security) | `pending` |
| µservice owner (axis-docs) | `pending` |
| Council-architecture | `pending` |

**DPO advice:** Residual risks all L or M after mitigations. Art. 36 prior consultation NOT triggered. Proceed with first-tenant onboarding subject to:
- Quarterly review of R-08 (DSR vs hold tension) and R-16/R-17 (AI assist scope).
- Annual review of this DPIA.
- Re-trigger on each pack activation.
- Pack-eu HR-context T1/T2 stays REFUSED at Cedar until ADR-DOCS-0005-cited per-tenant conformity assessment landing.

## Per-Pack Overlays

### pack-kr (KR PIPA + ISMS-P)

- **Supervisory authority**: 개인정보보호위원회 (PIPC).
- **Legal basis**: KR PIPA Art. 15(1)(1) consent-based for personal-pillar; Art. 15(1)(4) contract-performance for professional-pillar.
- **Cross-border**: KR PIPA Art. 17 SCC-equivalent gating; per-pack residency at ap-seoul-1.
- **Special-category**: KR PIPA Art. 23 SENSITIVE_PIPA_ART23 data-class for legal / clinical / political document content; cross-tenant disclosure refused by default.
- **Residual risk**: Low for default operation; Medium for tenant-opted-in cross-tenant share; mitigated by Cedar audit-chain.
- **DPO notification**: required for any new processing purpose.

### pack-eu (EDPB + national DPA oversight + EU AI Act)

- **Supervisory authority**: lead DPA per GDPR Art. 56 one-stop-shop.
- **Legal basis**: GDPR Art. 6(1)(b) contract-performance for professional docs; Art. 6(1)(a) explicit consent for AI writing-assist; Art. 6(1)(f) legitimate interest for service operation.
- **Cross-border**: GDPR Chapter V — SCC + supplementary measures for any cross-pack transfer; default = no cross-pack.
- **EU AI Act**: Annex III §3 employment-context REFUSED at Cedar layer for T1/T2 HR-overlays pending ADR-DOCS-0005 conformity assessment.
- **DPIA Art. 35 trigger**: AI writing-assist + auto-summary continuous profiling → trigger; this DPIA satisfies.
- **eIDAS PAdES**: legal-evidence PDF exports may carry advanced electronic signature per pack-eu overlay.
- **Residual risk**: Low-Medium; HR-context refusal stands until per-tenant conformity evidence.

### pack-us (FTC + state-AG oversight)

- **Supervisory authority**: FTC + state attorneys-general (CA / VA / CO / CT / UT) per state privacy laws.
- **Legal basis**: contract; sectoral laws apply per tenant residency.
- **Cross-border**: SCC for EU-tenant cross-pack.
- **Residual risk**: Low for default; Medium for cross-state cross-tenant share.

### pack-us-healthcare (HHS OCR + HIPAA)

- **Supervisory authority**: HHS Office for Civil Rights.
- **Legal basis**: HIPAA Privacy Rule; BAA in place for every tenant.
- **Cross-border**: forbidden by default; ePHI must remain in HIPAA-eligible US zones.
- **Special-category**: PHI data-class on every clinical-note field; minimum-necessary per HIPAA 45 CFR §164.502(b).
- **DPIA Art. 35 analog**: HIPAA Security Rule Risk Analysis per 45 CFR §164.308(a)(1)(ii)(A) — this DPIA satisfies the equivalent.
- **Attachment scan**: OPSWAT MetaDefender (HIPAA-compliance bar above ClamAV default).
- **Residual risk**: Medium; mitigated by per-BAA audit; pen-test annual scope includes healthcare-specific PHI-disclosure simulation.

### pack-jp (PPC + APPI)

- **Supervisory authority**: 個人情報保護委員会 (PPC).
- **Legal basis**: APPI Art. 18 consent for cross-tenant; Art. 17 specified-purpose.
- **Cross-border**: APPI Art. 24 — restricted to "adequate" countries.
- **Residual risk**: Low.

### pack-sg (PDPC + PDPA)

- **Supervisory authority**: Personal Data Protection Commission.
- **Legal basis**: PDPA Section 13 consent; deemed-consent for service operation per Section 15.
- **Cross-border**: PDPA Section 26 with comparable protection.
- **Residual risk**: Low.

### pack-au (OAIC + Privacy Act 1988)

- **Supervisory authority**: Office of the Australian Information Commissioner.
- **Legal basis**: Privacy Act 1988 APP 3 collection limitation; APP 5 notification.
- **Cross-border**: APP 8 accountability for cross-pack transfer.
- **Residual risk**: Low.

### pack-in (DPDPA 2023)

- **Supervisory authority**: Data Protection Board of India (DPBI; once constituted).
- **Legal basis**: DPDPA §6 consent; §7 legitimate-uses.
- **Cross-border**: DPDPA §16 whitelist-based.
- **Residual risk**: Low-Medium.

### pack-br (ANPD + LGPD)

- **Supervisory authority**: Autoridade Nacional de Proteção de Dados.
- **Legal basis**: LGPD Art. 7(I) consent; Art. 7(V) contract.
- **Cross-border**: LGPD Art. 33 — ANPD-approved transfer mechanism.
- **Residual risk**: Low.

### pack-ae (UAE Data Office + PDPL)

- **Supervisory authority**: UAE Data Office.
- **Legal basis**: PDPL Art. 5 consent; contract.
- **Cross-border**: PDPL Art. 22 — UAE DPA approval required.
- **Residual risk**: Low.

### pack-ksa (SDAIA + KSA PDPL)

- **Supervisory authority**: Saudi Data and AI Authority.
- **Legal basis**: PDPL Art. 6 lawful processing; Art. 7 consent.
- **Cross-border**: PDPL Art. 29 — SDAIA-approved mechanism.
- **Sharia retention**: per-tenant retention extension supported; refusal of premature deletion logged in audit-chain.
- **Residual risk**: Low-Medium.

## Re-review Triggers

- Annually (Q2).
- On every new pack activation.
- Change to processing purpose (§2.4) or data-class taxonomy.
- Change to AI writing-assist (T1/T2) scope per ADR-DOCS-0005.
- Sub-processor change.
- Breach notification triggered.
- Supervisory-authority guidance change.
- Post-incident (Sev-1 or Sev-2).

## References

- ADR-0028 (Bominal), ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140.
- ADR-DOCS-0001 through ADR-DOCS-0006.
- `microservices/docs/threat-model.md`, `compliance.md`, `policy/*`, `multi-region.md`, `incident-response.md`, `legal/{dpa-template,baa-template,sub-processors,transfer-register,ropa}.md`.
- ICO DPIA template; CNIL DPIA methodology; EDPB Guidelines 4/2019 + 9/2022; PIPC Notice 2020-7.
- GDPR Art. 35 + Art. 36; KR PIPA Art. 33; HIPAA 45 CFR §164.308.
- EU AI Act Regulation (EU) 2024/1689 Annex III §3.
- LGPD Art. 38; DPDPA 2023 §10–§11; UAE PDPL; KSA PDPL.
