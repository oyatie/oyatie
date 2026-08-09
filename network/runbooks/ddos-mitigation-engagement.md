---
doc_class: Runbook
title: DDoS Mitigation Engagement
status: Accepted
date: 2026-05-20
microservice: cloud-network
severity: sev0
audience: sre, network-engineer, security-engineer, incident-commander
owner_team: axis-cloud + network-operations + security-governance
doc_status: published
---

# Runbook: DDoS Mitigation Engagement

## Operator Contract
- Runbook id: cloud-network-ddos-mitigation-engagement.
- Primary namespace: `cloud-network`.
- Owning rotation: PagerDuty `oya-cloud-network-primary`.
- Security secondary: PagerDuty `oya-security-policy-primary`.
- Incident channel: `#inc-ddos`.
- Customer channel: `#support-edge-availability`.
- Protected surface: L3/L4 volumetric protection, L7 Envoy rate limits, Cilium policy, provider Shield/Cloud Armor, emergency blackhole.
- Provider surfaces: AWS Shield Advanced, GCP Cloud Armor, Azure DDoS Protection, upstream scrubbing partner.
- Safety invariant: emergency blackhole requires commander approval and tenant impact record.
- Isolation invariant: tenant-specific mitigation must not spill into other tenant routes.
- Stop condition: attack traffic is mitigated, legitimate traffic SLO is green, provider case is closed or handed off, and audit evidence is sealed.
- Evidence event: `EVT_CLOUD_NETWORK_DDOS_MITIGATION_INCIDENT`.
- Handoff API: `https://cloud-network.internal.oyatie.dev/v1/ddos/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-network-substrate/ddos?orgId=1&var-cell=prod-us-east-1`.
- Edge dashboard: `https://grafana.dev.oyatie.internal/d/cloud-network-substrate/edge-l7?orgId=1&var-surface=ingress`.
- Loki query: `{namespace="cloud-network",runbook="ddos-mitigation-engagement"}`.
- Canonical FAQ: `microservices/cloud-network/faqs/network-engineer-faq.md`.
- Related action: `cloud_network::Action::ActivateL7RateLimit`.
- Related action: `cloud_network::Action::EmergencyBlackhole`.

## Trigger Conditions
- Alert `CloudNetworkDdosVolumetricCritical` fires.
- Alert `CloudNetworkL7DdosCritical` fires.
- Alert `CloudNetworkEdgeSloBurn` fires for public ingress.
- Alert `CloudNetworkProviderShieldEngagementRequired` fires.
- Alert `CloudNetworkEmergencyBlackholeSuggested` fires.
- Metric `oya_cloud_network_edge_packets_per_second` exceeds baseline by 10x.
- Metric `oya_cloud_network_edge_bits_per_second` exceeds provider threshold.
- Metric `oya_cloud_network_l7_request_rate` exceeds tenant baseline by 20x.
- Metric `oya_cloud_network_envoy_429_ratio` exceeds 0.2.
- Metric `oya_cloud_network_edge_5xx_ratio` exceeds 0.02.
- Metric `oya_cloud_network_syn_flood_score` exceeds threshold.
- Metric `oya_cloud_network_http_fingerprint_cardinality` spikes.
- Metric `oya_cloud_network_provider_scrubbed_traffic_bps` increases.
- Tenant reports public endpoint unavailable.
- Provider notifies volumetric attack.
- WAF sees high-cardinality hostile paths.
- Cilium L7 policy sees one principal or IP prefix hammering route.
- Upstream DNS sees query flood for tenant hostnames.
- Public egress IP reputation degrades.
- Emergency services or regulated workloads are impacted.

