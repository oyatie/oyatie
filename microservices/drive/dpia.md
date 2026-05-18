---
doc_class: DPIA
template_id: TPL-DPIA
microservice: drive
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-drive
methodology: ICO DPIA + CNIL DPIA + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0140, ADR-DRIVE-0001, ADR-DRIVE-0002, ADR-DRIVE-0003, ADR-DRIVE-0004, ADR-DRIVE-0005, ADR-DRIVE-0006]
related_artifacts:
  - microservices/drive/threat-model.md
  - microservices/drive/policy/dual-context-isolation.md
  - microservices/drive/policy/data-residency.md
  - microservices/drive/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (auto-tag T1; smart-search T1)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES conditional (PHI in clinical files via pack-us-healthcare; sensitive Art. 23 PIPA via tagged docs)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
doc_status: published
---

# Data Protection Impact Assessment: drive µservice

## Step 1 — Need for a DPIA

Drive stores per-file bytes, file metadata (filename, path, mime, version chain), permission ACLs, share-link records, sync session manifests, and search index extracts. Two of three Art. 35(3) automatic triggers engaged:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a) Systematic + extensive evaluation | YES | T1 auto-tag + T1 smart-search continuously profile file content; T2 auto-organize defers under EU AI Act assessment. |
| Art. 35(3)(b) Large-scale special-category | YES (conditional) | Clinical-files (pack-us-healthcare) carry PHI; file content may contain PIPA Art. 23 categories. |
| Art. 35(3)(c) Public-area monitoring | NO | — |

Also: PIPC Notice 2020-7 (KR) mandates DPIA when sensitive personal information at scale → engaged for pack-kr; APPI voluntary scheme followed for pack-jp.

DPIA mandatory pre-deployment. Reviewed by EU DPAs (Art. 35) and KR PIPC (Art. 33) at first-tenant onboarding per jurisdiction.

## Step 2 — Describe the processing

### 2.1 Nature

**What:** File upload (multipart resumable), download (range), folder organisation, share-link issuance, permissions, sync delta (FastCDC + LBFS), full-text search (Tika + Meilisearch), preview (image/PDF/Office/video), virus scan, DLP scan, retention/WORM tier, legal-hold, third-party-app OAuth.

**How:** REST + S3-compatible + WebDAV + tus ingress → Postgres metadata (per-tenant RLS + tenant-DEK envelope) → object store (Garage / MinIO / SeaweedFS; per-tenant prefix; tenant-DEK-wrapped bytes) → Redis upload-session + delta cache → Meilisearch full-text → Tika extract → Workflow events to mail (attachment-bridge) + messenger (file-share embed) + audit-chain (seal) + observability + foundry-runtime (OCR/auto-tag).

**Where:** Per-pack region-pinned cluster (pack-kr → KR; pack-eu → EU; pack-us → US; pack-us-healthcare → BAA-eligible US; pack-jp → JP; etc.). Residency enforced via ADR-0117 + ADR-0140.

**When:** Continuous; on-demand for user actions; recurring background sweeps for retention + scan + WORM-floor + preview-cache pruning.

**Who:** Per the actor table in `threat-model.md` §"Actors".

### 2.2 Scope

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `PROFESSIONAL_FILE_CONTENT` | business documents, spreadsheets, presentations, design assets, source files | Art. 6(1)(b) contract + 6(1)(f) legitimate interest | 10⁶ files/day per medium tenant |
| `PERSONAL_FILE_CONTENT` | personal documents, photos, backups | Art. 6(1)(a) consent + 6(1)(b) | 10⁵/day per active user |
| `PII_IDENTIFYING` | filenames containing names, ownership ACL principals, share-link recipients | Art. 6(1)(b) contract | bound to file count |
| `PII_QUASI_IDENTIFIER` | folder paths, modification timestamps, IP in download logs | Art. 6(1)(f) legitimate interest | bound to file count |
| `SENSITIVE_PIPA_ART23` | files containing health/political/etc. content (tenant-flagged) | KR PIPA Art. 23(2) explicit consent | per flagged file |
| `PHI` (pack-us-healthcare only) | clinical files under BAA | HIPAA §164.502(a) Permitted Uses | per BAA tenant |
| `AUDIT` | file lifecycle + access + share + scan + immutability records | Art. 6(1)(c) legal obligation | 1 per file mutation + 1 per access |
| `SECRET` | tenant-DEK, share-link signing keys, API keys | not personal data | managed via OpenBao |

**Geographical scope:** per pack (per §2.1).

**Cross-border transfer:** forbidden by default; allowed with tenant-executed SCCs per Arts. 44–46 per `multi-region.md`.

