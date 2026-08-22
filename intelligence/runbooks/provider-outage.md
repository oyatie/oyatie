# Runbook — Upstream Provider Outage

**Service:** cloud-intelligence
**Owner:** council-foundry + ops-sre-reliability
**Severity:** Sev 2 (one provider down, fallback healthy) → Sev 1 (provider down with no healthy fallback)
**Related SLI:** `slos/availability.openslo.yaml`, `slos/error-rate.openslo.yaml`, `slos/completeness.openslo.yaml`
**Research grounding:** `design/hyperscaler-best-practice-brief.md` §1 (fallback chains), §10 (provider-outage is a distinct, separately-metered state from key-exhaustion; per-provider circuit breaker honoring `Retry-After`).

## What this is

An upstream provider (OpenAI/Codex, Anthropic, or Gemini) is failing **at the provider level** —
5xx, connection failures, or sustained `Retry-After` across *all* keys in the pool, regardless of
which key is used. This is **distinct** from key-exhaustion (`key-exhaustion.md`), where the keys
themselves are tripped. Here the per-provider circuit breaker opens and the gateway returns
**503 `gateway_provider_unavailable`** (after fallback is attempted) with `Retry-After`.

## Detection

- Alert: `cloud_intelligence_provider_breaker_open{provider="..."} == 1`.
- Metric: `cloud_intelligence_upstream_failures_total{provider,code=~"5..|transport"}` spiking across
  *all* fingerprints of a pool simultaneously (the tell that distinguishes this from per-key
  failure).
- SLO: availability + completeness fast-burn (mid-stream drops truncate streams → completeness burn).
- Symptom: 503 `{"error":{"type":"gateway_provider_unavailable", ...}}` + `Retry-After`; or, if a
  fallback absorbed it, elevated `cloud_intelligence_fallback_total{from,to}` with 200s.
- Provider status page confirms an incident.

## Triage (first 5 minutes)

1. **Confirm provider-level, not key-level.** `GET /admin/v1/pools` → if `active_keys > 0` but
   every request still fails 5xx, it is the provider, not the keys. Cross-check
   `cloud_intelligence_upstream_failures_total` is uniform across all fingerprints.
2. **Is a fallback absorbing it?** Check `cloud_intelligence_fallback_total{from="<down>",to="..."}`
   and the fallback pool's `active_keys`. If fallback is healthy and serving, impact is degraded
   latency/cost, not an outage → Sev 2.
3. **Check provider status.** OpenAI / Anthropic / Google status pages; correlate the breaker-open
   timestamp with the provider incident start.
4. **Honor `Retry-After`.** If the provider is returning `Retry-After`, the breaker is already
   using it as the cooldown (brief §10). Confirm the gateway's emitted `Retry-After` matches.

## Mitigation

### Lean on the fallback chain (brief §1)
- The failure ladder is in-key-retry → rotate → **provider-fallback** → 503. If a configured
  fallback pool (e.g. Anthropic for an OpenAI outage, within same-dialect constraints — see PRD
  open-question 1) is healthy, traffic should already be routing there.
- If the fallback is misconfigured or absent and this provider is critical, prioritize standing up
  / pointing at an alternate pool (config change + `POST /admin/v1/pools/{provider}/refresh` for
  the fallback).

### Protect the gateway and tenants (brief §10, OWASP LLM10)
- Do **not** disable the circuit breaker to "force through" traffic — that turns the gateway into
  a retry-storm amplifier against a downed provider (denial-of-wallet risk). Let the breaker hold.
- Confirm streams are aborting cleanly (not hanging): completeness SLI should show truncations as
  `termination!="done_sentinel"`, not stuck connections (brief §10 — never hang a stream).
- Communicate: tenants on the affected provider see 503 + `Retry-After`; tenants on other
  providers/fallbacks are unaffected (per-provider isolation).

## Recovery verification

- Provider status page reports resolved; `cloud_intelligence_upstream_failures_total` for that
  provider drops to baseline.
- The per-provider breaker closes (`provider_breaker_open == 0`); the gateway resumes routing to
  the primary pool (blacklisted keys restore lazily on `select`).
- Synthetic `POST /v1/chat/completions` against the recovered provider returns 200; a stream
  terminates with `data: [DONE]`.
- Availability + completeness burn alerts clear.

## Post-incident

- Review fallback coverage: did every critical pool have a healthy fallback? If not, add one
  (brief §1 — fallback chains as a first-class resilience layer).
- Review breaker tuning: did it open fast enough to stop the retry storm, and close cleanly on
  recovery? Adjust the open threshold / `Retry-After` consumption if it flapped.
- Confirm cost impact: a provider outage that fell back to a more expensive provider should show in
  `llm.usage.v1` cost dims (brief §8) — feed back into FinOps.
- Distinguish in the timeline: this was a provider-outage (`gateway_provider_unavailable`), NOT
  key-exhaustion (`gateway_key_exhausted`) — keep the two states separately metered (brief §10) so
  the post-mortem attributes correctly.

## Escalation

- Sev 1 (no healthy fallback for a critical provider): page council-foundry on-call.
- Engage the provider account owner; track the upstream incident id for the post-mortem.
