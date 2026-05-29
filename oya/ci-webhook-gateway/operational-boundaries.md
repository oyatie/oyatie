# Operational boundaries, capacity & cost/finops — CI Webhook Gateway

## Operational boundaries

- **In scope**: receive Forgejo webhooks, verify HMAC, parse PR events, kick the
  Jenkins `oyaCiLane` pipeline, emit audit-chain rows.
- **Out of scope (owned elsewhere)**:
  - Running governance gates + posting Forgejo commit statuses → Jenkins
    (`oyaCiLane.groovy`), the trusted runner (ADR-0367).
  - Adversarial code review → Intelligence service reviewer gate (ADR-0367 D2)
    — NOT wired yet (placeholder-debt `adr-0374-reviewer-gate-dispatch`).
  - Merge-queue ordering / speculative rebase → ADR-0111 — parked (ADR-0363 §3),
    tracked `adr-0374-merge-queue-admit`.
  - Branch-protection config + the actual merge → Forgejo (auto-merge on green).

## Incident response

See `runbooks/on-call.md`. Key rule: if the gateway is down, PRs against `dev`
stop being auto-gated — **fix the gateway; do NOT revert to admin-relax-merge**
(the seam this service exists to retire).

## Capacity model

- Stateless; horizontally scalable behind a Service. One replica suffices for
  the dogfood farm (PR volume is low).
- Per-delivery work is O(body): one HMAC, one JSON parse, one bounded TCP POST.
- Memory: ~tens of MiB (Tokio + Axum). CPU: negligible except the HMAC.
- Pod runtime tier: substrate-critical (a CI-gating outage blocks merges) but
  blast radius is bounded (no tenant data). Recommend Tier 2.

## Cost / finops

- One small pod (request ~50m CPU / 64Mi memory). Negligible compute cost.
- No storage (stateless). No egress beyond the in-cluster Jenkins kick.
- The expensive part of the pipeline is the Jenkins CI run, NOT this gateway;
  cost governance lives with the CI farm (ADR-0349/0360) + the per-changeset
  cost budgets (ADR-0113 carried-forward), not here.
- FinOps dimension (ADR-0344): attributed to the `oya-substrate` cost center,
  not to any tenant.

## Tenant isolation

The gateway is a **substrate** service, not tenant-scoped. It processes
repo-coordination metadata only (PR numbers, commit SHAs, branch names) — no
tenant PII, no tenant data path. Cedar `forbid`s any `tenant.data.read`
(`cedar/policies.cedar`). It runs in the `oya-ci` namespace, isolated from
tenant workloads.
