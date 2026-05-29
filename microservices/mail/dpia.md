---
doc_class: DPIA
template_id: TPL-DPIA
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-mail + ops-legal
deciders: council-privacy, ops-security, axis-mail, council-architecture, ops-legal
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33 (개인정보영향평가) + ANPD RIPD methodology (BR) + DPDPA §10-11 (IN)
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145), ADR-0208, ADR-0215]
related_specs: [/specs/microservices/mail.json, /specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/mail/threat-model.md
  - microservices/mail/policy/dual-context-isolation.md
  - microservices/mail/policy/data-residency.md
  - microservices/mail/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, sub-processor list, or pack activation
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (DLP + abuse classifier scan all mail; mail-to-Workflow handoff extracts content)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (PHI possible in pack-us-healthcare; KR PIPA Art. 23 sensitive when RRN/medical data present)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII), A.5.31 (legal/statutory)"
  - "SOC 2 Privacy criteria (P1-P8, 2017 TSC)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33 (영향평가)", "PIPA Enforcement Decree Art. 35", "PIPC Notice 2020-7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A) (risk analysis)", "§164.312(b)", "§164.502(b)", "§164.514"]
  pack-eu: ["GDPR Arts. 35 + 36 (prior consultation)", "EDPB Guidelines 4/2019", "EDPB Guidelines 9/2022", "ePrivacy Directive Art. 5"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Part III + IV", "MAS Notice 644"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12", "OAIC APP guidelines"]
  pack-in: ["DPDPA 2023 §10 + §11 + §9 (children)"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 38 (RIPD)", "ANPD methodology"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021 Art. 23"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: mail µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. Mail processing engages two of the three Art. 35(3) automatic triggers:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation including profiling | **YES** | DLP scan + Rspamd abuse classifier + mail-to-Workflow handoff all involve systematic evaluation of mail content; the abuse classifier scores per-message; the handoff extracts content to drive workflow decisions. |
| Art. 35(3)(b): Large-scale processing of special-category data (Art. 9) | **YES (conditional)** | Pack-us-healthcare may carry PHI in mail body/attachments unless rigorously redacted; pack-kr KR PIPA Art. 23 classes mail content sensitive when RRN/medical data present; pack-eu special-category data may transit. Conditional ⇒ pack-activated. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | mail is not a public-area monitoring tool. |

Additional: Korean PIPC Notice 2020-7 mandates a 개인정보영향평가 (DPIA-equivalent) for systems processing sensitive personal information at scale — engaged.

Therefore: a DPIA is mandatory pre-deployment. This document is the canonical DPIA reviewed by EU DPAs (per Art. 35), the Korean PIPC (per PIPA Art. 33), HIPAA OCR (when pack-us-healthcare active), and ANPD (when pack-br active).

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** mail ingests inbound SMTP messages addressed to tenant domains; verifies DKIM/SPF/DMARC; scans for spam/phishing via Rspamd; persists to per-tenant encrypted mailbox; exposes IMAP/JMAP/REST for end-user mailbox access; signs outbound submissions via DKIM and delivers to recipient MX; runs nightly retention sweep (respecting legal holds); supports scoped legal holds + four-eyes plaintext-disclosure for eDiscovery export; supports mail-to-Workflow handoff with explicit consent/policy basis.

**How:** Postfix SMTP receiver → DKIM/SPF/DMARC verifier → Rspamd abuse classifier → per-tenant routing → mailbox-store (Postgres + S3 with per-tenant DEK envelope encryption) → encrypted-token search index (Tantivy) → IMAP/JMAP/REST frontends → outbound submission via Postfix → DKIM sign → per-tenant IP pool delivery → bounce processor → reputation tracker. Cross-context isolation enforced at kernel layer per `dual-context-isolation`.

**Where:** Per-pack region-pinned mail clusters (pack-kr → KR / pack-eu → EU / pack-us-healthcare → US-HC / pack-us / pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa). Pack-pinning per ADR-0117.

**When:** Continuous; sub-second receive p99; nightly retention sweep; legal hold engage on-demand.

**Who:** Per actor table in `microservices/mail/threat-model.md` §"Actors": external senders; tenant employees (Personal + Professional context); mail admins; compliance officers; workflow operators; oyatie SREs; long-lived workers; external auditors.

### 2.2 Scope of the processing

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `PII_IDENTIFYING` (mail content + headers) | sender/recipient/CC/BCC; subject; body; attachments | Art. 6(1)(b) contract necessity (delivery) + Art. 6(1)(c) legal obligation (audit) | per medium tenant: ~10⁵ mail/day × ~10 KB avg = ~1 GB/day raw; encrypted at rest |
| `PII_QUASI_IDENTIFIER` (mailbox metadata) | mailbox ID, folder names, thread IDs, message-IDs | Art. 6(1)(b) contract | ~10⁴ metadata records/day per medium tenant |
| `SENSITIVE_PIPA_ART23` (KR pack) | mail content when containing RRN / medical / political / labour-union / sexual-life data | KR PIPA Art. 15 + 23 (consent at tenant onboarding) + Art. 23-2 (cross-border restriction) | varies; redactor minimises |
| `PHI` (pack-us-healthcare) | clinical data / patient identifiers in mail | HIPAA §164.502(a) TPO permitted use under BAA; HITECH §13402 | varies; redactor minimises |
| `AUDIT` (mail events + chain-of-custody seals) | MessageReceived, MessageSent, LegalHoldEngaged, etc. | Art. 6(1)(c) legal obligation (records); Art. 6(1)(f) legitimate interest | 1 record per event |
| `BEHAVIORAL_TENANT_PRODUCT` (mailbox usage stats, deliverability score) | per-tenant volume, bounce rate, reputation score | Art. 6(1)(b) + Art. 6(1)(f) | continuous |
| `SECRET` (DKIM private key, TLS cert, DEK envelope) | per-tenant cryptographic material | not personal data; managed under ISO 27001 A.5.17 + A.8.24 | per tenant |
| `BEHAVIORAL_PERSONAL` (Personal-context mailbox content) | personal mail body/headers | Art. 6(1)(b) end-user contract via tenant DPA; sub-processor relationship is direct-to-user via tenant | end-user-controlled |

**Geographical scope:** Per pack:
- pack-kr: OCI ap-seoul-1 — KR-resident mail stays in KR.
- pack-eu: OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) — EU-resident mail stays in EU.
- pack-us-healthcare: OCI us-ashburn-1 (HIPAA-eligible) — HIPAA covered.
- pack-us: OCI us-ashburn-1 + us-phoenix-1 — US-resident mail.
- pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa: each pinned to its primary region.

