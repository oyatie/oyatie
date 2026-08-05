---
doc_class: Contract
title: Cloud Secrets local resource-contract packet
microservice: cloud-secrets
owner_team: axis-cloud-secrets
kanban_task: t_41f4c66f
claim_ceiling: contract_and_red_fixtures_only
status: local_contract_packet
---

# Cloud Secrets local resource-contract packet

This packet tracks the local Cloud Control Plane resource-contract slice for
`cloud-secrets` resource types `Secret`, `SecretMount`, and `SecretPolicy`.
It is a contract/fixture artifact only. It does not claim production runtime,
live OpenBao/HSM/KMS actuation, provider/IaC/Kubernetes/OpenTofu/Argo mutation,
Resource Registry persistence, operation-ledger persistence, Cedar runtime,
audit-chain persistence, measured OpenTelemetry/OpenSLO evidence, billing/quota
enforcement, rollback controller, reconciliation controller, or raw secret
material handling beyond the metadata-only Cloud Secrets foundation already
listed in `manifest.json`.

Machine-readable packet: `cloud-secrets-resource-contract.json`.

## Authority and local boundaries

- `specs/cloud-control-plane-canonical.json` requires the hierarchy,
  ORN-addressed resource model, Resource Registry, Operation Ledger, and nine
  facets: lifecycle, identity, policy, quota, billing, audit, observability,
  rollback, and reconciliation.
- `specs/cloud-resource-catalog-target.json` names `cloud-secrets` resource
  types `Secret`, `SecretMount`, and `SecretPolicy` with OpenBao actuation and
  fail-closed SecretReference notes.
- `cloud/cloud-secrets/manifest.json` and this README keep Cloud Secrets at a
  metadata-only foundation: SecretReference, fail-closed bootstrap admission,
  zeroizing buffers, and metadata-only persistence seams.
- `contracts/secretprovider-rotation-contract.md` owns the SECRETS-001
  SecretProvider/OpenBao/HSM rotation vocabulary and drill envelope. This
  packet references vocabulary such as `RotationReceipt`, `dual_publish`,
  `DecryptOnlyKek`, `RevocationReceipt`, and `RevocationPush`; it does not
  duplicate the rotation runbook or claim live HSM/OpenBao operations.

## Resource summary

| Resource type | Intent | Contract ceiling |
| --- | --- | --- |
| `Secret` | Tenant/account/project/cell-scoped Secret metadata and versions while material stays behind SecretProvider/OpenBao/HSM/KMS boundaries. | Contract fields only; no registry/LRO/API/provider runtime. |
| `SecretMount` | Workload/cell-scoped binding that lets workload identity resolve a Secret through SecretProvider. | Contract fields only; no mount model/controller/workload delivery runtime. |
| `SecretPolicy` | Versioned policy resource for access, rotation, retention, residency, TTL, break-glass, and evidence/audit requirements. | Contract fields only; no Cedar runtime/policy registry/rollback primitive. |

Each resource type is mapped in JSON to the required facets and per-resource
fields: `quota_cost`, `billing_meters`, `audit_events`, `lifecycle_state`,
`owner`, `tenant_account_project`, `region_cell`, `slo_tier`, and
`deletion_retention_policy`.

## Gap and non-claim fields carried by the JSON packet

The machine-readable packet intentionally includes explicit gap/non-claim fields
for:

- ORN/resource registry identity.
- Durable operation ledger / long-running operation state.
- Cedar runtime and policy-snapshot evaluation.
- Quota and billing meters/enforcement.
- Audit event vocabulary vs. absent audit-chain persistence.
- OpenTelemetry/OpenSLO evidence, `query_digest`, and `evidence_digest` inputs.
- Rollback targets vs. absent rollback controller.
- Reconciliation targets vs. absent desired/actual controller.
- Live OpenBao/HSM/KMS/root-trust actuation.

These fields are part of the local contract because the current implementation
must stay honest about non-claims while future runtime lanes fill the gaps.

## RED/static fixture intent

`crates/oya-secrets-domain/tests/cloud_secrets_resource_contract.rs` is a
static contract guard. It rejects packets that omit any required resource type,
omit the nine facets/non-claim fields, or overclaim production/runtime/measured
SLO/audit-chain/OpenBao-HSM readiness by setting protected readiness flags to
`true`.

## De-dupe notes

- t_bc655724 owns SecretProvider/OpenBao/HSM rotation and the non-prod drill
  runbook. This packet references vocabulary only.
- t_49514ca4 owns measured OpenSLO/query-digest evidence production. This
  packet names required fields only.
- t_688c8b9b owns shared identity/secret/audit fixtures. This packet requires
  identity/policy/audit fields only.
- Root trust/KMS/HSM custody lanes own live custody evidence. This packet
  consumes metadata/evidence handles only.
