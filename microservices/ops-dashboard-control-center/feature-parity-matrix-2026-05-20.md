# ops-dashboard-control-center Feature Parity Matrix - 2026-05-20

Audit target: `microservices/ops-dashboard-control-center/`.
Counterpart 1: Datadog.
Counterpart 2: PagerDuty.
Counterpart 3: AWS CloudWatch plus AWS Systems Manager.
Service purpose evidence: `PRD.md:9-22`, `README.md:21-42`, `manifest.json:6-75`, `contracts/openapi/ops-dashboard-control-center.yaml:29-209`.
Method: compare the current service artifact surface to the union of the three counterpart product families.
Important scope boundary: ODCC is an Oyatie operator command and evidence control center, not a literal clone of any counterpart.
Important coverage rule: if a counterpart has a relevant operator-control capability, ODCC needs either native coverage, an explicit handoff, or a documented non-goal.
Important tenant rule: capability quality is uniform across `demo_trial`, `paid`, and `revenue_share`; deployment and billing overlays may constrain usage but not safety semantics.
Important retirement rule: this file does not introduce retired commercial tenant_class models.

## Counterpart 1 - Datadog Capability Surface

Datadog surface 01: infrastructure metrics ingestion and visualization.
Datadog surface 02: application performance monitoring and trace correlation.
Datadog surface 03: log ingestion, search, filtering, and archive workflows.
Datadog surface 04: dashboards that combine metrics, logs, traces, service metadata, and incidents.
Datadog surface 05: monitors and alerting with threshold, anomaly, forecast, composite, and SLO inputs.
Datadog surface 06: incident management linked to monitors, security signals, events, and cases.
Datadog surface 07: incident analytics that track response efficiency and customer impact.
Datadog surface 08: workflow automation triggered by incidents, monitors, or manual operator action.
Datadog surface 09: audit trail over platform and API activity.
Datadog surface 10: usage attribution and API consumption telemetry.
Datadog surface 11: service catalog and service ownership metadata.
Datadog surface 12: dependency mapping and service maps.
Datadog surface 13: Kubernetes, container, host, cloud, and network infrastructure views.
Datadog surface 14: multi-cloud integrations including AWS, Azure, Google Cloud, OCI, and Kubernetes.
Datadog surface 15: notebook and post-incident report composition.
Datadog surface 16: incident declaration from multiple signals.
Datadog surface 17: custom incident fields and searchable incident records.
Datadog surface 18: status and stakeholder update workflows.
Datadog surface 19: integration with chat and collaboration tools.
Datadog surface 20: team and role metadata for routing and ownership.
Datadog surface 21: event management and event correlation.
Datadog surface 22: security signal handoff to incident workflows.
Datadog surface 23: case management for follow-up work.
Datadog surface 24: SLO dashboards and burn-rate monitoring.
Datadog surface 25: API rate-limit observability through usage metrics and headers.
Datadog surface 26: log intake limits and payload validation.
Datadog surface 27: region and site choices for data residency.
Datadog surface 28: browser and mobile-accessible operator console.
Datadog surface 29: user, team, and permission administration.
Datadog surface 30: alert noise reduction and correlation.
Datadog surface 31: custom widgets and dashboard layout control.
Datadog surface 32: saved views and filtered incident lists.
Datadog surface 33: integration marketplace.
Datadog surface 34: compliance and audit evidence exports.
Datadog surface 35: real-time operational telemetry pivoting.
Datadog surface 36: incident timeline and activity log.
Datadog surface 37: monitor-to-incident conversion.
Datadog surface 38: incident severity, commander, and responder roles.
Datadog surface 39: postmortem handoff.
Datadog surface 40: governance through API keys, application keys, service accounts, and audit trail.
ODCC current match: strong on incident declaration, deployment command, evidence export, tenant posture, and Cedar policy evidence.
ODCC current gap: shallow on deep telemetry storage, log search, trace exploration, anomaly detection, event correlation, and service map ownership.
ODCC intended handoff: deep metrics/logs/traces belong to `observability`, while ODCC should expose pivots and command context.
ODCC parity criterion: every Datadog-like pivot must have a bounded handoff rather than duplicating the observability platform.

## Counterpart 2 - PagerDuty Capability Surface

