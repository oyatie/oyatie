---
doc_class: Runbook
title: Cross Cloud Permission Translation Failure
status: Accepted
date: 2026-05-20
microservice: cloud-iam
severity: sev1
audience: sre, security-engineer, iam-engineer
owner_team: axis-cloud + ops-sre-reliability + security-governance
doc_status: published
---

# Runbook: Cross Cloud Permission Translation Failure

## Operator Contract
- Runbook id: cloud-iam-cross-cloud-permission-translation-failure.
- Primary namespace: `cloud-iam`.
- Owning rotation: PagerDuty `oya-cloud-iam-primary`.
- Security secondary: PagerDuty `oya-security-policy-primary`.
- Incident channel: `#inc-cloud-iam`.
- Customer channel: `#support-cloud-iam-tenant-impact`.
- Source of truth: Cedar policy remains authoritative.
- Downstream targets: AWS IAM JSON, GCP IAM bindings, Azure RBAC assignments, Okta app grants.
- Translation digest: every target policy must carry the Cedar `blake3-256` digest pointer.
- Safety invariant: never hand-edit provider IAM when this runbook is active.
- Stop condition: translation backlog is empty, target digests match Cedar, and audit-chain has sealed mitigation plus resolution events.
- Evidence event: `EVT_CLOUD_IAM_TRANSLATION_FAILURE_INCIDENT`.
- Handoff API: `https://cloud-iam.internal.oyatie.dev/v1/translation/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/translation?orgId=1&var-cell=prod-us-east-1`.
- Audit dashboard: `https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/audit-chain?orgId=1&var-surface=translation`.
- Loki query: `{namespace="cloud-iam",runbook="cross-cloud-permission-translation-failure"}`.
- Canonical docs: `iam/cloud-iam/faqs/iam-engineer-faq.md`.
- Related policy path: `crates/oya-cloud-iam-domain`.
- Related API tests: `crates/oya-cloud-iam-api/tests/cloud_iam_api.rs`.

## Trigger Conditions
- Alert `CloudIamTranslationFailureCritical` fires for any production cell.
- Alert `CloudIamTargetDigestMismatch` fires for a provider target.
- Alert `CloudIamTranslationBacklogSloBurn` fires for 10 minutes.
- Metric `oya_cloud_iam_translation_failure_total` increases by more than 10 in 5 minutes.
- Metric `oya_cloud_iam_translation_backlog_depth` is above 1000 for 15 minutes.
- Metric `oya_cloud_iam_target_digest_mismatch_total` is non-zero.
- Metric `oya_cloud_iam_unrepresentable_cedar_policy_total` spikes for a single tenant.
- Metric `oya_cloud_iam_provider_write_error_ratio{target="aws"}` exceeds 0.02.
- Metric `oya_cloud_iam_provider_write_error_ratio{target="gcp"}` exceeds 0.02.
- Metric `oya_cloud_iam_provider_write_error_ratio{target="azure"}` exceeds 0.02.
- Metric `oya_cloud_iam_provider_write_error_ratio{target="okta"}` exceeds 0.02.
- Branch-protected `oya-ci-required` / cloud-ci production-snapshot gate for `cloud-iam-translation` fails.
- Support case is tagged `cloud-iam.translation.customer-visible`.
- A tenant reports denied access after a Cedar policy publish that should have permitted access.
- A tenant reports permitted access after a Cedar policy rollback that should have denied access.
- Audit-chain detects a missing `cloud_iam.translation.applied` event.
- Provider inventory shows a target policy without the current Cedar digest.
- Rollout history shows a new compiler build inside the last 60 minutes.
- Foundry admission reports policy promotion but target provider still has old assignment.
- Any manual provider-console change is detected during this incident.

