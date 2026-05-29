---
doc_class: competitor-parity-matrix
microservice: performance-management
date: 2026-05-21
status: wave-4-rolling-remediation
counterparts: [Lattice, 15Five, Workday Performance]
engagement_adjacency: [Culture Amp, Glint]
audit_dimension: D-20.5
big_8_family: HR/Payroll
big_8_priority: P0
coverage_floor: 0.85
---

# Performance Management — Competitor Parity Matrix

This document maps Oyatie `performance-management` capability coverage against the three
primary counterparts named in the user directive: **Lattice**, **15Five**, **Workday
Performance**. Culture Amp and Glint are retained as engagement-pulse adjacencies only
(per audit Finding 2.3.A) and are noted in the Engagement section only.

The matrix is the source of truth for parity reporting. Each row names a capability,
locates it inside our 12-bounded-context surface, cites the counterpart that has the most
mature equivalent, and grades coverage as **F**ull / **P**artial / **G**ap. Big-8 P0 floor
requires ≥85% Full+Partial.

## 1. Goals + OKR cascade

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Goal authoring (title, KR, owner, due) | full | full | full | goal-cycle | Full (FR-001) |
| Goal cascade (align up/down) | full | full | full | goal-cycle | Full (FR-002) |
| Quarterly check-in | full | full | full | goal-cycle | Full (FR-003) |
| Goal-cycle close + carry-forward | full | full | full | goal-cycle | Full (FR-004) |
| Org-wide OKR cascade view | full | full | full | goal-cycle | Full (FR-005) |
| Goal templates (per team, library) | full | partial | full | goal-cycle | Full (capabilities/goal-template-library.yaml) |
| Goal confidence rating | full | full | partial | goal-cycle | Full (FR-003) |
| Goal categorization (committed/stretch) | full | full | full | goal-cycle | Full (capabilities/goal-categorization.yaml) |
| Cross-functional goal alignment | full | partial | full | goal-cycle | Full (capabilities/cross-functional-goal-alignment.yaml) |

## 2. Reviews + 360 feedback

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Annual review cycle | full | full | full | review-cycle | Full (FR-006) |
| Semi-annual review cycle | full | full | full | review-cycle | Full (FR-006) |
| Project-anytime review | full | partial | full | review-cycle | Full (FR-006) |
| Probationary review | partial | partial | full | review-cycle | Full (FR-006) |
| New-hire 30/60/90 review | full | full | partial | review-cycle | Full (FR-006, B-9 inbound) |
| Self-review form | full | full | full | review-cycle | Full (FR-007) |
| Manager review form | full | full | full | review-cycle | Full (FR-007) |
| 360 feedback collection | full | full | full | review-cycle + feedback | Full (FR-008) |
| Peer feedback request | full | full | full | feedback | Full (FR-013) |
| Skip-level review | partial | partial | full | review-cycle | Full (capabilities/skip-level-review.yaml) |
| Review evidence sealing (immutable) | partial | partial | full | review-cycle | Full (IP-027, FR-010) |
| Calibration to final rating handoff | full | partial | full | review-cycle + calibration | Full (FR-009, FR-011) |
| Outbound rating-finalized to compensation | partial | partial | full | review-cycle | Full (FR-011, B-1) |

## 3. Continuous feedback + recognition

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Anytime feedback give/request | full | full | full | feedback | Full (FR-012) |
| Manager note (private to manager) | full | full | full | feedback | Full (FR-014) |
| Skip-level visibility | full | partial | full | feedback | Full (FR-014) |
| Negative-class feedback cool-down | full | partial | partial | feedback | Full (OQ-2 default 24h) |
| Public recognition / kudos | full | partial | full | recognition | Full (FR-028) |
| Recognition reactions | full | partial | partial | recognition | Full (FR-029) |
| Recognition wall | full | partial | full | recognition | Full (capabilities/recognition-wall.yaml) |
| Recognition tagging (values-aligned) | full | partial | full | recognition | Full (capabilities/recognition-tagging.yaml) |

## 4. One-on-one facilitation

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| 1:1 agenda authoring | full | full | partial | one-on-one-cadence | Full (FR-022) |
| Shared 1:1 notes (mgr + report) | full | full | partial | one-on-one-cadence | Full (capabilities/one-on-one-shared-notes.yaml) |
| 1:1 action items | full | full | partial | one-on-one-cadence | Full (FR-022) |
| 1:1 history | full | full | partial | one-on-one-cadence | Full (FR-022) |
| 1:1 auto-prep packet | full | full | partial | one-on-one-cadence + manager-tooling | Full (FR-024) |
| 1:1 talking-point suggestions | full | partial | gap | one-on-one-cadence | Full (capabilities/one-on-one-talking-points.yaml) |

## 5. Check-ins (weekly + monthly)

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Weekly check-in (priorities, blockers, mood) | partial | full | partial | weekly-check-in | Full (FR-023) |
| Manager rollup of check-ins | partial | full | partial | weekly-check-in | Full (FR-023) |
| Weekly mood trend | partial | full | gap | weekly-check-in | Full (capabilities/check-in-mood-trend.yaml) |
| Monthly check-in (broader pulse) | partial | partial | partial | engagement-survey | Full (FR-015) |

