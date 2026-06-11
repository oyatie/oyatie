# Cloud Intelligence service — Operational Boundaries

**Authority:** ADR-0373 (resilience state machine), ADR-0373 (audit), ADR-0090 (hyper backbone)
**Research grounding:** `design/hyperscaler-best-practice-brief.md` §10 (operational boundaries + failure modes), §4 (SLOs), §9 (audit governance).
**Last reviewed:** 2026-05-26

## What this document fixes

The hard operational lines the gateway must hold — the places where a tempting shortcut would
violate a research-grounded invariant. These are the "do not cross" boundaries that the SLOs,
runbooks, and failure-modes all assume.

## Boundary 1 — Never rotate forever (no DoS amplification)

When `KeyPool::select` returns `Exhausted` or a provider breaker is open, the gateway **fast-fails**
with 503 + `Retry-After` (= soonest restore). It does **not** keep rotating/retrying — that would
turn the gateway into a denial-of-wallet amplifier against a downed provider (OWASP LLM10, brief
§10). The retry budget is bounded (`max_attempts`); the ladder terminates at a graceful 503.

## Boundary 2 — Never hang a stream

If the first token does not arrive within the TTFT hard-timeout, the gateway **aborts and rotates**
(pre-first-token) rather than holding the connection open (brief §10). A mid-stream drop surfaces as
a truncation (completeness SLI burn), never a hung socket. Streams are byte-passthrough — the
gateway never buffers the whole stream to "complete" it.

## Boundary 3 — Three failure states stay distinct

Key-exhaustion, provider-outage, and tenant-rate-limit are **separate states / error `type`s / SLIs
/ runbooks** (brief §10). The gateway must never collapse them into a generic 503/500 — operations
needs to know which one fired to pick the right runbook. See `design/failure-modes.md`.

## Boundary 4 — Secret-provider-only keys, no plaintext fallback

Provider keys come **only** from owned secret-provider/KMS handles, in-memory only.
There is no plaintext file/env key source to fail-open to (brief §5). If the
secret-provider adapter is unreachable, the gateway serves last-good in-memory
keys and alerts; it never degrades to reading a key from disk.

## Boundary 5 — Audit is a hard requirement; metering is best-effort

`llm.audit.v1` emits on **every** invocation (100%, brief §9). If the audit cannot be guaranteed
(broker down + spool full), the gateway treats the gap as a Sev 1 — a silent hole in the immutable
record is a security event (alert-if-disabled, brief §9). This creates a deliberate **tension** with
availability (Boundary 6): the gateway prefers a recorded failure over an unrecorded success for the
audit stream. Metering (`llm.usage.v1`) may sample/drop without incident.

## Boundary 6 — The audit-vs-availability tension (explicit)

Boundary 5 means there is a real design tension: if the audit substrate is hard-down, does the
gateway fail requests (to preserve the 100%-audit invariant) or serve them un-audited (to preserve
availability)? **Resolution:** the gateway uses a **durable local spool** with bounded capacity;
while the spool has room, requests proceed and audit is buffered for later chain-sealing; if the
spool is exhausted, the gateway sheds load (503) rather than serve un-audited traffic for
audit-mandated tenants/operations. This is a documented, deliberate choice — not an accident — and
the error-budget SLI excludes spool-shed 503s from the "gateway fault" set since they are a
correctness-preserving action. (Implementation detail deferred to IP-001 T6; the boundary is fixed here.)

## Boundary 7 — Never log or return raw secrets

Logs, metrics, admin pool-status, and audit records carry keys **only** as hash fingerprints; raw
keys, bearer tokens, `Authorization` headers, prompts, and completions are never logged (brief §5,
§7). Admin pool-status returns `key_fingerprint`, never the key.

## Boundary 8 — Error budget excludes non-gateway faults

Client-cancellations and upstream-attributable 429/`Retry-After` waits are **excluded** from the
error budget and recorded separately (brief §4). A provider rate-limit storm burns the provider's
budget (surfaced via the per-provider breaker metrics), not the gateway's SLO. See
`slos/error-rate.openslo.yaml` and `slos/ttft.openslo.yaml`.

## Boundary 9 — SLO targets are provisional until measured

The brief (§4) is explicit that there is **no official vendor SLO** for TTFT/latency and that
published ranges vary widely. The gateway's SLO targets (TTFT p95 1500ms, etc.) are **conservative
starting hypotheses** to be replaced with measured baselines — they are labeled as provisional in
the SLO files. Operations must not treat them as contractual until a baseline exists.

## Boundary 10 — Proxy, not host; passthrough, not interpreter

The gateway proxies; it does not host models, interpret prompts, or execute output. The guardrail
PEP is a **hook** (no-op v1 + optional provider-native passthrough), not an in-house classifier
(brief §5). Response caching is out of MVP, and when it lands SSE is **non-cacheable** (brief §1 —
Cloudflare caches text/image only). These are the Non-Goals that keep the operational surface small.

## Capacity / runtime tier

- `pod_runtime_tier: 2` — the gateway brokers pooled provider credentials; Tier-2 isolation.
- The gateway holds **no durable state** beyond an in-memory key-pool cache resolved
  through owned secret-provider/KMS handles, so there is no gateway-owned state to
  replicate (DR is stateless-restart; the existing manifest DR non-claim stands).
  Pooled keys re-hydrate through the secret-provider port on restart.
- Scaling dimension: concurrent streams + per-tenant TPM; horizontal replicas share nothing except
  the (idempotent) secret-provider source and the (append-only) audit chain.

## References

- `design/hyperscaler-best-practice-brief.md` §1, §4, §5, §7, §9, §10.
- `design/failure-modes.md`, `design/audit-evidence-emission.md`, `design/data-residency.md`.
- `slos/*.openslo.yaml`, `runbooks/key-exhaustion.md`, `runbooks/provider-outage.md`.
