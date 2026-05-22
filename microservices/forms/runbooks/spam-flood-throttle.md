---
doc_class: Runbook
title: Spam flood throttle (anonymous-submission DoS containment)
microservice: forms
severity: "Sev-2 (cross-tenant impact) / Sev-3 (per-form burst)"
status: Accepted
owner_team: ops-sre-reliability + axis-forms + ops-security
date: 2026-05-17
related_artifacts:
  - microservices/forms/threat-model.md §"T-D-01" + §"T-D-03" + §"T-D-07"
  - microservices/forms/capacity-model.md
  - microservices/forms/PRD.md §"Horizontal Scalability"
  - /specs/microservices/forms.json §goals.scalability
doc_status: published
---

# Runbook: Spam flood throttle

## Purpose

Forms exposes anonymous-submission endpoints to the public internet. Botnet attacks targeting a tenant's published form can flood the response-store, exhaust Postgres connection pool, defeat captcha (via solver bot farms), and degrade legitimate-tenant throughput. This runbook covers detection + containment + recovery.

## Trigger

ONE of:

1. **`oya_forms_submission_total{form_id=<h>}` rate > 100/sec** (10x normal anon-submit baseline) for ≥ 2 min.
2. **`oya_forms_captcha_solve_rate{form_id=<h>} > 95%` AND submission rate > 10/sec** (suggests solver bot OR captcha bypass).
3. **`oya_forms_postgres_connection_pool_saturation > 0.9` ≥ 5 min.**
4. **`oya_forms_submission_pii_field_set_rate{form_id=<h>}` < 5%** for ≥ 5 min during high submit rate (suggests submitter omitting required PII fields = bot).
5. **`oya_forms_submission_429_total` rate > 100/min cluster-wide.**
6. **`oya_forms_valkey_memory_used_bytes / oya_forms_valkey_memory_max_bytes > 0.85`.**
7. **`oya_forms_workflow_trigger_queue_depth > 50k` ≥ 10 min** (downstream queue overflow).

## Severity

- Single form, no impact on others: Sev-3.
- Cross-tenant impact (≥ 2 tenants seeing degradation): Sev-2.
- Cluster capacity exhausted: Sev-1.
- Coordinated synchronised attack across packs: Sev-1 + engage ops-security.

## Impact

- Anonymous submits return 429 (correct fair-share per `threat-model.md` T-D-01).
- Legitimate tenants on the same form may see queueing.
- Workflow-engine triggers backpressure.
- Webhook DLQ may grow.

## Pre-checks

1. Identify burst source: `dashboards/response-pipeline.json` panel "submits by form top-N" + "submits by IP/24 top-N".
2. Verify captcha provider: `kubectl -n forms get pod -l app=captcha-sidecar -o wide`.
3. Verify HPA state: `kubectl -n forms get hpa response-collector-rest`.
4. Verify Postgres + Valkey health.
5. Classify attack pattern: low-IP-cardinality (likely script-kiddie) vs high-IP-cardinality (likely botnet).

## Recovery Path A — Single-form burst (tenant ack)

Cause: tenant's form was shared on a popular site (legitimate viral burst).

| Step | Action |
|---|---|
| 1 | Contact tenant via gtm-customer-success to confirm. |
| 2 | Apply per-form elevated rate-limit: `cargo run -p oya-dev-cli -- forms rate-limit --form <id> --burst-multiplier 5x --duration 2h`. |
| 3 | Scale response-collector HPA: `kubectl -n forms scale deployment/response-collector-rest --replicas 30`. |
| 4 | Verify captcha sidecar capacity (HPA scales). |
| 5 | Monitor; revert rate-limit elevation when burst subsides. |

## Recovery Path B — Single-form burst (suspected abuse)

Cause: bot pattern matches; tenant's form open to attack.

| Step | Action | Time |
|---|---|---|
| 1 | Engage ops-security + gtm-customer-success. | ≤ 10 min |
| 2 | Throttle per-form: `cargo run -p oya-dev-cli -- forms rate-limit --form <id> --burst-multiplier 0.1x --duration 1h`. | ≤ 5 min |
| 3 | Activate WAF anti-bot ruleset: `cargo run -p oya-dev-cli -- waf activate-ruleset --ms forms --ruleset anti-bot-v2`. | ≤ 5 min |
| 4 | Force fallback captcha (hCaptcha → Friendly Captcha challenge mode). | ≤ 5 min |
| 5 | Tenant notified; legitimate submitters see "complete a quick puzzle" UX. | – |
| 6 | If confirmed abuse: per ToS, pause form publish; engage tenant. | per priority |

