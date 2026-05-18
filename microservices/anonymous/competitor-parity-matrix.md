---
doc_class: CompetitorParityMatrix
template_id: TPL-COMPETITIVE
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: axis-anonymous + gtm
deciders: council-architecture, council-privacy, ops-security
review_cadence: per-milestone + on any new entrant in the pseudonymous-board space
related_artifacts:
  - microservices/anonymous/PRD.md §"Competitive Benchmark"
  - microservices/anonymous/decisions/ADR-ANON-0006-federation-refusal-and-anti-pattern-anchoring.md
  - microservices/anonymous/decisions/ADR-ANON-0005-abuse-classifier-bounds.md
doc_status: published
---

# Competitor Parity Matrix — anonymous µservice

## Purpose

Track competitive parity across feature, privacy posture, regulatory posture, and abuse-mitigation dimensions for every public competitor in the pseudonymous-board / affinity-anonymity space. Includes an explicit **"do NOT replicate"** column anchoring anti-patterns we refuse to inherit. This matrix is the canonical input for `gtm` positioning and the input for ADR-ANON-0005 (abuse-classifier bounds) and ADR-ANON-0006 (federation refusal).

## Reading the matrix

- **Parity (P)** = we match or exceed the competitor's capability.
- **Refuse to replicate (R)** = we intentionally do NOT inherit; column "anti-pattern" explains why.
- **Deferred (D)** = capability scheduled-for-distinct-tracked-work to a later milestone (still on the roadmap).
- **N/A** = not applicable to our positioning.

## Feature Parity

| Feature | Sidechat | YikYak | Whisper | Blind | Fishbowl | Jodel | Secret (defunct) | Reddit anon | 4chan | Burnbook (defunct) | oyatie:anonymous |
|---|---|---|---|---|---|---|---|---|---|---|---|
| University-bound affinity | ✓ | ✓ (geo) | – | – | – | ✓ (geo) | – | – | – | ✓ (school) | **P** — ADR-ANON-0007 university-affinity cluster, k=20 floor |
| Employer-bound affinity | – | – | – | ✓ | ✓ | – | – | – | – | – | **P** — ADR-ANON-0007 employer-affinity cluster, k=20 floor (k=10 small-employer fallback) |
| Geo-bound affinity | – | ✓ | ✓ | – | – | ✓ | – | – | – | – | **P** — ADR-ANON-0007 geo-affinity cluster, k=50 floor |
| Industry-bound affinity | – | – | – | – | ✓ | – | – | – | – | – | **P** — ADR-ANON-0007 industry-affinity cluster |
| Workspace-bound affinity | – | – | – | – | – | – | – | – | – | – | **P** — ADR-ANON-0007 workspace-affinity (tenant-internal) |
| Per-session pseudonymous handle | – | – | – | – | – | – | – | – | ✓ (no handle) | – | **P** — handle rotates per session + per channel; PRD FR-02 |
| Per-channel handle rotation | – | – | – | – | – | – | – | – | – | – | **P** — ADR-ANON-0001 blinded credential reissued per channel |
| Upvote/downvote | ✓ | ✓ | – | ✓ | ✓ | ✓ | – | ✓ | – | – | **P** — PRD FR-05; Wilson ranking ADR-COMM-0002 inherited |
| Threaded replies | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | – | **P** — PRD FR-04; depth cap 6 ADR-COMM-0005 inherited |
| Hashtags + within-affinity trending | ✓ | – | – | ✓ | – | ✓ | – | – | – | – | **P** — PRD FR-08/FR-09; Meilisearch (anonymised corpus) |
| Image attachments | ✓ | ✓ | ✓ | – | – | ✓ | – | – | ✓ | ✓ | **D** — scheduled-for-distinct-tracked-work to M03 (T2 + ClamAV/OPSWAT scan; ADR successor-IP) |
| Video attachments | – | – | – | – | – | – | – | – | ✓ | – | **D** — M03 |
| Anonymous DM (E2E) | – | – | – | ✓ (server-mediated) | – | – | – | – | – | – | **P (exceeds)** — RFC 9420 MLS; server holds ciphertext only (I6) |
| Notifications (no-real-name push) | ✓ | ✓ | – | ✓ | ✓ | ✓ | – | – | – | – | **P** — PRD FR-10; opaque-handle payloads |
| Accessibility captions | – | – | – | – | – | – | – | partial | – | – | **P** — PRD §"NFR / Accessibility" (subsequent-to-M02-completion) |
| Abuse reporting | partial | partial | partial | partial | partial | partial | – | ✓ | partial | – | **P (exceeds)** — PRD FR-12; chain-of-responsibility moderation ADR-COMM-0001 inherited |
| Appeal process | – | – | – | – | – | – | – | partial | – | – | **P (exceeds)** — PRD FR-20 + ADR-COMM-0001 inherited appeal hop (EU DSA Art. 14) |
| Age gate (COPPA <13) | partial | partial | partial | partial | partial | partial | – | partial | partial | **failed** | **P** — PRD FR-14; ban under-13 universal |
| Transparency report | – | – | – | – | – | – | – | ✓ (Reddit) | – | – | **P (exceeds)** — ADR-ANON-0003; quarterly + per-pack breakdown |
| Federation (ActivityPub / AT Proto / Matrix) | – | – | – | – | – | – | – | – | – | – | **R** — PERMANENTLY REFUSED; ADR-ANON-0006 |
| Third-party analytics SDK | ✓ (typical) | ✓ (typical) | ✓ (typical) | ✓ (typical) | ✓ (typical) | ✓ (typical) | ✓ | ✓ | ✓ | ✓ | **R** — PERMANENTLY REFUSED; I4; build-time lint |
| Persistent identifier across sessions | ✓ (some) | – | – | ✓ (email) | ✓ (LinkedIn) | – | – | ✓ (username) | – | – | **R** — handle rotates; persistence enables re-identification |