## Symptoms
- Edge latency and 5xx rise together.
- Ingress Envoy CPU is saturated.
- Provider DDoS dashboard shows attack vector.
- Large SYN flood targets tenant egress or ingress IP.
- L7 request rate comes from high-cardinality user agents.
- `ddos_vector=syn_flood` appears in edge logs.
- `ddos_vector=http_flood` appears in Envoy logs.
- `provider_mitigation_status=detecting` persists longer than 5 minutes.
- `l7_rate_limit_status=inactive` while L7 alert fires.
- Tenant routes share edge pool and neighboring tenants see latency.
- Legitimate traffic receives 429 because blunt rate limit was used.
- Attack traffic targets HTTP/3 or QUIC path.
- DNS query flood increases before HTTP flood.
- Flow logs show one country, ASN, or botnet pattern.
- Cilium drop logs rise but app pods remain healthy.
- Provider portal recommends scrubbing center engagement.
- Security asks for blackhole but customer support sees business-critical traffic.
- Customer impact is availability and trust.
- Severity is Sev0 when attack is active and customer-visible.
- Severity remains Sev1 if provider fully absorbs traffic without customer impact.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-network-ddos-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export HOST=app.synthetic.oyatie.dev`.
3. Acknowledge page: `pd incident ack --service cloud-network --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-ddos --severity sev0`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.surface=="ddos")'`.
6. Query packet rate: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_edge_packets_per_second{cell="'$CELL'"}'`.
7. Query bit rate: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_edge_bits_per_second{cell="'$CELL'"}'`.
8. Query L7 rate: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_l7_request_rate{tenant_id="'$TENANT'"}'`.
9. Query 429 ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_envoy_429_ratio{tenant_id="'$TENANT'"}'`.
10. Query 5xx ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_network_edge_5xx_ratio{tenant_id="'$TENANT'"}'`.
11. Open DDoS dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-network-substrate/ddos?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
12. Open edge dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-network-substrate/edge-l7?orgId=1&var-cell=$CELL&var-host=$HOST"`.
13. Read edge logs: `kubectl -n cloud-network logs deploy/envoy-gateway --since=30m | rg "ddos|rate_limit|429|fingerprint|http_flood"`.
14. Query flow logs: `oya network flow query --tenant $TENANT --host $HOST --since 30m --output json`.
15. Query WAF events: `oya security waf events --tenant $TENANT --host $HOST --since 30m --output json`.
16. Query DNS flood: `oya network dns query-rate --tenant $TENANT --host $HOST --since 30m --output json`.
17. Identify attack vector: `oya network ddos classify --tenant $TENANT --host $HOST --cell $CELL --output json`.
18. Identify top ASNs: `oya network ddos top-asn --tenant $TENANT --host $HOST --since 30m --limit 20`.
19. Identify top prefixes: `oya network ddos top-prefix --tenant $TENANT --host $HOST --since 30m --limit 50`.
20. Check provider mitigation: `oya network provider ddos status --tenant $TENANT --provider all --cell $CELL --output json`.
21. Open provider case draft: `oya network provider ddos case draft --tenant $TENANT --host $HOST --incident $INCIDENT_ID --output evidence/incidents/$INCIDENT_ID-provider-case.json`.
22. Check route blast radius: `oya network edge-pool tenants --host $HOST --cell $CELL --output table`.
23. Check legitimate traffic: `oya network ddos legitimate-traffic-estimate --tenant $TENANT --host $HOST --since 30m --output json`.
24. Check rate-limit state: `oya network l7 rate-limit status --tenant $TENANT --host $HOST --output yaml`.
25. Check blackhole eligibility: `oya network emergency-blackhole plan --tenant $TENANT --host $HOST --dry-run`.
26. Check Cedar authorization: `oya iam authz explain --tenant $TENANT --action cloud_network::Action::EmergencyBlackhole --resource $HOST`.
27. Check customer criticality: `oya tenancy workload criticality --tenant $TENANT --host $HOST --output json`.
28. Query audit mitigation events: `oya audit-chain query --event-class cloud_network.ddos.mitigation.applied --tenant $TENANT --since 24h`.
29. Query blackhole events: `oya audit-chain query --event-class cloud_network.emergency_blackhole.applied --tenant $TENANT --since 24h`.
30. Check support cases: `oya support cases list --tag edge-availability --tenant $TENANT --since 24h`.
31. Check status page: `oya statuspage incident status --service cloud-network --tenant $TENANT --output json`.
32. Check provider health: `oya network provider edge-health --tenant $TENANT --provider all --cell $CELL --output table`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-network --runbook ddos-mitigation-engagement --output evidence/incidents/$INCIDENT_ID.json`.
34. Export attack summary: `oya network ddos export --tenant $TENANT --host $HOST --since 30m --output evidence/incidents/$INCIDENT_ID-attack.json`.
35. Export flow sample: `oya network flow export --tenant $TENANT --host $HOST --since 30m --output evidence/incidents/$INCIDENT_ID-flows.json`.

### Diagnostic Decision Tree
```text
1. Is the attack customer-visible?
   |-- yes: keep Sev0 and engage provider mitigation.
   |-- no: keep Sev1 and monitor provider absorption.
