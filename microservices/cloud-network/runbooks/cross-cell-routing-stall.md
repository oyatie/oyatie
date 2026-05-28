---
doc_class: Runbook
title: Cross Cell Routing Stall
status: Accepted
date: 2026-05-20
microservice: cloud-network
severity: sev1
audience: sre, network-engineer, incident-commander
owner_team: axis-cloud + network-operations + ops-sre-reliability
doc_status: published
---

# Runbook: Cross Cell Routing Stall

## Operator Contract
- Runbook id: cloud-network-cross-cell-routing-stall.
- Primary namespace: `cloud-network`.
- Owning rotation: PagerDuty `oya-cloud-network-primary`.
- Cell secondary: PagerDuty `oya-cell-operations-primary`.
- Incident channel: `#inc-cloud-network`.
- Customer channel: `#support-network-routing`.
- Protected surface: per-tenant VPCs, Cilium clustermesh, cross-region peering, service discovery, ingress, egress, route health.
- Routing invariant: fail closed for untagged tenant traffic.
- Residency invariant: never route around a residency boundary without Cedar permit.
- Safety invariant: do not collapse tenant isolation to restore routing.
- Stop condition: route convergence is healthy, affected tenants pass canaries, and audit-chain has sealed route-change evidence.
- Evidence event: `EVT_CLOUD_NETWORK_CROSS_CELL_ROUTING_STALL_INCIDENT`.
- Handoff API: `https://cloud-network.internal.oyatie.dev/v1/routing/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-network-substrate/cross-cell-routing?orgId=1&var-cell=prod-us-east-1`.
- Flow dashboard: `https://grafana.dev.oyatie.internal/d/cloud-network-substrate/flow-logs?orgId=1&var-surface=cross-cell`.
- Loki query: `{namespace="cloud-network",runbook="cross-cell-routing-stall"}`.
- Canonical FAQ: `microservices/cloud-network/faqs/network-engineer-faq.md`.
- Related tutorial: `microservices/cloud-network/tutorials/provision-vpc-mtls-and-cedar-policy.md`.
- Related action: `cloud_network::Action::EnablePrivateServiceEndpoint`.
- Related action: `cloud_network::Action::RollbackPolicy`.

## Trigger Conditions
- Alert `CloudNetworkCrossCellRoutingStallCritical` fires.
- Alert `CloudNetworkRouteConvergenceLagHigh` fires for 10 minutes.
- Alert `CloudNetworkCiliumClusterMeshUnhealthy` fires.
- Alert `CloudNetworkResidencyRouteViolation` fires.
- Alert `CloudNetworkServiceDiscoveryStale` fires.
- Metric `oya_cloud_network_cross_cell_route_success_ratio` drops below 0.995.
- Metric `oya_cloud_network_route_convergence_lag_seconds` exceeds 120.
- Metric `oya_cloud_network_cilium_clustermesh_unhealthy_nodes` is non-zero.
- Metric `oya_cloud_network_service_discovery_stale_endpoint_total` increases.
- Metric `oya_cloud_network_residency_route_refusal_total` increases unexpectedly.
- Metric `oya_cloud_network_cross_cell_packet_drop_total` spikes.
- Metric `oya_cloud_network_bgp_peer_down_total` is non-zero.
- Metric `oya_cloud_network_wireguard_peer_handshake_age_seconds` exceeds 300.
- Tenant reports cross-region service unavailable.
- Cloud-compute reports pods healthy but service unreachable from another cell.
- Cloud-iam reports authorization succeeds but network connect fails.
- Cloud-kms reports tenant CA reachable in one cell but not peer cell.
- Cilium clustermesh reports endpoint identity sync lag.
- DNS service discovery returns stale cell endpoint.
- Audit-chain lacks route promotion event after policy change.

