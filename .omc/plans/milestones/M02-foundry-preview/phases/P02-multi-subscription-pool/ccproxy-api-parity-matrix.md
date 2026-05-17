---
doc_class: ParityMatrix
parent: ./INDEX.md
milestone: M02
phase: P02-multi-subscription-pool
status: in-progress
purpose: |
  Feature-by-feature comparison between ccproxy-api (https://github.com/CaddyGlow/ccproxy-api)
  and oyatie's P02 design. Each ccproxy-api feature is mapped to oyatie coverage with a
  status column (full | partial | gap | rejected) and the IP that closes each gap. Built
  from upstream research 2026-05-12 (ccproxy-api v0.2.10, May 2 2026 release).
length_cap: 120
canonical_source: https://github.com/CaddyGlow/ccproxy-api
---

# ccproxy-api ↔ oyatie P02 parity matrix

## A. Provider passthrough endpoints

| ccproxy-api feature | Module | oyatie status | Closing IP / location |
|---|---|---|---|
| `POST /v1/messages` (Anthropic shape) | `plugins/claude_api` | **full** | IP-002 (`messages_handler`) |
| `GET /v1/messages/count_tokens` | `plugins/claude_api` | **full** | IP-002 (`count_tokens_handler`) |
| `POST /v1/chat/completions` (OpenAI shape) | `plugins/codex` | **full** | IP-003 (`chat_completions_handler`) |
| `POST /v1/responses` (OpenAI Responses API) | `plugins/codex` | **partial** | IP-003 carries OpenAPI hook; first cut delivers Chat Completions; Responses API lands in P03 follow-up if customer-demand row is filed |
| `POST /v1/embeddings` | implicit through codex | **full** | IP-003 (`embeddings_handler`) |
| `GET /v1/models` | implicit | **full** | IP-003 (`models_handler`) translates internal capability registry |
| Anthropic ↔ OpenAI cross-shape translation | shared model-mapping layer | **full** | IP-002 + IP-003 share the internal `CapabilityInvokeRequest` shape; translators are bidirectional at the boundary |
| GitHub Copilot adapter (`plugins/copilot`) | dedicated | **rejected** | Not in P02 scope; Copilot is consumer of OAuth subscription, not a foundational provider for oyatie (out of MASTERPLAN Directive 4 first-cell coverage); revisit in M06 if customer evidence warrants |
| Claude SDK local adapter (`plugins/claude_sdk`) | local CLI handoff | **rejected** | oyatie does not embed Anthropic's Python SDK; the in-tree Rust HTTP client (per `.omc/standards/dependency-policy.md §5.1`) is the canonical path; the runtime-local-CLI handoff is the imported-session path in P00 (`ImportRegularSession`) |

## B. Multi-subscription / credential pool

| ccproxy-api feature | Module | oyatie status | Closing IP / location |
|---|---|---|---|
| Pool of credentials per provider | `plugins/credential_balancer` | **full** | IP-001 (`ProviderAccountPool`) |
| Round-robin rotation | `manager.py` strategy | **full** | IP-001 `PoolRoutingStrategy::RoundRobin` |
| Failover rotation (healthy-first) | `manager.py` strategy | **full** | IP-001 `PoolRoutingReason::FailoverFrom`; fallback chain on every decision |
| Health tracking via response codes | `hook.py` lifecycle hook | **full** | IP-001 reads `AccountHealthMap` (built by P00 health probes) + `UsageSnapshotMap` |
| Configurable failure status codes | `config.py` | **partial** | oyatie reuses P00 health probe taxonomy; explicit per-pool override surfaces in P03 |
| Cooldown periods per credential | `cooldown_seconds` | **full** | P00 state machine `Degraded` state + `recover()` transition |
| Max consecutive failures before disable | `max_failures_before_disable` | **full** | P00 state machine `disable(reason)` transition |
| Least-used / least-latency rotation | not in ccproxy-api | **full (net-new)** | IP-001 `PoolRoutingStrategy::LeastUsed` + `LeastLatency` + `LeastRemaining` |
| Sticky-session rotation | not in ccproxy-api | **full (net-new)** | IP-001 `PoolRoutingStrategy::Sticky(SessionId)` |

## C. Subscription token capture

| ccproxy-api feature | Module | oyatie status | Closing IP / location |
|---|---|---|---|
| Claude.ai OAuth PKCE flow | `plugins/oauth_claude` | **full** | IP-004 (`capture_subscription_token`, `SubscriptionOAuthFlow`) |
| Loopback redirect on port 35593 | `oauth_claude/provider.py` | **full** | IP-004 (`OAuthLoopbackServer` default port 35593) |
| Scopes `org:create_api_key`, `user:profile`, `user:inference` | `oauth_claude` config | **full** | IP-004 default scope set |
| OpenAI / Codex OAuth flow (`oauth_codex`) | `plugins/oauth_codex` | **partial** | IP-004 carries the `FlowKind::OpenAiOAuth` variant; upstream OAuth availability gates the smoke test (when OpenAI exposes the endpoint for the target subscription) |
| GitHub Copilot device-code flow | `plugins/copilot` | **rejected** | Copilot adapter rejected from P02 scope (see row A above) |
| Auto-pickup of `~/.claude/` CLI tokens | `claude_shared` | **gap — deferred** | Closing IP: file an enhancement under P03 if customer-demand evidence row exists; default posture is explicit operator-driven capture for audit traceability |
| Token storage location | filesystem ~/.config | **full + hardened** | IP-004 stores `SecretReference` via OpenBao (ADR-0043); never on filesystem outside OpenBao |
| Token refresh | `provider.py` refresh helper | **full** | IP-004 + P00 `RotateSecretReference` command |

## D. Observability + analytics

| ccproxy-api feature | Module | oyatie status | Closing IP / location |
|---|---|---|---|
| Structured access logging | `plugins/access_log` | **full** | OTel emission per `.omc/standards/observability.md` (inherited by IP-002 / IP-003) |
| Request/response tracing | `plugins/request_tracer` | **full** | OTel spans + audit chain |
| Prometheus metrics endpoint | `plugins/metrics` | **full** | `oya-platform-metrics-prom` already shipped in M01; IP-002 / IP-003 emit `compat_adapter_*` series |
| Pushgateway support | `plugins/metrics` | **partial** | Pushgateway is anti-pattern for long-lived services; oyatie uses pull; if a batch use case appears, scope an addendum |
| DuckDB analytics store | `plugins/duckdb_storage` + `plugins/analytics` | **partial** | oyatie persists usage + routing into the audit chain + ClickHouse 26.3 LTS (per `.omc/standards/dependency-policy.md §7`); DuckDB is rejected — ClickHouse covers OLAP at higher scale |
| Command replay (curl/xh generator) | `plugins/command_replay` | **gap — deferred** | Developer-convenience; revisit in P03 if reviewer-agents need it for debugging |
| Interactive permissions / MCP approval | `plugins/permissions` | **full** | Autonomy-ceiling enforcement (Cedar + runtime) per `.omc/standards/autonomy-ceiling.md` already in M01 |
| Pricing cache | `plugins/pricing` | **full** | P00 `Pricing` value type + `TokenCost` already in foundry-salvage spec |
| Token-limit normalization (`max_tokens` plugin) | `plugins/max_tokens` | **full** | IP-002 + IP-003 translators clamp to model spec at the egress boundary |

## E. Configuration + deployment

| ccproxy-api feature | Module | oyatie status | Closing IP / location |
|---|---|---|---|
| TOML configuration | `pyproject.toml` + per-plugin TOML | **rejected** | oyatie uses typed-config-via-cedar + Kubernetes ConfigMap + OpenBao; TOML is not the surface |
| `enabled_plugins` / `disabled_plugins` lists | global plugin gating | **full** | Cedar policy at runtime + Helm values at deploy time |
| Docker / docker-compose | `Dockerfile` + `docker-compose.yml` | **full** | Distroless-debian13 per Directive 5; Helm chart per M01-P13 |
| Systemd service templates | `systemd/` | **rejected** | Kubernetes-only deployment; per `.omc/standards/image-discipline.md` |
| Nix flake | `nix/` | **rejected** | Not a Nix shop; Bazel/Buck2 also rejected per `.omc/standards/dependency-policy.md §8` |
| `uvx` / `pipx` install | Python entry points | **rejected** | Distroless container deployment; no host-Python install path |

## F. Compliance / drift / ToS

| ccproxy-api feature | Module | oyatie status | Closing IP / location |
|---|---|---|---|
| Explicit ToS-acknowledgment ledger | n/a in ccproxy-api | **full (net-new)** | IP-006 (`ToSAcknowledgment`, `PoolingPolicyCheck`) |
| Upstream-OpenAPI drift detection | n/a in ccproxy-api | **full (net-new)** | IP-005 (`detect_drift`, nightly lane) |
| Audit-chain emission per routing decision | n/a in ccproxy-api | **full (net-new)** | IP-006 (`emit_pool_routing_event`) |
| Anti-correlation rules | n/a in ccproxy-api | **full (net-new)** | IP-006 (`AntiCorrelationRule` enum) |
| Per-tenant pool policy | n/a in ccproxy-api | **full (net-new)** | IP-006 (`TenantPoolingPolicy`) |

## G. Citations

- ccproxy-api canonical repo: https://github.com/CaddyGlow/ccproxy-api (MIT, v0.2.10 2026-05-02).
- ccproxy-api docs: https://caddyglow.github.io/ccproxy-api/.
- PyPI: https://pypi.org/project/ccproxy-api/.
- `credential_balancer` plugin: https://github.com/CaddyGlow/ccproxy-api/tree/main/ccproxy/plugins/credential_balancer.
- `oauth_claude` plugin README: https://github.com/CaddyGlow/ccproxy-api/blob/main/ccproxy/plugins/oauth_claude/README.md.
- `claude_api` plugin: https://github.com/CaddyGlow/ccproxy-api/tree/main/ccproxy/plugins/claude_api.
- `codex` plugin: https://github.com/CaddyGlow/ccproxy-api/tree/main/ccproxy/plugins/codex.
- Anthropic Messages API spec: https://docs.anthropic.com/en/api/messages.
- OpenAI Chat Completions API spec: https://platform.openai.com/docs/api-reference/chat.

## H. Summary

- **Full coverage:** 22 ccproxy-api capabilities reproduced 1-for-1 in typed Rust.
- **Partial coverage / deferred:** 5 (Responses API, configurable per-pool status codes, OpenAI OAuth depends on upstream availability, Pushgateway, DuckDB analytics — ClickHouse covers OLAP).
- **Net-new vs ccproxy-api:** 9 (least-used / least-latency / least-remaining / sticky-session rotation; ToS-ack ledger; upstream-API drift lane; audit-chain routing emission; anti-correlation rules; per-tenant pool policy).
- **Rejected:** 7 (Copilot adapter, claude_sdk local adapter, TOML config, systemd, Nix, uvx/pipx, host-Python install, DuckDB plugin) — substituted by the oyatie posture (typed config / Kubernetes-distroless / ClickHouse).
