# audit-chain feature parity matrix — 2026-05-20

## Header anchor block
1. Canonical sequence anchor: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2125` and `:3900-4235` require all six deployment contexts, OpenTofu-only IaC, OS matrix, Rust-strict implementation, OCI Always Free, and current counterpart discipline.
2. Master-plan machine anchor: `specs/master-plan-sequencing.json:704-868` defines `deployment_contexts`, `iac_substrate`, `supported_oses`, `language_policy`, and `oci_always_free`.
3. Service PRD anchor: `microservices/audit-chain/PRD.md:18-361` defines the audit-chain purpose, requirements, performance targets, capacity posture, and existing competitor claims.
4. Service architecture anchor: `microservices/audit-chain/ARCHITECTURE.md:1-754` defines the intended layers, policies, runtime shape, transport, observability, and credential isolation.
5. Counterpart source anchor: AWS CloudTrail docs (`https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-user-guide.html`, `https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-concepts.html`, `https://docs.aws.amazon.com/awscloudtrail/latest/userguide/eventreference.html`, `https://docs.aws.amazon.com/awscloudtrail/latest/userguide/WhatIsCloudTrail-Limits.html`), Google Cloud Audit Logs and Logging docs (`https://docs.cloud.google.com/logging/docs/audit`, `https://cloud.google.com/logging/docs/routing/overview`, `https://cloud.google.com/logging/quotas`, `https://cloud.google.com/logging/docs/logs-views`), and Microsoft Purview Audit docs (`https://learn.microsoft.com/en-us/purview/audit-solutions-overview`, `https://learn.microsoft.com/en-us/purview/audit-search`, `https://learn.microsoft.com/en-us/office/office-365-management-api/office-365-management-activity-api-reference`, `https://learn.microsoft.com/en-us/office/office-365-management-api/troubleshooting-the-office-365-management-activity-api`, `https://learn.microsoft.com/en-us/office/office-365-management-api/office-365-management-activity-api-schema`).

## Scope and method
This matrix uses the top-3 counterpart set assigned in chat history: AWS CloudTrail, Google Cloud Audit Logs, and Microsoft Purview Audit (`8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:15698`).
The matrix compares union coverage: if any counterpart has a capability, audit-chain must either provide it, deliberately delegate it to another Oyatie µservice, or record a gap for Wave 14.
The matrix does not treat Splunk or Datadog as primary counterparts because the current assignment supersedes older service-local matrices (`PRD.md:243-260`, `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:128-137`).
The matrix treats audit-chain's cryptographic Merkle proof surface as additive because none of the three counterparts expose an equivalent tenant-visible signed-proof API in the cited public docs.

## §1 AWS CloudTrail capability surface
AWS-01: CloudTrail records actions by users, roles, AWS services, console, CLI, SDKs, and APIs.
AWS-02: Event History provides a searchable and downloadable record of the past 90 days of management events.
AWS-03: Event History is automatically available when an AWS account is created.
AWS-04: Trails deliver and store events in Amazon S3.
AWS-05: Trails can deliver events to CloudWatch Logs.
AWS-06: Trails can deliver events to Amazon EventBridge.
AWS-07: CloudTrail supports management events.
AWS-08: CloudTrail supports data events.
AWS-09: CloudTrail supports network activity events.
AWS-10: CloudTrail supports Insights events.
AWS-11: Data events are not logged by default.
AWS-12: Network activity events are not logged by default.
AWS-13: Insights events are not logged by default.
AWS-14: CloudTrail Lake stores events in immutable event data stores.
AWS-15: CloudTrail Lake converts events into Apache ORC for query efficiency.
AWS-16: CloudTrail Lake supports SQL-based querying.
AWS-17: CloudTrail Lake can retain events up to 3,653 days under the one-year extendable retention option.
AWS-18: CloudTrail Lake can retain events up to 2,557 days under the seven-year retention option.
AWS-19: CloudTrail Lake can ingest existing CloudTrail logs from S3.
AWS-20: CloudTrail Lake can store organization events through AWS Organizations.
AWS-21: CloudTrail Lake can store events from multiple Regions and accounts.
AWS-22: CloudTrail Lake dashboards visualize top event trends.
AWS-23: CloudTrail Lake supports saved queries and query-result storage in S3.
AWS-24: Organization trails provide uniform event logging across member accounts.
AWS-25: Organization event data stores collect events for all accounts in an organization.
AWS-26: Delegated administrators can manage selected organization event data store operations.
AWS-27: Advanced event selectors control which events are captured.
AWS-28: A trail or event data store can include management, data, network activity, and Insights selectors.
AWS-29: CloudTrail Insights establishes baselines for API call rates.
AWS-30: CloudTrail Insights establishes baselines for API error rates.
AWS-31: CloudTrail Insights generates events when activity deviates from baseline.
AWS-32: CloudTrail supports log file integrity validation for delivered trail files.
AWS-33: CloudTrail can integrate with AWS Audit Manager evidence.
AWS-34: CloudTrail Lake supports integrations and channels for external event sources.
AWS-35: CloudTrail exposes APIs and SDKs for automation.
AWS-36: CloudTrail supports global service event handling.
AWS-37: CloudTrail supports multi-Region trails.
AWS-38: CloudTrail supports single-Region trails.
AWS-39: CloudTrail supports KMS/S3 encryption patterns through destination configuration.
AWS-40: CloudTrail supports tagging for trail and store resources.
AWS-41: CloudTrail quotas include 5 trails per Region.
AWS-42: CloudTrail quotas include 10 event data stores per Region.
AWS-43: CloudTrail event size is capped at 256 KiB for CloudWatch/EventBridge delivery.
AWS-44: CloudTrail S3 log files are capped at 50 MB before compression.
AWS-45: CloudTrail `LookupEvents` TPS is 2.
AWS-46: CloudTrail Lake can ingest non-AWS auditable data sources through integrations.
AWS-47: CloudTrail records who made a request, which service was used, action performed, and request parameters.
AWS-48: CloudTrail does not promise ordered stack traces; consumers must handle unordered records.
AWS-49: CloudTrail console supports viewing, filtering, downloading, creating trails, and querying event data stores.
AWS-50: CloudTrail pricing distinguishes management/data/network event ingestion and other auditable source ingestion.

