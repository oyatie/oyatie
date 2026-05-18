---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-network + council-architecture
deciders: axis-network, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0135, ADR-0131, ADR-0132, ADR-0133]
related_artifacts:
  - microservices/network/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-NETWORK gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (network µservice)

## Purpose

Quantitative + qualitative parity comparison against industry-leading Professional-network products. Drives `oya-governance-hyperscaler-maturity-claims` gate per HG-NETWORK (ADR-0123) and constrains what gtm-customer-success can claim in tenant sales conversations. Re-validated bi-annually because the Professional-network landscape moves quickly (EU AI Act enforcement waves; NYC LL144 enforcement waves; LinkedIn product changes; Microsoft Copilot for Sales integration; AI-recruiter feature parity wars).

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| LinkedIn (Microsoft) | global Professional network + jobs + recruiter + learning | scale (~1B accounts); deepest jobs + recruiter + sales ecosystem; Microsoft 365 integration | `learn.microsoft.com/linkedin` |
| Xing | DACH-region Professional network + jobs | German-market depth; works-council compliance; localised employer ecosystem | `dev.xing.com` |
| Wantedly | JP-region Professional + culture-fit network | Japan-market depth; values-and-culture-fit matching | `wantedly.com/developers` |
| VietnamWorks / JobStreet (SEABase) | SEA-region jobs + lite Professional profile | regional-market depth; jobs-centric | `vietnamworks.com/api`, `jobstreet.com.sg/api` |
| Glassdoor | employer-review + salary-insights | reviews + salary transparency | `glassdoor.com/developer` |
| AngelList / Wellfound | startup talent + jobs | startup-talent niche | `wellfound.com/api` |
| Hashnode | developer-community Professional content | developer-content niche; Web3 / OSS | `hashnode.com/docs/api` |
| Polywork | portfolio-first Professional network | portfolio + multi-role identity | `polywork.com/api` |
| Lunchclub | AI-matched Professional intros | 1-on-1 video matching | `lunchclub.com/api` |
| Bumble Bizz | Professional networking + dating-style swipe | swipe-mechanic + women-first onboarding | `bumble.com/api` |
| Shapr | swipe-style Professional intros | mobile-first swipe | `shapr.co/api` |
| Slack Communities | Professional groups + adjacent communities | group + chat-based community | `api.slack.com` |
| Indeed | global jobs network (no graph) | jobs-search depth; Glassdoor sibling | `developer.indeed.com` |
| Workday + SAP SuccessFactors (HRIS) | enterprise HRIS / talent management | enterprise-HR depth | `community.workday.com` |
| Greenhouse / Lever / Ashby (ATS) | applicant tracking | ATS deep workflow | `developers.greenhouse.io` |

## Feature Parity Matrix

### Profile + identity

| Capability | oyatie | LinkedIn | Xing | Wantedly | Indeed | Glassdoor |
|---|---|---|---|---|---|---|
| Profile (resume, experience, education, skills, certifications) | ✅ | ✅ | ✅ | ✅ | ✅ | partial |
| Verification badge (blue / org / gov / employer-confirm) | ✅ | partial (blue + emp-confirm) | partial | partial | partial | ❌ |
| Skill endorsements (Ed25519 chain) | ✅ unique chain | ✅ (no chain) | ✅ | partial | ❌ | ❌ |
| Long-form recommendations | ✅ | ✅ | ✅ | ✅ | partial | ❌ |
| Skill assessments + badge | ✅ | ✅ | partial | ❌ | partial | ❌ |
| Profile-export vCard 4.0 + JSON Resume | ✅ both | partial (JSON only via API) | ❌ | ❌ | ❌ | ❌ |
| GDPR Art. 20 portable export | ✅ native | ✅ (manual via DSR) | ✅ (manual) | partial | partial | partial |

### Connections + graph

| Capability | oyatie | LinkedIn | Xing | Wantedly | Slack Communities |
|---|---|---|---|---|---|
| 1st-degree connection edge | ✅ | ✅ | ✅ | ✅ | partial (channel-level) |
| 2nd-degree extension | ✅ | ✅ | ✅ | partial | ❌ |
| 3rd-degree visibility | ✅ | ✅ | ✅ | ❌ | ❌ |
| Connection-request note (≤ 300 chars) | ✅ | ✅ | ✅ | ✅ | n/a |
| Follow (asymmetric, distinct from connect) | ✅ | ✅ | partial | partial | n/a |
| Block / restrict / disconnect | ✅ | ✅ | ✅ | ✅ | partial |
| Degree-of-separation API | ✅ (SDK-exposed) | partial (UI only) | partial | ❌ | ❌ |

### Feed + content

