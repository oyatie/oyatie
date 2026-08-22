---
doc_class: Runbook
title: Key Material Quorum Loss
status: Accepted
date: 2026-05-20
microservice: cloud-kms
severity: sev0
audience: security-engineer, crypto-operator, sre
owner_team: axis-cloud + crypto-operations + compliance-security
doc_status: published
---

# Runbook: Key Material Quorum Loss

## Operator Contract
- Runbook id: cloud-kms-key-material-quorum-loss.
- Primary namespace: `cloud-kms`.
- Owning rotation: PagerDuty `crypto-operations-primary`.
- KMS secondary: PagerDuty `cloud-kms-primary`.
- Incident channel: `#inc-crypto-quorum`.
- Customer channel: `#support-cloud-kms-security`.
- Protected surface: M-of-N operator custody, HSM partition quorum, BYOK imports, cryptoshred approvals, paid key custody.
- Safety invariant: no destructive key action runs while quorum is below policy.
- Custody invariant: operator cards must be physically reconciled before quorum is restored.
- Evidence invariant: quorum state changes must be audit-chain anchored.
- Stop condition: quorum is restored or tenant is safely moved to a compliant standby custody group.
- Evidence event: `EVT_CLOUD_KMS_KEY_QUORUM_LOSS_INCIDENT`.
- Handoff API: `https://cloud-kms.internal.oyatie.dev/v1/quorum/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/quorum?orgId=1&var-cell=prod-us-east-1`.
- Custody dashboard: `https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/operator-custody?orgId=1&var-tenant_class=paid`.
- Loki query: `{namespace="cloud-kms",runbook="key-material-quorum-loss"}`.
- Canonical FAQ: `microservices/cloud-kms/faqs/kms-engineer-faq.md`.
- Related action: `cloud_kms::Action::Cryptoshred`.
- Related action: `cloud_kms::Action::RotateCmk`.
- Related action: `cloud_kms::Action::ImportByokMaterial`.
- Related action: `cloud_kms::Action::IssueClientCert`.

## Trigger Conditions
- Alert `CloudKmsOperatorQuorumLost` fires.
- Alert `CloudKmsCustodyCardMissing` fires.
- Alert `CloudKmsDestructiveActionBlockedByQuorum` fires.
- Alert `CloudKmsByokImportQuorumFailure` fires.
- Alert `CloudKmsCryptoshredQuorumFailure` fires.
- Metric `cloud_kms_operator_quorum_available` drops below required M.
- Metric `cloud_kms_operator_card_inventory_missing_total` is non-zero.
- Metric `cloud_kms_quorum_approval_timeout_total` increases.
- Metric `cloud_kms_destructive_action_blocked_total` increases.
- Metric `cloud_kms_byok_import_blocked_total` increases.
- Metric `cloud_kms_cryptoshred_blocked_total` increases.
- Metric `cloud_kms_rotation_blocked_by_quorum_total` increases.
- Custody reconciliation job misses monthly SLA.
- Tenant reports key rotation or cryptoshred request stuck.
- Compliance reviewer reports missing card attestation.
- HSM partition reports quorum script failure.
- Audit-chain lacks `cloud_kms.quorum.approved` for a destructive request.
- Two-person approval service is degraded.
- PIV/CAC validation fails for an operator card.
- Physical custody registry and HSM script disagree.

