---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P18-IP-003
title: Webhook receiver substrate + event router
status: scaffolded
tier: S
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
final_shape_compliance: true
dependency_additions:
  - axum (Rust HTTP framework) OR hyper-http-tower — decide at impl
source_adr: ../../../../../../docs/decisions/ADR-0112-webhook-driven-foundry-agent-invocation.md
purpose: Land the HTTP webhook receiver that turns the pipeline from poll-driven to event-driven — HMAC-verified, dedup'd, routed to Foundry agents per a canonical event-router table.
---

# M01-P18-IP-003 — Webhook receiver substrate + event router

## Scope

Implement ADR-0112 wave-A:

- New kernel `oya-vcs-webhook-receiver-kernel` — pure-domain
  HMAC verification + dedup-table parser. No HTTP.
- New app `oya-vcs-webhook-receiver-app` — HTTP receiver at
  `/webhook/github`. Verifies `X-Hub-Signature-256` HMAC against
  OpenBao-stored secret. Dedups by `X-GitHub-Delivery` against
  `registry/vcs/webhook-delivery-log.json` (7-day TTL). Routes per
  `registry/vcs/event-router.yaml`. Posts back via `gh api`.
- Initial 9-row event-router table seeded.
- `--simulate-delivery` test harness for local integration tests
  without exposing a public endpoint.

Bounded latency: p50 < 500 ms, p99 < 5 s. MAX_RETRIES = 3 with
fresh idempotency keys (`<delivery_id>:retry:<n>`).

## Dependencies

None for the kernel; the app provisions a webhook on the repo
which requires repo admin access at deploy time.

## Acceptance

- HMAC verification kernel passes 5 unit tests covering: valid sig
  accepted, invalid sig rejected, missing header rejected,
  malformed secret rejected, fail-closed on parser error.
- Dedup-table parser passes 4 unit tests covering: first delivery
  accepted, redelivery deduped, expired entry GCed at 7d,
  conflicting outcome on same delivery_id surfaced as anomaly.
- HTTP app `/webhook/github` endpoint accepts a synthetic
  `pull_request.opened` delivery (via `--simulate-delivery
  pr-opened.json`) and routes to `oya-foundry-vcs-orchestrator-app`
  (stubbed if IP-004 not yet landed).
- `registry/vcs/event-router.yaml` initialized with the 9 rows
  from ADR-0112.
- `registry/vcs/webhook-delivery-log.json` initialized empty
  append-only.
- Webhook secret stored at
  `sref://openbao/oya/foundry/github-webhook-secret`; access
  validated via OpenBao smoke probe.

## Symbols to grit-claim

- `crates/oya-vcs-webhook-receiver-kernel/src/lib.rs::*`
- `tools/oya-vcs-webhook-receiver-app/src/main.rs::main`
- `tools/oya-vcs-webhook-receiver-app/src/{routing,dedup,hmac,postback}.rs::*`
- `registry/vcs/event-router.yaml::*`
- `registry/vcs/webhook-delivery-log.json::*` (empty)

## Exit evidence

- `/evidence/agentic-vcs-pipeline/ip-003-webhook-receiver.json`
- `/evidence/agentic-vcs-pipeline/ip-003-simulate-delivery-smoke.json`
- `/evidence/agentic-vcs-pipeline/ip-003-hmac-fail-closed.json`