## Symptoms
- AWS role trust policies are missing the tenant-scoped external ID.
- AWS IAM JSON contains resources that do not match the Cedar resource hierarchy.
- GCP IAM bindings are present but attached to the wrong project or folder.
- Azure RBAC assignments are delayed or attached to the wrong scope.
- Okta app grants appear correct but Cedar entity membership is stale.
- `TranslationError::UnrepresentableInTarget` appears in worker logs.
- `target_digest_mismatch=true` appears in structured logs.
- `provider_write_status=throttled` repeats for one provider.
- `provider_write_status=forbidden` repeats after a cloud credential rotation.
- `tenant_id` skew shows one tenant owning more than 80 percent of backlog.
- `target_provider` skew shows one downstream cloud owns the backlog.
- `policy_version` in target provider is older than Cedar `policy_version`.
- Users with valid `User` principals receive provider-side access denied.
- Workload principals with fresh SPIFFE SVIDs cannot assume downstream roles.
- Cross-tenant bridge calls fail on provider target while Cedar evaluates allow.
- Break-glass policies compile locally but refuse provider write.
- Audit-chain has policy publish events but no target apply events.
- Translation workers restart with memory pressure after loading large entity stores.
- Provider API rate limits grow while translation worker CPU is below 50 percent.
- The incident is security-sensitive when target provider permits more than Cedar.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-iam-translation-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
3. Acknowledge the page: `pd incident ack --service cloud-iam --incident $INCIDENT_ID`.
4. Open the incident bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-cloud-iam --severity sev1`.
5. Query Alertmanager: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.service=="cloud-iam")'`.
6. Query failure metric: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(oya_cloud_iam_translation_failure_total[5m])'`.
7. Query backlog: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_iam_translation_backlog_depth{cell="'$CELL'"}'`.
8. Query digest mismatch: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_iam_target_digest_mismatch_total{cell="'$CELL'"}'`.
9. Query provider write error: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_iam_provider_write_error_ratio{cell="'$CELL'"}'`.
10. Open translation dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/translation?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
11. Open provider panel: `open "https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/provider-writes?orgId=1&var-cell=$CELL"`.
12. Read logs: `kubectl -n cloud-iam logs deploy/cloud-iam-translation-worker --since=30m | rg "TranslationError|target_digest_mismatch|cloud_iam.translation"`.
13. Check rollout: `kubectl -n cloud-iam rollout status deploy/cloud-iam-translation-worker --timeout=60s`.
14. List pods: `kubectl -n cloud-iam get pods -l app=translation-worker -o wide`.
15. Check recent deploys: `kubectl -n cloud-iam rollout history deploy/cloud-iam-translation-worker | tail -20`.
16. Inspect current Cedar hash: `oya iam policy digest --tenant $TENANT --cell $CELL --output json`.
17. Inspect target AWS hash: `oya iam target digest --tenant $TENANT --provider aws --cell $CELL --output json`.
18. Inspect target GCP hash: `oya iam target digest --tenant $TENANT --provider gcp --cell $CELL --output json`.
19. Inspect target Azure hash: `oya iam target digest --tenant $TENANT --provider azure --cell $CELL --output json`.
20. Inspect target Okta hash: `oya iam target digest --tenant $TENANT --provider okta --cell $CELL --output json`.
21. Run translation dry-run: `oya iam translate --tenant $TENANT --policy current --provider all --dry-run --explain`.
22. Run representability check: `oya iam translate lint --tenant $TENANT --provider aws,gcp,azure,okta --output table`.
23. Check entity freshness: `oya iam entity-cache status --tenant $TENANT --cell $CELL --output json`.
24. Check event invalidation: `oya audit-chain query --event-class cloud_iam.entity.updated --tenant $TENANT --since 30m`.
25. Check target apply events: `oya audit-chain query --event-class cloud_iam.translation.applied --tenant $TENANT --since 30m`.
26. Verify admission state: `oya vcs status --microservice cloud-iam --tenant $TENANT --output json`.
27. Check Foundry principal scope: `oya iam authz explain --principal oyatie.foundry.translation --action TranslateToAwsIam --tenant $TENANT`.
28. Inspect provider credentials: `oya secrets lease status --service cloud-iam --purpose provider-write --cell $CELL`.
29. Check AWS write path: `oya iam provider health --provider aws --tenant $TENANT --cell $CELL`.
30. Check GCP write path: `oya iam provider health --provider gcp --tenant $TENANT --cell $CELL`.
31. Check Azure write path: `oya iam provider health --provider azure --tenant $TENANT --cell $CELL`.
32. Check Okta write path: `oya iam provider health --provider okta --tenant $TENANT --cell $CELL`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-iam --runbook cross-cloud-permission-translation-failure --output evidence/incidents/$INCIDENT_ID.json`.
34. Preserve failing policy: `oya iam policy export --tenant $TENANT --version current --output evidence/incidents/$INCIDENT_ID.cedar`.
35. Preserve target state: `oya iam target export --tenant $TENANT --provider all --output evidence/incidents/$INCIDENT_ID-targets.json`.

### Diagnostic Decision Tree
```text
1. Does any provider target permit more than Cedar?
   |-- yes: treat as security incident; freeze translation and revoke widened target grants.
   |-- no: continue availability triage.