PagerDuty surface 01: incident triggering from alerts and events.
PagerDuty surface 02: service directory and service ownership.
PagerDuty surface 03: escalation policies and schedules.
PagerDuty surface 04: on-call routing and responder notification.
PagerDuty surface 05: incident acknowledgement, reassignment, escalation, and resolution.
PagerDuty surface 06: business service status and dependency-aware impact.
PagerDuty surface 07: event orchestration and noise suppression.
PagerDuty surface 08: operations console for incident and service state.
PagerDuty surface 09: AIOps grouping and related incident discovery.
PagerDuty surface 10: stakeholder communication and status update templates.
PagerDuty surface 11: status page integration.
PagerDuty surface 12: incident priority and incident type metadata.
PagerDuty surface 13: incident tasks and response plays.
PagerDuty surface 14: post-incident review and learning records.
PagerDuty surface 15: runbook automation and workflow automation.
PagerDuty surface 16: service graph and service dependencies.
PagerDuty surface 17: change event correlation.
PagerDuty surface 18: maintenance windows.
PagerDuty surface 19: team and user role administration.
PagerDuty surface 20: API access keys and scoped OAuth app access.
PagerDuty surface 21: REST API rate-limit headers and 429 behavior.
PagerDuty surface 22: webhook extensions.
PagerDuty surface 23: integrations with monitoring, ITSM, chat, and CI/CD tools.
PagerDuty surface 24: mobile push, phone, email, and SMS notification paths.
PagerDuty surface 25: notification rules and contact methods.
PagerDuty surface 26: service-level settings and standards.
PagerDuty surface 27: analytics and incident activity insights.
PagerDuty surface 28: dynamic notification behavior.
PagerDuty surface 29: business-impact and customer-impact fields.
PagerDuty surface 30: auditability of incident changes.
PagerDuty surface 31: automation approvals and delegated execution.
PagerDuty surface 32: responder accountability and escalation timeout behavior.
PagerDuty surface 33: cross-system integration with Datadog and CloudWatch.
PagerDuty surface 34: incident command workflow consistency.
PagerDuty surface 35: role-specific operations views.
PagerDuty surface 36: event rules, cache variables, and enrichment.
PagerDuty surface 37: service dependency import and manual mapping.
PagerDuty surface 38: response process templates.
PagerDuty surface 39: API pagination and bulk synchronization patterns.
PagerDuty surface 40: operational learning loop after incidents.
ODCC current match: strong on incident declaration, severity, remediation approval, on-call handoff runbook, evidence export, and rollback decision records.
ODCC current gap: no explicit schedules, contact methods, phone/SMS/push notification model, service directory parity, event orchestration, or stakeholder broadcast contract.
ODCC intended handoff: notification delivery and roster source of truth should be an identity/on-call/communications seam if ODCC does not own it.
ODCC parity criterion: PagerDuty-like response accountability must be represented either natively or as an explicit dependency contract.

## Counterpart 3 - AWS CloudWatch plus Systems Manager Capability Surface

AWS surface 01: CloudWatch metrics collection and custom metric publication.
AWS surface 02: CloudWatch dashboards across accounts and Regions.
AWS surface 03: CloudWatch alarms and composite alarms.
AWS surface 04: CloudWatch Logs and Logs Insights.
AWS surface 05: CloudWatch metric math and Metrics Insights.
AWS surface 06: CloudWatch SLO and Application Signals capabilities.
AWS surface 07: CloudWatch service dependency and state operations APIs.
AWS surface 08: CloudWatch investigations and operational analysis.
AWS surface 09: CloudWatch cross-account observability.
AWS surface 10: CloudWatch agent for hosts and nodes.
AWS surface 11: CloudWatch alarm action and mute controls.
AWS surface 12: metric streams and external sink integration.
AWS surface 13: Systems Manager OpsCenter for operational issues.
AWS surface 14: Systems Manager Automation runbooks.
AWS surface 15: Systems Manager Run Command.
AWS surface 16: Systems Manager State Manager associations.
AWS surface 17: Systems Manager Patch Manager.
AWS surface 18: Systems Manager Session Manager without bastion hosts.
AWS surface 19: Systems Manager Inventory.
AWS surface 20: Systems Manager Distributor.
AWS surface 21: Systems Manager Change Manager and approvals.
AWS surface 22: Systems Manager Incident Manager integrations.
AWS surface 23: managed node registration and fleet state.
AWS surface 24: IAM service role delegation for automations.
AWS surface 25: CloudTrail audit for management actions.
AWS surface 26: resource group targeting.
AWS surface 27: concurrency and error thresholds for automation.
AWS surface 28: automation queues and pending states.
AWS surface 29: multi-account and multi-Region automation.
AWS surface 30: hybrid and multicloud node management.
AWS surface 31: operational playbooks embedded in dashboards.
AWS surface 32: controlled remediation without direct SSH.
AWS surface 33: service quota awareness for control-plane APIs.
AWS surface 34: resource tagging and ownership filters.
AWS surface 35: maintenance windows.
AWS surface 36: compliance reporting for patches and inventory.
AWS surface 37: API throttling behavior for dashboard, alarm, and metric operations.
AWS surface 38: delegated administration.
AWS surface 39: secure command logging to S3, CloudWatch Logs, and CloudTrail.
AWS surface 40: operational visibility over cloud resources and managed nodes.
ODCC current match: strong on no-SSH command posture, rollback/deployment approvals, cluster health, evidence exports, and runbook-oriented operator flows.
ODCC current gap: no context-specific AWS guest mapping, no Systems Manager automation import/export contract, no node inventory schema, no Run Command analogue contract, and no OpenTofu modules.
ODCC intended handoff: cloud-provider primitives should remain behind Oyatie `cloud-*`, `cell`, `network`, `identity`, `observability`, and `cloud-iac` seams.
ODCC parity criterion: ODCC must normalize AWS-like operational primitives into Oyatie abstractions without directly becoming an AWS wrapper.

