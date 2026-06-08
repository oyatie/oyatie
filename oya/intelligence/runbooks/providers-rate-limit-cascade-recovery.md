---
doc_class: Runbook
title: Rate-limit cascade recovery
microservice: foundry-providers
severity: "Sev-2 (single-tenant throttle) / Sev-1 (multi-tenant cascade)"
status: Accepted
owner_team: axis-foundry + ops-sre-reliability + ops-finops
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-FP-02 rate-limit cascade)
  - microservices/intelligence/threat-model.md (T-02 rate-limit abuse)
  - microservices/intelligence/PRD.md §"FR-02"
doc_status: published
---

# Runbook: Rate-limit cascade recovery

## Trigger

ONE of:

1. **Automated** — `oya_foundry_providers_provider_429_total{tenant="<t>",vendor="<v>"}` rate exceeds the burn-rate alert threshold (default: 1 % of requests over 5 min).
2. **Manual** — tenant operator reports throttling; on-call investigates.
3. **Cost-ceiling breach** — `oya_foundry_providers_provider_cost_usd_total{tenant="<t>"}` exceeds the per-tenant rolling ceiling; the router has stopped invocations for that tenant; tenant operator reports stopped workload.

## Severity

- Single-tenant brief throttle, recovers within 60 s: **Sev-3** (no page; document).
- Sustained throttling on one tenant > 5 min: **Sev-2**.
- Multi-tenant cascade (multiple tenants throttled because vendor rate limit shared): **Sev-1**.

## Pre-checks

1. Identify which tenants are throttled: `oya_foundry_providers_provider_429_total{vendor="<v>"}` grouped by tenant.
2. Confirm whether the throttle is at the in-process token bucket (Valkey) or upstream (vendor returns 429).
3. Pull current rate vs ceiling: `oya_foundry_providers_provider_invocations_total[1m]` vs the configured per-tenant rate limit.
4. Check cost-ceiling status: `oya_foundry_providers_provider_cost_usd_total{tenant="<t>"}` vs ceiling.

## Steps (Single-tenant throttling)

| Step | Action | Time budget |
|---|---|---|
| 1 | Confirm cause: in-process bucket vs upstream-vendor throttle | ≤ 2 min |
| 2 | If in-process: increase per-tenant rate limit IF (a) cost ceiling allows, (b) vendor capacity allows. Use `cargo run -p oya-dev-cli -- providers set-rate-limit --tenant <t> --vendor <v> --limit <new> --duration 1h` (signed; audit-emitted) | ≤ 5 min |
| 3 | If upstream-vendor throttle: router will auto-route to next-best vendor for the tenant; verify failover working via `runbooks/provider-outage-failover.md` | ≤ 5 min |
| 4 | If cost ceiling breached: engage tenant operator to either raise ceiling or reduce workload | per tenant SLA |
| 5 | Monitor for return to baseline | ≤ 10 min |

## Steps (Multi-tenant cascade)

| Step | Action | Time budget |
|---|---|---|
| 1 | Declare Sev-1; open `#inc-<id>`; IC + OpsLead + axis-foundry SME | ≤ 5 min |
| 2 | Determine root cause: shared upstream vendor rate limit OR runaway tenant exhausting per-pack-pool | ≤ 10 min |
| 3 | If runaway tenant identified: enforce emergency rate cap on that tenant via CLI; engage tenant operator | ≤ 5 min |
| 4 | If shared upstream limit: contact vendor to request emergency quota increase (typically not real-time) | ≤ 30 min |
| 5 | In meanwhile: shift affected tenants to alternate vendors per `policy/data-residency.md` matrix | per matrix |
| 6 | Tenant comms via CommsLead | ≤ 30 min |
| 7 | Monitor for recovery: 429 rate returns to baseline | ≤ 1 h |
| 8 | Postmortem within 5 business days | – |

## Token-bucket Tuning Guidance

| Vendor | Default per-tenant per-min limit | Max per-tenant per-min cap | Notes |
|---|---|---|---|
| Anthropic API | 60 req/min, 60K tok/min | 600 req/min, 600K tok/min (tier-3 vendor account) | Per vendor docs; conservative default |
| OpenAI API | 60 req/min, 90K tok/min | 5000 req/min (tier-5) | Per vendor docs |
| Gemini API | 60 req/min, 32K tok/min | tier-dependent | Per vendor docs |
| in-house | per-pod-served-capacity | per pack capacity | self-served; no upstream throttle |

Token-bucket implementation lives in `oya-intelligence-providers-router-adapter` (Redis-backed). Per ADR-0117, per-pack Valkey Sentinel HA ensures bucket state survives a single-node failure.

## Cost Ceiling Tuning

Default per-tenant per-vendor per-day cost ceiling: 5× rolling-30d median. Operators may adjust via tenant-config update PR.

## Verification

- `oya_foundry_providers_provider_429_total[5m]` returns to baseline.
- Tenant workload resumes at expected throughput.
- `evidence/runbook-drills/rate-limit/<unix_ts>.json` recorded for the drill.

## References

- `microservices/intelligence/failure-modes.md` FM-FP-02.
- `microservices/intelligence/threat-model.md` T-02.
- Vendor-published rate-limit docs (Anthropic / OpenAI / Google).