## §2 Google Cloud Audit Logs capability surface
GCP-01: Cloud Audit Logs provides audit logs for projects, folders, and organizations.
GCP-02: Admin Activity audit logs record user-driven API calls and actions that modify resource configuration or metadata.
GCP-03: Admin Activity audit logs are always written and cannot be disabled.
GCP-04: Data Access audit logs record reads of metadata/configuration and reads/writes of user-provided resource data.
GCP-05: Data Access audit logs are disabled by default for most services except BigQuery.
GCP-06: System Event audit logs record Google-system actions that modify resources.
GCP-07: System Event audit logs are always written and cannot be disabled.
GCP-08: Policy Denied audit logs record access denials caused by security policy violations.
GCP-09: Policy Denied audit logs are generated by default.
GCP-10: Audit log entries are immutable.
GCP-11: Audit logs use `LogEntry` and `AuditLog` objects.
GCP-12: Audit logs include resource ownership at project, folder, or organization level.
GCP-13: IAM roles gate read access to Admin Activity, System Event, Policy Denied, and Data Access logs.
GCP-14: Private Logs Viewer is required for Data Access logs in the default bucket.
GCP-15: Cloud Logging routes entries through Log Router sinks.
GCP-16: Log sinks can route entries to log buckets.
GCP-17: Log sinks can route entries to a different Google Cloud project.
GCP-18: Log sinks can route entries to BigQuery datasets.
GCP-19: Log sinks can route entries to Cloud Storage buckets.
GCP-20: Log sinks can route entries to Pub/Sub topics for downstream integrations.
GCP-21: Multiple sinks can route the same log entry.
GCP-22: System-created `_Required` and `_Default` sinks exist.
GCP-23: `_Required` routes selected audit logs to the `_Required` bucket.
GCP-24: `_Default` routes broad logs to the `_Default` bucket.
GCP-25: Cloud Logging buckets store logs for Logs Explorer and Log Analytics.
GCP-26: Log buckets support linked BigQuery datasets.
GCP-27: Log buckets support customer-managed encryption keys.
GCP-28: Log views restrict access to subsets of logs in a bucket.
GCP-29: A log bucket can have up to 30 log views.
GCP-30: Log views use IAM policies.
GCP-31: Cloud Logging supports Logs Explorer.
GCP-32: Cloud Logging supports Log Analytics.
GCP-33: Logging API write requests are limited to 10 MB.
GCP-34: Regional `entries.write` ingestion can be 4.8 GB/minute per project in listed regions.
GCP-35: Live tailing supports up to 10 open sessions per project.
GCP-36: Live tailing returns up to 60,000 entries per minute.
GCP-37: Audit log max entry size is 512 KiB for Admin Activity.
GCP-38: Audit log max entry size is 512 KiB for Data Access.
GCP-39: Audit log max entry size is 512 KiB for System Event.
GCP-40: Audit log max entry size is 512 KiB for Policy Denied.
GCP-41: `_Required` bucket retention defaults to 400 days.
GCP-42: `_Default` bucket retention defaults to 30 days.
GCP-43: User-defined bucket retention defaults to 30 days.
GCP-44: Default/user-defined buckets can retain logs from 1 to 3,650 days.
GCP-45: Log Router temporary storage buffers disruptions but not configuration errors.
GCP-46: Log entries too far in the past or future can be discarded by routing rules.
GCP-47: Publicly shared resources do not generate Data Access logs for unauthenticated/public access.
GCP-48: Data Access enablement uses IAM AuditConfig at project, folder, billing account, or organization scope.
GCP-49: Workspace audit logs can be integrated into Cloud Logging.
GCP-50: Access Transparency logs are routed through required audit-log storage paths.

