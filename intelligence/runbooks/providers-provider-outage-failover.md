---
doc_class: Runbook
title: Provider outage — failover to next-best provider
microservice: foundry-providers
severity: "Sev-2 (single-provider degradation) / Sev-1 (multi-provider cascading)"
status: Accepted
owner_team: axis-foundry + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-FP-01 provider edge outage)
  - microservices/intelligence/incident-response.md
  - microservices/intelligence/PRD.md §"FR-02 router reads provider health"
  - microservices/observability/PRD.md
doc_status: published
---

# Runbook: Provider outage — failover to next-best provider

## Trigger

ONE of:

1. **Automated detection.** `provider-health-monitor` reports `oya_foundry_providers_provider_availability{vendor="<v>"} < 0.95` over a 60 s window OR `oya_foundry_providers_provider_p99_latency_ms{vendor="<v>"} > 5×baseline` over a 60 s window; provider-router auto-demotes `<v>` for the affected pack(s) and routes to the next-best provider.
2. **Vendor status page** — Anthropic/OpenAI/Google announces partial or full outage.
3. **Manual** — on-call IC declares provider outage based on tenant reports.

## Severity

- Single-vendor partial degradation, alternate vendor available, no tenant-visible errors: **Sev-3** (the system is working as designed; document but do not page).
- Single-vendor full outage, alternate vendor available, some queued requests retried successfully: **Sev-2** (degraded; tenant-visible latency spike).
- Multi-vendor cascading outage OR no compliant alternate in the tenant's pack: **Sev-1** (production impact; tenants seeing 5xx; declare immediately).

## Pre-checks

1. Confirm `provider-health-monitor` has demoted the affected vendor: `kubectl exec <provider-router-pod> -- curl -s localhost:9090/internal/router-state | jq '.demoted'` returns `["<vendor>"]`.
2. Confirm an alternate vendor is in the affected tenant's pack permitted-vendor matrix per `policy/data-residency.md`.
3. Confirm queued requests are being absorbed by the alternate.
4. Pull the latest `oya_foundry_providers_router_decided_total{vendor="<v>",reason=*}` for the last 5 min to confirm routing is shifting.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | If Sev-1 or Sev-2: open `#inc-<id>` Slack channel; assign IC; declare severity | ≤ 5 min |
| 2 | Confirm pre-checks above | ≤ 2 min |
| 3 | Verify alternate-vendor capacity headroom (per-tenant rate limit ≥ 2× current load) | ≤ 3 min |
| 4 | If alternate-vendor capacity insufficient: temporarily raise per-tenant rate limit via `cargo run -p oya-dev-cli -- providers raise-rate-limit --tenant <t> --vendor <alt-v> --multiplier 1.5 --duration 1h --reason "<id>"` (signed; audit-emitted) | ≤ 5 min |
| 5 | Verify per-tenant cost projection still under ceiling; if not, engage tenant operator + FinOps | ≤ 10 min |
| 6 | Tenant communication: status page update via CommsLead per `incident-response.md` template | ≤ 30 min |
| 7 | Monitor for vendor recovery: `oya_foundry_providers_provider_availability{vendor="<v>"}` returns ≥ 0.99 over 5 min | ongoing |
| 8 | When vendor recovers: provider-router auto-restores after a configurable cool-down (default 5 min); confirm by reading `router-state` JSON | ≤ 5 min after recovery |
| 9 | If primary vendor change is preferred long-term: open a tenant-config update PR to flip the capability-profile preference; merges through standard branch-protection | per priority |
| 10 | Postmortem within 5 business days if Sev-1 or Sev-2 | – |

## Rollback (of the failover itself)

If the alternate vendor causes a NEW regression (e.g., different response shape causes tenant tool-call failures):
1. Open `#inc-<id>-rollback`; declare Sev-1 if both primary + alternate degraded.
2. Per `runbooks/adapter-version-pin.md`: pin the tenant to a known-good adapter version OR redirect to the in-house adapter if capability fit allows.
3. If neither primary nor alternate is healthy: return `NoCompliantProvider` to tenants per `threat-model.md` T-09 mitigation (deterministic deny, no silent route).

## Verification

After completion:
- `oya_foundry_providers_provider_availability{vendor="<v>"}` ≥ 0.99 sustained 15 min.
- `provider-router-rest` p99 ≤ 5 ms (router decision; excludes upstream HTTP).
- Tenant-facing 5xx count returns to baseline (panel in `dashboards/provider-error-rate.json`).
- `ProviderInvoked` audit events flowing at expected rate to `foundry-evidence`.
- Postmortem published to `evidence/postmortems/<year>/<incident-id>.md`.

## Post-incident updates

- Postmortem published; action items tracked.
- If vendor SLA was violated: trigger vendor-credit claim through ops-finops.
- If routing heuristics performed poorly: refresh `provider-router-domain` weighting parameters; merge through standard PR review.
- If the residency matrix needed adjustment: update `policy/data-residency.md` via standard process.

## References

- `microservices/intelligence/failure-modes.md` FM-FP-01.
- `microservices/intelligence/incident-response.md`.
- `microservices/intelligence/PRD.md` §FR-02.
- `microservices/observability/PRD.md` (burn-rate evaluator).