## Privacy Posture (the load-bearing dimension)

| Dimension | Sidechat | YikYak | Whisper | Blind | Fishbowl | Jodel | Reddit anon | 4chan | oyatie:anonymous |
|---|---|---|---|---|---|---|---|---|---|
| **Platform can correlate user_id ↔ post_id** | ✓ (yes, server-side) | ✓ | ✓ (Whisper breach 2014 — server-side identity records discoverable) | ✓ (email retained) | ✓ (LinkedIn linkage) | ✓ | ✓ | ✓ (IP logs) | **R / I1** — **NO**; cryptographic blinding (ADR-ANON-0001); ONLY under legal-process Cedar gate (I7) |
| **Affinity attestation reveals identity to platform** | ✓ (school email) | – | – | ✓ (corporate email plaintext) | ✓ (LinkedIn handle) | – | – | – | **R / I2** — **NO**; BBS+ selective-disclosure (ADR-ANON-0002); platform learns affinity, never identity |
| **End-to-end encryption (DMs)** | – | – | – | – | – | – | – | – | **P / I6** — MLS RFC 9420 |
| **Retention default** | unbounded | 5 mi / unbounded | unbounded | unbounded | unbounded | unbounded | unbounded | unbounded | **P / I3** — 30 days default (tenant-selectable 30/60/90) |
| **Hard-delete propagation guarantee** | – | – | – | – | – | – | partial | – | **P / I3** — p99 ≤ 5s SLO; tombstone seal mandatory |
| **Legal-process disclosure dual-control** | unknown | unknown | unknown | unknown | unknown | unknown | unknown | unknown | **P (exceeds) / I7** — two distinct approvers + Cedar gate + 14-day notice + audit-chain seal + transparency report |
| **Cross-affinity federation** | – | – | – | – | – | – | – | – | **R / I5** — refused permanently |

## Regulatory Posture