## §3 Microsoft Purview Audit capability surface
MS-01: Microsoft Purview Audit Standard logs and searches audited activities for forensic, IT, compliance, and legal investigations.
MS-02: Audit Standard is enabled by default for organizations with appropriate subscription.
MS-03: Audit Standard captures searchable records for audited activities.
MS-04: Audit Standard includes thousands of searchable audit events.
MS-05: Audit Standard includes an Audit search tool in the Purview portal.
MS-06: Audit search supports critical audit log event search for user activities.
MS-07: Audit Premium builds on Audit Standard.
MS-08: Audit Premium provides audit log retention policies.
MS-09: Audit Premium provides longer retention of audit records.
MS-10: Audit Premium provides high-value intelligent insights.
MS-11: Audit Premium provides higher bandwidth access to the Office 365 Management Activity API.
MS-12: Audit Premium retains Exchange audit records for one year by default.
MS-13: Audit Premium retains SharePoint audit records for one year by default.
MS-14: Audit Premium retains OneDrive audit records for one year by default.
MS-15: Audit Premium retains Microsoft Entra audit records for one year by default.
MS-16: Other activities default to 180 days unless retention policies extend them.
MS-17: Ten-year retention is available with additional per-user add-on licensing.
MS-18: Custom audit retention policies can target service, activity, or user.
MS-19: High-value events include mail item access.
MS-20: High-value events include reply and forward activity.
MS-21: High-value events include Exchange Online searches.
MS-22: High-value events include SharePoint Online searches.
MS-23: Unified audit logging must be enabled for Management Activity API access.
MS-24: Management Activity API retrieves user, admin, system, and policy actions.
MS-25: Management Activity API covers Office 365 and Microsoft Entra activity logs.
MS-26: Management Activity API content types include Audit.AzureActiveDirectory.
MS-27: Management Activity API content types include Audit.Exchange.
MS-28: Management Activity API content types include Audit.SharePoint.
MS-29: Management Activity API content types include Audit.General.
MS-30: Management Activity API content types include DLP.All.
MS-31: Management Activity API creates subscriptions to content types.
MS-32: Management Activity API can poll for content blobs.
MS-33: Management Activity API can use webhook notifications.
MS-34: First content blobs can take up to 12 hours after subscription creation.
MS-35: Audit event availability for core services is typically 60 to 90 minutes.
MS-36: Content blobs remain available for API fetch for 7 days after notification.
MS-37: API baseline throttle is 2,000 requests per minute.
MS-38: Valid PublisherIdentifier can place clients in a 60,000 requests per minute tenant pool.
MS-39: UI export removes duplicates, while Management Activity API does not perform the same de-duplication.
MS-40: Management Activity API does not query by individual event specifics; clients download and index blobs.
MS-41: Audit schema has common and service-specific layers.
MS-42: Common schema includes record type, creation time, user type, action, user ID, client IP, and object ID concepts.
MS-43: Service-specific schema captures workload-specific properties.
MS-44: Audit log activities are documented across Microsoft 365 services.
MS-45: Purview role/permission assignment controls access to audit search.
MS-46: Audit Premium licensing controls premium event generation for users.
MS-47: New Search adds faster search, additional search options, and saved searches compared with classic search.
MS-48: Audit records support export and downstream forensic workflows.
MS-49: Audit retention policies can override default retention.
MS-50: Audit solutions are positioned for breach scope determination and compliance investigations.

