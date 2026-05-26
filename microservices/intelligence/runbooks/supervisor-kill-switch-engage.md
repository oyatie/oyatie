---
doc_class: Runbook
title: Kill-switch engage (manual / automated)
microservice: foundry-supervisor
severity: "Sev-1 (always when initiated by autonomy violation or fleet-wide; Sev-2 for planned tenant-scope engage)"
status: Accepted
owner_team: ops-security + axis-foundry-control-plane
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-01)
  - microservices/intelligence/incident-response.md
  - microservices/intelligence/policy/tenant-scope.cedar
doc_status: published
---

# Runbook: Kill-switch engage

## Trigger

ONE of:

1. **Automated** — supervisor detects autonomy-violation, guardrail-violation, fast-burn breach, or eval-regression event and auto-invokes engage on the scoped target.
2. **Tenant-initiated** — tenant DPO engages own-scope kill-switch via REST + Cedar.
3. **Manual fleet-wide** — ops-security on-call declares Sev-1 and engages fleet-wide kill-switch via 2-person rule.

## Severity

- Automated (autonomy/guardrail violation): **Sev-1**.
- Tenant-initiated scope-limited: Sev-2 if planned; Sev-1 if reactive.
- Manual fleet-wide: **Sev-1 always**.

## Pre-checks (manual fleet-wide only)

1. Confirm authority: ops-security on-call lead + one of {ops-security director, council-architecture chair, council-privacy chair}.
2. Confirm reason category from enum: `cross_tenant_leak | runaway_cost | safety_breach | breach_response | compliance_demand`.
3. Confirm scope: `tenant | capability | agent | fleet`.
4. Confirm cancel-window understanding: 5-second post-engage cancel is available.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>` Slack channel; assign IC; declare severity | ≤ 5 min |
| 2 | Confirm pre-checks (if manual fleet-wide); both signatures captured | ≤ 2 min |
| 3 | Invoke engage: `cargo run -p oya-dev-cli -- supervisor engage-kill-switch --scope <scope> --target <id> --reason "<enum>" --signature-bundle <openbao-jit-token>`. CLI: (a) verifies both signatures (fleet-wide); (b) writes `KillSwitch` CRD via Operator; (c) writes Valkey state (cache + propagation); (d) emits `KillSwitchEngaged` event Ed25519-signed; (e) audit-chain seal | ≤ 1 s engage p99 |
| 4 | Verify foundry-runtime workers refusing new invocations within p99 ≤ 1 s (CRD watch fan-out + Valkey pub-sub redundant channels) | ≤ 1 s p99 verified end-to-end |
| 5 | Verify `oya_supervisor_kill_switch_engaged{scope=<>, target=<>, reason=<>} == 1` in Mimir | ≤ 30 s |
| 6 | OnCall page received in two-channel corroboration | ≤ 60 s |
| 7 | CommsLead: status-page update for fleet-wide; tenant comms per `incident-response.md` template if tenant-scope | ≤ 30 min |
| 8 | If automated: file an Issue for the trigger event analysis | per priority |
| 9 | Postmortem within 5 business days (Sev-1) | – |

## Disengage

| Step | Action |
|---|---|
| 1 | Confirm cause cleared (autonomy violation root-caused + fixed; cost runaway mitigated; etc.) |
| 2 | Invoke disengage: `cargo run -p oya-dev-cli -- supervisor disengage-kill-switch --scope <scope> --target <id> --reason "cause-cleared" --signature-bundle <openbao-jit-token>`. For fleet-wide, 2-person rule applies. |
| 3 | Verify Valkey state cleared + CRD updated + workers resume accepting invocations |
| 4 | Audit-chain emits `KillSwitchDisengaged` |

## Verification

After completion:
- `oya_supervisor_kill_switch_engaged{scope=<>, target=<>}` reflects current state.
- `KillSwitchEngaged` / `KillSwitchDisengaged` event sealed in audit-chain.
- For Sev-1: per-changeset evidence at `microservices/intelligence/evidence/multispectrum/<change_id>-<unix_ts>.json`.
- Grafana OnCall incident closed.
- Status page reflects "Resolved" with timestamp.

## References

- ADR-0139 §"Automated rollback primitive" (precedent).
- ADR-0140 (retired per ADR-0145) (Cedar policy).
- `failure-modes.md` FM-01.
- `incident-response.md` §"Sev-1 response".
- `/specs/foundry-supervisor-control-plane.json` §"kill_switch".