## UNION-Coverage Matrix

| # | Union capability | Datadog | PagerDuty | AWS CloudWatch plus Systems Manager | ODCC current evidence | Coverage state |
| --- | --- | --- | --- | --- | --- | --- |
| 001 | Incident declaration | Yes | Yes | Partial through Incident Manager/alarms | `PRD.md:15-16`; OpenAPI incident route `contracts/openapi/ops-dashboard-control-center.yaml:29-71` | Covered design |
| 002 | Incident severity | Yes | Yes | Partial | Manifest seal event `OpsIncidentSeverityChanged` in `manifest.json:328-330` | Partial, event contract missing |
| 003 | Incident acknowledgement | Partial | Yes | Partial | SLO file `slos/incident-ack-latency.openslo.yaml` | Covered SLO, contract needs action semantics |
| 004 | Incident remediation approval | Workflow automation | Yes | Automation approvals | Capability `incident-remediation-approve.yaml`; manifest event `manifest.json:330` | Covered design, AsyncAPI gap |
| 005 | Escalation policy | Team routing | Yes | Incident Manager style | On-call handoff runbook exists at `runbooks/oncall-handoff-failure.md` | Partial |
| 006 | On-call schedule | Team metadata | Yes | Not primary | `IP-016-on-call-handoff-bc.md` exists | Gap for roster source |
| 007 | Contact methods | Integration | Yes | SNS/Incident Manager outside scope | No contact-method contract found | Gap |
| 008 | Phone/SMS/push delivery | Integration | Yes | SNS-like outside scope | No notification delivery contract found | Gap or dependency |
| 009 | Stakeholder updates | Yes | Yes | Dashboard/playbook | No explicit stakeholder broadcast contract found | Gap |
| 010 | Status page handoff | Partial | Yes | Partial | No status page contract found | Gap |
| 011 | Post-incident review | Yes | Yes | OpsCenter notes possible | `incident-response.md:76-81` post-incident section | Partial |
| 012 | Incident analytics | Yes | Yes | CloudWatch metrics | SLO and dashboard artifacts exist | Partial |
| 013 | Alert-to-incident conversion | Yes | Yes | Alarms | No alert-source conversion contract found | Gap |
| 014 | Event orchestration | Event management | Yes | EventBridge/SSM possible | AsyncAPI event channels exist | Partial |
| 015 | Noise suppression | Event correlation | Yes | Composite alarms | No suppression policy found | Gap |
| 016 | Related incidents | Analytics | AIOps | Investigations | No related-incident schema found | Gap |
| 017 | Service directory | Service catalog | Service directory | Resource groups/tags | Catalog YAML files exist | Partial |
| 018 | Service ownership | Teams/service catalog | Services/teams | Tags/IAM | Owner `ops-sre-reliability` in `manifest.json:5` | Covered basic |
| 019 | Service dependencies | Service map | Service graph | Application Signals/dependency APIs | `depends_on_microservices` in `manifest.json:531-546` | Partial |
| 020 | Business service impact | Incident fields | Business services | Resource groups | No customer/business-impact schema found | Gap |
| 021 | Dashboard overview | Yes | Operations console | CloudWatch dashboards | Dashboards directory has six artifacts | Covered design |
| 022 | Custom dashboard widgets | Yes | Configurable tables | CloudWatch widgets | JSON dashboards exist | Partial |
| 023 | Cross-account dashboard | Multi-org | Account views | Native | No six-context IaC or context dashboard schema | Gap |
| 024 | Cross-Region dashboard | Multi-site | Global ops | Native | `multi-region.md` exists | Partial |
| 025 | Metrics ingestion | Yes | Indirect | Native | Metric naming convention exists | Dependency on observability |
| 026 | Log search | Yes | Not primary | Logs Insights | No log query contract found | Dependency gap |
| 027 | Trace pivot | Yes | Not primary | X-Ray/App Signals related | `IP-015-observability-pivot.md` exists | Partial |
| 028 | SLO burn | Yes | Analytics | CloudWatch SLO | Nine SLO files exist | Covered design |
| 029 | Error-budget action | Monitors/workflows | Escalation | Alarms/automation | Runbooks and SLOs exist | Partial |
| 030 | Audit trail | Yes | Yes | CloudTrail | Manifest audit chain `manifest.json:325-340` | Covered design |
| 031 | Signed evidence export | Compliance export | Post-incident artifacts | CloudTrail/log exports | Evidence export route `contracts/openapi/ops-dashboard-control-center.yaml:189-209` | Strong differentiator |
| 032 | Evidence pack freshness | Not primary | Review artifacts | Logs/exports | `slos/evidence-pack-freshness.openslo.yaml` | Covered |
| 033 | Evidence pack integrity | Audit trail | Audit logs | CloudTrail/checksums | FAQ and tutorial discuss signing, but retired capability labels need rewrite | Partial |
| 034 | Policy decision review | Audit/security | Permissions/admin | IAM/CloudTrail | OpenAPI policy route `contracts/openapi/ops-dashboard-control-center.yaml:72-93` | Covered design |
| 035 | Cedar authorization | Not native | Permissions | IAM | Cedar policy files exist | Strong differentiator |
| 036 | Step-up authentication | Auth integration | User roles | IAM/MFA | Capability and SLO exist | Covered design |
| 037 | Idempotent commands | API design | API design | SSM execution tokens | OpenAPI idempotency header `contracts/openapi/ops-dashboard-control-center.yaml:211-216` | Covered design |
| 038 | Deployment approval | Workflow automation | Change events | Change Manager | OpenAPI deployment route `contracts/openapi/ops-dashboard-control-center.yaml:94-130` | Covered design |
| 039 | Progressive rollout | Workflow automation | Change correlation | SSM/CodeDeploy adjacent | `IP-003-deployment-approval-and-rollback.md` | Partial |
| 040 | Rollback execution | Workflow automation | Runbook automation | Systems Manager Automation | OpenAPI rollback route `contracts/openapi/ops-dashboard-control-center.yaml:131-152` | Covered design |
| 041 | Freeze window | Workflow guard | Maintenance windows | Maintenance windows | Manifest deployment bounded context mentions freeze windows `manifest.json:21-33` | Partial |
| 042 | Runbook automation | Workflow automation | Runbook automation | Systems Manager Automation | Runbooks directory exists | Partial |
| 043 | Automation concurrency | Workflow quotas | Automation rules | SSM concurrency/error thresholds | No concurrency model in contracts | Gap |
| 044 | Automation queue state | Workflow runs | Pending workflows | SSM pending queues | No queue-state contract found | Gap |
| 045 | Managed-node inventory | Infra views | Not primary | Systems Manager Inventory | Cluster health route exists | Partial |
| 046 | Node command execution | Not primary | Runbook action | Run Command | ODCC forbids shell; mediated workflows only | Covered by intentional non-goal |
| 047 | Session access | Not primary | Not primary | Session Manager | PRD forbids SSH and console bypass | Intentional non-goal |
| 048 | Patch compliance | Compliance dashboard | Not primary | Patch Manager | No patch compliance schema found | Gap or dependency |
| 049 | Cluster health | Infrastructure views | Service status | CloudWatch/SSM health | OpenAPI health route `contracts/openapi/ops-dashboard-control-center.yaml:153-170` | Covered design |
| 050 | Recovery workflow | Workflow automation | Runbook automation | Automation | Manifest recovery events `manifest.json:337-338` | Partial, AsyncAPI gap |
| 051 | Multi-region operation | Multi-site | Global service | Cross-Region | `multi-region.md` exists | Partial |
| 052 | Data residency | Site choices | Service region controls | Region/account controls | `residency-and-pack-boundary.md`; `policy/data-residency.md` | Covered design |
| 053 | Compliance packs | Compliance views | Audit reports | Compliance reporting | `compliance.md` and journey IPs exist | Covered design |
| 054 | Pack conflict resolution | Not primary | Not primary | Not primary | `IP-journey-j99-multi-pack-conflict-resolution.md` | Differentiator |
| 055 | Tenant isolation posture | Multi-org | Account/service scopes | Account/resource scopes | OpenAPI tenant route `contracts/openapi/ops-dashboard-control-center.yaml:171-188` | Covered design |
| 056 | Tenant class awareness | Usage/billing | Plan/account context | Account/quota context | No tenant-class terms found | Gap |
| 057 | Usage cap display | Usage attribution | Account limits | Service quotas | No tenant-class or cap schema | Gap |
| 058 | Revenue share display | Not native | Not native | Not native | No terms found | Gap |
| 059 | Billing usage pivot | Usage attribution | Billing integration | Cost Explorer adjacent | `IP-014-finops-portal-integration.md` and finops dependency | Partial |
| 060 | Cost budget guardrails | Usage attribution | Plan controls | AWS budgets adjacent | `cost-budget.md:16-26` | Partial |
| 061 | API rate-limit handling | Headers/usage metrics | Headers/429 | Service quotas/throttling | No ODCC public rate-limit contract found | Gap |
| 062 | API pagination | API design | REST lists | AWS APIs | OpenAPI route shapes need pagination review | Partial |
| 063 | API keys/OAuth | API/application keys | API keys/OAuth | IAM | Manifest depends on identity; no ODCC auth scheme detail in OpenAPI evidence | Partial |
| 064 | OpenAPI contract | Yes | Yes | Yes | ODCC OpenAPI exists | Covered |
| 065 | Async events | Events | Webhooks | EventBridge | ODCC AsyncAPI exists | Partial |
| 066 | Proto/RPC contract | Not primary | Not primary | AWS APIs | ODCC proto exists | Covered |
| 067 | Webhooks | Integrations | Webhooks | EventBridge | No webhook contract found | Gap or dependency |
| 068 | ChatOps | Integrations | ChatOps | Integrations | No chat contract found | Gap or dependency |
| 069 | Mobile operator UX | Mobile app | Mobile app | Console/mobile app | No mobile frontend artifact found | Gap |
| 070 | Web operator UX | Web app | Web app | Console | Dashboards and catalog app files exist, but no frontend source | Partial |
| 071 | Native desktop/mobile | Not primary | Mobile app | Console/apps | No Swift/Kotlin/WinUI3 source | Gap if claimed |
| 072 | Localization escalation | Not primary | Global support | Region-specific | PRD and runbook include localization escalation | Covered design |
| 073 | Korean escalation | Not primary | Not primary | Not primary | `runbooks/kr-localization-escalation.md` | Covered design |
| 074 | Government audit docket | Compliance | Compliance | Audit logs | `IP-journey-j126-3pao-docket-dashboard.md` | Differentiator |
| 075 | SOX controls test pane | Compliance | Compliance | CloudTrail evidence | `IP-journey-j137-corporate-internal-audit-sox-controls-test-audit-pane.md` | Differentiator |
| 076 | Cedar misuse policy pane | Security/audit | Security/audit | IAM audit | `IP-journey-j139-internal-audit-cedar-permit-misuse-policy-pane.md` | Differentiator |
| 077 | Export tracking surface | Exports | Post-incident docs | Logs/export | `IP-journey-j143-export-tracking-surface.md` | Differentiator |
| 078 | Ombudsman operator console | Not primary | Not primary | Not primary | `IP-journey-j19-ombudsman-operator-console.md` | Differentiator |
| 079 | Auditor console | Audit trail | Reports | CloudTrail/Config | `IP-journey-j68-auditor-console.md` | Differentiator |
| 080 | Operator evidence console | Audit/evidence | Postmortem docs | Logs/CloudTrail | Journey IPs j77-j88 | Differentiator |
| 081 | MSB/MTL overlay | Compliance | Not primary | Compliance | `IP-journey-j91-us-msb-mtl-overlay.md` | Differentiator |
| 082 | LGPD/DSAR overlay | Compliance | Not primary | Region compliance | `IP-journey-j92-br-lgpd-us-parent-dsar.md` | Differentiator |
| 083 | DPDPA/RBI overlay | Compliance | Not primary | Region compliance | `IP-journey-j93-in-dpdpa-rbi-overlay.md` | Differentiator |
| 084 | SOX404 overlay | Compliance | Not primary | Audit logs | `IP-journey-j94-sox404-public-company-controls.md` | Differentiator |
| 085 | ISO/SOC2 audit overlay | Compliance | Compliance | Compliance | `IP-journey-j95-iso27001-soc2-annual-audit.md` | Differentiator |
| 086 | MENA onboarding overlay | Region compliance | Not primary | Region controls | `IP-journey-j96-ksa-uae-mena-onboarding.md` | Differentiator |
| 087 | SG MAS overlay | Region compliance | Not primary | Region controls | `IP-journey-j97-sg-pdpa-mas-tenant.md` | Differentiator |
| 088 | AU APRA overlay | Region compliance | Not primary | Region controls | `IP-journey-j98-au-privacy-apra-cps234.md` | Differentiator |
| 089 | Data protection impact assessment | Security/compliance | Security/compliance | Compliance | `dpia.md` exists | Covered design |
| 090 | Threat model | Security docs | Security docs | IAM threat modeling | `threat-model.md` exists | Covered design |
| 091 | Failure modes | Reliability docs | Reliability docs | Well-Architected style | `failure-modes.md` exists | Covered design |
| 092 | Incident response plan | Incident docs | Native | OpsCenter/Incident Manager | `incident-response.md` exists | Covered design |
| 093 | Capacity model | Usage planning | Analytics | Service quotas | `capacity-model.md` exists | Covered design |
| 094 | Performance benchmark | Public docs | Public docs | Quotas | Existing benchmark stale; new report created this batch | Partial |
| 095 | Cost model | Usage attribution | Pricing | Service pricing/quotas | `cost-budget.md` exists | Partial |
| 096 | Migration from PagerDuty | Integrations | Native | Integrations | Migration playbook exists | Covered design |
| 097 | Migration from ServiceNow ITSM | Integrations | Integrations | OpsCenter integrations | Migration playbook exists | Covered design |
| 098 | Migration from incident.io | Integrations | Competitor | Integrations | Migration playbook exists | Covered design |
| 099 | Onboarding first week | Learning center | Onboarding docs | Console docs | `onboarding/sre-on-call-first-week.md` exists | Covered design |
| 100 | SRE FAQ | Docs | Docs | Docs | `faqs/sre-on-call-faq.md` exists but contains retired tier terms | Partial |
| 101 | Tutorial | Walkthrough | Walkthrough | Tutorials | Tutorial exists but contains retired tier terms | Partial |
| 102 | Rust SDK reference | API clients | API clients | SDKs/CLI | Reference implementation doc exists | Partial |
| 103 | SDK plan | API clients | API clients | SDKs | `sdk-plan.md` exists | Partial |
| 104 | Security policy-as-code | Not native | Permissions | IAM policies | Cedar files exist | Strong differentiator |
| 105 | Tenant-scope policy | Multi-org controls | Account scoping | IAM/resource scoping | `policy/cedar/tenant-scope-enforcement.cedar` | Covered design |
| 106 | Audit-emission policy | Audit trail | Audit logs | CloudTrail | `policy/cedar/audit-emission-required.cedar` | Covered design |
| 107 | Break-glass handling | Incident roles | Emergency overrides | IAM emergency access | `policy/cedar/emergency-services-bypass.cedar` | Partial |
| 108 | Abuse defense | Security detection | Noise/rate controls | Guardrails | `policy/cedar/abuse-defence.cedar` | Partial |
| 109 | Admin rollback runbook | Workflow automation | Runbook automation | Automation | `runbooks/admin-action-rollback.md` | Covered design |
| 110 | MFA cascade runbook | Auth/security | User notification | IAM/MFA | `runbooks/admin-mfa-cascade.md` | Covered design |
| 111 | Dashboard degradation runbook | Observability | Ops console | CloudWatch dashboards | `runbooks/dashboard-perf-degradation.md` | Covered design |
| 112 | Forensic handoff runbook | Security/audit | Post-incident | CloudTrail/logs | `runbooks/forensic-investigation-handoff.md` | Covered design |
| 113 | Pack quarantine runbook | Compliance/security | Not primary | Not primary | `runbooks/pack-author-quarantine.md` | Differentiator |
| 114 | Tenant violation runbook | Multi-org/security | Account/security | Account/IAM | `runbooks/tenant-scope-violation-detected.md` | Covered design |
| 115 | Deployment context support | Sites/integrations | Global accounts | AWS accounts/Regions | No context modules | Gap |
| 116 | OpenTofu deployability | Not primary | Terraform provider support | CloudFormation/SSM adjacent | Manifest says OpenTofu; no modules | Gap |
| 117 | OS support | Agent/platform docs | Browser/mobile docs | SSM agent OS support | No `supported-oses.json` | Gap |
| 118 | OCI Always Free profile | OCI integration | Not primary | Not AWS | No `iac/oci-guest/always-free/` | Gap |
| 119 | Provider-agnostic cloud abstraction | Multi-cloud integrations | Integrations | AWS-native | Dependencies include `cloud-iac`; no context proof | Partial |
| 120 | Manual console bypass prevention | Audit trail | Permissions | IAM/SSM | PRD forbids bypass | Covered intent |

