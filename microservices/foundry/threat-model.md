---
doc_class: THREAT-MODEL
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: axis-foundry + council-privacy + council-security
related_adrs: [ADR-0028, ADR-0117, ADR-0136, ADR-0137]
---

# Threat Model — foundry (consolidated across 6 BCs)

## Scope

This consolidated threat model covers the foundry µservice as a single
product surface. Each of the six BCs (runtime, supervisor, eval, evidence,
guardrails, providers) ships its own per-BC threat model preserved at
`bc-sources/<bc>/threat-model.md`. This document enumerates cross-BC threats
+ trust boundaries; per-BC threats remain authoritative in their archive.

## Trust Boundaries

The foundry µservice has the following trust boundaries:

1. **Tenant ↔ foundry/runtime SDK** — external untrusted; mTLS + tenant auth.
2. **foundry/runtime ↔ foundry/providers** — internal mTLS; per-pod SPIFFE.
3. **foundry/runtime ↔ foundry/guardrails** — internal mTLS; per-pod SPIFFE.
4. **foundry/runtime ↔ foundry/evidence** — internal mTLS; per-pod SPIFFE.
5. **foundry/runtime ↔ foundry/supervisor** — internal mTLS; per-pod SPIFFE.
6. **foundry/eval ↔ foundry/runtime** (replay sandbox) — internal mTLS;
   read-only registry-cache access; isolated pool.
7. **foundry/providers ↔ external LLM provider** (Anthropic / OpenAI /
   Gemini / in-house) — external untrusted; provider-side TLS + OpenBao-
   bound credentials never resident in any other BC.
8. **foundry/evidence ↔ audit-chain µservice** — internal mTLS.
9. **foundry ↔ tenancy µservice** (DSR cascade, tier ceiling) — internal mTLS.
10. **foundry ↔ observability µservice** — internal mTLS.

## Cross-BC threats (STRIDE)

### Spoofing

| ID | Threat | Affected BCs | Mitigation |
|---|---|---|---|
| S1 | Compromised runtime pod impersonates supervisor command | runtime, supervisor | Per-pod SPIFFE; supervision commands signed Ed25519 + verified at runtime |
| S2 | Compromised provider adapter impersonates a different provider | providers | Per-adapter SPIFFE; router refuses unrecognised SPIFFE |
| S3 | Cross-tenant session spoof via session-id collision | runtime | Session-id contains tenant prefix; Cedar default-deny cross-tenant |

### Tampering

| ID | Threat | Affected BCs | Mitigation |
|---|---|---|---|
| T1 | In-transit modification of capability descriptor (supervisor → runtime) | supervisor, runtime | mTLS + descriptor signing (Ed25519); runtime refuses unsigned |
| T2 | Tamper with evidence pack between recording and audit-chain | evidence | Ed25519 + Merkle per Bominal ADR-0028; audit-chain bridge verifies before seal |
| T3 | Tamper with eval golden-output store | eval | Golden outputs content-addressed (SHA256); replays verify hash |
| T4 | Tamper with guardrail ruleset to bypass enforcement | guardrails | Ruleset version signed; runtime refuses unsigned/older |

### Repudiation

| ID | Threat | Affected BCs | Mitigation |
|---|---|---|---|
| R1 | Tenant denies an invocation occurred | runtime, evidence | Audit-chain seal at InvocationCompleted; non-repudiation via Merkle |
| R2 | Operator denies engaging kill-switch | supervisor, evidence | Kill-switch engage emits signed audit-chain record + two-person admin where applicable |

### Information disclosure

| ID | Threat | Affected BCs | Mitigation |
|---|---|---|---|
| I1 | Provider credential leak from any non-providers BC | providers, runtime, supervisor, eval, evidence, guardrails | Credentials only resident in providers/openbao-adapter; never in env vars, logs, or cross-BC traffic; AC-X4 verification |
| I2 | Session content leak via cross-tenant Redis collision | runtime | Per-tenant Redis key prefix + Cedar enforcement |
| I3 | Eval replay leaks production session content | eval, runtime | Eval-run sandbox pool uses synthetic-only data per `policy/eval-synthetic-phi-only.md` |
| I4 | Evidence-pack export to regulator includes wrong-tenant content | evidence | Per-export Cedar scope check; regulator-export scope policy is BC-owned |
| I5 | Guardrail decision log leaks prompt content | guardrails | Decision log records decision + hash(prompt), not prompt itself |
| I6 | Audit-chain bridge leaks across audit boundaries | evidence, audit-chain | mTLS + per-tenant scoping at bridge |

