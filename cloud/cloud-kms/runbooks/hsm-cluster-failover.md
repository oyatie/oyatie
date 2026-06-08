---
doc_class: Runbook
title: HSM Cluster Failover
status: Accepted
date: 2026-05-20
microservice: cloud-kms
severity: sev1
audience: sre, security-engineer, kms-engineer
owner_team: axis-cloud + crypto-operations + ops-sre-reliability
doc_status: published
---

# Runbook: HSM Cluster Failover

## Operator Contract
- Runbook id: cloud-kms-hsm-cluster-failover.
- Primary namespace: `cloud-kms`.
- Owning rotation: PagerDuty `oya-cloud-kms-primary`.
- Crypto secondary: PagerDuty `oya-crypto-operations-primary`.
- Incident channel: `#inc-cloud-kms`.
- Customer channel: `#support-cloud-kms-tenant-impact`.
- Protected surface: CMK metadata, KEK availability, DEK unwrap, signing operations, tenant CA issuance.
- HSM families: Marvell LiquidSecurity 2, Thales Luna 7, Utimaco SecurityServer Se Gen2.
- Tier invariant: paid has at least three HSMs per cluster; paid has regional peer failover; paid has multi-vendor failover.
- Safety invariant: never export plaintext KEK material.
- Failover invariant: decrypt and sign may move to healthy HSM; cryptoshred stays paused until quorum and receipts are verified.
- Stop condition: active cluster is healthy, failover receipts are sealed, and decrypt/sign p99 returns to SLO for 30 minutes.
- Evidence event: `EVT_CLOUD_KMS_HSM_CLUSTER_FAILOVER_INCIDENT`.
- Handoff API: `https://cloud-kms.internal.oyatie.dev/v1/hsm/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/hsm-clusters?orgId=1&var-cell=prod-us-east-1`.
- Crypto dashboard: `https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/crypto-ops?orgId=1&var-tenant_class=paid`.
- Loki query: `{namespace="cloud-kms",runbook="hsm-cluster-failover"}`.
- Canonical FAQ: `microservices/cloud-kms/faqs/kms-engineer-faq.md`.
- Related tutorial: `microservices/cloud-kms/tutorials/envelope-encrypt-rotate-and-cryptoshred.md`.

## Trigger Conditions
- Alert `CloudKmsHsmClusterDegradedCritical` fires.
- Alert `CloudKmsHsmOperationTimeoutBurn` fires for 10 minutes.
- Alert `CloudKmsDecryptErrorRatioHigh` fires in any production cell.
- Alert `CloudKmsSignErrorRatioHigh` fires for tenant CA signing.
- Alert `CloudKmsHsmAttestationDrift` fires.
- Metric `oya_cloud_kms_hsm_available_nodes` falls below quorum for tier.
- Metric `oya_cloud_kms_hsm_operation_timeout_total` increases by more than 50 in 5 minutes.
- Metric `oya_cloud_kms_decrypt_error_ratio` exceeds 0.005.
- Metric `oya_cloud_kms_sign_error_ratio` exceeds 0.005.
- Metric `oya_cloud_kms_hsm_attestation_invalid_total` is non-zero.
- Metric `oya_cloud_kms_hsm_failover_attempt_total` increases without success.
- Metric `oya_cloud_kms_hsm_cluster_latency_p99_seconds` exceeds 3.
- Metric `oya_cloud_kms_peer_region_decrypt_fallback_total` increases.
- Synthetic probe `oya ops probe cloud-kms hsm-decrypt` fails twice.
- Synthetic probe `oya ops probe cloud-kms hsm-sign` fails twice.
- Tenant reports decrypt failures or certificate issuance failures.
- Cloud-iam reports session-token signer failures.
- Cloud-network reports tenant CA mTLS issuance failures.
- Foundry reports cosign signing failures.
- HSM inventory shows measured boot drift from known-good baseline.

