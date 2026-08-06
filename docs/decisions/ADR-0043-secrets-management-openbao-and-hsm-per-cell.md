---
id: ADR-0043
status: Rejected
doc_status: published
---

# ADR-0043: Secrets management — OpenBao (MPL-2; supersedes Vault BUSL), per-tenant per-cell HSM partition (KCminimum-shippable-tier + FIPS 140-3), per-capability SecretProvider

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0011, ADR-0028, ADR-0029, ADR-0036, ADR-0038, ADR-0039

---

## Context

Secrets management touches every axis: per-tenant API keys for Foundry adapters, per-cell HSM partitions for KMS, per-capability rotating tokens for subscription-mode AI providers, signing keys for the Trust framework's proof-of-erasure (per ADR-0038), TLS certs for the service mesh (per ADR-0044). The pack-of-19 foundation ADRs named secrets management as a need but did not pin (a) the secrets-store, (b) the HSM topology, (c) the per-capability provider model, (d) the rotation cadence, (e) the emergency rotation runbook.

The license dimension is sharp: HashiCorp Vault flipped to BUSL in 2023; OpenBao is the MPL-2 fork (LF Edge) and is license-clean for our product. The HSM dimension is regional: KR-launch requires KCminimum-shippable-tier (KR Cryptographic Module Validation Program) for any cell handling regulated data; global cells require FIPS 140-3. This ADR pins both.

---

## Decision

We adopt **OpenBao** (MPL-2) as the canonical secrets store; **per-tenant per-cell HSM partition** with **KCminimum-shippable-tier for KR cells + FIPS 140-3 globally**; a **rotating session-token vault** for Foundry subscription-mode adapters; a **per-capability `SecretProvider` trait** so axes never read raw secrets; **quarterly key-rotation drill** per cell; an **emergency rotation runbook** for compromise scenarios.

### OpenBao (MPL-2) supersedes Vault (BUSL)

```rust
// crates/oya-secrets-openbao
pub struct OpenBaoClient {
    pub addr: VaultAddress,             // per-cell endpoint
    pub auth: OpenBaoAuth,              // OIDC / Kubernetes auth method
    pub token_ttl: Duration,            // short-lived
    pub renewal: TokenRenewer,
}
```

- OpenBao is the LF Edge fork of Vault; binary-compatible with Vault 1.14 API; MPL-2 (clean for our product surface).
- Per-cell deployment; HA cluster of 3 or 5 nodes per cell.
- Auto-unseal via per-cell HSM partition (Shamir secret sharing for the unseal key, sealed in HSM).

### Per-tenant per-cell HSM partition

Each cell owns one or more HSM partitions; per-tenant key material lives in a tenant-bound partition.

| Cell type | HSM | Validation |
|---|---|---|
| KR primary cell (KR-Seoul1) | Thales Luna 7 (Phase 1 OCI managed); LG U+ network HSM (Phase 2 colo) | KCminimum-shippable-tier-validated module + FIPS 140-3 Level 3 |
| KR secondary cell (KR-Chuncheon) | Thales Luna 7 | KCminimum-shippable-tier + FIPS 140-3 Level 3 |
| Global cell | AWS CloudHSM (Phase 1); per-region commercial HSM (Phase 2) | FIPS 140-3 Level 3 |
| Healthcare cell | dedicated partition with K8 KEK rotation | KCminimum-shippable-tier + FIPS 140-3 Level 3 + ISO 27018 |
| Fintech cell | dedicated partition with K8 KEK rotation | KCminimum-shippable-tier + FIPS 140-3 Level 3 + PCI HSM |

Per-tenant KEK (Key Encryption Key) wraps:

- Per-tenant DEKs (Data Encryption Keys) for Workspace Mail body / Drive object / Meet recording (per ADR-0029).
- Per-tenant signing keys for proof-of-erasure (per ADR-0038).
- Per-tenant TLS private keys for tenant-domain mail (per ADR-0029).