**Cross-border transfer:** Forbidden by default per `policy/data-residency.md`. Allowed only with tenant-executed SCCs (EU + UK) or equivalent local mechanism. Recorded in `legal/transfer-register.md`.

### 2.3 Context of the processing

- **Data subjects:** Tenant employees (Personal + Professional context); external senders/recipients (joint controllership with tenant); tenant admin/compliance personnel.
- **Relationship to data subjects:** Joint controllership with tenant per Art. 26 (tenant is controller of employee + external-recipient data; oyatie is joint controller for the storage + processing portion). Joint-controllership terms recorded in tenant DPA template at `legal/dpa-template.md`.
- **Reasonable expectations:** Tenant employees expect mail to be processed by their employer's mail provider. External senders/recipients expect mail to be delivered (no expectation of processing-for-marketing). Personal-pillar users expect employer cannot read their personal mail (Bominal ADR-0215 invariant).
- **Previous experience:** Bominal mail (legacy communications mail surface) operated under same model with `oya-mail-* legacy-prefix migration` crates; no DPA-triggered complaints in 12 months. Inherited per `feedback_bominal_inheritance_precedence.md`.
- **Industry codes:** Voluntary alignment with M³AAWG senders-best-practices and DMARC.org.

### 2.4 Purposes of the processing