## Symptoms
- `KmsError::HsmClusterDegraded` appears in API logs.
- `hsm_operation_status=timeout` repeats across nodes in one cluster.
- `attestation_status=invalid` appears for one HSM serial number.
- `decrypt_fallback_region` appears for paid or paid tenants.
- HSM client pool reports all sockets unhealthy.
- One HSM vendor fails while peer vendor remains healthy.
- DEK unwrap fails but metadata API remains healthy.
- Sign operations fail for tenant CA, cosign, or session-token signer.
- Cryptoshred queue is paused by safety guard.
- Rotation jobs fail because current KEK cannot be promoted.
- `cmk_current_kek_id` remains stable while operation failures rise.
- CPU on cloud-kms pods is low while operation latency is high.
- HSM partition reports `offline`, `tamper`, `locked`, or `fips-error`.
- Peer region operations succeed with higher latency.
- The dashboard shows node-specific failure, not application-wide failure.
- Audit-chain operation events exist but HSM attestation receipt is missing.
- `cloud_kms.hsm.failover.started` exists without `cloud_kms.hsm.failover.completed`.
- Tenant workloads see transient decrypt latency spikes.
- Security surface is high because failed failover can cause incorrect key use.
- Customer impact is severe when decrypt fails for active workloads.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-kms-hsm-failover-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export TENANT_CLASS=paid`.
3. Acknowledge page: `pd incident ack --service cloud-kms --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-cloud-kms --severity sev1`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.service=="cloud-kms")'`.
6. Query available HSM nodes: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_kms_hsm_available_nodes{cell="'$CELL'",tier="'$TIER'"}'`.
7. Query operation timeouts: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_cloud_kms_hsm_operation_timeout_total[5m])'`.
8. Query decrypt errors: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_kms_decrypt_error_ratio{cell="'$CELL'"}'`.
9. Query sign errors: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_kms_sign_error_ratio{cell="'$CELL'"}'`.
10. Query attestation drift: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_cloud_kms_hsm_attestation_invalid_total[5m])'`.
11. Open HSM dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/hsm-clusters?orgId=1&var-cell=$CELL&var-tenant_class=$TIER"`.
12. Open crypto operations dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/crypto-ops?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
13. Read API logs: `kubectl -n cloud-kms logs deploy/cloud-kms-api --since=30m | rg "HsmClusterDegraded|hsm_operation_status|attestation"`.
14. Read HSM worker logs: `kubectl -n cloud-kms logs deploy/cloud-kms-hsm-worker --since=30m | rg "failover|partition|fips|tamper|timeout"`.
15. Check rollout: `kubectl -n cloud-kms rollout status deploy/cloud-kms-hsm-worker --timeout=60s`.
16. List pods: `kubectl -n cloud-kms get pods -l app=hsm-worker -o wide`.
17. Inspect HSM inventory: `oya kms hsm inventory --cell $CELL --tenant-class $TIER --output table`.
18. Inspect bad HSM: `oya kms hsm inspect --cell $CELL --serial <serial> --output json`.
19. Verify attestation: `oya kms hsm attest --cell $CELL --serial <serial> --output json`.
20. Check partition status: `oya kms hsm partition status --cell $CELL --tenant $TENANT --tenant-class $TIER --output json`.
21. Check CMK metadata: `oya kms cmk list --tenant $TENANT --cell $CELL --state active --output table`.
22. Test decrypt canary: `oya ops probe cloud-kms hsm-decrypt --tenant $TENANT --cell $CELL --tenant-class $TIER --output json`.
23. Test sign canary: `oya ops probe cloud-kms hsm-sign --tenant $TENANT --cell $CELL --tenant-class $TIER --output json`.
24. Check peer region: `oya kms hsm peer-region status --tenant $TENANT --cell $CELL --output json`.
25. Check failover plan: `oya kms hsm failover plan --tenant $TENANT --cell $CELL --tenant-class $TIER --dry-run`.
26. Check cryptoshred queue: `oya kms cryptoshred queue status --tenant $TENANT --cell $CELL --output json`.
27. Check rotation queue: `oya kms rotation queue status --tenant $TENANT --cell $CELL --output json`.
28. Check audit failover events: `oya audit-chain query --event-class cloud_kms.hsm.failover.started --tenant $TENANT --since 30m`.
29. Check operation receipts: `oya audit-chain query --event-class cloud_kms.hsm.operation.receipt --tenant $TENANT --since 30m`.
30. Check cloud-iam impact: `oya ops dependency impact --source cloud-kms --target cloud-iam --surface session-signing --since 30m`.
31. Check cloud-network impact: `oya ops dependency impact --source cloud-kms --target cloud-network --surface tenant-ca --since 30m`.
32. Check foundry impact: `oya ops dependency impact --source cloud-kms --target foundry --surface artifact-signing --since 30m`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-kms --runbook hsm-cluster-failover --output evidence/incidents/$INCIDENT_ID.json`.
34. Freeze HSM receipts: `oya kms hsm receipts export --tenant $TENANT --cell $CELL --since 30m --output evidence/incidents/$INCIDENT_ID-receipts.json`.
35. Freeze inventory: `oya kms hsm inventory --cell $CELL --tenant-class $TIER --output json > evidence/incidents/$INCIDENT_ID-hsm-inventory.json`.

### Diagnostic Decision Tree
```text
1. Is quorum below tenant_class minimum?
   |-- yes: pause cryptoshred and rotation, activate failover branch.
   |-- no: inspect latency, attestation, and client pool failures.
