---
doc_class: DPIA
template_id: TPL-DPIA
microservice: calendar
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-calendar
methodology: ICO DPIA + CNIL DPIA + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/calendar/threat-model.md
  - microservices/calendar/policy/event-isolation.md
  - microservices/calendar/policy/data-residency.md
  - microservices/calendar/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, or sub-processor list
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (cross-tenant availability + room utilisation profile tenant behavior)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES conditional (PHI in clinical-scheduling tenant via pack-us-healthcare; sensitive Art. 23 PIPA via subject lines)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — N/A"
doc_status: published
---

# Data Protection Impact Assessment: calendar µservice

## Step 1 — Need for a DPIA

Calendar processes per-event content (titles, descriptions, attendees, locations), attendance state, cross-tenant availability, and resource bookings. Two of three Art. 35(3) automatic triggers engaged:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a) Systematic + extensive evaluation | YES | Availability resolver continuously projects per-attendee free/busy across tenant boundaries; quasi-profiling. |
| Art. 35(3)(b) Large-scale special-category | YES (conditional) | Clinical-scheduling (pack-us-healthcare) carries PHI; meeting subjects may contain PIPA Art. 23 categories. |
| Art. 35(3)(c) Public-area monitoring | NO | — |

Also: PIPC Notice 2020-7 (KR) mandates DPIA when sensitive personal information at scale → engaged for pack-kr; APPI voluntary scheme followed for pack-jp.

DPIA mandatory pre-deployment. Reviewed by EU DPAs (Art. 35) and KR PIPC (Art. 33) at first-tenant onboarding per jurisdiction.

## Step 2 — Describe the processing

### 2.1 Nature

**What:** Event creation, attendance management, recurring expansion, free/busy projection (within and across tenants), resource booking, RFC 5545 ITIP invitations, RFC 5546 RSVP, RFC 5545 .ics import/export, RFC 4791 CalDAV.

**How:** REST + CalDAV ingress → Postgres event store (per-tenant RLS + tenant-DEK envelope encryption) → Valkey availability cache → Workflow events to mail (invitation delivery) + audit-chain (seal emission) + observability (telemetry).

**Where:** Per-pack region-pinned Postgres + Valkey (pack-kr → KR; pack-eu → EU; pack-us → US; pack-us-healthcare → BAA-eligible US; pack-jp → JP; etc.). Residency enforced via ADR-0117 + ADR-0140.

**When:** Continuous; on-demand for user actions; recurring background sweeps for retention + recurrence expansion.

**Who:** Per the actor table in `threat-model.md` §"Actors".

### 2.2 Scope

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `PROFESSIONAL_EVENT_CONTENT` | meeting title, description, location | Art. 6(1)(b) contract + 6(1)(f) legitimate interest | 10⁵ events/day per medium tenant |
| `PERSONAL_EVENT_CONTENT` | personal calendar entries | Art. 6(1)(a) consent + 6(1)(b) | 10⁴/day per active user |
| `PII_IDENTIFYING` | attendee emails, display names | Art. 6(1)(b) contract | 10× event count |
| `PII_QUASI_IDENTIFIER` | room names, locations, IP in CalDAV access | Art. 6(1)(f) legitimate interest | varies |
| `SENSITIVE_PIPA_ART23` | meeting subjects when tagged sensitive | KR PIPA Art. 23(2) explicit consent | per flagged event |
| `PHI` (pack-us-healthcare only) | clinical-scheduling content under BAA | HIPAA §164.502(a) Permitted Uses | per BAA tenant |
| `AUDIT` | event lifecycle records | Art. 6(1)(c) legal obligation | 1 per event mutation |
| `SECRET` | tenant-DEK, API keys | not personal data | managed via OpenBao |

**Geographical scope:** per pack (per §2.1).

**Cross-border transfer:** forbidden by default; allowed with tenant-executed SCCs per Arts. 44–46 per `multi-region.md`.

### 2.3 Context

