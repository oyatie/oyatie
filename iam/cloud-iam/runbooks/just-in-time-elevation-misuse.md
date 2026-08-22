---
doc_class: Runbook
title: Just In Time Elevation Misuse
status: Accepted
date: 2026-05-20
microservice: cloud-iam
severity: sev0
audience: security-engineer, sre, iam-engineer
owner_team: axis-cloud + security-governance + ops-sre-reliability
doc_status: published
---

# Runbook: Just In Time Elevation Misuse

## Operator Contract
- Runbook id: cloud-iam-just-in-time-elevation-misuse.
- Primary namespace: `cloud-iam`.
- Owning rotation: PagerDuty `security-policy-primary`.
- IAM secondary: PagerDuty `cloud-iam-primary`.
- Incident channel: `#inc-security-iam`.
- Customer channel: `#support-cloud-iam-security`.
- Protected surface: JIT elevation grants, emergency break-glass, Cedar role attachments, provider-target temporary roles.
- Safety invariant: revoke suspicious elevation before preserving convenience.
- Forensic invariant: preserve audit-chain evidence before deleting grants.
- Privacy invariant: do not post raw session recordings in incident chat.
- Stop condition: suspicious elevation is revoked, affected principals are contained, and audit-chain evidence is sealed.
- Evidence event: `EVT_CLOUD_IAM_JIT_ELEVATION_MISUSE_INCIDENT`.
- Handoff API: `https://cloud-iam.internal.oyatie.dev/v1/elevation/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/elevation?orgId=1&var-cell=prod-us-east-1`.
- Session dashboard: `https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/session-recording?orgId=1&var-surface=jit`.
- Loki query: `{namespace="cloud-iam",runbook="just-in-time-elevation-misuse"}`.
- Canonical FAQ: `microservices/cloud-iam/faqs/iam-engineer-faq.md`.
- Related action: `cloud_iam::Action::EmergencyBreakGlass`.
- Related action: `cloud_iam::Action::IssueCrossTenantToken`.
- Related action: `cloud_iam::Action::TranslateToAwsIam`.

## Trigger Conditions
- Alert `CloudIamJitElevationMisuseCritical` fires.
- Alert `CloudIamBreakGlassWithoutReviewer` fires.
- Alert `CloudIamElevationAnomalousDuration` fires.
- Alert `CloudIamPrivilegeGrantOutsideChangeWindow` fires.
- Metric `cloud_iam_jit_elevation_active_sessions` exceeds tenant baseline by 3 sigma.
- Metric `cloud_iam_break_glass_grant_total` is non-zero for paid tenant_class without PIV signature.
- Metric `cloud_iam_jit_elevation_denied_then_allowed_total` spikes for one principal.
- Metric `cloud_iam_elevation_reviewer_missing_total` is non-zero.
- Metric `cloud_iam_cross_tenant_elevation_total` is non-zero without dual-tenant permits.
- Metric `cloud_iam_elevation_session_recording_gap_total` is non-zero.
- Audit-chain sees `cloud_iam.elevation.granted` without matching approval event.
- Audit-chain sees `cloud_iam.elevation.used` after grant expiration.
- Support reports tenant admin seeing unexplained elevated access.
- Security reviewer reports suspicious role grant during incident response.
- Foundry pipeline principal requests emergency action outside declared pipeline.
- Provider target has a temporary admin role without Cedar grant.
- User principal escalates from low-risk role to high-risk role in one step.
- Workload principal receives human-only break-glass role.
- JIT elevation request lacks ticket, incident, or change reference.
- Session recording fails for a paid tenant_class break-glass action.