## Family Summary

Incident command family: ODCC has a coherent incident declaration, severity, remediation, SLO, and runbook design.
Incident command family: ODCC lacks mature schedule, contact-method, notification-delivery, stakeholder-broadcast, and related-incident semantics.
Incident command family: PagerDuty remains the strongest benchmark for response accountability and escalation.
Incident command family: Datadog remains a benchmark for incident analytics tied to telemetry.
Incident command family: AWS remains a benchmark for automation and resource-state handoff rather than responder management.
Deployment command family: ODCC has strong approval, rollback, idempotency, Cedar, and audit-chain intent.
Deployment command family: ODCC needs explicit automation concurrency, queue, freeze-window, and Systems Manager-like rate-control semantics.
Deployment command family: AWS Systems Manager is the key counterpart for controlled automation and delegated execution.
Deployment command family: PagerDuty contributes response workflow and escalation accountability around changes.
Deployment command family: Datadog contributes monitor/workflow-triggered change and incident correlation.
Cluster health family: ODCC has cluster health route, dashboards, capacity model, and failure-mode docs.
Cluster health family: ODCC must clarify whether it owns node inventory or only displays normalized `cell` and `observability` health.
Cluster health family: CloudWatch and Systems Manager set the strongest benchmark for resource and managed-node breadth.
Cluster health family: Datadog sets the strongest benchmark for telemetry correlation and infrastructure pivots.
Tenant posture family: ODCC has strong tenant posture design and Cedar policies.
Tenant posture family: ODCC lacks tenant-class semantics, usage-cap semantics, and billing overlay semantics.
Tenant posture family: Datadog multi-org, PagerDuty account/service scoping, and AWS account/resource scoping all imply that ODCC needs precise tenant filters.
Policy/audit/evidence family: ODCC is strongest here relative to counterparts.
Policy/audit/evidence family: signed evidence export, pack overlays, Cedar policy panes, and audit-chain posture are clear differentiators.
Policy/audit/evidence family: the gap is complete event contract coverage, not feature ambition.
Operational docs family: ODCC has strong runbook, onboarding, FAQ, DPIA, threat-model, and compliance breadth.
Operational docs family: ODCC needs a cleanup pass for retired tier vocabulary and scaffold-like repeated content.
Integration family: ODCC has manifest dependencies but lacks explicit handoff docs.
Integration family: ODCC must avoid duplicating `observability`, `identity`, `tenancy`, `cell`, `cloud-iac`, and `audit-chain`; it should bind to them through contracts.
Deployability family: ODCC is blocked on six-context OpenTofu, OS matrix, and OCI Always Free profile.
Runtime family: ODCC passes forbidden source extension scan but lacks local Rust crate implementation evidence.