2. Is the Cedar policy unrepresentable in one target provider?
   |-- yes: keep Cedar authoritative; stop provider write for that target and page policy engineering.
   |-- no: continue provider write triage.
3. Is backlog isolated to one provider?
   |-- yes: open provider-specific mitigation branch.
   |-- no: inspect translation-worker deployment and entity-cache invalidation.
4. Did a deploy happen inside the first failing window?
   |-- yes: prepare rollback after snapshot.
   |-- no: inspect credentials, provider quotas, and target-side API health.
5. Are audit-chain target apply events missing?
   |-- yes: keep incident open until replay seals.
   |-- no: close only after target digest parity holds for 30 minutes.
```

## Mitigation
1. Freeze policy promotion: incident hold PR against `dev` (normal VCS PR; branch-protected GitHub Actions `oya-ci-required` required; local/Jenkins rehearsals are non-authoritative).
2. Disable automated provider writes: `oya flags set oya.cloud_iam.translation.auto_apply=false --cell $CELL --reason $INCIDENT_ID`.
3. Keep Cedar evaluation online: `oya flags set oya.cloud_iam.cedar_authority_only=true --cell $CELL --reason $INCIDENT_ID`.
4. Open translation breaker: `oya ops breaker open cloud-iam-translation --cell $CELL --ttl 30m --reason $INCIDENT_ID`.
5. Stop widening writes: `oya iam translation guard enable --mode deny-widening --cell $CELL --reason $INCIDENT_ID`.
6. Revoke any widened target grant: `oya iam target revoke-widened --tenant $TENANT --provider all --dry-run`.
7. Execute revocation only after commander approval: `oya iam target revoke-widened --tenant $TENANT --provider all --confirm $INCIDENT_ID`.
8. Drain safe backlog: `oya iam translation drain --tenant $TENANT --mode digest-match-only --limit 200 --dry-run`.
9. Apply safe backlog: `oya iam translation drain --tenant $TENANT --mode digest-match-only --limit 200 --confirm $INCIDENT_ID`.
10. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface cloud-iam.translation --rps 5 --ttl 30m`.
11. Restart one worker when stuck: `kubectl -n cloud-iam rollout restart deploy/cloud-iam-translation-worker`.
12. Scale workers only after provider quotas are healthy: `kubectl -n cloud-iam scale deploy/cloud-iam-translation-worker --replicas=6`.
13. Roll back causal deploy: `kubectl -n cloud-iam rollout undo deploy/cloud-iam-translation-worker`.
14. Refresh provider write credentials: `oya secrets lease renew --service cloud-iam --purpose provider-write --cell $CELL`.
15. Refresh entity cache: `oya iam entity-cache invalidate --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
16. Replay apply events: `oya audit-chain replay --event-class cloud_iam.translation.applied --incident $INCIDENT_ID`.
17. Notify support: `oya notify support --incident $INCIDENT_ID --template cloud-iam-translation-degraded`.
18. Notify tenant admins when access may be denied: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --impact access-denial-risk`.
19. Notify security when over-permit is possible: `oya notify security --incident $INCIDENT_ID --impact provider-over-permit`.
20. Keep provider console writes blocked: `oya iam provider-console-lock enable --tenant $TENANT --reason $INCIDENT_ID`.