| Framework | Sidechat | YikYak | Whisper | Blind | Fishbowl | oyatie:anonymous |
|---|---|---|---|---|---|---|
| GDPR Arts. 5/11/17/22/25 | partial | partial | partial | partial | partial | **P (exceeds)** — pack-eu overlay; Art. 11 + Recital 26 pseudonymisation; right-to-erasure p99 ≤ 5s |
| EU DSA Arts. 14/16/17/27/28 | partial | partial | – | partial | partial | **P (exceeds)** — full appeal hop (Art. 14); statement-of-reasons (Art. 17); transparency (Arts. 27/28); quarterly report |
| EU AI Act Art. 50 transparency (classifier) | – | – | – | – | – | **P** — "AI-assessed" label on every classifier verdict; ADR-ANON-0005 |
| KR 통신비밀보호법 (anonymous-comm protection) | – | – | – | – | – | **P** — pack-kr overlay; legal-process Cedar gate per Art. 9 |
| KR PIPA Arts. 21/24-2/28 | – | – | – | – | – | **P** — pack-kr; alternative-pseudonymous-processing (Art. 24-2); right-to-erasure (Art. 21) |
| US Section 230 + COPPA + state anti-doxxing | partial | partial | – | partial | partial | **P** — pack-us; under-13 ban universal (COPPA §312.5); CA/NY/IL anti-doxxing |
| US 18 USC §2258A NCMEC | partial | partial | partial | partial | partial | **P** — FR-27; CSAM-suspect → CyberTipline within 48h |
| UK Online Safety Act 2023 | partial | partial | – | – | – | **P** — pack-uk; statutory duty of care + risk-assessment |
| UK Investigatory Powers Act 2016 | partial | partial | – | – | – | **P** — pack-uk; legal-process Cedar gate per IPA §57; gag-order doctrine respected |
| JP 通信の秘密 + APPI | – | – | – | – | – | **P** — pack-jp; legal-process Cedar gate per Telecom Business Act |
| FIPS 140-3 crypto boundary | – | – | – | – | – | **P** — blind-signatures + MLS in FIPS-validated module path |

## Anti-Pattern Anchoring (the "do NOT replicate" column)

| Competitor | Anti-pattern observed | Why we refuse | Mitigation in our µservice |
|---|---|---|---|
| **Whisper** | Server-side identity records discoverable; 2014 breach revealed Whisper retained location + user-correlation data despite "anonymous" branding. | Defeats I1 structurally. | ADR-ANON-0001 cryptographic blinding; platform structurally cannot answer "who wrote this" without court order |
| **Blind** | Corporate-email plaintext retained for "verification"; identity-correlation possible at platform level despite "anonymous" branding. | Defeats I2 structurally. | ADR-ANON-0002 BBS+ selective-disclosure; platform learns affinity, never identity |
| **Secret (defunct)** | No abuse-reporting, no moderation queue, no legal-process workflow. Led to harassment cascades and 2015 class-action shutdown. | Defeats community-safety bar; regulatory liability under EU DSA + Section 230 good-faith. | PRD FR-12/FR-19/FR-20 + ADR-COMM-0001 chain-of-responsibility moderation inherited + ADR-ANON-0005 classifier bounds |
| **Burnbook (defunct)** | School-bound app with no minor protection + no anti-harassment classifier; 2017 shutdown after bullying lawsuits. | Defeats minor protection + community-safety bar. | PRD FR-14 age-gate (COPPA universal) + FR-12 abuse-report + ADR-ANON-0005 classifier bounds + FR-23 university-affinity k=20 |
| **4chan** | No moderation accountability; CSAM hosting historical; federation-light failure mode (cross-board no-account leaks). | Defeats moderation accountability bar; CSAM = US 18 USC §2258A violation. | PRD FR-27 NCMEC reporting; ADR-ANON-0005 classifier T1; ADR-ANON-0006 federation refused |
| **YikYak (early)** | Single-campus k=1 effective anonymity; harassment cascades on small campuses. | Defeats k-anonymity bar. | ADR-ANON-0007 k=20 university floor with anonymisation-fallback for tiny schools |
| **Reddit anon** | Persistent username enables long-term correlation across topics. | Defeats handle-rotation bar (I1). | PRD FR-02 per-channel handle rotation |
| **Fishbowl** | LinkedIn-based verification leaks identity to LinkedIn even if not to fishbowl. | Defeats I2 (third-party identity leak). | ADR-ANON-0002 BBS+ flow; OIDC-with-blinding option (open question 2 in PRD) |
| **All competitors** | Ship Google Analytics / Mixpanel / Amplitude SDK in client. | Defeats I4 (third-party telemetry undermines anonymity claim). | I4 invariant + LEAN lane `oya-check-third-party-tracker-refused` |
| **All competitors** | Ad-hoc legal-process disclosure (no dual-control, no transparency report). | Defeats audit-grade compliance bar; regulatory liability under EU DSA Art. 27. | ADR-ANON-0003 dual-control + 14d-notice + audit-chain seal + transparency-report; FR-17/FR-18 |

