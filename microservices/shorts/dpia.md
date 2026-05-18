---
doc_class: DPIA
template_id: TPL-DPIA
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-shorts
deciders: council-privacy, ops-security, axis-shorts, council-architecture, ops-legal
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0008, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/shorts/threat-model.md
  - microservices/shorts/policy/dual-context-isolation.md
  - microservices/shorts/policy/data-residency.md
  - microservices/shorts/compliance.md
review_cadence: annually + on every change to processing purpose, data classes, sub-processor list, or classifier version
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (algorithmic For-You ranking + content-moderation classifier + ASR caption form profiling)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (minor-protection records; relationship graphs; PHI possible in pack-us-healthcare patient-ed; sensitive PIPA Art. 23)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — YES (public-by-default short videos are publicly accessible; watch-time tracked systematically)"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 8, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34 (privacy and protection of PII), A.5.31 (legal/statutory)"
  - "SOC 2 Privacy criteria (P1-P8)"
  - "EU DSA 2065/2022 Arts. 14, 16, 17, 20, 23, 24, 27, 28"
  - "EU AI Act 2024/1689 Arts. 9, 10, 11, 13, 14, 15, 50, 52, 73"
  - "EU AVMSD 2018/1808 Art. 28b (video-sharing-platform obligations)"
  - "DMCA Title II 17 USC §512"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 8 (child), 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33", "PIPA Enforcement Decree Art. 35", "PIPC Notice 2020-7", "KR 청소년 보호법", "KR Telecommunications Business Act"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A)", "§164.312(b)", "§164.502(b)", "§164.514", "US COPPA 15 USC §6501"]
  pack-eu: ["GDPR Art. 8 (child consent)", "GDPR Arts. 35 + 36", "EDPB Guidelines 4/2019", "EDPB Guidelines 9/2022", "EU DSA + AI Act + AVMSD", "UK Online Safety Act 2023", "FR Audiovisual Code", "DE NetzDG"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Part III + IV", "Online Safety (Miscellaneous Amendments) Act 2022"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12", "Online Safety Act 2021", "BOSE Determination 2022"]
  pack-in: ["DPDPA 2023 §9 + §10 + §11"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 14 (child) + 38"]
  pack-ae: ["UAE PDPL Art. 23"]
  pack-ksa: ["PDPL Art. 9"]
  pack-us: ["DMCA Title II 17 USC §512", "COPPA 15 USC §6501", "CA AB-2273", "UT Social Media Regulation Act"]
doc_status: published
---

# Data Protection Impact Assessment: shorts µservice

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires DPIA where processing is **likely to result in a high risk to the rights and freedoms of natural persons**. The shorts µservice engages all three triggers:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic profiling | **YES** | Algorithmic For-You ranking + content-moderation classifier + auto-caption ASR are profiling per GDPR + EU AI Act high-risk |
| Art. 35(3)(b): Large-scale special-category | **YES** | Minor-protection records (`SENSITIVE_CHILD_PROTECTION`); relationship graphs; PHI possible in pack-us-healthcare patient-ed; sensitive PIPA Art. 23 |
| Art. 35(3)(c): Public-area monitoring | **YES** | Public-by-default short videos are publicly accessible; watch-time tracked systematically per (viewer, video) |

KR PIPC's Notice 2020-7 mandates DPIA when processing handles sensitive personal information at scale — engaged when first pack-kr enterprise tenant exceeds threshold.

DPIA is mandatory pre-deployment. This document is the canonical DPIA reviewed by EU DPAs (Art. 35), KR PIPC (PIPA Art. 33), HIPAA OCR (post-BAA, if applicable), EU DSA Coordinator (Art. 24), EU AI Act notified body, EU AVMSD coordinator, UK Ofcom, AU eSafety Commissioner, and US Copyright Office at first-tenant onboarding per pack.

## Step 2 — Describe the processing

### 2.1 Nature of the processing

**What:** Creators upload short videos (≤ 60s); the system scans (OPSWAT / ClamAV), transcodes (ffmpeg 7.x to HLS/DASH ladder), fingerprint-matches (Chromaprint audio + DCT perceptual-hash video) against a copyright corpus, auto-captions (foundry-runtime ASR), generates thumbnails (poster + animated GIF), stores blobs (S3 + CDN), classifies content (NSFW + violence + minor-protection via foundry-runtime T2), ranks feed (algorithmic For-You + chronological), tracks watch-time, fans out notifications, federates metadata (Professional opt-in), issues DRM licenses (Premium-tier), and applies retention + DMCA + appeal + parental-controls workflows.

**How:** Client → ingress (TLS/WAF) → WebSocket gateway + REST → BC services (REST + worker) → Postgres (metadata, claims, ages, parental, audio-track) + Valkey (feed cache, watch-time, like-counters, trending, notifications) + S3 (video blobs + transcode variants + thumbnails + captions) + CloudFront-class CDN (signed-URL TTL ≤ 15min) + Meilisearch (hashtag + sound + creator search) + audit-chain seal + foundry-runtime (classifier + ranking + ASR) + Widevine / FairPlay / PlayReady (DRM).

**Where:** Per-pack region-pinned shorts clusters. pack-kr (KR), pack-eu (EU), pack-us (US), pack-us-healthcare (US, HIPAA-eligible), and conditional packs.

**When:** Continuous; sub-second latency for delivery; 60s cadence for SLO evaluation; 5-min windows for trending compute.

**Who:** Creators + viewers (tenants' customers); minors (heightened-protection actor); tenant operators; oyatie operators; rights-holders + DMCA agents (external); ontology + social + messenger + workflow-engine + audit-chain + foundry-runtime µservices (machine actors); ActivityPub federation peers (Professional-tier only, opt-in, metadata-only).

### 2.2 Scope of the processing

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `PII_IDENTIFYING` | Creator handle, display-name, bio, avatar, video author identity, claim claimant identity | Art. 6(1)(b) contract + Art. 6(1)(a) consent (for public visibility) | per profile + every video |
| `PII_QUASI_IDENTIFIER` | IPs, user-agents, geo-derived hints, device fingerprint for DRM, watch-time sessions | Art. 6(1)(f) (minimised at SDK) | per-session |
| `BEHAVIORAL_TENANT_PRODUCT` | Videos, comments, likes, shares, watch-time aggregates (Professional-tier) | Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest | ~10⁵ videos/day per medium tenant |
| `BEHAVIORAL_USER_CONTENT` | Videos, comments, likes (Personal-tier) | Art. 6(1)(b) personal contract + Art. 6(1)(a) consent | ~10⁴ videos/day per medium tenant |
| `RELATIONSHIP_GRAPH` | Inherits from social: follow / block / mute edges; arguable Art. 9 sensitivity | Art. 6(1)(b) contract; user-explicit | varies |
| `SENSITIVE_PIPA_ART23` | Sensitive videos (medical, juvenile, biometric) — pack-kr | KR PIPA Art. 15 + 23 + explicit consent | varies |
| `SENSITIVE_CHILD_PROTECTION` | Minor-account flag + parental-link record + age-attestation | GDPR Art. 8 + COPPA + LGPD Art. 14 + KR 청소년 보호법 + UK OSA + CA AB-2273 + UT SMRA | per minor signup + per parental-link |
| `PHI` (pack-us-healthcare) | Patient identifiers / clinical content in videos (patient-ed use case only) | HIPAA §164.502 Permitted Uses (under BAA) | targeted to 0 via redactor; auto-OFF by default |
| `AUDIT` | Upload / publish / repost / verdict / appeal / claim / counter-notice / takedown / disclosure events | Art. 6(1)(c) | 1 record per state transition |
| `SECRET` | Per-tenant DEK, session tokens, signing keys, federation peer keys, DRM per-content keys | not personal data | OpenBao-bound |

**Geographical scope:** Per pack-pinning (data-residency.md).

**Cross-border transfer:** Forbidden by default; allowed only with tenant SCCs (Arts. 44–46) + multi-region.md. Federation egress is per-tenant opt-in for Professional-tier only, metadata-only (no blob crosses pack boundary).

### 2.3 Context of the processing

- **Data subjects:** Creators + viewers of tenant applications (consumer users of B2C tenants; employee users of B2B tenants); minors (heightened-protection); tenant operators; oyatie operators; non-creators referenced in videos (e.g., subjects of journalism or commentary).
- **Relationship:** Joint controllership with tenant under Art. 26.
- **Reasonable expectations:** Creators expect public-by-default semantics + tenant-admin moderation under disclosed policy + copyright enforcement.
- **Previous experience:** **shorts is NET-NEW** in oyatie per ADR-0135; no Bominal predecessor (Bominal had no short-video product).
- **Industry codes:** EU DSA + EU AVMSD + UK Online Safety Act + AU Online Safety Act + CA AB-2273 + UT SMRA for content moderation transparency + minor protection; DMCA Title II for copyright.
- **Children:** all packs apply pack-aware child consent threshold (GDPR Art. 8 default 16y; member states may lower to 13y; COPPA US <13; KR <14; LGPD <12; AU <16); minor accounts get heightened-protection defaults via `parental-controls` BC.

## Step 3 — Consultation

### 3.1 Internal stakeholders

| Stakeholder | Engaged | Comments |
|---|---|---|
| council-privacy chair | YES | Confirmed high-risk classification |
| ops-security lead | YES | Confirmed threat-model alignment |
| axis-shorts lead | YES | Implementation owner |
| council-architecture chair | YES | Confirmed dual-context invariant + per-pack residency |
| ops-legal lead (DMCA + EU AVMSD) | YES | Confirmed DMCA safe harbor posture + DSA video-sharing-platform obligations |
| gtm-customer-success | YES | Tenant onboarding consent flow + DPA terms |

### 3.2 External stakeholders

Will be engaged via individual data subject access request (DSAR) flows + via lead supervisory authority engagement at first per-pack tenant activation. EU lead supervisory authority engaged when first EU enterprise tenant onboards (likely DPC Ireland for VLOP-track tenants). KR PIPC engaged at pack-kr launch.

## Step 4 — Necessity + proportionality

| Question | Answer |
|---|---|
| Lawful basis | Per data class (table above); core BCs depend on Art. 6(1)(b) contract + Art. 6(1)(f) legitimate interest; copyright-claim path depends on Art. 6(1)(c) legal obligation (DMCA) |
| Specified, explicit, legitimate purpose | YES — see PRD §"Tenant Value" |
| Adequate, relevant, limited to what is necessary | YES — data-class taxonomy enforced; minimum-necessary per BC; watch-time aggregated where possible |
| Accurate + up to date | YES — DSR cascade for correction; counter-notice path for false-positive moderation |
| Retention bounded | YES — pack-aware in `policy/data-residency.md` |
| Processed in a manner that ensures appropriate security | YES — see threat-model.md mitigation table |

Alternative-means proportionality: a less invasive ranking (chronological-only) is offered as user-controllable opt-out; algorithmic For-You is opt-in for minors per `age-gate` defaults; auto-caption is opt-in for HIPAA accounts; copyright fingerprint match runs only on upload (not on private personal videos).

## Step 5 — Identify + assess risks

### Risk register

| ID | Risk | Likelihood | Severity | Mitigation | Residual |
|---|---|---|---|---|---|
| R-01 | Cross-tenant video leak (RLS misconfig) | M | H | RLS + LEAN lane + Cedar | L |
| R-02 | PHI leak in patient-ed video (pack-us-healthcare) | L | H | Auto-caption + auto-moderation OFF by default; PHI redactor on opt-in path | L |
| R-03 | Cross-context routing (Personal short → Professional feed) | L | H | Compile-time + LEAN-lane data-model invariant | L |
| R-04 | Personal-tier federation leak | L | H | Inherits social DCI-08 compile-time invariant | L |
| R-05 | Watch-time profile re-identification | M | M | Per-tenant + Cedar scope; aggregate-only for non-owners; k-anonymity ≥ 10 in creator-analytics | L |
| R-06 | DRM key compromise | VL | VH | HSM-bound root + rotation 90d + revocation list | L |
| R-07 | Minor-list pivot (attacker enumerates minors) | L | H | Separate `shorts_age_attestations` + `shorts_parental_links` tables + dedicated Cedar entitlement | L |
| R-08 | Forged DMCA copyright-claim | H | H | Claimant business-verification + perjury-attestation + counter-notice + repeat-claimant detection | M |
| R-09 | Moderation classifier false-positive event | M | H | Per-release golden-set eval + bias audit + appeal workflow + rollback runbook | M |
| R-10 | EU AI Act non-compliance (Art. 50 transparency gap) | L | H | Per-verdict eu_ai_act_label + SDK helpers + CI lane | L |
| R-11 | EU AVMSD video-sharing-platform obligation gap | L | H | Per-pack overlay + Statement of Reasons + appeal SLA + minor-protection routing | L |
| R-12 | Cross-pack residency violation (replication misconfig) | L | H | Pack-pinning + LEAN-lane `oya-check-pack-residency` + Cedar pack-scope | L |
| R-13 | Algorithmic ranking discrimination (protected groups) | M | H | Bias-audit per release; disparity ratio ≥ 0.8; appeal-via-revert-to-chronological | M |
| R-14 | Moderation classifier false-positive harming free speech | M | H | Reversible auto-hide; appeal-workflow ≤ 7d SLA per EU DSA Art. 20; manual reviewer required > confidence threshold; per-release golden-set eval; rollback runbook | M |
| R-15 | DSA Art. 27 recommender transparency gap | L | M | ranking_explanation API on every algorithmic-mode feed render | L |
| R-16 | DMCA Safe Harbor disqualification (repeat-infringer policy not enforced) | L | H | Per-creator strike counter + auto-suspend on 3+ confirmed claims within 6mo; ops-legal weekly audit | L |
| R-17 | Creator-analytics re-identifies viewers (k-anonymity violation) | L | M | k-anonymity ≥ 10 floor; suppress slices < 10 viewers; no per-viewer drill-down | L |
| R-18 | Auto-caption ASR mistranscription harms creator reputation | M | M | EU AI Act Art. 50 label "AI-generated"; creator-override path; foundry-runtime evidence record | M |

## Step 6 — Identify measures to reduce risks

Per-risk measures captured in threat-model.md §Mitigations Catalog + compliance.md per pack. Load-bearing controls:

1. Cedar v4.2 default-deny + per-policy fragment.
2. Postgres RLS on every shorts table + dual-context invariant + age-attestation isolation.
3. ffmpeg + Chromaprint sandboxed via gVisor / Kata Container.
4. EU AI Act Art. 50 transparency label on every classifier + ASR output.
5. DMCA cycle (takedown + counter-notice + repeat-infringer) audited weekly by ops-legal.
6. Minor-account defaults: chronological-only + algorithmic-recommendation-opt-out + DM-restricted + no behavioural profile.
7. Audit-chain Ed25519 seal per state transition.
8. Four-eyes Professional disclosure with distinct principal IDs.
9. Pack-pinning + cross-pack replication forbidden by default.
10. k-anonymity ≥ 10 in creator-analytics aggregates.

## Step 7 — Sign-off + record outcomes

| Sign-off | Status |
|---|---|
| DPO (council-privacy chair) | pending |
| ops-security lead | pending |
| axis-shorts lead | pending |
| council-architecture chair | pending |
| ops-legal lead | pending |

## References

- GDPR Arts. 5, 6, 8, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46.
- EU DSA Regulation (EU) 2022/2065 Arts. 14, 16, 17, 20, 23, 24, 27, 28.
- EU AI Act 2024/1689 Arts. 9, 10, 11, 13, 14, 15, 50, 52, 73.
- EU AVMSD 2018/1808 Art. 28b.
- DMCA Title II 17 USC §512.
- COPPA 15 USC §6501.
- UK Online Safety Act 2023.
- CA AB-2273; UT Social Media Regulation Act.
- KR PIPA Arts. 3, 8, 15, 17, 22-2, 23, 28, 29, 29-2, 33.
- KR 청소년 보호법; KR Telecommunications Business Act.
- HIPAA 45 CFR §164.308-316, §164.502, §164.514.
- `microservices/shorts/threat-model.md`.
- `microservices/shorts/policy/dual-context-isolation.md`.
- `microservices/shorts/policy/data-residency.md`.
- `microservices/shorts/compliance.md`.
- `microservices/social/dpia.md` (sibling reference).
- ICO DPIA template (UK); CNIL DPIA methodology (FR).
- EDPB Guidelines 4/2019 + 9/2022.
