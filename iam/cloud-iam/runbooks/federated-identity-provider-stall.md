---
doc_class: Runbook
title: Federated Identity Provider Stall
status: Accepted
date: 2026-05-20
microservice: cloud-iam
severity: sev1
audience: sre, iam-engineer, tenant-admin-support
owner_team: axis-cloud + ops-sre-reliability + identity-security
doc_status: published
---

# Runbook: Federated Identity Provider Stall

## Operator Contract
- Runbook id: cloud-iam-federated-identity-provider-stall.
- Primary namespace: `cloud-iam`.
- Owning rotation: PagerDuty `oya-cloud-iam-primary`.
- Identity secondary: PagerDuty `oya-identity-federation-primary`.
- Incident channel: `#inc-cloud-iam`.
- Customer channel: `#support-idp-login-impact`.
- Protected flows: SAML ACS, OIDC callback, JIT provisioning, session-token issuance, Cedar entity materialisation.
- External IdPs: Okta, Entra ID, Google Workspace, PingFederate, tenant-owned SAML providers.
- Safety invariant: do not bypass IdP signature validation.
- Grace invariant: cached IdP metadata may be used only inside configured grace windows.
- Stop condition: login success returns to SLO, metadata freshness is green, and JIT queues are drained.
- Evidence event: `EVT_CLOUD_IAM_IDP_STALL_INCIDENT`.
- Handoff API: `https://cloud-iam.internal.oyatie.dev/v1/federation/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/federation?orgId=1&var-cell=prod-us-east-1`.
- JIT dashboard: `https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/jit-provisioning?orgId=1&var-cell=prod-us-east-1`.
- Loki query: `{namespace="cloud-iam",runbook="federated-identity-provider-stall"}`.
- Canonical docs: `iam/cloud-iam/tutorials/federate-okta-saml-and-issue-scoped-token.md`.
- Related FAQ: `iam/cloud-iam/faqs/iam-engineer-faq.md`.
- Related migration guide: `iam/cloud-iam/migration-playbooks/from-okta-and-aws-iam.md`.

## Trigger Conditions
- Alert `CloudIamFederationLoginSuccessBurn` fires for any production cell.
- Alert `CloudIamIdpMetadataExpired` fires for one tenant.
- Alert `CloudIamSamlAcsLatencyHigh` fires for 10 minutes.
- Alert `CloudIamOidcCallbackErrorRatioHigh` fires for 10 minutes.
- Alert `CloudIamJitProvisioningQueueStalled` fires.
- Metric `oya_cloud_iam_federation_login_success_ratio` drops below 0.995.
- Metric `oya_cloud_iam_saml_assertion_validation_error_total` spikes.
- Metric `oya_cloud_iam_oidc_jwks_fetch_error_total` spikes.
- Metric `oya_cloud_iam_idp_metadata_age_seconds` exceeds 82800.
- Metric `oya_cloud_iam_jit_provisioning_queue_depth` exceeds 500.
- Metric `oya_cloud_iam_jit_provisioning_lag_seconds` exceeds 300.
- Metric `oya_cloud_iam_session_token_issue_error_ratio` exceeds 0.01.
- Support case is tagged `cloud-iam.federation.login-stall`.
- A tenant admin reports that all SSO users are blocked.
- A tenant admin reports that one IdP works while another IdP fails.
- Provider status page shows Okta, Entra, or Google Workspace outage.
- Audit-chain shows `cloud_iam.federation.login.failed` without matching `session.issued`.
- Synthetic probe `oya ops probe cloud-iam federation-login` fails twice.
- CI snapshot gate `cloud-iam-federation` fails against production snapshot.
- IdP metadata signing certificate is inside the 24 hour hard-expiry window.