| Purpose | Necessity | Lawful basis |
|---|---|---|
| **Deliver mail to/from tenant** | Necessary for the tenant's contracted mail service | Art. 6(1)(b) contract |
| **Spam/phishing protection** | Necessary for security; legitimate-interest balancing assessment | Art. 6(1)(f) |
| **Retention policy enforcement** | Required by tenant's regulatory regime | Art. 6(1)(c) legal obligation |
| **Legal hold + eDiscovery** | Required for tenant's legal obligations (litigation, regulatory) | Art. 6(1)(c) |
| **DKIM signing of outbound** | Necessary for delivery + anti-spoofing | Art. 6(1)(b) + Art. 6(1)(f) |
| **Mail-to-Workflow handoff** | Optional tenant-feature; per-user explicit OR policy-basis per tenant DPA | Art. 6(1)(b) (when user-explicit) OR Art. 6(1)(f) (when policy-basis) |
| **Deliverability dashboard** | Necessary for tenant SLA management | Art. 6(1)(b) |
| **Audit-chain emission** | Mandatory for SOC 2 + ISO 27001 + HIPAA + KR PIPA + GDPR Art. 30 records-of-processing | Art. 6(1)(c) |
| **Marketing / unrelated commercial use of mail content** | NOT a purpose | N/A — explicitly excluded; mail content never used for marketing or training |
| **Aggregating mail patterns for product improvement** | Optional; differential-privacy-clean cross-tenant aggregates only | Art. 6(1)(f) + DP analysis published |

Purposes are explicit, legitimate, specified at tenant onboarding via DPA template (Art. 5(1)(b)).

## Step 3 — Consultation

| Stakeholder | Consulted? | Outcome |
|---|---|---|
| DPO (council-privacy chair) | YES | Sign-off pending; see §7 |
| Tenant representative (sample of 3 pre-GA tenants) | Scheduled — pre-GA | Feedback folded into §6 |
| Data subjects (employees + recipients) | Indirect via tenant DPA cascade | Joint-controllership clause cascades upstream-disclosure obligation |
| Supervisory authority (EU DPA / KR PIPC / etc.) | Prior consultation (Art. 36) — NOT triggered (no residual high risk after mitigations; see §6 + §7) | If residual > Medium, Art. 36 triggered |
| Information security (ops-security) | YES — co-author of threat-model.md | Threat-model + DPIA share residual-risk catalog |
| Legal team (ops-legal) | YES — co-deciders | eDiscovery, BAA, transfer-register templates |
| Workflow team (axis-workflow) | YES | mail-to-Workflow handoff consent design |
| Engineering teams (axis-mail + each consumer) | YES | dual-context invariant + retention engine design |
| External auditor (SOC 2 / ISO 27001 firm) | At first audit cycle | Cross-references this DPIA |

DPO independent advice + sign-off recorded at §7.

## Step 4 — Assess necessity and proportionality

| Question | Assessment |
|---|---|
| Is processing necessary to achieve the purpose? | YES — mail delivery is impossible without processing; retention/hold is mandatory per regulatory regimes. |
| Is there a less intrusive alternative? | Considered: end-to-end encrypted-only (Proton-Mail-style). Rejected for Professional context because regulated retention and legal hold are infeasible without server-side access. Adopted hybrid: Professional context server-side encrypted under tenant DEK with org-admin visibility; Personal context end-to-end encrypted under user-derived DEK (when user opts in) with org invisibility. |
| Is processing proportionate to the purpose? | YES — content access scoped per BC: SMTP frontends touch headers + body only at receive/send time; mailbox-store stores ciphertext; search index uses HMAC tokens; legal-hold accesses plaintext only under four-eyes. Per Art. 5(1)(c) data-minimisation. |
| Does processing achieve a public interest or substantial private interest? | YES — operational mail service is core to tenant; tenant's regulated retention serves regulatory mandate. |
| Could the purpose be achieved by anonymised / pseudonymised data? | PARTIALLY — pseudonymisation applied (per-tenant hashed customer-id; per-user hashed identifiers in metric labels). Full anonymisation would prevent mail delivery (which requires per-recipient resolution). Pseudonymisation is the proportionate compromise. |
| Lawful basis (Art. 6) | Identified per purpose in §2.4. |
| Special-category basis (Art. 9, if applicable) | pack-us-healthcare PHI: Art. 9(2)(h) (provision of health care under contract) + HIPAA BAA covering 45 CFR §164.504(e). pack-kr sensitive: PIPA Art. 23(2) explicit consent at tenant onboarding. |
| Transfer basis (Arts. 44-46) | §2.2 cross-border: SCC only; default residency by pack. |
| Retention | Per asset class in `threat-model.md` §"Assets & Data Classification" + per-pack overlays in `policy/data-residency.md`. Defaults: mail content per tenant policy (default 7y professional / user-controlled personal); audit-chain ≥ 1y (HIPAA 6y; KR-FSS 5y). |
| Rights of data subjects | Honoured per §6 mitigations: access (Art. 15), rectification (Art. 16), erasure (Art. 17), restriction (Art. 18), portability (Art. 20), objection (Art. 21), automated-decision-protections (Art. 22). |