### 2.3 Context

- **Data subjects:** end-users (the tenant's employees + invitees + customers); tenant operators; share-link viewers (external); oyatie operators (internal).
- **Relationship:** joint controllership with tenant (GDPR Art. 26) for end-user file data; oyatie sole processor for operational metadata.
- **Reasonable expectations:** tenant operators expect operational file storage; end-users expect storage per tenant's privacy notice; share-link viewers expect read-only scope.
- **Previous experience:** Bominal Workspace Drive + Connect Files inheritance; no DPA-triggered complaints in inheritance period.

### 2.4 Purposes

| Purpose | Necessity | Lawful basis |
|---|---|---|
| File upload + download + storage | Contracted | Art. 6(1)(b) |
| Folder + permissions | Contracted | Art. 6(1)(b) |
| Share-link issuance | Operational benefit; opt-in | Art. 6(1)(f) legitimate interest + per-link consent |
| Full-text search | Operational benefit | Art. 6(1)(b) + 6(1)(f) |
| Preview render | Operational benefit | Art. 6(1)(b) + 6(1)(f) |
| Virus scan | Security obligation | Art. 6(1)(c) + 6(1)(f) |
| DLP scan | Tenant security obligation | Art. 6(1)(c) + 6(1)(f) |
| Sync delta protocol | Contracted | Art. 6(1)(b) |
| Audit-chain emission | Records-of-processing (Art. 30) | Art. 6(1)(c) |
| Legal-hold preservation | Legal obligation | Art. 6(1)(c) |
| WORM immutability | Legal obligation (per-pack) | Art. 6(1)(c) |
| T1 OCR / auto-tag / smart-search | Operational benefit; opt-in | Art. 6(1)(a) consent |
| Marketing / unrelated commercial use | NOT a purpose | N/A |

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | Sign-off pending; see §7 |
| Sample of prospective tenants | Scheduled pre-GA | Feedback folded into §6 |
| End-users (indirect via tenant) | Joint-controllership clause | Tenant disclosure obligation |
| Supervisory authority (DPA / PIPC) | Art. 36 NOT triggered (no H residual after mitigations) | — |
| Information security (ops-security) | YES | Shared residual catalog with threat-model |
| Engineering (axis-drive + each consuming µservice) | YES | LEAN gates enforced |
| External auditor | At first audit cycle | Cross-references DPIA |

## Step 4 — Necessity and proportionality

| Question | Assessment |
|---|---|
| Necessary? | YES — file storage cannot occur without bytes-at-rest. |
| Less-intrusive alternative? | Considered: external storage links only (no in-platform bytes). Rejected: defeats integration value + delta-sync efficiency + WORM compliance posture. |
| Proportionate? | YES — minimum-necessary at type level; cross-tenant share limited to explicit grant; client-side E2E opt-in for Personal pillar. |
| Anonymisation possible? | Partial — chunk content-addresses are derived from cleartext but stored as opaque blobs; full anonymisation incompatible with retrieval purpose. |
| Lawful basis | Per §2.4 |
| Special-category (Art. 9) | pack-us-healthcare: Art. 9(2)(h) (health care provision) + HIPAA BAA. pack-kr Art. 23: PIPA Art. 23(2) explicit consent at flagged-file level. |
| Transfer basis | SCCs only; default residency by pack. |
| Retention | per file/jurisdiction; HIPAA pack ≥ 6y WORM; SEC 17a-4(f) pack ≥ 6y WORM; default 24mo + per-tenant policy override. |
| Subject rights | Art. 15/16/17/18/20/21/22 honoured per §6. |

## Step 5 — Risks to data subjects

| ID | Risk | L | S | Score |
|---|---|---|---|---|
| R-01 | Personal-file leaks into Professional-context list/search/preview | M-H | H | **H** |
| R-02 | Cross-tenant share-link leaks more than the file content | M | H | **H** |
| R-03 | Share-link enumeration via brute-force | M | M-H | **M-H** |
| R-04 | Search index exposes file content across tenants | L | H | **M** |
| R-05 | Preview render leaks via container escape | L | H | **M** |
| R-06 | Long retention (WORM) enables surveillance pattern across years | M | M-H | **M-H** |
| R-07 | Sync delta-protocol exposes chunk metadata to inference attacks | L | M | **L-M** |
| R-08 | DSR right-to-erasure incomplete due to versioning + legal-hold overlap | M | M | **M** |
| R-09 | Joint-controllership: tenant doesn't disclose oyatie's processing to end-users | M-H | M | **M-H** |
| R-10 | PHI processed without BAA (pack-us-healthcare tenant doesn't sign BAA but ships clinical files) | M | H | **H** |
| R-11 | Sub-processor breach (object-store / KMS / cloud provider) | L | H | **M** |
| R-12 | Cross-border transfer of EU-resident bytes via mis-routed S3 ingress | L | H | **M** |
| R-13 | Children's-data files (school / pediatric clinic tenant) without parental consent | L | H | **M-H** |
| R-14 | Tenant-DEK leaked via log → mass decryption | L | H | **M** |
| R-15 | Auditor mis-pivot across tenants | L | H | **M** |
| R-16 | OCR / auto-tag (T1) infers sensitive attributes from file content | M | M | **M** |
| R-17 | WORM lock cannot honour erasure even where compliance permits | L | M-H | **M** |

Cross-reference: every risk has at least one corresponding STRIDE / LINDDUN threat in `threat-model.md`.

## Step 6 — Risk-reducing measures

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 | Rust type-level Personal vs Professional separation; Cedar `dual-context-isolation.md`; per-context Meilisearch index; LEAN check `oya-check-context-isolation` | L | axis-drive |
| R-02 | Type-narrowed cross-tenant share projection; LEAN check `oya-check-cross-tenant-share-projection`; annual pen-test | L | ops-security |
| R-03 | 256-bit random link IDs; rate limit + anomaly detection + auto-block; Argon2id KDF | L | axis-drive + ops-security |
| R-04 | Per-tenant Meilisearch index; cross-tenant query refused at API layer; LEAN check `oya-check-search-tenant-scoped` | L | axis-drive |
| R-05 | gVisor + seccomp; no network + no host FS; rasterised output; CIS K8s 1.9.0; quarterly chaos exercise | L | ops-security |
| R-06 | Per-tenant retention policy; default 24mo; WORM only on tenant-elected files; legal-hold reconciled with DSR | L-M | council-privacy |
| R-07 | Chunk hashes stored never with plaintext; per-tenant Redis ACL; sync session bound to OIDC subject | L | axis-drive |
| R-08 | DSR cascade with version + hold overlap policy: erasure honoured except where hold; partial-erasure (preserve audit-chain seal pointer) where compliant | M (hold-vs-erasure tension is accepted) | council-privacy |
| R-09 | Tenant DPA mandates upstream disclosure; tenant-onboarding checklist verifies | L-M | council-privacy |
| R-10 | pack-us-healthcare onboarding requires BAA pre-ingest; non-signed tenants pre-flighted to non-PHI pack | L | council-privacy |
| R-11 | Sub-processor list at `legal/sub-processors.md`; DPA + SCCs per sub-processor; quarterly review | M (sub-processor risk irreducible) | council-privacy |
| R-12 | Pack-pinning at ingress; route by pack tag; LEAN check refuses cross-pack route | L | axis-drive |
| R-13 | Tenant DPA includes child-data clause; tenant affirms parental-consent process; drive does not collect age | L | council-privacy |
| R-14 | Secret-scanner CI lane; `Secret<T>` type strips Debug; 90d rotation; rotation event re-encrypts | M (human-error baseline) | ops-security |
| R-15 | Auditor JIT tokens tenant-scoped at row level; pen-test annually | L | ops-security |
| R-16 | T1 OCR/auto-tag user-confirmable (30s reversibility); audit-chain emission; tenant opt-out at user level | L-M | foundry-runtime + axis-drive |
| R-17 | WORM-tier election requires tenant-policy-officer two-person rule + retention floor declared at election time; deviations reviewed pre-election | M (compliance-vs-rights tension is accepted; reviewed annually) | council-privacy + compliance |

## Step 7 — Sign-off

| Sign-off | Status |
|---|---|
| DPO (council-privacy) | `pending` |
| Information Security Officer (ops-security) | `pending` |
| µservice owner (axis-drive) | `pending` |
| Council-architecture | `pending` |

**DPO advice:** Residual risks all L or M after mitigations. Art. 36 prior consultation NOT triggered. Proceed with first-tenant onboarding subject to:
- Quarterly review of R-08 (DSR vs hold tension) + R-17 (WORM vs rights tension).
- Annual review of this DPIA.
- Re-trigger on each pack activation.

## Per-Pack Overlays

### pack-kr (KR PIPA + ISMS-P + 전자문서법 + KR-FSS)

PIPA Art. 33 + Enforcement Decree Art. 35 mandate 개인정보영향평가. This document fulfils that obligation.

- **PIPA Art. 23 (sensitive)**: per-file sensitivity flag; flagged files get additional access restrictions + per-tenant Cedar policy override.
- **PIPA Art. 23-2 (cross-border sensitive)**: KR-resident sensitive files stay in pack-kr.
- **PIPA Art. 28 (storage period)**: retention bounded per asset table.
- **PIPA Art. 29 (technical safeguards)**: cross-mapped in §6.
- **PIPC Notice 2020-7 methodology**: Steps 1–7 align.
- **KR-FSS 5y retention floor**: WORM tier enforces for financial-sector tenants.

### pack-us (CCPA / CPRA + SEC 17a-4(f) + FINRA 4511)

- **SEC 17a-4(f)**: WORM storage required for broker-dealer records; ADR-DRIVE-0006 satisfies via object-store compliance-mode object-lock.
- **FINRA 4511**: retention period satisfied via WORM tier.
- **CCPA §1798.100 et seq.**: DSR cascade satisfies right-to-know.
- **CCPA §1798.105**: DSR cascade satisfies right-to-delete (reconciled with WORM where elected).

### pack-us-healthcare (HIPAA + BAA + FDA 21 CFR Part 11)

HIPAA §164.308(a)(1)(ii)(A) requires risk-analysis substantially equivalent to a DPIA. This document fulfils that.

- **§164.502(a) Permitted Uses (TPO)**: clinical files fall under Treatment + Operations.
- **§164.502(b) Minimum Necessary**: per-context isolation + Cedar minimum-necessary projection.
- **§164.504(e) BAA**: BAA template at `legal/baa-template.md`.
- **§164.310 Physical Safeguards**: inherited from cloud-k8s + cloud-provider HIPAA-eligibility.
- **§164.312(a)(2)(iv) Encryption controls**: Tenant-DEK envelope (FIPS 140-3 OpenBao Transit).
- **§164.312(b) Audit Controls**: Ed25519 audit-chain seal + retention ≥ 6y.
- **§164.316 Documentation**: WORM tier ensures ≥ 6y retention; this DPIA, threat-model, compliance retained.
- **§164.404 Notification**: breach chain in `incident-response.md` ≤ 60-day window.
- **FDA 21 CFR Part 11**: electronic records integrity for HIPAA-covered tenants; OPSWAT multi-engine scan.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS + EU AI Act)

This document is the GDPR Art. 35 DPIA for EU tenant processing.

- **EDPB Guidelines 4/2019 (Art. 25)**: explicit alignment in §4 + §6.
- **EDPB Guidelines 9/2022 (breach notification)**: 72h chain in `incident-response.md`.
- **NIS2**: 24h + 72h + 1mo reporting timelines when oyatie crosses thresholds.
- **eIDAS 910/2014**: audit-chain Ed25519 seals are AdES.
- **Schrems II + Arts. 44–50**: SCC-gated transfers only; transfer register kept.
- **EU AI Act**: T1 OCR / auto-tag / smart-search = limited-risk under Annex III analysis (no high-risk classification for general file-storage assist). T2 auto-organize in HR-context REFUSED at Cedar layer pending ADR-DRIVE-XXXX conformity assessment.

### pack-jp (APPI), pack-sg (PDPA), pack-au (Privacy Act), pack-in (DPDPA), pack-br (LGPD), pack-ae (UAE PDPL), pack-ksa (KSA PDPL)

Per-pack DPIA overlays at `regional-packs/<pack>/drive-dpia-overlay.md`. Each cites the supervisory authority + article-level legal basis + residual risk position; aligned 1:1 with the calendar µservice overlay matrix for consistency across the µservice catalog.

## Re-review Triggers

- Annually (Q2).
- On every new pack activation.
- Change to processing purpose (§2.4) or data-class taxonomy.
- Sub-processor change.
- Breach notification triggered.
- Supervisory-authority guidance change.
- Post-incident (Sev-1 or Sev-2).

## References

- ADR-0028 (Bominal), ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0140.
- ADR-DRIVE-0001 through ADR-DRIVE-0006.
- `microservices/drive/threat-model.md`, `compliance.md`, `policy/*.cedar`, `multi-region.md`, `incident-response.md`, `legal/{dpa-template,baa-template,sub-processors,transfer-register,ropa}.md`.
- ICO DPIA template; CNIL DPIA methodology; EDPB Guidelines 4/2019 + 9/2022; PIPC Notice 2020-7.
- GDPR Art. 35 + Art. 36; KR PIPA Art. 33; HIPAA 45 CFR §164.308.
- LGPD Art. 38; DPDPA 2023 §10–§11.
- SEC 17a-4(f); FINRA Rule 4511.
- EU AI Act Regulation (EU) 2024/1689.
