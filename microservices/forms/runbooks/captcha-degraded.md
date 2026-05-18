---
doc_class: Runbook
title: Captcha provider degraded (multi-provider fallback)
microservice: forms
severity: "Sev-2"
status: Accepted
owner_team: ops-sre-reliability + axis-forms + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/forms/threat-model.md §"T-D-03" + §"T-D-07" + §"T-S-05"
  - microservices/forms/runbooks/spam-flood-throttle.md
  - microservices/forms/decisions/ADR-FORMS-0002-captcha-and-anti-spam.md
doc_status: published
---

# Runbook: Captcha degraded (fail-closed)

## Purpose

When the primary captcha provider degrades, Forms MUST NOT fail-open (accept submits without verification — per `threat-model.md` T-D-07 invariant). This runbook executes the multi-provider fallback per ADR-FORMS-0002.

## Trigger

ONE of:

1. **`oya_forms_captcha_provider_up{provider="hcaptcha|turnstile|friendly"} == 0` ≥ 60s.**
2. **`oya_forms_captcha_verify_latency_seconds{provider=...,quantile="0.99"} > 3.0` ≥ 5 min.**
3. **`oya_forms_captcha_verify_error_total{provider=...}` rate > 5/min.**
4. **External signal**: provider status page reports incident.
5. **`oya_forms_submission_429_total` rate spike correlated with `oya_forms_captcha_verify_total` drop** — submitters falling back to manual queue.

## Severity

- Single provider degraded, fallback healthy: Sev-2.
- All configured providers degraded: Sev-1 + activate manual-review queue.

## Impact

- New anonymous submits blocked or slowed.
- Tenants on pack-eu / pack-kr / pack-us-hc cannot use reCAPTCHA fallback (forbidden per ADR-FORMS-0002).
- Manual-review queue grows; tenant alerted.

## Pre-checks

1. Identify failing provider: `dashboards/response-pipeline.json` panel "captcha provider health".
2. Check provider status page (linked from dashboard).
3. Determine pack(s) affected (provider routing per `policy/data-residency.md`).
4. Verify fallback provider health.

## Recovery Path A — Single-provider degraded (within-pack fallback healthy)

| Step | Action | Time |
|---|---|---|
| 1 | Switch pack to fallback: `cargo run -p oya-dev-cli -- forms captcha-provider --pack <pack> --provider <fallback>`. | ≤ 5 min |
| 2 | Verify submitter UX: render test-form; complete fallback challenge. | ≤ 5 min |
| 3 | Per-tenant comms: notify tenants in pack of provider switch (no action required). | per priority |
| 4 | Monitor: `oya_forms_captcha_verify_latency_seconds{provider=<fallback>}` healthy; submit rate normalises. | – |
| 5 | When primary recovers (provider status page + internal probe green for 30 min): revert. | per cadence |

## Recovery Path B — All configured providers degraded (Sev-1)

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-security + ops-sre-reliability + axis-forms. |
| 2 | Activate manual-review queue: `cargo run -p oya-dev-cli -- forms captcha-mode --manual-review --duration 4h`. |
| 3 | Submitter UX: "your submission is queued for review; tenant will follow up". |
| 4 | Tenant comms: per-tenant notification; tenant decides whether to disable form during outage. |
| 5 | Manual-review queue staffed by gtm-customer-success + tenant operators for affected tenants. |
| 6 | DO NOT fail-open: submissions without captcha verification stored as `pending_captcha=true`; not exposed to tenant analytics until verified. |

## Recovery Path C — Suspected bypass (captcha solve rate suspicious — see also spam-flood-throttle.md Recovery D)

Folded into spam-flood-throttle.md Path D. Cross-reference.

## Invariant: Fail-closed

Per `threat-model.md` T-D-07: captcha sidecar crash → submit returns 503; never accepted without verification. CI lane `oya-forms-captcha-fail-closed-conformance` asserts this.

## Pack-specific constraints

- **pack-eu**: reCAPTCHA forbidden (Schrems II). Fallback chain: hCaptcha → Friendly Captcha → manual review.
- **pack-kr**: reCAPTCHA forbidden (PIPA Art. 23-2). Same fallback chain.
- **pack-us-healthcare**: reCAPTCHA forbidden (BAA risk). Fallback chain: hCaptcha → Friendly Captcha → manual review.
- **pack-us / pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa**: Turnstile + hCaptcha chain.

## Verification

After recovery:
- `oya_forms_captcha_provider_up` = 1 for active provider.
- `oya_forms_captcha_verify_latency_seconds{quantile="0.99"} < 1.0`.
- `oya_forms_captcha_verify_error_total` rate < 1/min.
- Manual-review queue drained (if Path B was activated).

## Post-incident updates

- Postmortem within 5 business days.
- Provider SLA review per `legal/sub-processors.md`.
- If recurring: consider new provider per ADR-FORMS-0002 supersession.

## References

- ADR-FORMS-0002 captcha + anti-spam.
- `policy/data-residency.md` §"Pack determines captcha provider".
- `threat-model.md` T-D-03, T-D-07, T-S-05.
- hCaptcha + Cloudflare Turnstile + Friendly Captcha status pages.
- Schrems II ruling (CJEU C-311/18).
