---
doc_class: PolicyDocument
title: Credential-Isolation Policy
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-foundry
deciders: council-architecture, ops-security, axis-foundry
related_adrs: [ADR-0025, ADR-0028, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/intelligence/threat-model.md
  - microservices/intelligence/runbooks/credential-rotation.md
  - microservices/intelligence/runbooks/provider-credentials-revoke.md
  - microservices/intelligence/policy/data-residency.md
review_cadence: quarterly + after any credential-leak near-miss
doc_status: published
---

# Credential-Isolation Policy (foundry-providers µservice)

## Purpose

Define the binding invariants that prevent raw provider credentials (vendor API keys, subscription cookies, OAuth refresh tokens) from leaking into any non-OpenBao surface — repos, logs, structured-log span attributes, error messages, agent chat windows, build artefacts, evidence ledger payloads, Cargo lockfiles, container env vars, or shell history. Per durable user directive (2026-05-12): OpenBao is the **canonical** SecretReference path; deviation is a P0 incident.

This document encodes the policy as testable invariants enforced by:
1. Compile-time type discipline (`ResolvedCredential` is opaque).
2. CI lanes (`oya-foundry-providers-credential-isolation`, `oya-check-no-raw-credentials`).
3. Cedar policy (OpenBao token scope; read-only on `providers/*`).
4. Process discipline (2-person rule for adapter publish).

## Invariants

### CI-INV-01 — SecretReference is the only credential path

**Statement.** Every provider credential is referenced by a `SecretReference` URI of the shape `openbao://<pack>/<tenant>/providers/<vendor>/<credential-name>[?version=<n>]`. The URI is the only artefact that may appear in source, configuration, registry rows, ADRs, or PRs. The credential bytes themselves are resolved by `oya-foundry-providers-adapter-openbao` exclusively, in-process, just-in-time per upstream call.

**Conformance.** `oya-check-no-raw-credentials` LEAN lane greps the entire merged diff for credential-shaped strings (regex set in `crates/oya-check-no-raw-credentials/src/patterns.rs`); BLOCKER lane; zero-occurrence required.

### CI-INV-02 — ResolvedCredential is opaque

**Statement.** The in-process `ResolvedCredential` type:
- Has no `Serialize` / `Deserialize` impl.
- Has `Debug` impl that emits `ResolvedCredential { vendor: "...", redacted: true }`.
- Has no `Display` impl.
- Has `Drop` that zeroises the credential bytes (via `zeroize::Zeroize`).
- Crosses no FFI boundary as raw bytes; instead, an adapter takes the `ResolvedCredential` by value and uses its `with_credential(|bytes| { ... })` closure-based accessor.

**Conformance.** `oya-foundry-providers-credential-isolation` lane checks the `ResolvedCredential` type definition byte-for-byte against the canonical contract; any deviation is a hard fail.

### CI-INV-03 — No credential bytes in audit, logs, or traces

**Statement.** Adapters do not log credentials, do not put credentials in OTel span attributes, do not emit credentials in error messages, do not write credentials to disk, and do not include credentials in `ProviderInvoked` event payloads. Audit records include only:
- A `credential_ref` (the `SecretReference` URI).
- A `credential_lease_id` (opaque ID from OpenBao).
- A `credential_resolved_at` timestamp.

**Conformance.** Per-adapter regression test in `tests/integration/<vendor>_no_credential_leak.rs` injects a known-shape credential into a mock OpenBao response, runs the adapter end-to-end with a recording log subscriber + OTel collector, then asserts the recorded logs + spans + audit events contain zero occurrences of the credential bytes.

### CI-INV-04 — Lease-bounded credential lifetime

**Statement.** The adapter holds a `ResolvedCredential` only for the duration of one upstream HTTP call; on completion (success or error), the credential is dropped immediately. OpenBao lease TTL is bounded (default 15 min); on expiry, the adapter requests a fresh lease rather than retain the credential past lease expiry.

**Conformance.** `tests/integration/openbao_lease_lifecycle.rs` verifies lease expiry triggers re-resolution; flame-graph profiling confirms the credential allocation is freed within 1 ms of the upstream call completing.

### CI-INV-05 — OpenBao token scope minimisation

**Statement.** The pod-bound OpenBao token issued to a foundry-providers pod has only `read` scope on `openbao://<pack>/<tenant>/providers/*` for the packs+tenants that the pod is configured to serve. No `list`, no `write`, no `delete`, no `metadata` permissions.

**Conformance.** OpenBao policy is committed to `microservices/cloud-secrets/policy/foundry-providers.hcl` (cross-µservice ref); audit at deploy time verifies the issued token matches the policy hash.

### CI-INV-06 — No credentials in subscription-channel cookie persistence

**Statement.** Subscription transports (Claude Pro/Max, ChatGPT Plus, Gemini Advanced) hold session cookies. These cookies are credentials. They are stored in OpenBao as opaque blobs under `openbao://<pack>/<tenant>/providers/<vendor>/subscription-cookie`; the adapter never persists them to local disk, never writes them to redis, and never logs them.

**Conformance.** `oya-check-no-cookie-persistence` sub-lane greps for `cookie_store.save(`, `set-cookie`-handling-to-disk patterns, and `redis.set("*cookie*"` patterns; BLOCKER.

### CI-INV-07 — Rotation without downtime

**Statement.** Credential rotation (per the `runbooks/credential-rotation.md` procedure) must NOT cause tenant downtime. Old + new credentials may both be valid for a configurable overlap window (default 60 s); the adapter prefers the new credential as soon as it is observed in OpenBao.

**Conformance.** `tests/integration/rotation_zero_downtime.rs` rotates a credential under load (1000 RPS) and asserts zero failed requests; recorded in `evidence/runbook-drills/rotation/`.

### CI-INV-08 — No credential bytes in test fixtures

**Statement.** No test fixture, sample script, ADR snippet, README example, or documentation example contains real or synthetic vendor-shaped credentials. Examples reference SecretReference URIs only.

**Conformance.** `oya-check-no-raw-credentials` extends its regex sweep to `tests/`, `docs/`, ADRs, and PRD snippets; BLOCKER.

### CI-INV-09 — 2-person rule for adapter publish

**Statement.** Any new adapter crate publish (or version bump of an existing adapter) requires two-person review: one axis-foundry reviewer + one ops-security reviewer. The CODEOWNERS file at `microservices/intelligence/CODEOWNERS` encodes the rule.

**Conformance.** Branch protection rule on the `foundry-providers/src/crates/oya-foundry-providers-adapter-*` paths requires 2 reviews from distinct teams.

### CI-INV-10 — Audit-chain emission on every credential resolve

**Statement.** Every `OpenBao.resolve(SecretReference)` call emits a `CredentialResolved(tenant, vendor, lease_id, caller_ctx_hash)` event to the audit-chain; unusual resolution patterns surface in `observability` via the `oya_foundry_providers_credential_resolutions_total` rolling counter.

**Conformance.** `tests/integration/credential_audit_emission.rs` verifies the event is emitted on each resolution.

## Process Discipline

| Practice | Mandatory? |
|---|---|
| Adapter publish requires 2-person review (axis-foundry + ops-security) | Yes (CI-INV-09) |
| Credential rotation drill quarterly per pack per vendor | Yes |
| Annual red-team exercise targeting T-01 (credential theft) | Yes (per threat-model.md) |
| Adapter-version pin runbook practiced after every vendor breaking change | Yes |
| No agent (Claude / Codex / Gemini) is ever shown a raw credential in any context | Yes — this is the durable user directive 2026-05-12 |

## Cedar policy fragment

The OpenBao policy that issues tokens to foundry-providers pods is in `microservices/cloud-secrets/policy/foundry-providers.hcl`; the Cedar fragment that gates tenant-operator credential management is in `microservices/intelligence/policy/openbao-credential.cedar`.

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=credential-isolation --microservice foundry-providers` exits 0.
- Per-adapter test `tests/integration/<vendor>_no_credential_leak.rs` passes.
- Quarterly rotation drill: `evidence/runbook-drills/rotation/<unix_ts>.json` present.

## References

- ADR-0025 — foundry-as-engineering-platform.
- Durable user directive 2026-05-12 — OpenBao canonical SecretReference path.
- `microservices/intelligence/threat-model.md` T-01.
- `microservices/intelligence/runbooks/credential-rotation.md`.
- OpenBao docs — `openbao.org/docs`.
- Zeroize crate — `docs.rs/zeroize`.
