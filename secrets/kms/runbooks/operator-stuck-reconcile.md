---
doc_class: Runbook
title: Operator Stuck Reconcile
status: Accepted
date: 2026-06-10
microservice: cloud-kms
severity: sev2
audience: sre, kms-engineer, compliance-operator
owner_team: axis-cloud + crypto-operations
doc_status: published
---

# Runbook: Operator Stuck Reconcile

## Operator Contract
- Runbook id: cloud-kms-operator-stuck-reconcile.
- Primary namespace: `cloud-kms`.
- Owning rotation: PagerDuty `cloud-kms-primary`.
- Incident channel: `#inc-cloud-kms`.
- Protected surface: `KmsKeyRing`, `KmsSealingRoot`, active key versions, decrypt-only demotion, and quarantine evidence.
- Safety invariant: ambiguous or partial observed state must fail closed and must not create, rotate, demote, or quarantine key material.
- Evidence invariant: every reconcile cycle emits exactly one structured wide-event named `cloud_kms_operator_reconcile`.
- Metrics status: the Prometheus histogram in `cloud-kms-reconcile-convergence` is target-only in this slice; until the exporter mapping lands, use the structured wide-event fields `status`, `error_class`, and `convergence_seconds` as the live signal.
- Current actuation scope: the live Cloud KMS domain actuator reconciles domain-backed `KmsKeyRing` create/rotate paths, `KmsSealingRoot` creation, decrypt-only demotion, and key-ring quarantine through Cloud KMS domain lifecycle ports.
- State invariant: production startup requires `OYATIE_KMS_OPERATOR_STATE_PATH` on the mounted `cloud-kms-operator-state` PVC; missing durable state configuration is a startup failure, not an in-memory fallback.
- Stop condition: reconcile convergence SLO is green, no key ring is stuck in ambiguous observed state, and the audit event stream has a final successful or fail-closed event for every affected object.

## Trigger Conditions
- Alert `CloudKmsOperatorReconcileConvergenceBurn` fires.
- Alert `CloudKmsOperatorPartialObservedState` fires for more than two consecutive windows.
- The cloud console shows a `KmsKeyRing` with desired state newer than status for more than five minutes.
- The operator deployment health view shows repeated restarts or unavailable replicas.
- The audit event search shows repeated `cloud_kms_operator_reconcile` failures for the same tenant/key ring.

## Diagnostic Steps
1. Open the incident console and create a SEV2 incident with service `cloud-kms`, component `operator`, and symptom `stuck-reconcile`.
2. In Grafana, navigate to the Cloud KMS operator dashboard and set filters to the affected cell, tenant, and key ring.
3. Confirm whether the structured wide-event stream shows high `convergence_seconds` or repeating fail-closed partial-observation events. Treat the Prometheus SLO as target-only until the exporter mapping is live.
4. In the Kubernetes console, navigate to namespace `cloud-kms`, workload `cloud-kms-operator`, and verify available replicas, restart count, and the current image digest.
5. In the Kubernetes console, verify the `cloud-kms-operator-state` PVC is bound and mounted at the configured operator state path.
6. In the Kubernetes console, inspect the affected `KmsKeyRing` and `KmsSealingRoot` custom resources. Compare spec generation, status observed generation, status health, and key version list.
7. In the audit-chain console, search event name `cloud_kms_operator_reconcile` with the affected tenant and key ring. Confirm there is one event per reconcile cycle and record the latest `error_class`.
8. If `error_class=partial_observed_state`, check the Kubernetes console event stream for watch relist gaps, API server throttling, or missing status subresource permissions.
9. If `error_class=domain_actuation`, navigate to the Cloud KMS domain API console for the affected tenant and inspect key create, rotation, sealing-root, demotion, or quarantine receipts.
10. If the CRD status is `Ambiguous` or `Compromised`, verify that no new key version was created after the ambiguous status timestamp.

## Mitigation
1. Keep the incident in mitigation until the operator emits either a successful reconcile event or a fail-closed event for every affected object.
2. For `partial_observed_state`, use the Kubernetes console to restart only the `cloud-kms-operator` deployment after confirming CRD API availability is healthy.
3. For RBAC denial shown in the Kubernetes console, apply the approved GitOps remediation PR that restores `kms.oyatie.com` get/list/watch/update/status permissions for the operator service account.
4. For missing or read-only operator state storage, restore the `cloud-kms-operator-state` PVC through GitOps and restart the operator only after the PVC is bound.
5. For `domain_actuation`, use the Cloud KMS API console to mark the failed key create, rotate, sealing-root, demotion, or quarantine receipt as blocked and attach the incident id.
6. If a key ring is `Compromised`, page compliance secondary before any rotation or decrypt-only demotion is attempted.
7. If active key versions are duplicated, leave the newest active version serving and wait for the operator to demote older versions after observed state is complete.

## Resolution
1. Confirm in Grafana that `cloud-kms-reconcile-convergence` is green for two consecutive five-minute windows.
2. Confirm in the Kubernetes console that affected custom resources have fresh status and healthy observed state.
3. Confirm in audit-chain that the last event for every affected object has `status=succeeded` or an intentional fail-closed `error_class`.
4. Attach screenshots or API console exports for the dashboard, CRD status, and audit event evidence to the incident record.
5. Close the incident only after compliance secondary confirms no regulated tenant key material was rotated from ambiguous observed state.

## Verification Checklist
- Operator deployment has available replicas.
- `KmsKeyRing` and `KmsSealingRoot` statuses are fresh.
- Operator state PVC is bound and mounted at `OYATIE_KMS_OPERATOR_STATE_PATH`.
- No affected object has `read_consistency=Partial` or `read_consistency=Ambiguous`.
- Reconcile convergence SLO burn is green.
- Every affected cycle has exactly one structured wide-event.
- No manual key-material mutation was performed outside the Cloud KMS API console.