## §4 UNION-coverage matrix
| Capability | AWS | Google | Microsoft | UNION required | Oyatie audit-chain | Gap classification |
|---|---|---|---|---|---|---|
| Account or tenant API activity logging | yes | yes | yes | yes | yes via `AuditEvent.source_microservice` | present |
| User/admin/system/policy activity taxonomy | partial | yes | yes | yes | partial via event registry | gap: taxonomy breadth |
| Management/admin event class | yes | yes | yes | yes | yes | present |
| Data access event class | yes | yes | yes | yes | partial via payload data classes | gap: explicit per-service read/write taxonomy |
| Network or private endpoint activity events | yes | partial | no | yes | absent as first-class family | gap |
| Policy denied events | no | yes | yes | yes | partial via Cedar denials | gap: needs standardized denial family |
| Insights or anomaly events | yes | no | yes | yes | partial via verification-failed alerts | gap |
| Immutable event storage | yes | yes | yes | yes | yes plus WORM/Merkle | ahead |
| Searchable event history | yes | yes | yes | yes | yes via query API | present |
| 90-day free/recent event search | yes | no | maybe standard retention | yes | no explicit free tier policy | gap |
| Long-term retention to 7 years | yes | yes configurable | yes premium | yes | yes paid tenant_class/paid tenant_class | present |
| Long-term retention to 10 years | yes | yes configurable | yes add-on | yes | yes paid tenant_class/pack retention | present |
| Organization-wide aggregation | yes | yes | yes tenant-wide | yes | partial via tenant/pack | gap: org hierarchy semantics |
| Multi-region aggregation | yes | yes via routing | tenant services | yes | partial via multi-region doc | gap: context module evidence |
| Event data store abstraction | yes | log bucket | audit blob/search | yes | yes via Postgres/S3/SeaweedFS | present |
| SQL analytics | yes | yes Log Analytics/BigQuery | client-side/API search | yes | no SQL surface exposed | gap |
| Saved queries | yes | yes | yes new search | yes | not evidenced | gap |
| Dashboards | yes Lake dashboards | yes Cloud Logging dashboards | Purview search/reporting | yes | Grafana dashboards exist | partial |
| S3/Cloud Storage export | yes | yes | API/export | yes | export bundle | present |
| Pub/Sub or event streaming export | EventBridge | Pub/Sub | webhook/content API | yes | AsyncAPI events | present |
| External source ingestion | yes Lake channels | sinks/API | Management API intake only for Microsoft data | yes | REST emit accepts all services | present |
| Partner integrations | yes | Pub/Sub/SIEM | API/ISV | yes | not productized | gap |
| Delegated administration | yes | IAM | Purview roles | yes | Cedar policies partial | gap |
| Role-based access control | yes IAM | IAM | Purview roles | yes | Cedar/SPIFFE/OIDC | present |
| Private log viewer role equivalent | no | yes | role permissions | yes | auditor Cedar roles | present |
| Data access logs disabled by default toggle | yes optional | yes optional | license/policy controlled | yes | no service-level toggle matrix | gap |
| Default always-on admin logs | management history | Admin Activity | Standard audit | yes | emit required by app contracts | partial |
| Log views | no | yes | saved searches/roles | yes | no equivalent | gap |
| Customer-managed encryption | yes destination KMS | yes CMEK | Microsoft service encryption/customer key adjacent | yes | yes through HSM/key docs | partial |
| Log file integrity validation | yes | immutable entries | no direct | yes | Merkle/Ed25519 stronger | ahead |
| Tenant-visible proof API | no | no | no | no | yes | additive |
| Signed root publication | no direct | no direct | no direct | no | yes | additive |
| Public key endpoint | no direct | no direct | no direct | no | yes | additive |
| Per-tenant signing key | no | no | no | no | paid tenant_class yes | additive |
| HSM-resident signing | destination/KMS | CMEK/KMS | service managed | yes | paid tenant_class yes | ahead/partial |
| Audit event size limit | yes 256 KiB | yes 512 KiB | schema/blob based | yes | OpenAPI lacks explicit max | gap |
| Log file delivery size limit | yes 50 MB before compression | request limits | blob model | yes | no explicit export chunk max | gap |
| API TPS documented | LookupEvents 2 TPS | entries.write quotas | API requests per min | yes | not documented | gap |
| Event delivery latency documented | near real-time/5 min docs | ingestion/routing docs | 60-90 min core | yes | p99 targets exist for sealing | partial |
| Baseline anomaly detection | yes Insights | Monitoring/log analytics | Premium insights | yes | not equivalent | gap |
| Retention legal hold policy | S3 Object Lock adjacent | bucket retention | retention policies | yes | WORM retention docs | present |
| Compliance export bundles | Audit Manager/S3 | Cloud Storage/BigQuery | Purview export/API | yes | yes | present |
| DLP audit content type | no | no | yes DLP.All | yes | partial via data class | gap |
| Mailbox item access events | no | no | yes | yes | absent domain-specific source | delegated gap |
| SharePoint/Drive file access events | S3 data events | Workspace/Cloud data access | yes | yes | generic event class only | delegated gap |
| Entra/IAM directory activity | AWS IAM events | IAM admin logs | Entra logs | yes | generic identity emit | delegated gap |
| Organization/folder/project scope | account/org | project/folder/org | tenant | yes | tenant/pack/cell | partial |
| Multi-account delegated admin | yes | IAM folder/org roles | tenant roles | yes | not explicit | gap |
| Log bucket location controls | S3 bucket region | log bucket region | tenant/service region | yes | pack data residency | present |
| Cross-region route/sink | trails/stores | sinks | API/export | yes | multi-region sidecar | partial |
| On-prem/hybrid ingestion | Lake integrations | Ops Agent/custom logs | API/custom integrations | yes | REST emit | present |
| UI audit portal | console | Logs Explorer | Purview portal | yes | no service-local UI | delegated gap |
| API automation | yes | yes | yes | yes | yes | present |
| SDK automation | yes | yes | yes | yes | Rust reference only | partial |
| Schema evolution policy | JSON event versions | LogEntry/AuditLog | common/service schema | yes | OpenAPI/proto versions | partial |
| Duplicate handling | not primary | logging semantics | API no de-dup; UI de-dups | yes | idempotency key | present |
| Ordered event guarantees | no | no strict | no strict | yes to specify | partial via period IDs | gap |
| Export availability window | query results 7 days | bucket retention | API blobs 7 days | yes | export status no TTL | gap |
| High-value user audit | no direct | no direct | yes | yes | no named high-value actor class | gap |
| Activity-specific retention policies | event data store selectors | bucket retention/filter | retention policies | yes | pack retention | partial |
| Access Transparency/support access logs | no | yes | Customer Lockbox adjacent, not Audit core | yes | absent | gap |
| Query federation | yes Lake federation | BigQuery linked datasets | API/client indexing | yes | absent | gap |
| Cost/usage attribution | yes pricing and events | quotas/pricing | licensing | yes | cost-budget OCI only | gap |
| Always Free profile | no | free quotas not audit-specific | no | required by Oyatie doctrine | absent | Oyatie-specific gap |
| Six-context deployment | no | no | no | required by Oyatie doctrine | absent | Oyatie-specific gap |
| OpenTofu per-context provisioning | no | no | no | required by Oyatie doctrine | absent | Oyatie-specific gap |
| OS support manifest | no | no | no | required by Oyatie doctrine | absent | Oyatie-specific gap |
| Rust-only backend | no | no | no | required by Oyatie doctrine | pass | present |
| Merkle proof verification | no tenant-visible | no tenant-visible | no tenant-visible | additive | yes | additive |
| Ed25519 signed roots | no public API | no public API | no public API | additive | yes | additive |
| Chain replay runbook | partial | partial | partial | yes | yes runbooks | present |
| HSM key rotation runbook | partial | KMS docs | service managed | yes | yes | present |
| Verification failure incident path | partial | partial | partial | yes | yes | present |
| PII payload redaction guard | partial | partial | DLP events | yes | partial | gap |
| Data residency pack overlays | region controls | region buckets | tenant geography | yes | yes packs | partial |
| Sovereign pack support | GovCloud/etc | Assured Workloads adjacent | sovereign clouds adjacent | yes | paid tenant_class docs | partial |
| Regulatory pack roster | Audit Manager adjacent | compliance docs | Purview compliance | yes | yes | present |
| Public auditor verification | no | no | no | additive | yes | additive |
| Self-audit of audit queries | partial | audit search logs | yes likely | yes | yes in handoff matrix | present |
| Self-audit of exports | partial | partial | yes likely | yes | yes | present |
| Key-epoch overlap semantics | no direct | no direct | no direct | additive | yes | additive |
| Cryptographic root cross-channel validation | log integrity partial | no | no | additive | yes | additive |
| Tenant-scoped Cedar authorization | no Cedar | IAM | Purview roles | yes | yes | present |
| SPIFFE internal caller auth | no | no | no | Oyatie required | yes | present |
| Export bundle signature | partial | no | no | yes | yes | present |
| Auditor FAQ/onboarding | docs | docs | docs | yes | yes | present |
| Migration from Splunk/legacy audit | no | no | no | useful | yes | additive |
| Cloud provider activity coverage breadth | AWS-only | GCP-only | Microsoft 365-only | union | generic but not enumerated | gap |
| Microsoft 365 workload record types | no | no | yes | union | absent | gap |
| Google Cloud Policy Denied exact shape | no | yes | partial | union | absent | gap |
| AWS network activity exact shape | yes | no | no | union | absent | gap |
| CloudTrail Lake ORC-style optimized store | yes | no | no | union | no | gap |
| BigQuery linked dataset | no | yes | no | union | no | delegated gap |
| Purview high-value insights | no | no | yes | union | no | gap |
| API content blob polling/webhook | no | Pub/Sub | yes | union | AsyncAPI yes, blob polling no | partial |
| UI saved search | Lake saved query | Logs Explorer query | New Search saved searches | union | no UI | delegated gap |
| Query result retention | yes 7 days | bucket-based | blob 7 days | union | not specified | gap |
| Maximum log views per bucket | no | yes 30 | no | union | no | gap |
| Baseline request throttle | yes | yes | yes | union | not documented | gap |

