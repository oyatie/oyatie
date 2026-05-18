---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-011-hsm-integration-adapter-hsm
status: pending
owner: axis-cloud-secrets + ops-security
acceptance_lanes: [hsm-pkcs11-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: hsm-integration adapter-hsm

## Intent

Integrate with OCI Cloud-HSM (default) and Thales Luna (pack-kr) via PKCS#11 + KMIP. Provide signing operations, attestation, KEK generation in HSM.

## ChangeSet boundary

Five new crates: kernel, usecase, api, adapter-hsm, app. The `adapter-hsm` is backend-qualified per ADR-0105 Amendment 3.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-hsm-integration-kernel/` | `HsmPartition`, `KekHandle`, `AttestationReport` |
| `…/oya-cloud-secrets-hsm-integration-usecase/` | orchestrate sign + attest + KEK-generate |
| `…/oya-cloud-secrets-hsm-integration-api/` | typed contracts |
| `…/oya-cloud-secrets-hsm-integration-adapter-hsm/` | PKCS#11 binding (cryptoki crate); KMIP client |
| `…/oya-cloud-secrets-hsm-integration-app/` | binary + attestation cron |
| 5× catalog yamls | create |

## Code Shape

```rust
#[async_trait]
pub trait HsmPartitionClient: Send + Sync + Sealed {
    async fn sign(&self, alias: &KekAlias, payload: &[u8], alg: SignAlgorithm) -> Result<Signature, HsmError>;
    async fn generate_kek(&self, alias: &KekAlias, alg: KekAlgorithm, witnesses: [Spiffe; 2]) -> Result<(), HsmError>;
    async fn rotate_kek(&self, from: &KekAlias, to: &KekAlias, witnesses: [Spiffe; 2]) -> Result<(), HsmError>;
    async fn attest(&self) -> Result<AttestationReport, HsmError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p 'oya-cloud-secrets-hsm-integration-*'
# PKCS#11 smoke against OCI Cloud-HSM (CI staging)
cargo nextest run -p oya-cloud-secrets-hsm-integration-adapter-hsm --features pkcs11-smoke
```

## Test Plan

- PKCS#11 smoke against vendor emulator (SoftHSM) + against real OCI Cloud-HSM in staging.
- Attestation verification against vendor attestation chain.
- 4-eye KEK ceremony: refuse generation without 2 witness SPIFFE IDs.

## Halt Conditions

- KEK material extractable outside HSM — BLOCKER.
- Sign without 4-eye witness for KEK rotation — BLOCKER.

## Next IP

`IP-012-per-tenant-namespace-controller.md`
