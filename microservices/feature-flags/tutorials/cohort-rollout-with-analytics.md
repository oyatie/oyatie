---
doc_class: Tutorial
microservice: feature-flags
persona: engineer + analyst
language: rust
date: 2026-05-20
doc_status: published
---

# Cohort rollout end-to-end with `analytics`

You will: define a cohort over the `analytics` materialized view, target a flag at the cohort, roll the flag, and observe the outcome split in a Materialise notebook. Total time ≤ 45 min.

## Pre-requisites

The `feature-flags/tutorials/getting-started-rust-sdk.md` tutorial is complete. The `analytics` µservice is running locally (`make analytics-up`).

## Step 1 — Materialise the cohort

In `microservices/analytics/audiences/`, add `tutorial-cohort-power-users.yaml`:

```yaml
audience_id: tutorial-cohort-power-users
description: Tenants whose 7-day API call volume is in the top decile.
sql: |
  SELECT tenant_id
  FROM analytics.api_call_aggregate_7d
  WHERE p90_calls_per_day > 50000
window: 7d
refresh_interval: 5m
```

Publish: `cargo run -p oya-dev-cli -- analytics audience publish tutorial-cohort-power-users`. Confirm it materialised: `oya analytics audience inspect tutorial-cohort-power-users` shows non-zero membership.

## Step 2 — Reference the cohort from feature-flags

Edit `microservices/feature-flags/catalog/flags/tutorial.power-user-dashboard.yaml`:

```yaml
flag_key: tutorial.power-user-dashboard
intent: Surface the dashboard v2 to power users first; full rollout once latency budget holds.
lifecycle: release_toggle
variants:
  on: true
  off: false
default_variant: off
sunset_at: 2026-09-30
audit_required: false
targeting:
  - rule_id: r-power-user
    audience_ref: tutorial-cohort-power-users
    variant: on
```

The audience-ref form makes the audience-membership lookup a side-band call to `analytics` (cached 5 min). The cache TTL must be ≤ the audience refresh-interval; the publish lane enforces this.

## Step 3 — Evaluate

Same SDK call as Tutorial 1:

```rust
let v2 = ff.eval_bool("tutorial.power-user-dashboard", false).await?;
```

The SDK passes the request's `tenant_id`; the side-car evaluator checks audience membership via the cached `analytics` lookup.

## Step 4 — Observe the outcome split

The Materialise notebook `microservices/analytics/notebooks/tutorial-cohort-outcome.ipynb` joins:

- `audit_chain.flag_evaluation{flag_key="tutorial.power-user-dashboard"}` (the variant assignment).
- `analytics.product_outcome{event="dashboard_engagement"}` (the downstream effect we care about).

On `(tenant_id, time_bucket_1h)`. Run the notebook; the cell labelled "engagement-lift-by-variant" outputs a per-hour delta between `variant=on` and `variant=off`. With one day of cohort traffic the Wilson interval at α=0.05 settles to ± 4.2 % (which is too wide to call a winner — the tutorial is showing you the surface, not the statistics).

## Step 5 — Quick-pause if the metric regresses

If the `variant=on` engagement falls under `variant=off` by > 5 % in any hourly window, the lane `oya-governance-flag-experiment-guardrail` opens a P1 incident automatically. The on-call has the option to:

- `oya ff pause --flag tutorial.power-user-dashboard --reason guardrail-tripped` — temporarily route 100 % to default; the audit-chain `flag_paused` event captures the reason.
- `oya ff investigate --flag tutorial.power-user-dashboard` — opens a debug session with the per-tenant evaluation log enabled (capped at 1000 tenants for cost).

For the tutorial, manually pause and resume to feel the mechanics.

## Step 6 — Promote to 100 %

Edit the flag to delete the `audience_ref` rule and change `default_variant: on`:

```yaml
flag_key: tutorial.power-user-dashboard
default_variant: on
targeting: []
```

Publish. Within 60 s every tenant evaluates to `on`. Schedule the retire ≤ 90 days out (the `release_toggle` lifecycle).

## What you learned

- The audience-ref evaluation path: feature-flags asks analytics for membership, with a TTL-bound cache.
- The guardrail lane that auto-pauses on engagement regression.
- The promote-to-default-and-retire sunset pattern.

Reference reading: ADR-0316 §"Capability Tier — paid" §"Cohort eligibility freshness ≤ 5 min p99"; ADR-0145 §"Three-tier evaluator latency budget".
