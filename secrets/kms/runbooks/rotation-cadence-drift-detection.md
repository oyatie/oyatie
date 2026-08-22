---
doc_class: Runbook
title: Rotation Cadence Drift Detection
status: Accepted
date: 2026-05-20
microservice: cloud-kms
severity: sev2
audience: sre, kms-engineer, compliance-operator
owner_team: axis-cloud + crypto-operations + compliance-security
doc_status: published
---

# Runbook: Rotation Cadence Drift Detection

## Operator Contract
- Runbook id: cloud-kms-rotation-cadence-drift-detection.
- Primary namespace: `cloud-kms`.
- Owning rotation: PagerDuty `cloud-kms-primary`.
- Compliance secondary: PagerDuty `compliance-primary`.
- Incident channel: `#inc-cloud-kms`.
- Customer channel: `#support-cloud-kms-compliance`.
- Protected surface: CMK rotation schedules, KEK promotion, decrypt-only grace, PCI/SOC2/FedRAMP/KR K-FSI evidence.
- Safety invariant: do not force-rotate a CMK until decrypt-only grace and replica health are verified.
- Compliance invariant: production paid tenant_class CMKs cannot use `rotation-cadence none`.
- Evidence invariant: every cadence correction emits audit-chain evidence.
- Stop condition: drifted CMKs are corrected or have approved exception records, and compliance export is regenerated.
- Evidence event: `EVT_CLOUD_KMS_ROTATION_DRIFT_INCIDENT`.
- Handoff API: `https://cloud-kms.internal.oyatie.dev/v1/rotation/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/rotation?orgId=1&var-cell=prod-us-east-1`.
- Compliance dashboard: `https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/compliance-evidence?orgId=1&var-surface=rotation`.
- Loki query: `{namespace="cloud-kms",runbook="rotation-cadence-drift-detection"}`.
- Canonical FAQ: `microservices/cloud-kms/faqs/kms-engineer-faq.md`.
- Related migration guide: `microservices/cloud-kms/migration-playbooks/from-aws-kms-and-vault-enterprise.md`.

## Trigger Conditions
- Alert `CloudKmsRotationCadenceDrift` fires.
- Alert `CloudKmsRotationOverdueCritical` fires for any regulated tenant.
- Alert `CloudKmsDecryptOnlyGraceExpired` fires.
- Alert `CloudKmsRotationEvidenceExportStale` fires.
- Metric `cloud_kms_rotation_overdue_cmk_total` is non-zero.
- Metric `cloud_kms_rotation_cadence_drift_total` increases.
- Metric `cloud_kms_rotation_job_lag_seconds` exceeds 3600.
- Metric `cloud_kms_decrypt_only_kek_expired_total` is non-zero.
- Metric `cloud_kms_rotation_exception_expired_total` is non-zero.
- Metric `cloud_kms_rotation_policy_none_production_total` is non-zero for paid tenant_class.
- Compliance scan finds missing rotation evidence for PCI, SOC2, FedRAMP, or KR K-FSI.
- Tenant asks why a CMK has not rotated by contractual cadence.
- BYOK tenant rotation webhook is stale.
- AWS XKS or external key store reports stale external KEK.
- Rotation job is blocked by quorum loss.
- Rotation job is blocked by HSM cluster failover.
- Audit-chain lacks `cloud_kms.rotation.completed` after `cloud_kms.rotation.started`.
- Production snapshot gate `cloud-kms-rotation` fails.
- Rate card or tenant tenant_class upgrade changed cadence but CMK schedule did not update.
- Dev-only `rotation-cadence none` tag appears on production CMK.

