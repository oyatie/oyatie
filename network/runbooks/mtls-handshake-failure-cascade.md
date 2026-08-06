---
doc_class: Runbook
title: mTLS Handshake Failure Cascade
status: Accepted
date: 2026-05-20
microservice: cloud-network
severity: sev1
audience: sre, network-engineer, security-engineer
owner_team: axis-cloud + network-operations + security-governance
doc_status: published
---

# Runbook: mTLS Handshake Failure Cascade

## Operator Contract
- Runbook id: cloud-network-mtls-handshake-failure-cascade.
- Primary namespace: `cloud-network`.
- Owning rotation: PagerDuty `oya-cloud-network-primary`.
- Security secondary: PagerDuty `oya-security-policy-primary`.
- Incident channel: `#inc-cloud-network`.
- Customer channel: `#support-mtls-connectivity`.
- Protected surface: SPIFFE SVIDs, Cilium mTLS, Envoy ingress, tenant CA, certificate rotation, east-west L7 policy.
- Certificate authority: cloud-kms signs tenant CA and client cert material.
- Identity authority: cloud-iam binds workload principals.
- Safety invariant: do not disable mTLS to restore traffic.
- Rotation invariant: expired SVIDs should rotate, not be accepted.
- Stop condition: handshakes recover, certificate rotation is healthy, and no plaintext bypass is active.
- Evidence event: `EVT_CLOUD_NETWORK_MTLS_HANDSHAKE_CASCADE_INCIDENT`.
- Handoff API: `https://cloud-network.internal.oyatie.dev/v1/mtls/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-network-substrate/mtls?orgId=1&var-cell=prod-us-east-1`.
- Certificate dashboard: `https://grafana.dev.oyatie.internal/d/cloud-network-substrate/certificates?orgId=1&var-surface=spiffe`.
- Loki query: `{namespace="cloud-network",runbook="mtls-handshake-failure-cascade"}`.
- Canonical FAQ: `microservices/cloud-network/faqs/network-engineer-faq.md`.
- Related action: `cloud_network::Action::ManageEgressIpAllowlist`.
- Related dependency: `cloud-kms` tenant CA.
- Related dependency: `cloud-iam` workload principals.

## Trigger Conditions
- Alert `CloudNetworkMtlsHandshakeFailureCritical` fires.
- Alert `CloudNetworkSvidRotationLagHigh` fires.
- Alert `CloudNetworkTenantCaSigningFailure` fires.
- Alert `CloudNetworkEnvoyTlsErrorRatioHigh` fires.
- Alert `CloudNetworkCiliumMtlsPolicyDrift` fires.
- Metric `oya_cloud_network_mtls_handshake_failure_ratio` exceeds 0.01.
- Metric `oya_cloud_network_svid_rotation_lag_seconds` exceeds 600.
- Metric `oya_cloud_network_tenant_ca_sign_error_total` increases.
- Metric `oya_cloud_network_envoy_tls_handshake_error_total` spikes.
- Metric `oya_cloud_network_spiffe_identity_missing_total` is non-zero.
- Metric `oya_cloud_network_cert_expiry_seconds_min` is below 900.
- Metric `oya_cloud_network_cilium_mtls_policy_drift_total` is non-zero.
- Tenant reports service-to-service calls failing with TLS errors.
- Cloud-compute reports pods healthy but readiness probes fail through service mesh.
- Cloud-iam reports workload principal attestation drift.
- Cloud-kms reports tenant CA signing timeout.
- Envoy logs contain `tls: bad certificate` or `unknown ca`.
- Cilium logs contain L7 policy allow but TLS handshake failure.
- SVID rotation controller deploy happened recently.
- Certificate bundle config changed recently.

