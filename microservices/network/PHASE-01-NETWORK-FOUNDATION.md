---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-foundation
phase: P01-network-foundation
status: Active
entry_gate: |
  ADR-0126 (parallel Connect dissolution) + ADR-0131 + ADR-0132 + ADR-0133 accepted; /specs/products/connect/network.json published;
  observability µservice IP-001..IP-015 merged so network can author OpenSLO manifests and pass promotion-readiness gate; sibling
  social µservice P01 merged (network reuses dual-context-isolation pattern + Cedar shape; cross-references social ADRs as
  paired-pattern references).
exit_gate: |
  All 15 IPs merged; all ~165 crates compile + nextest green; oya gate validate per-microservice-layout --microservice network
  exits 0; oya gate validate professional-context-isolation --microservice network exits 0; oya gate validate
  eu-ai-act-employment-conformance --microservice network exits 0; oya gate validate endorsement-chain-integrity --microservice
  network exits 0; HG-NETWORK gate registers green; end-to-end profile + connect + post + endorse + reaction + comment +
  notification + inmail + jobs-handoff drill passes within performance budget; pack-kr overlay deployed to dedicated network
  cluster.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: network requires observability gate + tenancy + ontology + audit-chain + cedar
  - milestone: M02-foundation
    phase: P01-social-foundation
    reason: network reuses sibling social's dual-context-isolation pattern + Cedar shape (cross-reference, not import)
  - milestone: M02-foundation
    phase: P01-messenger-foundation
    reason: network's InMail-bridge BC depends on messenger µservice's Professional-tier surface being live (P01)