## Symptoms
- CMK `next_rotation_at` is in the past.
- `current_kek_id` age exceeds policy.
- `decrypt_only_until` is older than current timestamp.
- Rotation worker queue grows while HSM operations are healthy.
- Regulated tenant report export has missing CMK rows.
- `rotation_exception_status=expired` appears in logs.
- `policy_cadence=annual` but `cmk_cadence=none` appears in state.
- `tenant_tier=paid` but cadence remains demo_trial default.
- BYOK external key store reports stale version.
- KEK replicas are healthy but rotation job never starts.
- Audit-chain shows `rotation.started` without `rotation.completed`.
- Rotation job retries on one tenant only.
- Rotation job retries on all tenants after cadence policy rollout.
- Compliance dashboard turns red before customer impact.
- Decrypt operations still succeed, hiding compliance risk.
- Cryptoshred queue may be paused if expired decrypt-only KEK is involved.
- Support sees compliance questions, not availability complaints.
- Incident severity rises if stale KEK protects regulated data.
- Incident severity rises if exception approval expired.
- Incident severity rises if cadence drift was caused by policy compiler regression.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-kms-rotation-drift-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export CMK=cmk-synthetic`.
3. Acknowledge page: `pd incident ack --service cloud-kms --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-cloud-kms --severity sev2`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.surface=="kms-rotation")'`.
6. Query overdue CMKs: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_kms_rotation_overdue_cmk_total{cell="'$CELL'"}'`.
7. Query drift count: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_kms_rotation_cadence_drift_total{cell="'$CELL'"}'`.
8. Query job lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_kms_rotation_job_lag_seconds{cell="'$CELL'"}'`.
9. Query expired grace: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_kms_decrypt_only_kek_expired_total{cell="'$CELL'"}'`.
10. Query production none: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_kms_rotation_policy_none_production_total{cell="'$CELL'"}'`.
11. Open rotation dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/rotation?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
12. Open compliance dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/compliance-evidence?orgId=1&var-tenant=$TENANT"`.
13. Read rotation logs: `kubectl -n cloud-kms logs deploy/cloud-kms-rotation-worker --since=60m | rg "rotation|cadence|decrypt_only|exception"`.
14. Check rollout: `kubectl -n cloud-kms rollout status deploy/cloud-kms-rotation-worker --timeout=60s`.
15. List overdue CMKs: `oya kms rotation overdue --tenant $TENANT --cell $CELL --output table`.
16. Inspect CMK: `oya kms cmk get --tenant $TENANT --cmk $CMK --cell $CELL --output yaml`.
17. Inspect cadence policy: `oya kms rotation policy get --tenant $TENANT --cmk $CMK --output yaml`.
18. Inspect tenant_class cadence: `oya kms rotation tenant_class-policy --tenant $TENANT --tenant-class current --output yaml`.
19. Inspect exceptions: `oya kms rotation exception list --tenant $TENANT --cmk $CMK --output yaml`.
20. Check KEK age: `oya kms kek list --tenant $TENANT --cmk $CMK --cell $CELL --output table`.
21. Check replica health: `oya kms kek replica status --tenant $TENANT --cmk $CMK --cell $CELL --output json`.
22. Run safe drift check: `oya kms rotation drift scan --tenant $TENANT --cell $CELL --dry-run --output json`.
23. Run compliance export check: `oya kms evidence export --tenant $TENANT --surface rotation --dry-run --output evidence/incidents/$INCIDENT_ID-rotation-evidence.json`.
24. Check quorum dependency: `oya kms quorum requests list --tenant $TENANT --cell $CELL --state pending --filter rotation`.
25. Check HSM dependency: `oya kms hsm inventory --cell $CELL --tenant-class current --output table`.
26. Check audit starts: `oya audit-chain query --event-class cloud_kms.rotation.started --tenant $TENANT --since 30d`.
27. Check audit completions: `oya audit-chain query --event-class cloud_kms.rotation.completed --tenant $TENANT --since 30d`.
28. Check exception approvals: `oya audit-chain query --event-class cloud_kms.rotation.exception.approved --tenant $TENANT --since 365d`.
29. Check BYOK webhook: `oya kms byok webhook status --tenant $TENANT --cmk $CMK --output json`.
30. Check XKS backend: `oya kms xks status --tenant $TENANT --cmk $CMK --output json`.
31. Check tenant tenant_class change: `oya tenancy tenant_class history --tenant $TENANT --since 90d --output table`.
32. Check rate card relation: `oya billing tenant contract get --tenant $TENANT --fields tier,compliance_pack,kms_rotation`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-kms --runbook rotation-cadence-drift-detection --output evidence/incidents/$INCIDENT_ID.json`.
34. Export drift rows: `oya kms rotation drift scan --tenant $TENANT --cell $CELL --output json > evidence/incidents/$INCIDENT_ID-drift.json`.
35. Export policy rows: `oya kms rotation policy export --tenant $TENANT --output evidence/incidents/$INCIDENT_ID-policies.yaml`.

### Diagnostic Decision Tree
```text
1. Is any production paid tenant_class CMK configured with cadence none?
   |-- yes: treat as policy drift; set compliant cadence before rotation.
   |-- no: continue overdue triage.
