# intelligence-app — agent-dispatch gateway

A clean-room Rust **LLM key-pool reverse-proxy gateway**. It multiplexes the
Oyatie AI-agent fleet over pooled Codex / Claude / OpenAI / Gemini API keys,
rotating keys round-robin, blacklisting failing keys with a jittered cooldown,
and streaming provider responses (including SSE) straight through.

This is an original reimplementation of the *concept* of an LLM key-pool
reverse proxy. **No third-party source was read or copied.**

## Crates (ADR-0131 flat microservice, ADR-0105 layers)

| Crate | Layer | Responsibility |
|-------|-------|----------------|
| [`intelligence-app-kernel`](../../crates/intelligence-app-kernel) | `kernel` | **Pure** key-pool state machine. No I/O, no async, no clock, no RNG, no external crate. Round-robin selection (`AtomicUsize`), per-key `failure_count` + blacklist threshold, jittered cooldown timestamps, success-reset / lazy restore-to-active, the `ProviderChannel` enum (OpenAI / Anthropic / Gemini). Time and jitter are injected so every transition is deterministic and unit-tested. |
| [`intelligence-app-rest`](../../crates/intelligence-app-rest) | `rest` | The axum reverse-proxy app + binary. SSE streaming passthrough, per-provider channel adapters, owned secret-provider/KMS handles, failover/retry, two constant-time auth realms, Prometheus `/metrics`, hash-only logging. |

## v1 feature checklist

- **True SSE streaming passthrough** — the upstream `reqwest` byte stream is
  piped directly into the axum response body (`Body::from_stream`). Response
  bodies are **never** buffered, parsed, or logged. Works for both streamed and
  unary responses.
- **Per-provider channel adapters** — inject the correct auth + base URL:
  - OpenAI: `Authorization: Bearer <key>`
  - Anthropic: `x-api-key: <key>` + `anthropic-version: <ver>`
  - Gemini: `X-Goog-Api-Key: <key>`
- **Key pool** — round-robin selection, per-key failure counting, blacklist on
  a configurable threshold, jittered cooldown, success-reset, lazy restore.
- **Failover / retry** — on a configurable status set (default `429, 500, 502,
  503, 504`) the proxy rotates to the next key, with a max-attempt cap and
  jittered backoff. Transport errors are always retryable.
- **Two auth realms** — admin/control and ingress proxy-key, both compared in
  constant time via [`subtle`].
- **Owned secret-provider sourcing** — pooled credentials are resolved from
  `secret-ref://` / `kms-ref://` handles at startup and on periodic refresh.
  **No pooled key is ever read from a plaintext file or environment variable.**
  Concrete stores are transient adapters behind cloud-secrets/cloud-kms.
- **Prometheus metrics** — `intelligence_app_*`: per-key success/failure,
  retries, upstream latency histogram, active-key gauge, request outcomes.
- **Hash-only logging** — structured logs identify a key only by a
  non-reversible SHA-256-derived fingerprint. The key, the prompt, and the
  response body are never logged.

## Key sourcing (load-bearing security)

```
secret-ref://intelligence-app/<tenant>/<provider>/<seat>
kms-ref://intelligence-app/<tenant>/<provider>/<seat>
```

The gateway receives opaque handles and a short-lived secret-provider adapter
token from Kubernetes projection. The cloud-secrets/cloud-kms substrate owns
encryption-at-rest, KMS policy, and any backing-store adapters. There is no
plaintext file/env key source.

## Configuration

Non-secret routing config is declarative (a ConfigMap-mounted JSON file at
`$GATEWAY_CONFIG`). Secrets are sourced only from owned secret-provider/KMS
handles projected by Kubernetes.

```json
{
  "listen_addr": "0.0.0.0:8080",
  "secret_provider": { "address": "https://cloud-secrets-adapter.cloud-secrets.svc.cluster.local:8200", "handle_schemes": ["secret-ref://", "kms-ref://"] },
  "key_refresh_secs": 300,
  "groups": [
    {
      "name": "codex",
      "channel": "openai",
      "upstream_base_url": "https://api.openai.com",
      "secret_handle": "secret-ref://intelligence-app/dogfood/openai/default",
      "retry": { "retry_on_statuses": [429, 500, 502, 503, 504], "max_attempts": 3 }
    },
    {
      "name": "claude",
      "channel": "anthropic",
      "upstream_base_url": "https://api.anthropic.com",
      "secret_handle": "secret-ref://intelligence-app/dogfood/anthropic/default",
      "anthropic_version": "2023-06-01"
    }
  ]
}
```