## Symptoms
- Users loop between tenant IdP and Oyatie callback.
- SAML ACS returns `InvalidSignature` for one tenant.
- OIDC callback returns `JwksKeyNotFound` after IdP key rotation.
- Session token issuance times out after assertion validation succeeds.
- JIT provisioning creates `User` principal but misses role attachments.
- Cedar entity store lacks the expected `User::"<tenant>/<external_id>"`.
- The same user can authenticate through one IdP but not another.
- `metadata_grace_active=true` appears in structured logs.
- `idp_metadata_cert_expired=true` appears in structured logs.
- `jit_rule_match=none` appears for a tenant that previously matched.
- `external_id_conflict=true` appears for merger or dual-IdP tenants.
- `cloud_iam.federation.login.failed` grows while `cloud_iam.session.issued` is flat.
- JIT worker pods show high queue but low CPU.
- Metadata refresh worker shows provider-side TLS errors.
- Tenant-specific impact dominates federation metrics.
- Cell-wide impact affects all IdPs and points to cloud-iam callback path.
- Okta-specific impact usually presents as SAML assertion validation failures.
- Entra-specific impact usually presents as OIDC JWKS or issuer mismatch.
- Google Workspace impact often presents as group claim shape drift.
- PingFederate impact often presents as metadata endpoint timeout.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-iam-idp-stall-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export IDP=okta`.
3. Acknowledge page: `pd incident ack --service cloud-iam --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-cloud-iam --severity sev1`.
5. Query federation alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.service=="cloud-iam" and .labels.surface=="federation")'`.
6. Query login success: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_iam_federation_login_success_ratio{cell="'$CELL'"}'`.
7. Query SAML failures: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_cloud_iam_saml_assertion_validation_error_total[5m])'`.
8. Query OIDC failures: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_cloud_iam_oidc_callback_error_total[5m])'`.
9. Query metadata age: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_iam_idp_metadata_age_seconds{tenant_id="'$TENANT'",idp="'$IDP'"}'`.
10. Query JIT lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_iam_jit_provisioning_lag_seconds{tenant_id="'$TENANT'"}'`.
11. Open federation dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/federation?orgId=1&var-cell=$CELL&var-tenant=$TENANT&var-idp=$IDP"`.
12. Open JIT dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/jit-provisioning?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
13. Read ACS logs: `kubectl -n cloud-iam logs deploy/cloud-iam-federation-api --since=30m | rg "saml|oidc|InvalidSignature|JwksKeyNotFound|metadata_grace"`.
14. Read JIT logs: `kubectl -n cloud-iam logs deploy/cloud-iam-jit-worker --since=30m | rg "jit_rule|external_id|entity_materialized|role_attach"`.
15. Check federation rollout: `kubectl -n cloud-iam rollout status deploy/cloud-iam-federation-api --timeout=60s`.
16. Check metadata worker rollout: `kubectl -n cloud-iam rollout status deploy/cloud-iam-idp-metadata-worker --timeout=60s`.
17. Check JIT worker rollout: `kubectl -n cloud-iam rollout status deploy/cloud-iam-jit-worker --timeout=60s`.
18. Inspect tenant IdP config: `oya iam idp get --tenant $TENANT --idp $IDP --output yaml`.
19. Validate metadata signature: `oya iam idp metadata validate --tenant $TENANT --idp $IDP --cell $CELL`.
20. Fetch metadata manually: `oya iam idp metadata fetch --tenant $TENANT --idp $IDP --cell $CELL --output evidence/incidents/$INCIDENT_ID-metadata.xml`.
21. Check JWKS keys: `oya iam idp jwks inspect --tenant $TENANT --idp $IDP --cell $CELL`.
22. Run SAML canary: `oya ops probe cloud-iam saml-login --tenant $TENANT --idp $IDP --cell $CELL --output json`.
23. Run OIDC canary: `oya ops probe cloud-iam oidc-login --tenant $TENANT --idp $IDP --cell $CELL --output json`.
24. Inspect JIT rules: `oya iam idp jit-rules list --tenant $TENANT --idp $IDP --output yaml`.
25. Explain JIT match: `oya iam idp jit-rules explain --tenant $TENANT --idp $IDP --sample-assertion evidence/incidents/$INCIDENT_ID-assertion.json`.
26. Check user materialisation: `oya iam entity get --tenant $TENANT --uid 'User::"'$TENANT'/synthetic-canary"' --output json`.
27. Check role attachment: `oya iam role membership list --tenant $TENANT --principal 'User::"'$TENANT'/synthetic-canary"' --output json`.
28. Query audit login failures: `oya audit-chain query --event-class cloud_iam.federation.login.failed --tenant $TENANT --since 30m`.
29. Query session issues: `oya audit-chain query --event-class cloud_iam.session.issued --tenant $TENANT --since 30m`.
30. Check session signer: `oya secrets lease status --service cloud-iam --purpose session-signing --cell $CELL`.
31. Check callback ingress: `oya network route status --service cloud-iam --route /saml/v2/acs --cell $CELL`.
32. Check WAF blocks: `oya security waf events --service cloud-iam --path /oidc/v1/callback --since 30m`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-iam --runbook federated-identity-provider-stall --output evidence/incidents/$INCIDENT_ID.json`.
34. Preserve config: `oya iam idp export --tenant $TENANT --idp $IDP --output evidence/incidents/$INCIDENT_ID-idp.yaml`.
35. Preserve failed assertion hash only: `oya iam idp assertion hash --tenant $TENANT --idp $IDP --since 30m --output evidence/incidents/$INCIDENT_ID-assertion-hashes.json`.

### Diagnostic Decision Tree
```text
1. Are all tenants and IdPs failing in one cell?
   |-- yes: inspect cloud-iam callback ingress, session signer, and deployment.
   |-- no: continue tenant and IdP isolation.