Tenant deletion (or DSR erase) shreds the per-tenant KEK; all DEKs become unrecoverable (per ADR-0029 KMS-shred guarantee).

### Rotating session-token vault for Foundry subscription-mode adapters

Foundry adapters that authenticate to external AI providers (per ADR-0026) often use long-lived API keys. The session-token vault is the indirection layer:

```rust
// crates/oya-secrets-session-token-kernel
pub struct SessionTokenVault {
    pub provider_keys: BTreeMap<ProviderId, ProviderKeySet>,  // sealed in OpenBao
    pub session_issuer: SessionIssuer,
}

// Adapter never sees the long-lived key
let token = vault.issue_session(provider_id, ttl: Duration::minutes(15)).await?;
```

- Long-lived provider keys never leave OpenBao.
- Per-invocation short-lived session tokens issued; TTL ≤ 15 min.
- Per-session token usage audit-chained per ADR-0003.

### Per-capability `SecretProvider` trait

```rust
// crates/oya-secrets-openbao/src/provider.rs
pub trait SecretProvider {
    async fn get(&self, secret_ref: SecretRef) -> Result<SecretValue>;
    async fn rotate(&self, secret_ref: SecretRef) -> Result<RotationResult>;
    async fn revoke(&self, secret_ref: SecretRef) -> Result<()>;
}

pub enum SecretRef {
    Tenant { tenant_id: TenantId, key: String },
    Capability { capability_id: CapabilityId, key: String },
    Plugin { plugin_id: PluginId, tenant_id: TenantId, key: String },
    System { component: String, key: String },
}
```

Axes never read OpenBao directly; they receive a `SecretProvider` from the runtime, scoped to the axis's allowed `SecretRef` patterns. Cedar policy (ADR-0007) gates each `get` / `rotate` / `revoke`.

### Quarterly key-rotation drill per cell

Every quarter, per cell:

1. Initiate rotation of per-tenant KEKs in non-production tenants (rotate, validate decryption with old + new, retire old).
2. Initiate rotation of per-capability session-token issuers (rotate signing key; old tokens retire on TTL).
3. Initiate rotation of internal mTLS CA (per ADR-0044 service mesh).
4. Audit rotation outcomes; per-tenant proof-of-rotation emitted to the audit chain.
5. Update trust portal (per ADR-0038) with rotation evidence.

### Emergency rotation runbook

Triggered by: compromise indicator (audit-chain anomaly per ADR-0003), per-cell intrusion alert (per ADR-0042 observability), per-vendor compromise notification (e.g. CA breach, HSM firmware advisory), per-author key compromise (per ADR-0039 signed commits).

Runbook (per `docs/runbooks/`):

1. Quarantine compromised scope (cell / tenant / capability / author).
2. Issue out-of-band rotation order (operator-authenticated via per-cell HSM).
3. Rotate KEKs in compromised scope; invalidate all DEKs derived from old KEK.
4. Rotate session-token issuer.
5. Rotate mTLS CA.
6. Rotate signing keys.
7. Per-tenant trust-portal advisory + per-tenant DPO notification.
8. Audit-chain seal of the incident.
9. Postmortem-doctrine-replacement: per the prevention doctrine, lane runs a fix-the-system pass.

### Per-axis secret hygiene rules

- No secret in code (Trivy scan per ADR-0039 enforces).
- No secret in env (use `SecretProvider`).
- No secret in container image (build-time scan).
- No secret in config files (use `secret://` URIs that resolve via `SecretProvider`).
- No secret printed to logs (per `tracing` filter rules per ADR-0042).

### Anti-scope