| Capability | oyatie | LinkedIn | Xing | Hashnode | Slack Communities |
|---|---|---|---|---|---|
| Chronological feed | ✅ | partial (algorithmic default) | ✅ | ✅ | ✅ |
| Algorithmic feed (heuristic in P01, ML in P03) | ✅ | ✅ | partial | partial | partial |
| User-choice feed switcher | ✅ | partial | partial | ✅ | ✅ |
| Article + status + document + poll + carousel | ✅ | ✅ | partial | article-first | partial |
| Extended reactions (7-emoji Professional set) | ✅ | ✅ | partial | partial | ✅ |
| Hashtags + mentions | ✅ | ✅ | ✅ | ✅ | partial |
| Trending topics (Professional context) | ✅ | ✅ | ✅ | ✅ | partial |
| Repost / share / quote-post | ✅ | ✅ | partial | partial | n/a |
| Post-to-messenger deep-share | ✅ (InMail bridge) | ✅ | partial | n/a | n/a |
| Bookmarks (private) | ✅ | ✅ | partial | partial | ✅ |

### Jobs + recruiter

| Capability | oyatie | LinkedIn | Indeed | Greenhouse | Wantedly |
|---|---|---|---|---|---|
| Job postings | ✅ (handoff to ATS) | ✅ | ✅ | ✅ | ✅ |
| Job search + faceted filters | ✅ | ✅ | ✅ | ✅ | partial |
| Applicant referral flow | ✅ | ✅ | partial | ✅ | partial |
| Recruiter-search ranker (with bias audit) | ✅ (OFF default; EU AI Act gated) | ✅ (variable disclosure) | partial | partial | partial |
| ATS integration (handoff contract) | ✅ (clean boundary; contract-versioned events) | partial | partial | n/a (is ATS) | partial |
| HRIS integration (Workday / SAP / BambooHR) | ✅ (via webhook + SDK adapters) | partial | partial | ✅ | partial |
| EU AI Act Annex III §4 high-risk compliance | ✅ native (ADR-NET-0002) | scrambling post-2024 | scrambling | scrambling | scrambling |
| NYC LL144 candidate-notice SDK helper | ✅ native | partial (manual) | partial (manual) | partial | n/a (no NYC ops) |
| EEOC UGESP 4/5-rule monitoring | ✅ native dashboard | partial | partial | partial | n/a (no US ops) |
| CA AB-331 + CO SB 24-205 conformance | ✅ native | scrambling | scrambling | scrambling | n/a |

### InMail + messenger

| Capability | oyatie | LinkedIn | Xing | Wantedly |
|---|---|---|---|---|
| InMail (premium messenger to non-connected) | ✅ (bridge to messenger µservice; Professional-tier-only) | ✅ | ✅ | ✅ |
| Per-tenant InMail rate limit | ✅ configurable | ✅ tier-based | partial | partial |
| Spam classifier on InMail | ✅ (foundry-runtime T2) | ✅ | partial | partial |
| InMail-to-Personal-tier-DM bridge | ❌ (forbidden by design; ADR-NET-0003) | partial (account-level) | partial | partial |

### Pages + groups + events

| Capability | oyatie | LinkedIn | Xing | Slack Communities | Bumble Bizz |
|---|---|---|---|---|---|
| Company / brand Pages | ✅ | ✅ | ✅ | partial (workspace level) | ❌ |
| Multi-admin Pages | ✅ | ✅ | ✅ | ✅ | n/a |
| Page newsletter (mail bridge) | ✅ (mail µservice bridge) | ✅ | partial | n/a | n/a |
| Private + open groups | ✅ | ✅ | ✅ | ✅ | partial |
| Events + RSVP + calendar bridge | ✅ (calendar µservice bridge; iCal export) | ✅ | ✅ | partial | partial |
| Hybrid + virtual event support | ✅ (via calendar µservice video bridge) | ✅ | partial | partial | partial |

### Salary insights + transparency

| Capability | oyatie | LinkedIn | Glassdoor | Levels.fyi | Hashnode |
|---|---|---|---|---|---|
| Aggregate salary insights (per role + region) | ✅ (interface-only-pending-impl in P01; aggregate-only) | ✅ | ✅ | ✅ | ❌ |
| Per-individual salary disclosure | ❌ (forbidden by design) | ❌ | partial (employer reviews) | partial | ❌ |
| EU Pay Transparency Directive 2023/970 conformance | ✅ native | partial | partial | n/a (no EU ops) | n/a |
| Tenant-opt-in pay-band publish (Page-level) | ✅ | partial | n/a | n/a | n/a |

### Compliance + enterprise

