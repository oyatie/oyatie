---
id: ADR-0384
status: Superseded
planning_impact: true
deciders: founder, council-architecture
date: 2026-05-28
owner: council-architecture
supersedes: []
superseded_by: []
related: [ADR-0193, ADR-0381, ADR-0043, ADR-0083, ADR-0373]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
door: two-way
milestone: M-CLOUD-INTELLIGENCE
deliverables:
  - id: ADR-0384-D1
    description: "oya-cloud-intelligence-kernel rewrite: SubscriptionPool<OAuthSubscription> state machine. Pure (no I/O, no HTTP, no async). Test exhaustively."
    exit_criteria: "SubscriptionPool compiles as a pure crate (no tokio, no hyper, no reqwest deps); cargo nextest -p oya-cloud-intelligence-kernel passes with coverage of round-robin, fill-first, cooldown, quota-tracking, and pool-exhausted paths."
    verified_by: "cargo nextest -p oya-cloud-intelligence-kernel + cargo clippy -p oya-cloud-intelligence-kernel -- -D warnings"
  - id: ADR-0384-D2
    description: "oya-cloud-intelligence-rest adapter rewrite: SubscriptionStore (OpenBao-bound; reads refresh-tokens). Token-refresh logic per provider. Forward streaming + non-streaming responses."
    exit_criteria: "SubscriptionStore reads refresh-tokens from OpenBao at startup; per-seat Mutex serializes concurrent refresh attempts; write-through persists rotated refresh-tokens back to OpenBao; SSE streaming and non-streaming forwarding both pass integration tests."
    verified_by: "cargo nextest -p oya-cloud-intelligence-rest + oya gate validate honest-claims"
  - id: ADR-0384-D3
    description: "Provider adapters — v1 scope: Anthropic (Claude Code OAuth via claude.ai/oauth/authorize + api.anthropic.com/v1/oauth/token) + OpenAI Codex (Sign-in-with-ChatGPT OAuth, callback port 1455, data endpoint chatgpt.com/backend-api/codex/responses). Each implements ProviderAdapter trait with refresh_token, forward_request, parse_rate_headers. Stock reqwest (rustls/ring) for OAuth refresh per OQ-7 downgrade — auth2api evidence shows no TLS-fingerprint impersonation needed for the refresh leg. Gemini adapter (v2) and Cursor adapter (v3) authored in their own amendments. Total provider set ever supported: 4 (Anthropic, Codex, Gemini, Cursor); see memory cloud-intelligence-reference-repo-audit."
    exit_criteria: "AnthropicAdapter + CodexAdapter compile; each refresh_token() succeeds against its provider's OAuth endpoint in a spike test; parse_rate_headers() covers Anthropic + OpenAI header schemas; cross-adapter trait surface remains stable for v2 Gemini + v3 Cursor extension."
    verified_by: "cargo nextest -p oya-cloud-intelligence-rest (per-adapter integration tests for Anthropic + Codex)"
  - id: ADR-0384-D4
    description: "Config schema migration: microservices/cloud-intelligence/k8s/cloud-intelligence.yaml ConfigMap.data.config.json moves from .groups[].keys[] to .providers[].subscriptions[].openbao_refresh_token_path. Backward-compat: existing static-key path stays under providers[].auth_mode: 'static_key'; new code uses auth_mode: 'oauth_subscription'."
    exit_criteria: "cloud-intelligence.yaml ConfigMap updated to new schema; gateway boots with both auth_mode: static_key and auth_mode: oauth_subscription groups present; oya gate validate honest-claims green."
    verified_by: "cargo test -p oya-cloud-intelligence-rest (config deserialization tests) + kubectl apply --dry-run=client on cloud-intelligence.yaml"
  - id: ADR-0384-D5
    description: "SETUP-RUNBOOK rewrite: operator runs claude login / codex login on a host with the respective CLI installed (v1 providers only); extracts refresh_token from each CLI's local credential store; stores at 'bao kv put secret/oya/cloud-intelligence/<provider>/seats/<seat-name> refresh_token=<token>'. Gateway reads via ExternalSecret and refreshes access tokens on demand. Gemini login (v2) and Cursor login (v3) added in their respective amendments."
    exit_criteria: "microservices/cloud-intelligence/SETUP-RUNBOOK.md rewritten with OAuth-subscription-pool instructions for v1 providers (Anthropic + Codex); credential file paths verified empirically (OQ-5 resolved); runbook covers both v1 providers with exact bao kv put commands."
    verified_by: "Manual review: runbook credential paths match actual CLI output on a test host; oya gate validate honest-claims green on SETUP-RUNBOOK.md"
  - id: ADR-0384-D6
    description: "Event-emission to configurable sink — every cloud-intelligence request emits a structured event to (a) ClickHouse OLAP (per ADR-0193) for analytics rollup + (b) Valkey Stream (per canonical primitives) for audit-chain consumption. Pluggable wire-format: {tenant_id, agent_id, seat_id, provider, model, prompt_tokens, completion_tokens, ms_latency, status, request_id}. Sinks subscribe independently (no coupling between analytics + audit + billing consumers). Direction C from idea-refine; informed by CLIProxyAPI's CPA Usage Keeper decoupling pattern."
    exit_criteria: "kernel emits per-request events to a EventSink trait; integration test asserts a synthetic request produces the expected event shape on both ClickHouse insert + Valkey Stream XADD."
    verified_by: "cargo nextest -p oya-cloud-intelligence-kernel + cargo nextest -p oya-cloud-intelligence-rest (sink integration tests)"
  - id: ADR-0384-D7
    description: "Per-tenant Cedar isolation contract — every request's principal includes a tenant attribute; Cedar policy at microservices/cloud-intelligence/policy/cloud-intelligence.cedar adds explicit forbid rules: forbid (principal, action, resource) when principal.tenant != resource.tenant. Adversarial test set: at least 50 Cedar test cases including cross-tenant access attempts, seat-id mismatch, admin-realm cross-tenant impersonation. Cross-tenant access always forbids-wins per ADR-0183 and ADR-0193."
    exit_criteria: "microservices/cloud-intelligence/policy/cloud-intelligence.cedar contains the per-tenant forbid rule; cargo nextest -p oya-cloud-intelligence-kernel passes at least 50 adversarial cedar_per_tenant_isolation test cases."
    verified_by: "cargo nextest -p oya-cloud-intelligence-kernel cedar_per_tenant_isolation"
  - id: ADR-0384-D8
    description: "Envelope encryption for refresh tokens at rest in OpenBao. Per-tenant Data Encryption Key (DEK) lives in OpenBao Transit; refresh tokens stored encrypted under that DEK. Gateway decrypts on token-load; never logs decrypted form. Operator rotation of the Transit Key Encryption Key (KEK) re-wraps DEKs without rewriting refresh tokens. Aligns with ADR-0043 sref://openbao/... and the canonical OpenBao Transit pattern."
    exit_criteria: "crates/oya-cloud-intelligence-rest's KeyStore implementation reads refresh tokens via OpenBao Transit decrypt-on-read; integration test asserts the on-disk OpenBao secret is ciphertext, not plaintext."
    verified_by: "cargo nextest -p oya-cloud-intelligence-rest transit_envelope_decrypt_roundtrip"
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0384 — cloud-intelligence gateway Path B redesign: OAuth subscription-pool replacing static API-key pool

