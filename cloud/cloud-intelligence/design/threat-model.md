# Cloud Intelligence service — Threat Model

**Authority:** owned policy-engine port, ADR-0373 (key-pool resilience + per-tenant isolation), ADR-0373 (audit)
**Framework:** OWASP Top 10 for LLM Applications **2025** (proxy-relevant subset), plus STRIDE per surface.
**Research grounding:** `design/hyperscaler-best-practice-brief.md` §5 (threat model — proxy subset + vendor guardrail patterns), §7 (data residency), §9 (audit).
**Last reviewed:** 2026-05-26

## Scope

cloud-intelligence is a **reverse proxy**, not a model host, so only the proxy-relevant subset of the
OWASP LLM Top 10 applies. The brief (§5) identifies that subset: **LLM01** Prompt Injection,
**LLM02** Sensitive Information Disclosure, **LLM05** Improper Output Handling, **LLM07** System
Prompt Leakage, **LLM10** Unbounded Consumption (incl. denial-of-wallet + model extraction). The
remaining categories (LLM03 supply-chain of model weights, LLM04 data/model poisoning, LLM06
excessive agency, LLM08 vector/embedding weaknesses, LLM09 misinformation) are **out of scope** for
a proxy that does not train, store weights, host vectors, or act with agency — they belong to the
upstream provider or to the calling agent.

## Trust boundaries

1. **Agent/tenant ↔ gateway ingress.** Caller presents an ingress bearer token (constant-time
   check, `subtle`). The token maps to a tenant id + budget tier. No raw provider key ever crosses
   this boundary.
2. **Gateway ↔ cloud-secrets/cloud-kms.** Provider keys resolved from
   `secret-ref://` / `kms-ref://` handles through the owned secret-provider port;
   in-memory only.
3. **Gateway ↔ upstream provider.** The gateway injects the pooled provider auth header and proxies.
   The provider sees the gateway's egress identity, never the tenant's.
4. **Gateway ↔ event substrate.** `llm.usage.v1` (low-PII) + `llm.audit.v1` (immutable, access-
   controlled) + audit-chain.
5. **Admin/audit ↔ control plane.** A distinct admin bearer realm + a separate audit-read authority
   (owned policy-engine default-deny cross-realm; Cedar is a transient fixture).

## OWASP LLM Top 10 (2025) — proxy subset

### LLM01 — Prompt Injection
- **Threat:** A malicious prompt manipulates the model. As a proxy the gateway does not interpret
  prompts, but it is the natural **central PEP** to screen input.
- **Mitigation:** A pluggable input-guardrail **hook** at the ingress edge (brief §5 — no-op v1 +
  optional provider-native passthrough to Bedrock Guardrails / GCP Model Armor, which screen input
  AND output). The gateway does not itself classify content (Non-Goal); it provides the enforcement
  point so a guardrail provider can be wired without changing callers.
- **Residual risk:** Medium until a guardrail provider is wired (v1 hook is no-op).

### LLM02 — Sensitive Information Disclosure
- **Threat:** Prompts/completions carry tenant PII; logs or audit could leak it; one tenant could
  see another's data.
- **Mitigation:** Prompt/completion body logging **default-OFF** (brief §7); when opted-in, bodies
  spill to a residency-pinned bucket with a redaction pass, referenced by URI, never inline in
  events. Hash-only logging — raw key/`Authorization`/prompt/completion never logged (brief §5).
  Per-tenant key pools + owned policy-engine cross-tenant default-deny prevent cross-tenant disclosure (brief §6).
  Audit-read gated behind a distinct authority + SIEM (brief §7).
- **Residual risk:** Low.

### LLM05 — Improper Output Handling
- **Threat:** The model's output (which may contain injected markup, scripts, or unsafe content) is
  passed downstream without scrutiny.
- **Mitigation:** SSE is byte-passthrough by design (the gateway must not corrupt the stream), so
  output handling is the **caller's** responsibility — documented explicitly. The same guardrail
  hook (LLM01) can screen output via provider-native passthrough (Bedrock screens output too). The
  gateway never executes or interprets model output.
- **Residual risk:** Medium (delegated to caller + optional guardrail).

### LLM07 — System Prompt Leakage
- **Threat:** A system prompt embedded by a calling agent leaks via the response or logs.
- **Mitigation:** The gateway never logs request bodies (brief §5), so it cannot leak system
  prompts through its own telemetry. The audit record carries only token counts + hashed refs, not
  prompt text, unless opt-in body-spill is enabled (and then only to an access-controlled,
  residency-pinned store).
