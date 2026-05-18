---
doc_class: ImplementationPlan
impl_plan_id: IP-008-sealing-adapter-hsm
status: pending
owner: cloud-secrets + axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, layer-correctness, audit-chain-hsm-iam-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: oya-audit-chain-sealing-adapter-hsm

## Intent

PKCS#11 / KMIP adapter for OCI Cloud-HSM. Implements `SignerPort` from sealing-kernel. Per-pack HSM partition handle resolved from OpenBao at startup. Private key never enters process memory.

## Crate Naming

`oya-audit-chain-sealing-adapter-hsm` per ADR-0105 Amendment 3 `*-adapter-<backend>` pattern; backend=`hsm`.

## Concrete File Targets

| Path | Action |
|---|---|
| `.../src/crates/oya-audit-chain-sealing-adapter-hsm/Cargo.toml` | create — dep `cryptoki` for PKCS#11 |
| `.../src/lib.rs` | create |
| `.../src/session.rs` | create — PKCS#11 session lifecycle; short-lived (≤ 24h cert) |
| `.../src/signing.rs` | create — `sign(root_hash) -> Signature` via PKCS#11 Ed25519 mechanism |
| `.../src/key_resolver.rs` | create — resolve pack → partition handle → key handle from OpenBao |
| `.../tests/sign_correctness.rs` | create — sign + locally-verify; mismatch = fail |
| `.../tests/integration_softhsm.rs` | create — CI runs against SoftHSM stub with equivalent PKCS#11 interface |

## Code Shape

```rust
// signing.rs
pub struct HsmSigner {
    session: cryptoki::session::Session,
    key_handle: cryptoki::object::ObjectHandle,
    public_key_fp: KeyFingerprint,
}

#[async_trait]
impl SignerPort for HsmSigner {
    async fn sign(&self, root_hash: &[u8; 32]) -> Result<Signature, KernelError> {
        let mechanism = Mechanism::Eddsa;
        let signature = self.session.sign(&mechanism, self.key_handle, root_hash)
            .map_err(KernelError::HsmSigningError)?;

        // Load-bearing local-verify per policy/seal-integrity.md FM-SI-02
        let public_key = self.session.get_public_key(self.key_handle)?;
        if !ed25519_dalek::verify(public_key, root_hash, &signature) {
            return Err(KernelError::HsmSigningMismatch);
        }

        Ok(Signature(signature))
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-audit-chain-sealing-adapter-hsm
cargo nextest run -p oya-audit-chain-sealing-adapter-hsm --features integration-softhsm
cargo run -p oya-dev-cli -- gate validate audit-chain-hsm-iam-conformance
```

## Halt Conditions

- Sign + local-verify mismatch — block; chain-integrity-suspect.
- PKCS#11 session not SPIFFE-bound — block; T-S-02 threat.
- Private key extraction succeeds (it should not be possible) — fundamental control failure.

## References

- Bominal ADR-0028 §"HSM signing".
- ISO 27001 A.5.17 + A.8.24.
- OCI Cloud-HSM PKCS#11 docs.
- `cryptoki` Rust PKCS#11 binding docs.
- RFC 8032 Ed25519 spec.
