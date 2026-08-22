# Cloud Intelligence service — Failure Modes (the failure ladder + FMEA)

**Authority:** ADR-0373 (key-pool resilience state machine), ADR-0090 (hyper backbone)
**Owner:** council-foundry + ops-sre-reliability
**Research grounding:** `design/hyperscaler-best-practice-brief.md` §1 (resiliency = LB pool + circuit breaker + bounded retries + fallback chains), §10 (operational boundaries — detect→remove→cooldown→auto-restore; distinguish key-exhaustion vs provider-outage vs tenant-rate-limit).
**Last reviewed:** 2026-05-26

## The failure ladder (canonical resolution order)

Per the brief §10 "Adopt", every upstream failure is resolved by climbing this ladder in order,
stopping at the first rung that succeeds:

```
  ┌─ 1. IN-KEY RETRY ──────────────────────────────────────────────────────────┐
  │   Only on TRANSIENT failures (statuses 429,500,502,503,504 + transport).     │
  │   Bounded (max_attempts) + jittered backoff. Idempotent only.                │
  │   NEVER retry past the first streamed token (brief §10 — don't replay a       │
  │   half-emitted stream). For streams, a pre-first-token failure may retry;     │
  │   a mid-stream failure aborts to rung 4.                                      │
  └──────────────────────────────────────────────────────────────────────────────┘
            │ still failing
            ▼
  ┌─ 2. ROTATE KEY ────────────────────────────────────────────────────────────┐
  │   KeyPool::select → next ACTIVE key (round-robin). The failed key's          │
  │   record_failure() increments its counter; at blacklist_threshold it trips   │
  │   to Blacklisted with cooldown = base + jitter (Retry-After if upstream       │
  │   supplied one).                                                              │
  └──────────────────────────────────────────────────────────────────────────────┘
            │ pool exhausted (select → Exhausted)
            ▼
  ┌─ 3. PROVIDER FALLBACK ─────────────────────────────────────────────────────┐
  │   Route to a configured alternate provider pool (same-dialect for MVP;       │
  │   cross-dialect re-translation deferred — PRD open-question 1). LiteLLM       │
  │   fallbacks / context_window_fallbacks / content_policy_fallbacks pattern.   │
  └──────────────────────────────────────────────────────────────────────────────┘
            │ no fallback / fallback also exhausted
            ▼
  ┌─ 4. GRACEFUL 503 ──────────────────────────────────────────────────────────┐
  │   OpenAI-shaped error envelope + Retry-After = soonest cooldown-restore.     │
  │   type = gateway_key_exhausted (keys tripped) OR                             │
  │          gateway_provider_unavailable (provider/breaker open).               │
  │   FAST-FAIL — never rotate forever (OWASP LLM10, brief §10).                 │
  └──────────────────────────────────────────────────────────────────────────────┘
```

## The three distinct, separately-metered failure states (brief §10)

The brief is explicit that these must be **separate states / error `type`s / SLIs** — conflating
them blinds operations:

| State | Cause | HTTP / error type | Metered as | Runbook |
|---|---|---|---|---|
| **Key-exhaustion** | Every key in the pool is blacklisted/cooling-down | 503 `gateway_key_exhausted` | `status=key_exhausted`; counts vs availability | `runbooks/key-exhaustion.md` |
| **Provider-outage** | Provider-level 5xx/transport across all keys; breaker open | 503 `gateway_provider_unavailable` | `status=provider_unavailable`; availability + completeness | `runbooks/provider-outage.md` |
| **Tenant-rate-limit** | Tenant exceeded its budget/rate | 429 `budget_exceeded` / `rate_limit_error` | `status=budget_exceeded`; **excluded** from error budget | (correct backpressure; not an incident) |

## FMEA

### 1. Key-pool state machine (kernel)

#### 1.1 All keys blacklisted simultaneously
- **Detection:** `KeyPool::select` → `Exhausted`; `cloud_intelligence_active_keys == 0`.
- **Immediate:** 503 `gateway_key_exhausted` + `Retry-After` = soonest restore; lazy restore heals
  the pool automatically as cooldowns expire.
- **Long-term:** Right-size pool depth + per-tenant budgets/headroom (brief §6).
- **Residual:** Low (self-healing; jitter prevents synchronized re-trip).

#### 1.2 Cooldown too short → thrash (re-trip immediately)
- **Detection:** High `cloud_intelligence_key_blacklist_total` churn.
- **Immediate:** Increase `cooldown_base_millis`.
- **Long-term:** Tune via measured restore success; keep jitter non-zero.
- **Residual:** Low.

#### 1.3 Cooldown too long → slow recovery
- **Detection:** Availability burn persists after upstream recovers.
- **Immediate:** `record_success` via a synthetic probe restores a key out-of-band (kernel supports
  this); or reduce `cooldown_base_millis`.
- **Residual:** Low.

#### 1.4 jitter_seed not fresh per call → synchronized restore (thundering herd)
- **Detection:** Multiple keys restore in the same instant; coordinated re-trip.
- **Immediate:** Verify the runtime supplies fresh entropy to `record_failure(jitter_seed)`.
- **Long-term:** Property test the jitter source distribution.
- **Residual:** Low (the kernel folds the seed deterministically; correctness depends on a varied seed).