- **Residual risk:** Low (gateway-side); the calling agent owns its own system-prompt hygiene.

### LLM10 — Unbounded Consumption (denial-of-wallet + model extraction)
- **Threat:** Runaway or hostile consumption drains provider budget (denial-of-wallet); bulk
  extraction of model behavior; a retry storm amplifies an upstream failure into a cost event.
- **Mitigation (front-line control):** The jittered-cooldown key-pool state machine
  (`crates/oya-cloud-intelligence-kernel`) — failing keys are blacklisted and cooled down; **jitter
  prevents thundering-herd restore** (brief §5, §10). Per-tenant **hard token/cost budgets** across
  concurrent windows + max-prompt-size precheck + per-key concurrency (brief §5, §8). All-keys-
  cooling-down **fast-fails** with `Retry-After` rather than rotating forever — the gateway must
  not be a DoS amplifier (brief §10). Per-realm rate limits.
- **Residual risk:** Low (this is the gateway's primary purpose).

## STRIDE per surface

### Spoofing
- **Threat:** Caller forges a tenant id or presents a stolen token.
- **Mitigation:** Constant-time bearer check (`subtle`); tenant derived from the token, not from a
  client-supplied header (the `X-Oyatie-Tenant` override is honored only for privileged principals).
  owned policy-engine rule `principal.tenant_id == resource.tenant_id`.

### Tampering
- **Threat:** Attacker tampers with the audit record or the SSE stream in flight.
- **Mitigation:** Audit records are hash-chained into `evidence/audit-chain.jsonl`
  (`audit_chain_prev_hash`) — tamper-evident (brief §9). TLS on every hop (ingress,
  secret-provider adapter, upstream, broker).

### Repudiation
- **Threat:** A tenant denies making an expensive call.
- **Mitigation:** 100% immutable `llm.audit.v1` emission per invocation with hashed ingress-token +
  key-pool-member refs, token counts, cost, status; alert-if-disabled (brief §9).

### Information disclosure
- **Threat:** Raw provider keys, tokens, or prompts leak via logs, metrics, or admin surfaces.
- **Mitigation:** Secret-provider/KMS handles only, in-memory only; kernel sees only
  `KeyFingerprint`; admin pool-status returns fingerprints, never raw keys (brief §5);
  body logging default-OFF (brief §7).

### Denial of service
- **Threat:** A tenant floods the gateway or drains the provider budget.
- **Mitigation:** Per-tenant rate + token budgets keyed on tenant id (not the shared key) so one
  tenant's storm fails *that tenant* with 429, not the gateway (brief §6); reserved headroom vs
  shared provider TPM; the LLM10 controls above.

### Elevation of privilege
- **Threat:** An ingress token performs an admin/audit action.
- **Mitigation:** Two distinct realms; the owned policy-engine port default-denies
  admin/audit actions unless the principal is in the corresponding realm (the
  bundled Cedar file is a transient fixture for this port); constant-time check per realm.

## High-impact incident scenarios

1. **Provider key leak.** Detection: anomalous usage on a fingerprint / provider abuse report.
   Response: rotate the key behind its `secret-ref://` / `kms-ref://` handle,
   `POST .../refresh`; audit-chain forensic review; the key was never on disk/env
   (brief §5), narrowing the leak surface to memory/transit.
2. **Denial-of-wallet attack.** Detection: budget-burn spike on one tenant; `budget_exceeded` 429s.
   Response: the per-tenant cap already contained it to that tenant; tighten the tenant's budget;
   review max-prompt-size precheck. (Runbook: `runbooks/key-exhaustion.md`.)
3. **Audit emission disabled (tamper).** Detection: alert-if-disabled fires (brief §9). Response:
   treat as Sev 1 — a gap in the immutable record; restore emission, reconcile the chain, forensic.
4. **Cross-tenant disclosure via mis-scoped token.** Detection: policy-engine denial + audit event; or
   audit review shows a tenant id mismatch. Response: revoke the token, lock the realm, GDPR/PIPA
   notification path if PII was exposed (see `design/data-residency.md`).

## References

- OWASP Top 10 for LLM Applications 2025 (brief §5; list official, per-category detail cross-checked supplemental).
- AWS Bedrock Guardrails "how it works" + model-invocation logging; GCP Model Armor; Cloudflare Guardrails + DLP (brief §5).
- `design/hyperscaler-best-practice-brief.md` §5, §7, §9.
- `policy/cloud-intelligence.cedar`; `design/data-residency.md`; `design/audit-evidence-emission.md`.