## Symptoms
- Cryptoshred commands refuse with `KmsError::QuorumUnavailable`.
- BYOK import commands refuse before material unwrap.
- Rotation jobs stay in `waiting_for_quorum`.
- Tenant CA issuance for paid tenants waits on quorum.
- HSM partition accepts read operations but refuses custody scripts.
- `operator_card_status=missing` appears in custody logs.
- `quorum_script_status=failed` appears in HSM logs.
- `approval_timeout=true` appears in request records.
- Audit-chain shows request submitted but no quorum approval.
- Operator card inventory shows stale monthly reconciliation.
- One custodian has expired PIV certificate.
- One custodian is removed from tenant roster but still in HSM script.
- `M=3 N=5` policy is configured but only two valid operators are available.
- A tenant tries to lower quorum during incident.
- A support request asks for manual override of cryptoshred.
- HSM inventory indicates partition moved without custody update.
- The action queue grows only for destructive or import actions.
- Decrypt and sign continue to work while quorum-gated actions fail.
- The incident is Sev0 when key material custody is uncertain.
- The incident remains Sev0 until compliance owner signs downscope.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-kms-quorum-loss-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export CMK=cmk-synthetic`.
3. Acknowledge page: `pd incident ack --service crypto-operations --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-crypto-quorum --severity sev0`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.surface=="kms-quorum")'`.
6. Query available quorum: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_kms_operator_quorum_available{tenant_id="'$TENANT'"}'`.
7. Query missing cards: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_kms_operator_card_inventory_missing_total{tenant_id="'$TENANT'"}'`.
8. Query approval timeouts: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(cloud_kms_quorum_approval_timeout_total[5m])'`.
9. Query blocked actions: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(cloud_kms_destructive_action_blocked_total[5m])'`.
10. Open quorum dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/quorum?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
11. Open custody dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-kms-substrate/operator-custody?orgId=1&var-cell=$CELL&var-tenant=$TENANT"`.
12. Read quorum logs: `kubectl -n cloud-kms logs deploy/cloud-kms-quorum-api --since=60m | rg "quorum|custody|operator_card|PIV|CAC"`.
13. Read HSM script logs: `kubectl -n cloud-kms logs deploy/cloud-kms-hsm-worker --since=60m | rg "quorum_script|partition|cryptoshred|byok"`.
14. Check custody roster: `oya kms quorum roster --tenant $TENANT --cell $CELL --output yaml`.
15. Check physical inventory: `oya kms custody inventory --tenant $TENANT --cell $CELL --output yaml`.
16. Compare HSM script roster: `oya kms hsm quorum-script inspect --tenant $TENANT --cell $CELL --cmk $CMK --output yaml`.
17. Verify operator card one: `oya kms custody card verify --tenant $TENANT --operator <operator-1> --cell $CELL`.
18. Verify operator card two: `oya kms custody card verify --tenant $TENANT --operator <operator-2> --cell $CELL`.
19. Verify operator card three: `oya kms custody card verify --tenant $TENANT --operator <operator-3> --cell $CELL`.
20. Check PIV expiry: `oya kms custody piv-expiry --tenant $TENANT --cell $CELL --output table`.
21. Check pending requests: `oya kms quorum requests list --tenant $TENANT --cell $CELL --state pending --output table`.
22. Inspect blocked request: `oya kms quorum request get --tenant $TENANT --request <request-id> --output json`.
23. Check audit submissions: `oya audit-chain query --event-class cloud_kms.quorum.requested --tenant $TENANT --since 24h`.
24. Check audit approvals: `oya audit-chain query --event-class cloud_kms.quorum.approved --tenant $TENANT --since 24h`.
25. Check audit denials: `oya audit-chain query --event-class cloud_kms.quorum.denied --tenant $TENANT --since 24h`.
26. Check CMK state: `oya kms cmk get --tenant $TENANT --cmk $CMK --cell $CELL --output json`.
27. Check cryptoshred queue: `oya kms cryptoshred queue status --tenant $TENANT --cell $CELL --output json`.
28. Check rotation queue: `oya kms rotation queue status --tenant $TENANT --cell $CELL --output json`.
29. Check BYOK import queue: `oya kms byok import queue status --tenant $TENANT --cell $CELL --output json`.
30. Verify no override flag: `oya flags get oya.cloud_kms.quorum.override --tenant $TENANT --cell $CELL --output yaml`.
31. Freeze evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-kms --runbook key-material-quorum-loss --output evidence/incidents/$INCIDENT_ID.json`.
32. Export custody roster: `oya kms quorum roster --tenant $TENANT --cell $CELL --output json > evidence/incidents/$INCIDENT_ID-roster.json`.
33. Export pending requests: `oya kms quorum requests list --tenant $TENANT --cell $CELL --state pending --output json > evidence/incidents/$INCIDENT_ID-requests.json`.
34. Notify commander: `oya notify security-commander --incident $INCIDENT_ID --summary "KMS key material quorum loss triage started"`.
35. Start containment timer: `oya incident timer set --incident $INCIDENT_ID --name quorum-containment --target 30m`.

### Diagnostic Decision Tree
```text
1. Is quorum below policy for any destructive action?
   |-- yes: keep cryptoshred, BYOK import, and rotation paused.
   |-- no: inspect request-specific approval timeout.
