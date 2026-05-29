---
doc_class: MigrationPlaybook
microservice: ops-dashboard-control-center
vendor: PagerDuty + incident.io + ServiceNow ITSM + FireHydrant + Atlassian Statuspage
date: 2026-05-20
doc_status: published
---

# Migration playbook — PagerDuty / incident.io / ServiceNow ITSM / FireHydrant / Atlassian Statuspage → oyatie ODCC

Audience: an oyatie tenant migrating their operator-surface stack from PagerDuty + incident.io + ServiceNow ITSM (or any combination) to oyatie's `ops-dashboard-control-center` µservice.

## Why this migration is non-trivial

- **PagerDuty** owns the on-call rotation + escalation + alerting topology. Migrating means re-modelling the rotation in oyatie + recreating the routing rules from PD's "event-rule" engine.
- **incident.io / FireHydrant / Rootly** own the incident-management workflow (Slack-channel-creation, status updates, postmortem-template). Migration requires re-modelling the incident lifecycle in ODCC.
- **ServiceNow ITSM** owns the change-management approval workflow (CAB approvals, change tickets). Migration requires re-modelling the approval gates as Cedar policies + ODCC deployment-approval commands.
- **Atlassian Statuspage** owns the customer-facing comms surface. ODCC is operator-facing; customer comms remain on Statuspage OR migrate to oyatie comms-email + oyatie tenant-portal.

The 80/20: the alert ingestion + incident-state-machine + audit-trail ports cleanly via the auto-converter; the 20% needing care is on-call rotation modelling + ServiceNow CAB approval logic translation.

## Step 1 — Inventory existing operator-surface stack (≤ 2 weeks)

For PagerDuty:

```sh
oya odcc migrate inventory \
    --source pagerduty \
    --pagerduty-api-token "$PD_TOKEN" \
    --pagerduty-account-id "$PD_ACCOUNT" \
    --out inventory/pagerduty.yaml
```

Captures: services, schedules, escalation-policies, event-rules, integrations (Slack, Jira, Datadog), incident-priorities, response-plays, runbook URLs.

For incident.io:

```sh
oya odcc migrate inventory \
    --source incident-io \
    --incident-io-api-token "$INCIDENT_IO_TOKEN" \
    --out inventory/incident-io.yaml
```

Captures: incident-types, severity-levels, role-assignments, custom-fields, post-mortem-templates, Slack integration.

For ServiceNow ITSM:

```sh
oya odcc migrate inventory \
    --source servicenow-itsm \
    --servicenow-instance "company.service-now.com" \
    --servicenow-api-token "$SNOW_TOKEN" \
    --modules "incident,problem,change,cab,knowledge,runbook" \
    --out inventory/servicenow-itsm.yaml
```

Captures: change-types, approval-policies, CAB-membership, change-templates, runbook KB articles, incident-priority-matrix.

For FireHydrant:

```sh
oya odcc migrate inventory \
    --source firehydrant \
    --firehydrant-api-token "$FH_TOKEN" \
    --out inventory/firehydrant.yaml
```

For Atlassian Statuspage:

```sh
oya odcc migrate inventory \
    --source statuspage \
    --statuspage-api-token "$SP_TOKEN" \
    --out inventory/statuspage.yaml
```

## Step 2 — Model the on-call rotation in oyatie identity (≤ 2-4 weeks)

PagerDuty schedules don't directly map; you model them as oyatie operator-principals + scheduling rules:

```sh
oya identity operator-rotation create \
    --rotation-name oncall-sre-syd-rotation \
    --members "alex@drill-acme,brenda@drill-acme,carlos@drill-acme,denise@drill-acme,erik@drill-acme" \
    --schedule "follow-the-sun-syd-eu-us" \
    --rotation-cycle "weekly" \
    --tier "primary" \
    --escalation-after 15m \
    --escalation-to "oncall-sre-syd-rotation-secondary"
```

Repeat for secondary + manager rotations. Each rotation is itself a Cedar principal that Cedar gates can permit.

Translate PagerDuty event-rules → Cedar policies:

PagerDuty event-rule: "if incident severity = SEV1 AND service = messenger, page primary on-call AND escalate-after 10m":

```cedar
permit (
    principal == OperatorRotation::"oncall-sre-syd-rotation",
    action == Action::"odcc::incident::declare",
    resource is OdccIncident
) when {
    resource.severity == "SEV1" &&
    resource.service == "messenger"
};
```