## Symptoms
- East-west HTTP calls fail before application handler.
- Envoy reports `SSLV3_ALERT_CERTIFICATE_EXPIRED`.
- Envoy reports `UNKNOWN_CA`.
- Envoy reports `CERTIFICATE_VERIFY_FAILED`.
- Cilium flow logs show L7 policy allow followed by connection reset.
- Workloads with newly rotated SVIDs work while old workloads fail.
- Workloads in one node pool fail because SPIRE agent is stale.
- Tenant CA bundle differs across nodes.
- `spiffe_id` is missing from workload identity map.
- Certificate expiry dashboard shows cliff near zero.
- `svid_rotation_status=stalled` appears in controller logs.
- `tenant_ca_sign_status=timeout` appears in network controller logs.
- mTLS failure cascades into cloud-iam login callbacks or cloud-kms calls.
- HTTP/3 ingress remains healthy for public traffic.
- Private service endpoints fail for affected tenant.
- Restarting app pod temporarily fixes handshake by forcing SVID refresh.
- Security impact is high when operators request plaintext bypass.
- Availability impact is high when all service-to-service traffic fails.
- Severity rises to Sev0 if plaintext bypass is enabled.
- Severity remains Sev1 if failures are fail-closed.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-network-mtls-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export SERVICE=synthetic-echo`.
3. Acknowledge page: `pd incident ack --service cloud-network --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-cloud-network --severity sev1`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.surface=="mtls")'`.
6. Query handshake ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_mtls_handshake_failure_ratio{cell="'$CELL'"}'`.
7. Query SVID lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_svid_rotation_lag_seconds{tenant_id="'$TENANT'"}'`.
8. Query CA sign errors: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_cloud_network_tenant_ca_sign_error_total[5m])'`.
9. Query TLS errors: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_cloud_network_envoy_tls_handshake_error_total[5m])'`.
10. Query expiry: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_cert_expiry_seconds_min{tenant_id="'$TENANT'"}'`.
11. Open mTLS dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-network-substrate/mtls?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
12. Open certificate dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-network-substrate/certificates?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
13. Read Envoy logs: `kubectl -n cloud-network logs deploy/envoy-gateway --since=30m | rg "TLS|certificate|handshake|UNKNOWN_CA|expired"`.
14. Read Cilium logs: `kubectl -n kube-system logs ds/cilium --since=30m | rg "mtls|spiffe|certificate|l7"`.
15. Read SPIRE logs: `kubectl -n spire logs deploy/spire-server --since=30m | rg "svid|rotation|attestation|tenant_ca"`.
16. Check SPIRE agents: `kubectl -n spire get pods -l app=spire-agent -o wide`.
17. Check workload SVID: `oya network spiffe svid inspect --tenant $TENANT --service $SERVICE --cell $CELL --output json`.
18. Check tenant CA: `oya network tenant-ca status --tenant $TENANT --cell $CELL --output json`.
19. Check CA signing path: `oya kms sign-key status --purpose tenant-ca --tenant $TENANT --cell $CELL --output json`.
20. Check workload principal: `oya iam workload attest --tenant $TENANT --service $SERVICE --cell $CELL --output json`.
21. Check Cilium mTLS policy: `oya network mtls policy get --tenant $TENANT --service $SERVICE --cell $CELL --output yaml`.
22. Check Cilium policy drift: `oya network mtls policy drift --tenant $TENANT --cell $CELL --output json`.
23. Check certificate bundle hash: `oya network cert-bundle hash --tenant $TENANT --cell $CELL --output table`.
24. Run mTLS canary: `oya ops probe cloud-network mtls --tenant $TENANT --service $SERVICE --cell $CELL --output json`.
25. Run peer canary: `oya ops probe cloud-network mtls --tenant $TENANT --service $SERVICE --from-node-pool all --cell $CELL`.
26. Query flow logs: `oya network flow query --tenant $TENANT --service $SERVICE --since 30m --protocol tls --output json`.
27. Check recent cert rollout: `oya network cert rollout history --tenant $TENANT --cell $CELL --limit 20`.
28. Check recent policy rollout: `oya network policy history --tenant $TENANT --cell $CELL --limit 20`.
29. Check plaintext bypass flags: `oya flags get oya.cloud_network.mtls.plaintext_bypass --tenant $TENANT --cell $CELL --output yaml`.
30. Query audit cert events: `oya audit-chain query --event-class cloud_network.svid.rotated --tenant $TENANT --since 24h`.
31. Query audit CA events: `oya audit-chain query --event-class cloud_network.tenant_ca.signed --tenant $TENANT --since 24h`.
32. Check cloud-kms impact: `oya ops dependency impact --source cloud-kms --target cloud-network --surface tenant-ca --since 30m`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-network --runbook mtls-handshake-failure-cascade --output evidence/incidents/$INCIDENT_ID.json`.
34. Export cert state: `oya network cert-bundle export --tenant $TENANT --cell $CELL --output evidence/incidents/$INCIDENT_ID-certs.json`.
35. Export failed handshakes: `oya network mtls failures export --tenant $TENANT --cell $CELL --since 30m --output evidence/incidents/$INCIDENT_ID-handshakes.json`.

### Diagnostic Decision Tree
```text
1. Is plaintext bypass enabled?
   |-- yes: disable immediately and raise severity.
   |-- no: continue fail-closed triage.