2. Is traffic volumetric L3/L4?
   |-- yes: engage provider scrubbing and edge pool isolation.
   |-- no: inspect L7 fingerprint and WAF/rate-limit.
3. Will rate-limit harm legitimate traffic more than attack traffic?
   |-- yes: use targeted fingerprint, ASN, or prefix controls.
   |-- no: apply bounded L7 rate-limit.
4. Is emergency blackhole recommended?
   |-- yes: require incident commander and tenant impact approval.
   |-- no: keep route online with mitigation.
5. Is neighboring tenant impact present?
   |-- yes: move tenant to isolated edge pool.
   |-- no: continue tenant-specific mitigation.
```

## Mitigation
1. Engage provider mitigation: `oya network provider ddos engage --tenant $TENANT --host $HOST --incident $INCIDENT_ID --provider all`.
2. Apply targeted WAF rule dry-run: `oya security waf rule plan --tenant $TENANT --host $HOST --incident $INCIDENT_ID --dry-run`.
3. Apply targeted WAF rule confirmed: `oya security waf rule apply --tenant $TENANT --host $HOST --incident $INCIDENT_ID --confirm`.
4. Apply L7 rate limit: `oya network l7 rate-limit set --tenant $TENANT --host $HOST --rps <safe-rps> --ttl 30m --reason $INCIDENT_ID`.
5. Block hostile prefix dry-run: `oya network edge block-prefix --tenant $TENANT --host $HOST --prefix <cidr> --dry-run`.
6. Block hostile prefix confirmed: `oya network edge block-prefix --tenant $TENANT --host $HOST --prefix <cidr> --ttl 30m --confirm $INCIDENT_ID`.
7. Isolate tenant edge pool: `oya network edge-pool isolate --tenant $TENANT --host $HOST --reason $INCIDENT_ID --confirm`.
8. Enable provider scrubbing: `oya network provider ddos scrub --tenant $TENANT --host $HOST --provider all --confirm $INCIDENT_ID`.
9. Lower DNS TTL: `oya network dns ttl set --tenant $TENANT --host $HOST --ttl 60 --reason $INCIDENT_ID`.
10. Emergency blackhole dry-run: `oya network emergency-blackhole apply --tenant $TENANT --host $HOST --dry-run`.
11. Emergency blackhole confirmed only with commander: `oya network emergency-blackhole apply --tenant $TENANT --host $HOST --confirm $INCIDENT_ID`.
12. Notify support: `oya notify support --incident $INCIDENT_ID --template ddos-active`.
13. Notify tenant admin: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template ddos-mitigation-active`.
14. Notify status page owner: `oya statuspage incident update --service cloud-network --tenant $TENANT --message "DDoS mitigation active"`.
15. Hold risky network deploys: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
16. Emit mitigation audit: `oya audit-chain emit --event-class EVT_CLOUD_NETWORK_DDOS_MITIGATION_INCIDENT --incident $INCIDENT_ID --field mitigation=provider-engaged`.
17. Keep mitigation TTLs explicit.
18. Keep blackhole decision in incident channel.
19. Keep neighboring tenant blast radius checked every 10 minutes.
20. Keep provider case id in evidence.