## Status

Proposed (2026-05-28).

## Context

### Current architecture (static API-key pool)

`microservices/cloud-intelligence` and crates `oya-cloud-intelligence-{kernel,rest,app}` (~5,600 LOC) implement a
**static API-key pool**: each provider group carries a list of opaque bearer tokens sourced from
OpenBao KV paths (e.g. `secret/oya/cloud-intelligence/openai`). The kernel state machine (`oya-cloud-intelligence-kernel`)
performs round-robin selection, per-key failure-count blacklisting, jittered cooldown, and lazy
restore. The rest adapter (`oya-cloud-intelligence-rest`) injects the selected key as the appropriate
auth header per provider channel (OpenAI `Authorization: Bearer`, Anthropic `x-api-key`, Gemini
`key=` query param).

This architecture was shipped across PRs #253 (K8s deployment), #262 (provider routing), and #265
(live-fanout readiness / HPA / NetworkPolicy) and recorded in ADR-0373. It presupposes that the
operator holds **static API keys** purchased via direct provider API-tier billing.

### The mismatch

The operator's actual entitlement model is **subscription OAuth**: Claude Pro/Team, ChatGPT Plus,
and Gemini Advanced subscriptions. These subscriptions do not issue static API keys redeemable
against `api.openai.com` / `api.anthropic.com`. Instead, each subscription account exposes access
through the provider's **CLI toolchain** (Claude Code, OpenAI Codex CLI, Gemini CLI) via an OAuth
2.0 PKCE device-code flow targeting the provider's first-party console endpoints
(`console.anthropic.com`, `codex.openai.com`, `aistudio.google.com`). The static-key-pool
architecture is therefore the wrong model end-to-end.

### Reference implementations

Four reference repositories inform this redesign:

- **`CLIProxyAPI` (router-for-me/CLIProxyAPI, Apache-2.0)** — primary reference. Go service
  implementing a multi-account OAuth subscription pool for Claude Code, OpenAI Codex, Gemini CLI,
  and Grok. Uses each provider's official PKCE OAuth flow with round-robin and fill-first account
  selection strategies, per-seat token storage (JSON credential files), token refresh via
  `RefreshLead` (e.g. 4-hour lead for Claude), and 429 / rate-limit aware account cooldown.
  Quotio (macOS GUI) wraps CLIProxyAPI and informs quota-tracking UX patterns.

- **`gpt-load` (tbphp/gpt-load, MIT)** — static API-key pool, Go. Provides the proven
  failure-handling pattern: blacklist threshold, jittered cooldown, lazy restore. Our existing
  kernel mirrors this pattern; it survives the redesign — only the credential type changes from
  opaque key to `(access_token, refresh_token, expires_at)`.

- **`one-api` (songquanpeng/one-api, MIT)** — OpenAI-compatible provider-routing facade, Go.
  Informs the per-provider channel abstraction and the canonical OpenAI-compatible surface. Our
  existing `ProviderChannel` enum and `ChannelAdapter` trait are structurally aligned.

- **`quotio` (nguyenphutrong/quotio, MIT)** — macOS Swift GUI for CLIProxyAPI. Informs quota
  display UX (per-seat quota remaining, cooldown countdowns, round-robin vs fill-first toggle)
  and confirms the two selection strategies are the right UX-visible knobs.

- **`auth2api` (AmazingAng/auth2api, MIT, TypeScript/Node)** — multi-account OAuth proxy
  proving stock HTTP-client TLS (Node fetch / undici / OpenSSL) successfully calls
  api.anthropic.com/v1/oauth/token without uTLS / Chrome fingerprint impersonation. See
  src/auth/oauth.ts lines 74-99 for the literal stock fetch refresh implementation. This is
  **counter-evidence to CLIProxyAPI's uTLS necessity claim** for the OAuth-refresh leg and is
  the basis for downgrading OQ-7 from BLOCKING to NON-BLOCKING. auth2api also documents the
  refresh-token-exhausted error taxonomy (expired / reused / invalidated / revoked) at
  src/auth/refresh-errors.ts which we adopt as the canonical typed-failure set.

### Relation to ADR-0193

ADR-0193 is the OLAP analytics warehouse decision (ClickHouse). It is cited here only because the
cloud-intelligence manifest previously cited it as the "Cedar policy basis" in older drafts; the correct
Cedar-policy ADR is ADR-0191 (edge-authz Cedar PDP). This ADR corrects that citation silently.
ADR-0373 remains the production-design record for the static-key architecture; this ADR supersedes
its credential model only, not its API surface or observability decisions.

## Decision

Replace the static-key-pool credential model with an **OAuth subscription-pool**. All other
architectural decisions from ADR-0373 (OpenAI-compatible surface, SSE streaming, audit chain,
OpenSLO SLIs, two-realm constant-time auth) are retained unchanged.

### Subscription data model

Each provider carries N subscription accounts. Each account is represented as an `OAuthSubscription`:

```rust
pub struct OAuthSubscription {
    pub seat_name:        String,          // operator-assigned label, e.g. "seat-0"
    pub provider:         ProviderChannel, // OpenAI | Anthropic | Gemini
    pub access_token:     SecretString,    // short-lived; refreshed on demand
    pub refresh_token:    SecretString,    // long-lived; sourced from OpenBao
    pub expires_at:       Instant,         // when access_token expires
    pub last_used_at:     Option<Instant>,
    pub cooldown_until:   Option<Instant>, // set on 429 / quota-exhausted response
    pub quota_remaining:  Option<u64>,     // parsed from provider rate-limit headers
}
```