## Headline Gap Analysis

Gap 01: ODCC is command/evidence rich but notification/accountability light.
Evidence: incident routes and runbooks exist, but contact methods and schedule contracts are absent.
Counterpart pressure: PagerDuty expects schedules, escalation policies, responder notification, and escalation timeouts.
Correction: add an on-call accountability handoff or declare the owning microservice for schedules and contact methods.

Gap 02: ODCC is telemetry-pivot aware but not a telemetry platform.
Evidence: `IP-015-observability-pivot.md` exists and dashboards exist, but no log search, trace query, or metric ingestion implementation exists.
Counterpart pressure: Datadog expects deep metrics, logs, traces, dashboards, monitors, and analytics.
Correction: document observability handoff contracts and define the exact pivot payloads ODCC consumes.

Gap 03: ODCC is safe-command oriented but lacks automation rate-control semantics.
Evidence: rollback and deployment routes exist, but no automation queue/concurrency/error threshold contract is visible.
Counterpart pressure: Systems Manager Automation exposes concurrency, queue, pending, and error threshold concepts.
Correction: add command queue, concurrency, and abort semantics to contracts and SLOs.

Gap 04: ODCC has evidence exports but event contract coverage is incomplete.
Evidence: manifest expects 11 audit-chain seal events; AsyncAPI publishes 6 channel families.
Counterpart pressure: all three counterparts expose auditable incident/action histories.
Correction: publish all seal-event channels or state which events are internal to audit-chain.