## §5 Capability families summary table
| Family | Union required count | Oyatie present | Partial | Missing | Notes |
|---|---:|---:|---:|---:|---|
| Collection and event taxonomy | 14 | 5 | 5 | 4 | Generic emit exists, but provider/workload-specific taxonomies lag |
| Storage, retention, and integrity | 18 | 11 | 5 | 2 | Merkle proof is strong; query TTL and size limits missing |
| Query, search, analytics, and UI | 17 | 3 | 3 | 11 | Query API exists; SQL analytics, saved search, UI are absent or delegated |
| Routing, export, and integrations | 14 | 7 | 4 | 3 | AsyncAPI/export exist; partner and blob models are incomplete |
| Access control and delegation | 11 | 5 | 4 | 2 | Cedar is strong; admin delegation model needs parity |
| Operations, quotas, and performance | 12 | 2 | 3 | 7 | SLOs exist; published quotas and limits are incomplete |
| Compliance and investigation | 15 | 8 | 5 | 2 | Packs/runbooks strong; Purview high-value events missing |
| Deployment and portability | 9 | 1 | 0 | 8 | Rust passes; context/OpenTofu/OS/OCI demo_trial tenant_class gaps remain |
| Cryptographic additive surface | 8 | 8 | 0 | 0 | Oyatie ahead of counterparts on tenant-visible proof |