2. Is any operator card missing, expired, or unassigned?
   |-- yes: route through custody recovery, not software override.
   |-- no: inspect HSM quorum script and approval service.
3. Does HSM script roster differ from custody roster?
   |-- yes: freeze destructive actions and patch roster projection.
   |-- no: inspect PIV validation and audit-chain emission.
4. Is tenant requesting a quorum reduction?
   |-- yes: reject during incident and route through governance change process.
   |-- no: continue restoration.
5. Are audit approvals missing after valid approval?
   |-- yes: replay audit-chain before action resumes.
   |-- no: close after successful dry-run action and compliance sign-off.
```

## Mitigation
1. Pause destructive actions: `oya flags set oya.cloud_kms.destructive_actions.pause=true --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
2. Pause BYOK import: `oya flags set oya.cloud_kms.byok.import.pause=true --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
3. Pause rotation: `oya flags set oya.cloud_kms.rotation.pause=true --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
4. Confirm no override: `oya flags set oya.cloud_kms.quorum.override=false --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
5. Freeze pending requests: `oya kms quorum requests freeze --tenant $TENANT --cell $CELL --reason $INCIDENT_ID`.
6. Reconcile custody roster dry-run: `oya kms custody reconcile --tenant $TENANT --cell $CELL --dry-run`.
7. Reconcile custody roster confirmed: `oya kms custody reconcile --tenant $TENANT --cell $CELL --confirm $INCIDENT_ID`.
8. Remove invalid operator dry-run: `oya kms quorum operator remove --tenant $TENANT --operator <operator> --dry-run`.
9. Remove invalid operator confirmed: `oya kms quorum operator remove --tenant $TENANT --operator <operator> --confirm $INCIDENT_ID`.
10. Add replacement operator only after governance approval: `oya kms quorum operator add --tenant $TENANT --operator <replacement> --confirm $INCIDENT_ID`.
11. Update HSM script from approved roster: `oya kms hsm quorum-script sync --tenant $TENANT --cell $CELL --confirm $INCIDENT_ID`.
12. Replay missing approval events: `oya audit-chain replay --event-class cloud_kms.quorum.approved --tenant $TENANT --incident $INCIDENT_ID`.
13. Notify tenant custodian: `oya notify tenant-custodian --tenant $TENANT --incident $INCIDENT_ID --template kms-quorum-loss`.
14. Notify compliance: `oya notify compliance --incident $INCIDENT_ID --category key-custody`.
15. Freeze evidence paths: `oya evidence freeze --incident $INCIDENT_ID --paths evidence/incidents/$INCIDENT_ID.json,evidence/incidents/$INCIDENT_ID-roster.json`.
16. Emit mitigation audit: `oya audit-chain emit --event-class EVT_CLOUD_KMS_KEY_QUORUM_LOSS_INCIDENT --incident $INCIDENT_ID --field mitigation=destructive-actions-paused`.
17. Keep decrypt/sign online if they do not require the lost quorum.
18. Keep cryptoshred refused until quorum is restored.
19. Keep BYOK import refused until quorum is restored.
20. Keep quorum reduction requests out of incident mitigation.

## Resolution
1. Restore required M-of-N operator availability.
2. Replace missing card through custody ceremony.
3. Remove departed custodian from HSM script and custody roster.
4. Renew expired PIV/CAC certificates.
5. Patch roster projection if HSM script drifted from custody registry.
6. Patch approval service if valid cards timed out.
7. Patch audit writer if approval events were missing.
8. Add regression fixture for missing operator card.
9. Add regression fixture for roster/script mismatch.
10. Run domain tests: `cargo test -p cloud-kms-domain quorum -- --nocapture`.
11. Run API tests: `cargo test -p cloud-kms-api quorum -- --nocapture`.
12. Run production gate: `cargo run -p dev-cli -- gate validate cloud-kms-quorum --production-snapshot --cell $CELL`.
13. Run destructive dry-run only: `oya kms cryptoshred --tenant $TENANT --cmk $CMK --dry-run --require-quorum`.
14. Unpause gated actions: `oya flags set oya.cloud_kms.destructive_actions.pause=false --tenant $TENANT --cell $CELL --reason resolved-$INCIDENT_ID`.
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_KMS_KEY_QUORUM_LOSS_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudKmsOperatorQuorumLost` is green.
- `CloudKmsCustodyCardMissing` is green.
- `cloud_kms_operator_quorum_available` meets policy.
- Custody roster and HSM script roster match.
- All active operator cards verify.
- Destructive dry-run proves quorum without executing destruction.
- BYOK import dry-run proves quorum without importing material.
- Rotation dry-run proves quorum without promotion.
- Audit-chain contains requested, approved, mitigation, and resolution events.
- Compliance owner signs off before close.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-kms-key-material-quorum-loss
microservice: cloud-kms
event_class: EVT_CLOUD_KMS_KEY_QUORUM_LOSS_INCIDENT
incident_id: <INC-...>
severity: sev0
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Key Material Quorum Loss postmortem

