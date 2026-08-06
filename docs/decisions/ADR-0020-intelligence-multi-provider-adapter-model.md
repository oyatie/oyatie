---
id: ADR-0020
status: Rejected
doc_status: published
---

# ADR-0020: Foundry multi-provider adapter model — `ProviderAdapter` trait, ProviderAuth, capability-level routing

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0021 (capability registry + MCP gateway), ADR-0022 (autonomy ceiling enforcement), ADR-0024 (eval harness + replay), ADR-0026 (in-house model substrate roadmap)

---

## Context

Foundry is the force-multiplier axis: every other axis (cloud, search, ads, saas, vertical, workspace) invokes Foundry capabilities, and each capability invocation must reach a model provider. The current draft surfaces three frontier-LLM vendors (Anthropic, OpenAI, Google), each in two operating modes (subscription-auth, e.g. Claude Pro / ChatGPT Plus / Gemini Advanced — and API-auth, with billed keys). Without a normalized contract, every capability would either pin to a single provider (hard cost-floor; single point of vendor failure) or implement ad-hoc per-provider logic (combinatorial maintenance, no failover, no cost ceiling, no per-tenant routing).

The forces are: (a) we want capability authors to write provider-agnostic code; (b) we want the runtime to route per tenant and per capability so a healthcare tenant can pin a regional pack while a marketing tenant maximizes throughput; (c) we want failover so that one provider's outage does not propagate to every Oyatie surface; (d) we want subscription-auth modes to remain first-class for the developer-tier experience without relaxing the audit posture; (e) we want a per-tenant cost ceiling that the router enforces before the model is hit, not after the bill arrives.

---

## Decision

We introduce a single normalized provider contract in `oya-intelligence-adapter-kernel` and wire every concrete provider through it. The runtime — not the capability author — chooses which adapter handles a given invocation.

### Trait surface (`oya-intelligence-adapter-kernel`)

```rust
// crates/oya-intelligence-adapter-kernel/src/lib.rs
pub trait ProviderAdapter: Send + Sync {
    fn id(&self) -> ProviderId;                      // e.g. "anthropic-api", "openai-subscription"
    fn auth_mode(&self) -> ProviderAuthMode;         // Api | Subscription
    fn capabilities(&self) -> ProviderCapabilitySet; // tools, vision, streaming, function-calling
    fn invoke(
        &self,
        prompt: PromptEnvelope,
        tools: ToolSchemaSet,
        policy: InvocationPolicy,
    ) -> impl Stream<Item = Event> + Send;
}

pub enum ProviderAuth {
    Subscription {
        session_token: SessionTokenRef, // rotated; vault-backed
        provider_account: ProviderAccountId,
    },
    Api {
        secret_ref: SecretRef,           // resolved by SecretProvider, never inlined
        billing_account: BillingAccountId,
    },
}

pub struct InvocationPolicy {
    pub max_cost_usd: Decimal,
    pub max_latency_ms: u32,
    pub data_class_allowlist: DataClassSet,
    pub residency: RegionPackId,
    pub failover_chain: Vec<ProviderId>, // resolved per capability
}

pub enum Event {
    StreamStart { provider: ProviderId, request_id: RequestId },
    Token(TokenChunk),
    ToolCall(ToolCallChunk),
    Usage(UsageRecord),     // tokens in/out, USD attributed
    StreamEnd(EndReason),
    Error(AdapterError),    // typed; failover trigger conditions are explicit
}
```

### Concrete adapter crates

One crate per (provider × auth-mode):

- `oya-intelligence-adapter-anthropic-api`
- `oya-intelligence-adapter-anthropic-subscription`
- `oya-intelligence-adapter-openai-api`
- `oya-intelligence-adapter-openai-subscription`
- `oya-intelligence-adapter-gemini-api`
- `oya-intelligence-adapter-gemini-subscription`

Subscription adapters are **headless adapters behind an authenticated session**: they manage a rotating session-token vault (`oya-intelligence-adapter-session-vault`) backed by the SecretProvider; tokens rotate on a configured cadence and on every detected challenge. API adapters resolve their secret via `SecretProvider` (OpenBao-backed per the platform secret contract) — the secret is never inlined in catalog YAML, never logged, and never returned in audit-chain payloads.

### Routing decision

The runtime computes the route per invocation using a deterministic resolver:

```rust
// crates/oya-intelligence-adapter-router/src/lib.rs
pub fn resolve_route(
    capability: &Capability,
    tenant: &Tenant,
    request: &InvocationRequest,
) -> Result<RouteChain, RouteError> {
    // 1. Start from capability.provider_preference (ordered)
    // 2. Filter by tenant.region_pack residency rules
    // 3. Filter by capability.data_class_allowlist
    // 4. Filter by tenant.cost_ceiling (per capability + monthly)
    // 5. Apply tenant.provider_overrides (e.g. "this tenant pins claude-api for all PHI capabilities")
    // 6. Return ordered chain: [primary, failover1, failover2, ...]
    //    Example outcome: [claude-api, openai-api, gemini-subscription]
}
```

### Failover semantics

Failover is at the **invocation** boundary (not mid-stream). A `RetryableError` from the primary causes the router to drop to the next adapter in the chain; non-retryable errors (e.g. policy refusal, cost-ceiling breach) terminate the invocation. Failover events are emitted to the audit chain as `EVT-FOUNDRY-PROVIDER-FAILOVER`.

### Per-tenant cost ceiling

