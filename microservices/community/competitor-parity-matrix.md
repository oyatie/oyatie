---
doc_class: CompetitorParityMatrix
template_id: TPL-COMPETITOR-PARITY-MATRIX
microservice: community
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0056, ADR-0135, ADR-0131, ADR-0133]
doc_status: published
---

# Competitor Parity Matrix: community µservice

Reference benchmarks per parallel-session ADR-0135 + `feedback_quality_performance_scalability_bar.md`. Targets: Atlassian Community + Microsoft Yammer/Viva Engage + Salesforce Community Cloud + Discourse + Slack channels + Stack Overflow Teams.

## Feature Parity

| Feature | Atlassian | MS Viva Engage | Salesforce Community | Discourse | Slack | SOF Teams | **oyatie community** |
|---|---|---|---|---|---|---|---|
| Org-wide announcements | ✅ | ✅ | ✅ | ✅ | partial | partial | ✅ |
| Q&A with accepted answers | ✅ | partial | ✅ | partial | ❌ | ✅ | ✅ |
| Threaded discussion forums | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ |
| Knowledge-base articles | partial | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ |
| Voting (up/down/score) | partial | partial | ✅ | ✅ | reactions | ✅ | ✅ |
| Moderation queue + actions | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ |
| Cross-content search | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Subscriptions / notifications | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Mentions (`@user`) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tags / taxonomy | ✅ | partial | ✅ | ✅ | partial | ✅ | ✅ |
| Public-read spaces | ✅ | ❌ | ✅ | ✅ | ❌ | partial | ✅ |
| Tenant-scoped audit log | partial | ✅ | ✅ | partial | partial | ✅ | ✅ (audit-chain) |
| Cross-product entity links | ❌ | partial (M365) | ✅ (Salesforce) | ❌ | partial | ❌ | ✅ (ontology) |
| AI-summarised threads | partial | ✅ (Copilot) | partial | ❌ | ✅ (AI) | partial | M03 (foundry-runtime) |
| API + SDK | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Multi-region data residency | partial | ✅ | ✅ | self-host | partial | partial | ✅ |
| HIPAA-ready (BAA) | partial | ✅ | ✅ | self-host | partial | ✅ | ✅ (pack-us-healthcare) |
| SOC 2 + ISO 27001 | ✅ | ✅ | ✅ | partial | ✅ | ✅ | ✅ |
| GDPR + KR PIPA + APPI | partial | ✅ | ✅ | self-host | partial | partial | ✅ |
| Open-source code | ❌ | ❌ | ❌ | ✅ | ❌ | ❌ | partial (substrate Bominal-inherited) |
| Federated cross-org | ❌ | partial | partial | ❌ | partial | ❌ | M04 |
| Open standards (ActivityPub etc.) | ❌ | ❌ | ❌ | partial | ❌ | ❌ | M04 evaluate |

## Performance Parity

| Metric | Atlassian | Discourse | SOF Teams | **oyatie community** |
|---|---|---|---|---|
| Feed render p99 | ~800 ms | ~400 ms | ~300 ms | **300 ms** |
| Search p99 | ~1 s | ~600 ms | ~500 ms | **500 ms** |
| Vote cast p99 | ~200 ms | ~100 ms | ~150 ms | **100 ms** |
| Post create p99 | ~400 ms | ~300 ms | ~250 ms | **250 ms** |

## Differentiators

- **Cross-product entity links via ontology** — community thread can deep-link to an entity instance; competitors don't have a typed ontology substrate.
- **Foundry-guardrails moderation bridge** — classifier-driven spam / abuse detection with tenant-tunable thresholds; first-party.
- **audit-chain Ed25519 sealing** — every moderation action is provably auditable; SOC 2 / GDPR / KR PIPA / HIPAA in one substrate.
- **Per-region data residency by default** — competitors offer "EU residency" as opt-in; oyatie is per-region by design.
- **SLO + promotion gating** — community release pointer advances only when burn-rate green; competitive surfaces don't gate releases on community SLO.

## Gaps (acknowledged)

- AI-summarised threads: M03 via foundry-runtime.
- Federated cross-org: M04.
- Marketplace of community add-ons: M04.

## Update Cadence

Quarterly competitive sweep. Per `feedback_no_silent_regression.md`, regressions vs. previous matrix are blocking.
