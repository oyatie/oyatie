---
doc_class: SDKPlan
microservice: cloud-secrets
status: Accepted
date: 2026-05-17
owner_team: axis-cloud-secrets
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/cloud-secrets/contracts/proto/cloud-secrets.proto
  - microservices/cloud-secrets/policy/secret-isolation.md
doc_status: published
---

# SDK Plan: SecretReference SDK (cloud-secrets)

## Purpose

Define the SecretReference SDK that every oyatie µservice imports to consume secrets via `${openbao:secret/<path>}` references. The SDK is **the primary integration surface** of cloud-secrets — REST is admin-only; resolution always flows through SDK.

## Languages

| Language | Crate / package | Binding | Status |
|---|---|---|---|
| Rust (canonical) | `cloud-secrets-secret-reference-resolver-sdk` | native | M01 launch |
| TypeScript | `@oyatie/cloud-secrets-sdk` | napi-rs (Rust core, Node binding) | M01 launch |
| Python | `oyatie-cloud-secrets` | pyo3 (Rust core, Python binding) | M01 launch |
| Go (future) | `github.com/oyatie/cloud-secrets-go` | cgo wrapping Rust core OR native gRPC | subsequent-to-M01-completion |
| Java/Kotlin (future) | `dev.oyatie:cloud-secrets-sdk` | JNI wrapping Rust core OR native gRPC | subsequent-to-M01-completion |

The Rust core is the canonical implementation; bindings ensure protocol fidelity (revocation push, cache semantics, audit emission).

## Core API surface

### Rust

```rust
use cloud_secrets_secret_reference_resolver_sdk::{
    SecretClient, SecretReference, Secret,
};

let client = SecretClient::builder()
    .endpoint("openbao-kr.oyatie.dev:8200")
    .spiffe_identity_from_kubernetes()  // auto-detected
    .build()
    .await?;

// Resolve a reference; returned value is Secret<T> — zeroised on drop
let secret_ref = SecretReference::parse(
    "openbao:secret/tenant:a1b2c3d4e5f6g7h8/workflow-engine/oauth-client-secret",
)?;

let value: Secret<String> = client.resolve(&secret_ref).await?;

// Use the secret via scoped callback; value is masked after callback returns
client.with_secret(&secret_ref, |s| {
    // s: &str  — usable within this closure only
    http_client.set_bearer(s);
    http_client.fetch(...)?;
    Ok(())
}).await?;

// Subscribe to revocation push (typically spawned at boot)
let mut revocations = client.subscribe_revocations().await?;
while let Some(event) = revocations.next().await {
    // SDK has already flushed cache for event.secret_path_hash
    // Consumer may take action (e.g., re-resolve eagerly)
}
```

### TypeScript (via napi-rs)

```typescript
import {
  SecretClient,
  SecretReference,
  Secret,
} from '@oyatie/cloud-secrets-sdk';

const client = await SecretClient.build({
  endpoint: 'openbao-kr.oyatie.dev:8200',
  spiffeIdentityFromKubernetes: true,
});

const secretRef = SecretReference.parse(
  'openbao:secret/tenant:a1b2c3d4e5f6g7h8/workflow-engine/oauth-client-secret',
);

await client.withSecret(secretRef, async (value: string) => {
  // value is usable within this callback only
  return httpClient.fetch(url, {
    headers: { Authorization: `Bearer ${value}` },
  });
});
```

### Python (via pyo3)

```python
from oyatie_cloud_secrets import SecretClient, SecretReference

client = SecretClient.build(
    endpoint="openbao-kr.oyatie.dev:8200",
    spiffe_identity_from_kubernetes=True,
)

secret_ref = SecretReference.parse(
    "openbao:secret/tenant:a1b2c3d4e5f6g7h8/workflow-engine/oauth-client-secret",
)

with client.with_secret(secret_ref) as value:
    # value usable within `with` block only
    requests.get(url, headers={"Authorization": f"Bearer {value}"})
```

## Required SDK Behaviour Contract

Every SDK MUST honour the following contract; CI lane `check-sdk-contract-conformance` validates against a reference test set.

### C-01: SecretReference URI parsing

ABNF:

```
SecretReferenceURI = "openbao:secret/" path [ "@" version ]
path               = segment *( "/" segment )
segment            = 1*( ALPHA / DIGIT / "_" / "-" / "." / ":" )
version            = "v" 1*DIGIT
```

Parser refuses any deviation; security boundary.

### C-02: Secret<T> wrapper

