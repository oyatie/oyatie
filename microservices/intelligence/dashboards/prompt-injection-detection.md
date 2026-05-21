---
doc_class: DashboardSpec
title: Prompt-Injection Detection
microservice: intelligence
status: Accepted
owner_team: ops-security + axis-intelligence + council-privacy
related_artifacts:
  - microservices/intelligence/runbooks/prompt-injection-detected.md
  - microservices/intelligence/threat-model.md (T-T-01, T-T-02)
doc_status: published
---

# Prompt-Injection Detection Dashboard

## Purpose

Per-call prompt-injection detection rate; trended over time + broken down by tenant / audience /
modality / pack / injection-kind (direct vs indirect).

## Panels

### Injection-detection rate (per 1k dispatches)

```promql
1000 * sum(rate(oya_intelligence_prompt_injection_detected_total[1h]))
       / sum(rate(oya_intelligence_dispatch_total{status=~"completed|refused"}[1h]))
```

### Injection kind breakdown

```promql
sum(rate(oya_intelligence_prompt_injection_detected_total[1h])) by (injection_kind)
```

### Top tenants by detection count

```promql
topk(20, sum(rate(oya_intelligence_prompt_injection_detected_total[24h])) by (tenant_id))
```

### Classifier confidence distribution

Histogram bucket of post-call classifier score for detected injections.

```promql
sum(rate(oya_intelligence_prompt_injection_classifier_score_bucket[1h])) by (le)
```

## Alerts

- Burn alert: rate > 10x baseline → page axis-intelligence.
- Spike alert: rate > 100x baseline for 5 min → Sev-2 paging cascade.
- Tenant-cohort spike: single-tenant rate > 1000x its 30d baseline → tenant abuse signal.

## Forensic workflow

For every detected event:
1. Audit-tap record emitted with `PromptInjectionDetected` schema (see asyncapi-intelligence-events-v1.yaml).
2. Forensic export available via:
   ```bash
   cargo run -p oya-dev-cli -- intelligence audit-tap-export \
     --window 24h --filter prompt-injection-detected
   ```
3. Engage runbook `runbooks/prompt-injection-detected.md`.

## References

- `microservices/intelligence/runbooks/prompt-injection-detected.md`.
- `microservices/intelligence/threat-model.md` T-T-01, T-T-02.
- OWASP LLM Top 10 (2025) LLM01.