## Resolution
1. Patch translator only after the failing policy and target exports are preserved.
2. If representability failed, add an explicit `UnrepresentableInTarget` error branch.
3. If digest propagation failed, patch target writer to persist the Cedar digest before completion.
4. If entity cache was stale, patch invalidation consumer for `cloud_iam.entity.*`.
5. If provider credentials failed, patch lease renewal and add credential-expiry alerting.
6. If provider quota failed, patch retry jitter and per-provider queue partitioning.
7. Add regression fixture: `fixtures/cloud-iam/translation/$INCIDENT_ID.json`.
8. Run translator test: `cargo test -p oya-cloud-iam-domain translation -- --nocapture`.
9. Run API test: `cargo test -p oya-cloud-iam-api cloud_iam_policy_translation -- --nocapture`.
10. Verify the branch-protected production-snapshot gate for `cloud-iam-translation` in `oya-ci-required` / cloud-ci for `$CELL`; do not use local dev-cli output as merge authority.
11. Re-enable auto apply for one tenant: `oya flags set oya.cloud_iam.translation.auto_apply=true --tenant $TENANT --cell $CELL`.
12. Re-run digest comparison: `oya iam target digest --tenant $TENANT --provider all --cell $CELL --expect cedar-current`.
13. Close breaker: `oya ops breaker close cloud-iam-translation --cell $CELL --reason resolved-$INCIDENT_ID`.
14. Unhold promotion: recovery PR against `dev` (normal VCS PR; branch-protected GitHub Actions `oya-ci-required` required; local/Jenkins rehearsals are non-authoritative).
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_IAM_TRANSLATION_FAILURE_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudIamTranslationFailureCritical` is green for 30 minutes.
- `CloudIamTargetDigestMismatch` is green for 30 minutes.
- `oya_cloud_iam_translation_backlog_depth` is zero for the affected tenant.
- `oya_cloud_iam_target_digest_mismatch_total` does not increase.
- Provider targets report the current Cedar digest.
- Cedar `authorize` and target provider access produce the same decision on the canary matrix.
- `cloud_iam.translation.applied` events are sealed in audit-chain.
- No provider console changes are detected during the incident window.
- Support confirms no new `cloud-iam.translation.customer-visible` cases.
- The incident evidence file contains Cedar export, target export, metrics, and commands.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-iam-cross-cloud-permission-translation-failure
microservice: cloud-iam
event_class: EVT_CLOUD_IAM_TRANSLATION_FAILURE_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Cross Cloud Permission Translation Failure postmortem

## Summary
- What failed in Cedar-to-provider translation.
- Which tenants, providers, and cells were affected.
- Whether provider targets over-permitted, under-permitted, or only lagged.

## Timeline
- Detection:
- First mitigation:
- Digest parity restored:
- Audit-chain sealed:

## Customer Impact
- Access denied count:
- Access over-permit count:
- Provider targets affected:

## Root Cause
- Translator defect:
- Provider write defect:
- Entity-cache defect:
- Credential or quota defect:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Dashboard or alert update:
```

## Escalation Path
- Page `oya-cloud-iam-primary` immediately for all Sev1 incidents.
- Page `oya-security-policy-primary` if provider target may permit more than Cedar.
- Page `oya-cloud-provider-ops` when provider API write errors exceed 0.02 for 10 minutes.
- Page `oya-audit-chain-primary` if apply events are missing or replay fails.
- Notify `#inc-cloud-iam` for all operators.
- Notify `#security-policy-review` for over-permit risk.
- Notify `#support-cloud-iam-tenant-impact` for customer-visible deny risk.
- Engage tenant admin only through Support-approved templates.
- Escalate to legal when over-permit affects regulated resources.
- Escalate to executive incident commander if more than 20 tenants are affected.

## Cross-µservice Coordination
- `audit-chain`: verify `cloud_iam.translation.applied` and incident events are sealed.
- `tenancy`: quarantine a tenant if translation state could cross tenant boundaries.
- `cloud-kms`: confirm provider credential signing keys are valid when write auth fails.
- `cloud-network`: maintain ingress to IAM APIs and block provider-console emergency paths if needed.
- `cloud-billing`: annotate any SLA-credit candidate caused by access denial.
- `foundry`: pause policy-promotion pipelines for cloud-iam until unhold.
- `workflow-engine`: pause workflows that depend on just-in-time privilege until parity is restored.
- `comms-email`: send tenant admin notifications with approved incident copy.
- `observability`: pin dashboard snapshots to the incident evidence file.
- `security`: review all over-permit possibilities before incident close.
- `compliance`: decide whether regulated access-control notification is required.
- `support`: tag cases with `cloud-iam.translation.customer-visible`.

## Runbook Maintenance
- Review this runbook after every translation incident.
- Add any missing provider-specific command discovered during response.
- Add any new provider target to the trigger metric list.
- Update dashboard URLs when Grafana dashboard ids change.
- Keep commands dry-run first unless this runbook explicitly says confirm.