## Symptoms
- `decision=permit reason=break_glass` appears without reviewer id.
- `jit_elevation_ttl_seconds` exceeds tenant_class policy maximum.
- `principal_kind=Workload` appears with human emergency role.
- `cross_tenant_bridge=true` appears without both tenant permits.
- `session_recording_status=missing` appears on paid tenant_class sessions.
- Provider target role remains after Cedar grant expiration.
- Audit-chain has approval event but approver principal is not in reviewer set.
- `change_window_id` is empty during production privilege grant.
- `mfa_strength` is lower than tenant policy for the role.
- `piv_signature_status=missing` appears for paid tenant_class break-glass.
- Login IP or workload attestation source differs from principal baseline.
- Elevation grant is used to mutate cloud-kms keys, cloud-network policies, or billing ledgers.
- Tenant admin can see an emergency role but not the reason code.
- Multiple denied attempts are followed by one permit after policy edit.
- Role attachment was created by Foundry without approved pipeline action.
- Role grant was translated to AWS/GCP/Azure target but Cedar revoked it.
- Session token lifetime is longer than the JIT grant lifetime.
- `cloud_iam.elevation.revoked` is missing after `cloud_iam.elevation.expired`.
- IAM dashboard shows high severity grant with no active incident bridge.
- Security case is already active when this runbook opens.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-iam-jit-misuse-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export PRINCIPAL=unknown`.
3. Acknowledge security page: `pd incident ack --service security-policy --incident $INCIDENT_ID`.
4. Create security bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-security-iam --severity sev0`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.surface=="jit-elevation")'`.
6. Query active sessions: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_iam_jit_elevation_active_sessions{cell="'$CELL'"}'`.
7. Query break-glass grants: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(cloud_iam_break_glass_grant_total[5m])'`.
8. Query reviewer gaps: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_iam_elevation_reviewer_missing_total{cell="'$CELL'"}'`.
9. Query recording gaps: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_iam_elevation_session_recording_gap_total{cell="'$CELL'"}'`.
10. Open dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/elevation?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
11. Open session panel: `open "https://grafana.dev.oyatie.internal/d/cloud-iam-substrate/session-recording?orgId=1&var-principal=$PRINCIPAL"`.
12. Read elevation logs: `kubectl -n cloud-iam logs deploy/cloud-iam-elevation-api --since=60m | rg "elevation|break_glass|reviewer|session_recording"`.
13. List active grants: `oya iam elevation list --tenant $TENANT --cell $CELL --active --output table`.
14. Inspect suspicious grant: `oya iam elevation get --tenant $TENANT --principal $PRINCIPAL --output json`.
15. Explain Cedar decision: `oya iam authz explain --tenant $TENANT --principal $PRINCIPAL --action EmergencyBreakGlass --resource "*"`.
16. Check reviewer approval: `oya iam elevation approval get --tenant $TENANT --principal $PRINCIPAL --output json`.
17. Verify MFA strength: `oya iam principal authn-context --tenant $TENANT --principal $PRINCIPAL --output json`.
18. Verify PIV signature: `oya iam elevation piv-verify --tenant $TENANT --principal $PRINCIPAL --incident $INCIDENT_ID`.
19. Check session recording: `oya iam session recording status --tenant $TENANT --principal $PRINCIPAL --since 60m`.
20. Check audit grant: `oya audit-chain query --event-class cloud_iam.elevation.granted --tenant $TENANT --principal $PRINCIPAL --since 24h`.
21. Check audit use: `oya audit-chain query --event-class cloud_iam.elevation.used --tenant $TENANT --principal $PRINCIPAL --since 24h`.
22. Check audit revoke: `oya audit-chain query --event-class cloud_iam.elevation.revoked --tenant $TENANT --principal $PRINCIPAL --since 24h`.
23. Check provider target residue: `oya iam target temporary-roles list --tenant $TENANT --principal $PRINCIPAL --provider all`.
24. Check cross-service mutations: `oya audit-chain query --principal $PRINCIPAL --since 24h --actions kms,network,billing,tenant`.
25. Check Foundry provenance: `oya foundry principal provenance --principal $PRINCIPAL --since 24h --output json`.
26. Check change window: `oya change window status --tenant $TENANT --principal $PRINCIPAL --output json`.
27. Check ticket reference: `oya incident reference verify --principal $PRINCIPAL --tenant $TENANT --since 24h`.
28. Check source IP: `oya iam principal access-pattern --tenant $TENANT --principal $PRINCIPAL --since 7d`.
29. Check workload attestation: `oya iam workload attest --tenant $TENANT --principal $PRINCIPAL --cell $CELL`.
30. Lock evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-iam --runbook just-in-time-elevation-misuse --output evidence/incidents/$INCIDENT_ID.json`.
31. Freeze session recording: `oya iam session recording freeze --tenant $TENANT --principal $PRINCIPAL --incident $INCIDENT_ID`.
32. Export grant records: `oya iam elevation export --tenant $TENANT --principal $PRINCIPAL --output evidence/incidents/$INCIDENT_ID-grants.json`.
33. Export provider residue: `oya iam target temporary-roles export --tenant $TENANT --principal $PRINCIPAL --provider all --output evidence/incidents/$INCIDENT_ID-target-roles.json`.
34. Notify commander: `oya notify security-commander --incident $INCIDENT_ID --summary "JIT elevation misuse triage started"`.
35. Start containment timer: `oya incident timer set --incident $INCIDENT_ID --name containment --target 15m`.

### Diagnostic Decision Tree
```text
1. Is active elevated access still present?
   |-- yes: revoke first, preserve evidence second, then continue.
   |-- no: preserve forensic state and continue.