## Step 5 — Identify and assess risks to data subjects

| ID | Risk to data subject | Likelihood | Severity | Score |
|---|---|---|---|---|
| R-01 | Cross-tenant mailbox leak (rival tenant infers business) | L-M | H | **H** |
| R-02 | Cross-pillar mailbox leak (employer reads employee personal mail) | M (developer-error class) | H | **H** |
| R-03 | Legal-hold bypass — held material erased before discovery | L | H | **M-H** |
| R-04 | eDiscovery export discloses more than scoped (over-disclosure) | M | M-H | **M-H** |
| R-05 | Long retention enables surveillance pattern (12+ years of mail; longitudinal profile) | M | M-H | **M-H** |
| R-06 | Mail-to-Workflow handoff extracts content without genuine consent | M | M | **M** |
| R-07 | SMTP spoofing of tenant identity reaches employees (phishing) | M | M-H | **M-H** |
| R-08 | Cross-border transfer of EU mail data via misrouted ingest | L | H | **M** |
| R-09 | DSR (right-to-erasure) incomplete because mail spans multiple stores (Postgres + S3 + search index + audit-chain) | M | M | **M** |
| R-10 | Joint-controllership confusion: tenant doesn't disclose oyatie's processing to employees/recipients | M-H | M | **M-H** |
| R-11 | Children's data (DPDPA §9; pack-in) processed without parental consent | L | H | **M-H** |
| R-12 | PHI in mail without BAA (pack-us-healthcare; tenant ships clinical content without signing BAA) | M | H | **H** |
| R-13 | Subpoena-driven mass disclosure without notice (gag order constraint) | L | M-H | **M** |
| R-14 | DKIM signing key compromise → spoofed mail attributed to tenant | L | H | **M** |
| R-15 | DLP scanner false-positive blocks legitimate confidential mail | M | M | **M** |
| R-16 | Auditor mis-pivot from tenant-A to tenant-B during engagement | L | H | **M** |
| R-17 | End user (employee) cannot exercise GDPR/PIPA rights against employer because rights are routed through employer | M-H | M | **M-H** |

Cross-reference: every risk has at least one mitigation in §6 + at least one corresponding STRIDE/LINDDUN threat in `threat-model.md`.