## Symptoms
- Ingress to local cell works but peer cell calls fail.
- Cross-region requests time out after DNS resolution.
- Cilium identity cache is stale for one tenant.
- `route_convergence_status=stalled` appears in controller logs.
- `clustermesh_status=degraded` appears in Cilium logs.
- `residency_refusal=true` appears for traffic that should be in-region.
- `wireguard_last_handshake_seconds` grows for peer cell.
- `bgp_session_state=idle` appears for one peering link.
- `service_discovery_endpoint_age_seconds` exceeds TTL.
- Tenant socket cookie tags are present but peer enforcer drops packets.
- Untagged packets are routed to quarantine network.
- HTTP/3 ingress remains healthy but east-west traffic fails.
- Private service endpoint traffic works inside a cell but fails across cells.
- Flow logs show drops at L3/L4 rather than application 5xx.
- DNS cache points to decommissioned endpoint.
- A recent CiliumNetworkPolicy rollback changed cross-cell allow rules.
- A recent tenant region add did not provision cross-region peering.
- Customer impact is tenant-visible availability degradation.
- Severity rises to Sev0 if residency boundary is bypassed.
- Severity remains Sev1 if traffic is refused rather than misrouted.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-network-cross-cell-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export PEER_CELL=prod-us-west-2; export TENANT=synthetic-canary`.
3. Acknowledge page: `pd incident ack --service cloud-network --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-cloud-network --severity sev1`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.surface=="cross-cell-routing")'`.
6. Query route success: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_cross_cell_route_success_ratio{cell="'$CELL'",peer_cell="'$PEER_CELL'"}'`.
7. Query convergence lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_route_convergence_lag_seconds{cell="'$CELL'"}'`.
8. Query clustermesh nodes: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_cilium_clustermesh_unhealthy_nodes{cell="'$CELL'"}'`.
9. Query packet drops: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_cloud_network_cross_cell_packet_drop_total[5m])'`.
10. Query WireGuard handshakes: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_wireguard_peer_handshake_age_seconds{cell="'$CELL'",peer_cell="'$PEER_CELL'"}'`.
11. Open routing dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-network-substrate/cross-cell-routing?orgId=1&var-cell=$CELL&var-peer=$PEER_CELL&var-tenant=$TENANT"`.
12. Open flow dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-network-substrate/flow-logs?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
13. Read controller logs: `kubectl -n cloud-network logs deploy/cloud-network-route-controller --since=60m | rg "route|clustermesh|residency|convergence"`.
14. Read Cilium status: `kubectl -n kube-system exec ds/cilium -- cilium status --verbose`.
15. Check clustermesh: `kubectl -n kube-system exec ds/cilium -- cilium clustermesh status --verbose`.
16. Check Cilium endpoints: `kubectl -n kube-system exec ds/cilium -- cilium endpoint list | rg "$TENANT|$PEER_CELL"`.
17. Check Cilium identities: `kubectl -n kube-system exec ds/cilium -- cilium identity list | rg "$TENANT"`.
18. Check service discovery: `oya network service-discovery status --tenant $TENANT --cell $CELL --peer-cell $PEER_CELL --output json`.
19. Check route table: `oya network route table --tenant $TENANT --cell $CELL --peer-cell $PEER_CELL --output table`.
20. Check peering: `oya network peering status --tenant $TENANT --cell $CELL --peer-cell $PEER_CELL --output json`.
21. Check BGP: `oya network bgp peers --cell $CELL --peer-cell $PEER_CELL --output table`.
22. Check WireGuard: `oya network wireguard peers --tenant $TENANT --cell $CELL --peer-cell $PEER_CELL --output json`.
23. Run cross-cell canary: `oya ops probe cloud-network cross-cell --tenant $TENANT --from $CELL --to $PEER_CELL --output json`.
24. Run service canary: `oya ops probe cloud-network service-route --tenant $TENANT --service synthetic-echo --from $CELL --to $PEER_CELL`.
25. Query flow logs: `oya network flow query --tenant $TENANT --from-cell $CELL --to-cell $PEER_CELL --since 30m --output json`.
26. Check residency policy: `oya network residency route-check --tenant $TENANT --from $CELL --to $PEER_CELL --output json`.
27. Check Cedar policy: `oya iam authz explain --tenant $TENANT --action cloud_network::Action::CrossCellRoute --resource $PEER_CELL`.
28. Check policy rollout: `oya network policy history --tenant $TENANT --cell $CELL --limit 20`.
29. Check region declaration: `oya tenancy regions get --tenant $TENANT --output yaml`.
30. Check audit route events: `oya audit-chain query --event-class cloud_network.route.promoted --tenant $TENANT --since 24h`.
31. Check rollback events: `oya audit-chain query --event-class cloud_network.policy.rollback.applied --tenant $TENANT --since 24h`.
32. Check cloud-iam dependency: `oya ops dependency impact --source cloud-network --target cloud-iam --surface auth-callback --since 30m`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-network --runbook cross-cell-routing-stall --output evidence/incidents/$INCIDENT_ID.json`.
34. Export route state: `oya network route export --tenant $TENANT --cell $CELL --peer-cell $PEER_CELL --output evidence/incidents/$INCIDENT_ID-routes.json`.
35. Export flow sample: `oya network flow export --tenant $TENANT --from-cell $CELL --to-cell $PEER_CELL --since 30m --output evidence/incidents/$INCIDENT_ID-flows.json`.

### Diagnostic Decision Tree
```text
1. Is there any residency route violation?
   |-- yes: keep traffic refused and page security/compliance.
   |-- no: continue availability triage.
