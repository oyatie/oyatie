---
doc_class: ImplementationPlan
impl_plan_id: IP-011-verification-stack
status: pending
owner: axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, layer-correctness, port-location]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: verification BC (7 crates)

## Intent

Full verifier stack: kernel + domain + usecase + api + adapter + rest + sdk. KeyResolver respects key-rotation epochs; verification is pure-function over published artifacts; SDK is offline-verifiable reference implementation.

## Crates introduced (7)

- `oya-audit-chain-verification-kernel`
- `oya-audit-chain-verification-domain`
- `oya-audit-chain-verification-usecase`
- `oya-audit-chain-verification-api`
- `oya-audit-chain-verification-adapter`
- `oya-audit-chain-verification-rest`
- `oya-audit-chain-verification-sdk`

## Key code (domain)

```rust
// verification-domain/src/lib.rs
pub fn verify(
    envelope: &EventEnvelope,
    proof: &MerkleProof,
    signed_root: &SignedRoot,
    public_key: &Ed25519PublicKey,
) -> Verdict {
    // 1. Verify Ed25519 signature
    if !ed25519_dalek::verify(public_key, &signed_root.root_hash, &signed_root.signature) {
        return Verdict::Failed(VerifyReason::SignatureInvalid);
    }

    // 2. Compute leaf_hash from envelope (canonical-serialise + RFC 6962 leaf prefix)
    let leaf = leaf_hash(&envelope.canonical_serialize());

    // 3. Walk Merkle proof; must reach signed_root.root_hash
    let computed_root = walk_proof(leaf, proof);
    if computed_root != signed_root.root_hash {
        return Verdict::Failed(VerifyReason::ProofInvalid);
    }

    // 4. Verify chain link: signed_root.prior_root_hash must reference a published prior root
    // (chain integrity verification; recursive ≤ ceil(log2(periods_since_genesis)))
    // Optional: caller may skip if they trust signed_root in isolation
    // Default: verify chain to depth N or to genesis

    Verdict::Ok
}
```

## Test plan

Property test: 10k random `EventEnvelope`s × 10k random mutations (mutate payload byte, mutate proof sibling, mutate root, mutate signature, mutate signer key). Every mutation must classify `verified=false` with correct reason.

## Acceptance Gates

```bash
cargo nextest run -p oya-audit-chain-verification-domain
cargo nextest run -p oya-audit-chain-verification-domain --features proptest -- proptest_tamper
cargo nextest run -p oya-audit-chain-verification-sdk --features integration
```

## References

- `microservices/audit-chain/policy/seal-integrity.md` §"SI-13..SI-15".
- `microservices/audit-chain/capabilities/verify-merkle.yaml`.
- RFC 6962 + RFC 8032.
