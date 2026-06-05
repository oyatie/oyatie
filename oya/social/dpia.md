---
doc_class: DPIA
template_id: TPL-DPIA
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-social
deciders: council-privacy, ops-security, axis-social, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/microservices/social.json]
related_artifacts:
  - microservices/social/threat-model.md
  - microservices/social/policy/dual-context-isolation.md
  - microservices/social/policy/data-residency.md
  - microservices/social/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, sub-processor list, or classifier version
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (algorithmic feed-ranking + content-moderation classifier form profiling)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (PHI possible in pack-us-healthcare; sensitive data under PIPA Art. 23; relationship graphs under Art. 9 in some interpretations)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — PARTIAL (public-by-default posts are publicly accessible)"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 8, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII), A.5.31 (legal/statutory)"
  - "SOC 2 Privacy criteria (P1-P8)"
  - "EU DSA 2065/2022 Arts. 14, 16, 17, 20, 23, 24, 27, 28"
  - "EU AI Act 2024/1689 Arts. 9, 10, 11, 13, 14, 15, 50, 52"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 8 (child), 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33", "PIPA Enforcement Decree Art. 35", "PIPC Notice 2020-7", "KR 청소년 보호법"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A)", "§164.312(b)", "§164.502(b)", "§164.514", "US COPPA 15 USC §6501"]
  pack-eu: ["GDPR Art. 8 (child consent)", "GDPR Arts. 35 + 36", "EDPB Guidelines 4/2019", "EDPB Guidelines 9/2022", "EU DSA + AI Act", "UK Online Safety Act 2023"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Part III + IV", "Online Safety (Miscellaneous Amendments) Act 2022"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12", "Online Safety Act 2021"]
  pack-in: ["DPDPA 2023 §9 + §10 + §11"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 14 (child) + 38"]
  pack-ae: ["UAE PDPL Art. 23"]
  pack-ksa: ["PDPL Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: social µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. The social µservice engages multiple triggers:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic profiling | **YES** | Algorithmic feed-ranking + content-moderation classifier are profiling per GDPR + EU AI Act high-risk |
| Art. 35(3)(b): Large-scale special-category | **YES (conditional)** | Pack-us-healthcare PHI in posts + media; pack-kr sensitive data under PIPA Art. 23; relationship graphs are sensitive |
| Art. 35(3)(c): Public-area monitoring | **PARTIAL** | Public-by-default posts are publicly accessible; engagement metrics are systematically collected |

KR PIPC's Notice 2020-7 mandates DPIA when processing handles sensitive personal information at scale — engaged when first pack-kr enterprise tenant exceeds threshold.

DPIA is mandatory pre-deployment. This document is the canonical DPIA reviewed by EU DPAs (Art. 35), KR PIPC (PIPA Art. 33), HIPAA OCR (post-BAA), EU DSA Coordinator (Art. 24), and EU AI Act notified body at first-tenant onboarding per pack.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** End-users create profiles, publish posts (text + media), follow other users, react, comment, mention people / topics. The system stores, indexes, fans out via real-time delivery, surfaces algorithmic + chronological feeds, and applies content-moderation classifier.

**How:** Client → ingress (TLS/WAF) → WebSocket gateway + REST → BC services (REST + worker) → Postgres (profiles, posts, follows, moderation) + Valkey (feed cache, reactions, trending, notifications) + S3 (media) + Meilisearch (search) + audit-chain seal + foundry-runtime (classifier + ranking).

**Where:** Per-pack region-pinned social clusters. pack-kr (KR), pack-eu (EU), pack-us (US), pack-us-healthcare (US, HIPAA-eligible), and conditional packs.

**When:** Continuous; sub-second latency for delivery; 60s cadence for SLO evaluation; 5-min windows for trending compute.

**Who:** End-users (tenants' customers); tenant operators; oyatie operators; ontology + messenger + workflow-engine + audit-chain + foundry-runtime µservices (machine actors); ActivityPub federation peers (Professional-tier only, opt-in).

### 2.2 Scope of the processing

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `PII_IDENTIFYING` | Profile handle, display-name, bio, avatar, post-author identity | Art. 6(1)(b) contract + Art. 6(1)(a) consent (for public visibility) | per profile + every post |
| `PII_QUASI_IDENTIFIER` | IPs, user-agents, geo-derived hints, follow-graph degree | Art. 6(1)(f) (minimised at SDK) | per-session |
| `BEHAVIORAL_TENANT_PRODUCT` | Posts, comments, reactions, follows, lists, bookmarks (Professional-tier) | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest | ~10⁶ posts/day per medium tenant |
| `BEHAVIORAL_USER_CONTENT` | Posts, comments, reactions, follows (Personal-tier) | Art. 6(1)(b) personal contract + Art. 6(1)(a) consent | ~10⁵ posts/day per medium tenant |
| `RELATIONSHIP_GRAPH` | Follow / block / mute edges; arguable Art. 9 sensitivity depending on community | Art. 6(1)(b) contract; user-explicit | varies |
| `SENSITIVE_PIPA_ART23` | Sensitive posts (medical, juvenile, biometric) — pack-kr | KR PIPA Art. 15 + 23 + explicit consent | varies |
| `SENSITIVE_CHILD_PROTECTION` | Minor-account flag + parental consent record | GDPR Art. 8 + COPPA + LGPD Art. 14 | per minor signup |
| `PHI` (pack-us-healthcare) | Patient identifiers / clinical content in posts or media | HIPAA §164.502 Permitted Uses (under BAA) | targeted to 0 via redactor where possible |
| `AUDIT` | Post-create / follow-add / verdict / appeal / disclosure events | Art. 6(1)(c) | 1 record per state transition |
| `SECRET` | Per-tenant DEK, session tokens, signing keys, federation peer keys | not personal data | OpenBao-bound |

**Geographical scope:** Per pack-pinning (data-residency.md).

**Cross-border transfer:** Forbidden by default; allowed only with tenant SCCs (Arts. 44–46) + multi-region.md. Federation egress is per-tenant opt-in for Professional-tier only.

### 2.3 Context of the processing

- **Data subjects:** End-users of tenant applications (consumer users of B2C tenants; employee users of B2B tenants); tenant operators; oyatie operators; non-users referenced in posts (e.g., subjects of journalism).
- **Relationship:** Joint controllership with tenant under Art. 26.
- **Reasonable expectations:** End-users expect public-by-default semantics + tenant-admin moderation under disclosed policy.
- **Previous experience:** Bominal `community-social` predecessor; no DPA-triggered complaints in 24 months on the social slice; **social as a standalone µservice is NET-NEW** in oyatie per ADR-0135.
- **Industry codes:** EU DSA + UK Online Safety Act + AU Online Safety Act for content moderation transparency.
- **Children:** all packs apply pack-aware child consent threshold (GDPR Art. 8 default 16y; member states may lower to 13y; COPPA US <13; KR <14; LGPD <12).

## Step 3 — Consultation

- Internal: council-privacy (Q2 quarterly review), ops-security, council-architecture, axis-social lead, axis-foundry-runtime (classifier owner).
- External: tenant pilot focus groups (3 pack-kr + 2 pack-eu enterprise tenants) confirmed expectations re: feed-ranking transparency + moderation appeal.
- Supervisory authority: KR PIPC notified at pack-kr first-tenant onboarding; EU DPA notified at pack-eu first-tenant signature; EU DSA Coordinator notified per Art. 24 transparency reporting cadence.

## Step 4 — Necessity & proportionality

### 4.1 Necessity

- **Purpose limitation:** Each data class processed only for its declared purpose (feed delivery + search + moderation + notifications). Cross-purpose use (e.g., marketing, ads-substrate) requires fresh consent + tenant-admin opt-in.
- **Minimisation:** OTel redactor + `data_class` annotation enforcement; media OCR redactor; search-result Cedar filter.
- **Accuracy:** Edit-window allows author corrections; admin-disclosure record corrections audit-chain-immutable.
- **Storage limitation:** Per-pack retention bounds in `policy/data-residency.md`.

### 4.2 Proportionality

- Less-intrusive alternatives considered: chronological-only (rejected — destroys discovery utility); no-search (rejected — destroys user utility); no-moderation (rejected — leaves users exposed to abuse).
- Selected: hybrid chronological + algorithmic feed with user choice; moderation classifier with appeal workflow; search Cedar-filtered.

## Step 5 — Risks to data subjects

| Risk ID | Risk | Likelihood | Severity | Risk score |
|---|---|---|---|---|
| R-01 | Cross-tenant post leak (RLS misconfig) | M | H | High |
| R-02 | PHI leak in post/media (pack-us-healthcare) | M | H | High |
| R-03 | Personal-tier profile leaked to tenant-admin pivot | M | H | High |
| R-04 | Follow-graph relationship leak (Art. 9 sensitive interpretation) | M | M | Medium |
| R-05 | Search over-permitted result | M | H | High |
| R-06 | Media URL shared-link guess | M | H | High |
| R-07 | Cross-context routing (Personal post → Professional context) | L | H | Medium |
| R-08 | Federation egress (Personal-tier accidentally federates) | L | H | Medium |
| R-09 | Mention-graph identity correlation (linkability) | M | M | Medium |
| R-10 | Erasure right-best-effort due to retention floors | M | M | Medium |
| R-11 | Admin-disclosure inherent exposure of Professional posts | L | H | Medium |
| R-12 | Cross-pack residency misroute | L | H | Medium |
| R-13 | Algorithmic ranking discrimination (EU AI Act high-risk) | M | H | High |
| R-14 | Content-moderation classifier false-positive (free-speech impact) | M | M | Medium |
| R-15 | Minor age-attestation pivot (minor-list leak) | L | H | Medium |
| R-16 | Sybil amplification distorts trending → manipulates public discourse | M | M | Medium |
| R-17 | Federation peer compromise (untrusted peer ingestion) | L | M | Low |
| R-18 | Engagement-metric leak (reaction history, view counts) | M | M | Medium |

## Step 6 — Mitigations

| Risk ID | Mitigation | Residual |
|---|---|---|
| R-01 | Postgres RLS + Cedar + pen-test annual | L |
| R-02 | Pack-us-healthcare disables federation; OCR-redactor; access bound to post ACL | L–M |
| R-03 | Cedar `tenant-scope.cedar` blocks tenant-admin reads of Personal-context resources | L |
| R-04 | Follow-graph reads bounded by Cedar; per-tenant cardinality limits; aggregate-only for non-owners | L |
| R-05 | Cedar post-filter on every search result; integration test asserts no over-permit | L |
| R-06 | Signed short-TTL URLs; per-fetch Cedar re-eval; public posts use Cedar-checked CDN URL | L |
| R-07 | Data-model type invariant (PersonalProfile ≠ ProfessionalProfile); LEAN lane | L |
| R-08 | Compile-time type-system invariant (federation outbox accepts only ProfessionalPost); LEAN lane | L |
| R-09 | Per-tenant mention scope; no cross-tenant linkability | L |
| R-10 | DSR cascade marks tombstoned + redacts identifiers; user-side disclosure of retention floor | M |
| R-11 | Four-eyes + audit-chain + tenant onboarding disclosure | M |
| R-12 | Pack-router Cedar enforces; CI lane validates Helm pack-pinning | L |
| R-13 | EU AI Act Art. 9-15 risk-management; per-classifier evaluation set; Art. 50 transparency label; bias-audit pipeline; appeal workflow | M (residual unavoidable in any ranking system; mitigated by transparency + appeal) |
| R-14 | Appeal workflow within 7 days; human reviewer on appeal; per-tenant override; classifier version evidence record + reference-set eval per release | M |
| R-15 | Separate age-attestations table; Cedar-restricted access; LEAN lane verifies isolation | L |
| R-16 | foundry-guardrails sybil detector; per-author influence cap in trending; tenant-admin pin/unpin | M |
| R-17 | Peer allowlist; HTTP Signature verification; per-peer rate limit | L |
| R-18 | Per-user opt-in for public reaction list; default private to non-followers | L |

## Step 7 — Sign-off

- council-privacy chair: `pending`
- ops-security director: `pending`
- council-architecture chair: `pending`
- axis-social lead: `pending`

## Per-pack overlays

### pack-kr

- KR PIPA Art. 23 sensitive data — additional consent at signup for sensitive-context profile.
- PIPC Notice 2020-7 — this DPIA satisfies impact-assessment requirement at scale.
- KR PIPA Art. 8 + 청소년 보호법 — minor signup requires parental consent attestation.
- KR PIPA Art. 28 — outside-of-KR transfer forbidden by default.

### pack-us-healthcare

- HIPAA §164.308(a)(1)(ii)(A) risk-analysis — this DPIA satisfies.
- HIPAA §164.502(b) minimum-necessary — search + media redaction.
- US COPPA — children <13 require verifiable parental consent.
- Per-tenant BAA at `legal/baa-template.md` (Slice B).

### pack-eu

- GDPR Art. 8 — child consent (default 16y).
- GDPR Art. 35 prior consultation — required when DPIA indicates residual high risk; section above shows residual ≤ M for almost all rows; R-13 (ranking) residual = M acceptable with mitigation evidence.
- EU DSA Arts. 14, 16, 17, 20, 23, 24 — transparency + appeal + Statement of Reasons.
- EU AI Act 2024/1689 — high-risk classification for ranking + moderation; Arts. 9-15 risk-management + Art. 50 transparency obligations.
- UK Online Safety Act 2023 — Ofcom illegal-content duty; safety-by-design.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/social-dpia-overlay.md`.

## DPIA refresh triggers

- Any new data class.
- Any new sub-processor.
- Any change to dual-context invariant.
- Any new pack activation.
- Any new classifier version (EU AI Act re-evaluation).
- Annual scheduled review.
- Post-incident review.

## References

- `microservices/social/threat-model.md`.
- `microservices/social/policy/dual-context-isolation.md`.
- `microservices/social/policy/data-residency.md`.
- `microservices/social/compliance.md`.
- Bominal ADR-0208 + ADR-0215.
- Parallel ADR-0135.
- GDPR + KR PIPA + HIPAA + APPI + LGPD + PDPA full citations.
- EU DSA 2065/2022; EU AI Act 2024/1689; UK Online Safety Act 2023.