This ADR does not own per-tenant identity (per ADR-0002). Does not own audit chain (per ADR-0003). Does not own per-capability autonomy ceiling (per ADR-0007). Does not own service mesh mTLS issuance (per ADR-0044, but ADR-0044 consumes this ADR's CA).

---

## Consequences

### Positive

- OpenBao adoption resolves the BUSL license issue with Vault.
- Per-tenant per-cell HSM partition + KCminimum-shippable-tier / FIPS 140-3 satisfies KR + global regulatory bars in one architecture.
- Per-capability `SecretProvider` trait means axes never see raw secrets; secret hygiene is enforced by interface, not by review.
- Session-token vault gives subscription-mode adapters per-invocation rotation without per-call provider re-authentication latency.
- Quarterly drill validates rotation works before we need it in anger.

### Negative

- HSM cost is real (per-partition per-cell licensing + hardware lease).
- KCminimum-shippable-tier validation is a one-time-per-module-version expensive process; HSM upgrades carry validation cost.
- Per-cell HSM topology multiplies operational surface.
- Emergency rotation requires out-of-band operator authentication infrastructure (which itself must be HSM-backed).

### Operational

- Per-cell HSM health alarmed; per-cell HSM partition utilization > 80% triggers capacity review.
- Per-tenant key rotation completion tracked.
- Per-quarter drill outcome audit-chained.
- Per-cell secret-leak detection (Trivy + per-microservice scanners) wired to alarms.
- Annual KCminimum-shippable-tier / FIPS audit per cell.
- Per-region HSM vendor relationship review.

---

## Alternatives considered

### Alternative A — HashiCorp Vault (BUSL)

- **Pros:** mature; large community.
- **Cons:** BUSL forbidden in product surface per License Policy.
- **Rejected because:** license posture incompatible.

### Alternative B — Cloud-provider KMS (OCI Vault / AWS KMS) only, no in-cell HSM

- **Pros:** managed; less ops.
- **Cons:** per-tenant residency commitments require us to control the HSM partition; cloud-provider KMS does not satisfy KCminimum-shippable-tier module validation requirements without specific configurations.
- **Rejected because:** per-tenant per-cell HSM partition is the regulator-defensible posture.

### Alternative C — Per-axis secret store (each axis runs its own)

- **Pros:** microservice-team independence.
- **Cons:** N stores; per-store drift; per-rotation-drill multiplied; cohesion violated.
- **Rejected because:** secrets are a substrate concern.

### Alternative D — No per-capability `SecretProvider` trait; raw OpenBao access from every axis

- **Pros:** simpler.
- **Cons:** secret hygiene becomes per-author discipline; secret leaks happen in unaudited code paths.
- **Rejected because:** the trait surface is the enforcement mechanism.

---

## Open questions

1. **Q1.** Per-tenant KEK rotation cadence — quarterly or annual? Default: quarterly for regulated tenants (HC / FIN / PUB); annual for general SaaS. → ADR-0034.
2. **Q2.** Per-cell HSM vendor diversity — single vendor or multi-vendor? Default: single vendor per cell at GA; multi-vendor per region at Phase 2. → ADR-0028.
3. **Q3.** Session-token TTL default — 15 min or 5 min? Default: 15 min; 5 min for `proxy`-tier autonomy actions per ADR-0007. → ADR-0007.
4. **Q4.** Out-of-band emergency rotation — break-glass YubiHSM or per-cell tamper-evident binder? Default: YubiHSM per operator + tamper-evident binder for cell-recovery secret; both required (M-of-N). → owner: `foundry`.
5. **Q5.** Per-tenant encryption-BYOK (Bring Your Own Encryption Key) at GA, or W+12? Default: W+12 (regulated tenants); GA preview only. → ADR-0034.

---

## References

- `docs/PRD.md` §10 (security program), §11 (per-tenant residency)
- `docs/DESIGN.md` §11 (secrets management), §10 (cross-microservice contracts)
- KR KCminimum-shippable-tier (Korean Cryptographic Module Validation Program) — KISA guidance
- FIPS 140-3 (NIST); ISO 27018 (cloud privacy); PCI HSM standards
- OpenBao docs (LF Edge); OpenBao migration guide from Vault
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0007 (Cedar + persona tier), ADR-0011 (capability registry), ADR-0028 (cloud), ADR-0029 (workspace KMS-shred), ADR-0036 (plugin), ADR-0038 (trust portal), ADR-0039 (supply chain), ADR-0044 (service mesh mTLS)
