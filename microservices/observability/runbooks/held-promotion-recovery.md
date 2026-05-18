---
doc_class: Runbook
title: Held-promotion recovery
microservice: observability
severity: "Sev-3 (operational delay) / Sev-2 (if persistent)"
status: Accepted
owner_team: axis-observability
date: 2026-05-17
related_artifacts:
  - microservices/observability/failure-modes.md (FM-03 worker, FM-11 ruler, FM-14 lane flaky)
  - microservices/observability/incident-response.md
  - microservices/observability/policy/tenant-isolation.md
doc_status: published
---

# Runbook: Held-promotion recovery

## Trigger

A µservice's promotion is held (`verdict=held` or `rejected`) when the µservice owner expected `eligible`. Manifests as a CI lane failure on `oya-vcs-promotion-readiness` or a tenant-reported "my deploy isn't going through."

## Severity

- Single-µservice hold of ≤ 1h with clear cause: Sev-3.
- Multi-µservice hold OR persistent > 1h OR cause unclear: Sev-2 (gate is fail-closed and operationally blocking real promotions).

## Pre-checks

1. Confirm the held SHA: `cargo run -p oya-dev-cli -- vcs status --microservice <ms> --sha <sha>`.
2. Verify the µservice has an OpenSLO manifest: `ls microservices/<ms>/slos/*.openslo.yaml`. If empty, the gate's fail-closed default applies (verdict=`rejected`); see Recovery Path A.
3. Verify Mimir reachability from CI runner: `curl -s https://mimir-<pack>.oyatie.dev/ready` returns 200.
4. Verify Mimir tenant `oya-ci` API key has not been rotated outside the CI runner's expectation.

## Recovery Path A — OpenSLO manifest missing

Cause: the µservice has no `slos/*.openslo.yaml` files; gate fail-closed default = `rejected`.

| Step | Action |
|---|---|
| 1 | Author OpenSLO manifest(s) for the µservice per `docs/standards/observability-slo.md` (Slice D); one SLI per the canonical SLI catalog (availability + latency + correctness + freshness as minimum). |
| 2 | PR review + merge. |
| 3 | Worker hot-reloads the manifest; verdict transitions to `held` then (after burn-rate window) `eligible`. |
| 4 | Re-run CI lane. |

## Recovery Path B — Burn-rate breached (real)

Cause: a real burn-rate alert fires; the gate correctly holds.

| Step | Action |
|---|---|
| 1 | Investigate the breach: dashboard at `https://grafana-<pack>.oyatie.dev/d/microservice/<ms>` shows fast-burn / slow-burn rates. |
| 2 | Fix the underlying regression (typically in the µservice's adapter, kernel, or downstream-dep). |
| 3 | New SHA with the fix lands; gate auto-evaluates; transitions to `eligible` after clean window. |
| 4 | Promotion resumes. |

## Recovery Path C — Worker outage (FM-03)

Cause: `slo-engine-worker` is down; verdicts not being emitted; everything appears `held`.

| Step | Action |
|---|---|
| 1 | Verify per `runbooks/evaluator-down.md`; engage axis-observability on-call. |
| 2 | While worker is recovering, all µservices are held (correct fail-closed behavior). |
| 3 | If urgent business need, invoke manual override (see Path E). |

## Recovery Path D — Mimir read failure from CI (FM-14)

Cause: lane is flaky due to transient Mimir read failure; verdict is actually `eligible` but lane reports a read error.

| Step | Action |
|---|---|
| 1 | Retry the CI lane (idempotent). |
| 2 | If persistent: check Mimir self-SLI (`https://grafana-<pack>.oyatie.dev/d/mimir-self/overview`). |
| 3 | If Mimir read path degraded: declare Sev-2 incident; engage ops-sre-reliability; activate `runbooks/mimir-outage.md`. |
| 4 | If only the CI tenant `oya-ci` is failing: rotate the `oya-ci` Mimir API key via OpenBao; CI lane uses the new key on next run. |

## Recovery Path E — Manual gate override (2-person rule + audit)

Use ONLY when business need is urgent AND the cause is operational (not a real burn-rate breach). NEVER bypass for a real SLO violation.

| Step | Action |
|---|---|
| 1 | IC opens override request: `cargo run -p oya-dev-cli -- vcs override-eligibility --microservice <ms> --sha <sha> --env <env> --reason "<rfc-with-jira-ticket>"`. |
| 2 | CLI requires 2-person rule: a second on-call engineer confirms via second-channel signature (ops-security has signing key access via OpenBao JIT). |
| 3 | Override emits an `oya_promotion_manual_override_total{microservice, sha, env}` metric + audit-chain seal. |
| 4 | Override expires automatically after 1 promotion event; cannot be reused. |
| 5 | Postmortem within 5 business days — why was override needed? What process gap allowed it? |

## Verification

After completion:
- `cargo run -p oya-dev-cli -- vcs status --microservice <ms> --sha <sha>` shows `eligible` (or for Path E: `eligible (manual-override)`).
- Tenant promotion proceeds on next gate-tick.
- Audit-chain seal log records the recovery action.

## Post-incident updates

- Document any new failure pattern in `failure-modes.md`.
- If Path E was used: postmortem assigns action item to harden the gate against the underlying flake.
- If Path A was used: surface the manifest-authoring delay to engineering manager.

## References

- `microservices/observability/failure-modes.md`.
- `microservices/observability/incident-response.md`.
- `/specs/agentic-slo-gated-promotion.json`.
- `docs/standards/observability-slo.md` (Slice D).