Environment (all injected from k8s Secrets at deploy; never plaintext files):

| Var | Purpose |
|-----|---------|
| `GATEWAY_CONFIG` | Path to the ConfigMap JSON above (non-secret). |
| `OYATIE_CLOUD_INTEL_SECRET_PROVIDER_TOKEN` | Short-lived token for the owned secret-provider adapter. |
| `ADMIN_TOKEN` | Admin/control realm token. |
| `INGRESS_PROXY_KEYS` | Comma-separated ingress proxy-keys for the agent fleet. |

## Ingress surface

| Route | Purpose |
|-------|---------|
| `ANY /proxy/{group}/{*rest}` | Reverse proxy. Caller presents `x-proxy-key`; the gateway forwards to `<upstream_base_url>/<rest>` with pooled auth injected. |
| `GET /healthz` | Liveness. |
| `GET /metrics` | Prometheus exposition. |

## Live fanout: how parallel agents use this gateway

After the operator-runtime steps in [SETUP-RUNBOOK.md](./SETUP-RUNBOOK.md) are complete (secret-provider handles registered, image built, Deployment Ready, HPA active), parallel agents (Claude Code / Codex / Anthropic SDK / OpenAI SDK / Gemini SDK callers) can route through the gateway instead of each agent calling provider APIs directly.

### Agent-side env vars

Each agent sets these BEFORE invoking its provider SDK:

```sh
# Anthropic SDK (Claude Code, claude-cli)
export ANTHROPIC_BASE_URL="http://intelligence-app.intelligence-app.svc.cluster.local:8080/v1/anthropic"
export ANTHROPIC_API_KEY="$INGRESS_PROXY_KEY"   # one of the ingress proxy keys from secret-ref://intelligence-app/ingress-proxy-keys

# OpenAI SDK (Codex, openai-cli)
export OPENAI_BASE_URL="http://intelligence-app.intelligence-app.svc.cluster.local:8080/v1/openai"
export OPENAI_API_KEY="$INGRESS_PROXY_KEY"

# Gemini SDK
export GEMINI_BASE_URL="http://intelligence-app.intelligence-app.svc.cluster.local:8080/v1/gemini"
export GEMINI_API_KEY="$INGRESS_PROXY_KEY"
```

### Namespace opt-in (cell-boundary)

The Cilium L3/L4 NetworkPolicy at `infra/cilium/cell-boundaries/intelligence-app-ingress.netpol.yaml` only allows traffic from namespaces labelled `oyatie.gateway-client=true`. For each agent-hosting namespace:

```sh
kubectl label namespace <my-agent-ns> oyatie.gateway-client=true
```

### What the gateway does for the agent

1. Strips the agent's `Authorization` header.
2. Looks up the next available key from the per-provider key pool (round-robin with cooldown on 429/5xx per ADR-0193 + ADR-0381 D1).
3. Re-signs the upstream request with the provider's expected header (`Authorization: Bearer ...` for OpenAI, `x-api-key + anthropic-version` for Anthropic, `X-Goog-Api-Key` for Gemini).
4. Forwards SSE / streaming responses back to the agent unchanged.

### Horizontal scaling

- Deployment baseline: `replicas: 3`.
- HPA: 3 → 20 replicas, target 60% CPU + 75% memory utilization.
- Each replica is stateless; key-pool state is per-replica (no cross-replica coordination — the per-provider key set is small enough that independent round-robin tolerates skew).
- For N concurrent agents, the gateway scales horizontally without coordination overhead.

### Observability

Once observability lands (PR #260 + ADR-0383), the gateway emits OTel metrics on:
- request rate per provider + per group
- key-pool occupancy / blacklist count / cooldown remaining
- upstream latency (p50/p95/p99 per provider)

## Status / non-claims

`CS-CLOUD-INTELLIGENCE-AGENT-DISPATCH-001` is a **code-backed local foundation**: the
workspace builds, clippy is clean, and unit tests pass. There is **no** live
deployment, container image, k8s manifest, measured SLO, persistence, or
audit-chain runtime. See [`manifest.json`](./manifest.json) for the full
machine-readable claim set and explicit non-claims.
