# Runbook — Key Pool Exhaustion

**Service:** cloud-intelligence
**Owner:** council-foundry + ops-sre-reliability
**Severity:** Sev 2 (single provider pool exhausted) → Sev 1 (all provider pools exhausted; fleet inference stalled)
**Related SLI:** `slos/availability.openslo.yaml`, `slos/error-rate.openslo.yaml`
**Research grounding:** `design/hyperscaler-best-practice-brief.md` §10 (key-exhaustion is a distinct, separately-metered state); OWASP LLM10 Unbounded Consumption / denial-of-wallet.

## What this is

Every key in one (or more) provider pool is `Blacklisted` and still in cooldown, so
`KeyPool::select` returns `Exhausted`. The gateway returns **503 `gateway_key_exhausted`** with
`Retry-After` = the soonest cooldown-restore time. This is **distinct** from a provider-wide
outage (see `provider-outage.md`) — here the *keys* are tripped (rate-limit / auth / quota
failures accumulated past `blacklist_threshold`), not the provider itself.

Per the brief's OWASP LLM10 control, the gateway **fast-fails** in this state rather than
rotating forever — it must not become a denial-of-wallet amplifier.

## Detection

- Alert: `oya_cloud_intelligence_pool_exhausted{provider="..."} == 1` (pool has zero active keys).
- Metric: `oya_cloud_intelligence_active_keys{provider="..."}` gauge at 0 (mirrors `KeyPool::active_count`).
- SLO: availability fast-burn alert (503s count against availability).
- Symptom: callers see 503 with body `{"error":{"type":"gateway_key_exhausted", ...}}` + `Retry-After`.
- Audit/usage: `status="key_exhausted"` records on `llm.audit.v1` / `llm.usage.v1`.

## Triage (first 5 minutes)

1. **Scope.** One pool or all? `GET /admin/v1/pools` (admin bearer) → per-pool `active_keys`,
   `blacklisted_keys`, `soonest_restore_epoch_ms`. One pool = Sev 2; all pools = Sev 1.
2. **Why blacklisted?** Inspect the dominant upstream failure on the metric
   `oya_cloud_intelligence_key_failures_total{provider,code}`:
   - `429` dominating → provider rate-limit / quota; the keys are healthy but throttled.
   - `401`/`403` dominating → **key is invalid/revoked/rotated upstream** — refresh will NOT
     help until the owned secret-provider handle resolves to corrected material.
   - `5xx` dominating → likely a provider-outage masquerading as key failure → switch to
     `provider-outage.md`.
3. **Check `soonest_restore`.** If it is seconds away, the pool will self-heal (lazy restore on
   next `select`); confirm before any manual action.

## Mitigation

### If keys are throttled (429): wait + shed
- The cooldown already honors upstream `Retry-After` (brief §10). Confirm `Retry-After` on the
  503 is sane (= soonest restore). The pool restores lazily — **no key is permanently lost**.
- Reduce load: tighten per-tenant budgets for the noisiest tenant(s) (identify via
  `oya_cloud_intelligence_tokens_total{tenant}`); the offending tenant should be the one feeling 429
  `budget_exceeded`, not the whole fleet (brief §6 — fail that tenant, not the gateway).
- If a fallback provider pool has capacity, confirm the failover ladder is routing to it
  (`oya_cloud_intelligence_fallback_total`); if not, check the fallback config.

### If keys are invalid (401/403): rotate the secret-provider handle
- The key material behind the `secret-ref://` / `kms-ref://` handle is stale/revoked.
  Update it in cloud-secrets/cloud-kms (the gateway only resolves opaque handles).
- Force a refresh: `POST /admin/v1/pools/{provider}/refresh` (admin bearer + Idempotency-Key) →
  re-reads the owned secret-provider port and rebuilds the in-memory pool. **Never** edit keys
  on the pod/disk — there is no plaintext key store (brief §5).
- Verify: `GET /admin/v1/pools` shows `active_keys > 0`.

### If pool is simply too small: add keys
- Add more secret-provider handles, then `POST .../refresh`. More keys = more
  rotation headroom before exhaustion.

## Recovery verification

- `oya_cloud_intelligence_active_keys{provider}` > 0 for every pool.
- A synthetic `POST /v1/chat/completions` returns 200 (non-stream) and a streamed request
  terminates with `data: [DONE]` (completeness SLI green).
- Availability + error-rate burn alerts clear.
- Audit chain shows `status="ok"` records resuming.

## Post-incident

- If 429-driven: revisit per-tenant budgets and reserved headroom vs shared provider TPM
  (brief §6, §8) — exhaustion under normal load means headroom is mis-sized.
- If 401/403-driven: review the upstream key-rotation calendar and whether the
  secret-provider refresh cadence (`key_refresh_secs`) was too slow to pick up the rotated secret.
- Tune `blacklist_threshold` / `cooldown_base_millis` / `cooldown_jitter_millis` if cooldowns
  were too long (slow recovery) or too short (thrash). Jitter must stay non-zero to prevent
  thundering-herd restore (brief §10).
- Confirm the gateway never rotated forever (no DoS amplification — OWASP LLM10): the 503 should
  have fired promptly once `select` returned `Exhausted`.

## Escalation

- Sev 1 (all pools): page council-foundry on-call; the AI agent fleet is degraded.
- Upstream key/quota issues that need a provider-side fix: engage the account owner for that
  provider (OpenAI / Anthropic / Google).