### Denial of service

| ID | Threat | Affected BCs | Mitigation |
|---|---|---|---|
| D1 | Capability registry-cache thrash on hot-reload storm | runtime, supervisor | Hot-reload debounced + per-tenant rate limit |
| D2 | Provider router cascade failure on outage | providers | Circuit-breaker + per-provider fallback chain |
| D3 | Evidence-pack assembly exhausts S3 backend | evidence | Per-tenant pack-size quota; scheduled-for-distinct-tracked-work export queue |
| D4 | Guardrail classifier-model serving overload | guardrails | HPA + ONNX runtime warm pool |
| D5 | Eval-run GPU pool exhaustion blocks production replays | eval | Eval pool isolated from production runtime pool |
| D6 | Kill-switch engage cascades to all tenants | supervisor | Kill-switch is per-tenant or per-fleet-scope by default; global-kill requires two-person admin |

### Elevation of privilege

| ID | Threat | Affected BCs | Mitigation |
|---|---|---|---|
| E1 | Tenant capability requests autonomy tier above ceiling | runtime, guardrails, supervisor | Autonomy-tier-gate refuses at dispatch; supervisor refuses at registration |
| E2 | Cross-tenant pivot via shared runtime pod | runtime | Per-pod single-tenant binding; Cedar refuses cross-tenant request even if pod is shared |
| E3 | Eval-run executes production capabilities | eval, runtime | Eval pool's runtime workers are sandbox-tagged; production runtime refuses sandbox-originated invocations |
| E4 | Supervisor command replayed after revocation | supervisor | Commands carry monotonic supervisor_command_seq; revoked commands refused |

## Data classification

Per Bominal ADR-0028 + ADR-0117 + `feedback_quality_performance_scalability_bar.md`:

| Data | Class | Resident BC | Touchpoints |
|---|---|---|---|
| Capability descriptor | INTERNAL_ONLY (descriptor); BEHAVIORAL_TENANT_PRODUCT (per-tenant variants) | supervisor (canonical) + runtime (cache) + eval (read) | mTLS; signed |
| Session content | SENSITIVE_PIPA_ART23 (per session jurisdiction) | runtime | Redis TLS + per-pack KMS |
| Invocation result | BEHAVIORAL_TENANT_PRODUCT | runtime + evidence (sealed) | mTLS; audit-chain |
| Provider credential | RESTRICTED_PROVIDER_SECRET | providers (OpenBao) | OpenBao only; never elsewhere |
| Guardrail decision log | AUDIT | guardrails + evidence | audit-chain |
| Eval golden output | INTERNAL_ONLY | eval | content-addressed; signed |
| Evidence pack | AUDIT | evidence | Ed25519+Merkle |
| Supervision event | AUDIT | supervisor + evidence | audit-chain |
| Regulator export | REGULATOR_SCOPE | evidence | Cedar-scoped + signed |

## Per-BC threat archives

Each BC's full per-BC threat model is preserved at:

- `bc-sources/runtime/threat-model.md` — capability-executor + session-state
  + invocation-orchestrator + runtime-pool + capability-registry-cache
  threats (526 lines).
- `bc-sources/supervisor/threat-model.md` — fleet-lifecycle + kill-switch
  + autonomy-policy + supervision-bus threats (492 lines).
- `bc-sources/eval/threat-model.md` — eval-runner + parity-analyzer +
  replay-engine + golden-store threats (684 lines).
- `bc-sources/evidence/threat-model.md` — capability-invocation-recorder +
  evidence-pack-builder + regulator-export + audit-chain-bridge threats
  (144 lines).
- `bc-sources/guardrails/threat-model.md` — prompt-classifier +
  output-validator + autonomy-tier-gate + content-safety + jailbreak-detector
  + AI-slop-detector threats (588 lines).
- `bc-sources/providers/threat-model.md` — router + 8 adapters (Anthropic
  API/Sub + OpenAI API/Sub + Gemini API/Sub + in-house + OpenBao) threats
  (262 lines).

The per-BC archives MUST be consulted for BC-internal STRIDE coverage; this
top-level threat model covers only cross-BC seams + trust boundaries.

## References

- ADR-0028: Audit-chain Ed25519 + Merkle.
- ADR-0117: Data-residency + jurisdiction codes.
- ADR-0136: Foundry as single µservice.
- ADR-0137: Foundry bounded contexts.
- `feedback_no_silent_regression.md` — public-contract preservation.