Gap 05: ODCC is tenant-posture aware but tenant-class blind.
Evidence: tenant posture schema exists; `tenant_class`, `demo_trial`, `paid`, and `revenue_share` terms are absent.
Counterpart pressure: Datadog usage attribution, PagerDuty account plan context, and AWS service quotas all expose account-level constraint context.
Correction: add tenant-class overlays to schemas, dashboards, runbooks, and performance targets.

Gap 06: ODCC has compliance depth but stale capability-model language.
Evidence: FAQ, tutorial, benchmark, and tenant_class retirement marker directory still contain retired commercial tier references.
Counterpart pressure: uniform quality bar means compliance and evidence integrity must not degrade by commercial package label.
Correction: rewrite the affected lines under Wave 15J with tenant-class and deployment-context overlays.

Gap 07: ODCC has service catalog files but no complete dependency handoff.
Evidence: manifest lists 16 dependencies; no `cross-microservice-handoffs.md` exists.
Counterpart pressure: Datadog service catalog, PagerDuty service directory, and AWS resource groups depend on explicit ownership metadata.
Correction: add dependency handoffs for identity, tenancy, observability, audit-chain, cell, cloud-iac, and finops-portal first.

Gap 08: ODCC has deployability intent but lacks context proof.
Evidence: manifest says OpenTofu engine; `iac/` has no OpenTofu context modules.
Counterpart pressure: AWS Systems Manager and CloudWatch are operationally real only because account/Region deployment is explicit.
Correction: add six context wrappers or service-local N/A rationales.