## Recovery Path C — Botnet-style high-IP-cardinality attack (Sev-1)

Cause: thousands of unique IPs, low per-IP rate, but high cluster rate.

| Step | Action |
|---|---|
| 1 | Declare Sev-1; engage ops-security + ops-sre-reliability + axis-forms. |
| 2 | Activate WAF DDoS ruleset: `cargo run -p oya-dev-cli -- waf activate-ruleset --ms forms --ruleset emergency-ddos-v1`. |
| 3 | Enable captcha-hard mode (challenge every submit, no risk-based skip): `cargo run -p oya-dev-cli -- forms captcha-mode --hard --duration 6h`. |
| 4 | Enable per-ASN rate-limit at WAF: `cargo run -p oya-dev-cli -- waf rate-limit-asn --ms forms --asn auto --rps 5`. |
| 5 | Per-pack tenant comms if any tenant impacted. |
| 6 | If suspect of state-sponsored attack: engage council-legal-compliance per pack regulatory. |

## Recovery Path D — Captcha solver-bot ring (Sev-2)

Cause: captcha solve rate > 95% suggests human-solver-farm OR successful bypass.

| Step | Action |
|---|---|
| 1 | Engage ops-security; capture sample tokens. |
| 2 | Force multi-provider captcha (challenge with hCaptcha + Turnstile both): per ADR-FORMS-0002. |
| 3 | Add per-submitter-hash velocity check; > 3 submits / hour = challenge mode. |
| 4 | Escalate to captcha provider (hCaptcha / Cloudflare): submit attack pattern. |
| 5 | If solver-farm confirmed: engage council-legal-compliance for cease-and-desist. |

## Recovery Path E — Workflow-trigger queue overflow

Cause: form-on-submit triggers a workflow; workflow-engine backpressured.

| Step | Action |
|---|---|
| 1 | Verify engine spec-store: `microservices/workflow-engine/dashboards/spec-store-health.json`. |
| 2 | If engine slow: engage workflow-engine on-call per `microservices/workflow-engine/runbooks/spec-store-perf.md`. |
| 3 | Forms-side: increase trigger-buffer + retry policy (already exp-backoff); submitter UX unchanged (response accepted; trigger queued). |
| 4 | If queue exceeds cap: per-tenant trigger throttle activated. |

## Recovery Path F — Postgres connection-pool saturated

| Step | Action |
|---|---|
| 1 | Verify Postgres health: `kubectl -n forms exec postgres-primary -- psql -c "SELECT count(*) FROM pg_stat_activity"`. |
| 2 | Scale Postgres connection pooler (pgbouncer): `kubectl -n forms edit configmap pgbouncer-config`. |
| 3 | If shard skew: re-balance Citus per ADR-0164. |
| 4 | If sustained: scale Citus workers (add nodes). |

## Verification

After recovery:
- `oya_forms_submission_total` rate returns to baseline.
- Captcha solve rate < 70% (humans).
- Postgres connection pool < 70% saturation.
- Valkey memory < 80%.
- Workflow-trigger queue depth < 1k.
- Submission p95 ≤ 150ms.

## Post-incident updates

- Postmortem within 5 business days (immediate for Sev-1).
- If legitimate viral burst: update `capacity-model.md` traffic pattern.
- If attack: WAF rule permanent; per-form captcha-hard mode evaluated for default-on.
- If captcha bypass: provider escalation; review ADR-FORMS-0002.
- Per-pack tenant comms.

## References

- `microservices/forms/PRD.md` §"Horizontal Scalability".
- `microservices/forms/threat-model.md` T-D-01, T-D-03, T-D-07.
- `microservices/forms/capacity-model.md`.
- ADR-FORMS-0002 captcha + anti-spam.
- Google SRE Workbook ch. 21 (Cascading failures).
- Cloudflare DDoS protection patterns — `developers.cloudflare.com/ddos-protection/`.
- hCaptcha incident response playbook — `docs.hcaptcha.com/`.