2. Is attestation invalid for any HSM?
   |-- yes: isolate that HSM and page crypto operations before routing traffic.
   |-- no: continue failover health checks.
3. Are decrypt and sign both failing?
   |-- yes: cluster or partition issue; use peer cluster or peer region.
   |-- no: isolate operation-specific key, signer, or tenant CA path.
4. Does peer region pass canary?
   |-- yes: fail over bounded tenants to peer region.
   |-- no: escalate to global crypto incident.
5. Are operation receipts missing?
   |-- yes: keep incident open and replay receipts.
   |-- no: close after 30 minutes of SLO recovery.
```

## Mitigation
1. Hold risky operations: `oya flags set oya.cloud_kms.cryptoshred.pause=true --cell $CELL --reason $INCIDENT_ID`.
2. Pause rotation: `oya flags set oya.cloud_kms.rotation.pause=true --cell $CELL --reason $INCIDENT_ID`.
3. Open HSM breaker: `oya ops breaker open cloud-kms-hsm-cluster --cell $CELL --ttl 30m --reason $INCIDENT_ID`.
4. Isolate invalid HSM: `oya kms hsm isolate --cell $CELL --serial <serial> --reason $INCIDENT_ID --dry-run`.
5. Confirm isolation: `oya kms hsm isolate --cell $CELL --serial <serial> --reason $INCIDENT_ID --confirm`.
6. Fail over tenant dry-run: `oya kms hsm failover --tenant $TENANT --cell $CELL --to-peer --dry-run`.
7. Fail over tenant confirmed: `oya kms hsm failover --tenant $TENANT --cell $CELL --to-peer --confirm $INCIDENT_ID`.
8. Fail over tenant_class cohort only with commander approval: `oya kms hsm failover --tenant-class $TIER --cell $CELL --to-peer --confirm $INCIDENT_ID`.
9. Reduce HSM client concurrency: `oya flags set oya.cloud_kms.hsm.max_inflight=50 --cell $CELL --reason $INCIDENT_ID`.
10. Enable peer-region decrypt fallback: `oya flags set oya.cloud_kms.decrypt.peer_region_fallback=true --tenant $TENANT --cell $CELL`.
11. Keep new key creation paused: `oya flags set oya.cloud_kms.cmk.create.pause=true --cell $CELL --reason $INCIDENT_ID`.
12. Restart stuck worker: `kubectl -n cloud-kms rollout restart deploy/cloud-kms-hsm-worker`.
13. Renew HSM client certs: `oya kms hsm client-cert renew --cell $CELL --service cloud-kms --reason $INCIDENT_ID`.
14. Notify dependent owners: `oya notify service-owner --incident $INCIDENT_ID --microservice cloud-iam,cloud-network,foundry`.
15. Notify tenant admins when decrypt impact is confirmed: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template kms-decrypt-degraded`.
16. Preserve receipts: `oya evidence freeze --incident $INCIDENT_ID --paths evidence/incidents/$INCIDENT_ID-receipts.json`.
17. Emit mitigation audit: `oya audit-chain emit --event-class EVT_CLOUD_KMS_HSM_CLUSTER_FAILOVER_INCIDENT --incident $INCIDENT_ID --field mitigation=failover-active`.
18. Keep isolated HSM offline until crypto operations completes vendor procedure.
19. Keep cryptoshred paused until quorum and receipts are verified.
20. Keep rotation paused until decrypt and sign canaries are stable.