2. Is rotation overdue because quorum or HSM is unhealthy?
   |-- yes: invoke key quorum or HSM failover runbook first.
   |-- no: inspect rotation worker and policy projection.
3. Is there an approved active exception?
   |-- yes: regenerate evidence and update dashboard.
   |-- no: correct cadence and schedule safe rotation.
4. Is decrypt-only grace expired?
   |-- yes: pause cryptoshred and page compliance.
   |-- no: proceed with safe rotation.
5. Did tenant tenant_class or compliance pack change recently?
   |-- yes: patch cadence propagation from tenancy or billing.
   |-- no: patch rotation scheduler or metadata state.
```

## Mitigation
1. Pause cryptoshred when expired grace is present: `oya flags set oya.cloud_kms.cryptoshred.pause=true --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
2. Hold new cadence policy deploys: incident hold PR against `dev` (normal VCS PR; branch-protected GitHub Actions `presubmit` required; local/Jenkins rehearsals are non-authoritative).
3. Freeze evidence export: `oya evidence freeze --incident $INCIDENT_ID --paths evidence/incidents/$INCIDENT_ID-drift.json`.
4. Correct invalid cadence dry-run: `oya kms rotation policy set --tenant $TENANT --cmk $CMK --cadence annual --dry-run`.
5. Correct invalid cadence confirmed: `oya kms rotation policy set --tenant $TENANT --cmk $CMK --cadence annual --confirm $INCIDENT_ID`.
6. Schedule safe rotation dry-run: `oya kms rotation schedule --tenant $TENANT --cmk $CMK --window next-safe --dry-run`.
7. Schedule safe rotation confirmed: `oya kms rotation schedule --tenant $TENANT --cmk $CMK --window next-safe --confirm $INCIDENT_ID`.
8. Start urgent rotation only with compliance approval: `oya kms rotation start --tenant $TENANT --cmk $CMK --confirm $INCIDENT_ID`.
9. Regenerate evidence export: `oya kms evidence export --tenant $TENANT --surface rotation --output evidence/incidents/$INCIDENT_ID-rotation-evidence.json`.
10. Mark approved exception: `oya kms rotation exception record --tenant $TENANT --cmk $CMK --reason $INCIDENT_ID --expires <date>`.
11. Restart stuck rotation worker: `kubectl -n cloud-kms rollout restart deploy/cloud-kms-rotation-worker`.
12. Drain scheduler backlog: `oya kms rotation queue drain --tenant $TENANT --cell $CELL --limit 50 --dry-run`.
13. Drain scheduler backlog confirmed: `oya kms rotation queue drain --tenant $TENANT --cell $CELL --limit 50 --confirm $INCIDENT_ID`.
14. Notify compliance: `oya notify compliance --incident $INCIDENT_ID --category kms-rotation-drift`.
15. Notify tenant admin when contractual cadence missed: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template kms-rotation-delay`.
16. Emit mitigation audit: `oya audit-chain emit --event-class EVT_CLOUD_KMS_ROTATION_DRIFT_INCIDENT --incident $INCIDENT_ID --field mitigation=cadence-corrected`.
17. Keep force-rotation disabled until replica health is green.
18. Keep cadence exception short-lived and owner-bound.
19. Keep all old KEKs decrypt-only until grace is valid.
20. Keep compliance export attached to incident evidence.

## Resolution
1. Patch cadence projection from tier, rate card, or compliance pack.
2. Patch rotation scheduler if overdue jobs did not enqueue.
3. Patch BYOK webhook handling if external rotation drifted.
4. Patch exception expiry checks if expired exceptions remained green.
5. Patch evidence exporter if compliance rows were missing.
6. Add regression fixture for paid tenant_class production cadence none.
7. Add regression fixture for tenant_class upgrade cadence propagation.
8. Add regression fixture for exception expiry.
9. Run domain tests: `cargo test -p cloud-kms-domain rotation -- --nocapture`.
10. Run API tests: `cargo test -p cloud-kms-api rotation -- --nocapture`.
11. Verify the branch-protected production-snapshot gate for `cloud-kms-rotation` in `presubmit` / cloud-ci for `$CELL`; do not use local dev-cli output as merge authority.
12. Verify cadence scan: `oya kms rotation drift scan --tenant $TENANT --cell $CELL --expect none`.
13. Verify evidence export: `oya kms evidence export --tenant $TENANT --surface rotation --expect complete`.
14. Unhold promotions: recovery PR against `dev` (normal VCS PR; branch-protected GitHub Actions `presubmit` required; local/Jenkins rehearsals are non-authoritative).
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_KMS_ROTATION_DRIFT_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudKmsRotationCadenceDrift` is green.
- `CloudKmsRotationOverdueCritical` is green.
- `cloud_kms_rotation_overdue_cmk_total` is zero.
- `cloud_kms_rotation_policy_none_production_total` is zero.
- `cloud_kms_decrypt_only_kek_expired_total` is zero.
- Drift scan returns no unapproved rows.
- Compliance export includes all affected CMKs.
- Rotation started and completed events are sealed.
- Any exception has owner, expiry, and compliance approval.
- Tenant admin notification is attached if contractual cadence was missed.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-kms-rotation-cadence-drift-detection
microservice: cloud-kms
event_class: EVT_CLOUD_KMS_ROTATION_DRIFT_INCIDENT
incident_id: <INC-...>
severity: sev2
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Rotation Cadence Drift Detection postmortem