## §6 Headline gap analysis
Gap 01: Six-context deployment parity is absent; implement `deployment_contexts` in `manifest.json` and OpenTofu modules for all six contexts, citing ADR-0328 D-15.
Gap 02: OpenTofu parity is absent; replace Terraform references in `IP-001-storage-backend-iac.md:38`, `threat-model.md:466`, and `iac/helm/audit-storage/values.yaml:4`.
Gap 03: OS support manifest is absent; add `supported-oses.json` with the 13 Tier-1 rows, 2 Tier-2 test-only rows, exclusions, packages, and CI lanes.
Gap 04: OCI demo_trial Always Free is absent; add `iac/oci-guest/always-free/` and reconcile demo_trial tenant_class capacity with OCI limits.
Gap 05: Counterpart taxonomy breadth is missing; define CloudTrail-style management/data/network/Insights, Google Admin/Data/System/PolicyDenied, and Purview workload record families.
Gap 06: Search analytics parity is missing; add query jobs, saved searches, SQL/columnar analytics or deliberate delegation to observability/analytics.
Gap 07: Admin portal parity is missing; decide whether audit-chain owns a minimal auditor UI or delegates to compliance/ops-dashboard-control-center.
Gap 08: Delegated administration parity is incomplete; map organization/folder/tenant delegated admin semantics into Cedar and tenancy.
Gap 09: Published quotas and limits are incomplete; define event max size, export chunk max, query TPS, emit TPS, and API throttle behavior.
Gap 10: Delivery and availability windows are incomplete; define equivalent of CloudTrail 90-day history, Purview 7-day API blob fetch, and query-result TTL.
Gap 11: Google log views are not represented; implement audit-chain filtered views or delegate them to query-stack with Cedar-visible view definitions.
Gap 12: Purview high-value user/event audit is not represented; model high-value principals and priority retention in policy and retention matrix.
Gap 13: CloudTrail Insights parity is incomplete; add anomaly baseline event classes for API call rate, error rate, seal rate, and verification failures.
Gap 14: Access Transparency support-access parity is missing; define support/operator access events and require sealing on privileged support reads.
Gap 15: Partner/external integration packaging is incomplete; add ingestion channel onboarding and generated SDK provenance for external emitters.