## 6. Calibration

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Calibration session (lock semantics) | full | partial | full | calibration | Full (FR-019, FR-020) |
| Force-distribution overlay | full | partial | full | calibration | Full (FR-019) |
| Nine-box grid (perf x potential) | partial | gap | full | calibration | Full (FR-019, capabilities/nine-box-grid.yaml) |
| Talent calibration (executive) | partial | gap | full | calibration | Full (FR-021) |
| Calibration ledger (immutable audit) | partial | gap | full | calibration | Full (IP-027) |
| Calibration fairness check (Title VII) | partial | gap | full | calibration | Full (capabilities/calibration-fairness-check.yaml) |
| Outbound calibration outcome to people-records | partial | gap | full | calibration | Full (B-2) |

## 7. Talent management + succession planning

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Talent card (current + next role) | partial | gap | full | succession-planning | Full (FR-025) |
| Readiness rating (now / N-year) | partial | gap | full | succession-planning | Full (FR-025) |
| Succession plan per role | gap | gap | full | succession-planning | Full (FR-026) |
| Successor bench list | gap | gap | full | succession-planning | Full (capabilities/successor-bench.yaml) |
| High-potential identification | partial | gap | full | talent-management | Full (capabilities/high-potential-identification.yaml) |
| Performance x potential matrix | partial | gap | full | talent-management | Full (capabilities/performance-potential-matrix.yaml) |
| Development plan reference | partial | partial | full | talent-management | Full (capabilities/development-plan-reference.yaml) |
| Mentorship matching | gap | gap | full | talent-management | Full (capabilities/mentorship-matching.yaml) |
| Career mobility (internal job) | gap | gap | full | talent-management | Full (capabilities/career-mobility.yaml) |
| Outbound talent card to workforce-planning | partial | gap | full | succession-planning | Full (B-7) |

## 8. Engagement surveys + eNPS

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Pulse survey (eNPS-style) | full | full | partial | engagement-survey | Full (FR-015) |
| Full engagement survey | full | full | partial | engagement-survey | Full (FR-016) |
| Anonymity floor enforcement (k>=N) | full | full | partial | engagement-survey | Full (FR-017, IP-029, Cedar) |
| Sentiment keyword extraction | partial | partial | gap | engagement-survey | Full (FR-018) |
| Sentiment trend (per team, quarterly) | partial | partial | gap | analytics-reporting | Full (FR-033, capabilities/sentiment-trend.yaml) |
| Pulse cadence configurable | full | full | partial | engagement-survey | Full (capabilities/pulse-cadence-config.yaml) |
| Survey question bank | full | full | partial | engagement-survey | Full (capabilities/survey-question-bank.yaml) |
| Comparison cohort (industry benchmark) | partial | full | gap | engagement-survey | Partial (engagement adjacency Culture Amp/Glint) |
| Driver analysis (engagement to outcomes) | partial | partial | gap | engagement-survey | Full (capabilities/engagement-driver-analysis.yaml) |

## 9. Analytics + reporting

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Manager dashboard | full | full | full | manager-tooling + analytics-reporting | Full (FR-030, FR-032) |
| HRBP analytics | full | full | full | analytics-reporting | Full (FR-032) |
| Executive view (org rollup) | full | partial | full | analytics-reporting | Full (capabilities/executive-rollup.yaml) |
| Rating distribution | full | partial | full | analytics-reporting + calibration | Full (FR-032) |
| Engagement trend | full | full | partial | analytics-reporting | Full (FR-032) |
| Feedback volume + sentiment | partial | full | partial | analytics-reporting | Full (FR-033) |
| Calibration fairness (EEOC) | partial | gap | full | analytics-reporting + calibration | Full (capabilities/calibration-fairness-check.yaml) |
| CSV/Excel export (pack-redacted) | full | full | full | analytics-reporting | Full (FR-034) |
| Per-team breakdown | full | full | full | analytics-reporting | Full (capabilities/per-team-breakdown.yaml) |

## 10. Manager tools

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Manager dashboard | full | full | full | manager-tooling | Full (FR-030) |
| 1:1 prep packet | full | full | partial | manager-tooling | Full (FR-024) |
| Performance summary draft (LLM-assisted) | partial | partial | gap | manager-tooling | Full (FR-031) |
| Review form draft helper | partial | partial | partial | manager-tooling | Full (capabilities/review-draft-helper.yaml) |
| Team pulse view | full | full | partial | manager-tooling | Full (capabilities/team-pulse-view.yaml) |
| Goal coaching prompts | partial | partial | gap | manager-tooling | Full (capabilities/goal-coaching-prompts.yaml) |
| Feedback nudge | full | partial | gap | manager-tooling | Full (capabilities/feedback-nudge.yaml) |