`InvocationPolicy.max_cost_usd` is a hard pre-flight cap: the router refuses to dispatch if the running monthly tenant spend plus the worst-case projected invocation cost would exceed the ceiling. Soft warnings fire at 80%; hard stop at 100%. Per-tenant per-capability ceilings override the global tenant ceiling.

### CI lanes

- `foundry-adapter-contract` — verifies every concrete adapter satisfies the `ProviderAdapter` trait and all events match the kernel schema (compile-time + integration smoke).
- `foundry-adapter-failover` — synthetic chaos lane: forces primary adapter errors and asserts the router walks the chain correctly.
- `foundry-adapter-cost-ceiling` — invariant lane: asserts the router rejects invocations that would breach the ceiling.
- `foundry-adapter-secret-isolation` — asserts no adapter logs, persists, or returns the resolved secret in any payload.

---

## Consequences

### Positive
- Capability authors target one trait; the trait is provider-shape, not provider-specific.
- Subscription mode is first-class — developer-tier experience is preserved without weakening the audit posture.
- Failover is structural; no capability needs to handle multi-provider logic in domain code.
- Cost ceilings are enforced before model hits, eliminating the "surprise bill" failure mode.
- Adding a new provider is one crate plus a router preference entry; no other capability needs to change.

### Negative
- The trait surface forces every provider to fit the streaming-event shape; providers with quirky APIs require adapter-internal translation that may lose fidelity.
- Subscription session-token rotation is a non-trivial substrate (vault, refresh worker, challenge detection) that adds operational surface.
- Cost-ceiling pre-flight requires a per-provider cost-projection model; mis-projection produces false rejects.

### Operational
- Runbook: `runbooks/foundry/provider-quota-exhausted.md` — documents how to roll a provider out of the failover chain during incidents.
- Runbook: `runbooks/foundry/subscription-token-expired.md` — documents the rotation cadence, challenge handling, and break-glass for revoked sessions.
- On-call: subscription-auth adapters are the most fragile surface; alerts on session-revoke rate.
- Per-release: every new adapter goes through the contract lane and the failover lane.

---

## Alternatives considered

1. **Single provider (Anthropic-only).** Pros: simplest. Cons: vendor-lock; no failover; impossible at the cost-floor of a multi-region GA. Rejected — kills the force-multiplier thesis.
2. **Per-capability provider hard-coding.** Pros: capability author has full control. Cons: combinatorial maintenance; no per-tenant routing; failover requires ad-hoc per-capability code. Rejected — non-cohesive.
3. **External orchestration framework (LangChain / LangGraph as the runtime).** Pros: off-the-shelf. Cons: external system-of-record for capability execution; license + maturity + cohesion concerns; cannot enforce our autonomy ceiling shape; cannot enforce our cost ceiling pre-flight. Rejected per the build-vs-buy posture (Foundry runtime is in-house-obligatory).
4. **Subscription-only mode (no API mode).** Pros: cheaper for low-volume. Cons: subscription terms forbid heavy programmatic use at production scale; no SLA. Rejected.
5. **API-only mode (no subscription mode).** Pros: enterprise-clean. Cons: developer-tier and dev-loop experience suffers; loses cost-arbitrage when subscription seats are already paid. Rejected.

---

## Resolved (this revision, 2026-05-09)

1. **Subscription-mode tenant attribution.** Foundry maintains a `SubscriptionBinding` registry mapping each developer-account-held subscription to a scoped set of tenants (one-to-many or one-to-one) at registration time. Per-invocation cost attribution flows through the binding to the tenant cost-center. Subscription invocations without an active `SubscriptionBinding` are denied at the adapter boundary; the binding registration itself emits to the audit chain. Cross-tenant subscription pooling requires explicit Founder + council-architecture sign-off and a per-pool cost-attribution policy.
2. **Mid-stream partial-then-error failover policy: restart, not replay.** When the primary adapter emits a partial token stream then errors, the partial response is discarded, the original prompt is restarted on the failover adapter, and a `ResponseRestartEvent` is emitted to the audit chain so observability attributes cost to both attempts. Replay-mid-stream is rejected because token-level position semantics differ across providers (Anthropic / OpenAI / Gemini tokenize differently and would produce inconsistent stitched output). Capabilities that cannot tolerate restart (e.g. side-effecting tool-use chains) declare `failover_mode: deny` in their capability record and surface the original error to the caller.
3. **Per-tenant cost ceiling enforcement** is split: Cedar policy authors the *predicate* (per-tenant per-capability budget headroom) so it composes with the autonomy ceiling check; a dedicated `oya-intelligence-cost-budget-kernel` evaluates the predicate at adapter pre-flight and tracks the running spend window. Same surface as autonomy; same audit-chain footprint.
4. **Subscription session-token vault** is the platform `SecretProvider` (per ADR-0043 secrets-management) with a Foundry-specific lease type that knows how to refresh subscription challenges. No Foundry-internal duplicate vault.

## Open questions

(none material as of 2026-05-09; the four prior open questions are resolved above. Future open questions land here as they surface.)

---

## References

- Internal: ADR-0021 (capability registry; consumes the route chain), ADR-0022 (autonomy ceiling; gates whether a route is allowed at all), ADR-0024 (eval harness; A/B tests routing decisions), ADR-0026 (in-house substrate; extends the trait to `oya-internal-<model-id>`).
- Architectural posture: in-house obligatory (per the build-vs-buy matrix); external providers are adapters behind a sealed port.
- Flat-crates binding: the sealed provider-adapter contract lives in `crates/oya-intelligence-adapter-kernel`; provider-specific adapters land under `crates/oya-foundry-*` or approved runtime crates. The retired `services/agent/daemon` path is historical only and must not be recreated.