## "What would we lose if we tried to replicate the anti-pattern" — explicit cost statement

| Anti-pattern temptation | Imagined benefit | What we'd lose |
|---|---|---|
| "Just store user_id alongside post_id for fast deletion" | -50ms hard-delete latency | I1 violated; tenant trust gone; class-action exposure (Whisper precedent) |
| "Store employer-email plaintext for affinity verify" | simpler crypto stack; -100ms attest-verify | I2 violated; Blind-precedent class-action exposure |
| "Federate posts to peer servers via ActivityPub" | larger network effect | I1 + I7 both violated (peer servers can't be bound to legal-process Cedar gate); regulatory liability under EU DSA cross-border |
| "Ship Google Analytics for product metrics" | richer product analytics | I4 violated; Cookie-banner regression; ePrivacy Art. 5(3) violation |
| "Persist username across sessions" | better cross-thread continuity for users | I1 weakened; long-term correlation possible |
| "Single-employee affinity verification (k=1)" | usable by tiny employers | I2 effectively defeated (single employee = identity); ADR-ANON-0007 prohibits |
| "Skip transparency report" | less legal exposure for embarrassing disclosure stats | EU DSA Art. 27 violation; community-trust regression |

## Industry Bench Numbers (best-effort public sources)

| Metric | Sidechat (2024) | YikYak (2024) | Blind (2024) | Fishbowl (2024) | oyatie:anonymous (target M02) |
|---|---|---|---|---|---|
| Monthly active users | ~5M | ~2M | ~8M | ~1M | (per-tenant; not aggregated) |
| Feed-render p95 | unknown | ~300ms | ~250ms | ~400ms | **≤ 250ms** (better than median) |
| Post-create p99 | unknown | ~200ms | ~150ms | ~300ms | **≤ 250ms** |
| Hard-delete propagation | unbounded | unbounded | unbounded | unbounded | **≤ 5s p99** (best in class) |

Sources:
- sidechat.lol (limited public docs)
- yikyak.com (limited public docs)
- teamblind.com (employer-verification model, public TOS)
- fishbowlapp.com (LinkedIn acquisition disclosures 2022)
- jodel.com (German market public docs)
- Whisper breach analysis (Washington Post, 2014)
- Secret shutdown analysis (Recode, 2015)
- Burnbook shutdown analysis (TechCrunch, 2017)

## References

- PRD-anonymous §"Competitive Benchmark"
- ADR-ANON-0005 (abuse-classifier bounds + EU AI Act Art. 50)
- ADR-ANON-0006 (federation refusal — anchored against 4chan + ActivityPub anti-pattern)
- ADR-ANON-0007 (affinity-cluster k-anonymity floor — anchored against YikYak + Blind anti-patterns)
- EU Digital Services Act (Regulation 2022/2065)
- EU AI Act (Regulation 2024/1689)
- US 18 USC §2258A (NCMEC CyberTipline)
- Whisper / The Guardian + Washington Post 2014 disclosures on server-side correlation
