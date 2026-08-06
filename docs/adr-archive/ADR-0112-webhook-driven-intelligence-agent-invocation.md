---
status: Superseded
deciders: council-foundry, council-foundry-vcs, council-security
date: 2026-05-16
owner: council-foundry-vcs
supersedes: []
superseded_by: [ADR-0363]
related:
  - ADR-0110-changeset-state-machine.md
  - ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
  - ADR-0113-vcs-orchestrator-end-to-end.md
  - ADR-0058-commit-signing-via-ssh-and-ed25519.md
purpose: Define the webhook-driven Foundry-agent invocation substrate (HTTP receiver, HMAC verify, dedup, retry, post-back) that makes the agentic pipeline event-driven instead of poll-driven.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0112: Webhook-driven Foundry agent invocation

## Context

The agentic pipeline (per ADR-0110/0111) is event-driven by design.
Today, the pieces that should react to events poll instead:

- IP-004 (pr-review dispatcher) fires via GitHub Actions
  `workflow_run:` trigger — a coarse, GitHub-controlled poll that
  fires once per workflow completion. Latency ~30s; granularity
  per-workflow-run.
- IP-005 (CI fix-loop) same model — only fires on workflow_run
  failure events.
- IP-006 (merge-queue) reads admission-log files via filesystem
  scans (per the merge-queue fix-loop tick loop).
- Promotion workflows (dev→staging→production, ADR-0110 wave-B)
  trigger on `push:` + cron schedules.