## Summary
- Which tenant, CMK, tier, and compliance pack drifted.
- Whether cadence was overdue, missing, or exception-related.
- Whether any encrypted data exceeded policy age.

## Timeline
- Drift detected:
- Cadence corrected:
- Rotation scheduled:
- Evidence regenerated:
- Audit sealed:

## Compliance Impact
- PCI:
- SOC2:
- FedRAMP:
- KR K-FSI:

## Root Cause
- Tier propagation:
- Scheduler:
- BYOK webhook:
- Evidence exporter:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Compliance report:
```

## Escalation Path
- Page `cloud-kms-primary` for rotation drift.
- Page `compliance-primary` for regulated tenant or evidence gaps.
- Page `crypto-operations-primary` when drift is blocked by quorum or HSM health.
- Page tenant custodian when BYOK or XKS external rotation is stale.
- Notify `#inc-cloud-kms` with tenant, CMK, and compliance pack scope.
- Notify `#support-cloud-kms-compliance` before tenant-facing messages.
- Notify `#legal-review` when contractual rotation cadence was missed.
- Escalate to executive incident commander when regulated evidence is missing for more than one day.
- Engage external key-store owner through tenant support path.
- Keep cadence exceptions visible in incident bridge.

## Cross-µservice Coordination
- `tenancy`: confirm tenant tenant_class and compliance pack changes.
- `cloud-billing`: confirm rate card or contract tenant_class that sets rotation cadence.
- `audit-chain`: seal rotation, exception, mitigation, and resolution events.
- `cloud-iam`: verify operators can approve rotation and exceptions.
- `workflow-engine`: pause workflows waiting on rotation until schedule is known.
- `compliance`: own evidence export and reporting decision.
- `support`: manage tenant-facing rotation delay communication.
- `observability`: attach rotation and compliance dashboard snapshots.
- `security`: review stale KEK risk.
- `foundry`: pause cadence policy deploys until corrected.
- `cloud-network`: verify no mTLS tenant CA rotation is blocked by stale CMK.
- `comms-email`: send approved all-clear.

## Runbook Maintenance
- Add new compliance packs to Trigger Conditions.
- Keep cadence thresholds aligned with tenant_class matrix.
- Keep BYOK and XKS drift checks current.
- Review this runbook after every cadence policy change.
- Keep force-rotation warnings explicit.
