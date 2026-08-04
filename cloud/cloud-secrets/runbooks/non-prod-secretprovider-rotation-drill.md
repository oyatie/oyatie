---
doc_class: Runbook
title: Non-production SecretProvider rotation drill
microservice: cloud-secrets
owner_team: axis-cloud-secrets + ops-security
kanban_task: t_bc655724
created_at_utc: 2026-07-01T09:12:39Z
severity_default: GameDay / non-production
claim_ceiling: non_prod_drill_metadata_only
status: specified
---

# Runbook: Non-production SecretProvider rotation drill

This runbook specifies the first non-production rotation drill artifact for SECRETS-001. It is a metadata-only drill contract and does not execute OpenBao, HSM, Kubernetes, provider, or audit-chain operations. Use it to drive a later game day in a synthetic or non-production cell after the runtime owner provides safe endpoints and credentials through approved console/API/GitOps surfaces.

Companion contract: `cloud/cloud-secrets/contracts/secretprovider-rotation-contract.md`.

## Safety boundary

- Use only synthetic or non-production tenants, references, and cells.
- Do not paste raw secret values, OpenBao tokens, HSM PINs, Shamir shares, private keys, kubeconfigs, provider credentials, or audit signing material into the artifact.
- Do not use this runbook as production evidence. Production rotation requires live runtime evidence, independent review, and protected `oya-ci-required` governance.
- If any step requires an unapproved manual CLI, SSH shell, direct database mutation, plaintext fallback, or direct OpenBao/HSM console access outside the approved operator surface, stop and file a blocker/fix-forward card.

## Drill fixture for this slice

The first SECRETS-001 drill artifact should use the current contract fixture unless a later cell-runtime owner replaces it:

| Field | Fixture value |
| --- | --- |
| `artifact_id` | `SECRETS-001-NONPROD-ROTATION-DRILL-20260701-kr-seoul-1-a-001` |
| `artifact_kind` | `secretprovider_nonprod_rotation_drill` |
| `claim_ceiling` | `non_prod_drill_metadata_only` |
| `tenant_scope` | `ten_cloud_iac_oyatie_cloud_provider` or a later synthetic descendant of it |
| `cell_id` | `oyatie-cloud-provider-kr-seoul-1-a-001` |
| `pack_id` | `kr` |
| `residency_class` | `strict_kr` |
| `reference_scope` | one synthetic SecretReference for `cloud-secrets` or `cloud-iac` bootstrap metadata only |
| `openbao_namespace_ref` | `cell/oyatie-cloud-provider-kr-seoul-1-a-001/tenant/<tenant_hash>` |
| `hsm_partition_ref` | `hsm://kr/oyatie-cloud-provider-kr-seoul-1-a-001/nonprod/<partition-id>` metadata pointer only |
| `rotation_policy_id` | `rotpol://cloud-secrets/nonprod/kek-quarterly-v1` |

## Pre-flight checklist

Record each item as `pass`, `fail`, or `not_run_with_reason` in the artifact.

1. Scope confirms non-production tenant/cell only.
2. SecretReference handle is syntactically valid and contains no raw material.
3. Workload identity is available for the synthetic principal; identityless fallback is refused.
4. Cedar context includes tenant, microservice/capability, home cell, purpose, policy fragment version, and actor.
5. OpenBao namespace reference exists as metadata and is cell-local.
6. HSM partition reference has metadata for validation class, attestation ref, and expiry; it contains no credentials.
7. Audit sink readiness is either proven or explicitly recorded as a non-claim/blocker. Mutating rotation must not proceed without an audit path in a real drill.
8. Bounded TTL cache policy is visible and does not exceed the contract ceiling.
9. Rollback window and dual-publish duration are declared before any rotation action.
10. Raw-secret scan over the artifact payload passes.

## Drill sequence

The later runtime game-day owner should execute through the approved console/API/GitOps action surface. This contract names the state transitions that must be evidenced; it deliberately avoids inventing unimplemented CLI commands.

1. Create a `requested` drill record with the artifact id, tenant scope, cell id, rotation policy id, reason `nonprod-quarterly-drill`, and witness refs.
2. Run preflight and mark the record `preflighted` only if identity, Cedar, OpenBao namespace, HSM partition metadata, audit readiness, and raw-secret absence pass.
3. Create a new KEK/secret version for the synthetic reference and mark it `new_version_created`.
4. Enter `dual_publish`: new version encrypts/wraps forward; old version is decrypt-only for the declared window.
5. Validate old-version decrypt-only behavior: historical wrapped DEK/secret reads succeed only through decrypt paths and cannot perform new wraps.
6. Validate new-version forward behavior: new synthetic write/wrap uses the new version and records a version label.
7. Validate bounded TTL/fail-closed cache behavior: fresh cache hits may resolve; expired cache with unavailable control plane fails closed.
8. Validate revocation/subscriber behavior if the drill includes a revoke phase; p99 target and subscriber count are recorded as metadata.
9. Promote the new version as primary only after validation passes.
10. Seal the metadata-only artifact with expected event refs or explicit audit non-claim fields.
11. Retire or retain the old version according to the declared decrypt-only retention rule; do not destroy any key material unless this is a runtime-approved destructive drill.