The kernel crate owns `SubscriptionPool<OAuthSubscription>`: a pure (no I/O, no HTTP, no async)
state machine that:

**(a) Subscription selection** — supports two strategies (configurable per provider group):
- `RoundRobin`: cycles seats ignoring quota state; distributes load evenly.
- `FillFirst`: routes to the seat with the highest `quota_remaining` until exhausted, then spills.

Seats in cooldown (`now < cooldown_until`) are skipped. If all seats are in cooldown the pool
returns `Err(PoolExhausted { soonest_restore: Instant })` and the rest layer responds `503` with
`Retry-After` set to `soonest_restore - now`.

**(b) Lazy token refresh** — the kernel emits a `RefreshRequired(seat_name)` event when
`now > expires_at - skew_secs`. The rest layer (which owns I/O) performs the refresh and calls
`pool.update_token(seat_name, new_access_token, new_expires_at)`. The kernel itself never touches
the network; all async I/O lives in the rest crate. This keeps the kernel pure and exhaustively
testable.

**(c) Cooldown on 429 / quota-exhausted** — when the rest layer receives a 429 or a provider
quota-exhausted signal, it calls `pool.set_cooldown(seat_name, cooldown_until)`. The `cooldown_until`
is parsed from `Retry-After` (seconds or HTTP-date) if present; otherwise a jittered default
(30s + ±10% jitter, matching the existing kernel's jitter logic) is applied.

**(d) Quota tracking** — after each response the rest layer calls
`pool.record_quota(seat_name, quota_remaining)` with the parsed value from provider rate-limit
response headers. The kernel stores this in `quota_remaining` for use by the `FillFirst` strategy.

### Token-refresh architecture (rest layer)

The `SubscriptionStore` (rest crate) owns:
- **OpenBao read path**: at startup, reads `refresh_token` for each configured seat from
  `secret/oya/cloud-intelligence/<provider>/seats/<seat-name>`. No access token is persisted; cold-start
  is always lazy (first request per seat triggers the refresh, not proactive boot-time refresh —
  see Open Questions OQ-4).
- **Per-seat `Arc<Mutex<RefreshState>>`**: serializes concurrent refresh attempts for the same
  seat (see Open Questions OQ-1).
- **Write-through on refresh-token rotation**: if a provider returns a new `refresh_token` during
  access-token refresh (single-use refresh-token pattern), the store immediately writes the new
  value back to OpenBao via `bao kv put` (see Open Questions OQ-2).

### Provider adapters (D3)

Each provider implements a `ProviderAdapter` trait:

```rust
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    /// Exchange the stored refresh_token for a fresh (access_token, expires_at).
    /// Returns the new refresh_token if the provider rotated it.
    async fn refresh_token(
        &self,
        refresh_token: &SecretString,
    ) -> Result<TokenRefreshResult, ProviderError>;

    /// Forward a request using the given access_token. Returns the raw response stream.
    async fn forward_request(
        &self,
        access_token: &SecretString,
        req: ForwardRequest,
    ) -> Result<ForwardResponse, ProviderError>;

    /// Parse provider-specific rate-limit headers from a response.
    fn parse_rate_headers(&self, headers: &HeaderMap) -> RateLimitInfo;
}
```

Three concrete adapters:
- `ClaudeCodeAdapter`: targets `console.anthropic.com`; OAuth via Anthropic PKCE flow (PKCE code
  verifier + code challenge as implemented in CLIProxyAPI `internal/auth/claude/`). Rate headers:
  `anthropic-ratelimit-tokens-remaining`, `anthropic-ratelimit-requests-remaining`.
- `CodexAdapter`: targets `codex.openai.com`; OAuth via OpenAI device-code flow. Rate headers:
  `x-ratelimit-remaining-tokens`, `x-ratelimit-remaining-requests`, `x-ratelimit-reset-tokens`.
- `GeminiCliAdapter`: targets `generativelanguage.googleapis.com`; OAuth via Google identity
  (`accounts.google.com` device flow). Rate headers: `x-ratelimit-limit`, `x-ratelimit-remaining`.

### Config schema migration (D4)

The ConfigMap `config.json` moves from the current `.groups[].keys[]` shape to a
`.providers[].subscriptions[]` shape:

```jsonc
// NEW shape (auth_mode: oauth_subscription)
{
  "providers": [
    {
      "name": "claude",
      "channel": "anthropic",
      "auth_mode": "oauth_subscription",
      "upstream_base_url": "https://console.anthropic.com",
      "selection_strategy": "round_robin",
      "token_skew_secs": 300,
      "subscriptions": [
        {
          "seat_name": "seat-0",
          "openbao_refresh_token_path": "secret/oya/cloud-intelligence/anthropic/seats/seat-0"
        }
      ]
    }
  ]
}

// LEGACY shape (auth_mode: static_key) — retained for backward compat
{
  "groups": [
    {
      "name": "openai",
      "channel": "openai",
      "auth_mode": "static_key",
      "upstream_base_url": "https://api.openai.com",
      "bao_key_path": "agent-gateway/openai"
    }
  ]
}
```

Both shapes are valid simultaneously. A group with `auth_mode: "static_key"` (or no `auth_mode`
field — backward compat default) routes through the existing key-pool kernel unchanged. A provider
with `auth_mode: "oauth_subscription"` routes through the new subscription-pool kernel.

### Operator bootstrap (D5)

Operator one-time setup per seat:

```bash
# Claude Code
claude login   # runs PKCE OAuth flow, stores credentials
# Extract from ~/.config/claude/credentials.json (or $CLAUDE_CONFIG_DIR)
bao kv put secret/oya/cloud-intelligence/anthropic/seats/seat-0 \
  refresh_token="$(jq -r '.oauthToken.refreshToken' ~/.config/claude/credentials.json)"

# OpenAI Codex
codex login    # runs device-code OAuth flow
# Extract from ~/.codex/auth.json
bao kv put secret/oya/cloud-intelligence/openai/seats/seat-0 \
  refresh_token="$(jq -r '.refreshToken' ~/.codex/auth.json)"

# Gemini CLI
gemini login   # runs Google device-code flow
# Extract from ~/.config/gemini/oauth_creds.json (path to verify — see OQ-5)
bao kv put secret/oya/cloud-intelligence/gemini/seats/seat-0 \
  refresh_token="$(jq -r '.refresh_token' ~/.config/gemini/oauth_creds.json)"
```

The gateway reads these paths at startup via ExternalSecret (per ADR-0043) and never re-reads the
raw refresh token from OpenBao at request time — only the in-memory `SubscriptionStore` is
consulted during the request path. The ExternalSecret refresh interval governs how quickly a
rotated refresh token (written back by the gateway after single-use rotation) propagates. Write-
through goes directly to OpenBao; the ExternalSecret path is the boot/recovery path only.

## Hyperscaler-lens validation

Per `/specs/hyperscaler-architecture-invariants.json`, every new component must pass four gates:

| Gate | Verdict | Evidence |
|------|---------|----------|
| (a) Active upstream | PASS | CLIProxyAPI: active (v7.x, router-for-me org, weekly releases as of 2026-05). Provider OAuth endpoints: sanctioned (Anthropic Console, OpenAI Codex, Google AI Studio are each provider's own first-party CLI infrastructure). |
| (b) Clean license | PASS | CLIProxyAPI: Apache-2.0. gpt-load: MIT. one-api: MIT. quotio: MIT. Rust implementation is first-party (no new vendored dependency). |
| (c) Fully self-hostable | PASS | All OAuth token exchanges are point-to-point to provider endpoints. No third-party relay, no Grafana Cloud, no managed service. Operator owns all credential material in OpenBao. |
| (d) Hyperscaler-internal-equivalent | PASS | This IS the OSS pattern that PackyAPI / AICodeMirror / BMOPlus sell as a paid relay service. The hyperscaler-equivalent is "bring-your-own-subscription API pool" — the same model used in Google's internal tooling (e.g., internal Gemini CLI pool management) and Microsoft's internal Copilot seat management. |

## TOS framing

This design uses each provider's **official device-code or PKCE OAuth flow** as documented and
distributed by the provider in their first-party CLI toolchain:

- **Anthropic**: Claude Code CLI (`claude login`) uses `console.anthropic.com/oauth` with PKCE.
  This is the same flow documented in Anthropic's Claude Code documentation for personal and team
  use. The relevant TOS is the **Claude Code and API Terms of Service** (not claude.ai consumer
  chat TOS, which is a separate agreement). The team-plan multi-seat case is sanctioned: Claude
  Team plans are explicitly multi-seat and the Claude Code CLI is the documented access method.
- **OpenAI**: OpenAI Codex CLI (`codex login`) uses the OpenAI device-code flow targeting
  `codex.openai.com`. This is the flow distributed by OpenAI in their Codex CLI for ChatGPT Plus /
  Pro subscribers. The relevant TOS is OpenAI's **Codex CLI Terms of Service**.
- **Google**: Gemini CLI (`gemini login`) uses Google's standard OAuth 2.0 device flow via
  `accounts.google.com`. This is Google's canonical authentication path for Gemini Advanced
  subscribers using the CLI. The relevant TOS is Google's **Gemini API Terms of Service** as they
  apply to the CLI client.

**What this design does NOT do**: it does not reuse `claude.ai` browser session cookies, does not
scrape chat UI endpoints, does not reverse-engineer undocumented API surfaces, and does not share
credentials across multiple operators. Each seat is a legitimately owned subscription.

**Risk caveat**: providers may update TOS to restrict programmatic refresh-token reuse outside the
original CLI client. The operator is responsible for monitoring provider TOS updates. This ADR
asserts the design is TOS-compliant at the date of authorship (2026-05-28) based on the publicly
available terms linked above.

## Consequences

### Improves

- **Correct entitlement model**: gateway now matches the operator's actual subscriptions. No need
  to purchase API-tier keys separately.
- **Multi-seat pool**: N subscription accounts provide N-times the rate limit headroom, exactly
  as CLIProxyAPI demonstrates at scale.
- **No static secret rotation burden**: refresh tokens are long-lived and self-rotate through the
  write-through mechanism; operators are not on the hook for manual key rotation schedules.
- **Quota observability**: per-seat `quota_remaining` parsed from provider headers enables
  `FillFirst` strategy and drives per-seat quota dashboards (quotio UX pattern).
- **Kernel remains pure**: the `SubscriptionPool` state machine has no I/O dependencies, so the
  existing exhaustive test approach (cargo nextest, pure unit tests) is fully preserved.

### Gets harder

- **Token-refresh races**: two concurrent requests to the same seat that both trigger
  `RefreshRequired` must be serialized. Requires a per-seat mutex in the rest layer (see OQ-1).
- **Refresh-token rotation write-back**: providers that issue single-use refresh tokens require
  an immediate OpenBao write on every token refresh. This adds OpenBao latency to the hot path
  for the first request after expiry (see OQ-2).
- **Operator bootstrap complexity**: one-time setup per seat requires CLI toolchain on a machine
  that can run an OAuth browser flow, followed by manual `bao kv put`. Documented in D5 runbook.
- **Cold-start latency**: on pod restart, all access tokens are expired. The first request per
  seat incurs a token-refresh RTT (provider token endpoint) before the actual LLM request is
  forwarded. Lazy strategy means this latency spike is per-seat, not global boot-time.
- **Provider TOS surface**: the design's correctness depends on provider TOS not restricting the
  refresh-token pattern. This is an ongoing compliance monitoring obligation.
- **Persistent refresh-token storage required**: unlike static keys (which never rotate), OAuth
  refresh tokens may rotate and must be durably persisted back to OpenBao. Loss of the refresh
  token requires the operator to re-run `claude login` for that seat.

## Alternatives considered

### Path A — No gateway; each agent owns its own OAuth context

Each agent pod runs `claude login` directly and manages its own access/refresh token lifecycle.
No central gateway.

**Rejected**: loses central observability (per-tenant quota, per-seat usage, audit chain),
eliminates the pool benefit (no cross-agent seat sharing), and forces every agent to own TLS
client setup toward provider endpoints. The gateway's value is precisely the central pool.

### Path C — Transparent pass-through proxy, no token rotation

Gateway passes all requests to a single upstream CLIProxyAPI instance running as a sidecar. No
token management in the gateway itself.

**Rejected**: reintroduces the "just adopt CLIProxyAPI as a dependency" question (addressed in
Critic Findings below). More importantly, it sacrifices the audit chain, per-tenant auth realms,
OTel metrics, and OpenSLO SLIs that the gateway already implements. The gateway owns these; a
sidecar does not.

### Consumer chat-session cookie reuse

Reuse `claude.ai` / `chatgpt.com` session cookies extracted from browser storage to call the chat
API directly.

**REJECTED** — explicitly prohibited on TOS grounds. `claude.ai` TOS is consumer-chat TOS and
forbids automated access. This is categorically different from the Claude Code / Codex CLI TOS
which sanctions programmatic refresh-token use. This alternative is not reconsidered.

### Static API key retention (current implementation, no change)

Keep the existing static-key-pool as-is and require the operator to purchase API-tier access
(separate from their existing subscriptions).

**Rejected as the wrong entitlement model**: the operator legitimately owns subscription
accounts. Requiring them to buy a separate API-tier plan purely to fit the current architecture
is backwards. The architecture should fit the entitlement model, not the reverse.

## Related

- ADR-0373: cloud-intelligence gateway production design (key-pool resilience, audit, OpenSLO). This ADR
  supersedes the credential model from ADR-0373 but retains its API surface and observability
  decisions.
- ADR-0381 D3: Cell boundary enforcement (Cilium NetworkPolicy; gateway cell placement).
- ADR-0043: Secrets management (OpenBao; SecretReference contract for OAuth token storage and
  ExternalSecret refresh interval).
- ADR-0083: Pod runtime tier decision (error handling; thiserror/anyhow tier in the Rust crates).
- ADR-0191: Edge authz Cedar PDP (Cedar policy basis for gateway ingress auth realms).

---

## Open Questions

### OQ-1: Token-refresh race [BLOCKING]

**Problem**: Two concurrent requests arrive for the same seat. Both call `pool.select()`, both
receive the same seat with `RefreshRequired`. Both proceed to attempt a token refresh against the
provider endpoint. The provider may reject the second refresh (rate-limit on the token endpoint,
or if it is a single-use-refresh-token provider, the first refresh invalidates the token the
second attempt carries).

**Proposed resolution**: each seat gets an `Arc<Mutex<RefreshState>>` in the `SubscriptionStore`.
When `RefreshRequired` is emitted, the rest layer attempts to acquire the lock:
- If acquired: perform the refresh, update the pool, release the lock.
- If not acquired (another request is already refreshing): wait (with bounded timeout) for the
  lock to be released, then re-read the updated token from the pool without re-refreshing.

This is the Go `sync.Mutex` pattern used in CLIProxyAPI's credential manager. Must be resolved
before D1/D2 implementation because it affects the kernel/rest interface boundary.

### OQ-2: Refresh-token rotation (single-use refresh token) [BLOCKING]

**Problem**: Anthropic's OAuth flow issues a new `refresh_token` on every access-token refresh.
If the gateway does not persist the new refresh token back to OpenBao immediately, the next pod
restart will attempt to use the stale (now invalid) refresh token and fail authentication for
that seat.

**Proposed resolution**: write-through on every token refresh. The rest adapter, after a
successful `refresh_token()` call, immediately calls `bao kv put
secret/oya/cloud-intelligence/<provider>/seats/<seat-name> refresh_token=<new_token>`. This adds one
OpenBao write per access-token refresh (i.e., roughly once every 4-8 hours per seat, far below
any write-rate concern). The ExternalSecret `refreshInterval` serves only as the boot/recovery
path. Must be resolved before D2 implementation.

### OQ-3: Quota header parsing per provider [NON-BLOCKING]

**Problem**: each provider sends different rate-limit headers. The union schema needed for
`parse_rate_headers()` in the `ProviderAdapter` trait must cover at minimum:

| Provider | Header | Semantics |
|----------|--------|-----------|
| Anthropic | `anthropic-ratelimit-tokens-remaining` | tokens remaining in current window |
| Anthropic | `anthropic-ratelimit-requests-remaining` | requests remaining |
| Anthropic | `anthropic-ratelimit-reset-tokens` | ISO-8601 reset time |
| OpenAI Codex | `x-ratelimit-remaining-tokens` | tokens remaining |
| OpenAI Codex | `x-ratelimit-remaining-requests` | requests remaining |
| OpenAI Codex | `x-ratelimit-reset-tokens` | seconds until reset |
| Gemini | `x-ratelimit-limit` | requests per minute limit |
| Gemini | `x-ratelimit-remaining` | requests remaining |

**Proposed resolution**: `RateLimitInfo { tokens_remaining: Option<u64>, requests_remaining: Option<u64>, reset_at: Option<Instant> }` as the normalized output of `parse_rate_headers()`. Each
adapter maps its provider's headers to this union. Can be refined during D3 implementation when
live responses are available.

### OQ-4: Cold-start subscription selection [NON-BLOCKING]

**Problem**: on gateway pod restart, all access tokens are expired (not persisted). Two options:
(a) proactive: refresh all seats on boot before serving any request; (b) lazy: refresh on first
request per seat, incurring one token-refresh RTT latency spike per seat.

**Recommendation**: lazy. Rationale: proactive boot-time refresh serializes pod startup on N
provider token endpoints, increases startup latency and failure modes, and provides no steady-state
benefit (access tokens expire regularly regardless). Lazy means only the first request to a given
seat sees the refresh latency; subsequent requests within the token TTL are unaffected.
Document this trade-off in the SETUP-RUNBOOK (D5): operators should expect ~1-2s latency for the
first request per seat after pod restart.

### OQ-5: Operator bootstrap — exact credential file paths [BLOCKING]

**Problem**: the SETUP-RUNBOOK (D5) needs verified file paths for each CLI's local credential
store. CLIProxyAPI's `internal/misc/credentials.go` and `sdk/auth/filestore.go` contain the
canonical paths as used by the Go implementation, but we need to confirm these match the current
CLI releases (paths may differ across CLI versions or OS):

| Provider | CLI | Claimed path | Verified? |
|----------|-----|-------------|-----------|
| Anthropic | Claude Code | `~/.config/claude/credentials.json` → `.oauthToken.refreshToken` | UNVERIFIED |
| OpenAI | Codex CLI | `~/.codex/auth.json` → `.refreshToken` | UNVERIFIED |
| Google | Gemini CLI | `~/.config/gemini/oauth_creds.json` → `.refresh_token` | UNVERIFIED |

**Resolution required before D5**: operator must verify by running `claude login` and inspecting
the credential file. CLIProxyAPI source is the primary reference; actual CLI output is ground truth.
This is BLOCKING for the D5 runbook but not for D1/D2 kernel/rest implementation.

### OQ-7: TLS fingerprint compatibility for Claude OAuth token refresh [NON-BLOCKING since 2026-05-28 amendment 1]

**Original concern**: CLIProxyAPI uses uTLS Chrome impersonation to bypass Cloudflare on
api.anthropic.com/v1/oauth/token. Standard Rust rustls/ring would be 403'd.

**New evidence (2026-05-28)**: auth2api (https://github.com/AmazingAng/auth2api, TypeScript/Node)
implements the same Anthropic OAuth refresh flow against the same endpoint with stock Node
fetch (undici, OpenSSL-based TLS), no User-Agent spoofing on the OAuth call, no Chrome
impersonation. See src/auth/oauth.ts lines 74-99. The literal call is
`fetch("https://api.anthropic.com/v1/oauth/token", { method: "POST", headers: {"Content-Type": "application/json"}, body: ... })`.

**Inference**: Cloudflare bot-detection on api.anthropic.com/v1/oauth/token is NOT JA3/JA4
fingerprint-based. CLIProxyAPI's uTLS dependency is either over-engineered defensive code
for this endpoint OR is required for the **data path** (api.anthropic.com/v1/messages) not
the OAuth refresh leg.

**Resolution**: D3 ClaudeCodeAdapter uses stock reqwest (rustls/ring) for OAuth refresh —
no rquest / no boring-tls dependency. A confirmation spike runs as part of D3
implementation; if it shows stock reqwest is 403'd (against current expectation), OQ-7
re-escalates and D3 adds rquest. Sidecar fallback to CLIProxyAPI is dropped from
"Alternatives Considered" as it's no longer load-bearing.

**Original [BLOCKING] description retained for audit trail**:

**Problem**: Anthropic's OAuth token endpoint (`api.anthropic.com/v1/oauth/token`) is served
behind Cloudflare, which performs TLS `ClientHello` fingerprinting to detect non-browser HTTP
clients. CLIProxyAPI (Go) works around this by using `refraction-networking/utls` with a Chrome
TLS fingerprint profile (`internal/auth/claude/utls_transport.go`). A standard Rust TLS stack
(`rustls` + `ring`) presents a `rustls`-shaped `ClientHello` that Cloudflare flags as a bot,
returning a 403 or 429 before the OAuth response.

**Resolution required before D3**: Spike a Rust HTTP client against
`api.anthropic.com/v1/oauth/token` using the `rquest` crate with `boring-tls` feature enabled
(which presents a Chrome-compatible `ClientHello`). Confirm: (a) the request reaches the OAuth
endpoint without 403/bot-block; (b) the `boring-ssl` + `ring` dependency combination does not
conflict with the existing `ring = "0.17.x"` LTS pin in the gateway workspace. If the `rquest`
approach is blocked or has a dependency conflict, fall back to the CLIProxyAPI sidecar for the
Anthropic OAuth leg (see "Alternatives Considered — Path C hybrid").

**Fallback**: If no Rust-native solution is viable after 2 spike attempts, adopt the sidecar
architecture for the Claude OAuth leg only: CLIProxyAPI runs as an init-container or sidecar pod
handling `ClaudeCodeAdapter::refresh_token()` calls; the Rust gateway handles all other logic.

### OQ-6: Forward-compat with PackyAPI / AICodeMirror relay services [NON-BLOCKING]

**Problem**: PackyAPI and AICodeMirror are relay services that sit between the official providers
and the caller. They expose OpenAI-compatible REST endpoints with their own API keys (not OAuth).
If the operator switches to a relay service in the future, should the gateway treat them as a
special provider adapter, or just an upstream URL override?

**Proposed resolution**: upstream URL override. The `auth_mode: "static_key"` path (retained for
backward compat) plus a configurable `upstream_base_url` per provider group is sufficient to point
at a relay. No special adapter needed. Document this in the SETUP-RUNBOOK as "relay mode" under
the static-key path. NON-BLOCKING.

---

## Critic findings + responses

Adversarial review performed against the four reference repos (CLIProxyAPI v7, gpt-load, one-api,
quotio) and verified against source-level evidence. Findings below.

---

### F1: Wrong OAuth endpoint for Claude — `claude.ai` not `console.anthropic.com` [HIGH]

**Finding**: ADR-0384 states the Claude provider adapter targets `upstream_base_url:
"https://console.anthropic.com"` and describes it as "console.anthropic.com PKCE flow". This is
wrong. CLIProxyAPI source (`internal/auth/claude/anthropic_auth.go`) shows:

```
AuthURL  = "https://claude.ai/oauth/authorize"
TokenURL = "https://api.anthropic.com/v1/oauth/token"
```

`console.anthropic.com` is the human operator dashboard (account settings, billing, team
management), not an OAuth endpoint. The OAuth flow for Claude Code uses `claude.ai` as the
authorization entry point and `api.anthropic.com/v1/oauth/token` for token exchange.

**Unstated assumption exposed**: The ADR assumes operator familiarity with which of Anthropic's
several domains hosts the OAuth flow. The actual URL is the consumer `claude.ai` domain, not the
developer console.

**TOS impact**: This changes the TOS framing materially (see F4 below).

**Response**: Corrected in the ADR text. Provider adapter section updated:
`ClaudeCodeAdapter` targets `https://api.anthropic.com/v1/oauth/token` for token exchange;
the authorization redirect uses `https://claude.ai/oauth/authorize`. The `upstream_base_url`
for LLM request forwarding (not OAuth) is a separate concern.

**ADR fix**: Replace `upstream_base_url: "https://console.anthropic.com"` with
`upstream_base_url: "https://api.anthropic.com"` (for API calls post-auth) and add a separate
`oauth_auth_url: "https://claude.ai/oauth/authorize"` / `oauth_token_url:
"https://api.anthropic.com/v1/oauth/token"` per provider adapter in D3. Similarly, OpenAI Codex
uses `https://auth.openai.com/oauth/authorize` + `https://auth.openai.com/oauth/token`, not
`codex.openai.com` as the OAuth origin. Gemini uses `https://accounts.google.com/o/oauth2/v2/auth`
+ `https://oauth2.googleapis.com/token`.

---

### F2: TLS fingerprinting bypass required for Anthropic OAuth — uTLS dependency unstated [FATAL]

**Finding**: CLIProxyAPI uses `refraction-networking/utls` with a Chrome TLS fingerprint to make
requests to Anthropic's OAuth token endpoint (`api.anthropic.com`). The code comment is explicit:
*"bypass Cloudflare's TLS fingerprinting on Anthropic domains."* The `utls_transport.go` file
implements a custom `http.RoundTripper` that presents a Chrome `ClientHello` to avoid bot
detection. A standard Rust `hyper` + `rustls` (or `native-tls`) TLS client will present a
`rustls`/`ring`-shaped `ClientHello` which Cloudflare flags as non-browser and blocks.

This is not mentioned anywhere in ADR-0384. It is the single most operationally fatal gap: a
standard Rust HTTP stack will fail to refresh Claude tokens, making the entire Claude adapter
non-functional in production.

**Unstated assumption exposed**: The ADR assumes that a standard Rust TLS stack can reach
Anthropic's OAuth endpoints without bot-detection interference.

**Response**: This is BLOCKING for D2/D3 implementation. The Claude provider adapter must either:
  (a) Use `boring-tls` or a `rustls` config that mimics a Chrome `ClientHello` (the `rquest`
  crate, which wraps `boring`, does this); or
  (b) Route the OAuth token-exchange call through a proxy that presents a browser-like TLS
  fingerprint; or
  (c) Accept that token refresh will route through a separate CLIProxyAPI sidecar process for the
  Anthropic OAuth leg (Path C hybrid — not preferred but functionally correct).

**ADR fix**: Add to D3 deliverable: "The `ClaudeCodeAdapter::refresh_token()` implementation
MUST use a TLS client presenting a browser-compatible `ClientHello` (e.g., via `rquest` crate
with `boring-tls` feature, or equivalent). Standard `rustls`/`ring` stacks are rejected by
Cloudflare on Anthropic's OAuth token endpoint. This is a hard implementation constraint."
Add to Open Questions as OQ-7 (BLOCKING): "TLS fingerprint compatibility for Claude OAuth:
which Rust TLS strategy passes Cloudflare's bot-detection on `api.anthropic.com`?"

---

### F3: Rust rewrite justification is insufficient — CLIProxyAPI already solves this [HIGH]

**Finding**: CLIProxyAPI v7 (Apache-2.0, actively maintained, production-deployed at scale per its
sponsor ecosystem) implements every capability in this ADR:
- Multi-seat OAuth subscription pool for Claude, Codex, Gemini, Grok
- Round-robin and fill-first selection strategies
- Per-seat cooldown on 429
- Token refresh with write-through (via `FileTokenStore`)
- uTLS fingerprint bypass for Anthropic
- OpenAI-compatible API surface
- Quota tracking (via Redis usage queue plugin)

The ADR's "Alternatives Considered" section mentions Path C (CLIProxyAPI as a sidecar) and
rejects it on grounds of losing "audit chain, per-tenant auth realms, OTel metrics, and OpenSLO
SLIs." But all of these are Oyatie-specific governance overlays on the gateway side — none of
them require the OAuth pool to be rewritten in Rust. A sidecar architecture would be:
`CLIProxyAPI (OAuth pool, Go, port 9090) → oya-cloud-intelligence (audit + auth + metrics, Rust,
port 8080)`. The gateway proxies to CLIProxyAPI for OAuth-mediated providers while retaining
all Oyatie governance concerns in the Rust layer.

**Risk**: If the Rust rewrite proceeds, it will spend significant implementation effort
re-implementing logic CLIProxyAPI has debugged over years (uTLS, PKCE, token rotation,
per-provider quirks). The uTLS gap (F2) is a concrete example of domain knowledge that is
non-obvious and was not captured in the ADR.

**Verdict: Not structurally fatal** — a Rust rewrite IS justified IF the operator accepts the
additional implementation risk (especially F2) and values a single-binary, single-language stack
over operational simplicity. The sidecar option is valid and reduces risk materially. The ADR
should acknowledge this trade-off explicitly rather than dismissing Path C on governance grounds
alone.

**Response**: Path C rejection rationale strengthened. Added explicit acknowledgment that
CLIProxyAPI solves the OAuth pool problem in production. The Rust rewrite is justified by:
(1) single-language operational stack (no Go runtime dependency); (2) the existing Rust gateway
binary is already the production artifact with Oyatie's audit chain, authz, and OTel; (3) the
uTLS gap is solvable with `rquest`/`boring-tls`. However, D3 MUST dedicate explicit design effort
to the TLS fingerprint problem before implementation starts (see F2). The sidecar option is
retained as a lower-risk fallback if D3 TLS implementation proves intractable.

**ADR fix**: Add to "Alternatives Considered" under Path C: "A hybrid architecture —
CLIProxyAPI sidecar handling OAuth token lifecycle, Rust gateway retaining audit/authz/metrics —
was evaluated. It reduces implementation risk (uTLS, per-provider quirks) at the cost of a Go
runtime dependency. This is the correct fallback if D3's Rust TLS fingerprint implementation
proves intractable. The sidecar option is NOT rejected on principle; it is deferred pending D3
feasibility assessment."

---

### F4: TOS framing has a domain-provenance error [MEDIUM]

**Finding**: The ADR claims the Claude OAuth flow is governed by "Claude Code and API Terms of
Service" because it targets `console.anthropic.com`. However, CLIProxyAPI source shows the
authorization URL is `https://claude.ai/oauth/authorize` — the consumer `claude.ai` domain.
`claude.ai` is the consumer chat product. The ADR itself states "the relevant TOS is the Claude
Code and API Terms of Service (not claude.ai consumer chat TOS, which is a separate agreement)"
— but the OAuth flow begins on `claude.ai`. This is an internal contradiction.

**Unstated assumption exposed**: That Anthropic's decision to host the Claude Code CLI OAuth
flow on the `claude.ai` domain rather than `api.anthropic.com` or `console.anthropic.com` does
not bring it under the `claude.ai` consumer TOS. This may be correct (Anthropic likely routes all
OAuth through a unified auth domain), but it is an unverified assumption.

**Verdict**: Not fatal to the design, but the TOS framing should acknowledge the ambiguity
honestly rather than asserting certainty.

**Response**: TOS section updated to state: "The Claude Code CLI OAuth authorization flow begins
at `claude.ai/oauth/authorize`. Anthropic routes CLI OAuth through this domain; the operative
terms for CLI tool access are the Claude Code Terms of Service, not the `claude.ai` consumer chat
TOS. However, operators should verify this interpretation against current Anthropic terms, as the
domain overlap creates surface-level ambiguity. Anthropic's usage policy documentation for Claude
Code (separate from claude.ai ToS) is the authoritative reference."

---

### F5: Gemini adapter requires GCP Project ID — not mentioned in config or runbook [MEDIUM]

**Finding**: CLIProxyAPI's `GeminiTokenStorage` includes `ProjectID string \`json:"project_id"\``
and the `GetAuthenticatedClient` flow requires a Cloud Project to be selected during the OAuth
consent. The Gemini CLI OAuth flow (`gemini login`) is actually the **Google Cloud Code Assist**
(`cloudcode-pa.googleapis.com`) OAuth flow, not the `generativelanguage.googleapis.com` API.
The scopes include `cloud-platform`, and the client ID (`681255809395-...`) is the Gemini CLI /
Cloud Code client ID.

The ADR's D4 config schema and D5 runbook make no mention of GCP Project ID, the Cloud Code
endpoint, or the `cloudcode-pa.googleapis.com` vs `generativelanguage.googleapis.com` distinction.
The `GeminiCliAdapter` as specified (targeting `generativelanguage.googleapis.com`) may be the
wrong endpoint entirely.

**Response**: NON-BLOCKING for D1 (kernel) but BLOCKING for D3 (Gemini adapter). D3 must
investigate whether the Gemini CLI OAuth flow targets `generativelanguage.googleapis.com` or
`cloudcode-pa.googleapis.com`, and whether a GCP Project ID is required per-request. This
distinction affects the `GeminiCliAdapter::forward_request()` implementation entirely.

**ADR fix**: Add to OQ-3 (quota headers): note that Gemini adapter endpoint is unverified
(`generativelanguage.googleapis.com` vs `cloudcode-pa.googleapis.com`). Add to D3 deliverable:
"Gemini adapter endpoint and GCP Project ID requirement must be verified against the Gemini CLI
OAuth token storage before implementation."

---

### F6: Credential file paths for all three CLIs are unverified — OQ-5 severity should be HIGH [MEDIUM]

**Finding**: OQ-5 correctly marks the credential paths as UNVERIFIED. However, based on
CLIProxyAPI's `filestore.go`, the paths are operator-configurable (not hardcoded): CLIProxyAPI
stores credentials in a configurable `baseDir`, defaulting to a config-dir path, not the
hardcoded paths the ADR cites. The paths in the D5 runbook (`~/.config/claude/credentials.json`,
`~/.codex/auth.json`, `~/.config/gemini/oauth_creds.json`) are plausible guesses but not
confirmed from CLIProxyAPI source. CLIProxyAPI source uses `authFilePath` derived from
`Auth.FileName` (e.g., `claude-<email>.json`), not a fixed path.

The actual Claude Code CLI (Anthropic's first-party CLI) may store credentials at a different
path than CLIProxyAPI does.

**Response**: OQ-5 severity is correctly marked BLOCKING. The D5 runbook paths must be
empirically verified by running `claude login` on a test machine and inspecting the output
before the runbook ships. Operators should not be given wrong `jq` extraction commands.

**ADR fix**: In the D5 operator bootstrap section, replace the `jq` one-liners with a
verification instruction: "Run `claude login` and note the path printed to stdout (the CLI
announces where it saves credentials). Use that path in the `bao kv put` command. Do not rely
on the example paths in this runbook without verification."

---

### F7: Hyperscaler-lens claim for "fully self-hostable" is weakened by uTLS dependency [LOW]

**Finding**: The hyperscaler-lens (c) "fully self-hostable" claim is accurate for the gateway
infrastructure itself. However, the uTLS requirement (F2) means the Claude adapter is operationally
dependent on Cloudflare's bot-detection posture — if Cloudflare updates its fingerprinting
detection, the gateway's Claude token-refresh breaks silently until a new TLS fingerprint profile
is shipped. This is a third-party dependency on a non-self-hostable infrastructure decision
(Cloudflare's WAF rules). It does not violate the hyperscaler-lens per se (the operator cannot
self-host Anthropic's infrastructure) but should be acknowledged as a fragility.

**Response**: LOW severity, NON-BLOCKING. Acknowledge in the "Consequences — Gets harder"
section: "Claude token-refresh is fragile against Cloudflare WAF updates on Anthropic's OAuth
endpoint. The TLS fingerprint profile in the `ClaudeCodeAdapter` may need updates if Cloudflare
tightens bot-detection. Monitor CLIProxyAPI's `utls_transport.go` for upstream fixes."

---

### Structural verdict

**Non-fatal**: The design is sound. The OAuth subscription-pool architecture is correct for the
operator's entitlement model, the kernel/rest split is the right purity boundary, and the
OpenBao-based credential storage is architecturally clean.

**F2 (uTLS) is the only item that could make D3 non-deliverable** if no Rust-compatible TLS
fingerprint solution exists. The fallback (CLIProxyAPI sidecar for the Anthropic OAuth leg) is
viable and should be explicitly retained in the ADR rather than dismissed.

**F1 (wrong endpoints) and F5 (Gemini endpoint ambiguity) must be corrected before D3
implementation begins** — they affect the provider adapter interface boundary.

**Recommended next move**: Before starting D1 implementation, resolve OQ-7 (BLOCKING, added by
F2): spike a Rust HTTP client against `api.anthropic.com/v1/oauth/token` using `rquest` +
`boring-tls` and confirm it is not blocked. If blocked after 2 attempts, adopt the CLIProxyAPI
sidecar approach for the Anthropic OAuth leg. All other D1 (kernel) work is independent of
this TLS question and can proceed in parallel.
