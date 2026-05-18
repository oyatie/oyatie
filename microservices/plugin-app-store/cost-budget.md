---
doc_class: CostBudget
title: "Cost budget"
microservice: plugin-app-store
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Cost budget


## Per-installation cost target

- Per-installation steady-state: $0.05 / month (compute + storage attribution).
- Per-plugin runtime (free plugins): $0.20 / month per 100k requests.
- Per-plugin runtime (paid plugins): cost passed through to subscription tier.

## Per-developer cost target

- Per-developer steady-state: $0.50 / month (sandbox + portal share).
- Per-payout: $0.10 settlement cost (amortized across batch).
- Per-tax-form: $0.05 / year.

## Vetting pipeline cost

- Per-submission: $0.50 (Trivy + Cosign + Wasmtime ephemeral run).
- Per-year at 10k submissions: $5k.

## Budget breach response

| Threshold | Action |
|---|---|
| 50% of monthly | Slack alert |
| 75% | Email council |
| 100% | Throttle non-paid tier |
| 150% | Emergency review |