## Resolution
1. Remove temporary WAF rules after attack subsides.
2. Remove prefix blocks after TTL or confirmed attack end.
3. Restore DNS TTL.
4. Move tenant back from isolated edge pool only after clean canary.
5. Close provider mitigation case after provider confirms attack ended.
6. Patch detection if attack was not classified quickly.
7. Patch rate-limit templates if legitimate traffic was over-throttled.
8. Patch edge pool isolation if neighboring tenants were impacted.
9. Run domain tests: `cargo test -p oya-cloud-network-domain ddos -- --nocapture`.
10. Run LB API tests: `cargo test -p oya-cloud-network-lb-api cloud_network_lb_api -- --nocapture`.
11. Run production gate: `cargo run -p oya-dev-cli -- gate validate cloud-network-ddos --production-snapshot --cell $CELL`.
12. Verify edge SLO: `oya ops watch --metric oya_cloud_network_edge_5xx_ratio --threshold 0.005 --window 30m --tenant $TENANT`.
13. Unhold deploys: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
14. Update status page all-clear: `oya statuspage incident update --service cloud-network --tenant $TENANT --message "DDoS mitigation resolved"`.
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_NETWORK_DDOS_MITIGATION_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudNetworkDdosVolumetricCritical` is green.
- `CloudNetworkL7DdosCritical` is green.
- `oya_cloud_network_edge_5xx_ratio` is below 0.005.
- `oya_cloud_network_envoy_429_ratio` returns to normal.
- Provider mitigation status is clean or handed off.
- Legitimate traffic canary passes.
- Neighboring tenant edge metrics are normal.
- Temporary mitigations have owner and expiry.
- Audit-chain contains mitigation and resolution events.
- Support and status page have all-clear updates.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-network-ddos-mitigation-engagement
microservice: cloud-network
event_class: EVT_CLOUD_NETWORK_DDOS_MITIGATION_INCIDENT
incident_id: <INC-...>
severity: sev0
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# DDoS Mitigation Engagement postmortem

## Summary
- Which tenant, host, edge pool, and attack vector were involved.
- Which provider mitigation was engaged.
- Whether blackhole, WAF, rate-limit, or edge isolation was used.

## Timeline
- Attack detected:
- Provider engaged:
- Mitigation active:
- Attack subsided:
- Temporary rules removed:

## Customer Impact
- Availability:
- Legitimate throttling:
- Neighboring tenants:
- Status page:

## Root Cause
- Attack vector:
- Detection gap:
- Mitigation gap:
- Communication gap:

## Corrective Actions
- Owner:
- Due date:
- Detection improvement:
- Runbook update:
```

## Escalation Path
- Page `oya-cloud-network-primary` for every active DDoS alert.
- Page `oya-security-policy-primary` for L7 or abusive principal analysis.
- Page `oya-cell-operations-primary` if edge pool or cell fabric is saturated.
- Page provider support when scrubbing or Shield/Cloud Armor engagement is required.
- Notify `#inc-ddos` with tenant, host, vector, and mitigation.
- Notify `#support-edge-availability` before tenant messages.
- Notify `#statuspage` when customer-visible.
- Notify `#legal-review` for law-enforcement or abuse reporting.
- Escalate to executive incident commander before emergency blackhole.
- Keep customer success informed for high-value tenant attacks.

## Cross-µservice Coordination
- `security`: classify attacker patterns and approve L7 controls.
- `cloud-iam`: suspend abusive principals if authenticated traffic is involved.
- `cloud-compute`: verify upstream pods are healthy after edge mitigation.
- `audit-chain`: seal mitigation, blackhole, and resolution events.
- `tenancy`: identify tenant criticality and contacts.
- `support`: manage tenant cases and impact wording.
- `comms-email`: send DDoS active and all-clear notices.
- `observability`: attach DDoS, edge, flow, and provider dashboards.
- `cloud-billing`: annotate SLA-credit candidates.
- `workflow-engine`: pause workflows that would amplify retries.
- `foundry`: pause edge config deploys during active attack.
- `compliance`: review regulated availability obligations.

## Runbook Maintenance
- Add new attack fingerprints after every incident.
- Keep provider engagement commands current.
- Keep emergency blackhole approval requirements explicit.
- Review this runbook before high-traffic launches.
- Add every new edge provider to Trigger Conditions.