- **Data subjects:** end-users (the tenant's employees + invitees + customers); tenant operators; external attendees (RSVP-only); oyatie operators (internal).
- **Relationship:** joint controllership with tenant (GDPR Art. 26) for end-user event data; oyatie sole processor for operational metadata.
- **Reasonable expectations:** tenant operators expect operational scheduling; end-users expect scheduling per tenant's privacy notice; external attendees expect RSVP-only access.
- **Previous experience:** Bominal Connect Calendar inheritance per ADR-0208; no DPA-triggered complaints in inheritance period.

### 2.4 Purposes

| Purpose | Necessity | Lawful basis |
|---|---|---|
| Event scheduling | Contracted | Art. 6(1)(b) |
| Cross-tenant availability | Operational benefit; opt-in | Art. 6(1)(f) legitimate interest + per-grant consent |
| Resource booking | Contracted | Art. 6(1)(b) |
| Invitation delivery | Contracted | Art. 6(1)(b) |
| .ics / CalDAV interoperability | Portability (Art. 20) | Art. 6(1)(b) + 6(1)(c) |
| Audit-chain emission | Records-of-processing (Art. 30) | Art. 6(1)(c) |
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
| Engineering (axis-calendar + each consuming µservice) | YES | LEAN gates enforced |
| External auditor | At first audit cycle | Cross-references DPIA |

## Step 4 — Necessity and proportionality

| Question | Assessment |
|---|---|
| Necessary? | YES — scheduling cannot occur without event content. |
| Less-intrusive alternative? | Considered: external scheduling links only (no in-platform event content). Rejected: defeats integration value + RFC 5545 portability. |
| Proportionate? | YES — minimum-necessary at type level; cross-tenant projection limited to free/busy; data-class annotations enforce. |
| Anonymisation possible? | Partial — cross-tenant projection is anonymised at availability layer (free/busy only); full anonymisation incompatible with scheduling purpose. |
| Lawful basis | Per §2.4 |
| Special-category (Art. 9) | pack-us-healthcare: Art. 9(2)(h) (health care provision) + HIPAA BAA. pack-kr Art. 23: PIPA Art. 23(2) explicit consent at flagged-event level. |
| Transfer basis | SCCs only; default residency by pack. |
| Retention | per event/jurisdiction; HIPAA pack ≥ 6y; default 24mo + per-tenant policy override. |
| Subject rights | Art. 15/16/17/18/20/21/22 honoured per §6. |

## Step 5 — Risks to data subjects

| ID | Risk | L | S | Score |
|---|---|---|---|---|
| R-01 | Personal-event content leaks into Professional-context query | M-H | H | **H** |
| R-02 | Cross-tenant availability projection leaks event details | M | H | **H** |
| R-03 | Attendee enumeration (timing / RSVP token) | M | M | **M** |
| R-04 | .ics import / export contains attendee emails beyond minimum-necessary | M | M | **M** |
| R-05 | Room calendar exposes confidential meeting subject | M | M | **M** |
| R-06 | Long retention enables surveillance pattern across years | M | M-H | **M-H** |
| R-07 | Automated recurrence expansion creates load on tenant attendees (notification storm) | L | M | **L-M** |
| R-08 | DSR right-to-erasure incomplete due to recurring + legal-hold overlap | M | M | **M** |
| R-09 | Joint-controllership: tenant doesn't disclose oyatie's processing to end-users | M-H | M | **M-H** |
| R-10 | PHI processed without BAA (pack-us-healthcare tenant doesn't sign BAA but ships clinical events) | M | H | **H** |
| R-11 | Sub-processor breach (Postgres cluster operator / cloud provider) | L | H | **M** |
| R-12 | Cross-border transfer of EU-resident event data via mis-routed CalDAV ingress | L | H | **M** |
| R-13 | Children's-data scheduling (school / hospital tenant) without parental consent | L | H | **M-H** |
| R-14 | Tenant-DEK leaked via log → mass decryption | L | H | **M** |
| R-15 | Auditor mis-pivot across tenants | L | H | **M** |

Cross-reference: every risk has at least one corresponding STRIDE / LINDDUN threat in `threat-model.md`.

## Step 6 — Risk-reducing measures

| Risk | Measures | Mitigated to | Owner |
|---|---|---|---|
| R-01 | Rust type-level Personal vs Professional separation; Cedar `event-isolation.cedar`; LEAN check `oya-check-context-isolation` | L | axis-calendar |
| R-02 | Type-narrowed projection; LEAN check `oya-check-cross-tenant-availability-projection`; annual pen-test | L | ops-security |
| R-03 | Constant-time RSVP response + rate limit + HMAC token | M (timing baseline) | ops-security |
| R-04 | Role-based attendee filtering in export; opt-in for public CalDAV details | L | axis-calendar |
| R-05 | Room calendar default-projection strips subject; Cedar `room-calendar-projection.cedar` | L | axis-calendar |
| R-06 | Aggressive retention defaults; DSR cascade; cold-tier per-event access requires admin JIT | L-M | council-privacy |
| R-07 | RRULE bound (5y horizon); attendee max 1000/event; per-tenant invitation budget | L | axis-calendar |
| R-08 | DSR cascade with legal-hold overlap policy: erasure honoured except where hold; partial-erasure (preserve event minus identifier) where compliant | M (hold-vs-erasure tension is accepted) | council-privacy |
| R-09 | Tenant DPA mandates upstream disclosure; tenant-onboarding checklist verifies | L-M | council-privacy |
| R-10 | pack-us-healthcare onboarding requires BAA pre-ingest; non-signed tenants pre-flighted to non-PHI pack | L | council-privacy |
| R-11 | Sub-processor list at `legal/sub-processors.md`; DPA + SCCs per sub-processor; quarterly review | M (sub-processor risk irreducible) | council-privacy |
| R-12 | Pack-pinning at ingress; route by pack tag; LEAN check refuses cross-pack route | L | axis-calendar |
| R-13 | Tenant DPA includes child-data clause; tenant affirms parental-consent process; calendar does not collect age | L | council-privacy |
| R-14 | Secret-scanner CI lane; `Secret<T>` type strips Debug; 90d rotation; rotation event re-encrypts | M (human-error baseline) | ops-security |
| R-15 | Auditor JIT tokens tenant-scoped at row level; pen-test annually | L | ops-security |

## Step 7 — Sign-off

| Sign-off | Status |
|---|---|
| DPO (council-privacy) | `pending` |
| Information Security Officer (ops-security) | `pending` |
| µservice owner (axis-calendar) | `pending` |
| Council-architecture | `pending` |

**DPO advice:** Residual risks all L or M after mitigations. Art. 36 prior consultation NOT triggered. Proceed with first-tenant onboarding subject to:
- Quarterly review of R-08 (DSR vs hold tension).
- Annual review of this DPIA.
- Re-trigger on each pack activation.

## Per-Pack Overlays

### pack-kr (KR PIPA + ISMS-P)

PIPA Art. 33 + Enforcement Decree Art. 35 mandate 개인정보영향평가. This document fulfils that obligation.

- **PIPA Art. 23 (sensitive)**: per-event sensitivity flag; flagged events carry additional access restrictions.
- **PIPA Art. 23-2 (cross-border sensitive)**: KR-resident sensitive events stay in pack-kr.
- **PIPA Art. 28 (storage period)**: retention bounded per asset table.
- **PIPA Art. 29 (technical safeguards)**: cross-mapped in §6.
- **PIPC Notice 2020-7 methodology**: Steps 1–7 align.
- **PIPA Art. 33-2 (DPO)**: oyatie's council-privacy chair serves as DPO for KR-resident tenants.
- **Korean holidays + lunar calendar**: pack-kr ships locale-specific holiday data; events on public holidays trigger reschedule prompt.

### pack-us-healthcare (HIPAA)

HIPAA §164.308(a)(1)(ii)(A) requires risk-analysis substantially equivalent to a DPIA. This document fulfils that.

- **§164.502(a) Permitted Uses (TPO)**: clinical scheduling falls under Treatment + Operations.
- **§164.502(b) Minimum Necessary**: cross-tenant projection enforces at type level.
- **§164.504(e) BAA**: BAA template at `legal/baa-template.md`.
- **§164.310 Physical Safeguards**: inherited from cloud-k8s + cloud-provider HIPAA-eligibility.
- **§164.312(b) Audit Controls**: Ed25519 audit-chain seal + retention ≥ 6y.
- **§164.404 Notification**: breach chain in `incident-response.md` ≤ 60-day window.
- **45 CFR Part 164 Subpart D**: integrated.
- **FDA 21 CFR Part 11**: when clinical scheduling touches research subjects, audit-chain seal satisfies electronic-signature requirements.

### pack-eu (GDPR + EDPB + NIS2 + eIDAS)

This document is the GDPR Art. 35 DPIA for EU tenant processing.

- **EDPB Guidelines 4/2019 (Art. 25)**: explicit alignment in §4 + §6.
- **EDPB Guidelines 9/2022 (breach notification)**: 72h chain in `incident-response.md`.
- **NIS2**: 24h + 72h + 1mo reporting timelines when thresholds crossed.
- **eIDAS 910/2014**: audit-chain Ed25519 seals satisfy AdES (Art. 26).
- **Schrems II + Arts. 44–46**: SCC-gated transfers only; transfer register kept.
- **Children's data (Art. 8)**: inherited via tenant's age-gating.

### pack-jp (APPI)

APPI voluntary risk-assessment scheme satisfied.

- **APPI Art. 17 (purpose)**: declared at tenant onboarding.
- **APPI Art. 21 (cross-border)**: pack-jp JP-resident.
- **APPI Art. 23 (joint use)**: tenant disclosure obligation.
- **APPI Art. 27 (cross-border consent)**: explicit at onboarding for cross-pack.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack DPIA overlays at `regional-packs/<pack>/calendar-dpia-overlay.md`.

## Re-review Triggers

- Annually (Q2).
- On every new pack activation.
- Change to processing purpose (§2.4) or data-class taxonomy.
- Sub-processor change.
- Breach notification triggered.
- Supervisory-authority guidance change.
- Post-incident (Sev-1 or Sev-2).

## References

- ADR-0028 (Bominal), ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140.
- `microservices/calendar/threat-model.md`, `compliance.md`, `policy/*.cedar`, `multi-region.md`, `incident-response.md`, `legal/{dpa-template,baa-template,sub-processors,transfer-register,ropa}.md`.
- ICO DPIA template; CNIL DPIA methodology; EDPB Guidelines 4/2019 + 9/2022; PIPC Notice 2020-7.
- GDPR Art. 35 + Art. 36; KR PIPA Art. 33; HIPAA 45 CFR §164.308.
- LGPD Art. 38; DPDPA 2023 §10–§11.

## Per-Pack DPIA Overlay Sections (2026-05-17 additive)

These overlays append per ADR-0133 11-pack-overlay program with the
concrete DPIA delta for each pack. Each overlay names the supervisory
authority that governs the pack + the article-level legal basis +
the residual-risk position.

### pack-kr (PIPC oversight)

- **Supervisory authority**: 개인정보보호위원회 (PIPC).
- **Legal basis**: KR PIPA Art. 15(1)(1) consent-based for personal-pillar; Art. 15(1)(4) contract-performance for professional-pillar.
- **Cross-border**: KR PIPA Art. 17 SCC-equivalent gating; per-pack residency at ap-seoul-1.
- **Special-category**: KR PIPA Art. 23 SENSITIVE_PIPA_ART23 data-class for relationship graph; cross-tenant disclosure refused by default.
- **Residual risk**: Low for default operation; Medium for tenant-opted-in cross-tenant invites; mitigated by Cedar audit-chain.
- **DPO notification**: required for any new processing purpose.

### pack-eu (EDPB + national DPA oversight)

- **Supervisory authority**: lead DPA per GDPR Art. 56 one-stop-shop (e.g., CNIL for FR-based tenants).
- **Legal basis**: GDPR Art. 6(1)(b) contract-performance for professional events; Art. 6(1)(a) explicit consent for cross-tenant invite acceptance; Art. 6(1)(f) legitimate interest for service operation.
- **Cross-border**: GDPR Chapter V — SCC + supplementary measures for any cross-pack transfer; default = no cross-pack.
- **EU AI Act**: Annex III §3 employment-context REFUSED at Cedar layer for T1/T2 HR-overlays pending ADR-CAL-XXXX conformity assessment.
- **DPIA Art. 35 trigger**: cross-tenant free/busy + AI scheduling (T1) → trigger; this DPIA satisfies.
- **Residual risk**: Low-Medium; auto-decline (T1) and auto-block focus (T2) reviewed via supervisory-authority engagement.

### pack-us (FTC + state-AG oversight)

- **Supervisory authority**: FTC + state attorneys-general (CA / VA / CO / CT / UT) per state privacy laws.
- **Legal basis**: contract; sectoral laws (CCPA / CPRA / VCDPA / CPA) apply per tenant residency.
- **Cross-border**: SCC for EU-tenant cross-pack; no special restriction for intra-US cross-region.
- **Residual risk**: Low for default; Medium for cross-state cross-tenant invites.

### pack-us-healthcare (HHS OCR oversight)

- **Supervisory authority**: HHS Office for Civil Rights (HIPAA).
- **Legal basis**: HIPAA Privacy Rule; BAA in place for every tenant (per `legal/baa-template.md`).
- **Cross-border**: forbidden by default; ePHI must remain in the US healthcare cluster.
- **Special-category**: PHI data-class on every appointment field; minimum-necessary per HIPAA 45 CFR §164.502(b).
- **DPIA Art. 35 analog**: HIPAA Security Rule Risk Analysis per 45 CFR §164.308(a)(1)(ii)(A) — this DPIA satisfies the equivalent.
- **CalDAV backend**: SabreDAV per ADR-CAL-0001 — higher session ceiling for healthcare-appointment workflows.
- **Residual risk**: Medium; mitigated by per-BAA audit; pen-test annual scope includes healthcare-specific PHI-disclosure simulation.

### pack-jp (PPC oversight)

- **Supervisory authority**: 個人情報保護委員会 (PPC).
- **Legal basis**: APPI Art. 18 consent for cross-tenant; Art. 17 specified-purpose.
- **Cross-border**: APPI Art. 24 — restricted to "adequate" countries; cross-pack to KR / SG / AU requires per-tenant consent.
- **Residual risk**: Low.

### pack-sg (PDPC oversight)

- **Supervisory authority**: Personal Data Protection Commission (PDPC).
- **Legal basis**: PDPA Section 13 consent; deemed-consent for service operation per Section 15.
- **Cross-border**: PDPA Section 26 with comparable protection in destination jurisdiction.
- **Residual risk**: Low.

### pack-au (OAIC oversight)

- **Supervisory authority**: Office of the Australian Information Commissioner (OAIC).
- **Legal basis**: Privacy Act 1988 APP 3 collection limitation; APP 5 notification.
- **Cross-border**: APP 8 accountability for cross-pack transfer.
- **Residual risk**: Low.

### pack-in (DPDPA oversight)

- **Supervisory authority**: Data Protection Board of India (DPBI; once constituted).
- **Legal basis**: DPDPA §6 consent; §7 legitimate-uses.
- **Cross-border**: DPDPA §16 cross-border restrictions; whitelist-based.
- **Residual risk**: Low-Medium (DPBI guidance still emerging at 2026-05).

### pack-br (ANPD oversight)

- **Supervisory authority**: Autoridade Nacional de Proteção de Dados (ANPD).
- **Legal basis**: LGPD Art. 7(I) consent; Art. 7(V) contract.
- **Cross-border**: LGPD Art. 33 — ANPD-approved transfer mechanism.
- **Residual risk**: Low.

### pack-ae (UAE DPA oversight)

- **Supervisory authority**: UAE Data Office (under Federal PDPL).
- **Legal basis**: PDPL Art. 5 consent; contract.
- **Cross-border**: PDPL Art. 22 — UAE DPA approval required.
- **Hijri calendar**: special-purpose tz/calendar overlay; DPIA scope unchanged.
- **Residual risk**: Low.

### pack-ksa (SDAIA oversight)

- **Supervisory authority**: Saudi Data and AI Authority (SDAIA).
- **Legal basis**: PDPL Art. 6 lawful processing; Art. 7 consent.
- **Cross-border**: PDPL Art. 29 — SDAIA-approved mechanism.
- **Hijri calendar**: as pack-ae.
- **Sharia retention**: per-tenant retention extension supported; refusal of premature deletion logged in audit-chain.
- **Residual risk**: Low-Medium.