Auto-page rules (PD's "auto-page" via integration with monitoring tools) translate to ODCC's alert-routing rules (`oya odcc alert-routing add`).

## Step 3 — Model the incident lifecycle in ODCC (≤ 2-4 weeks)

incident.io / FireHydrant / Rootly all use a state machine: open → mitigating → fixed → resolved → postmortem-complete. ODCC's state machine:

| incident.io / FireHydrant | ODCC equivalent | Notes |
|---|---|---|
| Open | DECLARED | `oya odcc incident declare` |
| Mitigating | IN_PROGRESS | `oya odcc incident update --state IN_PROGRESS` |
| Fixed | MITIGATED | `oya odcc incident update --state MITIGATED` |
| Resolved | RESOLVED | `oya odcc incident resolve` |
| Postmortem complete | POSTMORTEM_COMPLETE | `oya odcc incident postmortem complete` |

For Slack-channel-creation:

```sh
oya odcc incident integration slack \
    --workspace oya-messenger-syd-1 \
    --channel-template "incident-{date}-{slug}" \
    --auto-invite-members "from-rotation"
```

oyatie comms layer (messenger) auto-creates the incident channel.

For post-mortem templates:

```sh
oya odcc postmortem template register \
    --template-name "blameless-5-why" \
    --template-file templates/postmortem-blameless-5-why.md \
    --required-fields "summary,timeline,contributing-factors,5-why-root-cause,corrective-actions"
```

## Step 4 — Translate ServiceNow CAB approvals → Cedar policies (≤ 4-8 weeks)

This is the longest-pole step. ServiceNow CAB membership + approval logic doesn't translate 1:1; you re-model as Cedar policies:

ServiceNow CAB approval rule: "Change to production messenger service requires approval from messenger-team-lead + sre-on-call + security-officer":

```cedar
permit (
    principal,
    action == Action::"odcc::deployment::approve",
    resource is OdccDeployment
) when {
    resource.service == "messenger" &&
    resource.cell.includes("prod") &&
    context.approvals.contains(Principal::"messenger-team-lead") &&
    context.approvals.contains(Principal::"oncall-sre-syd-rotation") &&
    context.approvals.contains(Principal::"security-officer-rotation")
};
```

The Cedar evaluator requires all three approval principals to have signed (each via WebAuthn step-up) before the deployment-approve command is permitted. This is enforced at the gate (not the application).

For "standard change" CAB pre-approval (recurring changes that bypass CAB):

```cedar
permit (
    principal == OperatorRotation::"oncall-sre-syd-rotation",
    action == Action::"odcc::deployment::approve",
    resource is OdccDeployment
) when {
    resource.deployment_type == "standard-change-pre-approved" &&
    resource.service in ["messenger", "social", "calendar"] &&
    resource.canary_stage <= 5
};
```

For "emergency change":

```cedar
permit (
    principal,
    action == Action::"odcc::deployment::approve",
    resource is OdccDeployment
) when {
    resource.deployment_type == "emergency-change" &&
    context.linked_incident.severity in ["SEV0", "SEV1"] &&
    context.approvals.count >= 1 &&
    context.step_up_tier >= 2
};
```

## Step 5 — Re-author alert integrations (≤ 2-4 weeks)

PagerDuty integrates with ~ 700 monitoring tools out-of-box. ODCC integrates via:

```sh
oya odcc alert-routing source register \
    --source-name datadog \
    --webhook-url "https://odcc-api.drill-syd-1.oyatie.local/v1/alerts/datadog-webhook" \
    --hmac-secret-source kms://syd-hsm-cluster-prod-1/odcc-datadog-webhook-secret \
    --idempotency-extraction "from-event-key"
```

For Prometheus AlertManager:

```sh
oya odcc alert-routing source register \
    --source-name prometheus-alertmanager \
    --webhook-url "https://odcc-api.drill-syd-1.oyatie.local/v1/alerts/alertmanager-webhook" \
    --hmac-secret-source kms://syd-hsm-cluster-prod-1/odcc-alertmanager-webhook-secret \
    --idempotency-extraction "from-alert-fingerprint"
```

Map alert → incident-declare-rule:

```sh
oya odcc alert-routing rule add \
    --source-name datadog \
    --match-condition "alert.alert_type=='metric' && alert.priority=='P1' && alert.tags contains 'service:messenger'" \
    --action "declare-incident" \
    --severity SEV1 \
    --commander-rotation oncall-sre-syd-rotation \
    --comm-channel-template "incident-{date}-{alert-slug}"
```

## Step 6 — Migrate runbooks (≤ 2-4 weeks)

ServiceNow KB articles + PagerDuty runbook URLs → oyatie ODCC journey-anchored runbooks:

```sh
oya odcc runbook import \
    --source servicenow-kb \
    --kb-file inventory/servicenow-runbooks.json \
    --target-dir microservices/messenger/runbooks/ \
    --convert-to oyatie-runbook-format
```

The converter normalises the format + emits a `runbooks/<service>-<failure-class>.md` per article. Each runbook is then journey-anchored (linked from the incident-class panel in ODCC).

## Step 7 — Dual-run period (≤ 4-8 weeks)

Run PagerDuty + ODCC in parallel:

- Alerts route to BOTH PagerDuty + ODCC.
- Incident declaration happens in both systems.
- ServiceNow CAB approvals are mirrored to ODCC `deployment::approve` commands.

Compare:

```sh
oya odcc migrate dual-run-divergence \
    --since 24h \
    --compare-fields "incident-state,severity,commander,mitigation-actions" \
    --report-out dual-run-divergence-2026-05-20.json
```

Common divergence:

- PagerDuty's "snooze incident" has no ODCC equivalent (ODCC has hold/postpone instead).
- ServiceNow's "change-collision-detection" has no ODCC equivalent (ODCC has change-conflict-Cedar-rule instead).
- incident.io's "private incident channel" maps cleanly to ODCC's "incident comm channel" but channel-IDs differ.

## Step 8 — Cutover (≤ 2 weeks)

Cutover one source at a time:

1. Statuspage: keep running (customer-facing); ODCC ↔ Statuspage bridge updates Statuspage on incident state changes.
2. ServiceNow ITSM: cutover by retiring the CAB-approval gate in ServiceNow + Cedar-only gate in ODCC.
3. PagerDuty: cutover by changing the alert webhook URL to ODCC only.
4. incident.io / FireHydrant: cutover by retiring their incident-channel-creation hook.

Per-source cutover:

```sh
oya odcc migrate cutover \
    --source pagerduty \
    --target oyatie-odcc \
    --cutover-time 2026-08-15T09:00:00-04:00 \
    --rollback-buffer 30d
```

At cutover, the source becomes read-only (no new incidents); ODCC becomes authoritative.

## Step 9 — Decommission (≤ 1 month)

```sh
oya odcc migrate decommission \
    --source pagerduty \
    --evidence-out evidence/migrations/pagerduty-to-odcc-drill-acme.json
```

Evidence file includes: PagerDuty rotation IDs migrated, event-rules translated to Cedar policies, runbooks imported, integrations re-pointed. Used for SOC2 audit of migration.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| On-call rotation migration breaks 24/7 coverage | Critical | dual-run with both PagerDuty + ODCC paging the same humans for 4-8 wk; verify same humans paged |
| ServiceNow CAB approval logic doesn't translate 1:1 | Critical | per-change-type re-modelling; legal + audit + change-mgmt review |
| PagerDuty event-rule engine is more expressive than Cedar in some edge cases | High | identify edge cases; either extend Cedar policy OR keep rule in monitoring-tool layer |
| Audit-trail discontinuity between PagerDuty + ODCC | High | dual-run audit-chain emission; reconcile post-cutover |
| Status-update flow change (incident.io's Slack-channel-template differs from ODCC's) | Medium | retrain on-call team on new channel-name convention |
| Runbook URLs in alert payloads break | Medium | dual-run alert routing; verify runbook URLs work |
| ServiceNow custom-field translations (CAB-defined) | High | per-tenant review; some fields map to Cedar attributes, some to incident metadata |
| WebAuthn enrollment delay for on-call team | Medium | start WebAuthn enrollment 4 wk before cutover |
| Sovereign-pack tenants require additional approval logic | Critical | per-pack Cedar policy review; legal pre-cutover |
| FIPS-140-2 Level 3 HSM partition availability per region | High | provision HSM 4 wk before cutover; alarm partition-unavailability |

## Source citations

- PagerDuty Migration Guide (vendor docs, accessed 2026-05).
- ServiceNow ITSM Best Practices Guide v22 (CAB approval semantics).
- incident.io API Reference (vendor docs, accessed 2026-05).
- FireHydrant Documentation (accessed 2026-05).
- ITIL v4 Foundation (change-management vocabulary).