## Resolution
1. Replace or recover failed HSM through vendor procedure.
2. Restore measured boot baseline only after crypto operations signs off.
3. Patch HSM client pool if failover selection caused retries to unhealthy nodes.
4. Patch receipt writer if failover completed without audit receipts.
5. Patch peer-region fallback if tenant routing was incorrect.
6. Patch rotation guard if rotation continued while quorum was below minimum.
7. Add regression fixture for HSM node timeout and peer failover.
8. Add regression fixture for invalid attestation isolation.
9. Run domain tests: `cargo test -p oya-cloud-kms-domain hsm -- --nocapture`.
10. Run API tests: `cargo test -p oya-cloud-kms-api hsm_failover -- --nocapture`.
11. Run production gate: `cargo run -p oya-dev-cli -- gate validate cloud-kms-hsm --production-snapshot --cell $CELL`.
12. Re-enable rotation: `oya flags set oya.cloud_kms.rotation.pause=false --cell $CELL --reason resolved-$INCIDENT_ID`.
13. Re-enable cryptoshred: `oya flags set oya.cloud_kms.cryptoshred.pause=false --cell $CELL --reason resolved-$INCIDENT_ID`.
14. Close breaker: `oya ops breaker close cloud-kms-hsm-cluster --cell $CELL --reason resolved-$INCIDENT_ID`.
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_KMS_HSM_CLUSTER_FAILOVER_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudKmsHsmClusterDegradedCritical` is green for 30 minutes.
- `CloudKmsHsmOperationTimeoutBurn` is green for 30 minutes.
- `oya_cloud_kms_hsm_available_nodes` meets tenant_class quorum.
- `oya_cloud_kms_decrypt_error_ratio` is below 0.001.
- `oya_cloud_kms_sign_error_ratio` is below 0.001.
- HSM attestation receipts validate for active nodes.
- Decrypt canary passes in the affected cell.
- Sign canary passes in the affected cell.
- Cryptoshred and rotation queues are unpaused only after quorum is healthy.
- Audit-chain contains failover started, completed, mitigation, and resolution events.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-kms-hsm-cluster-failover
microservice: cloud-kms
event_class: EVT_CLOUD_KMS_HSM_CLUSTER_FAILOVER_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# HSM Cluster Failover postmortem

## Summary
- Which HSM cluster, tier, vendor, and cell failed.
- Which operations were affected: decrypt, sign, rotate, cryptoshred, tenant CA.
- Whether peer region or peer vendor failover was used.

## Timeline
- First HSM error:
- Quorum lost:
- Failover started:
- Failover completed:
- Receipts sealed:

## Customer Impact
- Tenants affected:
- Decrypt failures:
- Sign failures:
- Added latency:

## Root Cause
- Hardware:
- Attestation:
- Client pool:
- Network:
- Receipt writer:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Dashboard or alert update:
```

## Escalation Path
- Page `oya-cloud-kms-primary` for any HSM cluster degradation.
- Page `oya-crypto-operations-primary` for attestation drift, quorum loss, or HSM isolation.
- Page `oya-cloud-iam-primary` when session signing depends on affected keys.
- Page `oya-cloud-network-primary` when tenant CA issuance fails.
- Page `oya-intelligence-primary` when artifact signing fails.
- Notify `#inc-cloud-kms` with cell, tier, vendor, and tenant scope.
- Notify `#support-cloud-kms-tenant-impact` for decrypt or sign impact.
- Notify compliance when regulated tenants lose FIPS-backed operation.
- Escalate to executive incident commander when more than one cell loses quorum.
- Engage vendor support only through crypto operations.

## Cross-µservice Coordination
- `cloud-iam`: verify session-token signing and authorization token flows.
- `cloud-network`: verify tenant CA and mTLS certificate signing.
- `foundry`: pause artifact signing pipelines that use affected CMKs.
- `audit-chain`: seal HSM operation receipts and failover events.
- `tenancy`: identify affected regulated tenant tiers.
- `cloud-billing`: annotate SLA-credit candidates for cryptographic unavailability.
- `observability`: attach HSM and crypto dashboards to evidence.
- `comms-email`: send tenant notifications.
- `security`: review attestation drift and tamper flags.
- `compliance`: review FIPS/FedRAMP/SOG-IS reporting obligations.
- `support`: tag cases with `cloud-kms.hsm-failover.customer-visible`.
- `workflow-engine`: pause workflows requiring key creation or cryptoshred.

## Runbook Maintenance
- Add new HSM vendor failure signatures after every incident.
- Keep quorum thresholds aligned with tenant_class matrix.
- Keep cryptoshred pause rule explicit.
- Keep peer-region fallback commands tested in drills.
- Review this runbook during quarterly HSM ceremony rehearsal.