## §7 Additive surface
Additive 01: Tenant-visible Merkle proof verification API is stronger than public counterpart surfaces (`contracts/openapi/audit-chain.yaml:106-140`).
Additive 02: Ed25519 signed roots give cryptographic evidence beyond normal audit-log immutability (`contracts/openapi/audit-chain.yaml:114-125`).
Additive 03: Public key endpoint gives verifier independence (`contracts/proto/audit-chain.proto:27-28`, `:121-128`).
Additive 04: Per-tenant signing keys at paid tenant_class provide tenant-specific custody (`ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:100-103`).
Additive 05: Root cross-channel divergence is explicitly a Sev-1 failure (`failure-modes.md:74-85`).
Additive 06: Genesis mismatch is explicitly a Sev-1 chain-integrity condition (`failure-modes.md:87-98`).
Additive 07: Retention applications are themselves audit events, preserving deletion accountability (`failure-modes.md:191-202`).
Additive 08: Cedar-scoped public verification separates proof verification from privileged query (`cross-microservice-handoffs.md:36-38`, `:151-164`).
Additive 09: Compliance packs are first-class documents inside the service path (`packs/*.md` inventory).
Additive 10: Runbook suite is service-local and broad, with nine runbooks at 281 lines each.
Additive 11: Journey-specific seal plans give product-event specificity across many user journeys.
Additive 12: Audit-chain self-audits query, export, proof read, and verification request events (`cross-microservice-handoffs.md:81-84`).
Additive 13: Fail-closed behavior for malformed or mismatched proofs is explicitly captured (`cross-microservice-handoffs.md:248-249`).
Additive 14: Event emission retries preserve event id and seal order (`cross-microservice-handoffs.md:227-233`).
Additive 15: The service has an explicit DSR retention cascade failure model (`failure-modes.md:178-189`).
Additive 16: The service treats audit-chain incidents as regulatory-notification triggers (`incident-response.md:150-218`).
Additive 17: The service separates chain availability from evidence integrity, which prevents hiding tamper under uptime metrics.
Additive 18: The service exposes export bundle signatures, not only raw download links (`contracts/openapi/audit-chain.yaml:192-201`).
Additive 19: The service provides an offline Rust reference implementation for emit-and-verify (`reference-implementations/emit-and-verify-rust-sdk.md`).
Additive 20: The service's additive surface is valuable only if Wave 14 remediates deployment, OS, OpenTofu, and counterpart breadth gaps before implementation agents build from it.