2. Was the elevation approved by the correct reviewer set?
   |-- no: treat as misuse and keep Sev0.
   |-- yes: inspect scope, TTL, MFA, and recording.
3. Did the principal mutate KMS, network, billing, tenancy, or provider IAM?
   |-- yes: coordinate the affected microservice runbooks.
   |-- no: keep containment inside cloud-iam.
4. Does provider target still carry temporary role residue?
   |-- yes: revoke target residue and run translation digest check.
   |-- no: continue audit-chain verification.
5. Is session recording missing for a tenant_class policy that requires it?
   |-- yes: keep incident open and page compliance.
   |-- no: close after revoke, evidence freeze, and corrective action.
```

## Mitigation
1. Revoke suspicious grant dry-run: `oya iam elevation revoke --tenant $TENANT --principal $PRINCIPAL --reason $INCIDENT_ID --dry-run`.
2. Revoke suspicious grant confirmed: `oya iam elevation revoke --tenant $TENANT --principal $PRINCIPAL --reason $INCIDENT_ID --confirm`.
3. Suspend principal if active misuse continues: `oya identity principal suspend --tenant $TENANT --principal $PRINCIPAL --reason $INCIDENT_ID`.
4. Revoke provider target roles dry-run: `oya iam target temporary-roles revoke --tenant $TENANT --principal $PRINCIPAL --provider all --dry-run`.
5. Revoke provider target roles confirmed: `oya iam target temporary-roles revoke --tenant $TENANT --principal $PRINCIPAL --provider all --confirm $INCIDENT_ID`.
6. Freeze further elevation for tenant: `oya flags set oya.cloud_iam.elevation.freeze=true --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
7. Require two reviewers: `oya iam elevation policy set --tenant $TENANT --min-reviewers 2 --ttl 4h --reason $INCIDENT_ID`.
8. Shorten max TTL: `oya iam elevation policy set --tenant $TENANT --max-ttl 15m --reason $INCIDENT_ID`.
9. Enforce PIV for paid tenant_class: `oya iam elevation policy set --tenant $TENANT --require-piv true --reason $INCIDENT_ID`.
10. Enable session recording hold: `oya iam session recording hold --tenant $TENANT --principal $PRINCIPAL --incident $INCIDENT_ID`.
11. Quarantine workload principal: `oya iam workload quarantine --tenant $TENANT --principal $PRINCIPAL --ttl 60m --reason $INCIDENT_ID`.
12. Pause Foundry principal if implicated: `oya foundry principal pause --principal $PRINCIPAL --reason $INCIDENT_ID`.
13. Hold cloud-iam promotions: incident hold PR against `dev` (normal VCS PR; branch-protected GitHub Actions `presubmit` required; local/Jenkins rehearsals are non-authoritative).
14. Notify affected service owners: `oya notify service-owner --incident $INCIDENT_ID --microservice cloud-iam`.
15. Notify tenant admin through security copy: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template jit-elevation-contained`.
16. Preserve evidence: `oya evidence freeze --incident $INCIDENT_ID --paths evidence/incidents/$INCIDENT_ID.json`.
17. Emit mitigation event: `oya audit-chain emit --event-class EVT_CLOUD_IAM_JIT_ELEVATION_MISUSE_INCIDENT --incident $INCIDENT_ID --field mitigation=revoked`.
18. Create security case: `oya security case create --incident $INCIDENT_ID --category iam-jit-misuse --tenant $TENANT`.
19. Rotate session signing keys if compromise is plausible: `oya kms sign-key rotate --purpose cloud-iam-session --tenant $TENANT --reason $INCIDENT_ID`.
20. Keep all emergency overrides disabled until commander approves all-clear.

## Resolution
1. Patch Cedar policy if reviewer or TTL constraints were incomplete.
2. Patch elevation API if approval or ticket reference was not mandatory.
3. Patch provider translation if target temporary roles survived revoke.
4. Patch session recording if recording was missing or inaccessible.
5. Patch audit-chain emission if grant, use, and revoke events were incomplete.
6. Add regression for workload principal receiving human-only role.
7. Add regression for expired grant still permitting provider target access.
8. Add regression for missing reviewer id.
9. Add regression for paid tenant_class PIV signature requirement.
10. Run domain tests: `cargo test -p cloud-iam-domain elevation -- --nocapture`.
11. Run API tests: `cargo test -p cloud-iam-api elevation -- --nocapture`.
12. Verify the branch-protected production-snapshot gate for `cloud-iam-elevation-policy` in `presubmit` / cloud-ci for `$CELL`; do not use local dev-cli output as merge authority.
13. Verify no active grants: `oya iam elevation list --tenant $TENANT --active --expect none`.
14. Verify no provider residue: `oya iam target temporary-roles list --tenant $TENANT --principal $PRINCIPAL --provider all --expect none`.
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_IAM_JIT_ELEVATION_MISUSE_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudIamJitElevationMisuseCritical` is green.
- `CloudIamBreakGlassWithoutReviewer` is green.
- `cloud_iam_jit_elevation_active_sessions` returns to baseline.
- No active suspicious grant remains for the tenant.
- No provider target temporary role remains for the principal.
- Audit-chain contains grant, use, revoke, mitigation, and resolution events.
- Session recording is frozen when required by tenant_class policy.
- Security case has owner and severity.
- Tenant admin notification was sent when tenant impact existed.
- Regression tests and policy gate are recorded in evidence.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-iam-just-in-time-elevation-misuse
microservice: cloud-iam
event_class: EVT_CLOUD_IAM_JIT_ELEVATION_MISUSE_INCIDENT
incident_id: <INC-...>
severity: sev0
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Just In Time Elevation Misuse postmortem