2. Are certificates expired or near expiry?
   |-- yes: repair SVID rotation and tenant CA signing.
   |-- no: inspect bundle trust and policy drift.
3. Is tenant CA signing failing?
   |-- yes: invoke cloud-kms signer path.
   |-- no: inspect SPIRE agent and workload identity.
4. Is failure isolated to one node pool?
   |-- yes: restart or repair SPIRE agents in that pool.
   |-- no: inspect tenant CA bundle and Envoy rollout.
5. Does mTLS canary pass after rotation?
   |-- yes: watch handshake ratio for 30 minutes.
   |-- no: escalate to security and network operations.
```

## Mitigation
1. Disable plaintext bypass: `oya flags set oya.cloud_network.mtls.plaintext_bypass=false --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
2. Hold mTLS policy deploys: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
3. Freeze cert rollout automation: `oya flags set oya.cloud_network.cert_rollout.auto=false --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
4. Refresh SVID dry-run: `oya network spiffe svid rotate --tenant $TENANT --service $SERVICE --cell $CELL --dry-run`.
5. Refresh SVID confirmed: `oya network spiffe svid rotate --tenant $TENANT --service $SERVICE --cell $CELL --confirm $INCIDENT_ID`.
6. Refresh tenant CA bundle: `oya network tenant-ca bundle refresh --tenant $TENANT --cell $CELL --confirm $INCIDENT_ID`.
7. Restart stale SPIRE agents: `kubectl -n spire rollout restart daemonset/spire-agent`.
8. Restart Envoy only after bundle refresh: `kubectl -n cloud-network rollout restart deploy/envoy-gateway`.
9. Roll back bad mTLS policy: `oya network mtls policy rollback --tenant $TENANT --service $SERVICE --to-version <n> --confirm $INCIDENT_ID`.
10. Roll back bad cert bundle: `oya network cert rollout rollback --tenant $TENANT --cell $CELL --to-version <n> --confirm $INCIDENT_ID`.
11. Open cloud-kms signer breaker only if sign path is failing: `oya ops breaker open cloud-network-tenant-ca-signing --cell $CELL --ttl 30m --reason $INCIDENT_ID`.
12. Rate-limit retries: `oya ops rate-limit set --tenant $TENANT --surface cloud-network.mtls --rps 100 --ttl 30m`.
13. Notify dependent services: `oya notify service-owner --incident $INCIDENT_ID --microservice cloud-iam,cloud-kms,cloud-compute`.
14. Notify support: `oya notify support --incident $INCIDENT_ID --template mtls-handshake-degraded`.
15. Notify tenant admin when service-to-service impact is visible: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template mtls-handshake-degraded`.
16. Emit mitigation audit: `oya audit-chain emit --event-class EVT_CLOUD_NETWORK_MTLS_HANDSHAKE_CASCADE_INCIDENT --incident $INCIDENT_ID --field mitigation=svid-rotation-active`.
17. Keep expired certificates refused.
18. Keep mTLS policy enforcement on.
19. Keep service discovery unchanged unless route evidence demands it.
20. Keep failed handshake evidence attached.