| Capability | oyatie | LinkedIn | Xing | Wantedly | Workday HRIS |
|---|---|---|---|---|---|
| eDiscovery hold (Professional-tier) | ✅ | partial | partial | ❌ | ✅ |
| Retention per regulatory pack | ✅ (11 packs) | tenant-level only | tenant-level | tenant-level | ✅ |
| HIPAA BAA (when health-context surface) | conditional (pack-us-hc) | ❌ | ❌ | ❌ | conditional |
| KR PIPA + KISA | ✅ pack-kr | partial | partial | partial | partial |
| KR 직장 갑질 (workplace harassment) handler | ✅ native abuse category | ❌ | ❌ | ❌ | partial |
| KR 근로기준법 work-record floor | ✅ pack-kr retention floor | partial | partial | partial | ✅ |
| EU AI Act Annex III §4 (employment) high-risk | ✅ native | scrambling | scrambling | scrambling | scrambling |
| GDPR Art. 22 opt-out surface | ✅ native UI | partial | partial | partial | partial |
| GDPR Art. 20 portability (vCard + JSON Resume + JSON) | ✅ native | partial (manual DSR) | partial (manual) | partial | partial |
| Four-eyes admin disclosure | ✅ | ❌ | ❌ | ❌ | partial |
| Cedar / Rego / OPA policy | ✅ Cedar v4.2 | partial | partial | partial | partial (proprietary) |
| EU DSA transparency report | ✅ native (Art. 24) | required (Art. 24); partial reports | required; partial | required; partial | n/a (not VLOP) |
| Ed25519 audit-chain + endorsement-chain | ✅ both | ❌ (opaque logs) | ❌ | ❌ | partial (audit log only) |
| Minor protections (chronological-only feed; no recruiter exposure) | ✅ | ✅ (limited recruiter exposure) | partial | n/a | n/a |
| Federation (ActivityPub) | ❌ P01 (scheduled-for-distinct-tracked-work; ADR-NET future) | ❌ | ❌ | ❌ | n/a |

### Substrate + integration

| Capability | oyatie | LinkedIn | Xing | Workday | Greenhouse ATS |
|---|---|---|---|---|---|
| Self-hosted (no vendor lock) | ✅ (Helm + Kustomize) | ❌ | ❌ | ❌ | ❌ |
| Multi-region data-residency | ✅ 11 packs | partial (regions) | DACH only | enterprise-tier | enterprise-tier |
| OpenSLO + agentic gate | ✅ | ❌ | ❌ | ❌ | ❌ |
| Workflow + Ontology native integration | ✅ | ❌ | ❌ | proprietary | proprietary |
| Per-tenant Cedar policy | ✅ | ❌ | ❌ | partial | partial |
| ATS-handoff contract (event-versioned) | ✅ to Tier-G ATS µservice | partial | partial | ✅ proprietary | ✅ proprietary |
| HRIS adapter SDK | ✅ (Workday / SAP / BambooHR / Personio patterns) | partial | partial | ✅ (is HRIS) | partial |

## Quantitative Performance Parity

| Metric | oyatie target | LinkedIn ref | Xing ref | Notes |
|---|---|---|---|---|
| Profile-view p95 | ≤ 150ms | ~150ms (per Microsoft Engineering published targets) | ~250ms | parity |
| Feed-render p95 (top 50) | ≤ 200ms | ~250ms | ~350ms | parity / better |
| Connection-action p99 | ≤ 150ms | ~100-150ms (published) | n/a | parity |
| Search-people p95 (most-searched surface) | ≤ 250ms | ~300ms | ~400ms | better |
| Search-content p95 | ≤ 500ms | ~500ms | ~700ms | parity |
| Search-jobs p95 | ≤ 400ms | ~400ms | ~600ms | parity |
| InMail-send p95 | ≤ 100ms | ~150ms | n/a | parity / better |
| Notification fanout p99 (30k followers) | ≤ 2s | ~3s (estimated) | n/a | parity / better |
| Notification fanout p99 (300k followers) | ≤ 5s | ~5-10s (estimated) | n/a | parity / better |
| Endorsement add p99 | ≤ 120ms | ~150ms (estimated) | n/a | parity |
| Media transcode (image) p95 | ≤ 2s | ~3s | n/a | parity |
| Media transcode (video HLS) p95 | ≤ 90s | ~60-90s | n/a | parity |
| vCard / JSON Resume export p95 | ≤ 300ms | n/a (DSR is async hours) | n/a | better |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | ML-driven feed ranking (vs heuristic in P01) | axis-network + axis-foundry-runtime | M03 |
| 2 | ML-driven recruiter-stub ranker (with EU AI Act conformity assessment) | axis-network + axis-foundry-runtime + ops-compliance | M04 |
| 3 | LinkedIn-Learning equivalent (out-of-scope P01; interface-only-pending-impl) | axis-network | M05-onward (separate µservice or activated stub) |
| 4 | Mature B2B sales-navigator equivalent | axis-network + gtm | M05-onward |
| 5 | Mobile SDK polish (iOS/Android parity with LinkedIn native) | axis-network + gtm | M02-onward1 |
| 6 | ActivityPub Professional-tier federation (out-of-scope P01) | axis-network + council-architecture + ops-security | successor-IP ADR-NET |
| 7 | AT Protocol Professional federation | axis-network + council-architecture | successor-IP ADR-NET |
| 8 | Verified-handle global uniqueness (vs per-tenant) | axis-network + gtm | ADR-NET successor-IP |