## Summary
- Which tenant, CMK family, tier, and custody group lost quorum.
- Which actions were blocked.
- Whether any key material action proceeded while quorum was unhealthy.

## Timeline
- Quorum loss detected:
- Gated actions paused:
- Custody reconciled:
- Quorum restored:
- Audit sealed:

## Custody Evidence
- Roster export:
- HSM script export:
- Card verification:

## Root Cause
- Missing card:
- Expired credential:
- Roster drift:
- Approval service:

## Corrective Actions
- Owner:
- Due date:
- Ceremony update:
- Regression test:
```

## Escalation Path
- Page `crypto-operations-primary` for every quorum loss.
- Page `cloud-kms-primary` for software queue, HSM script, or API failures.
- Page `compliance-primary` for regulated custody evidence gaps.
- Page tenant custodian contacts when tenant-owned operator cards are missing.
- Notify `#inc-crypto-quorum` with tenant, tier, and action queue scope.
- Notify `#support-cloud-kms-security` before tenant-facing messages.
- Notify `#legal-review` when cryptoshred or deletion rights are delayed.
- Escalate to executive incident commander when destructive actions are blocked for more than 4 hours.
- Escalate to vendor support only through crypto operations.
- Do not accept verbal approval as quorum restoration.

## Cross-µservice Coordination
- `audit-chain`: seal quorum request, approval, denial, mitigation, and resolution events.
- `tenancy`: confirm tenant custody roster and current tenant admin contacts.
- `cloud-iam`: verify operator principals and PIV/CAC attributes.
- `cloud-network`: preserve access to HSM management plane without widening egress.
- `workflow-engine`: pause workflows that request cryptoshred, BYOK import, or rotation.
- `compliance`: own regulated custody and deletion-rights reporting.
- `support`: manage tenant custodian communication.
- `cloud-billing`: annotate SLA-credit candidates for delayed destructive requests.
- `observability`: attach quorum and custody dashboard snapshots.
- `security`: review any override attempt.
- `foundry`: pause key-custody mutation pipelines.
- `comms-email`: send approved external notifications.

## Runbook Maintenance
- Update custody command names after every ceremony tooling change.
- Keep destructive-action pause commands explicit.
- Add any new quorum-gated action to Trigger Conditions.
- Review this runbook during every paid custody ceremony.
- Never add a software override path to this runbook.