None of these:
- React to per-commit events (e.g., "agent pushed a fix to PR-B's
  branch — re-validate the queue").
- Dedup duplicate deliveries (GitHub redelivers webhooks on
  failure).
- Carry the changeset_id context across event boundaries.
- Update the changeset-event-log (per ADR-0110) on every transition.

ADR-0112 locks the webhook-receiver substrate so all of the above
become async + correlated + idempotent.

## Decision

A new webhook-receiver app receives GitHub
webhook deliveries and routes them to Foundry agents.

### Receiver shape

- HTTP endpoint exposed at `/webhook/github` (hosted on the
  Foundry control plane — initial deployment is the existing
  Anthropic-API substrate; future deployments may move to a
  dedicated mesh service).
- Verifies `X-Hub-Signature-256` HMAC against the webhook secret
  stored in OpenBao at `sref://openbao/oya/foundry/github-webhook-secret`
  (per the SecretReference contract).
- Dedups by `X-GitHub-Delivery` header (GitHub-supplied UUID,
  unique per delivery, stable across redeliveries). Dedup table
  is the `registry/vcs/webhook-delivery-log.json` event-sourced
  log with TTL = 7 days.
- Routes by `X-GitHub-Event` + payload `action` to a registered
  Foundry agent per the event-router table.
- Posts the routed result back to GitHub (commit, comment,
  check_run, merge-queue admission, etc) via `gh api`.

### Event-router table

The router is a closed mapping from `(event, action)` to the
agent that handles it:

| GitHub event | action | Foundry agent | Purpose |
|---|---|---|---|
| `pull_request` | `opened` | orchestrator | Begin changeset state transition to `pr_open` |
| `pull_request` | `synchronize` | merge-queue + IP-005 | Fix-at-any-stage re-validate (ADR-0111) |
| `pull_request` | `closed` | orchestrator | If `merged=true`, transition to `merged_dev` |
| `workflow_run` | `completed` (conclusion=success) | IP-004 dispatcher | Run multispectrum review |
| `workflow_run` | `completed` (conclusion=failure) | IP-005 dispatcher | Run fix-loop with retry budget |
| `check_suite` | `completed` | merge-queue | Re-evaluate queue (some checks landed) |
| `push` (to `dev`) | — | promotion workflow | Trigger dev→staging fast-forward |
| `push` (to `staging`) | — | promotion workflow | Trigger staging→production fast-forward |
| `pull_request_review` | `submitted` | orchestrator | Update changeset state if human override |

The table is canonical config at
`registry/vcs/event-router.yaml`. New events MUST go through ADR
amendment (no silent additions).

### Idempotency contract

Every webhook delivery has:
- `delivery_id` = `X-GitHub-Delivery` header value (GitHub-issued
  UUIDv4).
- `dedup_outcome` recorded in
  `registry/vcs/webhook-delivery-log.json`:
  - `accepted` (first time we saw this delivery_id; routing fired)
  - `deduplicated` (we've seen this delivery_id before; no-op)
  - `routing_failed` (event/action not in router table; logged + rejected)
  - `agent_invocation_failed` (agent returned non-zero; logged + alerted)

Re-deliveries (GitHub retries) are safe because the receiver
short-circuits at the dedup check.

### Crash-safe replay

The webhook-delivery-log is append-only. On receiver restart, the
log is replayed forward; any event whose dedup outcome is
`agent_invocation_failed` is retried with a fresh idempotency key
that includes the retry count (`<delivery_id>:retry:<n>`). The
retry budget is `MAX_RETRIES = 3` per delivery; exceeded retries
escalate to `oya-governance-webhook-stuck` lane.

### Signature handling (security)

- HMAC verification fails closed: any delivery without a valid
  HMAC is rejected before dedup check (so attackers can't poison
  the dedup table with crafted IDs).
- HMAC computation uses `sha256(webhook_secret + raw_payload)`
  matching GitHub's documented scheme.
- Webhook secret rotation: lifecycle managed via
  `oya-governance-secret-rotation-lifecycle` (crate authored at
  `crates/oya-governance-secret-rotation-lifecycle/`; lane scans
  OpenBao secret-version timestamps and fails the gate when any
  webhook secret exceeds 90-day rotation TTL). Operationally:
  `cargo run -p oya-dev-cli -- gate validate secret-rotation-lifecycle`
  exits 0 only when every webhook secret in the OpenBao path
  `sref://openbao/oya/foundry/webhook-secrets/*` has `rotated_at`
  within 90 days of `now`.
- Outbound `gh api` calls (post-back) use a separate
  PAT-or-app-token also stored in OpenBao at
  `sref://openbao/oya/foundry/github-app-token`. The token's
  scope is constrained to: write check_runs, post comments,
  modify refs (for promotion), create/modify PRs (for IP-005
  fix proposals).

### Bounded latency SLO

- p50 webhook → agent invocation: < 500 ms
- p99: < 5 s
- Stuck (no agent response): timeout = 60 s, then retry with
  fresh idempotency key.
- Per-changeset webhook fan-in cap: 1000 events / 24 h (prevents
  runaway loops).

## Consequences

### Positive

- Pipeline becomes truly event-driven; ~30 s GitHub Actions
  workflow_run latency replaced by ~500 ms webhook latency.
- Cross-event correlation via `changeset_id` (extracted from PR
  body or branch name on `pull_request.opened`); enables
  ADR-0110 state-machine transitions.
- Dedup makes the substrate replay-safe; agents can be restarted
  without re-firing on every old event.
- HMAC + sref-only secret access eliminates webhook-spoofing
  threat class.

### Negative

- New hosted endpoint to operate (Foundry control plane). Initial
  deployment piggy-backs on existing substrate (no new infra)
  but eventually wants dedicated service.
- Webhook secret rotation is currently manual; lifecycle lane is
  a successor-IP.
- Event-router table grows over time; the
  `oya-governance-event-router-completeness` lane (new,
  wave-C) asserts every GitHub event we care about has a router
  entry, else we silently miss events.

### Neutral

- HTTP server impl in Rust uses `axum` or `hyper-http-tower`
  (Foundry substrate decides). For receiver-side, both are LTS;
  pick at impl time.

## Implementation sequencing

- **Wave A** (this ADR Accepted):
  1. The webhook-receiver kernel — pure-domain HMAC
     verification + dedup table parser. No HTTP.
  2. The webhook-receiver app — HTTP receiver, routes
     to kernel, persists delivery log, dispatches to agents via
     in-process invocation OR queue (decide at impl).
  3. `registry/vcs/event-router.yaml` — canonical router table
     seeded with the 9 rows above.
  4. `registry/vcs/webhook-delivery-log.json` — empty append-only
     log to begin.
- **Wave B**:
  - Provision GitHub webhook on the oyatie repo pointing at the
    receiver endpoint.
  - Configure HMAC secret in OpenBao.
  - Retrofit IP-004/005/006 to be invoked via the router (instead
    of via workflow_run triggers).
- **Wave C**:
  - `oya-governance-event-router-completeness` lane.
  - `oya-governance-webhook-stuck` lane (alerts on
    `agent_invocation_failed` exceeding `MAX_RETRIES`).
  - `oya-governance-webhook-delivery-log-monotonic` lane
    (asserts no delivery_id appears twice with conflicting
    outcomes).

## Naming justification

- The webhook-receiver app (RETIRED per ADR-0363) followed the
  `-app` role per ADR-0056, with a `-kernel` companion.
- Router table file at `registry/vcs/event-router.yaml` — under
  the `vcs` substrate (matching the merge-queue + changeset
  registries).

## Open questions

1. Should event-router rows include an
   `agent_invocation_budget_usd` field for per-event cost caps?
   **Decision: defer to wave-B**; the `cost_budget_remaining`
   on the changeset (per ADR-0110) is the primary cost guard.
   Per-event caps are a fine-grained refinement.
2. Should webhook deliveries that route to multiple agents (fan-out)
   be supported? **Decision: NO in v1** — each event routes to
   exactly one agent. Multi-agent fan-out is composed by having
   that one agent invoke others.
3. How do we test the receiver without exposing a public endpoint
   to GitHub? **Decision: a `--simulate-delivery` flag on the
   app accepts a JSON payload + signature on stdin and exercises
   the full routing path locally.** This is also the canonical
   integration-test surface.