owner_team: axis-network
related_adrs: [ADR-0008, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0133, ADR-0134]
related_specs: [/specs/products/connect/network.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
doc_status: published
---

# P01-network-foundation: Land the network µservice end-to-end (first-phase scope)

## Purpose

This phase ships the foundation of the network µservice per parallel ADR-0126 + ADR-0132 + ADR-0133: professional-profile + professional-graph + connection-request + post-composition + feed-timeline + reactions + mentions + hashtags + trending-topics + notifications + content-moderation + abuse-reporting + search + skill-assessments + profile-verification + pages + groups + events-bridge + jobs-handoff + inmail-bridge + endorsement-engine + accessibility-captions + recruiter-stub (OFF) + services-marketplace-stub (OFF) + learning-stub (OFF) + salary-insights-stub.

It advances master-plan principles:
- Hyperscaler-grade in every practice (LinkedIn / Xing / Wantedly / Glassdoor / Indeed parity + native Workflow + Ontology integration).
- Nothing deferred within scope (no FUTURE stubs in scope; every NFR covered; stubs that are explicitly OFF-by-default are scoped via dedicated ADR follow-ups).
- No silent regression (production-tier change gated by observability ADR-0130).
- Per-microservice flat layout (ADR-0131 native authoring).
- Professional-context isolation by data model (NOT runtime flag) per parallel ADR-0126; reuses social pattern; never federates to Personal-tier.
- EU AI Act Annex III §4 (employment, workers management, access to self-employment) high-risk obligations operative from day-1 per ADR-NET-0002.

## Scope

### In-scope (first phase)

| µservice | Bounded Contexts | Crate count |
|---|---|---|
| `network` | `professional-profile`, `professional-graph`, `connection-request`, `post-composition`, `feed-timeline`, `reactions`, `mentions`, `hashtags`, `trending-topics`, `notifications`, `inmail-bridge`, `endorsement-engine`, `skill-assessments`, `profile-verification`, `pages`, `groups`, `events-bridge`, `jobs-handoff`, `recruiter-stub` (OFF), `services-marketplace-stub` (OFF), `learning-stub` (OFF), `salary-insights-stub`, `search`, `accessibility-captions`, `abuse-reporting` | ~165 crates |

Plus cross-cutting:
- `.github/branch-protection.yaml` — add `release/network/*` pattern protection.
- `/specs/hyperscaler-gates.json` — register HG-NETWORK per ADR-0133.
- `Cargo.toml` (workspace) — register ~165 crates.
- `docs/standards/professional-context-isolation.md` — overlay reference (cross-cutting; per parallel ADR-0126; reuses social's authoritative dual-context-isolation.md).

### Out-of-scope (deferred to follow-up phases or follow-up ADRs)

- **Federation (ActivityPub / AT Protocol Professional-tier)** — `network` does NOT federate in P01; Professional Network is Professional-only and Personal-tier never crosses in. Federation deferred to ADR-NET follow-up if demand emerges (PRD Open Question 6).
- **Recruiter-tooling-stub activation** — OFF by default; activation requires per-tenant opt-in + NYC Local Law 144 bias-audit attestation + CA AB-331 transparency hook. Activation deferred to ADR-NET follow-up after M03 (PRD Open Question 1).
- **Services-marketplace-stub activation** — stubbed (off by default); deferred to ADR-NET follow-up after M04 (PRD Open Question 2).
- **Learning-stub activation** — stubbed (off by default); deferred to ADR-NET follow-up after M05 (PRD Open Question 3).
- **ML-driven feed ranking + recruiter ranking** — P01 ships chronological-first + heuristic-algorithmic ranking; ML-driven ranking deferred to P03 (depends on foundry-runtime model deployment + EU AI Act notified-body engagement). Parallel to social ADR-SOC-0001 staging strategy.
- **Voice / video post type (live-streaming + audio-rooms)** — deferred to follow-up sibling µservice if demand surfaces.
- **AT Protocol Professional federation** — deferred (PRD Open Question 6).
- **Salary-insights data sourcing** — stub only in P01; aggregate-only; per-individual disclosure forbidden; sourcing strategy deferred to PRD Open Question 7.

## Implementation Plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-iac-bootstrap.md`](IP-001-iac-bootstrap.md) | Helm/Kustomize/Terraform for network cluster; Postgres + Redis + Meilisearch + S3 + ClamAV/OPSWAT + ImageMagick + ffmpeg | pending | axis-network + ops-sre-reliability | observability IP-001 |
| [`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md) | Cargo workspace + ~165 crate scaffolds per ADR-0131 | pending | axis-network | — |
| [`IP-003-professional-profile-bc.md`](IP-003-professional-profile-bc.md) | `professional-profile` kernel + domain + usecase + api + adapter-postgres + rest + sdk + app; resume sections; vCard 4.0 + JSON Resume export | pending | axis-network | IP-002 |
| [`IP-004-professional-graph-and-connection-request-bcs.md`](IP-004-professional-graph-and-connection-request-bcs.md) | `professional-graph` + `connection-request` BCs end-to-end; degree-of-separation; per-week rate limit | pending | axis-network | IP-003 |
| [`IP-005-post-composition-bc.md`](IP-005-post-composition-bc.md) | `post-composition` BC end-to-end + media transcode adapters + document attach | pending | axis-network | IP-003 |
| [`IP-006-feed-timeline-and-reactions-bcs.md`](IP-006-feed-timeline-and-reactions-bcs.md) | `feed-timeline` BC with fanout-on-write + fanout-on-read + Redis hot-cache; `reactions` BC with extended Professional set | pending | axis-network | IP-004 + IP-005 |
| [`IP-007-endorsement-engine-bc.md`](IP-007-endorsement-engine-bc.md) | `endorsement-engine` BC; Ed25519 chain via audit-chain; per-endorser signature; revocation flow | pending | axis-network + axis-audit-chain | IP-003 |
| [`IP-008-skill-assessments-and-profile-verification-bcs.md`](IP-008-skill-assessments-and-profile-verification-bcs.md) | `skill-assessments` + `profile-verification` BCs (ID-attest + employer-confirm) | pending | axis-network + council-privacy | IP-003 |
| [`IP-009-pages-groups-events-bcs.md`](IP-009-pages-groups-events-bcs.md) | `pages` + `groups` + `events-bridge` BCs together; mail + calendar bridges | pending | axis-network + axis-mail + axis-calendar | IP-005 |
| [`IP-010-inmail-bridge-bc.md`](IP-010-inmail-bridge-bc.md) | `inmail-bridge` BC; Professional-tier-only routing to messenger µservice; rate-limit + spam-classifier | pending | axis-network + axis-messenger | IP-004 |
| [`IP-011-jobs-handoff-bc.md`](IP-011-jobs-handoff-bc.md) | `jobs-handoff` BC; contract-versioned event handoff to ATS µservice; jobs-search facets | pending | axis-network + axis-ats | IP-005 |
| [`IP-012-mentions-hashtags-trending-notifications-bcs.md`](IP-012-mentions-hashtags-trending-notifications-bcs.md) | `mentions` + `hashtags` + `trending-topics` + `notifications` BCs together | pending | axis-network | IP-005 + IP-006 |
| [`IP-013-search-and-cedar-filter.md`](IP-013-search-and-cedar-filter.md) | `search` BC with Meilisearch + faceted index + Cedar post-filter | pending | axis-network | IP-005 + IP-009 + IP-011 |
| [`IP-014-recommender-fairness-and-bias-lane.md`](IP-014-recommender-fairness-and-bias-lane.md) | EU AI Act + EEOC bias-audit lane wiring; `oya-check-eu-ai-act-employment-conformance` lane; OpenSLO recommender-fairness manifest; bias dashboards | pending | axis-network + axis-foundry-runtime + council-privacy + ops-compliance | IP-006 + IP-007 + IP-011 |
| [`IP-015-hg-network-registration-and-branch-protection.md`](IP-015-hg-network-registration-and-branch-protection.md) | HG-NETWORK hyperscaler-grade conformance gate per ADR-0133 + branch-protection | pending | axis-network + council-architecture | IP-014 |

## Per-IP Test Coverage Threshold

| Class | Coverage line / branch | Test types required |
|---|---|---|
| kernel | 90 % / 80 % | per-port-trait + per-entity unit; sealed-trait smoke; data-class annotation check |
| domain | 90 % / 80 % | pure-math / pure-logic unit |
| usecase | 85 % / 75 % | orchestrator unit with port mocks; happy + error path |
| adapter | 80 % / 70 % | integration vs real backend (Postgres / Redis / S3 / Meilisearch / ClamAV / messenger-bridge / calendar-bridge / mail-bridge / ats-bridge) where feasible; otherwise contract-mock |
| rest | 85 % / 75 % | per-endpoint happy + 401 + 403 + 422 |
| worker | 85 % / 75 % | event-loop unit + integration |
| app | 75 % / 65 % | smoke startup |

E2E: ≥ 1 per AC-NN row in PRD (25 e2e tests minimum).

## Phase-Gate Verification Bundle

Required CI lanes green on every commit + on phase-exit:

- `oya gate validate per-microservice-layout --microservice network`
- `oya gate validate professional-context-isolation --microservice network`
- `oya gate validate authority-cohesion --microservice network` (HG-NETWORK)
- `oya gate validate hyperscaler-maturity-claims --microservice network`
- `oya gate validate shardability --microservice network`
- `oya gate validate statelessness --microservice network`
- `oya gate validate layer-correctness --microservice network`
- `oya gate validate port-location --microservice network`
- `oya gate validate bnf-v4-1 --microservice network`
- `oya gate validate cedar-policy-spec --microservice network`
- `oya gate validate eu-ai-act-employment-conformance --microservice network`
- `oya gate validate endorsement-chain-integrity --microservice network`
- `oya gate validate jobs-handoff-contract --microservice network`
- `oya gate validate version-pinning-conformance` (LTS pins for Postgres/Redis/Meilisearch/ClamAV/OPSWAT/ImageMagick/ffmpeg)
- `oya gate validate compliance-evidence-recency --microservice network`

## Phase Exit Bundle

1. All 15 IPs merged.
2. All ~165 crates `cargo nextest` green; coverage thresholds met per class.
3. End-to-end drill in pack-kr cluster: profile-create → connection-request → accept → post → repost → comment → reaction → endorsement → recommendation → InMail-send → jobs-handoff → notification → moderation-verdict → appeal completes within performance envelope.
4. Capacity tier XS deployed: 20 tenants, ~1M Professional MAU, ~500 post/sec sustained, OpenSLO burn-rate green for 7 days.
5. Bias-audit lane green: 4/5-rule disparity ratio for recruiter-stub ranker (synthetic golden-set; real production deployment requires recruiter-stub activation per ADR-NET-0002).
6. Postmortem + sign-off by council-architecture, ops-security, council-privacy, ops-compliance, axis-network lead.
