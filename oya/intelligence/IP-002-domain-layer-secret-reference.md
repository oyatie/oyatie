---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-intelligence-two-layer-mvp
impl_plan_id: IP-002-domain-layer-secret-reference
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-intelligence + ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, oya-governance-per-microservice-layout]
---

# IP-002: Domain layer — SecretReference value object

## Intent

Author the `SecretReference` value object + `CredentialHandle` opaque handle type in the
`oya-intelligence-credential-resolver-domain` crate, per ADR-0255 §D-4 (provider-BYOK) and ADR-0296
(sidecar credential-handle).

## ChangeSet boundary

New crate `oya-intelligence-credential-resolver-domain` under
`microservices/intelligence/crates/`. No I/O. The handle is a marker type with no `.value()`
accessor; only the sidecar can resolve to a credential.

## Concrete file targets

| Path | Action |
|---|---|
| `microservices/intelligence/crates/oya-intelligence-credential-resolver-domain/Cargo.toml` | create |
| `microservices/intelligence/crates/oya-intelligence-credential-resolver-domain/src/lib.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-credential-resolver-domain/src/secret_reference.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-credential-resolver-domain/src/credential_handle.rs` | create |
| `microservices/intelligence/crates/oya-intelligence-credential-resolver-domain/tests/handle_opacity.rs` | create |

## Code shape

```rust
pub enum SecretReferenceKind {
    OpenbaoPath,
    PlatformDefault,
}

pub struct SecretReference {
    pub kind: SecretReferenceKind,
    pub value: Option<String>,        // "${openbao:secret/<tenant>/<provider>}" for OpenbaoPath
    pub bound_tenant: Option<TenantId>,
}

/// Opaque short-lived credential handle. There is NO accessor for the underlying credential;
/// only the sidecar (via Unix domain socket) can inject it at HTTP-call assembly time per ADR-0296.
pub struct CredentialHandle {
    handle_id: HandleId,                 // private; opaque
    bound_tenant: TenantId,
    bound_provider: Provider,
    bound_audience: Audience,
    expires_at: SystemTime,
    signature: Ed25519Signature,
}

impl CredentialHandle {
    pub fn handle_id(&self) -> &HandleId { &self.handle_id }
    pub fn bound_tenant(&self) -> &TenantId { &self.bound_tenant }
    pub fn bound_provider(&self) -> Provider { self.bound_provider }
    pub fn bound_audience(&self) -> Audience { self.bound_audience }
    pub fn expires_at(&self) -> SystemTime { self.expires_at }
    pub fn signature(&self) -> &Ed25519Signature { &self.signature }
    pub fn is_expired(&self, now: SystemTime) -> bool { now >= self.expires_at }

    /// NO `value()` accessor. The underlying secret is never accessible to Rust code; it lives
    /// only in the OpenBao sidecar's memory.
}
```

## Acceptance gates

```bash
cargo check  -p oya-intelligence-credential-resolver-domain
cargo clippy -p oya-intelligence-credential-resolver-domain -- -D warnings
cargo nextest run -p oya-intelligence-credential-resolver-domain
```

## Test plan

- Compile-time assertion: `CredentialHandle` has no accessor returning the underlying credential
  (verified by ast-grep test in `tests/handle_opacity.rs`).
- `is_expired` correctness across boundary cases.

## Halt conditions

- Any PR that adds a `value()`-style accessor on `CredentialHandle` fails the
  `oya-governance-credential-handle-opacity` lane.

## Next IP

[`IP-003-domain-layer-refusal-decision.md`](IP-003-domain-layer-refusal-decision.md)

## References

- ADR-0255 §D-4 (provider-BYOK), ADR-0296 (sidecar credential-handle).
- `microservices/intelligence/threat-model.md` T-I-02.
