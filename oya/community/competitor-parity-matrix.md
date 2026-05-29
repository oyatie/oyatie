---
doc_class: CompetitorParityMatrix
template_id: TPL-COMPETITOR-PARITY-MATRIX
microservice: community
status: Accepted
date: 2026-05-21
owner_team: axis-community
related_adrs: [ADR-0056, ADR-0131, ADR-0132, ADR-0138]
doc_status: published
---

# Competitor Parity Matrix: community µservice

Wave 15K retires the standalone `network` µservice and merges its
LinkedIn-class professional content into `community`. The top-3 counterparts are
now **Reddit / Teamblind / Handshake**. LinkedIn Jobs/Profile/Recruiter remain
secondary anchors for the jobs, profile, connections, InMail, endorsement, and
recruiter subset only; LinkedIn's engagement-optimized text feed is explicitly
not a target.

## Product-Pillar Parity

| Pillar | Reddit | Teamblind | Handshake | LinkedIn jobs/profile/recruiter | **oyatie community** |
|---|---|---|---|---|---|
| Subcommunities / spaces | Native subreddits | Company/topic boards | Employer/community pages | Groups/pages | **Y** spaces, channels, roles |
| Threaded posts + comments | Native | Native | limited | limited | **Y** post-store + thread-tree |
| Voting / ranking-by-vote | Native | partial | N | N | **Y** Wilson ranking + brigade defence |
| Moderation chains | Native mod tools | Workplace moderation | employer moderation | page/admin moderation | **Y** Cedar + audit-chain moderation pipeline |
| Anonymous workplace posting | N | Native | N | N | **Y** persona-anchored Teamblind mode |
| Workplace verification | N | Native employer verification | partial | employer/profile verification | **Y** verified workplace badges + blinded credentials |
| Job search | N | limited | Native | Native | **Y** jobs-recruiter BC |
| Applications / resume submission | N | N | Native | Native | **Y** application/referral + resume handoff |
| Employer pages | community pages only | company boards | Native | Native | **Y** employer pages inside pages-events |
| Professional profile / resume | basic profile | anonymous profile | student/candidate profile | Native | **Y** professional-profile BC |
| Connections / mutual graph | follows only | workplace affiliation | recruiter/candidate links | Native | **Y** professional-graph + connection-request |
| InMail-equivalent outreach | N | N | recruiter messaging | Native | **Y** messenger-backed InMail bridge |
| Endorsements + recommendations | awards only | N | limited | Native | **Y** signed endorsement + recommendation chain |
| Skill assessments | N | N | partial | Native | **Y** skill-assessments BC |
| Recruiter tooling | N | N | Native campus/employer tools | Native | **Y** default-off recruiter-stub with bias gates |
| Engagement-optimized feed | Hot/rising by community | topic sort | N | Native | **Forbidden**; community uses vote/moderation/relevance signals |

## Secondary Forum / KB References

| Feature | Discourse | Circle | Vanilla Forums | GitHub Discussions | Zendesk Help Center | **oyatie community** |
|---|---|---|---|---|---|---|
| Long-form KB articles | partial | partial | partial | partial | Native | **Y** immutable revision model |
| Q&A accept-answer | plugin | partial | partial | Native | partial | **Y** |
| Tags / taxonomy | Native | Native | Native | labels | categories | **Y** |
| Email-to-post / digest | Native | partial | partial | notifications | notifications | **Y** |
| Public help center | partial | N | partial | N | Native | **Y** |
| API + SDK | Y | partial | Y | Y | Y | **Y** |

## Performance Parity

| Metric | Reddit target family | Teamblind target family | Handshake / LinkedIn jobs target family | **oyatie community** |
|---|---|---|---|---|
| Feed render p99 | ≤ 800 ms | ≤ 800 ms | N/A | **300 ms** per-space |
| Search p99 | ≤ 1 s | ≤ 1 s | ≤ 1 s job/profile search | **500 ms** |
| Vote cast p99 | ≤ 200 ms | ≤ 200 ms | N/A | **100 ms** |
| Post create p99 | ≤ 400 ms | ≤ 400 ms | N/A | **250 ms** |
| Profile view p95 | N/A | partial | ≤ 300 ms | **150 ms target inherited from network** |
| Connection action p95 | N/A | N/A | ≤ 200 ms | **50 ms target inherited from network** |
| InMail send p95 | N/A | N/A | ≤ 300 ms | **100 ms target inherited from network** |
| Job handoff ack | N/A | N/A | ≤ 1 s | **1 s event-handoff target** |

## Differentiators

- **One community substrate for forums + anonymous workplace + jobs + profile.**
  No duplicate `network` data plane; professional identity is a community pillar.
- **Audit-sealed anonymity and employment actions.** Teamblind-style blinded
  credentials, endorsements, recruiter searches, and job handoffs emit
  audit-chain records.
- **Signed endorsement and recommendation chain.** Endorsements remain
  reconstructible and tamper-evident after the merge.
- **Employment-law gates on recruiter search.** Recruiter-stub is default-off
  and requires tenant-admin entitlement plus bias-audit readiness.
- **Hard feed boundary.** LinkedIn's engagement-feed mechanics are rejected;
  community ranking stays vote/moderation/relevance driven.

## Gaps

- Community JSON machine-readable PRD projection still needs a successor to the
  retired network JSON shape.
- ATS remains a downstream handoff surface; community owns job posting,
  application/referral metadata, and handoff events, not applicant-pipeline
  state.
- Re-audit is required against Reddit / Teamblind / Handshake after Wave 15K
  lands; the prior Discourse / Circle / Vanilla coverage remains useful only
  for the forum subset.

## Update Cadence

Quarterly competitive sweep, plus immediate re-audit after any change to the
four-pillar scope. Per `feedback_no_silent_regression.md`, regressions vs. this
matrix are blocking.