2. Is Cilium clustermesh unhealthy?
   |-- yes: repair clustermesh and identity sync.
   |-- no: inspect BGP, WireGuard, and service discovery.
3. Is DNS or service discovery stale?
   |-- yes: refresh service discovery and clear bad endpoints.
   |-- no: inspect route policy and peering.
4. Did a network policy change precede the stall?
   |-- yes: rollback policy after evidence snapshot.
   |-- no: inspect infrastructure peering.
5. Does cross-cell canary pass after mitigation?
   |-- yes: watch route convergence for 30 minutes.
   |-- no: escalate to cell operations.
```

## Mitigation
1. Hold network policy deploys: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
2. Freeze route automation: `oya flags set oya.cloud_network.routing.auto_promote=false --cell $CELL --reason $INCIDENT_ID`.
3. Keep residency guard enabled: `oya flags set oya.cloud_network.residency.enforce=true --cell $CELL --reason $INCIDENT_ID`.
4. Refresh service discovery: `oya network service-discovery refresh --tenant $TENANT --cell $CELL --peer-cell $PEER_CELL --confirm $INCIDENT_ID`.
5. Resync Cilium identities: `oya network cilium identity-resync --tenant $TENANT --cell $CELL --peer-cell $PEER_CELL --dry-run`.
6. Confirm identity resync: `oya network cilium identity-resync --tenant $TENANT --cell $CELL --peer-cell $PEER_CELL --confirm $INCIDENT_ID`.
7. Repair clustermesh: `oya network cilium clustermesh repair --cell $CELL --peer-cell $PEER_CELL --dry-run`.
8. Confirm clustermesh repair: `oya network cilium clustermesh repair --cell $CELL --peer-cell $PEER_CELL --confirm $INCIDENT_ID`.
9. Reestablish WireGuard peer: `oya network wireguard peer refresh --tenant $TENANT --cell $CELL --peer-cell $PEER_CELL --confirm $INCIDENT_ID`.
10. Reestablish BGP peering: `oya network bgp peer reset --cell $CELL --peer-cell $PEER_CELL --confirm $INCIDENT_ID`.
11. Roll back bad policy: `oya network policy rollback --tenant $TENANT --policy <policy> --to-version <n> --confirm $INCIDENT_ID`.
12. Quarantine untagged flows: `oya network quarantine enable --tenant $TENANT --reason $INCIDENT_ID --ttl 60m`.
13. Rate-limit retries: `oya ops rate-limit set --tenant $TENANT --surface cloud-network.cross-cell --rps 50 --ttl 30m`.
14. Notify affected services: `oya notify service-owner --incident $INCIDENT_ID --microservice cloud-compute,cloud-iam,cloud-kms`.
15. Notify support: `oya notify support --incident $INCIDENT_ID --template cross-cell-routing-degraded`.
16. Emit mitigation audit: `oya audit-chain emit --event-class EVT_CLOUD_NETWORK_CROSS_CELL_ROUTING_STALL_INCIDENT --incident $INCIDENT_ID --field mitigation=route-repair-active`.
17. Keep tenant isolation intact.
18. Keep residency denials active.
19. Keep manual provider console edits forbidden.
20. Keep flow evidence attached.

## Resolution
1. Patch route controller if route convergence stalled.
2. Patch service discovery if stale endpoint TTL exceeded.
3. Patch Cilium identity projection if tenant tags were stale.
4. Patch WireGuard peer refresh if handshakes expired.
5. Patch BGP health detection if down peer was missed.
6. Patch residency policy if a valid route was falsely refused.
7. Add regression fixture for cross-cell route convergence.
8. Add regression fixture for stale service endpoint.
9. Run domain tests: `cargo test -p oya-cloud-network-domain routing -- --nocapture`.
10. Run VPC API tests: `cargo test -p oya-cloud-network-vpc-api cloud_network_vpc_api -- --nocapture`.
11. Run LB API tests if ingress changed: `cargo test -p oya-cloud-network-lb-api cloud_network_lb_api -- --nocapture`.
12. Run production gate: `cargo run -p oya-dev-cli -- gate validate cloud-network-routing --production-snapshot --cell $CELL`.
13. Re-enable route automation: `oya flags set oya.cloud_network.routing.auto_promote=true --cell $CELL --reason resolved-$INCIDENT_ID`.
14. Unhold deploys: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_NETWORK_CROSS_CELL_ROUTING_STALL_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudNetworkCrossCellRoutingStallCritical` is green.
- `oya_cloud_network_cross_cell_route_success_ratio` is above 0.995.
- `oya_cloud_network_route_convergence_lag_seconds` is below 60.
- Cilium clustermesh status is healthy.
- WireGuard peer handshake age is below 120 seconds.
- BGP peers are established.
- Cross-cell canary passes.
- Flow logs show expected allow or expected residency refusal.
- Audit-chain contains route repair and resolution events.
- Support reports no new routing cases for affected tenants.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-network-cross-cell-routing-stall
microservice: cloud-network
event_class: EVT_CLOUD_NETWORK_CROSS_CELL_ROUTING_STALL_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Cross Cell Routing Stall postmortem