## 11. Mobile + cross-platform

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Swift iOS app | full | full | full | mobile (SDK) | Full (FR-035) |
| Kotlin Android app | full | full | full | mobile (SDK) | Full (FR-035) |
| Mobile goal check-in | full | full | partial | mobile | Full (FR-035) |
| Mobile feedback give | full | full | partial | mobile | Full (FR-035) |
| Mobile recognition | full | partial | gap | mobile | Full (FR-035) |
| Mobile pulse response | full | full | partial | mobile | Full (FR-035) |
| Mobile 1:1 agenda read | full | full | gap | mobile | Full (FR-035) |
| Slack/Teams integration | full | full | partial | (workplace-integration substrate) | Full (substrate) |
| Email digest | full | full | full | (notifications substrate) | Full (substrate) |

## 12. Compliance + audit

| Capability | Lattice | 15Five | Workday Performance | Our context | Our coverage |
|---|---|---|---|---|---|
| Full audit trail of review/feedback | partial | partial | full | compliance | Full (FR-036) |
| Rating-change audit | partial | partial | full | compliance | Full (FR-036) |
| Calibration outcome audit | partial | gap | full | compliance | Full (FR-036) |
| Engagement release audit | partial | partial | partial | compliance | Full (FR-036) |
| GDPR DSAR support | partial | partial | full | compliance | Full (pack overlay) |
| Right-to-erasure | partial | partial | full | compliance | Full (pack overlay) |
| EU works-council consultation flow | gap | gap | partial | compliance | Full (pack `eu-worker-council`) |
| EEOC Title VII fairness check | partial | gap | full | compliance + calibration | Full (pack `us-labor`) |
| Korea PIPA controls | gap | gap | partial | compliance | Full (pack `kr-pipa`) |
| HIPAA workforce mode | gap | gap | partial | compliance | Full (pack `hipaa`) |

## 13. Coverage tally

| Section | Capabilities | Full | Partial | Gap |
|---|---:|---:|---:|---:|
| 1. Goals + OKR cascade | 9 | 9 | 0 | 0 |
| 2. Reviews + 360 | 13 | 13 | 0 | 0 |
| 3. Feedback + recognition | 8 | 8 | 0 | 0 |
| 4. One-on-one | 6 | 6 | 0 | 0 |
| 5. Check-ins | 4 | 4 | 0 | 0 |
| 6. Calibration | 7 | 7 | 0 | 0 |
| 7. Talent + succession | 10 | 10 | 0 | 0 |
| 8. Engagement + eNPS | 9 | 8 | 1 | 0 |
| 9. Analytics + reporting | 9 | 9 | 0 | 0 |
| 10. Manager tools | 7 | 7 | 0 | 0 |
| 11. Mobile + cross-platform | 9 | 9 | 0 | 0 |
| 12. Compliance + audit | 10 | 10 | 0 | 0 |
| **Total** | **101** | **100** | **1** | **0** |

Coverage: 100/101 Full + 1/101 Partial = **100% Full+Partial / 99% Full**. Big-8 P0 floor of
>=85% Full+Partial is met (margin: +15 percentage points). Net new capability YAMLs authored
in this wave: 26 (see `capabilities/` directory).

## 14. Migration

For tenants migrating from one of the three primary counterparts, we provide first-class
migration tooling at IP-027 (review-calibration-fairness-ledger) and the contracts in
`contracts/hr-handoff-*.asyncapi.yaml`:

| Counterpart | Migration path |
|---|---|
| Lattice | export-API import via ontology projection at IP-003; goal-cycle + review-cycle + feedback + 1:1 in-flight preserved; engagement-pulse history imported aggregate-only. |
| 15Five | export-API import; weekly check-in history imported; OKR + reviews + 1:1 in-flight preserved; HR Outcomes Dashboard equivalents auto-generated. |
| Workday Performance | XML/JSON export through Workday Studio integration; goal management + reviews + calibration + 9-box + succession imported; works-council pack auto-activated for EU tenants. |

All three counterparts use the same five-phase migration plan documented in PRD §J.

## 15. Migration risk and unsupported features

Counterpart features intentionally unsupported in Oyatie:

- **Lattice Compensation** module: delegated to `compensation` µservice.
- **15Five Strivescore** proprietary scoring: we publish raw signals; downstream tenants
  can compute their own composite.
- **Workday HCM Core**: delegated to `people-records` (HRIS).

## 16. Source citations

- Lattice product surface: public marketing pages + customer documentation; verified
  2026-05-21 via vendor-public-only sources.
- 15Five product surface: public marketing pages + customer documentation; verified
  2026-05-21.
- Workday Performance module: HCM module docs + analyst reports (Gartner, Forrester);
  verified 2026-05-21.

## 17. References

- PRD: `PRD.md` Section C, Section D
- README: `README.md`
- ARCHITECTURE: `ARCHITECTURE.md`
- Feature parity matrix: `feature-parity-matrix-2026-05-20.md`
- Audit: `coherence-audit-2026-05-20.md` Section 5
- Implementation plans: IP-026 to IP-037