## Resolution
1. Patch SVID rotation controller if rotation lag caused expiry.
2. Patch tenant CA signing integration if cloud-kms signer timed out.
3. Patch trust-bundle rollout if nodes had inconsistent CA bundle.
4. Patch SPIRE agent health detection if one node pool lagged.
5. Patch Envoy TLS config if ALPN or CA validation changed incorrectly.
6. Patch Cilium policy projection if mTLS policy drifted.
7. Add regression fixture for expired SVID fail-closed.
8. Add regression fixture for stale trust bundle.
9. Run domain tests: `cargo test -p oya-cloud-network-domain mtls -- --nocapture`.
10. Run LB API tests: `cargo test -p oya-cloud-network-lb-api cloud_network_lb_api -- --nocapture`.
11. Run production gate: `cargo run -p oya-dev-cli -- gate validate cloud-network-mtls --production-snapshot --cell $CELL`.
12. Verify mTLS canary: `oya ops probe cloud-network mtls --tenant $TENANT --service $SERVICE --cell $CELL --expect healthy`.
13. Re-enable cert automation: `oya flags set oya.cloud_network.cert_rollout.auto=true --tenant $TENANT --cell $CELL --reason resolved-$INCIDENT_ID`.
14. Unhold deploys: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_NETWORK_MTLS_HANDSHAKE_CASCADE_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudNetworkMtlsHandshakeFailureCritical` is green.
- `oya_cloud_network_mtls_handshake_failure_ratio` is below 0.001.
- `oya_cloud_network_svid_rotation_lag_seconds` is below 120.
- `oya_cloud_network_cert_expiry_seconds_min` is above tier floor.
- Tenant CA signing succeeds.
- SPIFFE SVID inspection returns valid identity.
- mTLS canary passes from all node pools.
- Plaintext bypass flag is false.
- Audit-chain contains SVID rotation, tenant CA, mitigation, and resolution events.
- Support reports no new mTLS cases.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-network-mtls-handshake-failure-cascade
microservice: cloud-network
event_class: EVT_CLOUD_NETWORK_MTLS_HANDSHAKE_CASCADE_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# mTLS Handshake Failure Cascade postmortem

## Summary
- Which tenant, service, cell, and certificate path failed.
- Whether failure was SVID, tenant CA, trust bundle, Envoy, or policy.
- Whether plaintext bypass was requested or enabled.

## Timeline
- First TLS alert:
- SVID rotation started:
- Bundle refreshed:
- Canary passed:
- Audit sealed:

## Customer Impact
- Services affected:
- Calls failed:
- Duration:
- Security posture:

## Root Cause
- Rotation:
- Tenant CA:
- Trust bundle:
- SPIRE agent:
- Envoy or Cilium:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Dashboard or alert update:
```

## Escalation Path
- Page `oya-cloud-network-primary` for mTLS handshake failures.
- Page `oya-security-policy-primary` if plaintext bypass is requested or enabled.
- Page `oya-cloud-kms-primary` if tenant CA signing fails.
- Page `oya-cloud-iam-primary` if workload identity is missing.
- Page `oya-cloud-compute-primary` if node-pool SPIRE agent health is bad.
- Notify `#inc-cloud-network` with tenant, service, and cell.
- Notify `#support-mtls-connectivity` before tenant-facing copy.
- Notify `#compliance-review` if plaintext bypass occurred.
- Escalate to executive incident commander if multiple regulated tenants are affected.
- Keep fail-closed posture unless security commander explicitly records exception.

## Cross-µservice Coordination
- `cloud-kms`: sign tenant CA and validate key health.
- `cloud-iam`: verify workload principals and SPIFFE identity binding.
- `cloud-compute`: verify pods, node pools, and SPIRE agents.
- `audit-chain`: seal SVID, tenant CA, mitigation, and resolution events.
- `tenancy`: verify tenant tier and certificate rotation cadence.
- `security`: own plaintext bypass decisions.
- `support`: manage service-to-service connectivity cases.
- `observability`: attach mTLS and certificate dashboards.
- `comms-email`: send tenant degradation and all-clear notices.
- `foundry`: pause cert or policy deploys while hold is active.
- `workflow-engine`: pause workflows requiring affected service calls.
- `compliance`: review regulated mTLS control impact.

## Runbook Maintenance
- Add new TLS error signatures after every incident.
- Keep SPIRE and Cilium commands aligned with installed versions.
- Keep plaintext bypass prohibition explicit.
- Review this runbook after every certificate rotation policy change.
- Add every new service-mesh component to Diagnostic Steps.