Gap 09: ODCC has runbook breadth but not full execution telemetry.
Evidence: runbooks exist, but command queue, automation outcomes, and external workflow status are not contractually modeled.
Counterpart pressure: PagerDuty workflows and Systems Manager automations expose state transitions and execution history.
Correction: add workflow execution state and evidence links to OpenAPI, proto, AsyncAPI, and dashboards.

Gap 10: ODCC has strong policy-as-code posture but limited UX implementation evidence.
Evidence: Cedar files exist; no frontend source path exists.
Counterpart pressure: all three counterparts are operator consoles with production-grade UX.
Correction: add frontend source or a referenced frontend package path under the allowed language policy.

## Additive Surface Recommended for ODCC

Additive 01: `OnCallAccountabilityProjection` with schedule id, escalation policy id, current responder, backup responder, contact-route class, and source microservice.
Additive 02: `IncidentStakeholderUpdate` with audience, status channel, message hash, localization pack, and audit-chain receipt.
Additive 03: `AlertToIncidentSource` with observability signal id, detector id, dedup key, suppression status, and incident linkage.
Additive 04: `AutomationExecutionState` with command id, queue state, concurrency bucket, error threshold, abort state, and evidence export id.
Additive 05: `RecoveryWorkflowEvent` AsyncAPI channels matching `OpsRecoveryWorkflowStarted` and `OpsRecoveryWorkflowCompleted`.
Additive 06: `OpsIncidentRemediationApproved` AsyncAPI channel to close the manifest/event gap.
Additive 07: `OpsPolicyDecisionReviewed` AsyncAPI channel to close the policy evidence gap.
Additive 08: `TenantClassOverlay` with `demo_trial`, `paid`, and `revenue_share` enum values.
Additive 09: `DemoTrialUsageCap` with cap id, current consumption, reset window, and OCI Always Free profile linkage.
Additive 10: `PaidContractualSlo` with SLO id, entitlement source, compliance-pack allowance, and BYOK allowance.
Additive 11: `RevenueShareSubstrateCost` with at-cost budget, customer gross-revenue basis, and margin guardrail.
Additive 12: `DeploymentContextReadiness` with six context ids, IaC module path, last plan evidence, last policy-pack result, and last smoke result.
Additive 13: `OpenTofuPlanEvidence` with module digest, provider lock digest, plan id, policy result, cost event, and signer.
Additive 14: `SupportedOsEvidence` with OS id, architecture, lane class, build result, smoke result, and exception.
Additive 15: `NodeHealthProjection` normalized from `cell`, `observability`, and provider-specific adapters.
Additive 16: `ServiceDependencyProjection` with upstream service, contract id, freshness SLO, fallback behavior, and owner.
Additive 17: `CommandRateLimitState` with tenant bucket, operator bucket, emergency bucket, and retry-after.
Additive 18: `EvidencePackSigningPosture` with signer class, key custody, HSM or equivalent control, and notarization status.
Additive 19: `CompliancePackContext` with active packs, pack conflicts, residency boundary, and evidence requirements.
Additive 20: `OperatorUxSourceRef` pointing to the allowed frontend package or Leptos/WASM SSR web package.

