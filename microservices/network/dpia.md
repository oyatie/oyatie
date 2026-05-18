---
doc_class: DPIA
template_id: TPL-DPIA
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-network
deciders: council-privacy, ops-security, axis-network, council-architecture, ops-compliance, ops-legal
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33 + EDPB Guidelines 4/2019 + EU AI Act FRIA (fundamental-rights-impact-assessment)
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-NET-0001, ADR-NET-0002, ADR-NET-0003, ADR-NET-0004, ADR-NET-0005, ADR-NET-0006]
related_specs: [/specs/microservices/network.json]
related_artifacts:
  - microservices/network/threat-model.md
  - microservices/network/policy/professional-context-isolation.md
  - microservices/network/policy/data-residency.md
  - microservices/network/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, sub-processor list, classifier / recommender / recruiter ranker version, or recruiter-stub activation per tenant
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (recommender + recruiter ranker + endorsement aggregation form profiling in employment context)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (employment records, relationship graphs under Art. 9 interpretations; potential PHI in pack-us-healthcare)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — PARTIAL (public Professional profiles are publicly accessible; engagement metrics systematically collected)"
  - "EU AI Act FRIA mandatory: HIGH-RISK system per Annex III §4 (employment, workers management, access to self-employment)"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 8, 9, 13, 14, 17, 21, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII), A.5.31 (legal/statutory)"
  - "ISO 30414:2018 (HR analytics + workforce reporting)"
  - "SOC 2 Privacy criteria (P1-P8)"
  - "EU DSA 2065/2022 Arts. 14, 16, 17, 20, 23, 24, 27, 28"
  - "EU AI Act 2024/1689 Arts. 9, 10, 11, 13, 14, 15, 27, 50, 52, 73 (Annex III §4 — employment, workers management, access to self-employment)"
  - "EU Equal Treatment Directives 2000/43/EC (racial / ethnic origin) + 2000/78/EC (employment)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33", "PIPA Enforcement Decree Art. 35", "PIPC Notice 2020-7", "KR 근로기준법", "KR 직장 갑질 protections", "KR 통신비밀보호법", "KR 채용절차의 공정화에 관한 법률"]
  pack-us: ["EEOC UGESP 1978", "Title VII Civil Rights Act 1964", "ADA 1990", "ADEA 1967", "OFCCP regulations", "CCPA + CPRA", "NYC AI Hiring Law (Local Law 144-2021)", "CA AB-331", "CO SB-205", "IL AI Video Interview Act"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A)", "§164.312(b)", "§164.502(b)", "§164.514"]
  pack-eu: ["GDPR Arts. 21 + 22 + 35 + 36", "EDPB Guidelines 4/2019 + 5/2021 (automated decisions)", "EU DSA + AI Act", "EU Equal Treatment Directives", "UK Equality Act 2010 + ICO ADM guidance", "Council of Europe Convention 108+"]
  pack-jp: ["APPI Arts. 17, 18, 27", "JP 労働基準法", "JP 労働契約法"]
  pack-sg: ["PDPA Part III + IV", "PDPC Employment guidance"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12", "AHRC AI guidance", "Fair Work Act 2009"]
  pack-in: ["DPDPA 2023 §9 + §10 + §11"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 38", "Brazilian CLT"]
  pack-ae: ["UAE PDPL Art. 23", "Federal Decree-Law 33/2021 (Labour)"]
  pack-ksa: ["PDPL Art. 9", "KSA Labor Law"]
doc_status: published
---

# Data Protection Impact Assessment: network µservice

## Step 1 — Identify the need for a DPIA + FRIA

GDPR Art. 35(1) requires DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. The network µservice engages multiple triggers, and EU AI Act 2024/1689 Art. 27 + Annex III §4 mandates a fundamental-rights-impact-assessment (FRIA) for high-risk employment-context AI systems. This document satisfies both DPIA + FRIA obligations.

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic profiling | **YES** | Recommender + recruiter ranker + endorsement aggregation are profiling per GDPR + EU AI Act high-risk Annex III §4 |
| Art. 35(3)(b): Large-scale special-category | **YES** | Employment records (network-specific data class); relationship graphs (Art. 9 interpretations); potential PHI in pack-us-healthcare |
| Art. 35(3)(c): Public-area monitoring | **PARTIAL** | Public Professional profiles are publicly accessible; engagement metrics systematically collected |
| EU AI Act FRIA (Annex III §4 — employment) | **YES (mandatory)** | Recruiter + jobs ranker + endorsement aggregation when used for employment intent qualify under "employment, workers management, access to self-employment" |

KR PIPC's Notice 2020-7 mandates DPIA when processing handles sensitive personal information at scale — engaged when first pack-kr enterprise tenant exceeds threshold. NYC Local Law 144-2021 mandates annual bias-audit on AEDTs (automated employment decision tools); CA AB-331 mandates impact assessment for ADS (automated decision systems) in employment.

DPIA + FRIA are mandatory pre-deployment. This document is the canonical DPIA + FRIA reviewed by EU DPAs (Art. 35), EU AI Act notified body (Art. 27), KR PIPC (PIPA Art. 33), HIPAA OCR (post-BAA), EU DSA Coordinator (Art. 24), EEOC + OFCCP (US), NYC Department of Consumer and Worker Protection (when NYC tenant activates recruiter-stub), and equivalent supervisory authorities per active pack.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** End-users create Professional profiles (resume + skills + certifications + education), connect / follow other Professionals, publish posts (article + status + document + poll + carousel), react, comment, endorse skills, write long-form recommendations, take skill-assessment quizzes, request profile verification (ID-attest + employer-confirm), send InMails (premium messenger-bridge), follow Pages, create Groups, schedule Events (with calendar bridge), post / apply to Jobs (with ATS handoff), invoke recruiter-search ranker (when tenant-enabled).

**How:** Client → ingress (TLS/WAF) → WebSocket gateway + REST → BC services (REST + worker) → Postgres (profiles, posts, graph, endorsements, recommendations, pages, groups, events, jobs, recruiter-audit) + Valkey (feed cache, reactions, trending, notifications, InMail rate-budget) + S3 (media + documents) + Meilisearch (search; faceted) + audit-chain seal (per-state-transition + per-endorser Ed25519 chain) + foundry-runtime (recommender + recruiter ranker; HIGH-RISK per Annex III §4) + messenger-bridge (InMail) + calendar-bridge (events) + mail-bridge (Page newsletter) + ats-bridge (jobs handoff).

**Where:** Per-pack region-pinned network clusters. pack-kr (KR), pack-eu (EU), pack-us (US), pack-us-healthcare (US, HIPAA-eligible), and conditional packs.

**When:** Continuous; sub-second latency for delivery; 60s cadence for SLO evaluation; 5-min windows for trending compute; per-release bias-audit + per-invocation bias-audit on recruiter-search.

**Who:** End-users (Professionals); tenant operators (admins, compliance officers, moderators, recruiters when activated, Page admins, Group admins, security admins); oyatie operators; ontology + messenger + calendar + mail + ATS + workflow-engine + audit-chain + foundry-runtime µservices (machine actors).

### 2.2 Scope of the processing

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `PII_IDENTIFYING` | Profile handle, display-name, headline, contact, summary, locale | Art. 6(1)(b) contract + Art. 6(1)(a) consent (for public visibility) | per profile |
| `EMPLOYMENT_RECORD` (network-specific) | Resume sections, endorsements, recommendations, skill-assessments, certifications, education | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest (Professional networking) + Art. 9(2)(b) (when employment-context special-category) | per profile + per endorsement + per recommendation |
| `PII_QUASI_IDENTIFIER` | IPs, user-agents, geo-derived hints, connection-graph degree, mention-graph | Art. 6(1)(f) (minimised at SDK) | per-session |
| `BEHAVIORAL_TENANT_PRODUCT` | Posts, comments, reactions, connections, pages, groups, events, jobs | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest | ~10⁵ posts/day per medium tenant |
| `RELATIONSHIP_GRAPH` | Connection / follow / block / restrict edges; arguable Art. 9 sensitivity depending on community | Art. 6(1)(b) contract; user-explicit | varies |
| `SENSITIVE_VERIFICATION` | ID-attest artifacts (national ID, passport scan); employer-confirm tokens | Art. 6(1)(a) explicit consent + Art. 9(2)(a) explicit consent for biometric | per verification request |
| `SENSITIVE_PIPA_ART23` | Sensitive profile fields (race, religion, health, political views) — pack-kr | KR PIPA Art. 15 + 23 + explicit consent | varies |
| `PHI` (pack-us-healthcare) | Health-context content in posts or media | HIPAA §164.502 Permitted Uses (under BAA) | targeted to 0 via redactor where possible |
| `AUDIT` | Profile / post / connection / endorsement / recommendation / InMail / jobs-handoff / recruiter-invocation / bias-audit / Art. 22 opt-out events | Art. 6(1)(c) | 1 record per state transition |
| `SECRET` | Per-tenant DEK, per-endorser signing keys, session tokens, federation peer keys | not personal data | OpenBao-bound |

**Geographical scope:** Per pack-pinning (data-residency.md).

**Cross-border transfer:** Forbidden by default; allowed only with tenant SCCs (Arts. 44–46). `network` does NOT federate in P01.

### 2.3 Context of the processing

- **Data subjects:** End-users of tenant applications (Professional end-users; never minors — Professional network is 18+); tenant operators; oyatie operators; non-users referenced in posts (e.g., subjects of recommendations).
- **Relationship:** Joint controllership with tenant under Art. 26.
- **Reasonable expectations:** End-users expect public-by-default Professional profiles + tenant-admin moderation under disclosed policy + recruiter access (when activated) under candidate-notice obligations.
- **Previous experience:** Bominal `connect-network` predecessor; no DPA-triggered complaints in 24 months on the network slice; **network as a standalone µservice is NET-NEW** in oyatie per ADR-0135.
- **Industry codes:** EU DSA + EU AI Act + EEOC UGESP + NYC LL 144 + CA AB-331 + CO SB-205 for employment-context AI transparency.
- **Children:** Professional network is 18+; minor accounts blocked at signup.

## Step 3 — Consultation

- Internal: council-privacy (Q2 quarterly review), ops-security, council-architecture, axis-network lead, axis-foundry-runtime (recommender + recruiter ranker owner), ops-legal (employment-law overlay), ops-compliance (bias-audit cadence).
- External: tenant pilot focus groups (3 pack-kr + 2 pack-eu + 2 pack-us enterprise tenants) confirmed expectations re: recruiter transparency + endorsement-chain integrity + GDPR Art. 22 opt-out path.
- Supervisory authority: KR PIPC notified at pack-kr first-tenant onboarding; EU DPA + EU AI Act notified-body notified at pack-eu first-tenant signature; EU DSA Coordinator notified per Art. 24 transparency reporting cadence; EEOC + OFCCP + NYC DCWP notified at pack-us first-tenant signature with recruiter-stub activation.

## Step 4 — Necessity & proportionality

### 4.1 Necessity

- **Purpose limitation:** Each data class processed only for its declared purpose (Professional networking + endorsements + recommendations + jobs-handoff + recruiter-search when activated). Cross-purpose use (e.g., marketing, ads) is OFF by default; requires fresh consent + tenant-admin opt-in.
- **Minimisation:** OTel redactor + `data_class` annotation enforcement; media OCR redactor; search-result Cedar filter; recruiter-search scope bounded to tenant.
- **Accuracy:** Edit-window allows author corrections; admin-disclosure record corrections audit-chain-immutable; endorsement revocation flow allows correction of incorrect endorsements.
- **Storage limitation:** Per-pack retention bounds in `policy/data-residency.md`; recruiter-audit ≥ 6y for OFCCP / EEOC retention floor.

### 4.2 Proportionality

- Less-intrusive alternatives considered: chronological-only feed (rejected — destroys discovery utility); no-recruiter-search (rejected — competitive disadvantage; mitigated to OFF-by-default + activation gates); no-endorsement-aggregation in ranking (scheduled-for-distinct-tracked-work to PRD Open Question 4 — under review).
- Selected: hybrid chronological + heuristic-algorithmic feed with user choice; recruiter-stub OFF-by-default with activation pre-condition; endorsement aggregation display-only in P01 (ranking-impact scheduled-for-distinct-tracked-work to ADR-NET successor-IP).

### 4.3 EU AI Act FRIA (fundamental-rights-impact-assessment)

Per EU AI Act Art. 27 (deployment of high-risk AI systems by deployers; deployer = oyatie + tenant jointly), the FRIA must address:

| Dimension | Assessment |
|---|---|
| Purpose | Recruiter-search ranker + jobs ranker + endorsement aggregation used for employment decision-making |
| Scope of use | Tenant-scoped only; never cross-tenant; OFF by default for recruiter-stub |
| Affected groups | Job-seekers (all genders, ages, races, abilities) + protected classes under EEOC + Equal Treatment Directives |
| Likely impact on fundamental rights | Risk of disparate impact on protected groups; risk of opacity (right to explanation); risk of cascading impact (rejected candidate at one tenant may be cascaded if endorsement signals propagate) |
| Mitigations | Bias-audit per release (4/5-rule statistical bound); Art. 22 opt-out per user; Art. 50 transparency label per decision; Art. 27 recommender explanation API; activation pre-condition gates (NYC LL 144 + CA AB-331 + CO SB-205); 4-axis fairness dashboard published |
| Residual | M — bias-audit cannot eliminate disparate impact; transparency + opt-out + human-review path provide redress |
| Notification | Candidate-notice obligation per NYC LL 144 (NYC) + CA AB-331 (CA) + CO SB-205 (CO); tenant DPA includes notice templates |

## Step 5 — Risks to data subjects

| Risk ID | Risk | Likelihood | Severity | Risk score |
|---|---|---|---|---|
| R-01 | Cross-tenant profile / post leak (RLS misconfig) | M | H | High |
| R-02 | PHI leak in post/media (pack-us-healthcare) | L | H | Medium |
| R-03 | Employment-record leaked to tenant-admin pivot | M | H | High |
| R-04 | Connection-graph relationship leak (Art. 9 sensitive interpretation) | M | M | Medium |
| R-05 | Search over-permitted result | M | H | High |
| R-06 | Media / document URL shared-link guess | M | H | High |
| R-07 | Cross-context routing (Personal `social` post → Professional `network` context) | L | H | Medium |
| R-08 | InMail body leaked beyond messenger µservice scope | L | H | Medium |
| R-09 | Mention-graph identity correlation across `social` + `network` (linkability) | M | M | Medium |
| R-10 | Erasure right-best-effort due to retention floors (esp. recruiter-audit ≥ 6y for OFCCP) | M | M | Medium |
| R-11 | Admin-disclosure inherent exposure of Professional posts + InMail | L | H | Medium |
| R-12 | Cross-pack residency misroute | L | H | Medium |
| R-13 | Recruiter-search ranker discrimination (EU AI Act + EEOC disparate impact) | M | H | High |
| R-14 | Jobs ranker discrimination (EU AI Act + EEOC + Equal Treatment Directives) | M | H | High |
| R-15 | Endorsement aggregation amplifies bias when ranking-impacting | M | H | High (scheduled-for-distinct-tracked-work via PRD OQ4) |
| R-16 | Profile-verification artifact (ID-attest) leak | L | H | Medium |
| R-17 | Sybil amplification distorts trending → manipulates Professional discourse | M | M | Medium |
| R-18 | Engagement-metric leak (endorsement history, view counts) | M | M | Medium |
| R-19 | Forged endorsement (signing-key compromise) | L | H | Medium |
| R-20 | Salary-insights de-anonymises individual at small-cell | M | M | Medium |
| R-21 | Job-search activity disclosed to current employer (linkability T-L-04) | M | M | Medium |
| R-22 | Recruiter-stub activated on tenant without NYC LL 144 / CA AB-331 pre-condition | L | H | Medium |
| R-23 | Bias-audit threshold misconfigured (false-pass on 4/5-rule) | L | H | Medium |
| R-24 | Jobs-handoff event leaks candidate PII to ATS beyond consented scope | M | H | High |
| R-25 | GDPR Art. 22 opt-out not honored (recommender continues to profile opted-out user) | L | H | Medium |
| R-26 | KR 직장 갑질 abuse report buried in general queue | M | M | Medium |

## Step 6 — Mitigations

| Risk ID | Mitigation | Residual |
|---|---|---|
| R-01 | Postgres RLS + Cedar + pen-test annual | L |
| R-02 | Pack-us-healthcare disables PHI-context features by default; OCR-redactor; access bound to post ACL | L |
| R-03 | Cedar `tenant-scope.cedar` bounds tenant-admin reads to consented scope; per-user privacy settings honored; LEAN lane | L |
| R-04 | Graph reads bounded by Cedar; per-tenant cardinality limits; aggregate-only for non-owners | L |
| R-05 | Cedar post-filter on every search result; integration test asserts no over-permit | L |
| R-06 | Signed short-TTL URLs; per-fetch Cedar re-eval; public posts use Cedar-checked CDN URL | L |
| R-07 | Data-model type invariant (`network` types are Professional-only; no shared type with `social`); LEAN lane | L |
| R-08 | InMail body lives at messenger µservice; network holds only routing metadata; four-eyes + audit-chain at both µservices | L |
| R-09 | Per-µservice ontology scope; cross-µservice correlation forbidden via Ontology Cedar | L |
| R-10 | DSR cascade marks tombstoned + redacts identifiers; recruiter-audit retention floor disclosed in DPA; chain redaction replaces identifier with `«erased»` | M |
| R-11 | Four-eyes + audit-chain + tenant onboarding disclosure | M |
| R-12 | Pack-router Cedar enforces; CI lane validates Helm pack-pinning | L |
| R-13 | EU AI Act Art. 9-15 + 27 + 50 risk-management; per-release bias-audit (4/5-rule); per-invocation transparency; Art. 22 opt-out; appeal workflow | M (residual unavoidable in any ranking system; mitigated by transparency + opt-out + appeal) |
| R-14 | Same as R-13; jobs ranker bound to same bias-audit lane | M |
| R-15 | P01: endorsement aggregation is display-only; ranking-impact scheduled-for-distinct-tracked-work to ADR-NET successor-IP per PRD OQ4 | L (P01) |
| R-16 | Separate ID-attest table; Cedar-restricted access; encryption at rest; LEAN lane | L |
| R-17 | foundry-guardrails sybil detector; per-author influence cap in trending; tenant-admin pin/unpin | M |
| R-18 | Endorsement / view counts: per-user opt-in for public history; default private to non-followers | L |
| R-19 | Per-endorser Ed25519 key bound to OpenBao + device; revocation flow; replay-resistant nonce | L |
| R-20 | k-anonymity ≥ 5 enforced; aggregate-only; LEAN lane | L |
| R-21 | Best-effort policy: job-search-related signals omitted from current-employer Page admin scope | M |
| R-22 | Activation pre-condition lane refuses recruiter-stub activation without NYC LL 144 + CA AB-331 + CO SB-205 attestation on file | L |
| R-23 | Bias-audit threshold attested per release; tampering audit-chain-sealed; quarterly council-privacy review | L |
| R-24 | Handoff payload scoped to minimum-necessary; contract-versioned per ADR-NET-0004; ATS µservice attests scope at receipt | L |
| R-25 | Opt-out record signed + audit-chain sealed; recommender invocation checks opt-out before profiling; LEAN lane | L |
| R-26 | Dedicated `harassment-workplace` abuse category; elevated severity; tenant ops-security notified within 1h | L |

## Step 7 — Sign-off

- council-privacy chair: `pending`
- ops-security director: `pending`
- council-architecture chair: `pending`
- ops-compliance director: `pending`
- ops-legal director: `pending`
- axis-network lead: `pending`

## Per-pack overlays

### pack-kr

- KR PIPA Art. 23 sensitive data — additional consent at signup for sensitive-context profile fields.
- PIPC Notice 2020-7 — this DPIA satisfies impact-assessment requirement at scale.
- KR 근로기준법 — employment record retention floor.
- KR 직장 갑질 protections — dedicated harassment-workplace abuse category.
- KR 통신비밀보호법 — InMail intercept only via four-eyes audit.
- KR 채용절차의 공정화에 관한 법률 — recruiter-stub activation requires candidate-notice + bias-audit on file.
- KR PIPA Art. 28 — outside-of-KR transfer forbidden by default.

### pack-us

- EEOC UGESP 1978 + Title VII + ADA + ADEA — disparate-impact monitoring per release; 4/5-rule statistical bound; protected-group audit per `dashboards/recommender-fairness-and-bias.json`.
- OFCCP — when federal contractor tenant: recruiter-audit ≥ 6y retention.
- CCPA + CPRA — California opt-out + right-to-delete cascaded.
- NYC AI Hiring Law (Local Law 144-2021) — annual bias audit + candidate notice when recruiter-tooling activated; recruiter-stub activation pre-condition lane gates.
- CA AB-331 — automated-decision transparency obligations; pre-condition gate.
- CO SB-205 — Colorado AI Act developer + deployer obligations; pre-condition gate.
- IL AI Video Interview Act — video-interview features (if added later) require consent + storage limits.

### pack-us-healthcare

- HIPAA §164.308(a)(1)(ii)(A) risk-analysis — this DPIA satisfies.
- HIPAA §164.502(b) minimum-necessary — Professional posts that surface health-context default to PHI-redactor.
- Per-tenant BAA at `legal/baa-template.md` (Slice B).

### pack-eu

- GDPR Art. 21 — right to object to profiling surfaced per-decision.
- GDPR Art. 22 — right to human review of automated decisions surfaced per recommender + recruiter + jobs ranker invocation.
- GDPR Art. 35 prior consultation — required when DPIA indicates residual high risk; section above shows residual ≤ M for almost all rows; R-13/R-14 (ranker fairness) residual = M acceptable with mitigation evidence + post-deployment monitoring.
- EU DSA Arts. 14, 16, 17, 20, 23, 24, 27, 28 — transparency + appeal + Statement of Reasons.
- EU AI Act 2024/1689 Annex III §4 — recruiter + jobs ranker + endorsement aggregation HIGH-RISK; Arts. 9-15 risk-management + Art. 27 recommender transparency + Art. 50 transparency + Art. 73 serious incident reporting operative.
- EU Equal Treatment Directives 2000/43/EC + 2000/78/EC — disparate-impact monitoring per Art. 9 racial/ethnic + Art. 1 employment.
- UK Equality Act 2010 + ICO ADM guidance — UK-tenant overlay.
- Council of Europe Convention 108+ — broader processing protections.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack overlays at `regional-packs/<pack>/network-dpia-overlay.md`.

## DPIA + FRIA refresh triggers

- Any new data class.
- Any new sub-processor.
- Any change to Professional-context invariant.
- Any new pack activation.
- Any new classifier / recommender / recruiter ranker version (EU AI Act re-evaluation).
- Any recruiter-stub activation on a new tenant (NYC LL 144 + CA AB-331 + CO SB-205 cross-check + FRIA refresh).
- Annual scheduled review.
- Post-incident review.
- Bias-audit threshold change.
- Any change to endorsement-chain integrity invariant (ADR-NET-0005).

## References

- `microservices/network/threat-model.md`.
- `microservices/network/policy/professional-context-isolation.md`.
- `microservices/network/policy/data-residency.md`.
- `microservices/network/compliance.md`.
- `microservices/network/decisions/ADR-NET-0002-recommender-ai-act-eeoc-bounds.md`.
- Bominal ADR-0208 + ADR-0215.
- Parallel ADR-0135.
- GDPR + KR PIPA + HIPAA + APPI + LGPD + PDPA full citations.
- EU DSA 2065/2022; EU AI Act 2024/1689 Annex III §4; EU Equal Treatment Directives 2000/43/EC + 2000/78/EC.
- EEOC UGESP 1978; Title VII Civil Rights Act 1964; ADA 1990; ADEA 1967; OFCCP.
- NYC Local Law 144-2021; CA AB-331; CO SB-205; IL AI Video Interview Act.
- KR PIPA + 근로기준법 + 직장 갑질 + 통신비밀보호법 + 채용절차의 공정화에 관한 법률.
- ISO 30414:2018 (HR analytics).