## Required artifact body

Use this body shape for the first drill artifact. Values below are placeholders or fixture values; real runtime evidence fields stay empty until a game day runs.

```yaml
artifact_id: SECRETS-001-NONPROD-ROTATION-DRILL-20260701-kr-seoul-1-a-001
artifact_kind: secretprovider_nonprod_rotation_drill
claim_ceiling: non_prod_drill_metadata_only
kanban_task: t_bc655724
source_contract: cloud/cloud-secrets/contracts/secretprovider-rotation-contract.md
scope:
  environment: non_prod
  tenant_scope: ten_cloud_iac_oyatie_cloud_provider
  tenant_scope_is_synthetic_or_non_prod: true
  cell_id: oyatie-cloud-provider-kr-seoul-1-a-001
  pack_id: kr
  residency_class: strict_kr
provider_refs:
  secret_reference: openbao:secret/tenant:fixture/cloud-secrets/nonprod-drill@v1
  openbao_namespace_ref: cell/oyatie-cloud-provider-kr-seoul-1-a-001/tenant/<tenant_hash>
  hsm_partition_ref: hsm://kr/oyatie-cloud-provider-kr-seoul-1-a-001/nonprod/<partition-id>
  rotation_policy_id: rotpol://cloud-secrets/nonprod/kek-quarterly-v1
preflight_checks:
  non_prod_scope: not_run_with_reason
  secret_reference_no_raw_material: not_run_with_reason
  workload_identity_present: not_run_with_reason
  cedar_context_complete: not_run_with_reason
  openbao_namespace_cell_local: not_run_with_reason
  hsm_partition_attested: not_run_with_reason
  audit_sink_ready: not_run_with_reason
  ttl_cache_policy_bounded: not_run_with_reason
  rollback_window_declared: not_run_with_reason
  artifact_raw_secret_scan: not_run_with_reason
rotation:
  state_sequence:
    - requested
    - preflighted
    - new_version_created
    - dual_publish
    - validated
    - promoted
    - retired
    - sealed
  old_version_policy: decrypt_only_during_dual_publish
  new_version_policy: encrypt_forward_only_after_creation
  reencrypt_existing_ciphertext: false
validation_checks:
  old_version_decrypt_only: not_run_with_reason
  new_version_encrypt_forward: not_run_with_reason
  wrapped_dek_routes_by_kek_version: not_run_with_reason
  cache_expired_control_plane_unavailable_fails_closed: not_run_with_reason
  revocation_push_deadline_recorded: not_run_with_reason
  audit_events_expected:
    - SecretRotated
    - KekRotated
    - KekAttested
    - RevocationPush
rollback:
  dual_publish_rollback_allowed: true
  rollback_action: restore provider primary pointer to prior decrypt-only version inside window; no plaintext fallback
  destructive_destroy_allowed: false
non_claims:
  - no production rotation executed
  - no live OpenBao/HSM/API call executed by this artifact
  - no audit-chain persistence claimed unless real event ids are attached later
  - no raw secret material captured
  - no generated JSON hand edits
```

## Pass/fail criteria for a later game day

A later non-production game day passes only if:

1. The artifact is complete and contains no raw secret material.
2. The synthetic reference resolves before and after rotation only through SecretProvider.
3. New writes/wraps use the new version; old version is decrypt-only.
4. Expired cache plus unavailable control plane fails closed.
5. Audit/event refs are present or the drill is explicitly blocked before mutation.
6. Rollback can restore the prior provider pointer during dual-publish without plaintext fallback.
7. HSM partition and OpenBao namespace evidence stay cell-local.

The drill fails closed if any preflight item is missing, if the path crosses a residency/cell boundary without an explicit contract, if any raw material appears in evidence, or if rotation relies on an unapproved manual CLI/SSH/database path.

## Rollback and blocker handling

- During dual-publish, rollback means restoring the SecretProvider primary pointer to the previous decrypt-only-capable version and pausing promotion. It does not mean copying plaintext or re-encrypting all historical ciphertext.
- After the dual-publish window, rollback requires a new rotation event; do not resurrect destroyed material.
- If OpenBao namespace, HSM partition, workload identity, Cedar, audit, or revocation evidence is unavailable, record the blocker and stop before mutation.

## References

- `cloud/cloud-secrets/contracts/secretprovider-rotation-contract.md`
- `cloud/cloud-secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md`
- `cloud/cloud-secrets/runbooks/hsm-key-rotation.md`
- `cloud/cloud-secrets/runbooks/rotation-cascade-recovery.md`
- `cloud/cloud-kms/crates/oya-cloud-kms-enclave-kernel/src/chain.rs`
- `cloud/cloud-kms/crates/oya-cloud-kms-enclave-kernel/src/token.rs`
- `cloud/cloud-kms/crates/oya-cloud-kms-enclave-kernel/src/dek_cache.rs`
- `docs/decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md`
- `docs/decisions/ADR-0161-csi-storage-class-canonical.md`
- `docs/decisions/ADR-0536-hyperscaler-grounded-substrate-decision-matrix.md`
- `docs/decisions/ADR-0537-dogfood-bootstrap-order-rust-owned-stack-doctrine.md`