## Step 6 — Identify measures to reduce risk

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 (cross-tenant leak) | Postgres RLS + S3 IAM + per-tenant DEK + LEAN `oya-check-rls-policy-conformance`; annual pen-test; weekly threat hunt | L | ops-security |
| R-02 (cross-pillar leak) | ContextBoundaryGuard at kernel + `oya-check-dual-context-cross-boundary` LEAN lane + Personal-pillar user-derived DEK (org cannot decrypt); annual pen-test against pillar boundary | L | axis-mail + council-privacy |
| R-03 (hold bypass) | Hold-before-purge invariant kernel-enforced; retention sweep re-validates hold state on every run; audit-chain on every hold action | L | council-privacy + axis-mail |
| R-04 (over-disclosure on export) | Four-eyes rule for plaintext disclosure; scope explicit and approved separately; bundle digest re-derives from source blocks; auditor scope tenant-bounded | L | ops-legal + council-privacy |
| R-05 (long-retention surveillance) | Retention defaults aggressive; tenant + per-pack statutory floors enforced; DSR cascade honours Art. 17; tenant DPA discloses retention to employees | L-M | council-privacy |
| R-06 (handoff without consent) | Handoff requires explicit user action (UI button) OR tenant policy basis (declared in DPA); audit-chain on every handoff; per-tenant opt-out at user level | L | axis-mail + axis-workflow + council-privacy |
| R-07 (spoofing reaches employees) | DKIM/SPF/DMARC inbound verification + Rspamd + per-tenant tightening; lookalike-domain monitoring | M (targeted lookalike baseline; mitigated via training + monitoring) | ops-deliverability + ops-security |
| R-08 (cross-border misroute) | Pack-pinning at SMTP-receiver level; routing by tenant.jurisdiction tag; misroute caught by integration test | L | axis-mail |
| R-09 (DSR incompleteness) | DSR cascade (per `oya-dsr-cascade-runner`) queries Postgres + S3 + search index + audit-chain (best-effort within retention); 30-day SLA; soft-delete with grace; documented limitations | M (best-effort within retention is the accepted residual) | council-privacy |
| R-10 (joint-controllership confusion) | Tenant DPA mandates upstream disclosure clause; tenant onboarding verifies disclosure in tenant privacy notice + employee mail notice; non-disclosure = onboarding refused | L-M | council-privacy + gtm-customer-success |
| R-11 (children's data) | DPDPA §9 + GDPR Art. 8: tenant DPA includes child-data clause; tenant must affirm parental-consent process; mail inherits tenant's age-gating | L | council-privacy |
| R-12 (PHI without BAA) | pack-us-healthcare onboarding requires BAA pre-sign; non-BAA tenants pre-flighted to pack-us (non-HC); PHI redactor at SDK level for surfaces that would otherwise capture content | L | council-privacy + sales-legal |
| R-13 (subpoena gag) | Notice-to-tenant clause in DPA; minimisation of disclosure scope; transparency report annual; legal team review of every request | M (regulatory inevitability) | ops-legal + council-privacy |
| R-14 (DKIM key compromise) | OpenBao + 2-person rule + 90d rotation; LEAN `oya-check-dkim-key-rotation-conformance`; public-key DNS monitoring | L | ops-security + ops-deliverability |
| R-15 (DLP false-positive) | Tenant-configurable thresholds; per-user override channel (with admin approval); audit-chain on every DLP block | L-M | axis-mail + tenant admin |
| R-16 (auditor mis-pivot) | Auditor JIT tokens tenant-scoped at folder level; annual pen-test against auditor boundary | L | ops-security |
| R-17 (rights cascade through employer) | Employee mail notice mandated in tenant DPA; per-pack residency law (where applicable) routes rights directly via PIPC/DPA bypassing employer; DSR cascade honours both routes | M (regulatory baseline; employees retain direct PIPC/DPA channels) | council-privacy |

## Step 7 — Sign-off and record outcomes

| Sign-off | Status | Signatory |
|---|---|---|
| Data Protection Officer (council-privacy chair) | `pending` | TBA at first-tenant onboarding |
| Information Security Officer (ops-security chair) | `pending` | TBA |
| µservice owner (axis-mail lead) | `pending` | TBA |
| Legal officer (ops-legal lead) | `pending` | TBA |
| Council-architecture chair | `pending` | TBA |

**DPO advice:**
Residual risks after mitigations are all rated L or M (no H or M-H residuals remain after mitigations). Therefore Art. 36 prior consultation is NOT triggered. The DPO advises proceeding with first-tenant onboarding subject to:
- Quarterly review of R-02 (cross-pillar) — engineering-discipline metric over time.
- Annual review of this DPIA.
- Re-trigger on every pack-activation (each new pack engages distinct legal frameworks).
- Re-trigger on any change to dual-context invariant (Personal vs Professional).

**Outcomes documented:**
- Mitigations adopted: every measure in §6 in-scope for the M03 P01 IP series.
- Records-of-processing register: `legal/ropa.md`.
- Joint-controllership template: `legal/dpa-template.md`.
- BAA template (pack-us-healthcare): `legal/baa-template.md`.
- Transfer register: `legal/transfer-register.md`.

## Per-Pack Overlay Sections

### pack-kr (Korea PIPA + ISMS-P)

PIPA Art. 33 + Enforcement Decree Art. 35 require 개인정보영향평가 for systems processing sensitive personal information at scale. This document fulfils that obligation for KR tenants.

Additional KR considerations:
- **PIPA Art. 23 (sensitive PII)**: mail content treated as sensitive when RRN/medical data present.
- **PIPA Art. 23-2 (sensitive cross-border)**: KR-resident sensitive mail stays in pack-kr.
- **PIPA Art. 28 (storage period)**: retention bounded; statutory minimum per category.
- **PIPA Art. 29 (technical safeguards)**: cross-mapped in §6 to the 12 prescribed safeguards.
- **PIPA Art. 33-2 (DPO appointment)**: council-privacy chair = KR DPO.
- **KR 전자문서법 Arts. 5-7**: audit-chain Ed25519 seals satisfy electronic-document integrity, storage, verification.
- **KR-FSS regulated tenants**: mail retention floor 5y + KMS-in-KR + operator-access-KR-resident.

### pack-us-healthcare (HIPAA)

HIPAA §164.308(a)(1)(ii)(A) requires risk analysis substantially equivalent to a DPIA. This document fulfils that requirement.

Additional HIPAA considerations:
- **§164.502(a) Permitted Uses (TPO)**: mail operations fall under "Operations".
- **§164.502(b) Minimum Necessary**: PHI redaction at SDK level for surfaces that would capture content; mail-to-Workflow handoff requires explicit consent or BAA-declared basis.
- **§164.504(e) Business Associate**: oyatie acts as BA for HIPAA tenants; BAA template at `legal/baa-template.md`.
- **§164.310 Physical Safeguards**: inherited from cloud-k8s DPIA + OCI HIPAA-eligibility.
- **§164.312(b) Audit Controls**: Ed25519 audit-chain + 6y retention.
- **§164.404 Notification to Individuals**: 60d max; incident-response chain.
- **§164.406 Notification to Media**: >1000 affected.
- **§164.408 Notification to HHS**: OCR portal.
- **45 CFR Part 164 Subpart D**: integrated.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS + ePrivacy)

