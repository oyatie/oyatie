---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-foundation
phase: P01-team-channels-dm-threads-presence
status: Active
entry_gate: |
  ADR-0135 (parallel Connect dual-context) + ADR-0131 + ADR-0132 accepted; /specs/microservices/messenger.json published;
  observability µservice IP-001..IP-015 merged so messenger can author OpenSLO manifests and pass promotion-readiness gate.
exit_gate: |
  All 15 IPs merged; all 52 crates compile + nextest green; oya gate validate per-microservice-layout --microservice messenger
  exits 0; oya gate validate dual-context-isolation --microservice messenger exits 0; HG-MESSENGER gate registers green;
  end-to-end channel-create + post + thread-reply + presence + search + four-eyes-disclosure drill passes within performance budget;
  pack-kr overlay deployed to dedicated messenger cluster.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: messenger requires observability gate + tenancy + ontology + audit-chain + cedar
owner_team: axis-messenger
related_adrs: [ADR-0008, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/microservices/messenger.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
doc_status: published
---

# P01-team-channels-dm-threads-presence: Land the messenger µservice end-to-end

## Purpose

This phase ships the full messenger µservice per parallel ADR-0238 + ADR-0132: team channels + DMs + threads + read-receipts + file-sharing + reactions + @mentions + channel-level RBAC + message search + presence, dual-context-safe across personal (B2C) and professional (B2B).

It advances master-plan principles:
- Hyperscaler-grade in every practice (Slack/Teams-class feature parity + native Workflow + Ontology integration).
- Nothing scheduled-for-distinct-tracked-work (no FUTURE stubs; every NFR covered).
- No silent regression (production-tier change gated by observability ADR-0139).
- Per-microservice flat layout (ADR-0131 native authoring).
- Dual-context isolation by data model (NOT runtime flag) per parallel ADR-0238.

## Scope

### In-scope

| µservice | Bounded Contexts | Crate count |
|---|---|---|
| `messenger` | `channel-store`, `message-stream`, `thread-tree`, `read-receipt-tracker`, `file-attachment`, `mention-router`, `presence` | 52 crates |

Plus cross-cutting:
- `.github/branch-protection.yaml` — add `release/messenger/*` pattern protection.
- `/specs/hyperscaler-gates.json` — register HG-MESSENGER per ADR-0133.
- `Cargo.toml` (workspace) — register 52 crates.
- `docs/standards/dual-context-isolation.md` (NEW; cross-cutting per parallel ADR-0238).

### Out-of-scope

- Voice / video signaling (Open Question 2; successor-IP ADR + µservice).
- External Slack / Teams federation (Open Question 3; per-tenant opt-in adapter).
- Personal-DM E2E key escrow policy (Open Question 5; awaiting ADR).
- Workflow Studio shell integration UX (owned by `workflow-studio` µservice's PRD).

## Implementation Plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-websocket-gateway-iac.md`](IP-001-websocket-gateway-iac.md) | Helm/Kustomize for WebSocket gateway + Envoy / Cloudflare termination | pending | axis-messenger | observability IP-001 |
| [`IP-002-postgres-message-store-iac.md`](IP-002-postgres-message-store-iac.md) | Postgres HA + per-tenant sharding for message + channel + thread | pending | axis-messenger | observability IP-001 |
| [`IP-003-redis-presence-iac.md`](IP-003-redis-presence-iac.md) | Valkey cluster for presence + read-receipt; pub/sub topology | pending | axis-messenger | observability IP-001 |
| [`IP-004-attachment-store-iac.md`](IP-004-attachment-store-iac.md) | S3-compatible blob store; multipart upload; KMS encryption | pending | axis-messenger | observability IP-001 |
| [`IP-005-search-index-iac.md`](IP-005-search-index-iac.md) | Tantivy (self-hosted) + optional Elasticsearch fallback | pending | axis-messenger | observability IP-001 |
| [`IP-006-channel-store-kernel.md`](IP-006-channel-store-kernel.md) | `oya-messenger-channel-store-kernel` port traits + entities | pending | axis-messenger | — |
| [`IP-007-message-stream-kernel.md`](IP-007-message-stream-kernel.md) | `oya-messenger-message-stream-kernel` port traits + entities | pending | axis-messenger | IP-006 |
| [`IP-008-thread-tree-kernel.md`](IP-008-thread-tree-kernel.md) | `oya-messenger-thread-tree-kernel` | pending | axis-messenger | IP-007 |
| [`IP-009-read-receipt-tracker-kernel.md`](IP-009-read-receipt-tracker-kernel.md) | `oya-messenger-read-receipt-tracker-kernel` | pending | axis-messenger | IP-007 |
| [`IP-010-file-attachment-kernel.md`](IP-010-file-attachment-kernel.md) | `oya-messenger-file-attachment-kernel` | pending | axis-messenger | IP-007 |
| [`IP-011-mention-router-kernel.md`](IP-011-mention-router-kernel.md) | `oya-messenger-mention-router-kernel` | pending | axis-messenger | IP-007 |
| [`IP-012-presence-kernel.md`](IP-012-presence-kernel.md) | `oya-messenger-presence-kernel` | pending | axis-messenger | IP-006 |
| [`IP-013-rest-and-websocket-surface.md`](IP-013-rest-and-websocket-surface.md) | `*-rest` + `*-app` composition root | pending | axis-messenger | IP-006..IP-012 |
| [`IP-014-observability-slo.md`](IP-014-observability-slo.md) | OpenSLO manifests + dashboards + per-pack runbooks wiring | pending | axis-messenger + axis-observability | IP-013 |
| [`IP-015-hg-messenger-conformance.md`](IP-015-hg-messenger-conformance.md) | HG-MESSENGER hyperscaler-grade conformance gate per ADR-0133 | pending | axis-messenger + council-architecture | IP-013 |

## Per-IP Test Coverage Threshold

| Class | Coverage line / branch | Test types required |
|---|---|---|
| kernel | 90 % / 80 % | per-port-trait + per-entity unit; sealed-trait smoke; data-class annotation check |
| domain | 90 % / 80 % | pure-math / pure-logic unit |
| usecase | 85 % / 75 % | orchestrator unit with port mocks; happy + error path |
| adapter | 80 % / 70 % | integration vs real backend (Postgres / Valkey / S3 / Tantivy) where feasible; otherwise contract-mock |
| rest | 85 % / 75 % | per-endpoint happy + 401 + 403 + 422 |
| worker | 85 % / 75 % | event-loop unit + integration |
| app | 75 % / 65 % | smoke startup |

E2E: ≥ 1 per AC-NN row in PRD.
