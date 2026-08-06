# Cloud Intelligence service — Tenant Isolation

**Authority:** ADR-0373 (per-tenant key pools + isolation), owned policy-engine port
**Research grounding:** `design/hyperscaler-best-practice-brief.md` §6 (multi-tenant isolation + quotas — per-consumer keys, token-aware limits on an arbitrary counter-key, multi-level limits, reserved headroom).
**Last reviewed:** 2026-05-26

## The isolation unit: the per-tenant key pool

The brief (§6 "Adopt") is explicit: the **per-tenant key pool is the isolation unit**, and limits
are keyed on **tenant id, not the shared provider key**. Oyatie is a tenant of itself
(`oyatie-dogfood-tenancy`) — the dogfood tenant uses the same model, no internal bypass.

This is the central design choice that distinguishes a production gateway from the agent-dispatch
foundation: the foundation authenticated ingress proxy-keys but did **not** implement per-tenant
token-bucket isolation (`manifest.json` non-claim `tenant_rate_limit: not_claimed_runtime`). This
document specs the production posture.

## Adopted model (brief §6 "Adopt")

### 1. Tenant resolved from the ingress token
- The ingress bearer token maps to a tenant id (and its budget tier). The tenant is **derived from
  the token**, never trusted from a client-supplied header (the `X-Oyatie-Tenant` override is honored
  only for privileged dogfood/admin principals). The current Cedar policy file is
  a transient fixture for the owned policy-engine port.

### 2. Limits keyed on tenant id (not the shared provider key)
- Rate + token + cost limits are evaluated on the **tenant id** as the counter-key (Azure
  `llm-token-limit` on an arbitrary counter-key: subscription/IP/expression). This is what stops one
  tenant from consuming the shared provider key's whole TPM (brief §6).

### 3. Multi-scope, concurrent-window limits
- Limits apply at multiple scopes (**global → tenant → ingress-token**) and across **concurrent
  budget windows** (e.g. day AND month), LiteLLM-style multi-level limits. The most restrictive
  binding wins.

### 4. Reserved per-tenant headroom vs shared provider TPM
- Each tenant gets a reserved slice of the shared provider TPM. The sum of reserved headroom stays
  within the provider TPM, so **a tenant exhausting its slice fails THAT tenant with 429, not the
  gateway** (brief §6). This is the key blast-radius property: a noisy tenant cannot starve the
  fleet.

### 5. Reusable tiers
- Free / standard / enterprise / dogfood tiers (brief §6), each a reusable budget+limit envelope.
  See `design/cost-finops.md` for the budget side.

## Isolation layers (defense in depth)

| Layer | Mechanism | Prevents |
|---|---|---|
| Identity | tenant derived from ingress token (constant-time) | tenant spoofing |
| Authorization | owned policy-engine `principal.tenant_id == resource.tenant_id`; cross-tenant default-deny | cross-tenant access |
| Key pool | per-tenant pool selection | one tenant's key failures tripping another's pool |
| Budget | tenant-keyed token/cost caps, concurrent windows | denial-of-wallet spillover (OWASP LLM10) |
| Headroom | reserved per-tenant slice of provider TPM | one tenant starving the shared provider |
| Audit/metering | per-tenant `tenant_id` on every record | attribution + repudiation |
| Residency | per-tenant region pin for body-spill | cross-region data movement |

## Why per-tenant pools (not a single shared pool)

A single shared pool would mean one tenant's failing keys (e.g. a tenant whose traffic trips
rate-limits) would blacklist keys that *every* tenant depends on — a noisy-neighbor blast radius.
Per-tenant pools confine key-state transitions to the tenant that caused them. The pure kernel
(`KeyPool`) is per-pool by construction; the rest crate instantiates one pool (or pool set) per
tenant×provider.

## Dogfood tenancy (oyatie-dogfood-tenancy)

Oyatie's own AI agent fleet is a tenant of this gateway, with **no internal bypass** of the tenant
model (per the memory doctrine `oyatie-dogfood-tenancy`). The dogfood tenant has its own pool,
budget tier (provider-cost-priced), and audit/metering — it is metered and isolated exactly like an
external tenant. This is what makes the gateway's isolation claims credible: the operator eats its
own dog food.

## Interaction with cellular sharding

The gateway's `manifest.json#sharding_automation` retains the fleet doctrine (ADR-0348:
autosharding / auto-rebalance / dynamic-sharding) as intended-future control-plane-driven behavior;
the current foundation does not implement key-pool shard split/migration (manifest non-claim). When
sharding lands, tenant→cell placement honors residency + compliance packs (the same constraints as
the rest of the fleet), and per-tenant pools move with the tenant.

## Non-claims

- Per-tenant token-bucket rate limiting is **not** implemented by the current foundation
  (`CS-CLOUD-INTELLIGENCE-AGENT-DISPATCH-001`). This document specs the production posture (IP-001 T4); the
  manifest's `tenant_rate_limit: not_claimed_runtime` non-claim stands until that lands.

## References

- `design/hyperscaler-best-practice-brief.md` §6, §8.
- `design/cost-finops.md`, `policy/cloud-intelligence.cedar`, `design/data-residency.md`.
- Azure `llm-token-limit` (counter-key); LiteLLM virtual keys / multi-level limits; Kong AI-RLA (brief §6).