This document is the GDPR Art. 35 DPIA for EU-resident tenant processing.

Additional EU considerations:
- **EDPB Guidelines 4/2019** (Art. 25 by design): explicit alignment in §4 + §6.
- **EDPB Guidelines 9/2022** (breach notification): 72h chain in `incident-response.md`.
- **NIS2**: when oyatie crosses thresholds, 24h/72h/1mo timelines.
- **eIDAS 910/2014 Art. 26**: Ed25519 audit-chain seals = AdES.
- **ePrivacy Directive Art. 5**: e-mail confidentiality + processor stance.
- **Schrems II + Arts. 44-46**: no cross-border without SCC; transfer register.
- **Children's data (Art. 8)**: inherited via tenant age-gating.

### pack-jp (APPI)

APPI Arts. 17-27 cover most rules; APPI does not mandate DPIA-equivalent but encourages voluntary risk assessment. This document satisfies voluntary assessment.

- **APPI Art. 17**: purpose declared at tenant onboarding.
- **APPI Art. 21**: pack-jp residency.
- **APPI Art. 27**: sensitive-data consent via tenant DPA cascade.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack DPIA overlays at `regional-packs/<pack>/dpia-overlay.md` carry pack-specific legal-citation depth. Each follows this document's 7-step structure with local-law substitution.

## Re-review Triggers

- Annually (Q2).
- On every new pack activation.
- On any change to processing purpose (§2.4) or data-class taxonomy.
- On any sub-processor change.
- On any breach notification triggered.
- On supervisory-authority guidance change.
- Post-incident (any Sev-1/Sev-2).
- On any change to the dual-context invariant.

## References

- ADR-0008: Data use boundary.
- ADR-0028 (Bominal): Audit chain.
- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0135: dissolution; dual-context invariant.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-grouping forward policy.
- ADR-0140: Cedar policy enforcement.
- Bominal ADR-0208 / 0210 / 0215: inherited.
- `microservices/mail/threat-model.md` — paired security artifact.
- `microservices/mail/policy/{dual-context-isolation, data-residency}.md`.
- `microservices/mail/compliance.md`.
- `microservices/mail/incident-response.md`.
- `microservices/mail/legal/{dpa-template, baa-template, sub-processors, transfer-register, ropa}.md`.
- ICO DPIA template — `ico.org.uk`.
- CNIL DPIA methodology — `cnil.fr/en/PIA`.
- EDPB Guidelines 4/2019, 9/2022.
- PIPC Notice 2020-7.
- GDPR Art. 35 + Art. 36.
- KR PIPA Art. 33 + Enforcement Decree Art. 35.
- HIPAA 45 CFR §164.308(a)(1)(ii)(A).
- LGPD Art. 38; ANPD methodology.
- DPDPA 2023 §10-§11.