2. Is only one external IdP provider failing?
   |-- yes: activate provider-specific graceful degradation and notify affected tenants.
   |-- no: inspect JIT rules and metadata freshness.
3. Is metadata expired or JWKS missing?
   |-- yes: refresh metadata, apply grace only if signature chain is valid, and page identity secondary.
   |-- no: inspect assertion shape and JIT mapping.
4. Does assertion validate but session issuance fail?
   |-- yes: inspect cloud-kms session signer and audit-chain emission.
   |-- no: fix IdP validation or JIT materialisation.
5. Are JIT entities created without role attachments?
   |-- yes: pause new logins for affected tenant and drain JIT role-attach queue.
   |-- no: close after login canaries and audit events are green.
```

## Mitigation
1. Freeze risky IdP config changes: incident hold PR against `dev` (normal VCS PR; branch-protected GitHub Actions `oya-ci-required` required; local/Jenkins rehearsals are non-authoritative).
2. Enable federation incident hold: `oya flags set oya.cloud_iam.federation.incident_hold=true --cell $CELL --tenant $TENANT`.
3. Keep signature validation strict: `oya flags set oya.cloud_iam.federation.allow_unsigned_assertions=false --cell $CELL`.
4. Allow metadata grace only when cached certificate is still valid: `oya iam idp metadata grace enable --tenant $TENANT --idp $IDP --ttl 2h --reason $INCIDENT_ID`.
5. Disable metadata grace when certificate expired: `oya iam idp metadata grace disable --tenant $TENANT --idp $IDP --reason $INCIDENT_ID`.
6. Refresh metadata: `oya iam idp metadata refresh --tenant $TENANT --idp $IDP --cell $CELL`.
7. Refresh JWKS: `oya iam idp jwks refresh --tenant $TENANT --idp $IDP --cell $CELL`.
8. Pause affected IdP only: `oya iam idp pause --tenant $TENANT --idp $IDP --reason $INCIDENT_ID --ttl 30m`.
9. Keep alternate IdPs online: `oya iam idp route prefer-healthy --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
10. Drain JIT queue dry-run: `oya iam jit drain --tenant $TENANT --idp $IDP --limit 100 --dry-run`.
11. Drain JIT queue confirmed: `oya iam jit drain --tenant $TENANT --idp $IDP --limit 100 --confirm $INCIDENT_ID`.
12. Rebuild entity cache: `oya iam entity-cache invalidate --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
13. Renew session signer lease: `oya secrets lease renew --service cloud-iam --purpose session-signing --cell $CELL`.
14. Roll back causal federation deploy: `kubectl -n cloud-iam rollout undo deploy/cloud-iam-federation-api`.
15. Roll back causal JIT deploy: `kubectl -n cloud-iam rollout undo deploy/cloud-iam-jit-worker`.
16. Throttle login retries: `oya ops rate-limit set --tenant $TENANT --surface cloud-iam.federation-login --rps 10 --ttl 30m`.
17. Notify tenant admins: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template idp-login-degraded`.
18. Notify support: `oya notify support --incident $INCIDENT_ID --template cloud-iam-idp-stall`.
19. Record mitigation audit: `oya audit-chain emit --event-class EVT_CLOUD_IAM_IDP_STALL_INCIDENT --incident $INCIDENT_ID --field mitigation=active`.
20. Preserve privacy boundary: never export raw SAML assertion or ID token into incident chat.

