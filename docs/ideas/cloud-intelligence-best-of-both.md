# Agent-dispatch cloud-intelligence gateway — best-of-both design (gpt-load + one-api)

Clean-room Rust gateway that multiplexes the agent fleet over pooled Codex/Claude/OpenAI
subscriptions. Two security-reviewed references (2026-05-26), reimplemented — zero vendored code.

## Reviews (trust gate passed)
- **gpt-load** (actively maintained): **CLEAN** — no phone-home/backdoor/exfil; AES-GCM at-rest;
  constant-time auth; SSE byte-passthrough; no response tampering. = the **clean core architecture**.
- **one-api** (unmaintained, but reviewed): **MINOR-CONCERNS, trustworthy as a code reference** — no
  telemetry/backdoor/command-injection; clean supply chain. Operational gaps we DON'T copy (see Avoid).
  = the **mature feature breadth**.

## Core (from gpt-load) — v1 (building now)
Hyper/reqwest reverse proxy; per-provider channel adapters (OpenAI `Bearer`, Anthropic
`x-api-key`+version, Gemini `X-Goog-Api-Key`); key-pool round-robin (AtomicUsize) + failure-count
blacklist + cooldown + success-restore; failover/retry on 429/5xx + max-retry; **true SSE streaming
passthrough** (never buffer/parse/log bodies); two auth realms (admin vs proxy) constant-time
(`subtle`); Prometheus metrics; **key hashes only** in logs, body-logging off; **keys sourced from
OpenBao** (`secret/agent-gateway/<provider>`, `BAO_TOKEN` from k8s Secret) — never plaintext file/env.

## Best-of-both additions (from one-api, reviewed) — v1.5 integration, dispatch-critical first
- **(E) Per-AGENT-TOKEN quota — reserve-then-reconcile** [M, FOUNDATIONAL]. Estimate `prompt+max_tokens`,
  reject if over remaining, reconcile `actual−reserved` after. Re-keyed on **agent/token identity, NOT
  IP** (one-api's per-IP limiter is the wrong granularity for a NAT'd fleet — explicit fix). Skip the
  reserve when headroom is ample (their hot-path-write-avoidance trick). *Stops one agent draining the
  shared subscription pool.*
- **(D) Usage / token / latency attribution** [M]. Per-request structured record keyed by agent identity
  + channel + model + prompt/completion tokens + elapsed → our OTel/observability pipe (feeds DORA +
  per-agent accounting + the self-governing loop). Streaming needs local tiktoken-style counting.
- **(A) Two-tier priority + random-within-tier selection** [S]. `priority` + `weight` per key-group;
  pick top-priority bucket, random within it, spill to lower tier on exhaustion. Round-robin stays
  *inside* a tier.
- **(B) Structured retry + circuit-breaker auto-disable + background re-test** [M]. Retry on a *different*
  key-group; auto-disable a key on auth/quota/permission errors classified by **structured status +
  provider error-type (NOT substring matching** — their classifier is brittle); periodic health-probe
  re-enables. Per-key circuit-breaker state.
- **(C) Model-name mapping / aliasing per channel** [S]. `HashMap<String,String>` alias → resolve logical
  model ("claude-opus") → upstream model/key-group; preserve original name for retry + attribution.
- **(F) Adopt the clean 9-method `Adaptor` trait shape** [S/adaptor] (Init/GetRequestURL/SetupHeaders/
  ConvertRequest/DoRequest/DoResponse/GetModelList/GetChannelName) + a generic `OpenAICompatible`
  passthrough for the long tail. Implement only providers the fleet calls.
- **(H) Group/prefix routing + admin per-request channel override** [S] (debug pinning).
- **(G) Per-channel forced system-prompt** [S, optional, auditable] — only if we want central fleet guardrails.

## AVOID (anti-patterns from both reviews — explicit non-goals)
- Plaintext keys at rest → **OpenBao** (one-api Finding 1).
- Default root creds / weak demo secrets → none; any seeded admin gets a random secret (Finding 2,5).
- **Per-IP rate-limiting** → per-agent-token (Finding: wrong granularity behind fleet NAT).
- Substring error classifier → structured status + provider error-type.
- Vision-fetch SSRF → if we proxy image URLs, filter private-IP/metadata/loopback (Finding 3).
- CORS `*`+credentials → strict/empty (header-token gateway needs no CORS) (Finding 4).
- Web admin GUI / user-registration / OAuth / billing / CAPTCHA → declarative config, not a SaaS.
- one-api's race-condition + unseeded-rand bugs → clean async per-request state + proper RNG/atomic.

