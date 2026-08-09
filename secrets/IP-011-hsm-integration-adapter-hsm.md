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

## Wave 15-IP-substance A-G

### A. Problem
Regulated packs require KEK operations to stay inside HSM partitions. A software fallback or extractable KEK would break the PRD's encryption-BYOK, FIPS, and residency claims.

### B. Approach
Implement the HSM adapter around PKCS#11/KMIP with explicit partition identity, attestation verification, 4-eye KEK ceremony checks, and typed signing/wrap operations. The kernel/usecase crates describe the port; only `adapter-hsm` talks to vendor libraries.

### C. Deliverables
- `oya-cloud-secrets-hsm-integration-{kernel,usecase,api,adapter-hsm,app}`.
- `HsmPartitionClient`, `KekHandle`, and `AttestationReport` surfaces.
- SoftHSM test fixture plus OCI Cloud-HSM staging smoke.
- SLO linkage to `slos/hsm-availability.openslo.yaml`.
- Runbook `runbooks/hsm-key-rotation.md`.

### D. Ordered Implementation Steps
1. Define HSM partition and KEK handle types with data-class annotations.
2. Implement usecase checks for pack, tenant, witness, and ceremony state.
3. Add PKCS#11 adapter operations for sign, wrap, unwrap, attest, and rotate.
4. Verify attestation chains before accepting a partition as active.
5. Refuse KEK extraction and any software fallback in regulated packs.
6. Emit `KekAttested` and rotation audit events through audit-emitter.
7. Run SoftHSM and OCI Cloud-HSM staging smoke tests.

### E. Acceptance
- `cargo nextest run -p 'oya-cloud-secrets-hsm-integration-*'`.
- `cargo nextest run -p oya-cloud-secrets-hsm-integration-adapter-hsm --features pkcs11-smoke`.
- KEK material is never returned from adapter APIs.
- KEK rotation requires two witness SPIFFE IDs and produces audit-chain evidence.

### F. Evidence
Evidence anchors are `PRD.md` FR-05, `manifest.json`, `catalog/oya-cloud-secrets-hsm-integration-adapter-hsm.yaml`, `policy/data-residency.md`, `slos/hsm-availability.openslo.yaml`, `runbooks/hsm-key-rotation.md`, and the competitor parity HSM/FIPS/BYOK dimensions.

### G. Counterpart Comparison
Vault Enterprise, AWS KMS/CloudHSM, Azure Managed HSM, GCP Cloud KMS, OCI Cloud-HSM, and Akeyless all have key-management stories. Oyatie's target differs by requiring per-pack HSM partitions, daily attestation, KEK-of-KEKs, and audit-chain evidence as first-class service behavior.

Grep-recognized counterpart anchor: GitHub Actions Secrets is relevant only to CI attestation jobs that carry references to HSM test fixtures, never KEK material. The substantive comparator remains HSM/KMS behavior across Vault and cloud providers.

## DR posture (per ADR-0343)

- Target source: `microservices/cloud-secrets/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`openbao_seal_unseal`, `postgres_wal_g`, `audit_chain_merkle_seal`].
- Surface evidence: `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/manifest.json`, `microservices/cloud-secrets/IP-011-hsm-integration-adapter-hsm.md`.