## Resolution
1. Patch metadata refresh when certificate rollover was missed.
2. Patch JWKS cache keying when key id collision or stale JWKS caused the failure.
3. Patch JIT rule matcher when claim shape drift caused role attachment failure.
4. Patch session issuance when cloud-kms signer lease or audit-chain commit blocked tokens.
5. Patch callback ingress when WAF or mTLS blocked ACS or OIDC callback traffic.
6. Add regression fixture with redacted assertion claims.
7. Add tenant dual-IdP regression for external ID collision.
8. Run federation tests: `cargo test -p oya-cloud-iam-api federation -- --nocapture`.
9. Run JIT tests: `cargo test -p oya-cloud-iam-domain jit -- --nocapture`.
10. Verify the branch-protected production-snapshot gate for `cloud-iam-federation` in `oya-ci-required` / cloud-ci for `$CELL`; do not use local dev-cli output as merge authority.
11. Re-enable paused IdP: `oya iam idp resume --tenant $TENANT --idp $IDP --reason resolved-$INCIDENT_ID`.
12. Disable incident hold: `oya flags set oya.cloud_iam.federation.incident_hold=false --cell $CELL --tenant $TENANT`.
13. Run login canary: `oya ops probe cloud-iam federation-login --tenant $TENANT --idp $IDP --cell $CELL --expect session-issued`.
14. Unhold promotion: recovery PR against `dev` (normal VCS PR; branch-protected GitHub Actions `oya-ci-required` required; local/Jenkins rehearsals are non-authoritative).
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_IAM_IDP_STALL_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudIamFederationLoginSuccessBurn` is green for 30 minutes.
- `CloudIamIdpMetadataExpired` is green or acknowledged with tenant admin owner.
- `oya_cloud_iam_federation_login_success_ratio` is above 0.995.
- `oya_cloud_iam_jit_provisioning_queue_depth` is zero for the affected tenant.
- `oya_cloud_iam_jit_provisioning_lag_seconds` is below 60.
- SAML canary returns `session_issued=true`.
- OIDC canary returns `session_issued=true`.
- Cedar entity store contains the canary user and expected role attachments.
- Audit-chain contains login success and session issuance events.
- Support confirms no new `cloud-iam.federation.login-stall` cases.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-iam-federated-identity-provider-stall
microservice: cloud-iam
event_class: EVT_CLOUD_IAM_IDP_STALL_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Federated Identity Provider Stall postmortem

## Summary
- Which IdP, tenant, and cell stalled.
- Whether SAML, OIDC, JIT, session issuance, or ingress caused impact.
- Whether alternate IdPs remained available.

## Timeline
- Alert fired:
- Tenant admin notified:
- Metadata or JIT recovered:
- Canaries passed:

## Privacy Handling
- Raw assertions exported: no.
- Token payloads exported: no.
- Redacted claim fixture path:

## Root Cause
- Metadata:
- JWKS:
- JIT:
- Session signer:
- Network or WAF:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Alert update:
```

## Escalation Path
- Page `oya-cloud-iam-primary` for all federation Sev1 incidents.
- Page `oya-identity-federation-primary` when metadata, JWKS, or JIT is implicated.
- Page `oya-cloud-kms-primary` when session signer lease fails.
- Page `oya-cloud-network-primary` when ACS or callback ingress fails.
- Page `oya-audit-chain-primary` when session issuance cannot seal.
- Notify `#inc-cloud-iam` with tenant and IdP scope.
- Notify `#support-idp-login-impact` before tenant admin messages.
- Notify `#privacy-review` if raw identity assertions were exposed.
- Engage tenant admin through Support-approved copy only.
- Escalate to executive incident commander when more than 50 percent of tenants in a cell cannot log in.

## Cross-µservice Coordination
- `cloud-kms`: verify session-token signer keys and HSM signer health.
- `cloud-network`: verify callback ingress, WAF, mTLS, and DNS reachability.
- `audit-chain`: verify federation login and session issuance events.
- `tenancy`: verify tenant status and tenant-level IdP configuration.
- `comms-email`: send tenant admin notices and all-clear messages.
- `support`: tag and deduplicate tenant login reports.
- `workflow-engine`: pause workflows requiring fresh JIT elevation for affected tenants.
- `cloud-billing`: annotate SLA-credit candidates for prolonged login outage.
- `security`: review any emergency login path request.
- `foundry`: pause IdP config mutation pipelines.
- `observability`: attach federation and JIT dashboard snapshots.
- `compliance`: decide whether regulated authentication outage reporting applies.

## Runbook Maintenance
- Add new IdP provider-specific signatures after every incident.
- Update metadata grace limits when policy changes.
- Keep SAML and OIDC canary commands aligned with production API paths.
- Keep privacy handling explicit; never add raw assertion examples.
- Review this runbook during every IAM on-call handover.
