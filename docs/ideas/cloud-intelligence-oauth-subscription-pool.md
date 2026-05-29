# cloud-intelligence OAuth subscription-pool — idea-refine one-pager

**Status**: ideation artifact (2026-05-28). Forward-pointer to the in-flight cloud-intelligence OAuth ADR (in flight on chore/cloud-intelligence-oauth-pool-2026-05-28).
**Direction**: C (audit/billing/analytics spine from v1).
**Reference repos**: tbphp/gpt-load + songquanpeng/one-api + router-for-me/CLIProxyAPI + nguyenphutrong/quotio.

## Problem Statement

How might we build an OAuth-subscription-pool cloud-intelligence gateway in Oyatie that gives parallel agents transparent access to Claude Code / OpenAI Codex / Gemini CLI subscriptions, with gpt-load's reliability patterns and one-api's provider breadth, while being productizable for Oyatie tenants per oyatie-dogfood-tenancy?

## Recommended Direction — C: audit/billing/analytics spine

Single centralized gateway deployment serving all Oyatie tenants, with per-tenant Cedar-isolated SubscriptionPool. Every request emits a structured event to a configurable sink. Three consumers subscribe independently:

1. Analytics (ADR-0193 ClickHouse OLAP): per-tenant / per-agent / per-provider usage rollups.
2. Audit-chain: immutable log of every LLM request — what was sent (with PII redaction), what came back, who paid for it.
3. Billing: per-tenant chargeback derived from the event stream.

Gateway becomes the canonical event source for AI usage across the Oyatie cloud. Tenants bring their own OAuth subscriptions; gateway provides credential rotation, transparent provider proxying, per-tenant pool isolation, one observable+auditable+billable surface.

## Key Assumptions to Validate

- TOS-clean for productized multi-tenancy (tenants bring own subscriptions to operator-managed proxy) — verify each provider's Team-plan TOS.
- Refresh-token storage in OpenBao at productization scale (100s of tenants × N seats) — benchmark OpenBao read latency at 10k secrets.
- Event-emission throughput at 1000 RPS — stress-test the sink path.
- Per-tenant Cedar isolation correctness — 50+ adversarial Cedar tests.
- Provider rate-limit predictability of gpt-load patterns under OAuth — first-tenant pilot 2-week observation.
- Device-code OAuth bootstrap is one-time-per-seat — 30-day refresh-token stability observation.

## v1 Scope

In: multi-tenant SubscriptionPool kernel (Rust); Cedar policy (per-tenant isolation + admin/proxy realms); Claude Code provider adapter only; event-emission to ClickHouse + Valkey Stream; OpenBao envelope-encrypted refresh tokens; gpt-load patterns (cooldown/blacklist/cron-probe/dual-auth/zero-copy stream); one-api per-provider adapter taxonomy + /admin/test-subscription endpoint; HPA 3→20; Valkey-coordinated cross-replica state.

Out (deferred): OpenAI Codex (v2); Gemini CLI (v3); web admin UI (v2, Leptos not Vue); automated OAuth bootstrap (v2); predictive rate-limit ML (v3); cross-cell pool federation (v3); per-request caching (v2); broader provider taxonomy (backlog); tenant-hash sharding (v3 at 100+ tenants); white-label OSS distro (out of scope).

## Not Doing (and Why)

- Adopt CLIProxyAPI sidecar — violates Rust-everywhere doctrine.
- Per-tenant sidecar (Direction B) — fine at 1-10 tenants, breaks at 100+.
- Consumer-chat-session cookie reuse — TOS violation across all three providers.
- Subscription pooling that Oyatie owns — would constitute resale; forbidden.
- Vue 3 web UI — Leptos per ADR-0372 in v2.
- Real-time WebSocket multiplexing — REST+SSE only in v1.
- Predictive 429-avoidance ML — simple cooldown first; learn from real traffic.

## Open Questions

- Cross-tenant cost amortization (idle Tenant-A seat available to Tenant-B?): likely NO per TOS.
- Per-agent visibility axis within tenant in event stream.
- Audit-chain transport: Kafka vs NATS vs direct ClickHouse vs Valkey Streams (canonical primitive).
- Refresh-token revocation policy on seat-delete.
- First-tenant pilot: Oyatie's own dogfood agent fleet (per oyatie-dogfood-tenancy).

## Amendments to forward to the in-flight ADR

The agent drafting the ADR should add:
- D6 (NEW): event-emission to configurable sink (ClickHouse INSERT + Valkey Stream); pluggable wire-format spec.
- D7 (NEW): per-tenant Cedar policy contract; cross-tenant forbid-wins.
- D8 (NEW): envelope encryption for refresh tokens via OpenBao Transit (DEK per tenant).
- Hyperscaler-lens now scored on: per-tenant isolation adversarial test count + event-emission throughput floor + 10k+ secret storage.