## Coverage Verdict

ODCC already has a strong conceptual core.
Its strongest areas are incident command, deployment approval, rollback records, tenant posture, Cedar policy gates, audit-chain intent, and signed evidence export.
Its strongest differentiators against all three counterparts are compliance-pack overlays, Cedar policy evidence, signed evidence-pack export, and regulated-journey dashboards.
Its weakest counterpart area is PagerDuty-style responder accountability.
Its weakest Datadog area is deep telemetry exploration and event correlation.
Its weakest AWS area is deployable automation semantics and cloud/context proof.
The priority correction is not adding more loose docs.
The priority correction is binding existing docs to explicit contracts, context modules, event channels, tenant-class fields, and implementation evidence.
The current service is design-rich.
The current service is not yet union-complete against Datadog, PagerDuty, and AWS CloudWatch plus Systems Manager.
The current service should be allowed to retain its distinct Oyatie control-center identity.
The current service should not claim parity until notification/accountability, telemetry handoff, automation execution, tenant-class, and deployability gaps are resolved.

## Acceptance Criteria For A Future Parity Claim

Acceptance 01: every PagerDuty-like responder accountability feature has an ODCC owner, dependency owner, or explicit non-goal.
Acceptance 02: every Datadog-like telemetry pivot has a payload contract with `observability` instead of duplicating the telemetry platform.
Acceptance 03: every AWS-like automation control has queue, concurrency, error-threshold, and audit evidence semantics where ODCC owns the command path.
Acceptance 04: every tenant posture view includes tenant class without using it to lower safety or evidence quality.
Acceptance 05: every deployment context has OpenTofu readiness evidence or a service-local exclusion rationale.
Acceptance 06: every event in the manifest audit-chain list is covered by AsyncAPI or marked internal with a reason.
Acceptance 07: every dashboard family has a source of truth, freshness target, and failure-mode path.
Acceptance 08: every migration playbook names what ODCC imports, what it delegates, and what it deliberately refuses to own.
Acceptance 09: every compliance journey has a control-center surface, an evidence export path, and an audit-chain receipt.
Acceptance 10: every benchmark row in the performance report can be reproduced by a Rust test, load test, or context admission test.
Acceptance 11: every notification feature either has a delivery contract or a named dependency outside ODCC.
Acceptance 12: every policy review flow includes Cedar decision evidence and operator identity evidence.
Acceptance 13: every rollback and deployment command includes idempotency, step-up, audit-chain, and rollback evidence.
Acceptance 14: every service dependency in the manifest is explained in `cross-microservice-handoffs.md`.
Acceptance 15: parity is not claimed from breadth of Markdown; it is claimed from contract coverage, implementation evidence, and passing verification.
