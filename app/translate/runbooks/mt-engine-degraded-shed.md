---
doc_class: Runbook
title: MT engine degraded — shed + failover
microservice: translate
severity: "Sev-2 (single engine degraded) / Sev-1 (all engines degraded)"
status: Accepted
owner_team: ops-sre-reliability + axis-translate
date: 2026-05-17
related_artifacts:
  - microservices/translate/failure-modes.md (FM-01..FM-08)
  - microservices/translate/decisions/ADR-TRANSLATE-0001-mt-engine-routing-and-fallback.md
  - microservices/translate/dashboards/translation-pipeline.json
doc_status: published
---

# Runbook: MT engine degraded — shed + failover

## Trigger

Any of:

- FM-01 (DeepL upstream 5xx surge).
- FM-02 (Anthropic 429 sustained).
- FM-03 (Google Cloud Translation EU-region outage).
- FM-04 (all external vendors unavailable simultaneously).
- FM-05 (in-house cold-start spike).
- FM-06 (response shape anomaly).
- FM-07 (router decision latency exceeds budget).
- FM-08 (per-tenant token-bucket cascade).

## Severity

- Single engine degraded with failover available: **Sev-2**.
- Multiple engines degraded; some failover paths remaining: **Sev-2**.
- ALL external vendors degraded + in-house available: **Sev-2** (degraded but serving).
- ALL engines degraded including in-house: **Sev-1** (translate µservice down).

## Single-Engine Demote (Anthropic / OpenAI / Google / DeepL / in-house)

| Step | Action | Time budget |
|---|---|---|
| 1 | Detect: `oya_translate_engine_error_rate{vendor=<v>} > 5 %` rolling 5 m OR `oya_translate_engine_availability_rolling_15m{vendor=<v>} < 99 %` | t = 0 |
| 2 | Auto-demote (router): `cargo run -p oya-dev-cli -- translate demote --vendor <v> --region <r> --duration 5m` (or wait for engine-health monitor auto-demote per ADR-TRANSLATE-0001) | ≤ 1 min |
| 3 | Confirm router selecting alternates: `oya_translate_engine_routed_total{vendor=<v>}` rate dropping | ≤ 2 min |
| 4 | Confirm tenant-facing latency stable: `oya_translate_translation_request_latency_ms` p95 not regressing | ≤ 5 min |
| 5 | If sustained > 15 m: emit `EngineDemoted` audit event; notify ops-finance (cost-mix may shift to higher-cost alternate) | ≤ 15 min |
| 6 | Vendor recovery: when `oya_translate_engine_availability_rolling_15m{vendor=<v>} > 99 %` for 15 min sustained: auto-recover (router includes again); or manual `cargo run -p oya-dev-cli -- translate recover --vendor <v>` | per recovery |

## Multi-Engine Degraded (FM-04: all external out)

| Step | Action | Time budget |
|---|---|---|
| 1 | IC declares Sev-2; opens `#inc-translate-engines` | ≤ 5 min |
| 2 | Verify in-house adapter alive: `oya_translate_engine_availability_rolling_15m{vendor="in-house"} > 99 %` | ≤ 2 min |
| 3 | Scale foundry-runtime in-house MT capacity: `cargo run -p oya-dev-cli -- foundry-runtime scale --capability translate-mt-v1 --replicas 16` | ≤ 5 min |
| 4 | If in-house insufficient for tenant rate: enable per-tenant rate-throttling per `runbooks/rate-limit-cascade-recovery.md` (sibling pattern from foundry-providers) | ≤ 15 min |
| 5 | Notify tenants via status page: "degraded MT capacity; in-house only" | ≤ 30 min |
| 6 | Postmortem within 5 business days if vendor-side root cause; coordinate vendor-watch | ≤ 5 d |

## Per-Tenant Cascade (FM-08)

When a single tenant exhausts vendor quota via token-bucket:

| Step | Action | Time budget |
|---|---|---|
| 1 | Detect: `oya_translate_token_bucket_denials_total{tenant=<t>}` > tenant alert threshold | t = 0 |
| 2 | Confirm tenant has not exceeded contracted rate (per cost-budget.md): if yes, notify tenant operator with usage data; if no, scale capacity | ≤ 15 min |
| 3 | If routine traffic: tenant educates self-on dashboard; no action | ≤ 30 min |
| 4 | If runaway / abuse: ops-security engaged; tenant rate-limited per DPA breach posture | ≤ 60 min |

## Response Shape Anomaly (FM-06)

When a vendor changes their response format silently:

| Step | Action | Time budget |
|---|---|---|
| 1 | Detect: `oya_translate_response_shape_anomaly_total{vendor=<v>} > 0` | t = 0 |
| 2 | Demote vendor immediately: `cargo run -p oya-dev-cli -- translate demote --vendor <v>` | ≤ 5 min |
| 3 | Identify adapter version observed-shape mismatch | ≤ 30 min |
| 4 | Pin previous adapter version: `cargo run -p oya-dev-cli -- translate pin-adapter --vendor <v> --version <prev>` | ≤ 15 min |
| 5 | Engineering refactor: update adapter response-validator + canonical-shape mapper | per fix |
| 6 | Test against vendor-mock + canary deploy | per cycle |
| 7 | Re-recover vendor when adapter green | per recovery |

## Verification

After recovery:

- `oya_translate_engine_availability_rolling_15m{vendor=<v>} > 99 %` for 30 min sustained.
- Tenant-facing translation success rate ≥ 99.5 %.
- No active alerts on per-vendor SLI.

## Post-Incident

- Postmortem within 5 business days.
- Sev-1: regulator-notifiable if data-class boundary crossed; per `incident-response.md`.
- Update per-vendor SLI threshold if pattern emerges.

## References

- ADR-TRANSLATE-0001 (engine routing).
- `microservices/translate/failure-modes.md`.
- `microservices/translate/dashboards/translation-pipeline.json`.
- Per-vendor status pages (Anthropic / OpenAI / Google Cloud / DeepL).