## Summary
- Which tenant, cell, peer cell, and route surface stalled.
- Whether traffic was refused, dropped, or misrouted.
- Whether residency boundaries were involved.

## Timeline
- First route alert:
- Route repair started:
- Canary passed:
- Route automation restored:
- Audit sealed:

## Customer Impact
- Affected services:
- Affected tenants:
- Duration:
- Residency impact:

## Root Cause
- Clustermesh:
- Service discovery:
- BGP or WireGuard:
- Policy:
- Route controller:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Dashboard or alert update:
```

## Escalation Path
- Page `oya-cloud-network-primary` for routing stalls.
- Page `oya-cell-operations-primary` when multiple cells or cell fabric is affected.
- Page `oya-security-policy-primary` when residency or tenant isolation risk exists.
- Page `oya-cloud-compute-primary` when service endpoints are stale.
- Page `oya-audit-chain-primary` when route events do not seal.
- Notify `#inc-cloud-network` with tenant, cell, and peer cell.
- Notify `#support-network-routing` before tenant-facing messaging.
- Notify `#compliance-review` for residency route questions.
- Escalate to executive incident commander if more than one region pair is impacted.
- Keep all emergency route exceptions commander-approved.

## Cross-µservice Coordination
- `cloud-compute`: verify pod endpoints and workload health.
- `cloud-iam`: verify Cedar route authorization and tenant tags.
- `cloud-kms`: verify tenant CA availability for peer-cell TLS.
- `tenancy`: verify tenant allowed regions and residency pack.
- `audit-chain`: seal route, policy, mitigation, and resolution events.
- `observability`: attach routing and flow dashboards.
- `support`: manage tenant-visible routing cases.
- `comms-email`: send routing degradation and all-clear notices.
- `security`: review any isolation or residency concern.
- `compliance`: decide residency reporting obligations.
- `foundry`: pause network policy deploys while route automation is held.
- `workflow-engine`: pause workflows requiring cross-cell calls.

## Runbook Maintenance
- Add new route-controller failure signatures after every incident.
- Keep Cilium commands aligned with installed version.
- Keep residency guard warnings explicit.
- Review this runbook after each multi-region expansion.
- Add every new peer fabric to Diagnostic Steps.
