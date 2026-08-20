# Cloud Intelligence service — Cost / FinOps

**Authority:** ADR-0373 (per-tenant budgets), ADR-0373 (metering emission)
**Research grounding:** `design/hyperscaler-best-practice-brief.md` §8 (cost/FinOps — meter from provider-returned tokens + budget enforcement + attribution dims), §6 (per-tenant budgets).
**Last reviewed:** 2026-05-26

## Why FinOps is a first-class gateway concern

Frontier-model spend is the dominant variable cost of an AI fleet, and an unbounded gateway is a
denial-of-wallet vector (OWASP LLM10). The brief (§8) found the mature pattern: **meter from
provider-returned tokens, enforce per-tenant budgets across concurrent windows, and attribute with
rich dims** (Kong AI-RLA cost-per-window; LiteLLM hard spend caps per window; Azure
`emit-token-metric` dims feed showback; Cloudflare per-request cost + custom cost overrides).

## Adopted model (brief §8 "Adopt")

### 1. Meter on ACTUAL returned tokens (not the estimate)
- **Billing basis = the provider-returned token counts** (`usage.prompt_tokens`,
  `completion_tokens`), including the **streamed usage chunk** (`stream_options.include_usage`).
- The admission-time **estimate** (max-prompt-size precheck, Azure estimate-prompt-tokens) is used
  ONLY to decide whether to admit a request against the remaining budget — never to bill (brief §8).
  This avoids both over- and under-charging.

### 2. Per-invocation metric with attribution dims
- Every invocation emits a `llm.usage.v1` record (see `audit-evidence-emission.md`) with dims:
  **tenant, ingress-token (hashed), provider, model, operation, input/output token counts, cost,
  latency, ttft** (brief §8). These feed the FinOps showback/portal.

### 3. Per-tenant HARD budget caps across concurrent windows
- Budgets are **hard caps** keyed on tenant id, evaluated across **concurrent windows**
  (e.g. `$X/day` AND `$Y/month`, LiteLLM-style). Exceeding any window → 429 `budget_exceeded` for
  **that tenant only** (brief §6, §8).
- An **80%-of-budget soft-warn** is surfaced as a response header (`x-oyatie-tokens-remaining`
  trending toward 0) **before** the hard 429, so callers can react (brief §8).

### 4. Custom per-tenant unit costs
- The gateway supports **per-tenant unit costs** (brief §8 — dogfood/negotiated rates). Oyatie's own
  dogfood tenant may price at provider cost; external tenants at a marked-up rate. Cost in
  `llm.usage.v1` is computed with the tenant's unit cost (`cost_usd_minor_units`).

### 5. Reserved headroom vs shared provider TPM
- Per-tenant budgets are sized so that the sum of reserved headroom stays within the shared provider
  TPM; a tenant exhausting its slice fails *that tenant*, not the gateway (brief §6). See
  `design/tenant-isolation.md`.

## Cost flow

```
  admission:  estimate prompt tokens ──▶ check tenant budget (day AND month)
                                          │ over any window?
                                          ├─ yes ─▶ 429 budget_exceeded (this tenant only)
                                          └─ no  ─▶ admit; reserve estimated tokens
  response:   provider returns usage ──▶ meter on ACTUAL tokens
                                          ├─▶ cost = actual_tokens × per-tenant unit cost
                                          ├─▶ commit against budget windows (reconcile reservation)
                                          └─▶ emit llm.usage.v1 (tenant, provider, model, cost, …)
  showback:   llm.usage.v1 ──▶ FinOps portal rollups (tenant, provider, model, cell)
```

## Tiers

Reusable budget tiers (brief §6 — free/standard/enterprise), each a budget envelope across windows:

| Tier | Daily cap | Monthly cap | Per-request max tokens | Concurrency |
|---|---|---|---|---|
| free | low | low | small | 1 |
| standard | medium | medium | medium | small |
| enterprise | high / custom | high / custom | large | high |
| dogfood (Oyatie self-tenant) | provider-cost-priced | provider-cost-priced | large | high |

(Concrete numeric caps are per-deployment config, sourced from config plus owned secret-provider handles — PRD open-question 2.)

## Sustainability / showback dims

Consistent with the fleet's sustainability emission (the analytics µservice emits
`cost_usd_minor_units` per audit row), the gateway's `llm.usage.v1` carries `cost_usd_minor_units`
and the attribution dims (tenant, provider, model, cell) so per-tenant AI spend rolls up in the
FinOps portal. Carbon/energy attribution for inference is provider-side and out of scope here, but
the cost dim is the gateway's contribution to the ledger.

## Non-claims

- No live billing integration or FinOps portal wiring is implemented by the current foundation; the
  budget tiers and unit costs are config-sourced. Central billing/tenancy integration is a follow-on
  (PRD open-question 2).

## References

- `design/hyperscaler-best-practice-brief.md` §6, §8.
- `contracts/cloud-intelligence.asyncapi.yaml` (`UsagePayload` — the metering record).
- `design/tenant-isolation.md` (reserved headroom), `design/audit-evidence-emission.md`.
- Kong AI-RLA cost; LiteLLM hard spend caps; Azure emit-token-metric; Cloudflare per-request cost (brief §8).