### 2. Provider adapters + SSE passthrough (rest)

#### 2.1 Hung stream — no first token
- **Detection:** TTFT hard-timeout exceeded; completeness SLI burn.
- **Immediate:** Abort + rotate (rung 2); never hold the connection (brief §10).
- **Long-term:** Tune the TTFT hard-timeout to the provider's p99 cold start.
- **Residual:** Low.

#### 2.2 Mid-stream upstream drop
- **Detection:** Stream ends without `data: [DONE]`; `termination!="done_sentinel"`.
- **Immediate:** Cannot safely replay a half-emitted stream — surface the truncation to the caller;
  count as completeness burn.
- **Long-term:** Provider-fallback only helps pre-first-token; mid-stream truncation is inherent.
- **Residual:** Medium (inherent to streaming).

#### 2.3 Provider dialect drift (Anthropic/Gemini shape change)
- **Detection:** Adapter translation errors; 4xx from upstream on well-formed canonical input.
- **Immediate:** Pin the provider API version (`anthropic-version`); roll the adapter.
- **Long-term:** Contract tests per adapter against recorded provider fixtures.
- **Residual:** Low.

### 3. Secret-provider key store (rest)

#### 3.1 Secret-provider adapter unreachable on refresh
- **Detection:** `cloud_intelligence_key_refresh_failures_total` > 0.
- **Immediate:** Serve last-good in-memory keys (refresh is best-effort); alert. **Never** fail-open
  to a plaintext key source (brief §5).
- **Long-term:** cloud-secrets/cloud-kms adapter HA per cell.
- **Residual:** Low (in-memory cache survives transient secret-provider adapter blips).

#### 3.2 Key rotated upstream but not behind the registered handle
- **Detection:** Keys go 401/403 across the pool → looks like key-exhaustion.
- **Immediate:** Update the owned secret-provider/KMS handle + `POST .../refresh`
  (see `runbooks/key-exhaustion.md`).
- **Residual:** Low.

### 4. Auth realms (rest)

#### 4.1 Realm token leak
- **Detection:** Anomalous admin/ingress activity; audit review.
- **Immediate:** Rotate the realm token (k8s Secret); the constant-time check limits brute-force.
- **Long-term:** Short-lived tokens; per-realm rate limits.
- **Residual:** Low.

#### 4.2 Constant-time check regressed to early-return
- **Detection:** Timing-side-channel test in CI (compare equal-length vs differing-prefix tokens).
- **Immediate:** Restore `subtle::ConstantTimeEq`.
- **Residual:** Low (CI-guarded).

### 5. Per-tenant budgets (rest)

#### 5.1 Budget check races under concurrency → overspend
- **Detection:** Tenant spend exceeds cap in a window.
- **Immediate:** Atomic counter on the budget window; reserve-then-commit on admission.
- **Long-term:** Reconcile admission estimate vs actual-token metering (brief §8).
- **Residual:** Low (bounded by one in-flight request's tokens).

#### 5.2 Reserved headroom mis-sized vs shared provider TPM
- **Detection:** Pool 429s under aggregate load even though no single tenant is over budget.
- **Immediate:** Re-allocate per-tenant headroom (brief §6).
- **Residual:** Medium (capacity-planning, not a code bug).

### 6. Audit / metering emission (rest)

#### 6.1 Broker unavailable → audit cannot emit
- **Detection:** `cloud_intelligence_audit_emit_failures_total` > 0; alert-if-disabled.
- **Immediate:** Buffer + retry; if the audit cannot be guaranteed, treat as Sev 1 (the immutable
  record is a hard requirement, brief §9). Metering (usage) may sample/drop; audit may not.
- **Long-term:** Local durable spool with backpressure into the request path if the chain would gap.
- **Residual:** Medium (the audit-vs-availability tension — documented in `operational-boundaries.md`).

#### 6.2 Body-spill bucket in wrong region (residency violation)
- **Detection:** `residency_region` mismatch check.
- **Immediate:** Block the spill; bodies stay un-persisted (default-OFF posture); alert.
- **Long-term:** Per-tenant region pinned at the bucket (brief §7).
- **Residual:** Low.

## Composite failure modes

### C.1 Provider outage + its fallback also down
- **Detection:** Two breakers open.
- **Mitigation:** 503 `gateway_provider_unavailable` + `Retry-After`; no further amplification.
- **Residual:** Medium (rare; external).

### C.2 Secret-provider adapter down + key rotation needed simultaneously
- **Detection:** Refresh failing AND keys 401/403.
- **Mitigation:** Cannot refresh until the adapter returns; serve last-good (may be
  stale/invalid) → may degrade to key-exhaustion 503. Restore cloud-secrets/cloud-kms
  adapter health first.
- **Residual:** Medium.

## References

- `design/hyperscaler-best-practice-brief.md` §1, §10.
- `crates/cloud-intelligence-kernel/src/lib.rs` (state machine; unit tests cover 1.1–1.4 transitions).
- `runbooks/key-exhaustion.md`, `runbooks/provider-outage.md`.
- `design/operational-boundaries.md` (the audit-vs-availability tension), `design/tenant-isolation.md`.
