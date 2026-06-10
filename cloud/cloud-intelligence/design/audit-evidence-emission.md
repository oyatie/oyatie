# Cloud Intelligence service — Audit Evidence Emission

**Authority:** ADR-0373 (Bedrock-shaped audit + default-off body logging), ADR-0003 audit chain
**Research grounding:** `design/hyperscaler-best-practice-brief.md` §9 (audit evidence — Bedrock `ModelInvocationLog` reference schema; logging mandated, tamper-alert if disabled, IAM-restricted, SIEM-integrated), §3 (async/events), §7 (residency).
**Last reviewed:** 2026-05-26

## Reference schema: AWS Bedrock ModelInvocationLog

The brief (§9) sets the reference: AWS Bedrock's `ModelInvocationLog` — both sinks emit identical
JSON; bodies >100KB are externalized. Governance requirements the brief calls out: **logging
mandated, tamper-alert if disabled, IAM-restricted, SIEM-integrated.**

## Adopted record (brief §9 "Adopt")

Every invocation emits an **immutable** `llm.audit.v1` record (see
`contracts/cloud-intelligence.asyncapi.yaml#AuditPayload`) with the Bedrock-shaped field set:

```jsonc
{
  "schema_version": "1",
  "timestamp": "2026-05-26T12:00:00Z",
  "tenant_id": "tenant-acme",
  "request_id": "1f2e...-uuid",          // = correlationId = X-Request-Id
  "provider": "openai",                   // openai | anthropic | gemini
  "model_id": "gpt-4o",
  "operation": "chat.completions",        // chat.completions | embeddings
  "ingress_token_ref": "sha256:ab12…",    // HASHED ingress proxy-key (never raw)
  "key_pool_member_ref": "kf-7c…",        // HASHED KeyFingerprint of the pooled key used
  "input_token_count": 312,
  "output_token_count": 980,
  "status": "ok",                         // ok|error|rate_limited|key_exhausted|provider_unavailable|budget_exceeded|client_cancelled
  "latency_ms": 1840,
  "ttft_ms": 410,
  "cost_usd_minor_units": 47,
  "residency_region": "kr-seoul-1",
  "retry_after_seconds": null,
  "prompt_uri": null,                     // present ONLY on per-tenant body-spill opt-in
  "completion_uri": null,                 // present ONLY on per-tenant body-spill opt-in
  "audit_chain_prev_hash": "sha256:…"     // tamper-evident hash chain link
}
```

## Properties (brief §9 + repo audit-chain convention)

### 1. Append-only, hash-chained into `evidence/audit-chain.jsonl`
- Each record carries `audit_chain_prev_hash` linking it to the previous chain entry — a
  tamper-evident hash chain (brief §9). This is the same `evidence/audit-chain.jsonl` substrate the
  rest of the Oyatie fleet seals into; `manifest.json#audit_chain.seal_events` enumerates the gateway
  event types that seal.

### 2. 100% emission for audit; metering may sample
- The audit record emits on **every** invocation (brief §9 — 100% emission). The `llm.usage.v1`
  metering record may be sampled. **Audit may never be sampled or dropped silently** — if it cannot
  emit, that is a Sev 1 (see `design/failure-modes.md` §6.1 and `operational-boundaries.md`).

### 3. Body-spill >N KB, default-OFF
- Prompt/completion bodies are **absent** from the record by default (brief §7). On per-tenant
  opt-in, redacted bodies spill to a residency-pinned bucket and are referenced via
  `prompt_uri`/`completion_uri` (Bedrock >100KB externalization). See `design/data-residency.md`.

### 4. Hashed refs, never raw secrets
- `ingress_token_ref` and `key_pool_member_ref` are non-reversible hashes (the kernel's
  `KeyFingerprint` for the key). No raw key, bearer token, or `Authorization` header ever enters the
  record (brief §5, §9).

### 5. Alert-if-disabled (tamper detection)
- Disabling audit emission raises a tamper alert (brief §9 — "tamper-alert if disabled"). The
  control plane treats a silent gap in the chain as a security event.

### 6. CI-verified + access-controlled + SIEM
- The schema is versioned (`schema_version`) and gated in CI (brief §3). Read access is gated behind
  the owned policy-engine `AuditReader` authority (distinct from admin) and SIEM-forwarded
  (brief §7, §9; current Cedar file is a transient fixture).

## Two streams, one envelope

| Stream | Purpose | Sampling | Access | PII |
|---|---|---|---|---|
| `llm.audit.v1` | immutable invocation record | never (100%) | `AuditReader` + SIEM | none inline (bodies by URI on opt-in) |
| `llm.usage.v1` | FinOps metering / showback | may sample | FinOps | low-PII (token counts, cost) |

Both share the envelope (correlationId = request id, tenant, timestamp, schemaVersion) per
`contracts/cloud-intelligence.asyncapi.yaml` (AsyncAPI 3.1.0) so a single `request_id` ties audit ↔ usage ↔
OTel trace.

## manifest.json#audit_chain

```jsonc
"audit_chain": {
  "enabled": true,
  "seal_events": [
    "llm_invocation_audit",       // per-invocation immutable record
    "key_pool_refresh",           // secret-provider handle refresh (admin op)
    "key_blacklisted",            // a key tripped to cooldown (denial-of-wallet trail)
    "provider_breaker_open",      // provider-outage breaker opened
    "budget_exceeded"             // a tenant hit a hard budget cap
  ]
}
```

## Non-claims

- The current foundation (`CS-CLOUD-INTELLIGENCE-AGENT-DISPATCH-001`) does **not** implement runtime
  audit-chain persistence (manifest `audit_chain.enabled` was `false`); this document + the manifest
  update spec the production posture. Live emission lands with the rest-crate wiring (IP-001 T6).

## References

- `design/hyperscaler-best-practice-brief.md` §3, §7, §9.
- `contracts/cloud-intelligence.asyncapi.yaml` (`AuditPayload`, `UsagePayload`).
- `evidence/audit-chain.jsonl` (the fleet-wide hash-chained substrate).
- AWS Bedrock model-invocation logging reference (brief §9).