## Summary
- Which principal misused or appeared to misuse elevation.
- Which roles, providers, and resources were affected.
- Whether access was over-permitted, overlong, unreviewed, or unrecorded.

## Timeline
- Grant created:
- Misuse detected:
- Grant revoked:
- Provider residue cleared:
- Evidence sealed:

## Forensics
- Session recording path:
- Audit-chain range:
- Provider target export:

## Root Cause
- Policy gap:
- API guard gap:
- Translation revoke gap:
- Human process gap:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Policy update:
```

## Escalation Path
- Page `security-policy-primary` immediately for Sev0 JIT misuse.
- Page `cloud-iam-primary` for revocation, Cedar, and provider target residue.
- Page `cloud-kms-primary` if session signing or key rotation is required.
- Page `audit-chain-primary` when forensic events are incomplete.
- Page affected microservice owners when elevated action touched their resources.
- Notify `#inc-security-iam` with principal, tenant, and containment state.
- Notify `#support-cloud-iam-security` before tenant-facing copy.
- Notify `#compliance-review` when recording or approval evidence is missing.
- Engage legal if over-permit touched regulated data or cross-tenant resources.
- Escalate to executive incident commander when more than one tenant is affected.

## Cross-µservice Coordination
- `audit-chain`: seal grant, use, revoke, mitigation, and resolution event ranges.
- `cloud-kms`: rotate session signer or affected tenant keys when compromise is plausible.
- `cloud-network`: block egress for a workload principal under quarantine.
- `cloud-billing`: identify mutations to credits, rate cards, or invoices by the principal.
- `tenancy`: quarantine tenant scope if cross-tenant bridge grants were involved.
- `workflow-engine`: pause workflows that can request JIT elevation.
- `foundry`: pause implicated pipeline principals and collect provenance.
- `security`: own forensic case and commander decisions.
- `compliance`: decide notification obligations for missing recording or regulated access.
- `support`: manage tenant admin communications.
- `observability`: attach elevation and session dashboards to evidence.
- `cloud-iam`: own Cedar policy, role attachments, and provider residue cleanup.

## Runbook Maintenance
- Add every newly discovered misuse signature to Trigger Conditions.
- Keep actions mapped to Cedar names.
- Keep revocation commands dry-run first.
- Review tenant_class-specific recording and PIV requirements quarterly.
- Update cross-service coordination when a new privileged target surface is added.
