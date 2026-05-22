---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-foundation
phase: P01-social-foundation
status: Active
entry_gate: |
  ADR-0135 (parallel Connect dissolution) + ADR-0131 + ADR-0132 accepted; /specs/microservices/social.json published;
  observability µservice IP-001..IP-015 merged so social can author OpenSLO manifests and pass promotion-readiness gate.
exit_gate: |
  All 15 IPs merged; all ~115 crates compile + nextest green; oya gate validate per-microservice-layout --microservice social
  exits 0; oya gate validate dual-context-isolation --microservice social exits 0; HG-SOCIAL gate registers green;
  end-to-end profile + post + follow + feed-render + reaction + comment + notification drill passes within performance budget;
  pack-kr overlay deployed to dedicated social cluster.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: social requires observability gate + tenancy + ontology + audit-chain + cedar
owner_team: axis-social
related_adrs: [ADR-0008, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/microservices/social.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
doc_status: published
---

# P01-social-foundation: Land the social µservice end-to-end (first-phase scope)

## Purpose

This phase ships the foundation of the social µservice per parallel ADR-0238 + ADR-0132: user-profile + follow-graph + post-composition + feed-timeline + reactions + mentions + hashtags + trending-topics + notifications + content-moderation + search + age-verification + profile-verification + lists + bookmarks + abuse-reporting + appeal-workflow, dual-context-safe across Personal (B2C) and Professional (B2B).

It advances master-plan principles:
- Hyperscaler-grade in every practice (Twitter/X / Mastodon / Bluesky parity + native Workflow + Ontology integration).
- Nothing scheduled-for-distinct-tracked-work within scope (no FUTURE stubs; every NFR covered).
- No silent regression (production-tier change gated by observability ADR-0139).
- Per-microservice flat layout (ADR-0131 native authoring).
- Dual-context isolation by data model (NOT runtime flag) per parallel ADR-0238.

## Scope

### In-scope (first phase)

| µservice | Bounded Contexts | Crate count |
|---|---|---|
| `social` | `user-profile`, `follow-graph`, `post-composition`, `feed-timeline`, `reactions`, `mentions`, `hashtags`, `trending-topics`, `notifications`, `content-moderation`, `bookmarks`, `lists`, `search`, `profile-verification`, `age-verification` | ~108 crates |

Plus cross-cutting:
- `.github/branch-protection.yaml` — add `release/social/*` pattern protection.
- `/specs/hyperscaler-gates.json` — register HG-SOCIAL per ADR-0133.
- `Cargo.toml` (workspace) — register ~108 crates.
- `docs/standards/dual-context-isolation.md` (already authored cross-cutting per parallel ADR-0238).

### Out-of-scope (scheduled-for-distinct-tracked-work to successor-IP phases)

- **Federation (ActivityPub)** — scheduled-for-distinct-tracked-work to P02; opt-in per tenant; Personal-tier never federates. ADR-SOC-0004 establishes posture; impl follows.
- **Ranking model (ML-driven algorithmic feed)** — P01 ships chronological-first + simple recency-and-engagement heuristic ranking; ML-driven ranking scheduled-for-distinct-tracked-work to P03 (depends on `foundry-runtime` model deployment).
- **Ads-substrate-stub** — interface-only-pending-impl (off by default); not activated in P01. ADR successor-IP after M03.
- **Voice / video post type (stories-style)** — scheduled-for-distinct-tracked-work to successor-IP sibling µservice if demand surfaces.
- **AT Protocol federation (Bluesky)** — scheduled-for-distinct-tracked-work (Open Question 2 in PRD).

## Implementation Plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-iac-bootstrap.md`](IP-001-iac-bootstrap.md) | Helm/Kustomize/Terraform for social cluster; Postgres + Valkey + Meilisearch + S3 + ClamAV/OPSWAT + ImageMagick + ffmpeg | pending | axis-social + ops-sre-reliability | observability IP-001 |
| [`IP-002-cargo-workspace-bootstrap.md`](IP-002-cargo-workspace-bootstrap.md) | Cargo workspace + ~108 crate scaffolds per ADR-0131 | pending | axis-social | — |
| [`IP-003-user-profile-bc.md`](IP-003-user-profile-bc.md) | `user-profile` kernel + domain + usecase + api + adapter-postgres + rest + sdk + app | pending | axis-social | IP-002 |
| [`IP-004-follow-graph-bc.md`](IP-004-follow-graph-bc.md) | `follow-graph` BC end-to-end | pending | axis-social | IP-003 |
| [`IP-005-post-composition-bc.md`](IP-005-post-composition-bc.md) | `post-composition` BC end-to-end + media transcode adapters | pending | axis-social | IP-003 |
| [`IP-006-feed-timeline-bc.md`](IP-006-feed-timeline-bc.md) | `feed-timeline` BC with fanout-on-write + fanout-on-read + Valkey hot-cache | pending | axis-social | IP-004 + IP-005 |
| [`IP-007-reactions-bc.md`](IP-007-reactions-bc.md) | `reactions` BC; conflict-free counter; Valkey-buffered + Postgres flush | pending | axis-social | IP-005 |
| [`IP-008-mentions-and-hashtags-bc.md`](IP-008-mentions-and-hashtags-bc.md) | `mentions` + `hashtags` BCs together; Ontology client + topic emission | pending | axis-social | IP-005 |
| [`IP-009-trending-topics-bc.md`](IP-009-trending-topics-bc.md) | `trending-topics` BC with windowed compute | pending | axis-social | IP-008 |
| [`IP-010-notifications-bc.md`](IP-010-notifications-bc.md) | `notifications` BC; real-time WebSocket + digest worker; cross-µservice messenger bridge | pending | axis-social + axis-messenger | IP-006 |
| [`IP-011-content-moderation-bc.md`](IP-011-content-moderation-bc.md) | `content-moderation` BC; classifier client (foundry-runtime T2); abuse-reporting + appeal-workflow | pending | axis-social + axis-foundry-runtime | IP-005 |
| [`IP-012-search-and-cedar-filter.md`](IP-012-search-and-cedar-filter.md) | `search` BC with Meilisearch + Cedar post-filter | pending | axis-social | IP-005 |
| [`IP-013-age-verification-and-profile-verification.md`](IP-013-age-verification-and-profile-verification.md) | `age-verification` + `profile-verification` BCs together | pending | axis-social + council-privacy | IP-003 |
| [`IP-014-observability-slo.md`](IP-014-observability-slo.md) | OpenSLO manifests + dashboards + per-pack runbooks wiring | pending | axis-social + axis-observability | IP-006..IP-012 |
| [`IP-015-hg-social-registration-and-branch-protection.md`](IP-015-hg-social-registration-and-branch-protection.md) | HG-SOCIAL hyperscaler-grade conformance gate per ADR-0133 + branch-protection | pending | axis-social + council-architecture | IP-014 |

## Per-IP Test Coverage Threshold

| Class | Coverage line / branch | Test types required |
|---|---|---|
| kernel | 90 % / 80 % | per-port-trait + per-entity unit; sealed-trait smoke; data-class annotation check |
| domain | 90 % / 80 % | pure-math / pure-logic unit |
| usecase | 85 % / 75 % | orchestrator unit with port mocks; happy + error path |
| adapter | 80 % / 70 % | integration vs real backend (Postgres / Valkey / S3 / Meilisearch / ClamAV) where feasible; otherwise contract-mock |
| rest | 85 % / 75 % | per-endpoint happy + 401 + 403 + 422 |
| worker | 85 % / 75 % | event-loop unit + integration |
| app | 75 % / 65 % | smoke startup |

E2E: ≥ 1 per AC-NN row in PRD.

## Phase-Gate Verification Bundle

Required CI lanes green on every commit + on phase-exit:

- `oya gate validate per-microservice-layout --microservice social`
- `oya gate validate dual-context-isolation --microservice social`
- `oya gate validate authority-cohesion --microservice social` (HG-SOCIAL)
- `oya gate validate hyperscaler-maturity-claims --microservice social`
- `oya gate validate shardability --microservice social`
- `oya gate validate statelessness --microservice social`
- `oya gate validate layer-correctness --microservice social`
- `oya gate validate port-location --microservice social`
- `oya gate validate bnf-v4-1 --microservice social`
- `oya gate validate cedar-policy-spec --microservice social`
- `oya gate validate version-pinning-conformance` (LTS pins for Postgres/Valkey/Meilisearch/ClamAV/OPSWAT/ImageMagick/ffmpeg)
- `oya gate validate compliance-evidence-recency --microservice social`

## Phase Exit Bundle

1. All 15 IPs merged.
2. All ~108 crates `cargo nextest` green; coverage thresholds met per class.
3. End-to-end drill in pack-kr cluster: profile-create → post → repost → comment → reaction → follow → feed-render → notification → moderation-verdict → appeal completes within performance envelope.
4. Capacity tier XS deployed: 20 tenants, ~500k MAU, ~1k post/sec sustained, OpenSLO burn-rate green for 7 days.
5. Postmortem + sign-off by council-architecture, ops-security, council-privacy, axis-social lead.