Resolved values MUST be wrapped:
- Rust: `Secret<T: Zeroize>` with `Debug` returning `"[REDACTED]"`; `Display` not implemented; `Drop` zeroises.
- TS: `Secret<T>` class with `toString()` / `toJSON()` returning `'[REDACTED]'`; backing buffer zeroised on `dispose()`.
- Python: `Secret` context-manager; raw value accessible only inside `with` block.

### C-03: Cache TTL ≤60s

In-process LRU cache MUST clamp TTL to ≤60s. Server-supplied TTL hint may be lower but never higher.

### C-04: Revocation push consumption

SDK MUST open + maintain a server-sent-events stream for revocation push. On disconnect: auto-reconnect with backoff (1s → 2s → 4s → ... → 60s capped); on reconnect: server replays `revocations since <last-seen-event-id>`. If reconnect window exceeds server ring buffer: full cache flush.

### C-05: Audit emission acknowledgement

SDK acknowledges that every resolve emits a `SecretAccessed` event audit on the server-side. SDK MUST NOT bypass; offline mode is NOT supported.

### C-06: SPIFFE workload identity

SDK auto-detects SPIFFE SVID from the workload's Kubernetes ServiceAccount + cert-manager-issued cert. Manual override is supported only for testing.

### C-07: Constant-time comparison

Cache key lookup MUST use constant-time comparison (`subtle::ConstantTimeEq` or equivalent) to defend against timing-side-channel.

### C-08: No log emission

SDK MUST NOT log the resolved value. Debug logs MAY log the reference URI but never the value. Error logs MUST NOT echo the value or its prefix.

### C-09: HMAC validation

Cache values carry an HMAC computed over `(reference_uri, version, resolved_at)`. SDK validates HMAC on every cache-hit; mismatch evicts the entry + audit-emits `cache_integrity_mismatch`.

### C-10: Drop / dispose semantics

When `Secret<T>` is dropped:
- Rust: `Drop` impl calls `zeroize::zeroize` on the inner buffer.
- TS: explicit `dispose()` zeroises the backing typed-array; SDK provides automatic disposal via WeakRef / FinalizationRegistry where available.
- Python: `__exit__` zeroises the underlying bytes buffer.

## Versioning + deprecation

- Semver: major.minor.patch.
- LTS: every odd-minor (1.1, 1.3, 1.5, ...) is LTS for 6 months past next-odd-minor release.
- Sunset window: 6 months between deprecation announcement and EOL.
- CI lane `check-sdk-version-pin` refuses any consumer crate pinning an SDK version > 1 minor behind current.

## Performance budget

| Operation | p99 budget |
|---|---|
| `client.resolve(ref)` cache-hit | ≤10 ms |
| `client.resolve(ref)` cache-miss | ≤25 ms |
| `client.with_secret(ref, f)` overhead (excluding f) | ≤2 ms (Rust); ≤5 ms (TS/Python via FFI) |
| `client.subscribe_revocations()` event-handler latency | ≤1 ms from server emit |
| SDK boot time | ≤500 ms (TLS handshake + SVID load) |

Bench suite at `microservices/cloud-secrets/tests/bench/sdk-resolution-latency.rs`.

## Test plan

| Test class | Coverage | Tooling |
|---|---|---|
| Unit | URI parsing, cache TTL clamp, HMAC validation | `cargo test` / `npm test` / `pytest` |
| Integration | Resolve against sandbox OpenBao | testcontainers OpenBao 2.x |
| Cross-language smoke | Same SecretReference resolves to same value in Rust / TS / Python | shared fixture |
| Property | URI parser refuses malformed inputs | `proptest` (Rust) / `fast-check` (TS) / `hypothesis` (Python) |
| Performance bench | p99 budgets | criterion (Rust); benchmark.js (TS); pytest-benchmark (Python) |
| Chaos | Revocation push replay after disconnect | toxiproxy-driven SSE drops |

## Roadmap

- M01: Rust + TS + Python at v1.0.
- M02: Go SDK (native gRPC client).
- M03: Java/Kotlin SDK.
- M03: Tenant-side SDK for encryption-key BYOK upload from tenant CLI tools (ADR-0251 §D-10).
- Post-M03: WebAssembly target (Rust compiled to WASM for browser-side use in tenant admin tooling — strictly read-metadata; never resolve).

## References

- `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`
- `microservices/cloud-secrets/policy/secret-isolation.md`
- `microservices/cloud-secrets/PRD.md` FR-01, FR-02, FR-08
- Bominal ADR-0028 (audit-chain + data-class taxonomy)
- OpenBao SDK (informing patterns)
- AWS SDK for Secrets Manager (informing wrapper patterns)