## Key oyatie Differentiators (NOT in any competitor)

1. **Professional-context isolation by data-model invariant** — never federates to Personal-tier; compile-time invariant; no competitor enforces.
2. **11-pack residency by design** — no SaaS competitor matches breadth; LinkedIn is region-only.
3. **OpenSLO-gated promotion** — feature rollouts gated by burn-rate (ADR-0130); no competitor enforces.
4. **Cedar v4.2 policy substrate** — fine-grained per-resource policy; competitors expose only admin-level RBAC.
5. **Endorsement-chain Ed25519 cryptographic integrity** — competitors deliver display-only counts; oyatie endorsements are cryptographically verifiable.
6. **Four-eyes admin disclosure** — two-principal approval for PII reads; no competitor enforces.
7. **Workflow + Ontology native integration** — first-class events typed into Workflow Studio; competitors expose webhooks only.
8. **EU AI Act Annex III §4 + Art. 50 + Art. 27 transparency from day-1** — competitors scrambling post-2024.
9. **NYC LL144 + CA AB-331 + CO SB 24-205 conformance from day-1** — competitors partial / manual.
10. **GDPR Art. 22 opt-out + Art. 20 portability (vCard 4.0 + JSON Resume) native** — competitors partial.
11. **Personal-tier never bleeds into Professional context — compile-time guarantee** — no competitor matches; LinkedIn is Professional-only but cannot interoperate, while X/Meta blur.
12. **KR 직장 갑질 (workplace harassment) abuse category native** — no LinkedIn / Xing parity.
13. **Clean ATS-handoff boundary via contract-versioned events** — competitors couple posting + pipeline-management; oyatie cleanly separates.

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):

- ✅ "Professional-tier isolation enforced as a data-model invariant is unique to oyatie" (true as of 2026-05-17; review bi-annually).
- ✅ "11-pack residency exceeds LinkedIn / Xing / Indeed regional coverage" (true).
- ✅ "OpenSLO-gated feature rollout is unique to oyatie among production Professional networks" (review bi-annually).
- ✅ "Cedar v4.2 fine-grained policy substrate exceeds LinkedIn admin RBAC depth" (true).
- ✅ "EU AI Act Annex III §4 (employment) high-risk obligations operative from day-1" (true; review bi-annually).
- ✅ "NYC LL144 candidate-notice + bias-audit conformance ships in-SDK" (true).
- ✅ "Endorsement-chain Ed25519 cryptographic integrity is unique to oyatie" (true).
- ✅ "vCard 4.0 + JSON Resume + GDPR Art. 20 portable export native" (true).

Sales claims FORBIDDEN (per ADR-0123 hyperscaler-maturity-claim-gate):

- ❌ "oyatie network is faster than LinkedIn" (no published independent benchmark; would be unsourced superiority).
- ❌ "oyatie has more Professional features than LinkedIn" (feature-count is unmeasurable + LinkedIn has 15+ years ecosystem head start).
- ❌ "HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation).
- ❌ "LinkedIn-API compatible" (we accept LinkedIn-style incoming-webhook URL shape only; full LinkedIn-API parity not claimed).
- ❌ "Algorithm-free" (we ship hybrid; user can choose chronological but algorithmic is the default for engagement; don't market as algorithm-free).
- ❌ "EU AI Act certified" (we are operationally conformant; AI Act notified-body conformity assessment for recruiter-stub is an explicit Step in ADR-NET-0002; don't market as "certified" without active certificate).
- ❌ "Eliminates hiring bias" (we monitor + bound + report; bias cannot be eliminated, only bounded statistically).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes (new features / pricing / claims) | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-network |
| 3. Re-run quantitative benchmarks (load tests in staging cluster) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary rule updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `microservices/network/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-NETWORK gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0135 (Connect dissolution, parallel).
- ADR-0130 (agentic SLO-gated promotion).
- ADR-0132 (suite-and-bundle dissolution).
- ADR-0133 (industry best-practice conformance).
- Competitor docs as cited inline above.
- EU DSA 2065/2022; EU AI Act 2024/1689; EU Pay Transparency Directive 2023/970.
- NYC Admin Code §§20-870, 20-871, 20-872; CA AB-331; CO SB 24-205.