## §8 Parity Evidence Notes for Wave Aggregation
1. AWS event capture maps to audit-chain append requirements in `PRD.md:38-49`.
2. AWS Event history maps to audit-chain query and retention requirements in `PRD.md:38-49` and `PRD.md:77-91`.
3. AWS CloudTrail Lake maps to audit-chain query/export/retention surfaces, but query analytics parity is not yet proven.
4. AWS advanced selectors have no equivalent hard limit in audit-chain docs, creating a concrete parity gap.
5. AWS organization event data stores map to audit-chain tenant/cell architecture, but multi-tenant aggregation limits are not specified.
6. AWS external source channels map to audit-chain producer APIs and AsyncAPI contracts.
7. AWS log file integrity maps to audit-chain Merkle seal and HSM signing, where Oyatie's intended proof model is more explicit.
8. AWS CloudWatch/EventBridge delivery maps to audit-chain export and async event handoffs, but sink quotas are missing.
9. AWS retention up to 3,653 days maps to paid tenant_class long-retention targets, not current demo_trial tenant_class.
10. AWS LookupEvents maps to audit-chain query APIs, but audit-chain should exceed that low-throughput search path.
11. Google Admin Activity logs map to admin and control-plane event capture in audit-chain journeys.
12. Google Data Access logs map to data-access and evidence-access event classes, but audit-chain needs clearer data-access selector docs.
13. Google System Event logs map to system-generated audit events, including seal-worker and retention-cascade events.
14. Google Policy Denied logs map cleanly to Cedar deny evidence in audit-chain.
15. Google Log Router maps to audit-chain export routing, but the service lacks a sink-count matrix.
16. Google aggregated sinks map to tenant/org aggregation, but current docs do not give per-context module proof.
17. Google custom buckets map to hot/cold storage tiers, but bucket limits and retention controls are not fully published.
18. Google log views map to saved evidence views, but audit-chain docs do not set a count or access-model limit.
19. Google restricted fields map to privacy-preserving evidence access, which is not explicit enough in current docs.
20. Google 4.8 GB/minute major-region ingestion sets a high provider-scale bar that audit-chain target numbers have not measured.
21. Google 512 KiB audit entry max should become an audit-chain event-size rule or split-event policy.
22. Google 1-3650 day custom retention maps to paid tenant_class retention and legal-hold capabilities.
23. Google live tail maps to near-real-time investigator streams, which are not currently a first-class audit-chain feature.
24. Google query fanout maps to cell/bucket fanout and needs an audit-chain hard limit.
25. Google CMEK maps to customer-owned key and HSM partition surfaces, but context-specific key modules are missing.
26. Microsoft Audit Standard maps to demo_trial tenant_class baseline search and retention.
27. Microsoft 180-day default retention exceeds a simple 90-day hot-retention demo_trial tenant_class target unless demo_trial tenant_class exposes cold-searchable archive.
28. Microsoft Audit Premium one-year retention maps to paid tenant_class.
29. Microsoft 10-year add-on maps to paid tenant_class long-retention controls.
30. Microsoft 10 concurrent search jobs should become a published audit-chain search concurrency rule.
31. Microsoft one unfiltered search job should become an audit-chain guardrail for unbounded scans.
32. Microsoft 30-day completed search history should become a saved-search retention target or explicit non-goal.
33. Microsoft 2,000 requests/minute Activity API baseline should become a paid tenant_class egress target.
34. Microsoft content blobs by workload type map to audit-chain event families and export partitions.
35. Microsoft webhook validation should become a hard requirement for audit-chain export subscribers.
36. Microsoft Activity API duplicate behavior highlights the need for audit-chain idempotency and duplicate-event semantics.
37. Microsoft common plus service-specific schema maps to audit-chain common event envelope plus service-specific extension fields.
38. Microsoft unified auditing enablement maps to tenant onboarding and default audit-chain enablement requirements.
39. Microsoft high-value intelligent insights are not core audit-chain today; they may belong in analytics or compliance adjacency.
40. Microsoft eDiscovery and broader Purview compliance should be split from audit-chain unless evidence-log search directly depends on it.
41. Cross-counterpart event size is a hard gap because AWS and Google publish numeric limits and audit-chain does not.
42. Cross-counterpart request size is a hard gap because Google publishes `entries.write` size and audit-chain batch sizing is not explicit.
43. Cross-counterpart retention policy is partial because audit-chain has tenant_class targets but not full context and OCI Always Free reconciliation.
44. Cross-counterpart query/search concurrency is a gap because Microsoft and Google publish limits and audit-chain does not.
45. Cross-counterpart export/sink count is a gap because Google and AWS publish quota-like controls and audit-chain does not.
46. Cross-counterpart organization aggregation is partial because audit-chain has tenancy/cell language but lacks deployable context modules.
47. Cross-counterpart customer-managed keys are partial because audit-chain has HSM/key intent but no full context OpenTofu wiring.
48. Cross-counterpart dashboards/analytics are catch-up gaps because audit-chain focuses on proof and evidence, not managed analytics.
49. Cross-counterpart support/admin transparency is partial because audit-chain can seal admin actions but docs need a dedicated support-access event taxonomy.
50. Cross-counterpart API schema is partial parity because OpenAPI, AsyncAPI, and proto contracts exist.
51. Cross-counterpart generated clients are a documentation gap because `PRD.md:47` names TS/Python SDK bindings without generated-only provenance language.
52. Cross-counterpart proven deployment is a gap unique to Oyatie because counterparts are managed services and audit-chain must prove six-context deployment.
53. Cross-counterpart OS package matrix is a gap unique to Oyatie because counterparts hide OS while ADR-0328 requires explicit OS support.
54. Cross-counterpart source language is currently aligned because the service path has no forbidden non-Rust source files.
55. Cross-counterpart OpenTofu substrate is a gap because the service has Helm/Kustomize but no canonical context modules.
56. Cross-counterpart Terraform references are direct drift because service docs and Helm values still name Terraform paths.
57. Cross-counterpart OCI Always Free demo_trial is an Oyatie-specific requirement with no counterpart, so it must be reconciled internally.
58. Cross-counterpart Merkle proof is Oyatie-additive and should remain a differentiator rather than be weakened into ordinary log export.
59. Cross-counterpart chain-of-custody journey support is Oyatie-additive and supports regulated domain workflows.
60. Cross-counterpart tenant exit is only partial because export exists conceptually, but archive portability and seal continuity need implementation detail.
61. Cross-counterpart policy denied is promising because Cedar deny decisions can be sealed as first-class evidence.
62. Cross-counterpart field-level restriction is under-specified and should borrow the control idea from Google restricted fields without copying implementation.
63. Cross-counterpart saved investigations are under-specified and should be mapped to CloudTrail saved queries, Google views, and Purview search history.
64. Cross-counterpart long archive is stronger than vendors in target form if twenty-five-year archive is implemented.
65. Cross-counterpart measured benchmark evidence is weaker than vendors because only targets exist today.
66. Cross-counterpart incident response is reasonably documented in `incident-response.md:24-270`.
67. Cross-counterpart failure modes are reasonably documented in `failure-modes.md:30-283`.
68. Cross-counterpart cost budget is present, but HSM cost dominance means demo_trial/Always Free must be split.
69. Cross-counterpart capacity model is present, but lacks measured benchmark outputs.
70. Cross-counterpart state backend is missing for OpenTofu context modules.
71. Cross-counterpart module signing is missing against ADR-0039 and ADR-0328 expectations.
72. Cross-counterpart tenant onboarding is incomplete until `tofu init -> tofu plan -> tofu apply` is documented per context.
73. Cross-counterpart provider-neutral ports are architecturally described but not tested in context.
74. Cross-counterpart public-cloud parity is closest at paid tenant_class, not demo_trial tenant_class.
75. Cross-counterpart hyperscaler maturity remains partial until counterpart union coverage, OpenTofu, OS matrix, and benchmarks are all evidenced.
