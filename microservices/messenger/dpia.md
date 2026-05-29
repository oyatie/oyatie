---
doc_class: DPIA
template_id: TPL-DPIA
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-messenger
deciders: council-privacy, ops-security, axis-messenger, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/microservices/messenger.json]
related_artifacts:
  - microservices/messenger/threat-model.md
  - microservices/messenger/policy/dual-context-isolation.md
  - microservices/messenger/policy/data-residency.md
  - microservices/messenger/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — PARTIAL (mention-resolution + presence form profiling-adjacent processing)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (PHI possible in pack-us-healthcare channels; sensitive data under PIPA Art. 23)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII), A.5.31 (legal/statutory)"
  - "SOC 2 Privacy criteria (P1-P8)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33", "PIPA Enforcement Decree Art. 35", "PIPC Notice 2020-7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A)", "§164.312(b)", "§164.502(b)", "§164.514"]
  pack-eu: ["GDPR Arts. 35 + 36", "EDPB Guidelines 4/2019", "EDPB Guidelines 9/2022", "ePrivacy Directive 2002/58 Art. 5(3)"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Part III + Part IV"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12"]
  pack-in: ["DPDPA 2023 §10 + §11"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 38"]
  pack-ae: ["UAE PDPL Art. 23"]
  pack-ksa: ["PDPL Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: messenger µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. The messenger µservice engages:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic profiling | PARTIAL | Mention resolution + presence + read-receipt aggregations form a profiling-adjacent dataset, even if the primary purpose is delivery |
| Art. 35(3)(b): Large-scale special-category | **YES (conditional)** | Pack-us-healthcare PHI in channel bodies + attachments; pack-kr sensitive data under PIPA Art. 23 |
| Art. 35(3)(c): Public-area monitoring | NO | n/a |

KR PIPC's Notice 2020-7 mandates DPIA when processing handles sensitive personal information at scale — engaged when first pack-kr enterprise tenant exceeds threshold.

DPIA is mandatory pre-deployment. This document is the canonical DPIA reviewed by EU DPAs (Art. 35), KR PIPC (PIPA Art. 33), and HIPAA OCR (post-BAA) at first-tenant onboarding per pack.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** End-users post messages, threads, reactions, mentions, attachments to channels + DMs. The system stores, indexes, fans out via real-time delivery (WebSocket), and surfaces search + presence to authorised recipients.

**How:** Client → ingress (TLS/WAF) → WebSocket gateway → BC services (REST + worker) → Postgres (messages, channels, ACL, threads) + Valkey (presence, read-receipts) + S3 (attachments) + Tantivy/ES (search) + audit-chain seal.

**Where:** Per-pack region-pinned messenger clusters. pack-kr (KR), pack-eu (EU), pack-us (US), pack-us-healthcare (US, HIPAA-eligible), and conditional packs.

**When:** Continuous; sub-second latency for delivery; 60s cadence for SLO evaluation.

**Who:** End-users (tenants' customers); tenant operators; oyatie operators; ontology + mail + workflow-engine + audit-chain µservices (machine actors).

### 2.2 Scope of the processing

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `BEHAVIORAL_TENANT_PRODUCT` | Channel/DM message bodies (professional), read-receipts, presence, reactions | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest | ~10⁶ messages/day per medium tenant |
| `PII_IDENTIFYING` | User handles, mentions, message-author identity | Art. 6(1)(b) contract | every message |
| `PII_QUASI_IDENTIFIER` | IPs, user-agents, geo-derived presence | Art. 6(1)(f) (minimised at SDK) | per-session |
| `PERSONAL` (E2E-ciphertext) | Personal-context DMs | Art. 6(1)(b) personal contract; never decryptable server-side | ~10⁵ DMs/day per medium tenant |
| `SENSITIVE_PIPA_ART23` | Sensitive data in messages (medical, juvenile, biometric) — pack-kr | KR PIPA Art. 15 + 23 + explicit consent | varies |
| `PHI` (pack-us-healthcare) | Patient identifiers / clinical content in channel messages or attachments | HIPAA §164.502 Permitted Uses (under BAA) | targeted to 0 via redactor where possible |
| `AUDIT` | Channel-create / member-grant / disclosure / hold events | Art. 6(1)(c) | 1 record per state transition |
| `SECRET` | Per-tenant DEK, session tokens, signing keys | not personal data | OpenBao-bound |

**Geographical scope:** Per pack-pinning (data-residency.md).

**Cross-border transfer:** Forbidden by default; allowed only with tenant SCCs (Arts. 44–46) + multi-region.md.

### 2.3 Context of the processing

- **Data subjects:** End-users of tenant applications; tenant operators; oyatie operators.
- **Relationship:** Joint controllership with tenant under Art. 26.
- **Reasonable expectations:** End-users expect operational telemetry + admin disclosure-on-trigger per tenant onboarding notice.
- **Previous experience:** Bominal messenger predecessor; no DPA-triggered complaints in 24 months.
- **Industry codes:** ePrivacy Directive Art. 5(3) (communications confidentiality); voluntary alignment.

## Step 3 — Consultation

- Internal: council-privacy (Q2 quarterly review), ops-security, council-architecture, axis-messenger lead.
- External: tenant pilot focus groups (3 pack-kr + 2 pack-eu enterprise tenants) confirmed expectations re: presence visibility + admin disclosure transparency.
- Supervisory authority: KR PIPC notified at pack-kr first-tenant onboarding; EU DPA notified at pack-eu first-tenant signature.

## Step 4 — Necessity & proportionality

### 4.1 Necessity

- **Purpose limitation:** Each data class processed only for its declared purpose (delivery + search + presence). Cross-purpose use (e.g., marketing) requires fresh consent.
- **Minimisation:** OTel redactor + `data_class` annotation enforcement; attachment-preview redactor; search-result Cedar filter.
- **Accuracy:** Edit-window allows author corrections; admin-disclosure record corrections audit-chain-immutable.
- **Storage limitation:** Per-pack retention bounds in `policy/data-residency.md`.

### 4.2 Proportionality

- Less-intrusive alternatives considered: server-side plaintext-only (rejected — destroys personal-DM privacy); no-search (rejected — destroys user utility).
- Selected: server-stores professional with tenant-DEK; personal-DM ciphertext-only; search Cedar-filtered.

## Step 5 — Risks to data subjects

| Risk ID | Risk | Likelihood | Severity | Risk score |
|---|---|---|---|---|
| R-01 | Cross-tenant message leak (RLS misconfig) | M | H | High |
| R-02 | PHI leak in attachment preview (pack-us-healthcare) | M | H | High |
| R-03 | Personal-DM ciphertext break attempt by tenant admin | M | H | High |
| R-04 | Channel-name + member-list metadata leak | M | M | Medium |
| R-05 | Search over-permitted result | M | H | High |
| R-06 | Attachment URL shared-link guess | M | H | High |
| R-07 | Cross-context routing (personal-DM → professional channel) | L | H | Medium |
| R-08 | Mention-graph identity correlation (linkability) | M | M | Medium |
| R-09 | Erasure right-best-effort due to retention floors | M | M | Medium |
| R-10 | Admin-disclosure inherent exposure of DM bodies | L | H | Medium |
| R-11 | Cross-pack residency misroute | L | H | Medium |
| R-12 | Presence-leak: stalker-style monitoring | M | M | Medium |

## Step 6 — Mitigations

| Risk ID | Mitigation | Residual |
|---|---|---|
| R-01 | Postgres RLS + Cedar + pen-test annual | L |
| R-02 | Pack-us-healthcare disables auto-preview; OCR-redactor; access bound to message ACL | L–M |
| R-03 | Server stores ciphertext only; decrypt-attempt audit metric (target=0) | L |
| R-04 | Membership-bound metadata reads; cardinality limits | L |
| R-05 | Cedar post-filter on every search result; integration test asserts no over-permit | L |
| R-06 | Signed short-TTL URLs; per-fetch Cedar re-eval | L |
| R-07 | Data-model type invariant (DirectConversation ≠ Channel); LEAN lane | L |
| R-08 | Per-tenant mention scope; no cross-tenant linkability | L |
| R-09 | DSR cascade marks tombstoned + redacts identifiers; user-side disclosure of retention floor | M |
| R-10 | Four-eyes + audit-chain + tenant onboarding disclosure | M |
| R-11 | Pack-router Cedar enforces; CI lane validates Helm pack-pinning | L |
| R-12 | Presence opt-out toggle (user-side); presence aggregates per-channel only | L–M |

## Step 7 — Sign-off

- council-privacy chair: `pending`
- ops-security director: `pending`
- council-architecture chair: `pending`
- axis-messenger lead: `pending`

## Per-pack overlays

### pack-kr

- KR PIPA Art. 23 sensitive data — additional consent at channel-create for sensitive channels.
- PIPC Notice 2020-7 — this DPIA satisfies impact-assessment requirement at scale.
- KR PIPA Art. 28 — outside-of-KR transfer forbidden by default.

### pack-us-healthcare

- HIPAA §164.308(a)(1)(ii)(A) risk-analysis — this DPIA satisfies.
- HIPAA §164.502(b) minimum-necessary — search + preview redaction.
- Per-tenant BAA at `legal/baa-template.md`.

### pack-eu

- GDPR Art. 35 prior consultation — required when DPIA indicates residual high risk; section above shows residual ≤ M for all rows.
- ePrivacy Directive Art. 5(3) — covered by Cedar + RLS + E2E.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack overlays at `regional-packs/<pack>/messenger-dpia-overlay.md`.

## DPIA refresh triggers

- Any new data class.
- Any new sub-processor.
- Any change to dual-context invariant.
- Any new pack activation.
- Annual scheduled review.
- Post-incident review.

## References

- `microservices/messenger/threat-model.md`.
- `microservices/messenger/policy/dual-context-isolation.md`.
- `microservices/messenger/policy/data-residency.md`.
- `microservices/messenger/compliance.md`.
- Bominal ADR-0208 + ADR-0215.
- Parallel ADR-0135.
- GDPR + KR PIPA + HIPAA + APPI + LGPD + PDPA full citations.