## Dependency adoption — decision rule: "would a hyperscaler use this as a dependency?"
Applied per crate (enforced in the integrator QC pass; dependency-seam is report-only per ADR-0092 D14,
but the criterion is the real bar). Kernel stays PURE (no deps; confirmed ✓).
- **KEEP (hyperscaler-grade, blessed):** `tokio`, `axum`, `tower`, `hyper`, `tracing`, `serde`/`serde_json`.
- **DROP `reqwest` → use `hyper` client directly.** A hyperscaler builds a hot-path streaming proxy on
  `hyper` (zero-cost, already in-tree), not the `reqwest` convenience wrapper (heavier tree: own pool/
  redirect/cookie/multipart). Also `reqwest 0.13` is nonexistent — the agent hallucinated a version.
- **DROP `aes-gcm` + `pbkdf2`.** Encryption-at-rest is **OpenBao's** job — a hyperscaler does not embed a
  KDF+AEAD in an API gateway when a KMS owns key material. Removing them also deletes the gpt-load
  "hardcoded-salt" anti-pattern entirely. (Gateway holds no key long enough to encrypt; it fetches from
  OpenBao per-refresh into memory.)
- **Consolidate crypto on ONE vetted base** for the only needs (constant-time proxy-key compare +
  key-hash-for-logs): prefer **`ring`** (hyperscaler-grade, blessed, already in-tree from the identity
  build-plan) for `constant_time` + SHA-256; drop `subtle`/`sha2`/`hmac` if `ring` covers it (don't pull
  both RustCrypto *and* ring).
- **Metrics: reuse `shared-hyperscaler-metrics-adapter-prometheus`** (the existing seam) or emit
  **OTel** (ADR-0130) — do NOT add a raw `prometheus` dep per service (a hyperscaler reuses the shared
  metrics lib).
Net new deps after QC: ideally **zero** beyond the blessed set + `ring` (if not already in-tree).

## Horizontal scalability + cloud-native (idea-refine; v2 of the build)
**v1 built in-memory** (AtomicUsize round-robin, per-process failure/quota) → does NOT survive
horizontal scale: N replicas = uncoordinated round-robin, per-replica blacklist, and per-agent quota
enforced **N×** (each replica thinks it owns the whole budget — breaks fairness, the safety property).
Fork considered: (a) shared state in Valkey; (b) consistent-hash sticky sharding (no shared store, but
uneven + rebalance churn); (c) control-plane/xDS service. **Decision: (a)** — canonical, atomic, proven.
- **Stateless data-plane replicas**; coordination state in **Valkey 8.x** (active-key list, per-key
  health/cooldown, per-agent quota) via atomic `LMOVE`/`INCR`/reserve → global RR, fleet-wide blacklist,
  one true quota. Kernel gains a `PoolStore` trait: `InMemory` (single-node/test) | `Valkey` (prod).
- **Cloud-native**: Service + **HPA** (CPU/RPS), readiness/liveness probes, **graceful shutdown draining
  in-flight SSE**, PodDisruptionBudget, resource limits, ConfigMap + `BAO_TOKEN` Secret (12-factor), OTel
  traces/metrics. **Cell + shuffle-shard** key-group isolation (per the gateway-hyperscaler-bar research).
- Keys still per-replica-fetched from OpenBao (refresh); Valkey holds only non-secret pool/quota/health state.

## Dependency reconciliation (audit + hyperscaler criterion) — apply in QC pass
- **reqwest → hyper client** (drops the reqwest + aws-lc-rs tree; hyper already in-tree; reqwest 0.13 was nonexistent).
- **Drop `aes-gcm` + `pbkdf2`** + the unwired `LocalEncryptedKeyStore` — OpenBao owns crypto-at-rest.
- **Consolidate `sha2`/`hmac`/`subtle` on `ring`** (constant-time + SHA) — one crypto base, not RustCrypto+ring.
- **`prometheus` → reuse `shared-hyperscaler-metrics-adapter-prometheus` / OTel** (don't add a per-service metrics dep).
- Add `dependency-rationales` rows for any remaining new dep; **fix the dep-seam gate blindness** (read per-crate
  deps, not just `[workspace.dependencies]`) + a hyperscaler-allowlist subcheck → promote off report-only (separate task).
- Workspace-wide: **replace `serde_yaml`** (archived); move `ed25519-dalek`+`sha2` out of the `*-domain` (kernel-pure) crate.

## Sequencing
v1 (gpt-load core, OpenBao keys) builds now → land in `microservices/cloud-intelligence/` → **v1.5 integration**
folds the above (E,D,A,B,C,F first) → verify (build/clippy/gates) → **deploy to k3s** with `BAO_TOKEN`
from a Secret. Record as an ADR (decision + the working code together) — gateway is real implementation,
not aspirational doctrine. Quota/fairness (E) MUST be in before deploy (else a runaway agent drains the pool).
