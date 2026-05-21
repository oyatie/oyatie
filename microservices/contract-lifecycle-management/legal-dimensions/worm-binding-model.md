---
doc_class: LegalDimension
microservice: contract-lifecycle-management
dimension_id: L-016
authoritative_source: SEC 17a-4(f) + FINRA 4511 + SEC 31a-2 (Investment Advisers Act)
related_packs: [sec-17a-4, sox-404]
date: 2026-05-21
---

# WORM (Write Once, Read Many) Binding Model

WORM storage is required for certain regulated record classes. The 2023-amended SEC 17a-4(f) permits the audit-trail-system alternative, but many regulated tenants still prefer WORM for evidence-strength reasons.

## Canonical WORM binding per deployment context

### oyatie-public-cloud + aws-guest

**Primary: AWS S3 Object Lock in Compliance mode**

- Bucket created with `ObjectLockEnabled = true`.
- Retention applied per object with `Mode = COMPLIANCE` (cannot be overridden by root user).
- Retention period set to the maximum applicable retention from `legal-dimensions/retention-overlay-by-contract-type.md`.
- MFA Delete enabled on the bucket.
- KMS key policy denies key deletion while objects retain.

**Secondary (long-archive): AWS S3 Glacier Vault Lock**

- Vault lock policy locked at vault creation; cannot be modified.
- Retention period embedded in the lock policy.

### oci-guest

**Primary: OCI Object Storage Retention Lock in time-bound mode**

- Bucket created with `retention rules` configured.
- Time-bound retention rule locked (not deletable for the retention period).
- Lock-mode retention applied at object upload.

### on-prem + colo

**Primary: SeaweedFS Compliance mode (Rust port via the `seaweedfs-rs` adapter)**

- Filer configured with `compliance.enabled = true`.
- Bucket-level retention policy applied.
- Compliance mode cannot be disabled or modified once enabled.

**Alternative: NetApp SnapLock Compliance**

- SnapLock Compliance volume; cannot be deleted before retention.

**Alternative: Dell EMC Centera / ECS Compliance**

- Compliance-mode storage pool.

### oyatie-as-cloud-provider

Whichever the cloud-provider tenant chooses; Oyatie ships the OpenTofu module for each.

### oyatie-public-cloud sovereign cells (KR, EU)

**Primary: tenant-resident WORM**

- KR cell: SeaweedFS Compliance OR a customer-provisioned NetApp SnapLock per `kr-pipa-sovereign` overlay.
- EU cell: SeaweedFS Compliance OR a customer-provisioned NetApp SnapLock per `eu-eidas-qes` overlay.

## What gets WORM-locked

| Artefact | WORM applicability |
|---|---|
| Sealed signature packet | ALWAYS WORM-locked |
| Audit-chain event | ALWAYS WORM-locked |
| Signed contract body (immutable version) | WORM-locked when `sec-17a-4` or `sox-404` active |
| Pre-signature draft | Not WORM-locked (allow ordinary edits) |
| Redline event | WORM-locked when contract is in PRESERVATION_OBLIGATION_ACTIVE legal-hold state |
| Approval evidence | WORM-locked when `sox-404` active |
| Consumer disclosure evidence | WORM-locked when `esign` active |
| GDPR consent record | WORM-locked when `gdpr` active |
| Legal hold record | ALWAYS WORM-locked |

## WORM-locked metadata

Each WORM-locked object carries metadata:

```
WORMMetadata {
  object_id: ObjectId,
  worm_provider: WORMProvider,
  worm_mode: WORMMode,                   // compliance | governance
  retention_expires_at: Timestamp<RFC3339>,
  legal_hold: bool,
  lock_attestation: LockAttestation,     // provider-issued attestation
  audit_event_id: AuditEventId,
}
```

## Retention period extension

A WORM lock can be extended (longer retention) but cannot be shortened. Triggers for extension:

- Pack activation (e.g. `sec-17a-4` activated → extend to 6y).
- Legal hold applied → extend to indefinite (legal hold suspends retention countdown).
- Contract amendment with new retention obligations.

## Retention expiration

When retention expires AND no legal hold is active:

1. Cryptographic erasure (key destruction) on the encrypted blob.
2. WORM object remains in storage (cannot be deleted under compliance mode) but is unreadable.
3. Audit event records the cryptographic erasure.

Once the WORM container's own retention expires (typically 1-3 years beyond the object retention), the storage provider deletes the object physically.

## Provider attestation

Each WORM provider must furnish:

- Attestation that the WORM property cannot be overridden by Oyatie or by AWS / OCI / hardware vendor root.
- SOC-2 or ISO 27001 audit report.
- Periodic compliance assertion.

## Cedar gate

```cedar
forbid (
  principal,
  action in [Action::"WORMLockDisable", Action::"RetentionShorten",
             Action::"ComplianceModeDowngrade"],
  resource is StorageBucket
) when {
  resource.worm_mode == "compliance"
};

forbid (
  principal,
  action == Action::"ObjectDelete",
  resource is WORMLockedObject
) when {
  resource.retention_remaining_days > 0 ||
  resource.legal_hold == true
};
```

## Audit events

- `oya.contract.lifecycle.management.worm.object_locked`
- `oya.contract.lifecycle.management.worm.retention_extended`
- `oya.contract.lifecycle.management.worm.retention_expired`
- `oya.contract.lifecycle.management.worm.cryptographic_erasure`
- `oya.contract.lifecycle.management.worm.legal_hold_extended`

## Standards references

- 17 CFR § 240.17a-4(f) (SEC).
- FINRA Rule 4511.
- 17 CFR § 275.204-2 (Investment Advisers Act).
- IRS Rev. Proc. 97-22 (electronic records).
- SEC Release No. 34-96034 (Oct 2022 amendments).
- NIST SP 800-88 Rev. 1 (Media Sanitization).
