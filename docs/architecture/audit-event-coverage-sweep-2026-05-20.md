# Audit-Event Coverage Sweep — 2026-05-20

Status: audit-only checkpoint
Agent: `codex-audit-event-coverage-sweep`
Source ADR: `docs/decisions/ADR-0706-observability-live-apex.md`
Output path: `docs/architecture/audit-event-coverage-sweep-2026-05-20.md`

## §1 — Methodology

1. ADR-0263 is treated as the active observability emission contract; §D-13 requires durable tenant-visible state changes to seal audit-chain, receive `audit_id`, and carry that ID through log/span/metric emissions.
2. Strict service scope follows the requested selector: `microservices/<name>/policies/*.cedar` plus contract surfaces under `microservices/<name>/contracts/`; the selector matched 15 services.
3. No strict service has `contracts/asyncapi.json`; AsyncAPI YAML files such as `asyncapi-v1.yaml` and `local-asyncapi-v1.yaml` were included because they are the repo-local publish-channel contracts.
4. REST mutations were enumerated from OpenAPI `POST`, `PUT`, `PATCH`, and `DELETE` operations.
5. gRPC mutations were enumerated from `.proto` RPC names after excluding obvious read/query prefixes; `CheckLocalPolicy` remains in scope because these policy-bearing services use it as the local Cedar decision surface.
6. AsyncAPI publish channels were enumerated from AsyncAPI 3 `operations.*.action == send` entries.
7. A named audit-event class requires a concrete name in `x-audit-event`, an AsyncAPI publish/message class, a payload `audit_event_class` const or one-value enum, or exact IP evidence. Generic `DomainEvent`, `SloBurnEvent`, and bare `audit_event_class: string` hooks are not counted as concrete classes.
8. Registration was checked against the ADR-0263 reverse-cross-referenced classes from ADR-0297, ADR-0313, and ADR-0319. The directory `microservices/audit-chain/policy/event-class-registry/` referenced by ADR-0263 is absent in this checkout, so ADR-0263 text is the registry anchor for this sweep.
9. Recommended names prefer already registered ADR-0263 classes when semantics match; otherwise they use deterministic `<Service><Resource><Outcome>` CamelCase names for a follow-up registration wave.
10. Coverage in §2 is current named-class coverage, not registered-class compliance. Registered compliance is separately shown and is currently zero across strict scope.
11. Cross-reference evidence was checked in service IP slices (`IP-*.md`, `ip/IP-*.md`, `specs/IP-*.md`) and service AsyncAPI publish classes.
12. No source contracts, policies, IP slices, ADRs, or code were modified; this sweep document is the only authored artifact.

### §1.1 Registered ADR-0263 classes used for matching

- `AbuseDefenceBotBlocked` — ADR-0297 abuse-defence baseline
- `AbuseDefenceSpoofDetected` — ADR-0297 abuse-defence baseline
- `AbuseDefenceScrapeBlocked` — ADR-0297 abuse-defence baseline
- `AbuseDefenceChallengeIssued` — ADR-0297 abuse-defence baseline
- `AbuseDefenceChallengeSolved` — ADR-0297 abuse-defence baseline
- `AbuseDefenceChallengeFailed` — ADR-0297 abuse-defence baseline
- `AbuseDefenceRateLimitHit` — ADR-0297 abuse-defence baseline
- `AbuseDefenceRateLimitFallback` — ADR-0297 abuse-defence baseline
- `AbuseDefenceHoneypotHit` — ADR-0297 abuse-defence baseline
- `AbuseDefenceCanaryRecovered` — ADR-0297 abuse-defence baseline
- `AbuseDefenceQuotaExceeded` — ADR-0297 abuse-defence baseline
- `AbuseDefenceCredentialPwned` — ADR-0297 abuse-defence baseline
- `AbuseDefenceCredentialStuffing` — ADR-0297 abuse-defence baseline
- `AbuseDefenceAttestationFailed` — ADR-0297 abuse-defence baseline
- `AbuseDefenceVendorOutage` — ADR-0297 abuse-defence baseline
- `AbuseDefenceFragmentActivated` — ADR-0297 abuse-defence baseline
- `AbuseDefenceWatermarkRecovered` — ADR-0297 abuse-defence baseline
- `AbuseDefenceSPIREOutage` — ADR-0297 abuse-defence baseline
- `ConglomerateGrantCreated` — ADR-0313 conglomerate tenant hierarchy
- `ConglomerateGrantRevoked` — ADR-0313 conglomerate tenant hierarchy
- `ConglomerateParentReadAction` — ADR-0313 conglomerate tenant hierarchy
- `ConglomerateCrossJurisdictionResidencyEnforced` — ADR-0313 conglomerate tenant hierarchy
- `ConglomerateInformationBarrierCrossingRefused` — ADR-0313 conglomerate tenant hierarchy
- `ConglomeratePersonalTenantBoundaryRefused` — ADR-0313 conglomerate tenant hierarchy
- `OfficeScopeAssignmentCreated` — ADR-0319 front/middle/back office information barrier
- `OfficeScopeAssignmentChanged` — ADR-0319 front/middle/back office information barrier
- `OfficeScopeAssignmentRevoked` — ADR-0319 front/middle/back office information barrier
- `OfficeBoundaryClearanceRequested` — ADR-0319 front/middle/back office information barrier
- `OfficeBoundaryClearanceApproved` — ADR-0319 front/middle/back office information barrier
- `OfficeBoundaryClearanceDenied` — ADR-0319 front/middle/back office information barrier
- `OfficeBoundaryClearanceRevoked` — ADR-0319 front/middle/back office information barrier
- `OfficeBoundaryAttemptEvaluated` — ADR-0319 front/middle/back office information barrier
- `OfficeBoundaryAttemptDenied` — ADR-0319 front/middle/back office information barrier
- `OfficeBoundaryAttemptAllowed` — ADR-0319 front/middle/back office information barrier
- `InformationBarrierTaintAttached` — ADR-0319 front/middle/back office information barrier
- `InformationBarrierTaintDerived` — ADR-0319 front/middle/back office information barrier
- `InformationBarrierTaintReleased` — ADR-0319 front/middle/back office information barrier
- `RestrictedDealCreated` — ADR-0319 front/middle/back office information barrier
- `RestrictedDealParticipantAdded` — ADR-0319 front/middle/back office information barrier
- `RestrictedDealParticipantRemoved` — ADR-0319 front/middle/back office information barrier
- `RestrictedDealReleased` — ADR-0319 front/middle/back office information barrier
- `AdvisorRelationshipBarrierCreated` — ADR-0319 front/middle/back office information barrier
- `AdvisorRelationshipBarrierAttemptEvaluated` — ADR-0319 front/middle/back office information barrier
- `OfficePackOverlayActivated` — ADR-0319 front/middle/back office information barrier
- `OfficePackOverlayChanged` — ADR-0319 front/middle/back office information barrier
- `OfficePackOverlayRetired` — ADR-0319 front/middle/back office information barrier

### §1.2 Strict-scope service set

- `contact-center` — policies: 6; contract files: 6; IP slices scanned: 30.
- `contract-lifecycle-management` — policies: 6; contract files: 6; IP slices scanned: 30.
- `data-pipeline` — policies: 6; contract files: 6; IP slices scanned: 30.
- `data-warehouse` — policies: 6; contract files: 6; IP slices scanned: 30.
- `design-collaboration` — policies: 6; contract files: 6; IP slices scanned: 30.
- `financial-planning` — policies: 6; contract files: 6; IP slices scanned: 30.
- `healthcare-integration` — policies: 6; contract files: 6; IP slices scanned: 30.
- `incident-management` — policies: 6; contract files: 6; IP slices scanned: 30.
- `itsm` — policies: 6; contract files: 6; IP slices scanned: 30.
- `learning-management` — policies: 6; contract files: 6; IP slices scanned: 30.
- `marketing-automation` — policies: 6; contract files: 6; IP slices scanned: 30.
- `marketplace` — policies: 6; contract files: 3; IP slices scanned: 40.
- `performance-management` — policies: 6; contract files: 6; IP slices scanned: 30.
- `whiteboard` — policies: 6; contract files: 6; IP slices scanned: 30.
- `workplace-integration` — policies: 6; contract files: 3; IP slices scanned: 41.

## §2 — Per-microservice coverage table

| µservice | total endpoints | endpoints with named audit-event class | coverage % | registered named classes currently observed |
|---|---:|---:|---:|---:|
| `contact-center` | 8 | 1 | 12.5% | 0 |
| `contract-lifecycle-management` | 8 | 1 | 12.5% | 0 |
| `data-pipeline` | 8 | 1 | 12.5% | 0 |
| `data-warehouse` | 8 | 1 | 12.5% | 0 |
| `design-collaboration` | 8 | 1 | 12.5% | 0 |
| `financial-planning` | 8 | 1 | 12.5% | 0 |
| `healthcare-integration` | 8 | 1 | 12.5% | 0 |
| `incident-management` | 8 | 1 | 12.5% | 0 |
| `itsm` | 8 | 1 | 12.5% | 0 |
| `learning-management` | 8 | 1 | 12.5% | 0 |
| `marketing-automation` | 8 | 1 | 12.5% | 0 |
| `marketplace` | 15 | 14 | 93.3% | 0 |
| `performance-management` | 8 | 1 | 12.5% | 0 |
| `whiteboard` | 8 | 1 | 12.5% | 0 |
| `workplace-integration` | 15 | 14 | 93.3% | 0 |

### §2.1 Coverage notes

- Strict-scope endpoint total: 134.
- Endpoints with a current concrete class name: 41.
- Endpoints whose current concrete class is registered in ADR-0263: 0.
- Gaps catalogued in §3: 134.
- `marketplace` and `workplace-integration` have many named classes, but both still fail registered-class compliance and several REST operations reuse one class across distinct transitions.
- The thirteen local-operation services expose one concrete top-level action-accepted publish message each; local REST/gRPC mutation contracts and local domain/SLO publish surfaces do not bind endpoint-specific audit-event classes.

## §3 — Gaps catalogue

| gap id | priority | µservice | endpoint | missing-event-class-name | recommended fix |
|---|---|---|---|---|---|
| GAP-0001 | P1 | `contact-center` | `ASYNC SEND publishActionAccepted` | `ContactCenterActionAccepted` | Register `ContactCenterActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0002 | P0 | `contact-center` | `GRPC RPC InvokeAction` | `ContactCenterActionInvoked` | Add `ContactCenterActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0003 | P1 | `contact-center` | `ASYNC SEND publishDomainEvent` | `ContactCenterDomainEvent` | Add `ContactCenterDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0004 | P1 | `contact-center` | `ASYNC SEND publishSloBurnEvent` | `ContactCenterSloBurnEvent` | Add `ContactCenterSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0005 | P0 | `contact-center` | `REST POST /contact-center/v1/interactions/{resource_id}/policy-check` | `ContactCenterInteractionPolicyDecisionRecorded` | Add `ContactCenterInteractionPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0006 | P0 | `contact-center` | `REST POST /contact-center/v1/interactions/{resource_id}/operator-remediation` | `ContactCenterInteractionOperatorRemediationApplied` | Add `ContactCenterInteractionOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0007 | P0 | `contact-center` | `GRPC RPC CheckLocalPolicy` | `ContactCenterLocalPolicyDecisionRecorded` | Add `ContactCenterLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0008 | P0 | `contact-center` | `REST POST /contact-center/actions/{action_id}` | `ContactCenterActionAccepted` | Add `ContactCenterActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0009 | P1 | `contract-lifecycle-management` | `ASYNC SEND publishActionAccepted` | `ContractLifecycleManagementActionAccepted` | Register `ContractLifecycleManagementActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0010 | P0 | `contract-lifecycle-management` | `GRPC RPC InvokeAction` | `ContractLifecycleManagementActionInvoked` | Add `ContractLifecycleManagementActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0011 | P1 | `contract-lifecycle-management` | `ASYNC SEND publishDomainEvent` | `ContractLifecycleManagementDomainEvent` | Add `ContractLifecycleManagementDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0012 | P1 | `contract-lifecycle-management` | `ASYNC SEND publishSloBurnEvent` | `ContractLifecycleManagementSloBurnEvent` | Add `ContractLifecycleManagementSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0013 | P0 | `contract-lifecycle-management` | `REST POST /contract-lifecycle-management/v1/contract-workspaces/{resource_id}/policy-check` | `ContractLifecycleManagementContractWorkspacePolicyDecisionRecorded` | Add `ContractLifecycleManagementContractWorkspacePolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0014 | P0 | `contract-lifecycle-management` | `REST POST /contract-lifecycle-management/v1/contract-workspaces/{resource_id}/operator-remediation` | `ContractLifecycleManagementContractWorkspaceOperatorRemediationApplied` | Add `ContractLifecycleManagementContractWorkspaceOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0015 | P0 | `contract-lifecycle-management` | `GRPC RPC CheckLocalPolicy` | `ContractLifecycleManagementLocalPolicyDecisionRecorded` | Add `ContractLifecycleManagementLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0016 | P0 | `contract-lifecycle-management` | `REST POST /contract-lifecycle-management/actions/{action_id}` | `ContractLifecycleManagementActionAccepted` | Add `ContractLifecycleManagementActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0017 | P1 | `data-pipeline` | `ASYNC SEND publishActionAccepted` | `DataPipelineActionAccepted` | Register `DataPipelineActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0018 | P0 | `data-pipeline` | `GRPC RPC InvokeAction` | `DataPipelineActionInvoked` | Add `DataPipelineActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0019 | P1 | `data-pipeline` | `ASYNC SEND publishDomainEvent` | `DataPipelineDomainEvent` | Add `DataPipelineDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0020 | P1 | `data-pipeline` | `ASYNC SEND publishSloBurnEvent` | `DataPipelineSloBurnEvent` | Add `DataPipelineSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0021 | P0 | `data-pipeline` | `REST POST /data-pipeline/v1/pipeline-runs/{resource_id}/policy-check` | `DataPipelinePipelineRunPolicyDecisionRecorded` | Add `DataPipelinePipelineRunPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0022 | P0 | `data-pipeline` | `REST POST /data-pipeline/v1/pipeline-runs/{resource_id}/operator-remediation` | `DataPipelinePipelineRunOperatorRemediationApplied` | Add `DataPipelinePipelineRunOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0023 | P0 | `data-pipeline` | `GRPC RPC CheckLocalPolicy` | `DataPipelineLocalPolicyDecisionRecorded` | Add `DataPipelineLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0024 | P0 | `data-pipeline` | `REST POST /data-pipeline/actions/{action_id}` | `DataPipelineActionAccepted` | Add `DataPipelineActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0025 | P1 | `data-warehouse` | `ASYNC SEND publishActionAccepted` | `DataWarehouseActionAccepted` | Register `DataWarehouseActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0026 | P0 | `data-warehouse` | `GRPC RPC InvokeAction` | `DataWarehouseActionInvoked` | Add `DataWarehouseActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0027 | P1 | `data-warehouse` | `ASYNC SEND publishDomainEvent` | `DataWarehouseDomainEvent` | Add `DataWarehouseDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0028 | P1 | `data-warehouse` | `ASYNC SEND publishSloBurnEvent` | `DataWarehouseSloBurnEvent` | Add `DataWarehouseSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0029 | P0 | `data-warehouse` | `REST POST /data-warehouse/v1/warehouse-datasets/{resource_id}/policy-check` | `DataWarehouseWarehouseDatasetPolicyDecisionRecorded` | Add `DataWarehouseWarehouseDatasetPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0030 | P0 | `data-warehouse` | `REST POST /data-warehouse/v1/warehouse-datasets/{resource_id}/operator-remediation` | `DataWarehouseWarehouseDatasetOperatorRemediationApplied` | Add `DataWarehouseWarehouseDatasetOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0031 | P0 | `data-warehouse` | `GRPC RPC CheckLocalPolicy` | `DataWarehouseLocalPolicyDecisionRecorded` | Add `DataWarehouseLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0032 | P0 | `data-warehouse` | `REST POST /data-warehouse/actions/{action_id}` | `DataWarehouseActionAccepted` | Add `DataWarehouseActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0033 | P1 | `design-collaboration` | `ASYNC SEND publishActionAccepted` | `DesignCollaborationActionAccepted` | Register `DesignCollaborationActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0034 | P0 | `design-collaboration` | `GRPC RPC InvokeAction` | `DesignCollaborationActionInvoked` | Add `DesignCollaborationActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0035 | P1 | `design-collaboration` | `ASYNC SEND publishDomainEvent` | `DesignCollaborationDomainEvent` | Add `DesignCollaborationDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0036 | P1 | `design-collaboration` | `ASYNC SEND publishSloBurnEvent` | `DesignCollaborationSloBurnEvent` | Add `DesignCollaborationSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0037 | P0 | `design-collaboration` | `REST POST /design-collaboration/v1/design-files/{resource_id}/policy-check` | `DesignCollaborationDesignFilePolicyDecisionRecorded` | Add `DesignCollaborationDesignFilePolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0038 | P0 | `design-collaboration` | `REST POST /design-collaboration/v1/design-files/{resource_id}/operator-remediation` | `DesignCollaborationDesignFileOperatorRemediationApplied` | Add `DesignCollaborationDesignFileOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0039 | P0 | `design-collaboration` | `GRPC RPC CheckLocalPolicy` | `DesignCollaborationLocalPolicyDecisionRecorded` | Add `DesignCollaborationLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0040 | P0 | `design-collaboration` | `REST POST /design-collaboration/actions/{action_id}` | `DesignCollaborationActionAccepted` | Add `DesignCollaborationActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0041 | P1 | `financial-planning` | `ASYNC SEND publishActionAccepted` | `FinancialPlanningActionAccepted` | Register `FinancialPlanningActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0042 | P0 | `financial-planning` | `GRPC RPC InvokeAction` | `FinancialPlanningActionInvoked` | Add `FinancialPlanningActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0043 | P1 | `financial-planning` | `ASYNC SEND publishDomainEvent` | `FinancialPlanningDomainEvent` | Add `FinancialPlanningDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0044 | P1 | `financial-planning` | `ASYNC SEND publishSloBurnEvent` | `FinancialPlanningSloBurnEvent` | Add `FinancialPlanningSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0045 | P0 | `financial-planning` | `REST POST /financial-planning/v1/planning-cycles/{resource_id}/policy-check` | `FinancialPlanningPlanningCyclePolicyDecisionRecorded` | Add `FinancialPlanningPlanningCyclePolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0046 | P0 | `financial-planning` | `REST POST /financial-planning/v1/planning-cycles/{resource_id}/operator-remediation` | `FinancialPlanningPlanningCycleOperatorRemediationApplied` | Add `FinancialPlanningPlanningCycleOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0047 | P0 | `financial-planning` | `GRPC RPC CheckLocalPolicy` | `FinancialPlanningLocalPolicyDecisionRecorded` | Add `FinancialPlanningLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0048 | P0 | `financial-planning` | `REST POST /financial-planning/actions/{action_id}` | `FinancialPlanningActionAccepted` | Add `FinancialPlanningActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0049 | P1 | `healthcare-integration` | `ASYNC SEND publishActionAccepted` | `HealthcareIntegrationActionAccepted` | Register `HealthcareIntegrationActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0050 | P0 | `healthcare-integration` | `GRPC RPC InvokeAction` | `HealthcareIntegrationActionInvoked` | Add `HealthcareIntegrationActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0051 | P1 | `healthcare-integration` | `ASYNC SEND publishDomainEvent` | `HealthcareIntegrationDomainEvent` | Add `HealthcareIntegrationDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0052 | P1 | `healthcare-integration` | `ASYNC SEND publishSloBurnEvent` | `HealthcareIntegrationSloBurnEvent` | Add `HealthcareIntegrationSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0053 | P0 | `healthcare-integration` | `REST POST /healthcare-integration/v1/clinical-exchanges/{resource_id}/policy-check` | `HealthcareIntegrationClinicalExchangePolicyDecisionRecorded` | Add `HealthcareIntegrationClinicalExchangePolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0054 | P0 | `healthcare-integration` | `REST POST /healthcare-integration/v1/clinical-exchanges/{resource_id}/operator-remediation` | `HealthcareIntegrationClinicalExchangeOperatorRemediationApplied` | Add `HealthcareIntegrationClinicalExchangeOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0055 | P0 | `healthcare-integration` | `GRPC RPC CheckLocalPolicy` | `HealthcareIntegrationLocalPolicyDecisionRecorded` | Add `HealthcareIntegrationLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0056 | P0 | `healthcare-integration` | `REST POST /healthcare-integration/actions/{action_id}` | `HealthcareIntegrationActionAccepted` | Add `HealthcareIntegrationActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0057 | P1 | `incident-management` | `ASYNC SEND publishActionAccepted` | `IncidentManagementActionAccepted` | Register `IncidentManagementActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0058 | P0 | `incident-management` | `GRPC RPC InvokeAction` | `IncidentManagementActionInvoked` | Add `IncidentManagementActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0059 | P1 | `incident-management` | `ASYNC SEND publishDomainEvent` | `IncidentManagementDomainEvent` | Add `IncidentManagementDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0060 | P1 | `incident-management` | `ASYNC SEND publishSloBurnEvent` | `IncidentManagementSloBurnEvent` | Add `IncidentManagementSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0061 | P0 | `incident-management` | `REST POST /incident-management/v1/incident-commands/{resource_id}/policy-check` | `IncidentManagementIncidentCommandPolicyDecisionRecorded` | Add `IncidentManagementIncidentCommandPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0062 | P0 | `incident-management` | `REST POST /incident-management/v1/incident-commands/{resource_id}/operator-remediation` | `IncidentManagementIncidentCommandOperatorRemediationApplied` | Add `IncidentManagementIncidentCommandOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0063 | P0 | `incident-management` | `GRPC RPC CheckLocalPolicy` | `IncidentManagementLocalPolicyDecisionRecorded` | Add `IncidentManagementLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0064 | P0 | `incident-management` | `REST POST /incident-management/actions/{action_id}` | `IncidentManagementActionAccepted` | Add `IncidentManagementActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0065 | P1 | `itsm` | `ASYNC SEND publishActionAccepted` | `ItsmActionAccepted` | Register `ItsmActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0066 | P0 | `itsm` | `GRPC RPC InvokeAction` | `ITSMActionInvoked` | Add `ITSMActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0067 | P1 | `itsm` | `ASYNC SEND publishDomainEvent` | `ItsmDomainEvent` | Add `ItsmDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0068 | P1 | `itsm` | `ASYNC SEND publishSloBurnEvent` | `ItsmSloBurnEvent` | Add `ItsmSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0069 | P0 | `itsm` | `REST POST /itsm/v1/service-records/{resource_id}/policy-check` | `ITSMServiceRecordPolicyDecisionRecorded` | Add `ITSMServiceRecordPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0070 | P0 | `itsm` | `REST POST /itsm/v1/service-records/{resource_id}/operator-remediation` | `ITSMServiceRecordOperatorRemediationApplied` | Add `ITSMServiceRecordOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0071 | P0 | `itsm` | `GRPC RPC CheckLocalPolicy` | `ITSMLocalPolicyDecisionRecorded` | Add `ITSMLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0072 | P0 | `itsm` | `REST POST /itsm/actions/{action_id}` | `ITSMActionAccepted` | Add `ITSMActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0073 | P1 | `learning-management` | `ASYNC SEND publishActionAccepted` | `LearningManagementActionAccepted` | Register `LearningManagementActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0074 | P0 | `learning-management` | `GRPC RPC InvokeAction` | `LearningManagementActionInvoked` | Add `LearningManagementActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0075 | P1 | `learning-management` | `ASYNC SEND publishDomainEvent` | `LearningManagementDomainEvent` | Add `LearningManagementDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0076 | P1 | `learning-management` | `ASYNC SEND publishSloBurnEvent` | `LearningManagementSloBurnEvent` | Add `LearningManagementSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0077 | P0 | `learning-management` | `REST POST /learning-management/v1/learning-cohorts/{resource_id}/policy-check` | `LearningManagementLearningCohortPolicyDecisionRecorded` | Add `LearningManagementLearningCohortPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0078 | P0 | `learning-management` | `REST POST /learning-management/v1/learning-cohorts/{resource_id}/operator-remediation` | `LearningManagementLearningCohortOperatorRemediationApplied` | Add `LearningManagementLearningCohortOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0079 | P0 | `learning-management` | `GRPC RPC CheckLocalPolicy` | `LearningManagementLocalPolicyDecisionRecorded` | Add `LearningManagementLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0080 | P0 | `learning-management` | `REST POST /learning-management/actions/{action_id}` | `LearningManagementActionAccepted` | Add `LearningManagementActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0081 | P1 | `marketing-automation` | `ASYNC SEND publishActionAccepted` | `MarketingAutomationActionAccepted` | Register `MarketingAutomationActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0082 | P1 | `marketing-automation` | `ASYNC SEND publishDomainEvent` | `MarketingAutomationDomainEvent` | Add `MarketingAutomationDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0083 | P1 | `marketing-automation` | `ASYNC SEND publishSloBurnEvent` | `MarketingAutomationSloBurnEvent` | Add `MarketingAutomationSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0084 | P0 | `marketing-automation` | `REST POST /marketing-automation/v1/campaign-journeys/{resource_id}/policy-check` | `MarketingAutomationCampaignJourneyPolicyDecisionRecorded` | Add `MarketingAutomationCampaignJourneyPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0085 | P0 | `marketing-automation` | `REST POST /marketing-automation/v1/campaign-journeys/{resource_id}/operator-remediation` | `MarketingAutomationCampaignJourneyOperatorRemediationApplied` | Add `MarketingAutomationCampaignJourneyOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0086 | P0 | `marketing-automation` | `GRPC RPC CheckLocalPolicy` | `MarketingAutomationLocalPolicyDecisionRecorded` | Add `MarketingAutomationLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0087 | P0 | `marketing-automation` | `GRPC RPC InvokeAction` | `MarketingAutomationActionInvoked` | Add `MarketingAutomationActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0088 | P0 | `marketing-automation` | `REST POST /marketing-automation/actions/{action_id}` | `MarketingAutomationActionAccepted` | Add `MarketingAutomationActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0089 | P1 | `marketplace` | `ASYNC SEND publishMarketplaceDealOffered` | `MarketplaceDealOffered` | Register `MarketplaceDealOffered` under ADR-0263 and make payload class explicit. |
| GAP-0090 | P1 | `marketplace` | `ASYNC SEND publishMarketplaceDealAccepted` | `MarketplaceDealAccepted` | Register `MarketplaceDealAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0091 | P1 | `marketplace` | `ASYNC SEND publishMarketplaceEscrowReserved` | `MarketplaceEscrowReserved` | Register `MarketplaceEscrowReserved` under ADR-0263 and make payload class explicit. |
| GAP-0092 | P1 | `marketplace` | `ASYNC SEND publishMarketplaceEscrowReleased` | `MarketplaceEscrowReleased` | Register `MarketplaceEscrowReleased` under ADR-0263 and make payload class explicit. |
| GAP-0093 | P1 | `marketplace` | `ASYNC SEND publishMarketplaceDisputeOpened` | `MarketplaceDisputeOpened` | Register `MarketplaceDisputeOpened` under ADR-0263 and make payload class explicit. |
| GAP-0094 | P1 | `marketplace` | `ASYNC SEND publishMarketplaceRevenueShareAccrued` | `MarketplaceRevenueShareAccrued` | Register `MarketplaceRevenueShareAccrued` under ADR-0263 and make payload class explicit. |
| GAP-0095 | P1 | `marketplace` | `ASYNC SEND publishMarketplaceOrderExported` | `MarketplaceOrderExported` | Register `MarketplaceOrderExported` under ADR-0263 and make payload class explicit. |
| GAP-0096 | P0 | `marketplace` | `GRPC RPC SubmitDealSet` | `MarketplaceDealOffered` | Add `MarketplaceDealOffered` to contract emission metadata and IP acceptance criteria. |
| GAP-0097 | P1 | `marketplace` | `REST POST /marketplace/deal-sets` | `MarketplaceDealOffered` | Register `MarketplaceDealOffered` under ADR-0263 and make payload class explicit. |
| GAP-0098 | P0 | `marketplace` | `REST POST /marketplace/deal-sets/{deal_set_id}/accept` | `MarketplaceDealAccepted` | Replace current `MarketplaceDealOffered` binding with `MarketplaceDealAccepted` and register the class. |
| GAP-0099 | P0 | `marketplace` | `REST POST /marketplace/deal-sets/{deal_set_id}/settle` | `MarketplaceEscrowReleased` | Replace current `MarketplaceDealOffered` binding with `MarketplaceEscrowReleased` and register the class. |
| GAP-0100 | P0 | `marketplace` | `REST POST /marketplace/listings` | `MarketplaceListingPublished` | Replace current `MarketplaceDealOffered` binding with `MarketplaceListingPublished` and register the class. |
| GAP-0101 | P0 | `marketplace` | `REST POST /marketplace/escrow/holds` | `MarketplaceEscrowReserved` | Replace current `MarketplaceDealOffered` binding with `MarketplaceEscrowReserved` and register the class. |
| GAP-0102 | P0 | `marketplace` | `REST POST /marketplace/disputes` | `MarketplaceDisputeOpened` | Replace current `MarketplaceDealOffered` binding with `MarketplaceDisputeOpened` and register the class. |
| GAP-0103 | P0 | `marketplace` | `REST POST /marketplace/revenue-shares` | `MarketplaceRevenueShareAccrued` | Replace current `MarketplaceDealOffered` binding with `MarketplaceRevenueShareAccrued` and register the class. |
| GAP-0104 | P1 | `performance-management` | `ASYNC SEND publishActionAccepted` | `PerformanceManagementActionAccepted` | Register `PerformanceManagementActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0105 | P1 | `performance-management` | `ASYNC SEND publishDomainEvent` | `PerformanceManagementDomainEvent` | Add `PerformanceManagementDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0106 | P1 | `performance-management` | `ASYNC SEND publishSloBurnEvent` | `PerformanceManagementSloBurnEvent` | Add `PerformanceManagementSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0107 | P0 | `performance-management` | `REST POST /performance-management/v1/review-cycles/{resource_id}/policy-check` | `PerformanceManagementReviewCyclePolicyDecisionRecorded` | Add `PerformanceManagementReviewCyclePolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0108 | P0 | `performance-management` | `REST POST /performance-management/v1/review-cycles/{resource_id}/operator-remediation` | `PerformanceManagementReviewCycleOperatorRemediationApplied` | Add `PerformanceManagementReviewCycleOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0109 | P0 | `performance-management` | `GRPC RPC CheckLocalPolicy` | `PerformanceManagementLocalPolicyDecisionRecorded` | Add `PerformanceManagementLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0110 | P0 | `performance-management` | `REST POST /performance-management/actions/{action_id}` | `PerformanceManagementActionAccepted` | Add `PerformanceManagementActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0111 | P0 | `performance-management` | `GRPC RPC InvokeAction` | `PerformanceManagementActionInvoked` | Add `PerformanceManagementActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0112 | P1 | `whiteboard` | `ASYNC SEND publishActionAccepted` | `WhiteboardActionAccepted` | Register `WhiteboardActionAccepted` under ADR-0263 and make payload class explicit. |
| GAP-0113 | P1 | `whiteboard` | `ASYNC SEND publishDomainEvent` | `WhiteboardDomainEvent` | Add `WhiteboardDomainEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0114 | P1 | `whiteboard` | `ASYNC SEND publishSloBurnEvent` | `WhiteboardSloBurnEvent` | Add `WhiteboardSloBurnEvent` to contract emission metadata and IP acceptance criteria. |
| GAP-0115 | P0 | `whiteboard` | `REST POST /whiteboard/v1/whiteboard-sessions/{resource_id}/policy-check` | `WhiteboardSessionPolicyDecisionRecorded` | Add `WhiteboardSessionPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0116 | P0 | `whiteboard` | `REST POST /whiteboard/v1/whiteboard-sessions/{resource_id}/operator-remediation` | `WhiteboardSessionOperatorRemediationApplied` | Add `WhiteboardSessionOperatorRemediationApplied` to contract emission metadata and IP acceptance criteria. |
| GAP-0117 | P0 | `whiteboard` | `GRPC RPC CheckLocalPolicy` | `WhiteboardLocalPolicyDecisionRecorded` | Add `WhiteboardLocalPolicyDecisionRecorded` to contract emission metadata and IP acceptance criteria. |
| GAP-0118 | P0 | `whiteboard` | `REST POST /whiteboard/actions/{action_id}` | `WhiteboardActionAccepted` | Add `WhiteboardActionAccepted` to contract emission metadata and IP acceptance criteria. |
| GAP-0119 | P0 | `whiteboard` | `GRPC RPC InvokeAction` | `WhiteboardActionInvoked` | Add `WhiteboardActionInvoked` to contract emission metadata and IP acceptance criteria. |
| GAP-0120 | P1 | `workplace-integration` | `ASYNC SEND publishWorkplaceESignSessionCreated` | `WorkplaceESignSessionCreated` | Register `WorkplaceESignSessionCreated` under ADR-0263 and make payload class explicit. |
| GAP-0121 | P1 | `workplace-integration` | `ASYNC SEND publishWorkplaceSignatureCaptured` | `WorkplaceSignatureCaptured` | Register `WorkplaceSignatureCaptured` under ADR-0263 and make payload class explicit. |
| GAP-0122 | P1 | `workplace-integration` | `ASYNC SEND publishWorkplaceOfferGenerated` | `WorkplaceOfferGenerated` | Register `WorkplaceOfferGenerated` under ADR-0263 and make payload class explicit. |
| GAP-0123 | P1 | `workplace-integration` | `ASYNC SEND publishWorkplaceAgreementBound` | `WorkplaceAgreementBound` | Register `WorkplaceAgreementBound` under ADR-0263 and make payload class explicit. |
| GAP-0124 | P1 | `workplace-integration` | `ASYNC SEND publishWorkplaceRosterBindingGranted` | `WorkplaceRosterBindingGranted` | Register `WorkplaceRosterBindingGranted` under ADR-0263 and make payload class explicit. |
| GAP-0125 | P1 | `workplace-integration` | `ASYNC SEND publishWorkplaceClockEventAttested` | `WorkplaceClockEventAttested` | Register `WorkplaceClockEventAttested` under ADR-0263 and make payload class explicit. |
| GAP-0126 | P1 | `workplace-integration` | `ASYNC SEND publishWorkplaceDlpTraceSealed` | `WorkplaceDlpTraceSealed` | Register `WorkplaceDlpTraceSealed` under ADR-0263 and make payload class explicit. |
| GAP-0127 | P1 | `workplace-integration` | `REST POST /workplace/esign/sessions` | `WorkplaceESignSessionCreated` | Register `WorkplaceESignSessionCreated` under ADR-0263 and make payload class explicit. |
| GAP-0128 | P0 | `workplace-integration` | `REST POST /workplace/esign/sessions/{session_id}/sign` | `WorkplaceSignatureCaptured` | Replace current `WorkplaceESignSessionCreated` binding with `WorkplaceSignatureCaptured` and register the class. |
| GAP-0129 | P0 | `workplace-integration` | `REST POST /workplace/offer-letters` | `WorkplaceOfferGenerated` | Replace current `WorkplaceESignSessionCreated` binding with `WorkplaceOfferGenerated` and register the class. |
| GAP-0130 | P0 | `workplace-integration` | `REST POST /workplace/engagement-agreements` | `WorkplaceAgreementBound` | Replace current `WorkplaceESignSessionCreated` binding with `WorkplaceAgreementBound` and register the class. |
| GAP-0131 | P0 | `workplace-integration` | `REST POST /workplace/roster-bindings` | `WorkplaceRosterBindingGranted` | Replace current `WorkplaceESignSessionCreated` binding with `WorkplaceRosterBindingGranted` and register the class. |
| GAP-0132 | P0 | `workplace-integration` | `REST POST /workplace/clock-events` | `WorkplaceClockEventAttested` | Replace current `WorkplaceESignSessionCreated` binding with `WorkplaceClockEventAttested` and register the class. |
| GAP-0133 | P0 | `workplace-integration` | `REST POST /workplace/dlp-traces` | `WorkplaceDlpTraceSealed` | Replace current `WorkplaceESignSessionCreated` binding with `WorkplaceDlpTraceSealed` and register the class. |
| GAP-0134 | P0 | `workplace-integration` | `GRPC RPC SubmitWorkplaceAgreement` | `WorkplaceAgreementBound` | Add `WorkplaceAgreementBound` to contract emission metadata and IP acceptance criteria. |

### §3.1 Detailed gap records

#### GAP-0001 — `contact-center` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/contact-center/contracts/asyncapi-v1.yaml`
- Current class evidence: `ContactCenterActionAccepted`
- Missing-event-class-name / should emit: `ContactCenterActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `ContactCenterActionAccepted`
- IP exact-class evidence: `none`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/contact-center/contracts/asyncapi-v1.yaml` to bind `ContactCenterActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContactCenterActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0002 — `contact-center` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/contact-center/contracts/contact-center-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContactCenterActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/contact-center/contracts/contact-center-v1.proto` to bind `ContactCenterActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContactCenterActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0003 — `contact-center` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/contact-center/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContactCenterDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/contact-center/contracts/local-asyncapi-v1.yaml` to bind `ContactCenterDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContactCenterDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0004 — `contact-center` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/contact-center/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContactCenterSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/contact-center/contracts/local-asyncapi-v1.yaml` to bind `ContactCenterSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContactCenterSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0005 — `contact-center` — `REST POST /contact-center/v1/interactions/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/contact-center/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContactCenterInteractionPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/contact-center/contracts/local-openapi-v1.yaml` to bind `ContactCenterInteractionPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContactCenterInteractionPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0006 — `contact-center` — `REST POST /contact-center/v1/interactions/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/contact-center/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContactCenterInteractionOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/contact-center/contracts/local-openapi-v1.yaml` to bind `ContactCenterInteractionOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContactCenterInteractionOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0007 — `contact-center` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/contact-center/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContactCenterLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/contact-center/contracts/local-operations-v1.proto` to bind `ContactCenterLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContactCenterLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0008 — `contact-center` — `REST POST /contact-center/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/contact-center/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContactCenterActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `ContactCenterActionAccepted`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/contact-center/contracts/openapi-v1.yaml` to bind `ContactCenterActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContactCenterActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0009 — `contract-lifecycle-management` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/contract-lifecycle-management/contracts/asyncapi-v1.yaml`
- Current class evidence: `ContractLifecycleManagementActionAccepted`
- Missing-event-class-name / should emit: `ContractLifecycleManagementActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `ContractLifecycleManagementActionAccepted`
- IP exact-class evidence: `none`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/contract-lifecycle-management/contracts/asyncapi-v1.yaml` to bind `ContractLifecycleManagementActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContractLifecycleManagementActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0010 — `contract-lifecycle-management` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/contract-lifecycle-management/contracts/contract-lifecycle-management-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContractLifecycleManagementActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/contract-lifecycle-management/contracts/contract-lifecycle-management-v1.proto` to bind `ContractLifecycleManagementActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContractLifecycleManagementActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0011 — `contract-lifecycle-management` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/contract-lifecycle-management/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContractLifecycleManagementDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/contract-lifecycle-management/contracts/local-asyncapi-v1.yaml` to bind `ContractLifecycleManagementDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContractLifecycleManagementDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0012 — `contract-lifecycle-management` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/contract-lifecycle-management/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContractLifecycleManagementSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/contract-lifecycle-management/contracts/local-asyncapi-v1.yaml` to bind `ContractLifecycleManagementSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContractLifecycleManagementSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0013 — `contract-lifecycle-management` — `REST POST /contract-lifecycle-management/v1/contract-workspaces/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/contract-lifecycle-management/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContractLifecycleManagementContractWorkspacePolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/contract-lifecycle-management/contracts/local-openapi-v1.yaml` to bind `ContractLifecycleManagementContractWorkspacePolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContractLifecycleManagementContractWorkspacePolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0014 — `contract-lifecycle-management` — `REST POST /contract-lifecycle-management/v1/contract-workspaces/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/contract-lifecycle-management/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContractLifecycleManagementContractWorkspaceOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/contract-lifecycle-management/contracts/local-openapi-v1.yaml` to bind `ContractLifecycleManagementContractWorkspaceOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContractLifecycleManagementContractWorkspaceOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0015 — `contract-lifecycle-management` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/contract-lifecycle-management/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContractLifecycleManagementLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/contract-lifecycle-management/contracts/local-operations-v1.proto` to bind `ContractLifecycleManagementLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContractLifecycleManagementLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0016 — `contract-lifecycle-management` — `REST POST /contract-lifecycle-management/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/contract-lifecycle-management/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ContractLifecycleManagementActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `ContractLifecycleManagementActionAccepted`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/contract-lifecycle-management/contracts/openapi-v1.yaml` to bind `ContractLifecycleManagementActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ContractLifecycleManagementActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0017 — `data-pipeline` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/data-pipeline/contracts/asyncapi-v1.yaml`
- Current class evidence: `DataPipelineActionAccepted`
- Missing-event-class-name / should emit: `DataPipelineActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `DataPipelineActionAccepted`
- IP exact-class evidence: `none`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/data-pipeline/contracts/asyncapi-v1.yaml` to bind `DataPipelineActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataPipelineActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0018 — `data-pipeline` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/data-pipeline/contracts/data-pipeline-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataPipelineActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/data-pipeline/contracts/data-pipeline-v1.proto` to bind `DataPipelineActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataPipelineActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0019 — `data-pipeline` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataPipelineDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` to bind `DataPipelineDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataPipelineDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0020 — `data-pipeline` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataPipelineSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` to bind `DataPipelineSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataPipelineSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0021 — `data-pipeline` — `REST POST /data-pipeline/v1/pipeline-runs/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/data-pipeline/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataPipelinePipelineRunPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/data-pipeline/contracts/local-openapi-v1.yaml` to bind `DataPipelinePipelineRunPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataPipelinePipelineRunPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0022 — `data-pipeline` — `REST POST /data-pipeline/v1/pipeline-runs/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/data-pipeline/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataPipelinePipelineRunOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/data-pipeline/contracts/local-openapi-v1.yaml` to bind `DataPipelinePipelineRunOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataPipelinePipelineRunOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0023 — `data-pipeline` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/data-pipeline/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataPipelineLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/data-pipeline/contracts/local-operations-v1.proto` to bind `DataPipelineLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataPipelineLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0024 — `data-pipeline` — `REST POST /data-pipeline/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/data-pipeline/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataPipelineActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `DataPipelineActionAccepted`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/data-pipeline/contracts/openapi-v1.yaml` to bind `DataPipelineActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataPipelineActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0025 — `data-warehouse` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/data-warehouse/contracts/asyncapi-v1.yaml`
- Current class evidence: `DataWarehouseActionAccepted`
- Missing-event-class-name / should emit: `DataWarehouseActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `DataWarehouseActionAccepted`
- IP exact-class evidence: `none`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/data-warehouse/contracts/asyncapi-v1.yaml` to bind `DataWarehouseActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataWarehouseActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0026 — `data-warehouse` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/data-warehouse/contracts/data-warehouse-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataWarehouseActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/data-warehouse/contracts/data-warehouse-v1.proto` to bind `DataWarehouseActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataWarehouseActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0027 — `data-warehouse` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/data-warehouse/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataWarehouseDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/data-warehouse/contracts/local-asyncapi-v1.yaml` to bind `DataWarehouseDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataWarehouseDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0028 — `data-warehouse` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/data-warehouse/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataWarehouseSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/data-warehouse/contracts/local-asyncapi-v1.yaml` to bind `DataWarehouseSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataWarehouseSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0029 — `data-warehouse` — `REST POST /data-warehouse/v1/warehouse-datasets/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/data-warehouse/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataWarehouseWarehouseDatasetPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/data-warehouse/contracts/local-openapi-v1.yaml` to bind `DataWarehouseWarehouseDatasetPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataWarehouseWarehouseDatasetPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0030 — `data-warehouse` — `REST POST /data-warehouse/v1/warehouse-datasets/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/data-warehouse/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataWarehouseWarehouseDatasetOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/data-warehouse/contracts/local-openapi-v1.yaml` to bind `DataWarehouseWarehouseDatasetOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataWarehouseWarehouseDatasetOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0031 — `data-warehouse` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/data-warehouse/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataWarehouseLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/data-warehouse/contracts/local-operations-v1.proto` to bind `DataWarehouseLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataWarehouseLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0032 — `data-warehouse` — `REST POST /data-warehouse/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/data-warehouse/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DataWarehouseActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `DataWarehouseActionAccepted`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/data-warehouse/contracts/openapi-v1.yaml` to bind `DataWarehouseActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DataWarehouseActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0033 — `design-collaboration` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/design-collaboration/contracts/asyncapi-v1.yaml`
- Current class evidence: `DesignCollaborationActionAccepted`
- Missing-event-class-name / should emit: `DesignCollaborationActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `DesignCollaborationActionAccepted`
- IP exact-class evidence: `none`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/design-collaboration/contracts/asyncapi-v1.yaml` to bind `DesignCollaborationActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DesignCollaborationActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0034 — `design-collaboration` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/design-collaboration/contracts/design-collaboration-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DesignCollaborationActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/design-collaboration/contracts/design-collaboration-v1.proto` to bind `DesignCollaborationActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DesignCollaborationActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0035 — `design-collaboration` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/design-collaboration/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DesignCollaborationDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/design-collaboration/contracts/local-asyncapi-v1.yaml` to bind `DesignCollaborationDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DesignCollaborationDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0036 — `design-collaboration` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/design-collaboration/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DesignCollaborationSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/design-collaboration/contracts/local-asyncapi-v1.yaml` to bind `DesignCollaborationSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DesignCollaborationSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0037 — `design-collaboration` — `REST POST /design-collaboration/v1/design-files/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/design-collaboration/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DesignCollaborationDesignFilePolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/design-collaboration/contracts/local-openapi-v1.yaml` to bind `DesignCollaborationDesignFilePolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DesignCollaborationDesignFilePolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0038 — `design-collaboration` — `REST POST /design-collaboration/v1/design-files/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/design-collaboration/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DesignCollaborationDesignFileOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/design-collaboration/contracts/local-openapi-v1.yaml` to bind `DesignCollaborationDesignFileOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DesignCollaborationDesignFileOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0039 — `design-collaboration` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/design-collaboration/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DesignCollaborationLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/design-collaboration/contracts/local-operations-v1.proto` to bind `DesignCollaborationLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DesignCollaborationLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0040 — `design-collaboration` — `REST POST /design-collaboration/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/design-collaboration/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `DesignCollaborationActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `DesignCollaborationActionAccepted`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/design-collaboration/contracts/openapi-v1.yaml` to bind `DesignCollaborationActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `DesignCollaborationActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0041 — `financial-planning` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/financial-planning/contracts/asyncapi-v1.yaml`
- Current class evidence: `FinancialPlanningActionAccepted`
- Missing-event-class-name / should emit: `FinancialPlanningActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `FinancialPlanningActionAccepted`
- IP exact-class evidence: `none`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/financial-planning/contracts/asyncapi-v1.yaml` to bind `FinancialPlanningActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `FinancialPlanningActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0042 — `financial-planning` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/financial-planning/contracts/financial-planning-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `FinancialPlanningActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/financial-planning/contracts/financial-planning-v1.proto` to bind `FinancialPlanningActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `FinancialPlanningActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0043 — `financial-planning` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/financial-planning/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `FinancialPlanningDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/financial-planning/contracts/local-asyncapi-v1.yaml` to bind `FinancialPlanningDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `FinancialPlanningDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0044 — `financial-planning` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/financial-planning/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `FinancialPlanningSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/financial-planning/contracts/local-asyncapi-v1.yaml` to bind `FinancialPlanningSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `FinancialPlanningSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0045 — `financial-planning` — `REST POST /financial-planning/v1/planning-cycles/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/financial-planning/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `FinancialPlanningPlanningCyclePolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/financial-planning/contracts/local-openapi-v1.yaml` to bind `FinancialPlanningPlanningCyclePolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `FinancialPlanningPlanningCyclePolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0046 — `financial-planning` — `REST POST /financial-planning/v1/planning-cycles/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/financial-planning/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `FinancialPlanningPlanningCycleOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/financial-planning/contracts/local-openapi-v1.yaml` to bind `FinancialPlanningPlanningCycleOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `FinancialPlanningPlanningCycleOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0047 — `financial-planning` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/financial-planning/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `FinancialPlanningLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/financial-planning/contracts/local-operations-v1.proto` to bind `FinancialPlanningLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `FinancialPlanningLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0048 — `financial-planning` — `REST POST /financial-planning/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/financial-planning/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `FinancialPlanningActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `FinancialPlanningActionAccepted`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/financial-planning/contracts/openapi-v1.yaml` to bind `FinancialPlanningActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `FinancialPlanningActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0049 — `healthcare-integration` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/healthcare-integration/contracts/asyncapi-v1.yaml`
- Current class evidence: `HealthcareIntegrationActionAccepted`
- Missing-event-class-name / should emit: `HealthcareIntegrationActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `HealthcareIntegrationActionAccepted`
- IP exact-class evidence: `HealthcareIntegrationActionAccepted`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/healthcare-integration/contracts/asyncapi-v1.yaml` to bind `HealthcareIntegrationActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `HealthcareIntegrationActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0050 — `healthcare-integration` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/healthcare-integration/contracts/healthcare-integration-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `HealthcareIntegrationActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/healthcare-integration/contracts/healthcare-integration-v1.proto` to bind `HealthcareIntegrationActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `HealthcareIntegrationActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0051 — `healthcare-integration` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `HealthcareIntegrationDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml` to bind `HealthcareIntegrationDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `HealthcareIntegrationDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0052 — `healthcare-integration` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `HealthcareIntegrationSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml` to bind `HealthcareIntegrationSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `HealthcareIntegrationSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0053 — `healthcare-integration` — `REST POST /healthcare-integration/v1/clinical-exchanges/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/healthcare-integration/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `HealthcareIntegrationClinicalExchangePolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/healthcare-integration/contracts/local-openapi-v1.yaml` to bind `HealthcareIntegrationClinicalExchangePolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `HealthcareIntegrationClinicalExchangePolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0054 — `healthcare-integration` — `REST POST /healthcare-integration/v1/clinical-exchanges/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/healthcare-integration/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `HealthcareIntegrationClinicalExchangeOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/healthcare-integration/contracts/local-openapi-v1.yaml` to bind `HealthcareIntegrationClinicalExchangeOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `HealthcareIntegrationClinicalExchangeOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0055 — `healthcare-integration` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/healthcare-integration/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `HealthcareIntegrationLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/healthcare-integration/contracts/local-operations-v1.proto` to bind `HealthcareIntegrationLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `HealthcareIntegrationLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0056 — `healthcare-integration` — `REST POST /healthcare-integration/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/healthcare-integration/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `HealthcareIntegrationActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `HealthcareIntegrationActionAccepted`
- IP exact-class evidence: `HealthcareIntegrationActionAccepted`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/healthcare-integration/contracts/openapi-v1.yaml` to bind `HealthcareIntegrationActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `HealthcareIntegrationActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0057 — `incident-management` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/incident-management/contracts/asyncapi-v1.yaml`
- Current class evidence: `IncidentManagementActionAccepted`
- Missing-event-class-name / should emit: `IncidentManagementActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `IncidentManagementActionAccepted`
- IP exact-class evidence: `none`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/incident-management/contracts/asyncapi-v1.yaml` to bind `IncidentManagementActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `IncidentManagementActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0058 — `incident-management` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/incident-management/contracts/incident-management-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `IncidentManagementActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/incident-management/contracts/incident-management-v1.proto` to bind `IncidentManagementActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `IncidentManagementActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0059 — `incident-management` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/incident-management/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `IncidentManagementDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/incident-management/contracts/local-asyncapi-v1.yaml` to bind `IncidentManagementDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `IncidentManagementDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0060 — `incident-management` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/incident-management/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `IncidentManagementSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/incident-management/contracts/local-asyncapi-v1.yaml` to bind `IncidentManagementSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `IncidentManagementSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0061 — `incident-management` — `REST POST /incident-management/v1/incident-commands/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/incident-management/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `IncidentManagementIncidentCommandPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/incident-management/contracts/local-openapi-v1.yaml` to bind `IncidentManagementIncidentCommandPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `IncidentManagementIncidentCommandPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0062 — `incident-management` — `REST POST /incident-management/v1/incident-commands/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/incident-management/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `IncidentManagementIncidentCommandOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/incident-management/contracts/local-openapi-v1.yaml` to bind `IncidentManagementIncidentCommandOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `IncidentManagementIncidentCommandOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0063 — `incident-management` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/incident-management/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `IncidentManagementLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/incident-management/contracts/local-operations-v1.proto` to bind `IncidentManagementLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `IncidentManagementLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0064 — `incident-management` — `REST POST /incident-management/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/incident-management/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `IncidentManagementActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `IncidentManagementActionAccepted`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/incident-management/contracts/openapi-v1.yaml` to bind `IncidentManagementActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `IncidentManagementActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0065 — `itsm` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/itsm/contracts/asyncapi-v1.yaml`
- Current class evidence: `ItsmActionAccepted`
- Missing-event-class-name / should emit: `ItsmActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `ItsmActionAccepted`
- IP exact-class evidence: `ItsmActionAccepted`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/itsm/contracts/asyncapi-v1.yaml` to bind `ItsmActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ItsmActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0066 — `itsm` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/itsm/contracts/itsm-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ITSMActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/itsm/contracts/itsm-v1.proto` to bind `ITSMActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ITSMActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0067 — `itsm` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/itsm/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ItsmDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/itsm/contracts/local-asyncapi-v1.yaml` to bind `ItsmDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ItsmDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0068 — `itsm` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/itsm/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ItsmSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/itsm/contracts/local-asyncapi-v1.yaml` to bind `ItsmSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ItsmSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0069 — `itsm` — `REST POST /itsm/v1/service-records/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/itsm/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ITSMServiceRecordPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/itsm/contracts/local-openapi-v1.yaml` to bind `ITSMServiceRecordPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ITSMServiceRecordPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0070 — `itsm` — `REST POST /itsm/v1/service-records/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/itsm/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ITSMServiceRecordOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/itsm/contracts/local-openapi-v1.yaml` to bind `ITSMServiceRecordOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ITSMServiceRecordOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0071 — `itsm` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/itsm/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ITSMLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/itsm/contracts/local-operations-v1.proto` to bind `ITSMLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ITSMLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0072 — `itsm` — `REST POST /itsm/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/itsm/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `ITSMActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/itsm/contracts/openapi-v1.yaml` to bind `ITSMActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `ITSMActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0073 — `learning-management` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/learning-management/contracts/asyncapi-v1.yaml`
- Current class evidence: `LearningManagementActionAccepted`
- Missing-event-class-name / should emit: `LearningManagementActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `LearningManagementActionAccepted`
- IP exact-class evidence: `none`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/learning-management/contracts/asyncapi-v1.yaml` to bind `LearningManagementActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `LearningManagementActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0074 — `learning-management` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/learning-management/contracts/learning-management-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `LearningManagementActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/learning-management/contracts/learning-management-v1.proto` to bind `LearningManagementActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `LearningManagementActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0075 — `learning-management` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/learning-management/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `LearningManagementDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/learning-management/contracts/local-asyncapi-v1.yaml` to bind `LearningManagementDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `LearningManagementDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0076 — `learning-management` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/learning-management/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `LearningManagementSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/learning-management/contracts/local-asyncapi-v1.yaml` to bind `LearningManagementSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `LearningManagementSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0077 — `learning-management` — `REST POST /learning-management/v1/learning-cohorts/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/learning-management/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `LearningManagementLearningCohortPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/learning-management/contracts/local-openapi-v1.yaml` to bind `LearningManagementLearningCohortPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `LearningManagementLearningCohortPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0078 — `learning-management` — `REST POST /learning-management/v1/learning-cohorts/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/learning-management/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `LearningManagementLearningCohortOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/learning-management/contracts/local-openapi-v1.yaml` to bind `LearningManagementLearningCohortOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `LearningManagementLearningCohortOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0079 — `learning-management` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/learning-management/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `LearningManagementLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/learning-management/contracts/local-operations-v1.proto` to bind `LearningManagementLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `LearningManagementLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0080 — `learning-management` — `REST POST /learning-management/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/learning-management/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `LearningManagementActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `LearningManagementActionAccepted`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/learning-management/contracts/openapi-v1.yaml` to bind `LearningManagementActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `LearningManagementActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0081 — `marketing-automation` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/marketing-automation/contracts/asyncapi-v1.yaml`
- Current class evidence: `MarketingAutomationActionAccepted`
- Missing-event-class-name / should emit: `MarketingAutomationActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketingAutomationActionAccepted`
- IP exact-class evidence: `none`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/marketing-automation/contracts/asyncapi-v1.yaml` to bind `MarketingAutomationActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketingAutomationActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0082 — `marketing-automation` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `MarketingAutomationDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml` to bind `MarketingAutomationDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketingAutomationDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0083 — `marketing-automation` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `MarketingAutomationSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml` to bind `MarketingAutomationSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketingAutomationSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0084 — `marketing-automation` — `REST POST /marketing-automation/v1/campaign-journeys/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/marketing-automation/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `MarketingAutomationCampaignJourneyPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/marketing-automation/contracts/local-openapi-v1.yaml` to bind `MarketingAutomationCampaignJourneyPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketingAutomationCampaignJourneyPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0085 — `marketing-automation` — `REST POST /marketing-automation/v1/campaign-journeys/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/marketing-automation/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `MarketingAutomationCampaignJourneyOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/marketing-automation/contracts/local-openapi-v1.yaml` to bind `MarketingAutomationCampaignJourneyOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketingAutomationCampaignJourneyOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0086 — `marketing-automation` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/marketing-automation/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `MarketingAutomationLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/marketing-automation/contracts/local-operations-v1.proto` to bind `MarketingAutomationLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketingAutomationLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0087 — `marketing-automation` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/marketing-automation/contracts/marketing-automation-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `MarketingAutomationActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/marketing-automation/contracts/marketing-automation-v1.proto` to bind `MarketingAutomationActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketingAutomationActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0088 — `marketing-automation` — `REST POST /marketing-automation/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/marketing-automation/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `MarketingAutomationActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketingAutomationActionAccepted`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/marketing-automation/contracts/openapi-v1.yaml` to bind `MarketingAutomationActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketingAutomationActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0089 — `marketplace` — `ASYNC SEND publishMarketplaceDealOffered`
- Priority: `P1`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current class evidence: `MarketplaceDealOffered`
- Missing-event-class-name / should emit: `MarketplaceDealOffered`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDealOffered`
- IP exact-class evidence: `MarketplaceDealOffered`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/marketplace/contracts/asyncapi-v1.yaml` to bind `MarketplaceDealOffered`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceDealOffered`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0090 — `marketplace` — `ASYNC SEND publishMarketplaceDealAccepted`
- Priority: `P1`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current class evidence: `MarketplaceDealAccepted`
- Missing-event-class-name / should emit: `MarketplaceDealAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDealAccepted`
- IP exact-class evidence: `MarketplaceDealAccepted`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/marketplace/contracts/asyncapi-v1.yaml` to bind `MarketplaceDealAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceDealAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0091 — `marketplace` — `ASYNC SEND publishMarketplaceEscrowReserved`
- Priority: `P1`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current class evidence: `MarketplaceEscrowReserved`
- Missing-event-class-name / should emit: `MarketplaceEscrowReserved`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceEscrowReserved`
- IP exact-class evidence: `MarketplaceEscrowReserved`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/marketplace/contracts/asyncapi-v1.yaml` to bind `MarketplaceEscrowReserved`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceEscrowReserved`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0092 — `marketplace` — `ASYNC SEND publishMarketplaceEscrowReleased`
- Priority: `P1`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current class evidence: `MarketplaceEscrowReleased`
- Missing-event-class-name / should emit: `MarketplaceEscrowReleased`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceEscrowReleased`
- IP exact-class evidence: `MarketplaceEscrowReleased`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/marketplace/contracts/asyncapi-v1.yaml` to bind `MarketplaceEscrowReleased`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceEscrowReleased`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0093 — `marketplace` — `ASYNC SEND publishMarketplaceDisputeOpened`
- Priority: `P1`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current class evidence: `MarketplaceDisputeOpened`
- Missing-event-class-name / should emit: `MarketplaceDisputeOpened`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDisputeOpened`
- IP exact-class evidence: `MarketplaceDisputeOpened`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/marketplace/contracts/asyncapi-v1.yaml` to bind `MarketplaceDisputeOpened`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceDisputeOpened`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0094 — `marketplace` — `ASYNC SEND publishMarketplaceRevenueShareAccrued`
- Priority: `P1`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current class evidence: `MarketplaceRevenueShareAccrued`
- Missing-event-class-name / should emit: `MarketplaceRevenueShareAccrued`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceRevenueShareAccrued`
- IP exact-class evidence: `MarketplaceRevenueShareAccrued`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/marketplace/contracts/asyncapi-v1.yaml` to bind `MarketplaceRevenueShareAccrued`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceRevenueShareAccrued`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0095 — `marketplace` — `ASYNC SEND publishMarketplaceOrderExported`
- Priority: `P1`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current class evidence: `MarketplaceOrderExported`
- Missing-event-class-name / should emit: `MarketplaceOrderExported`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceOrderExported`
- IP exact-class evidence: `MarketplaceOrderExported`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/marketplace/contracts/asyncapi-v1.yaml` to bind `MarketplaceOrderExported`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceOrderExported`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0096 — `marketplace` — `GRPC RPC SubmitDealSet`
- Priority: `P0`
- Contract file: `microservices/marketplace/contracts/marketplace-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `MarketplaceDealOffered`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDealOffered`
- IP exact-class evidence: `MarketplaceDealOffered`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/marketplace/contracts/marketplace-v1.proto` to bind `MarketplaceDealOffered`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceDealOffered`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0097 — `marketplace` — `REST POST /marketplace/deal-sets`
- Priority: `P1`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current class evidence: `MarketplaceDealOffered`
- Missing-event-class-name / should emit: `MarketplaceDealOffered`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDealOffered`
- IP exact-class evidence: `MarketplaceDealOffered`
- Contract details: x-audit-event=MarketplaceDealOffered
- Recommended fix: update `microservices/marketplace/contracts/openapi-v1.yaml` to bind `MarketplaceDealOffered`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceDealOffered`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0098 — `marketplace` — `REST POST /marketplace/deal-sets/{deal_set_id}/accept`
- Priority: `P0`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current class evidence: `MarketplaceDealOffered`
- Missing-event-class-name / should emit: `MarketplaceDealAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDealAccepted,MarketplaceDealOffered`
- IP exact-class evidence: `MarketplaceDealAccepted,MarketplaceDealOffered`
- Contract details: x-audit-event=MarketplaceDealOffered
- Recommended fix: update `microservices/marketplace/contracts/openapi-v1.yaml` to bind `MarketplaceDealAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceDealAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0099 — `marketplace` — `REST POST /marketplace/deal-sets/{deal_set_id}/settle`
- Priority: `P0`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current class evidence: `MarketplaceDealOffered`
- Missing-event-class-name / should emit: `MarketplaceEscrowReleased`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDealOffered,MarketplaceEscrowReleased`
- IP exact-class evidence: `MarketplaceDealOffered,MarketplaceEscrowReleased`
- Contract details: x-audit-event=MarketplaceDealOffered
- Recommended fix: update `microservices/marketplace/contracts/openapi-v1.yaml` to bind `MarketplaceEscrowReleased`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceEscrowReleased`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0100 — `marketplace` — `REST POST /marketplace/listings`
- Priority: `P0`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current class evidence: `MarketplaceDealOffered`
- Missing-event-class-name / should emit: `MarketplaceListingPublished`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDealOffered`
- IP exact-class evidence: `MarketplaceDealOffered`
- Contract details: x-audit-event=MarketplaceDealOffered
- Recommended fix: update `microservices/marketplace/contracts/openapi-v1.yaml` to bind `MarketplaceListingPublished`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceListingPublished`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0101 — `marketplace` — `REST POST /marketplace/escrow/holds`
- Priority: `P0`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current class evidence: `MarketplaceDealOffered`
- Missing-event-class-name / should emit: `MarketplaceEscrowReserved`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDealOffered,MarketplaceEscrowReserved`
- IP exact-class evidence: `MarketplaceDealOffered,MarketplaceEscrowReserved`
- Contract details: x-audit-event=MarketplaceDealOffered
- Recommended fix: update `microservices/marketplace/contracts/openapi-v1.yaml` to bind `MarketplaceEscrowReserved`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceEscrowReserved`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0102 — `marketplace` — `REST POST /marketplace/disputes`
- Priority: `P0`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current class evidence: `MarketplaceDealOffered`
- Missing-event-class-name / should emit: `MarketplaceDisputeOpened`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDealOffered,MarketplaceDisputeOpened`
- IP exact-class evidence: `MarketplaceDealOffered,MarketplaceDisputeOpened`
- Contract details: x-audit-event=MarketplaceDealOffered
- Recommended fix: update `microservices/marketplace/contracts/openapi-v1.yaml` to bind `MarketplaceDisputeOpened`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceDisputeOpened`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0103 — `marketplace` — `REST POST /marketplace/revenue-shares`
- Priority: `P0`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current class evidence: `MarketplaceDealOffered`
- Missing-event-class-name / should emit: `MarketplaceRevenueShareAccrued`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `MarketplaceDealOffered,MarketplaceRevenueShareAccrued`
- IP exact-class evidence: `MarketplaceDealOffered,MarketplaceRevenueShareAccrued`
- Contract details: x-audit-event=MarketplaceDealOffered
- Recommended fix: update `microservices/marketplace/contracts/openapi-v1.yaml` to bind `MarketplaceRevenueShareAccrued`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `MarketplaceRevenueShareAccrued`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0104 — `performance-management` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/performance-management/contracts/asyncapi-v1.yaml`
- Current class evidence: `PerformanceManagementActionAccepted`
- Missing-event-class-name / should emit: `PerformanceManagementActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `PerformanceManagementActionAccepted`
- IP exact-class evidence: `none`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/performance-management/contracts/asyncapi-v1.yaml` to bind `PerformanceManagementActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `PerformanceManagementActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0105 — `performance-management` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/performance-management/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `PerformanceManagementDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/performance-management/contracts/local-asyncapi-v1.yaml` to bind `PerformanceManagementDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `PerformanceManagementDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0106 — `performance-management` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/performance-management/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `PerformanceManagementSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/performance-management/contracts/local-asyncapi-v1.yaml` to bind `PerformanceManagementSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `PerformanceManagementSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0107 — `performance-management` — `REST POST /performance-management/v1/review-cycles/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/performance-management/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `PerformanceManagementReviewCyclePolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/performance-management/contracts/local-openapi-v1.yaml` to bind `PerformanceManagementReviewCyclePolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `PerformanceManagementReviewCyclePolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0108 — `performance-management` — `REST POST /performance-management/v1/review-cycles/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/performance-management/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `PerformanceManagementReviewCycleOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/performance-management/contracts/local-openapi-v1.yaml` to bind `PerformanceManagementReviewCycleOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `PerformanceManagementReviewCycleOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0109 — `performance-management` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/performance-management/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `PerformanceManagementLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/performance-management/contracts/local-operations-v1.proto` to bind `PerformanceManagementLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `PerformanceManagementLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0110 — `performance-management` — `REST POST /performance-management/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/performance-management/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `PerformanceManagementActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `PerformanceManagementActionAccepted`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/performance-management/contracts/openapi-v1.yaml` to bind `PerformanceManagementActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `PerformanceManagementActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0111 — `performance-management` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/performance-management/contracts/performance-management-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `PerformanceManagementActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/performance-management/contracts/performance-management-v1.proto` to bind `PerformanceManagementActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `PerformanceManagementActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0112 — `whiteboard` — `ASYNC SEND publishActionAccepted`
- Priority: `P1`
- Contract file: `microservices/whiteboard/contracts/asyncapi-v1.yaml`
- Current class evidence: `WhiteboardActionAccepted`
- Missing-event-class-name / should emit: `WhiteboardActionAccepted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WhiteboardActionAccepted`
- IP exact-class evidence: `WhiteboardActionAccepted`
- Contract details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id
- Recommended fix: update `microservices/whiteboard/contracts/asyncapi-v1.yaml` to bind `WhiteboardActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WhiteboardActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0113 — `whiteboard` — `ASYNC SEND publishDomainEvent`
- Priority: `P1`
- Contract file: `microservices/whiteboard/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `WhiteboardDomainEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at
- Recommended fix: update `microservices/whiteboard/contracts/local-asyncapi-v1.yaml` to bind `WhiteboardDomainEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WhiteboardDomainEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0114 — `whiteboard` — `ASYNC SEND publishSloBurnEvent`
- Priority: `P1`
- Contract file: `microservices/whiteboard/contracts/local-asyncapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `WhiteboardSloBurnEvent`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event
- Recommended fix: update `microservices/whiteboard/contracts/local-asyncapi-v1.yaml` to bind `WhiteboardSloBurnEvent`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WhiteboardSloBurnEvent`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0115 — `whiteboard` — `REST POST /whiteboard/v1/whiteboard-sessions/{resource_id}/policy-check`
- Priority: `P0`
- Contract file: `microservices/whiteboard/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `WhiteboardSessionPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/whiteboard/contracts/local-openapi-v1.yaml` to bind `WhiteboardSessionPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WhiteboardSessionPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0116 — `whiteboard` — `REST POST /whiteboard/v1/whiteboard-sessions/{resource_id}/operator-remediation`
- Priority: `P0`
- Contract file: `microservices/whiteboard/contracts/local-openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `WhiteboardSessionOperatorRemediationApplied`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/whiteboard/contracts/local-openapi-v1.yaml` to bind `WhiteboardSessionOperatorRemediationApplied`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WhiteboardSessionOperatorRemediationApplied`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0117 — `whiteboard` — `GRPC RPC CheckLocalPolicy`
- Priority: `P0`
- Contract file: `microservices/whiteboard/contracts/local-operations-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `WhiteboardLocalPolicyDecisionRecorded`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/whiteboard/contracts/local-operations-v1.proto` to bind `WhiteboardLocalPolicyDecisionRecorded`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WhiteboardLocalPolicyDecisionRecorded`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0118 — `whiteboard` — `REST POST /whiteboard/actions/{action_id}`
- Priority: `P0`
- Contract file: `microservices/whiteboard/contracts/openapi-v1.yaml`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `WhiteboardActionAccepted`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WhiteboardActionAccepted`
- IP exact-class evidence: `WhiteboardActionAccepted`
- Contract details: no x-audit-event class
- Recommended fix: update `microservices/whiteboard/contracts/openapi-v1.yaml` to bind `WhiteboardActionAccepted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WhiteboardActionAccepted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0119 — `whiteboard` — `GRPC RPC InvokeAction`
- Priority: `P0`
- Contract file: `microservices/whiteboard/contracts/whiteboard-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `WhiteboardActionInvoked`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `none`
- IP exact-class evidence: `none`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/whiteboard/contracts/whiteboard-v1.proto` to bind `WhiteboardActionInvoked`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WhiteboardActionInvoked`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0120 — `workplace-integration` — `ASYNC SEND publishWorkplaceESignSessionCreated`
- Priority: `P1`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current class evidence: `WorkplaceESignSessionCreated`
- Missing-event-class-name / should emit: `WorkplaceESignSessionCreated`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceESignSessionCreated`
- IP exact-class evidence: `WorkplaceESignSessionCreated`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/workplace-integration/contracts/asyncapi-v1.yaml` to bind `WorkplaceESignSessionCreated`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceESignSessionCreated`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0121 — `workplace-integration` — `ASYNC SEND publishWorkplaceSignatureCaptured`
- Priority: `P1`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current class evidence: `WorkplaceSignatureCaptured`
- Missing-event-class-name / should emit: `WorkplaceSignatureCaptured`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceSignatureCaptured`
- IP exact-class evidence: `WorkplaceSignatureCaptured`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/workplace-integration/contracts/asyncapi-v1.yaml` to bind `WorkplaceSignatureCaptured`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceSignatureCaptured`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0122 — `workplace-integration` — `ASYNC SEND publishWorkplaceOfferGenerated`
- Priority: `P1`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current class evidence: `WorkplaceOfferGenerated`
- Missing-event-class-name / should emit: `WorkplaceOfferGenerated`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceOfferGenerated`
- IP exact-class evidence: `WorkplaceOfferGenerated`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/workplace-integration/contracts/asyncapi-v1.yaml` to bind `WorkplaceOfferGenerated`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceOfferGenerated`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0123 — `workplace-integration` — `ASYNC SEND publishWorkplaceAgreementBound`
- Priority: `P1`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current class evidence: `WorkplaceAgreementBound`
- Missing-event-class-name / should emit: `WorkplaceAgreementBound`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceAgreementBound`
- IP exact-class evidence: `WorkplaceAgreementBound`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/workplace-integration/contracts/asyncapi-v1.yaml` to bind `WorkplaceAgreementBound`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceAgreementBound`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0124 — `workplace-integration` — `ASYNC SEND publishWorkplaceRosterBindingGranted`
- Priority: `P1`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current class evidence: `WorkplaceRosterBindingGranted`
- Missing-event-class-name / should emit: `WorkplaceRosterBindingGranted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceRosterBindingGranted`
- IP exact-class evidence: `WorkplaceRosterBindingGranted`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/workplace-integration/contracts/asyncapi-v1.yaml` to bind `WorkplaceRosterBindingGranted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceRosterBindingGranted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0125 — `workplace-integration` — `ASYNC SEND publishWorkplaceClockEventAttested`
- Priority: `P1`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current class evidence: `WorkplaceClockEventAttested`
- Missing-event-class-name / should emit: `WorkplaceClockEventAttested`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceClockEventAttested`
- IP exact-class evidence: `WorkplaceClockEventAttested`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/workplace-integration/contracts/asyncapi-v1.yaml` to bind `WorkplaceClockEventAttested`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceClockEventAttested`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0126 — `workplace-integration` — `ASYNC SEND publishWorkplaceDlpTraceSealed`
- Priority: `P1`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current class evidence: `WorkplaceDlpTraceSealed`
- Missing-event-class-name / should emit: `WorkplaceDlpTraceSealed`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceDlpTraceSealed`
- IP exact-class evidence: `WorkplaceDlpTraceSealed`
- Contract details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref
- Recommended fix: update `microservices/workplace-integration/contracts/asyncapi-v1.yaml` to bind `WorkplaceDlpTraceSealed`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceDlpTraceSealed`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0127 — `workplace-integration` — `REST POST /workplace/esign/sessions`
- Priority: `P1`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current class evidence: `WorkplaceESignSessionCreated`
- Missing-event-class-name / should emit: `WorkplaceESignSessionCreated`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceESignSessionCreated`
- IP exact-class evidence: `WorkplaceESignSessionCreated`
- Contract details: x-audit-event=WorkplaceESignSessionCreated
- Recommended fix: update `microservices/workplace-integration/contracts/openapi-v1.yaml` to bind `WorkplaceESignSessionCreated`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceESignSessionCreated`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0128 — `workplace-integration` — `REST POST /workplace/esign/sessions/{session_id}/sign`
- Priority: `P0`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current class evidence: `WorkplaceESignSessionCreated`
- Missing-event-class-name / should emit: `WorkplaceSignatureCaptured`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceESignSessionCreated,WorkplaceSignatureCaptured`
- IP exact-class evidence: `WorkplaceESignSessionCreated,WorkplaceSignatureCaptured`
- Contract details: x-audit-event=WorkplaceESignSessionCreated
- Recommended fix: update `microservices/workplace-integration/contracts/openapi-v1.yaml` to bind `WorkplaceSignatureCaptured`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceSignatureCaptured`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0129 — `workplace-integration` — `REST POST /workplace/offer-letters`
- Priority: `P0`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current class evidence: `WorkplaceESignSessionCreated`
- Missing-event-class-name / should emit: `WorkplaceOfferGenerated`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceESignSessionCreated,WorkplaceOfferGenerated`
- IP exact-class evidence: `WorkplaceESignSessionCreated,WorkplaceOfferGenerated`
- Contract details: x-audit-event=WorkplaceESignSessionCreated
- Recommended fix: update `microservices/workplace-integration/contracts/openapi-v1.yaml` to bind `WorkplaceOfferGenerated`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceOfferGenerated`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0130 — `workplace-integration` — `REST POST /workplace/engagement-agreements`
- Priority: `P0`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current class evidence: `WorkplaceESignSessionCreated`
- Missing-event-class-name / should emit: `WorkplaceAgreementBound`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceAgreementBound,WorkplaceESignSessionCreated`
- IP exact-class evidence: `WorkplaceAgreementBound,WorkplaceESignSessionCreated`
- Contract details: x-audit-event=WorkplaceESignSessionCreated
- Recommended fix: update `microservices/workplace-integration/contracts/openapi-v1.yaml` to bind `WorkplaceAgreementBound`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceAgreementBound`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0131 — `workplace-integration` — `REST POST /workplace/roster-bindings`
- Priority: `P0`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current class evidence: `WorkplaceESignSessionCreated`
- Missing-event-class-name / should emit: `WorkplaceRosterBindingGranted`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceESignSessionCreated,WorkplaceRosterBindingGranted`
- IP exact-class evidence: `WorkplaceESignSessionCreated,WorkplaceRosterBindingGranted`
- Contract details: x-audit-event=WorkplaceESignSessionCreated
- Recommended fix: update `microservices/workplace-integration/contracts/openapi-v1.yaml` to bind `WorkplaceRosterBindingGranted`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceRosterBindingGranted`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0132 — `workplace-integration` — `REST POST /workplace/clock-events`
- Priority: `P0`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current class evidence: `WorkplaceESignSessionCreated`
- Missing-event-class-name / should emit: `WorkplaceClockEventAttested`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceClockEventAttested,WorkplaceESignSessionCreated`
- IP exact-class evidence: `WorkplaceClockEventAttested,WorkplaceESignSessionCreated`
- Contract details: x-audit-event=WorkplaceESignSessionCreated
- Recommended fix: update `microservices/workplace-integration/contracts/openapi-v1.yaml` to bind `WorkplaceClockEventAttested`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceClockEventAttested`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0133 — `workplace-integration` — `REST POST /workplace/dlp-traces`
- Priority: `P0`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current class evidence: `WorkplaceESignSessionCreated`
- Missing-event-class-name / should emit: `WorkplaceDlpTraceSealed`
- Gap reasons: `UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceDlpTraceSealed,WorkplaceESignSessionCreated`
- IP exact-class evidence: `WorkplaceDlpTraceSealed,WorkplaceESignSessionCreated`
- Contract details: x-audit-event=WorkplaceESignSessionCreated
- Recommended fix: update `microservices/workplace-integration/contracts/openapi-v1.yaml` to bind `WorkplaceDlpTraceSealed`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceDlpTraceSealed`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

#### GAP-0134 — `workplace-integration` — `GRPC RPC SubmitWorkplaceAgreement`
- Priority: `P0`
- Contract file: `microservices/workplace-integration/contracts/workplace-integration-v1.proto`
- Current class evidence: `NONE`
- Missing-event-class-name / should emit: `WorkplaceAgreementBound`
- Gap reasons: `MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`
- AsyncAPI class evidence: `WorkplaceAgreementBound`
- IP exact-class evidence: `WorkplaceAgreementBound`
- Contract details: no nearby audit-event class comment/option
- Recommended fix: update `microservices/workplace-integration/contracts/workplace-integration-v1.proto` to bind `WorkplaceAgreementBound`, update the relevant IP slice to require the ADR-0263 envelope, and register the class through ADR-0263 registry or successor registry artifact.
- Stop condition for fix wave: endpoint contract names `WorkplaceAgreementBound`, AsyncAPI/IP evidence contains the same class, and `oya gate validate audit-event-class-registered` recognizes the class.

## §4 — Recommended P0/P1/P2 priority fixes (top 30)

| rank | priority | µservice | endpoint | target class | why this is in the top 30 |
|---:|---|---|---|---|---|
| 1 | P0 | `contact-center` | `REST POST /contact-center/actions/{action_id}` | `ContactCenterActionAccepted` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 2 | P0 | `contact-center` | `REST POST /contact-center/v1/interactions/{resource_id}/operator-remediation` | `ContactCenterInteractionOperatorRemediationApplied` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 3 | P0 | `contact-center` | `REST POST /contact-center/v1/interactions/{resource_id}/policy-check` | `ContactCenterInteractionPolicyDecisionRecorded` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 4 | P0 | `contract-lifecycle-management` | `REST POST /contract-lifecycle-management/actions/{action_id}` | `ContractLifecycleManagementActionAccepted` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 5 | P0 | `contract-lifecycle-management` | `REST POST /contract-lifecycle-management/v1/contract-workspaces/{resource_id}/operator-remediation` | `ContractLifecycleManagementContractWorkspaceOperatorRemediationApplied` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 6 | P0 | `contract-lifecycle-management` | `REST POST /contract-lifecycle-management/v1/contract-workspaces/{resource_id}/policy-check` | `ContractLifecycleManagementContractWorkspacePolicyDecisionRecorded` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 7 | P0 | `data-pipeline` | `REST POST /data-pipeline/actions/{action_id}` | `DataPipelineActionAccepted` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 8 | P0 | `data-pipeline` | `REST POST /data-pipeline/v1/pipeline-runs/{resource_id}/operator-remediation` | `DataPipelinePipelineRunOperatorRemediationApplied` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 9 | P0 | `data-pipeline` | `REST POST /data-pipeline/v1/pipeline-runs/{resource_id}/policy-check` | `DataPipelinePipelineRunPolicyDecisionRecorded` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 10 | P0 | `data-warehouse` | `REST POST /data-warehouse/actions/{action_id}` | `DataWarehouseActionAccepted` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 11 | P0 | `data-warehouse` | `REST POST /data-warehouse/v1/warehouse-datasets/{resource_id}/operator-remediation` | `DataWarehouseWarehouseDatasetOperatorRemediationApplied` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 12 | P0 | `data-warehouse` | `REST POST /data-warehouse/v1/warehouse-datasets/{resource_id}/policy-check` | `DataWarehouseWarehouseDatasetPolicyDecisionRecorded` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 13 | P0 | `design-collaboration` | `REST POST /design-collaboration/actions/{action_id}` | `DesignCollaborationActionAccepted` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 14 | P0 | `design-collaboration` | `REST POST /design-collaboration/v1/design-files/{resource_id}/operator-remediation` | `DesignCollaborationDesignFileOperatorRemediationApplied` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 15 | P0 | `design-collaboration` | `REST POST /design-collaboration/v1/design-files/{resource_id}/policy-check` | `DesignCollaborationDesignFilePolicyDecisionRecorded` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 16 | P0 | `financial-planning` | `REST POST /financial-planning/actions/{action_id}` | `FinancialPlanningActionAccepted` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 17 | P0 | `financial-planning` | `REST POST /financial-planning/v1/planning-cycles/{resource_id}/operator-remediation` | `FinancialPlanningPlanningCycleOperatorRemediationApplied` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 18 | P0 | `financial-planning` | `REST POST /financial-planning/v1/planning-cycles/{resource_id}/policy-check` | `FinancialPlanningPlanningCyclePolicyDecisionRecorded` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 19 | P0 | `healthcare-integration` | `REST POST /healthcare-integration/actions/{action_id}` | `HealthcareIntegrationActionAccepted` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 20 | P0 | `healthcare-integration` | `REST POST /healthcare-integration/v1/clinical-exchanges/{resource_id}/operator-remediation` | `HealthcareIntegrationClinicalExchangeOperatorRemediationApplied` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 21 | P0 | `healthcare-integration` | `REST POST /healthcare-integration/v1/clinical-exchanges/{resource_id}/policy-check` | `HealthcareIntegrationClinicalExchangePolicyDecisionRecorded` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 22 | P0 | `incident-management` | `REST POST /incident-management/actions/{action_id}` | `IncidentManagementActionAccepted` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 23 | P0 | `incident-management` | `REST POST /incident-management/v1/incident-commands/{resource_id}/operator-remediation` | `IncidentManagementIncidentCommandOperatorRemediationApplied` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 24 | P0 | `incident-management` | `REST POST /incident-management/v1/incident-commands/{resource_id}/policy-check` | `IncidentManagementIncidentCommandPolicyDecisionRecorded` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 25 | P0 | `itsm` | `REST POST /itsm/actions/{action_id}` | `ITSMActionAccepted` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 26 | P0 | `itsm` | `REST POST /itsm/v1/service-records/{resource_id}/operator-remediation` | `ITSMServiceRecordOperatorRemediationApplied` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 27 | P0 | `itsm` | `REST POST /itsm/v1/service-records/{resource_id}/policy-check` | `ITSMServiceRecordPolicyDecisionRecorded` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 28 | P0 | `learning-management` | `REST POST /learning-management/actions/{action_id}` | `LearningManagementActionAccepted` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 29 | P0 | `learning-management` | `REST POST /learning-management/v1/learning-cohorts/{resource_id}/operator-remediation` | `LearningManagementLearningCohortOperatorRemediationApplied` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |
| 30 | P0 | `learning-management` | `REST POST /learning-management/v1/learning-cohorts/{resource_id}/policy-check` | `LearningManagementLearningCohortPolicyDecisionRecorded` | REST/gRPC state change lacks an endpoint-correct registered audit-event binding. |

### §4.1 Priority rationale

1. P0 covers REST and gRPC mutation surfaces where the contract currently has no class or a semantically wrong class.
2. P1 covers publish surfaces with concrete but unregistered names, generic publish names, or unconstrained `audit_event_class` fields.
3. P2 is reserved for documentation-only cleanup after contract and registry bindings land; this strict corpus has direct contract gaps instead.
4. The top 30 favors command entry points before downstream publish-only evidence because ADR-0263 ties observability to the committed state change.

## §5 — Cross-reference to existing IPs/ADRs/contracts that need to land the fix

### §5.1 Global ADR and registry cross-references

- `docs/decisions/ADR-0706-observability-live-apex.md` — registry contract and mandatory audit envelope; update it or point it at the successor registry when new service-specific classes land.
- `microservices/audit-chain/policy/event-class-registry/` — referenced by ADR-0263 but absent in this checkout; create it or replace it with the canonical successor registry before claiming registered-class compliance.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md` — current source for abuse-defence registered classes.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md` — current source for conglomerate registered classes.
- `docs/decisions/ADR-0709-general-live-apex.md` — current source for office/information-barrier registered classes.

### §5.2 Service-level fix ledger

#### `contact-center`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/contact-center/policies/local-callback-window-enforcement.cedar`, `microservices/contact-center/policies/local-emergency-caller-bypass.cedar`, `microservices/contact-center/policies/local-omnichannel-routing-scope.cedar`, `microservices/contact-center/policies/local-queue-rebalance-control.cedar`, `microservices/contact-center/policies/local-recording-consent-access.cedar`, `microservices/contact-center/policies/local-voice-transfer-authorization.cedar`
- Contract files in scope: `microservices/contact-center/contracts/asyncapi-v1.yaml`, `microservices/contact-center/contracts/contact-center-v1.proto`, `microservices/contact-center/contracts/local-asyncapi-v1.yaml`, `microservices/contact-center/contracts/local-openapi-v1.yaml`, `microservices/contact-center/contracts/local-operations-v1.proto`, `microservices/contact-center/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/contact-center/IP-001-tenant-scope-kernel.md`, `microservices/contact-center/IP-002-cedar-default-deny.md`, `microservices/contact-center/IP-003-ontology-projection.md`, `microservices/contact-center/IP-004-workflow-template-library.md`, `microservices/contact-center/IP-005-rest-contract-surface.md`, `microservices/contact-center/IP-006-async-event-surface.md`, `microservices/contact-center/IP-007-grpc-internal-surface.md`, `microservices/contact-center/IP-008-policy-eval-library-binding.md`, `microservices/contact-center/IP-009-credential-sidecar-binding.md`, `microservices/contact-center/IP-010-multi-region-cell-layout.md`, `microservices/contact-center/IP-011-observability-audit-events.md`, `microservices/contact-center/IP-012-abuse-defence-edge-waf.md`, `microservices/contact-center/IP-013-emergency-services-bypass.md`, `microservices/contact-center/IP-014-marketplace-dealset-settlement.md`, `microservices/contact-center/IP-015-data-residency-pack-overlays.md`, `microservices/contact-center/IP-016-backfill-replay-worker.md`, `microservices/contact-center/IP-017-cost-budget-enforcer.md`, `microservices/contact-center/IP-018-capacity-admission-control.md`, `microservices/contact-center/IP-019-sdk-client-generation.md`, `microservices/contact-center/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `ContactCenterActionAccepted`, `ContactCenterActionInvoked`, `ContactCenterDomainEvent`, `ContactCenterInteractionOperatorRemediationApplied`, `ContactCenterInteractionPolicyDecisionRecorded`, `ContactCenterLocalPolicyDecisionRecorded`, `ContactCenterSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `ContactCenterActionAccepted`; current=`ContactCenterActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contact-center/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `ContactCenterActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contact-center/contracts/contact-center-v1.proto`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `ContactCenterDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contact-center/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `ContactCenterSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contact-center/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /contact-center/v1/interactions/{resource_id}/policy-check` -> `ContactCenterInteractionPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contact-center/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /contact-center/v1/interactions/{resource_id}/operator-remediation` -> `ContactCenterInteractionOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contact-center/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `ContactCenterLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contact-center/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /contact-center/actions/{action_id}` -> `ContactCenterActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contact-center/contracts/openapi-v1.yaml`.

#### `contract-lifecycle-management`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/contract-lifecycle-management/policies/local-clause-policy-evaluation.cedar`, `microservices/contract-lifecycle-management/policies/local-draft-create-scope.cedar`, `microservices/contract-lifecycle-management/policies/local-obligation-extract-egress.cedar`, `microservices/contract-lifecycle-management/policies/local-redline-thread-access.cedar`, `microservices/contract-lifecycle-management/policies/local-renewal-notice-control.cedar`, `microservices/contract-lifecycle-management/policies/local-signature-packet-approval.cedar`
- Contract files in scope: `microservices/contract-lifecycle-management/contracts/asyncapi-v1.yaml`, `microservices/contract-lifecycle-management/contracts/contract-lifecycle-management-v1.proto`, `microservices/contract-lifecycle-management/contracts/local-asyncapi-v1.yaml`, `microservices/contract-lifecycle-management/contracts/local-openapi-v1.yaml`, `microservices/contract-lifecycle-management/contracts/local-operations-v1.proto`, `microservices/contract-lifecycle-management/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/contract-lifecycle-management/IP-001-tenant-scope-kernel.md`, `microservices/contract-lifecycle-management/IP-002-cedar-default-deny.md`, `microservices/contract-lifecycle-management/IP-003-ontology-projection.md`, `microservices/contract-lifecycle-management/IP-004-workflow-template-library.md`, `microservices/contract-lifecycle-management/IP-005-rest-contract-surface.md`, `microservices/contract-lifecycle-management/IP-006-async-event-surface.md`, `microservices/contract-lifecycle-management/IP-007-grpc-internal-surface.md`, `microservices/contract-lifecycle-management/IP-008-policy-eval-library-binding.md`, `microservices/contract-lifecycle-management/IP-009-credential-sidecar-binding.md`, `microservices/contract-lifecycle-management/IP-010-multi-region-cell-layout.md`, `microservices/contract-lifecycle-management/IP-011-observability-audit-events.md`, `microservices/contract-lifecycle-management/IP-012-abuse-defence-edge-waf.md`, `microservices/contract-lifecycle-management/IP-013-emergency-services-bypass.md`, `microservices/contract-lifecycle-management/IP-014-marketplace-dealset-settlement.md`, `microservices/contract-lifecycle-management/IP-015-data-residency-pack-overlays.md`, `microservices/contract-lifecycle-management/IP-016-backfill-replay-worker.md`, `microservices/contract-lifecycle-management/IP-017-cost-budget-enforcer.md`, `microservices/contract-lifecycle-management/IP-018-capacity-admission-control.md`, `microservices/contract-lifecycle-management/IP-019-sdk-client-generation.md`, `microservices/contract-lifecycle-management/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `ContractLifecycleManagementActionAccepted`, `ContractLifecycleManagementActionInvoked`, `ContractLifecycleManagementContractWorkspaceOperatorRemediationApplied`, `ContractLifecycleManagementContractWorkspacePolicyDecisionRecorded`, `ContractLifecycleManagementDomainEvent`, `ContractLifecycleManagementLocalPolicyDecisionRecorded`, `ContractLifecycleManagementSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `ContractLifecycleManagementActionAccepted`; current=`ContractLifecycleManagementActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contract-lifecycle-management/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `ContractLifecycleManagementActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contract-lifecycle-management/contracts/contract-lifecycle-management-v1.proto`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `ContractLifecycleManagementDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contract-lifecycle-management/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `ContractLifecycleManagementSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contract-lifecycle-management/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /contract-lifecycle-management/v1/contract-workspaces/{resource_id}/policy-check` -> `ContractLifecycleManagementContractWorkspacePolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contract-lifecycle-management/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /contract-lifecycle-management/v1/contract-workspaces/{resource_id}/operator-remediation` -> `ContractLifecycleManagementContractWorkspaceOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contract-lifecycle-management/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `ContractLifecycleManagementLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contract-lifecycle-management/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /contract-lifecycle-management/actions/{action_id}` -> `ContractLifecycleManagementActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/contract-lifecycle-management/contracts/openapi-v1.yaml`.

#### `data-pipeline`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/data-pipeline/policies/local-deadletter-replay-approval.cedar`, `microservices/data-pipeline/policies/local-ingest-source-scope.cedar`, `microservices/data-pipeline/policies/local-lineage-record-egress.cedar`, `microservices/data-pipeline/policies/local-null-rate-quarantine.cedar`, `microservices/data-pipeline/policies/local-quality-threshold-enforcement.cedar`, `microservices/data-pipeline/policies/local-transform-run-control.cedar`
- Contract files in scope: `microservices/data-pipeline/contracts/asyncapi-v1.yaml`, `microservices/data-pipeline/contracts/data-pipeline-v1.proto`, `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`, `microservices/data-pipeline/contracts/local-openapi-v1.yaml`, `microservices/data-pipeline/contracts/local-operations-v1.proto`, `microservices/data-pipeline/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/data-pipeline/IP-001-tenant-scope-kernel.md`, `microservices/data-pipeline/IP-002-cedar-default-deny.md`, `microservices/data-pipeline/IP-003-ontology-projection.md`, `microservices/data-pipeline/IP-004-workflow-template-library.md`, `microservices/data-pipeline/IP-005-rest-contract-surface.md`, `microservices/data-pipeline/IP-006-async-event-surface.md`, `microservices/data-pipeline/IP-007-grpc-internal-surface.md`, `microservices/data-pipeline/IP-008-policy-eval-library-binding.md`, `microservices/data-pipeline/IP-009-credential-sidecar-binding.md`, `microservices/data-pipeline/IP-010-multi-region-cell-layout.md`, `microservices/data-pipeline/IP-011-observability-audit-events.md`, `microservices/data-pipeline/IP-012-abuse-defence-edge-waf.md`, `microservices/data-pipeline/IP-013-emergency-services-bypass.md`, `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md`, `microservices/data-pipeline/IP-015-data-residency-pack-overlays.md`, `microservices/data-pipeline/IP-016-backfill-replay-worker.md`, `microservices/data-pipeline/IP-017-cost-budget-enforcer.md`, `microservices/data-pipeline/IP-018-capacity-admission-control.md`, `microservices/data-pipeline/IP-019-sdk-client-generation.md`, `microservices/data-pipeline/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `DataPipelineActionAccepted`, `DataPipelineActionInvoked`, `DataPipelineDomainEvent`, `DataPipelineLocalPolicyDecisionRecorded`, `DataPipelinePipelineRunOperatorRemediationApplied`, `DataPipelinePipelineRunPolicyDecisionRecorded`, `DataPipelineSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `DataPipelineActionAccepted`; current=`DataPipelineActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-pipeline/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `DataPipelineActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-pipeline/contracts/data-pipeline-v1.proto`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `DataPipelineDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `DataPipelineSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /data-pipeline/v1/pipeline-runs/{resource_id}/policy-check` -> `DataPipelinePipelineRunPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-pipeline/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /data-pipeline/v1/pipeline-runs/{resource_id}/operator-remediation` -> `DataPipelinePipelineRunOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-pipeline/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `DataPipelineLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-pipeline/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /data-pipeline/actions/{action_id}` -> `DataPipelineActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-pipeline/contracts/openapi-v1.yaml`.

#### `data-warehouse`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/data-warehouse/policies/local-completeness-threshold-guard.cedar`, `microservices/data-warehouse/policies/local-freshness-tier-control.cedar`, `microservices/data-warehouse/policies/local-lineage-export-egress.cedar`, `microservices/data-warehouse/policies/local-pipeline-sla-tier-scope.cedar`, `microservices/data-warehouse/policies/local-schema-change-approval.cedar`, `microservices/data-warehouse/policies/local-warehouse-query-access.cedar`
- Contract files in scope: `microservices/data-warehouse/contracts/asyncapi-v1.yaml`, `microservices/data-warehouse/contracts/data-warehouse-v1.proto`, `microservices/data-warehouse/contracts/local-asyncapi-v1.yaml`, `microservices/data-warehouse/contracts/local-openapi-v1.yaml`, `microservices/data-warehouse/contracts/local-operations-v1.proto`, `microservices/data-warehouse/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/data-warehouse/IP-001-tenant-scope-kernel.md`, `microservices/data-warehouse/IP-002-cedar-default-deny.md`, `microservices/data-warehouse/IP-003-ontology-projection.md`, `microservices/data-warehouse/IP-004-workflow-template-library.md`, `microservices/data-warehouse/IP-005-rest-contract-surface.md`, `microservices/data-warehouse/IP-006-async-event-surface.md`, `microservices/data-warehouse/IP-007-grpc-internal-surface.md`, `microservices/data-warehouse/IP-008-policy-eval-library-binding.md`, `microservices/data-warehouse/IP-009-credential-sidecar-binding.md`, `microservices/data-warehouse/IP-010-multi-region-cell-layout.md`, `microservices/data-warehouse/IP-011-observability-audit-events.md`, `microservices/data-warehouse/IP-012-abuse-defence-edge-waf.md`, `microservices/data-warehouse/IP-013-emergency-services-bypass.md`, `microservices/data-warehouse/IP-014-marketplace-dealset-settlement.md`, `microservices/data-warehouse/IP-015-data-residency-pack-overlays.md`, `microservices/data-warehouse/IP-016-backfill-replay-worker.md`, `microservices/data-warehouse/IP-017-cost-budget-enforcer.md`, `microservices/data-warehouse/IP-018-capacity-admission-control.md`, `microservices/data-warehouse/IP-019-sdk-client-generation.md`, `microservices/data-warehouse/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `DataWarehouseActionAccepted`, `DataWarehouseActionInvoked`, `DataWarehouseDomainEvent`, `DataWarehouseLocalPolicyDecisionRecorded`, `DataWarehouseSloBurnEvent`, `DataWarehouseWarehouseDatasetOperatorRemediationApplied`, `DataWarehouseWarehouseDatasetPolicyDecisionRecorded`
- Fix item: `ASYNC SEND publishActionAccepted` -> `DataWarehouseActionAccepted`; current=`DataWarehouseActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-warehouse/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `DataWarehouseActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-warehouse/contracts/data-warehouse-v1.proto`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `DataWarehouseDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-warehouse/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `DataWarehouseSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-warehouse/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /data-warehouse/v1/warehouse-datasets/{resource_id}/policy-check` -> `DataWarehouseWarehouseDatasetPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-warehouse/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /data-warehouse/v1/warehouse-datasets/{resource_id}/operator-remediation` -> `DataWarehouseWarehouseDatasetOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-warehouse/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `DataWarehouseLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-warehouse/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /data-warehouse/actions/{action_id}` -> `DataWarehouseActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/data-warehouse/contracts/openapi-v1.yaml`.

#### `design-collaboration`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/design-collaboration/policies/local-asset-preview-egress.cedar`, `microservices/design-collaboration/policies/local-comment-thread-scope.cedar`, `microservices/design-collaboration/policies/local-file-open-entitlement.cedar`, `microservices/design-collaboration/policies/local-handoff-export-approval.cedar`, `microservices/design-collaboration/policies/local-permission-check-default-deny.cedar`, `microservices/design-collaboration/policies/local-version-save-control.cedar`
- Contract files in scope: `microservices/design-collaboration/contracts/asyncapi-v1.yaml`, `microservices/design-collaboration/contracts/design-collaboration-v1.proto`, `microservices/design-collaboration/contracts/local-asyncapi-v1.yaml`, `microservices/design-collaboration/contracts/local-openapi-v1.yaml`, `microservices/design-collaboration/contracts/local-operations-v1.proto`, `microservices/design-collaboration/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/design-collaboration/IP-001-tenant-scope-kernel.md`, `microservices/design-collaboration/IP-002-cedar-default-deny.md`, `microservices/design-collaboration/IP-003-ontology-projection.md`, `microservices/design-collaboration/IP-004-workflow-template-library.md`, `microservices/design-collaboration/IP-005-rest-contract-surface.md`, `microservices/design-collaboration/IP-006-async-event-surface.md`, `microservices/design-collaboration/IP-007-grpc-internal-surface.md`, `microservices/design-collaboration/IP-008-policy-eval-library-binding.md`, `microservices/design-collaboration/IP-009-credential-sidecar-binding.md`, `microservices/design-collaboration/IP-010-multi-region-cell-layout.md`, `microservices/design-collaboration/IP-011-observability-audit-events.md`, `microservices/design-collaboration/IP-012-abuse-defence-edge-waf.md`, `microservices/design-collaboration/IP-013-emergency-services-bypass.md`, `microservices/design-collaboration/IP-014-marketplace-dealset-settlement.md`, `microservices/design-collaboration/IP-015-data-residency-pack-overlays.md`, `microservices/design-collaboration/IP-016-backfill-replay-worker.md`, `microservices/design-collaboration/IP-017-cost-budget-enforcer.md`, `microservices/design-collaboration/IP-018-capacity-admission-control.md`, `microservices/design-collaboration/IP-019-sdk-client-generation.md`, `microservices/design-collaboration/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `DesignCollaborationActionAccepted`, `DesignCollaborationActionInvoked`, `DesignCollaborationDesignFileOperatorRemediationApplied`, `DesignCollaborationDesignFilePolicyDecisionRecorded`, `DesignCollaborationDomainEvent`, `DesignCollaborationLocalPolicyDecisionRecorded`, `DesignCollaborationSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `DesignCollaborationActionAccepted`; current=`DesignCollaborationActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/design-collaboration/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `DesignCollaborationActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/design-collaboration/contracts/design-collaboration-v1.proto`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `DesignCollaborationDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/design-collaboration/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `DesignCollaborationSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/design-collaboration/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /design-collaboration/v1/design-files/{resource_id}/policy-check` -> `DesignCollaborationDesignFilePolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/design-collaboration/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /design-collaboration/v1/design-files/{resource_id}/operator-remediation` -> `DesignCollaborationDesignFileOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/design-collaboration/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `DesignCollaborationLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/design-collaboration/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /design-collaboration/actions/{action_id}` -> `DesignCollaborationActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/design-collaboration/contracts/openapi-v1.yaml`.

#### `financial-planning`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/financial-planning/policies/local-board-report-seal-egress.cedar`, `microservices/financial-planning/policies/local-budget-lock-control.cedar`, `microservices/financial-planning/policies/local-close-cycle-advance.cedar`, `microservices/financial-planning/policies/local-forecast-version-scope.cedar`, `microservices/financial-planning/policies/local-fx-rate-backfill-guard.cedar`, `microservices/financial-planning/policies/local-variance-explanation-approval.cedar`
- Contract files in scope: `microservices/financial-planning/contracts/asyncapi-v1.yaml`, `microservices/financial-planning/contracts/financial-planning-v1.proto`, `microservices/financial-planning/contracts/local-asyncapi-v1.yaml`, `microservices/financial-planning/contracts/local-openapi-v1.yaml`, `microservices/financial-planning/contracts/local-operations-v1.proto`, `microservices/financial-planning/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/financial-planning/IP-001-tenant-scope-kernel.md`, `microservices/financial-planning/IP-002-cedar-default-deny.md`, `microservices/financial-planning/IP-003-ontology-projection.md`, `microservices/financial-planning/IP-004-workflow-template-library.md`, `microservices/financial-planning/IP-005-rest-contract-surface.md`, `microservices/financial-planning/IP-006-async-event-surface.md`, `microservices/financial-planning/IP-007-grpc-internal-surface.md`, `microservices/financial-planning/IP-008-policy-eval-library-binding.md`, `microservices/financial-planning/IP-009-credential-sidecar-binding.md`, `microservices/financial-planning/IP-010-multi-region-cell-layout.md`, `microservices/financial-planning/IP-011-observability-audit-events.md`, `microservices/financial-planning/IP-012-abuse-defence-edge-waf.md`, `microservices/financial-planning/IP-013-emergency-services-bypass.md`, `microservices/financial-planning/IP-014-marketplace-dealset-settlement.md`, `microservices/financial-planning/IP-015-data-residency-pack-overlays.md`, `microservices/financial-planning/IP-016-backfill-replay-worker.md`, `microservices/financial-planning/IP-017-cost-budget-enforcer.md`, `microservices/financial-planning/IP-018-capacity-admission-control.md`, `microservices/financial-planning/IP-019-sdk-client-generation.md`, `microservices/financial-planning/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `FinancialPlanningActionAccepted`, `FinancialPlanningActionInvoked`, `FinancialPlanningDomainEvent`, `FinancialPlanningLocalPolicyDecisionRecorded`, `FinancialPlanningPlanningCycleOperatorRemediationApplied`, `FinancialPlanningPlanningCyclePolicyDecisionRecorded`, `FinancialPlanningSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `FinancialPlanningActionAccepted`; current=`FinancialPlanningActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/financial-planning/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `FinancialPlanningActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/financial-planning/contracts/financial-planning-v1.proto`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `FinancialPlanningDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/financial-planning/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `FinancialPlanningSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/financial-planning/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /financial-planning/v1/planning-cycles/{resource_id}/policy-check` -> `FinancialPlanningPlanningCyclePolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/financial-planning/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /financial-planning/v1/planning-cycles/{resource_id}/operator-remediation` -> `FinancialPlanningPlanningCycleOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/financial-planning/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `FinancialPlanningLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/financial-planning/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /financial-planning/actions/{action_id}` -> `FinancialPlanningActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/financial-planning/contracts/openapi-v1.yaml`.

#### `healthcare-integration`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/healthcare-integration/policies/local-breakglass-access-control.cedar`, `microservices/healthcare-integration/policies/local-fhir-exchange-consent.cedar`, `microservices/healthcare-integration/policies/local-hipaa-audit-completeness.cedar`, `microservices/healthcare-integration/policies/local-hl7-ingest-source-scope.cedar`, `microservices/healthcare-integration/policies/local-patient-consent-sync.cedar`, `microservices/healthcare-integration/policies/local-phi-delivery-authorization.cedar`
- Contract files in scope: `microservices/healthcare-integration/contracts/asyncapi-v1.yaml`, `microservices/healthcare-integration/contracts/healthcare-integration-v1.proto`, `microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml`, `microservices/healthcare-integration/contracts/local-openapi-v1.yaml`, `microservices/healthcare-integration/contracts/local-operations-v1.proto`, `microservices/healthcare-integration/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/healthcare-integration/IP-001-tenant-scope-kernel.md`, `microservices/healthcare-integration/IP-002-cedar-default-deny.md`, `microservices/healthcare-integration/IP-003-ontology-projection.md`, `microservices/healthcare-integration/IP-004-workflow-template-library.md`, `microservices/healthcare-integration/IP-005-rest-contract-surface.md`, `microservices/healthcare-integration/IP-006-async-event-surface.md`, `microservices/healthcare-integration/IP-007-grpc-internal-surface.md`, `microservices/healthcare-integration/IP-008-policy-eval-library-binding.md`, `microservices/healthcare-integration/IP-009-credential-sidecar-binding.md`, `microservices/healthcare-integration/IP-010-multi-region-cell-layout.md`, `microservices/healthcare-integration/IP-011-observability-audit-events.md`, `microservices/healthcare-integration/IP-012-abuse-defence-edge-waf.md`, `microservices/healthcare-integration/IP-013-emergency-services-bypass.md`, `microservices/healthcare-integration/IP-014-marketplace-dealset-settlement.md`, `microservices/healthcare-integration/IP-015-data-residency-pack-overlays.md`, `microservices/healthcare-integration/IP-016-backfill-replay-worker.md`, `microservices/healthcare-integration/IP-017-cost-budget-enforcer.md`, `microservices/healthcare-integration/IP-018-capacity-admission-control.md`, `microservices/healthcare-integration/IP-019-sdk-client-generation.md`, `microservices/healthcare-integration/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `HealthcareIntegrationActionAccepted`, `HealthcareIntegrationActionInvoked`, `HealthcareIntegrationClinicalExchangeOperatorRemediationApplied`, `HealthcareIntegrationClinicalExchangePolicyDecisionRecorded`, `HealthcareIntegrationDomainEvent`, `HealthcareIntegrationLocalPolicyDecisionRecorded`, `HealthcareIntegrationSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `HealthcareIntegrationActionAccepted`; current=`HealthcareIntegrationActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/healthcare-integration/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `HealthcareIntegrationActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/healthcare-integration/contracts/healthcare-integration-v1.proto`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `HealthcareIntegrationDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `HealthcareIntegrationSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /healthcare-integration/v1/clinical-exchanges/{resource_id}/policy-check` -> `HealthcareIntegrationClinicalExchangePolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/healthcare-integration/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /healthcare-integration/v1/clinical-exchanges/{resource_id}/operator-remediation` -> `HealthcareIntegrationClinicalExchangeOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/healthcare-integration/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `HealthcareIntegrationLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/healthcare-integration/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /healthcare-integration/actions/{action_id}` -> `HealthcareIntegrationActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/healthcare-integration/contracts/openapi-v1.yaml`.

#### `incident-management`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/incident-management/policies/local-escalation-policy-control.cedar`, `microservices/incident-management/policies/local-page-acknowledge-scope.cedar`, `microservices/incident-management/policies/local-page-dispatch-guard.cedar`, `microservices/incident-management/policies/local-postmortem-seal-required.cedar`, `microservices/incident-management/policies/local-stakeholder-update-egress.cedar`, `microservices/incident-management/policies/local-war-room-open-approval.cedar`
- Contract files in scope: `microservices/incident-management/contracts/asyncapi-v1.yaml`, `microservices/incident-management/contracts/incident-management-v1.proto`, `microservices/incident-management/contracts/local-asyncapi-v1.yaml`, `microservices/incident-management/contracts/local-openapi-v1.yaml`, `microservices/incident-management/contracts/local-operations-v1.proto`, `microservices/incident-management/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/incident-management/IP-001-tenant-scope-kernel.md`, `microservices/incident-management/IP-002-cedar-default-deny.md`, `microservices/incident-management/IP-003-ontology-projection.md`, `microservices/incident-management/IP-004-workflow-template-library.md`, `microservices/incident-management/IP-005-rest-contract-surface.md`, `microservices/incident-management/IP-006-async-event-surface.md`, `microservices/incident-management/IP-007-grpc-internal-surface.md`, `microservices/incident-management/IP-008-policy-eval-library-binding.md`, `microservices/incident-management/IP-009-credential-sidecar-binding.md`, `microservices/incident-management/IP-010-multi-region-cell-layout.md`, `microservices/incident-management/IP-011-observability-audit-events.md`, `microservices/incident-management/IP-012-abuse-defence-edge-waf.md`, `microservices/incident-management/IP-013-emergency-services-bypass.md`, `microservices/incident-management/IP-014-marketplace-dealset-settlement.md`, `microservices/incident-management/IP-015-data-residency-pack-overlays.md`, `microservices/incident-management/IP-016-backfill-replay-worker.md`, `microservices/incident-management/IP-017-cost-budget-enforcer.md`, `microservices/incident-management/IP-018-capacity-admission-control.md`, `microservices/incident-management/IP-019-sdk-client-generation.md`, `microservices/incident-management/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `IncidentManagementActionAccepted`, `IncidentManagementActionInvoked`, `IncidentManagementDomainEvent`, `IncidentManagementIncidentCommandOperatorRemediationApplied`, `IncidentManagementIncidentCommandPolicyDecisionRecorded`, `IncidentManagementLocalPolicyDecisionRecorded`, `IncidentManagementSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `IncidentManagementActionAccepted`; current=`IncidentManagementActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/incident-management/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `IncidentManagementActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/incident-management/contracts/incident-management-v1.proto`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `IncidentManagementDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/incident-management/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `IncidentManagementSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/incident-management/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /incident-management/v1/incident-commands/{resource_id}/policy-check` -> `IncidentManagementIncidentCommandPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/incident-management/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /incident-management/v1/incident-commands/{resource_id}/operator-remediation` -> `IncidentManagementIncidentCommandOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/incident-management/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `IncidentManagementLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/incident-management/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /incident-management/actions/{action_id}` -> `IncidentManagementActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/incident-management/contracts/openapi-v1.yaml`.

#### `itsm`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/itsm/policies/local-change-approval-window.cedar`, `microservices/itsm/policies/local-cmdb-relation-write.cedar`, `microservices/itsm/policies/local-incident-ticket-scope.cedar`, `microservices/itsm/policies/local-knowledge-publish-approval.cedar`, `microservices/itsm/policies/local-problem-link-control.cedar`, `microservices/itsm/policies/local-sla-recompute-guard.cedar`
- Contract files in scope: `microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/itsm/IP-001-tenant-scope-kernel.md`, `microservices/itsm/IP-002-cedar-default-deny.md`, `microservices/itsm/IP-003-ontology-projection.md`, `microservices/itsm/IP-004-workflow-template-library.md`, `microservices/itsm/IP-005-rest-contract-surface.md`, `microservices/itsm/IP-006-async-event-surface.md`, `microservices/itsm/IP-007-grpc-internal-surface.md`, `microservices/itsm/IP-008-policy-eval-library-binding.md`, `microservices/itsm/IP-009-credential-sidecar-binding.md`, `microservices/itsm/IP-010-multi-region-cell-layout.md`, `microservices/itsm/IP-011-observability-audit-events.md`, `microservices/itsm/IP-012-abuse-defence-edge-waf.md`, `microservices/itsm/IP-013-emergency-services-bypass.md`, `microservices/itsm/IP-014-marketplace-dealset-settlement.md`, `microservices/itsm/IP-015-data-residency-pack-overlays.md`, `microservices/itsm/IP-016-backfill-replay-worker.md`, `microservices/itsm/IP-017-cost-budget-enforcer.md`, `microservices/itsm/IP-018-capacity-admission-control.md`, `microservices/itsm/IP-019-sdk-client-generation.md`, `microservices/itsm/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `ITSMActionAccepted`, `ITSMActionInvoked`, `ITSMLocalPolicyDecisionRecorded`, `ITSMServiceRecordOperatorRemediationApplied`, `ITSMServiceRecordPolicyDecisionRecorded`, `ItsmActionAccepted`, `ItsmDomainEvent`, `ItsmSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `ItsmActionAccepted`; current=`ItsmActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/itsm/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `ITSMActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/itsm/contracts/itsm-v1.proto`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `ItsmDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/itsm/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `ItsmSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/itsm/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /itsm/v1/service-records/{resource_id}/policy-check` -> `ITSMServiceRecordPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/itsm/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /itsm/v1/service-records/{resource_id}/operator-remediation` -> `ITSMServiceRecordOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/itsm/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `ITSMLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/itsm/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /itsm/actions/{action_id}` -> `ITSMActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/itsm/contracts/openapi-v1.yaml`.

#### `learning-management`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/learning-management/policies/local-assessment-attempt-control.cedar`, `microservices/learning-management/policies/local-certificate-issue-gate.cedar`, `microservices/learning-management/policies/local-cohort-enrollment-scope.cedar`, `microservices/learning-management/policies/local-content-delivery-entitlement.cedar`, `microservices/learning-management/policies/local-course-publish-approval.cedar`, `microservices/learning-management/policies/local-session-attendance-access.cedar`
- Contract files in scope: `microservices/learning-management/contracts/asyncapi-v1.yaml`, `microservices/learning-management/contracts/learning-management-v1.proto`, `microservices/learning-management/contracts/local-asyncapi-v1.yaml`, `microservices/learning-management/contracts/local-openapi-v1.yaml`, `microservices/learning-management/contracts/local-operations-v1.proto`, `microservices/learning-management/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/learning-management/IP-001-tenant-scope-kernel.md`, `microservices/learning-management/IP-002-cedar-default-deny.md`, `microservices/learning-management/IP-003-ontology-projection.md`, `microservices/learning-management/IP-004-workflow-template-library.md`, `microservices/learning-management/IP-005-rest-contract-surface.md`, `microservices/learning-management/IP-006-async-event-surface.md`, `microservices/learning-management/IP-007-grpc-internal-surface.md`, `microservices/learning-management/IP-008-policy-eval-library-binding.md`, `microservices/learning-management/IP-009-credential-sidecar-binding.md`, `microservices/learning-management/IP-010-multi-region-cell-layout.md`, `microservices/learning-management/IP-011-observability-audit-events.md`, `microservices/learning-management/IP-012-abuse-defence-edge-waf.md`, `microservices/learning-management/IP-013-emergency-services-bypass.md`, `microservices/learning-management/IP-014-marketplace-dealset-settlement.md`, `microservices/learning-management/IP-015-data-residency-pack-overlays.md`, `microservices/learning-management/IP-016-backfill-replay-worker.md`, `microservices/learning-management/IP-017-cost-budget-enforcer.md`, `microservices/learning-management/IP-018-capacity-admission-control.md`, `microservices/learning-management/IP-019-sdk-client-generation.md`, `microservices/learning-management/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `LearningManagementActionAccepted`, `LearningManagementActionInvoked`, `LearningManagementDomainEvent`, `LearningManagementLearningCohortOperatorRemediationApplied`, `LearningManagementLearningCohortPolicyDecisionRecorded`, `LearningManagementLocalPolicyDecisionRecorded`, `LearningManagementSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `LearningManagementActionAccepted`; current=`LearningManagementActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/learning-management/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `LearningManagementActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/learning-management/contracts/learning-management-v1.proto`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `LearningManagementDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/learning-management/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `LearningManagementSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/learning-management/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /learning-management/v1/learning-cohorts/{resource_id}/policy-check` -> `LearningManagementLearningCohortPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/learning-management/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /learning-management/v1/learning-cohorts/{resource_id}/operator-remediation` -> `LearningManagementLearningCohortOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/learning-management/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `LearningManagementLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/learning-management/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /learning-management/actions/{action_id}` -> `LearningManagementActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/learning-management/contracts/openapi-v1.yaml`.

#### `marketing-automation`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/marketing-automation/policies/local-attribution-export-egress.cedar`, `microservices/marketing-automation/policies/local-campaign-segment-scope.cedar`, `microservices/marketing-automation/policies/local-consent-gated-automation.cedar`, `microservices/marketing-automation/policies/local-deliverability-inspection.cedar`, `microservices/marketing-automation/policies/local-journey-launch-approval.cedar`, `microservices/marketing-automation/policies/local-suppression-list-enforcement.cedar`
- Contract files in scope: `microservices/marketing-automation/contracts/asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-openapi-v1.yaml`, `microservices/marketing-automation/contracts/local-operations-v1.proto`, `microservices/marketing-automation/contracts/marketing-automation-v1.proto`, `microservices/marketing-automation/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/marketing-automation/IP-001-tenant-scope-kernel.md`, `microservices/marketing-automation/IP-002-cedar-default-deny.md`, `microservices/marketing-automation/IP-003-ontology-projection.md`, `microservices/marketing-automation/IP-004-workflow-template-library.md`, `microservices/marketing-automation/IP-005-rest-contract-surface.md`, `microservices/marketing-automation/IP-006-async-event-surface.md`, `microservices/marketing-automation/IP-007-grpc-internal-surface.md`, `microservices/marketing-automation/IP-008-policy-eval-library-binding.md`, `microservices/marketing-automation/IP-009-credential-sidecar-binding.md`, `microservices/marketing-automation/IP-010-multi-region-cell-layout.md`, `microservices/marketing-automation/IP-011-observability-audit-events.md`, `microservices/marketing-automation/IP-012-abuse-defence-edge-waf.md`, `microservices/marketing-automation/IP-013-emergency-services-bypass.md`, `microservices/marketing-automation/IP-014-marketplace-dealset-settlement.md`, `microservices/marketing-automation/IP-015-data-residency-pack-overlays.md`, `microservices/marketing-automation/IP-016-backfill-replay-worker.md`, `microservices/marketing-automation/IP-017-cost-budget-enforcer.md`, `microservices/marketing-automation/IP-018-capacity-admission-control.md`, `microservices/marketing-automation/IP-019-sdk-client-generation.md`, `microservices/marketing-automation/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `MarketingAutomationActionAccepted`, `MarketingAutomationActionInvoked`, `MarketingAutomationCampaignJourneyOperatorRemediationApplied`, `MarketingAutomationCampaignJourneyPolicyDecisionRecorded`, `MarketingAutomationDomainEvent`, `MarketingAutomationLocalPolicyDecisionRecorded`, `MarketingAutomationSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `MarketingAutomationActionAccepted`; current=`MarketingAutomationActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketing-automation/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `MarketingAutomationDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `MarketingAutomationSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /marketing-automation/v1/campaign-journeys/{resource_id}/policy-check` -> `MarketingAutomationCampaignJourneyPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketing-automation/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /marketing-automation/v1/campaign-journeys/{resource_id}/operator-remediation` -> `MarketingAutomationCampaignJourneyOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketing-automation/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `MarketingAutomationLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketing-automation/contracts/local-operations-v1.proto`.
- Fix item: `GRPC RPC InvokeAction` -> `MarketingAutomationActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketing-automation/contracts/marketing-automation-v1.proto`.
- Fix item: `REST POST /marketing-automation/actions/{action_id}` -> `MarketingAutomationActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketing-automation/contracts/openapi-v1.yaml`.

#### `marketplace`
- Endpoints scanned: 15
- Gaps catalogued: 15
- Policy files needing class-to-action alignment: `microservices/marketplace/policies/deal-accept.cedar`, `microservices/marketplace/policies/deal-offer-create.cedar`, `microservices/marketplace/policies/escrow-release.cedar`, `microservices/marketplace/policies/escrow-reserve.cedar`, `microservices/marketplace/policies/mediation-open.cedar`, `microservices/marketplace/policies/revenue-share-accrue.cedar`
- Contract files in scope: `microservices/marketplace/contracts/asyncapi-v1.yaml`, `microservices/marketplace/contracts/marketplace-v1.proto`, `microservices/marketplace/contracts/openapi-v1.yaml`
- IP slices scanned for existing class evidence: `microservices/marketplace/IP-journey-j101-deal-settlement-ledger.md`, `microservices/marketplace/IP-journey-j102-deal-settlement-ledger.md`, `microservices/marketplace/IP-journey-j103-deal-settlement-ledger.md`, `microservices/marketplace/IP-journey-j107-deal-settlement-ledger.md`, `microservices/marketplace/IP-journey-j108-deal-settlement-ledger.md`, `microservices/marketplace/IP-journey-j112-deal-settlement-ledger.md`, `microservices/marketplace/IP-journey-j146-seller-flow-and-escrow.md`, `microservices/marketplace/IP-journey-j23-seller-listing.md`, `microservices/marketplace/IP-journey-j24-buyer-order.md`, `microservices/marketplace/IP-journey-j29-sale-event-emitter.md`, `microservices/marketplace/IP-journey-j52-order-ledger.md`, `microservices/marketplace/IP-journey-j55-seller-buyer-mediation.md`, `microservices/marketplace/IP-journey-j65-order-export.md`, `microservices/marketplace/IP-journey-j69-appointment-and-service-commitments.md`, `microservices/marketplace/IP-journey-j73-revenue-share.md`, `microservices/marketplace/ip/IP-001-deal-set-kernel.md`, `microservices/marketplace/ip/IP-002-settlement-ledger-domain.md`, `microservices/marketplace/ip/IP-003-offer-command-usecase.md`, `microservices/marketplace/ip/IP-004-buyer-order-rest-api.md`, `microservices/marketplace/ip/IP-005-seller-listing-rest-api.md`; plus 20 more
- Classes to register or bind: `MarketplaceDealAccepted`, `MarketplaceDealOffered`, `MarketplaceDisputeOpened`, `MarketplaceEscrowReleased`, `MarketplaceEscrowReserved`, `MarketplaceListingPublished`, `MarketplaceOrderExported`, `MarketplaceRevenueShareAccrued`
- Fix item: `ASYNC SEND publishMarketplaceDealOffered` -> `MarketplaceDealOffered`; current=`MarketplaceDealOffered`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishMarketplaceDealAccepted` -> `MarketplaceDealAccepted`; current=`MarketplaceDealAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishMarketplaceEscrowReserved` -> `MarketplaceEscrowReserved`; current=`MarketplaceEscrowReserved`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishMarketplaceEscrowReleased` -> `MarketplaceEscrowReleased`; current=`MarketplaceEscrowReleased`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishMarketplaceDisputeOpened` -> `MarketplaceDisputeOpened`; current=`MarketplaceDisputeOpened`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishMarketplaceRevenueShareAccrued` -> `MarketplaceRevenueShareAccrued`; current=`MarketplaceRevenueShareAccrued`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishMarketplaceOrderExported` -> `MarketplaceOrderExported`; current=`MarketplaceOrderExported`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/asyncapi-v1.yaml`.
- Fix item: `GRPC RPC SubmitDealSet` -> `MarketplaceDealOffered`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/marketplace-v1.proto`.
- Fix item: `REST POST /marketplace/deal-sets` -> `MarketplaceDealOffered`; current=`MarketplaceDealOffered`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /marketplace/deal-sets/{deal_set_id}/accept` -> `MarketplaceDealAccepted`; current=`MarketplaceDealOffered`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /marketplace/deal-sets/{deal_set_id}/settle` -> `MarketplaceEscrowReleased`; current=`MarketplaceDealOffered`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /marketplace/listings` -> `MarketplaceListingPublished`; current=`MarketplaceDealOffered`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /marketplace/escrow/holds` -> `MarketplaceEscrowReserved`; current=`MarketplaceDealOffered`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /marketplace/disputes` -> `MarketplaceDisputeOpened`; current=`MarketplaceDealOffered`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /marketplace/revenue-shares` -> `MarketplaceRevenueShareAccrued`; current=`MarketplaceDealOffered`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/marketplace/contracts/openapi-v1.yaml`.

#### `performance-management`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/performance-management/policies/local-calibration-lock-control.cedar`, `microservices/performance-management/policies/local-feedback-visibility.cedar`, `microservices/performance-management/policies/local-goal-alignment-approval.cedar`, `microservices/performance-management/policies/local-hr-export-egress.cedar`, `microservices/performance-management/policies/local-rating-change-guard.cedar`, `microservices/performance-management/policies/local-review-cycle-scope.cedar`
- Contract files in scope: `microservices/performance-management/contracts/asyncapi-v1.yaml`, `microservices/performance-management/contracts/local-asyncapi-v1.yaml`, `microservices/performance-management/contracts/local-openapi-v1.yaml`, `microservices/performance-management/contracts/local-operations-v1.proto`, `microservices/performance-management/contracts/openapi-v1.yaml`, `microservices/performance-management/contracts/performance-management-v1.proto`
- IP slices scanned for existing class evidence: `microservices/performance-management/IP-001-tenant-scope-kernel.md`, `microservices/performance-management/IP-002-cedar-default-deny.md`, `microservices/performance-management/IP-003-ontology-projection.md`, `microservices/performance-management/IP-004-workflow-template-library.md`, `microservices/performance-management/IP-005-rest-contract-surface.md`, `microservices/performance-management/IP-006-async-event-surface.md`, `microservices/performance-management/IP-007-grpc-internal-surface.md`, `microservices/performance-management/IP-008-policy-eval-library-binding.md`, `microservices/performance-management/IP-009-credential-sidecar-binding.md`, `microservices/performance-management/IP-010-multi-region-cell-layout.md`, `microservices/performance-management/IP-011-observability-audit-events.md`, `microservices/performance-management/IP-012-abuse-defence-edge-waf.md`, `microservices/performance-management/IP-013-emergency-services-bypass.md`, `microservices/performance-management/IP-014-marketplace-dealset-settlement.md`, `microservices/performance-management/IP-015-data-residency-pack-overlays.md`, `microservices/performance-management/IP-016-backfill-replay-worker.md`, `microservices/performance-management/IP-017-cost-budget-enforcer.md`, `microservices/performance-management/IP-018-capacity-admission-control.md`, `microservices/performance-management/IP-019-sdk-client-generation.md`, `microservices/performance-management/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `PerformanceManagementActionAccepted`, `PerformanceManagementActionInvoked`, `PerformanceManagementDomainEvent`, `PerformanceManagementLocalPolicyDecisionRecorded`, `PerformanceManagementReviewCycleOperatorRemediationApplied`, `PerformanceManagementReviewCyclePolicyDecisionRecorded`, `PerformanceManagementSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `PerformanceManagementActionAccepted`; current=`PerformanceManagementActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/performance-management/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `PerformanceManagementDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/performance-management/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `PerformanceManagementSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/performance-management/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /performance-management/v1/review-cycles/{resource_id}/policy-check` -> `PerformanceManagementReviewCyclePolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/performance-management/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /performance-management/v1/review-cycles/{resource_id}/operator-remediation` -> `PerformanceManagementReviewCycleOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/performance-management/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `PerformanceManagementLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/performance-management/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /performance-management/actions/{action_id}` -> `PerformanceManagementActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/performance-management/contracts/openapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `PerformanceManagementActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/performance-management/contracts/performance-management-v1.proto`.

#### `whiteboard`
- Endpoints scanned: 8
- Gaps catalogued: 8
- Policy files needing class-to-action alignment: `microservices/whiteboard/policies/local-board-export-egress.cedar`, `microservices/whiteboard/policies/local-board-open-scope.cedar`, `microservices/whiteboard/policies/local-crdt-merge-control.cedar`, `microservices/whiteboard/policies/local-cursor-broadcast-rate.cedar`, `microservices/whiteboard/policies/local-shape-update-acl.cedar`, `microservices/whiteboard/policies/local-stroke-persistence-guard.cedar`
- Contract files in scope: `microservices/whiteboard/contracts/asyncapi-v1.yaml`, `microservices/whiteboard/contracts/local-asyncapi-v1.yaml`, `microservices/whiteboard/contracts/local-openapi-v1.yaml`, `microservices/whiteboard/contracts/local-operations-v1.proto`, `microservices/whiteboard/contracts/openapi-v1.yaml`, `microservices/whiteboard/contracts/whiteboard-v1.proto`
- IP slices scanned for existing class evidence: `microservices/whiteboard/IP-001-tenant-scope-kernel.md`, `microservices/whiteboard/IP-002-cedar-default-deny.md`, `microservices/whiteboard/IP-003-ontology-projection.md`, `microservices/whiteboard/IP-004-workflow-template-library.md`, `microservices/whiteboard/IP-005-rest-contract-surface.md`, `microservices/whiteboard/IP-006-async-event-surface.md`, `microservices/whiteboard/IP-007-grpc-internal-surface.md`, `microservices/whiteboard/IP-008-policy-eval-library-binding.md`, `microservices/whiteboard/IP-009-credential-sidecar-binding.md`, `microservices/whiteboard/IP-010-multi-region-cell-layout.md`, `microservices/whiteboard/IP-011-observability-audit-events.md`, `microservices/whiteboard/IP-012-abuse-defence-edge-waf.md`, `microservices/whiteboard/IP-013-emergency-services-bypass.md`, `microservices/whiteboard/IP-014-marketplace-dealset-settlement.md`, `microservices/whiteboard/IP-015-data-residency-pack-overlays.md`, `microservices/whiteboard/IP-016-backfill-replay-worker.md`, `microservices/whiteboard/IP-017-cost-budget-enforcer.md`, `microservices/whiteboard/IP-018-capacity-admission-control.md`, `microservices/whiteboard/IP-019-sdk-client-generation.md`, `microservices/whiteboard/IP-020-catalog-layer-registration.md`; plus 10 more
- Classes to register or bind: `WhiteboardActionAccepted`, `WhiteboardActionInvoked`, `WhiteboardDomainEvent`, `WhiteboardLocalPolicyDecisionRecorded`, `WhiteboardSessionOperatorRemediationApplied`, `WhiteboardSessionPolicyDecisionRecorded`, `WhiteboardSloBurnEvent`
- Fix item: `ASYNC SEND publishActionAccepted` -> `WhiteboardActionAccepted`; current=`WhiteboardActionAccepted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, UNBOUNDED_AUDIT_EVENT_CLASS_FIELD, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/whiteboard/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishDomainEvent` -> `WhiteboardDomainEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/whiteboard/contracts/local-asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishSloBurnEvent` -> `WhiteboardSloBurnEvent`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/whiteboard/contracts/local-asyncapi-v1.yaml`.
- Fix item: `REST POST /whiteboard/v1/whiteboard-sessions/{resource_id}/policy-check` -> `WhiteboardSessionPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/whiteboard/contracts/local-openapi-v1.yaml`.
- Fix item: `REST POST /whiteboard/v1/whiteboard-sessions/{resource_id}/operator-remediation` -> `WhiteboardSessionOperatorRemediationApplied`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/whiteboard/contracts/local-openapi-v1.yaml`.
- Fix item: `GRPC RPC CheckLocalPolicy` -> `WhiteboardLocalPolicyDecisionRecorded`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/whiteboard/contracts/local-operations-v1.proto`.
- Fix item: `REST POST /whiteboard/actions/{action_id}` -> `WhiteboardActionAccepted`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/whiteboard/contracts/openapi-v1.yaml`.
- Fix item: `GRPC RPC InvokeAction` -> `WhiteboardActionInvoked`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/whiteboard/contracts/whiteboard-v1.proto`.

#### `workplace-integration`
- Endpoints scanned: 15
- Gaps catalogued: 15
- Policy files needing class-to-action alignment: `microservices/workplace-integration/policies/clock-attest.cedar`, `microservices/workplace-integration/policies/dlp-trace-seal.cedar`, `microservices/workplace-integration/policies/esign-initiate.cedar`, `microservices/workplace-integration/policies/esign-sign.cedar`, `microservices/workplace-integration/policies/offer-generate.cedar`, `microservices/workplace-integration/policies/roster-bind.cedar`
- Contract files in scope: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`, `microservices/workplace-integration/contracts/openapi-v1.yaml`, `microservices/workplace-integration/contracts/workplace-integration-v1.proto`
- IP slices scanned for existing class evidence: `microservices/workplace-integration/IP-journey-j109-esign-roster-binding.md`, `microservices/workplace-integration/IP-journey-j110-esign-roster-binding.md`, `microservices/workplace-integration/IP-journey-j112-esign-roster-binding.md`, `microservices/workplace-integration/IP-journey-j113-esign-roster-binding.md`, `microservices/workplace-integration/IP-journey-j114-esign-roster-binding.md`, `microservices/workplace-integration/IP-journey-j121-esign-closing-package.md`, `microservices/workplace-integration/IP-journey-j132-offer-letter-esign-per-jurisdiction.md`, `microservices/workplace-integration/IP-journey-j134-engagement-agreement-and-staffing-aware-offer.md`, `microservices/workplace-integration/IP-journey-j140-internal-audit-dlp-egress-cross-tenant-trace.md`, `microservices/workplace-integration/IP-journey-j37-clock-in-geofence.md`, `microservices/workplace-integration/IP-journey-j38-e-sign-session.md`, `microservices/workplace-integration/IP-journey-j51-e-sign-on-po.md`, `microservices/workplace-integration/IP-journey-j54-e-signature.md`, `microservices/workplace-integration/IP-journey-j56-offer-e-sign.md`, `microservices/workplace-integration/IP-journey-j63-informed-consent.md`, `microservices/workplace-integration/IP-journey-j70-e-sign.md`, `microservices/workplace-integration/ip/IP-001-agreement-kernel.md`, `microservices/workplace-integration/ip/IP-002-esign-session-domain.md`, `microservices/workplace-integration/ip/IP-003-signature-proof-usecase.md`, `microservices/workplace-integration/ip/IP-004-offer-letter-rest-api.md`; plus 21 more
- Classes to register or bind: `WorkplaceAgreementBound`, `WorkplaceClockEventAttested`, `WorkplaceDlpTraceSealed`, `WorkplaceESignSessionCreated`, `WorkplaceOfferGenerated`, `WorkplaceRosterBindingGranted`, `WorkplaceSignatureCaptured`
- Fix item: `ASYNC SEND publishWorkplaceESignSessionCreated` -> `WorkplaceESignSessionCreated`; current=`WorkplaceESignSessionCreated`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishWorkplaceSignatureCaptured` -> `WorkplaceSignatureCaptured`; current=`WorkplaceSignatureCaptured`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishWorkplaceOfferGenerated` -> `WorkplaceOfferGenerated`; current=`WorkplaceOfferGenerated`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishWorkplaceAgreementBound` -> `WorkplaceAgreementBound`; current=`WorkplaceAgreementBound`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishWorkplaceRosterBindingGranted` -> `WorkplaceRosterBindingGranted`; current=`WorkplaceRosterBindingGranted`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishWorkplaceClockEventAttested` -> `WorkplaceClockEventAttested`; current=`WorkplaceClockEventAttested`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/asyncapi-v1.yaml`.
- Fix item: `ASYNC SEND publishWorkplaceDlpTraceSealed` -> `WorkplaceDlpTraceSealed`; current=`WorkplaceDlpTraceSealed`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/asyncapi-v1.yaml`.
- Fix item: `REST POST /workplace/esign/sessions` -> `WorkplaceESignSessionCreated`; current=`WorkplaceESignSessionCreated`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /workplace/esign/sessions/{session_id}/sign` -> `WorkplaceSignatureCaptured`; current=`WorkplaceESignSessionCreated`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /workplace/offer-letters` -> `WorkplaceOfferGenerated`; current=`WorkplaceESignSessionCreated`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /workplace/engagement-agreements` -> `WorkplaceAgreementBound`; current=`WorkplaceESignSessionCreated`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /workplace/roster-bindings` -> `WorkplaceRosterBindingGranted`; current=`WorkplaceESignSessionCreated`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /workplace/clock-events` -> `WorkplaceClockEventAttested`; current=`WorkplaceESignSessionCreated`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/openapi-v1.yaml`.
- Fix item: `REST POST /workplace/dlp-traces` -> `WorkplaceDlpTraceSealed`; current=`WorkplaceESignSessionCreated`; reasons=`UNREGISTERED_AUDIT_EVENT_CLASS, SEMANTIC_CLASS_MISMATCH, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/openapi-v1.yaml`.
- Fix item: `GRPC RPC SubmitWorkplaceAgreement` -> `WorkplaceAgreementBound`; current=`NONE`; reasons=`MISSING_NAMED_AUDIT_EVENT_CLASS, RECOMMENDED_CLASS_NEEDS_REGISTRY_ENTRY`; contract=`microservices/workplace-integration/contracts/workplace-integration-v1.proto`.

### §5.3 Endpoint evidence ledger

#### ENDPOINT-0001 — `contact-center` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/contact-center/contracts/asyncapi-v1.yaml`
- Current named class: `ContactCenterActionAccepted`
- Recommended ADR-0263 class: `ContactCenterActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `ContactCenterActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0002 — `contact-center` — `GRPC RPC InvokeAction`
- Contract file: `microservices/contact-center/contracts/contact-center-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContactCenterActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0003 — `contact-center` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/contact-center/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContactCenterDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0004 — `contact-center` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/contact-center/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContactCenterSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0005 — `contact-center` — `REST POST /contact-center/v1/interactions/{resource_id}/policy-check`
- Contract file: `microservices/contact-center/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContactCenterInteractionPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0006 — `contact-center` — `REST POST /contact-center/v1/interactions/{resource_id}/operator-remediation`
- Contract file: `microservices/contact-center/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContactCenterInteractionOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0007 — `contact-center` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/contact-center/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContactCenterLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0008 — `contact-center` — `REST POST /contact-center/actions/{action_id}`
- Contract file: `microservices/contact-center/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContactCenterActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `ContactCenterActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0009 — `contract-lifecycle-management` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/contract-lifecycle-management/contracts/asyncapi-v1.yaml`
- Current named class: `ContractLifecycleManagementActionAccepted`
- Recommended ADR-0263 class: `ContractLifecycleManagementActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `ContractLifecycleManagementActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0010 — `contract-lifecycle-management` — `GRPC RPC InvokeAction`
- Contract file: `microservices/contract-lifecycle-management/contracts/contract-lifecycle-management-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContractLifecycleManagementActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0011 — `contract-lifecycle-management` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/contract-lifecycle-management/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContractLifecycleManagementDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0012 — `contract-lifecycle-management` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/contract-lifecycle-management/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContractLifecycleManagementSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0013 — `contract-lifecycle-management` — `REST POST /contract-lifecycle-management/v1/contract-workspaces/{resource_id}/policy-check`
- Contract file: `microservices/contract-lifecycle-management/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContractLifecycleManagementContractWorkspacePolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0014 — `contract-lifecycle-management` — `REST POST /contract-lifecycle-management/v1/contract-workspaces/{resource_id}/operator-remediation`
- Contract file: `microservices/contract-lifecycle-management/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContractLifecycleManagementContractWorkspaceOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0015 — `contract-lifecycle-management` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/contract-lifecycle-management/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContractLifecycleManagementLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0016 — `contract-lifecycle-management` — `REST POST /contract-lifecycle-management/actions/{action_id}`
- Contract file: `microservices/contract-lifecycle-management/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ContractLifecycleManagementActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `ContractLifecycleManagementActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0017 — `data-pipeline` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/data-pipeline/contracts/asyncapi-v1.yaml`
- Current named class: `DataPipelineActionAccepted`
- Recommended ADR-0263 class: `DataPipelineActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `DataPipelineActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0018 — `data-pipeline` — `GRPC RPC InvokeAction`
- Contract file: `microservices/data-pipeline/contracts/data-pipeline-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataPipelineActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0019 — `data-pipeline` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataPipelineDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0020 — `data-pipeline` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataPipelineSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0021 — `data-pipeline` — `REST POST /data-pipeline/v1/pipeline-runs/{resource_id}/policy-check`
- Contract file: `microservices/data-pipeline/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataPipelinePipelineRunPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0022 — `data-pipeline` — `REST POST /data-pipeline/v1/pipeline-runs/{resource_id}/operator-remediation`
- Contract file: `microservices/data-pipeline/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataPipelinePipelineRunOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0023 — `data-pipeline` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/data-pipeline/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataPipelineLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0024 — `data-pipeline` — `REST POST /data-pipeline/actions/{action_id}`
- Contract file: `microservices/data-pipeline/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataPipelineActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `DataPipelineActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0025 — `data-warehouse` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/data-warehouse/contracts/asyncapi-v1.yaml`
- Current named class: `DataWarehouseActionAccepted`
- Recommended ADR-0263 class: `DataWarehouseActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `DataWarehouseActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0026 — `data-warehouse` — `GRPC RPC InvokeAction`
- Contract file: `microservices/data-warehouse/contracts/data-warehouse-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataWarehouseActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0027 — `data-warehouse` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/data-warehouse/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataWarehouseDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0028 — `data-warehouse` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/data-warehouse/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataWarehouseSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0029 — `data-warehouse` — `REST POST /data-warehouse/v1/warehouse-datasets/{resource_id}/policy-check`
- Contract file: `microservices/data-warehouse/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataWarehouseWarehouseDatasetPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0030 — `data-warehouse` — `REST POST /data-warehouse/v1/warehouse-datasets/{resource_id}/operator-remediation`
- Contract file: `microservices/data-warehouse/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataWarehouseWarehouseDatasetOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0031 — `data-warehouse` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/data-warehouse/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataWarehouseLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0032 — `data-warehouse` — `REST POST /data-warehouse/actions/{action_id}`
- Contract file: `microservices/data-warehouse/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DataWarehouseActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `DataWarehouseActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0033 — `design-collaboration` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/design-collaboration/contracts/asyncapi-v1.yaml`
- Current named class: `DesignCollaborationActionAccepted`
- Recommended ADR-0263 class: `DesignCollaborationActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `DesignCollaborationActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0034 — `design-collaboration` — `GRPC RPC InvokeAction`
- Contract file: `microservices/design-collaboration/contracts/design-collaboration-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DesignCollaborationActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0035 — `design-collaboration` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/design-collaboration/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DesignCollaborationDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0036 — `design-collaboration` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/design-collaboration/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DesignCollaborationSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0037 — `design-collaboration` — `REST POST /design-collaboration/v1/design-files/{resource_id}/policy-check`
- Contract file: `microservices/design-collaboration/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DesignCollaborationDesignFilePolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0038 — `design-collaboration` — `REST POST /design-collaboration/v1/design-files/{resource_id}/operator-remediation`
- Contract file: `microservices/design-collaboration/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DesignCollaborationDesignFileOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0039 — `design-collaboration` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/design-collaboration/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DesignCollaborationLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0040 — `design-collaboration` — `REST POST /design-collaboration/actions/{action_id}`
- Contract file: `microservices/design-collaboration/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `DesignCollaborationActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `DesignCollaborationActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0041 — `financial-planning` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/financial-planning/contracts/asyncapi-v1.yaml`
- Current named class: `FinancialPlanningActionAccepted`
- Recommended ADR-0263 class: `FinancialPlanningActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `FinancialPlanningActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0042 — `financial-planning` — `GRPC RPC InvokeAction`
- Contract file: `microservices/financial-planning/contracts/financial-planning-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `FinancialPlanningActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0043 — `financial-planning` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/financial-planning/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `FinancialPlanningDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0044 — `financial-planning` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/financial-planning/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `FinancialPlanningSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0045 — `financial-planning` — `REST POST /financial-planning/v1/planning-cycles/{resource_id}/policy-check`
- Contract file: `microservices/financial-planning/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `FinancialPlanningPlanningCyclePolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0046 — `financial-planning` — `REST POST /financial-planning/v1/planning-cycles/{resource_id}/operator-remediation`
- Contract file: `microservices/financial-planning/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `FinancialPlanningPlanningCycleOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0047 — `financial-planning` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/financial-planning/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `FinancialPlanningLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0048 — `financial-planning` — `REST POST /financial-planning/actions/{action_id}`
- Contract file: `microservices/financial-planning/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `FinancialPlanningActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `FinancialPlanningActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0049 — `healthcare-integration` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/healthcare-integration/contracts/asyncapi-v1.yaml`
- Current named class: `HealthcareIntegrationActionAccepted`
- Recommended ADR-0263 class: `HealthcareIntegrationActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `HealthcareIntegrationActionAccepted`
- IP cross-reference: `HealthcareIntegrationActionAccepted`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0050 — `healthcare-integration` — `GRPC RPC InvokeAction`
- Contract file: `microservices/healthcare-integration/contracts/healthcare-integration-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `HealthcareIntegrationActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0051 — `healthcare-integration` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `HealthcareIntegrationDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0052 — `healthcare-integration` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/healthcare-integration/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `HealthcareIntegrationSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0053 — `healthcare-integration` — `REST POST /healthcare-integration/v1/clinical-exchanges/{resource_id}/policy-check`
- Contract file: `microservices/healthcare-integration/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `HealthcareIntegrationClinicalExchangePolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0054 — `healthcare-integration` — `REST POST /healthcare-integration/v1/clinical-exchanges/{resource_id}/operator-remediation`
- Contract file: `microservices/healthcare-integration/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `HealthcareIntegrationClinicalExchangeOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0055 — `healthcare-integration` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/healthcare-integration/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `HealthcareIntegrationLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0056 — `healthcare-integration` — `REST POST /healthcare-integration/actions/{action_id}`
- Contract file: `microservices/healthcare-integration/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `HealthcareIntegrationActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `HealthcareIntegrationActionAccepted`
- IP cross-reference: `HealthcareIntegrationActionAccepted`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0057 — `incident-management` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/incident-management/contracts/asyncapi-v1.yaml`
- Current named class: `IncidentManagementActionAccepted`
- Recommended ADR-0263 class: `IncidentManagementActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `IncidentManagementActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0058 — `incident-management` — `GRPC RPC InvokeAction`
- Contract file: `microservices/incident-management/contracts/incident-management-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `IncidentManagementActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0059 — `incident-management` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/incident-management/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `IncidentManagementDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0060 — `incident-management` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/incident-management/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `IncidentManagementSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0061 — `incident-management` — `REST POST /incident-management/v1/incident-commands/{resource_id}/policy-check`
- Contract file: `microservices/incident-management/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `IncidentManagementIncidentCommandPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0062 — `incident-management` — `REST POST /incident-management/v1/incident-commands/{resource_id}/operator-remediation`
- Contract file: `microservices/incident-management/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `IncidentManagementIncidentCommandOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0063 — `incident-management` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/incident-management/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `IncidentManagementLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0064 — `incident-management` — `REST POST /incident-management/actions/{action_id}`
- Contract file: `microservices/incident-management/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `IncidentManagementActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `IncidentManagementActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0065 — `itsm` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/itsm/contracts/asyncapi-v1.yaml`
- Current named class: `ItsmActionAccepted`
- Recommended ADR-0263 class: `ItsmActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `ItsmActionAccepted`
- IP cross-reference: `ItsmActionAccepted`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0066 — `itsm` — `GRPC RPC InvokeAction`
- Contract file: `microservices/itsm/contracts/itsm-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ITSMActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0067 — `itsm` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/itsm/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ItsmDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0068 — `itsm` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/itsm/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ItsmSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0069 — `itsm` — `REST POST /itsm/v1/service-records/{resource_id}/policy-check`
- Contract file: `microservices/itsm/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ITSMServiceRecordPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0070 — `itsm` — `REST POST /itsm/v1/service-records/{resource_id}/operator-remediation`
- Contract file: `microservices/itsm/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ITSMServiceRecordOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0071 — `itsm` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/itsm/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ITSMLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0072 — `itsm` — `REST POST /itsm/actions/{action_id}`
- Contract file: `microservices/itsm/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `ITSMActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0073 — `learning-management` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/learning-management/contracts/asyncapi-v1.yaml`
- Current named class: `LearningManagementActionAccepted`
- Recommended ADR-0263 class: `LearningManagementActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `LearningManagementActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0074 — `learning-management` — `GRPC RPC InvokeAction`
- Contract file: `microservices/learning-management/contracts/learning-management-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `LearningManagementActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0075 — `learning-management` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/learning-management/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `LearningManagementDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0076 — `learning-management` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/learning-management/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `LearningManagementSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0077 — `learning-management` — `REST POST /learning-management/v1/learning-cohorts/{resource_id}/policy-check`
- Contract file: `microservices/learning-management/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `LearningManagementLearningCohortPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0078 — `learning-management` — `REST POST /learning-management/v1/learning-cohorts/{resource_id}/operator-remediation`
- Contract file: `microservices/learning-management/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `LearningManagementLearningCohortOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0079 — `learning-management` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/learning-management/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `LearningManagementLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0080 — `learning-management` — `REST POST /learning-management/actions/{action_id}`
- Contract file: `microservices/learning-management/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `LearningManagementActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `LearningManagementActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0081 — `marketing-automation` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/marketing-automation/contracts/asyncapi-v1.yaml`
- Current named class: `MarketingAutomationActionAccepted`
- Recommended ADR-0263 class: `MarketingAutomationActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketingAutomationActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0082 — `marketing-automation` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `MarketingAutomationDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0083 — `marketing-automation` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `MarketingAutomationSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0084 — `marketing-automation` — `REST POST /marketing-automation/v1/campaign-journeys/{resource_id}/policy-check`
- Contract file: `microservices/marketing-automation/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `MarketingAutomationCampaignJourneyPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0085 — `marketing-automation` — `REST POST /marketing-automation/v1/campaign-journeys/{resource_id}/operator-remediation`
- Contract file: `microservices/marketing-automation/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `MarketingAutomationCampaignJourneyOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0086 — `marketing-automation` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/marketing-automation/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `MarketingAutomationLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0087 — `marketing-automation` — `GRPC RPC InvokeAction`
- Contract file: `microservices/marketing-automation/contracts/marketing-automation-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `MarketingAutomationActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0088 — `marketing-automation` — `REST POST /marketing-automation/actions/{action_id}`
- Contract file: `microservices/marketing-automation/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `MarketingAutomationActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketingAutomationActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0089 — `marketplace` — `ASYNC SEND publishMarketplaceDealOffered`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current named class: `MarketplaceDealOffered`
- Recommended ADR-0263 class: `MarketplaceDealOffered`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDealOffered`
- IP cross-reference: `MarketplaceDealOffered`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0090 — `marketplace` — `ASYNC SEND publishMarketplaceDealAccepted`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current named class: `MarketplaceDealAccepted`
- Recommended ADR-0263 class: `MarketplaceDealAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDealAccepted`
- IP cross-reference: `MarketplaceDealAccepted`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0091 — `marketplace` — `ASYNC SEND publishMarketplaceEscrowReserved`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current named class: `MarketplaceEscrowReserved`
- Recommended ADR-0263 class: `MarketplaceEscrowReserved`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceEscrowReserved`
- IP cross-reference: `MarketplaceEscrowReserved`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0092 — `marketplace` — `ASYNC SEND publishMarketplaceEscrowReleased`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current named class: `MarketplaceEscrowReleased`
- Recommended ADR-0263 class: `MarketplaceEscrowReleased`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceEscrowReleased`
- IP cross-reference: `MarketplaceEscrowReleased`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0093 — `marketplace` — `ASYNC SEND publishMarketplaceDisputeOpened`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current named class: `MarketplaceDisputeOpened`
- Recommended ADR-0263 class: `MarketplaceDisputeOpened`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDisputeOpened`
- IP cross-reference: `MarketplaceDisputeOpened`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0094 — `marketplace` — `ASYNC SEND publishMarketplaceRevenueShareAccrued`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current named class: `MarketplaceRevenueShareAccrued`
- Recommended ADR-0263 class: `MarketplaceRevenueShareAccrued`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceRevenueShareAccrued`
- IP cross-reference: `MarketplaceRevenueShareAccrued`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0095 — `marketplace` — `ASYNC SEND publishMarketplaceOrderExported`
- Contract file: `microservices/marketplace/contracts/asyncapi-v1.yaml`
- Current named class: `MarketplaceOrderExported`
- Recommended ADR-0263 class: `MarketplaceOrderExported`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceOrderExported`
- IP cross-reference: `MarketplaceOrderExported`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0096 — `marketplace` — `GRPC RPC SubmitDealSet`
- Contract file: `microservices/marketplace/contracts/marketplace-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `MarketplaceDealOffered`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDealOffered`
- IP cross-reference: `MarketplaceDealOffered`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0097 — `marketplace` — `REST POST /marketplace/deal-sets`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current named class: `MarketplaceDealOffered`
- Recommended ADR-0263 class: `MarketplaceDealOffered`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDealOffered`
- IP cross-reference: `MarketplaceDealOffered`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=MarketplaceDealOffered

#### ENDPOINT-0098 — `marketplace` — `REST POST /marketplace/deal-sets/{deal_set_id}/accept`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current named class: `MarketplaceDealOffered`
- Recommended ADR-0263 class: `MarketplaceDealAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDealAccepted,MarketplaceDealOffered`
- IP cross-reference: `MarketplaceDealAccepted,MarketplaceDealOffered`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=MarketplaceDealOffered

#### ENDPOINT-0099 — `marketplace` — `REST POST /marketplace/deal-sets/{deal_set_id}/settle`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current named class: `MarketplaceDealOffered`
- Recommended ADR-0263 class: `MarketplaceEscrowReleased`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDealOffered,MarketplaceEscrowReleased`
- IP cross-reference: `MarketplaceDealOffered,MarketplaceEscrowReleased`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=MarketplaceDealOffered

#### ENDPOINT-0100 — `marketplace` — `REST POST /marketplace/listings`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current named class: `MarketplaceDealOffered`
- Recommended ADR-0263 class: `MarketplaceListingPublished`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDealOffered`
- IP cross-reference: `MarketplaceDealOffered`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=MarketplaceDealOffered

#### ENDPOINT-0101 — `marketplace` — `REST POST /marketplace/escrow/holds`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current named class: `MarketplaceDealOffered`
- Recommended ADR-0263 class: `MarketplaceEscrowReserved`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDealOffered,MarketplaceEscrowReserved`
- IP cross-reference: `MarketplaceDealOffered,MarketplaceEscrowReserved`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=MarketplaceDealOffered

#### ENDPOINT-0102 — `marketplace` — `REST POST /marketplace/disputes`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current named class: `MarketplaceDealOffered`
- Recommended ADR-0263 class: `MarketplaceDisputeOpened`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDealOffered,MarketplaceDisputeOpened`
- IP cross-reference: `MarketplaceDealOffered,MarketplaceDisputeOpened`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=MarketplaceDealOffered

#### ENDPOINT-0103 — `marketplace` — `REST POST /marketplace/revenue-shares`
- Contract file: `microservices/marketplace/contracts/openapi-v1.yaml`
- Current named class: `MarketplaceDealOffered`
- Recommended ADR-0263 class: `MarketplaceRevenueShareAccrued`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `MarketplaceDealOffered,MarketplaceRevenueShareAccrued`
- IP cross-reference: `MarketplaceDealOffered,MarketplaceRevenueShareAccrued`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=MarketplaceDealOffered

#### ENDPOINT-0104 — `performance-management` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/performance-management/contracts/asyncapi-v1.yaml`
- Current named class: `PerformanceManagementActionAccepted`
- Recommended ADR-0263 class: `PerformanceManagementActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `PerformanceManagementActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0105 — `performance-management` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/performance-management/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `PerformanceManagementDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0106 — `performance-management` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/performance-management/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `PerformanceManagementSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0107 — `performance-management` — `REST POST /performance-management/v1/review-cycles/{resource_id}/policy-check`
- Contract file: `microservices/performance-management/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `PerformanceManagementReviewCyclePolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0108 — `performance-management` — `REST POST /performance-management/v1/review-cycles/{resource_id}/operator-remediation`
- Contract file: `microservices/performance-management/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `PerformanceManagementReviewCycleOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0109 — `performance-management` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/performance-management/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `PerformanceManagementLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0110 — `performance-management` — `REST POST /performance-management/actions/{action_id}`
- Contract file: `microservices/performance-management/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `PerformanceManagementActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `PerformanceManagementActionAccepted`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0111 — `performance-management` — `GRPC RPC InvokeAction`
- Contract file: `microservices/performance-management/contracts/performance-management-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `PerformanceManagementActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0112 — `whiteboard` — `ASYNC SEND publishActionAccepted`
- Contract file: `microservices/whiteboard/contracts/asyncapi-v1.yaml`
- Current named class: `WhiteboardActionAccepted`
- Recommended ADR-0263 class: `WhiteboardActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WhiteboardActionAccepted`
- IP cross-reference: `WhiteboardActionAccepted`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_class present but unconstrained; required=tenant_id,principal_id,audit_event_class,event_time,deal_set_id

#### ENDPOINT-0113 — `whiteboard` — `ASYNC SEND publishDomainEvent`
- Contract file: `microservices/whiteboard/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `WhiteboardDomainEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_event_id present; required=tenant_id,event_name,resource_id,data_class,audit_event_id,occurred_at

#### ENDPOINT-0114 — `whiteboard` — `ASYNC SEND publishSloBurnEvent`
- Contract file: `microservices/whiteboard/contracts/local-asyncapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `WhiteboardSloBurnEvent`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: AsyncAPI publish operation action=send; details: required=tenant_id,service,slo_name,burn_rate,objective,window,first_domain_event

#### ENDPOINT-0115 — `whiteboard` — `REST POST /whiteboard/v1/whiteboard-sessions/{resource_id}/policy-check`
- Contract file: `microservices/whiteboard/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `WhiteboardSessionPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0116 — `whiteboard` — `REST POST /whiteboard/v1/whiteboard-sessions/{resource_id}/operator-remediation`
- Contract file: `microservices/whiteboard/contracts/local-openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `WhiteboardSessionOperatorRemediationApplied`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0117 — `whiteboard` — `GRPC RPC CheckLocalPolicy`
- Contract file: `microservices/whiteboard/contracts/local-operations-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `WhiteboardLocalPolicyDecisionRecorded`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0118 — `whiteboard` — `REST POST /whiteboard/actions/{action_id}`
- Contract file: `microservices/whiteboard/contracts/openapi-v1.yaml`
- Current named class: `NONE`
- Recommended ADR-0263 class: `WhiteboardActionAccepted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WhiteboardActionAccepted`
- IP cross-reference: `WhiteboardActionAccepted`
- Enumeration evidence: OpenAPI mutation method POST; details: no x-audit-event class

#### ENDPOINT-0119 — `whiteboard` — `GRPC RPC InvokeAction`
- Contract file: `microservices/whiteboard/contracts/whiteboard-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `WhiteboardActionInvoked`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `none`
- IP cross-reference: `none`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

#### ENDPOINT-0120 — `workplace-integration` — `ASYNC SEND publishWorkplaceESignSessionCreated`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current named class: `WorkplaceESignSessionCreated`
- Recommended ADR-0263 class: `WorkplaceESignSessionCreated`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceESignSessionCreated`
- IP cross-reference: `WorkplaceESignSessionCreated`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0121 — `workplace-integration` — `ASYNC SEND publishWorkplaceSignatureCaptured`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current named class: `WorkplaceSignatureCaptured`
- Recommended ADR-0263 class: `WorkplaceSignatureCaptured`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceSignatureCaptured`
- IP cross-reference: `WorkplaceSignatureCaptured`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0122 — `workplace-integration` — `ASYNC SEND publishWorkplaceOfferGenerated`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current named class: `WorkplaceOfferGenerated`
- Recommended ADR-0263 class: `WorkplaceOfferGenerated`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceOfferGenerated`
- IP cross-reference: `WorkplaceOfferGenerated`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0123 — `workplace-integration` — `ASYNC SEND publishWorkplaceAgreementBound`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current named class: `WorkplaceAgreementBound`
- Recommended ADR-0263 class: `WorkplaceAgreementBound`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceAgreementBound`
- IP cross-reference: `WorkplaceAgreementBound`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0124 — `workplace-integration` — `ASYNC SEND publishWorkplaceRosterBindingGranted`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current named class: `WorkplaceRosterBindingGranted`
- Recommended ADR-0263 class: `WorkplaceRosterBindingGranted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceRosterBindingGranted`
- IP cross-reference: `WorkplaceRosterBindingGranted`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0125 — `workplace-integration` — `ASYNC SEND publishWorkplaceClockEventAttested`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current named class: `WorkplaceClockEventAttested`
- Recommended ADR-0263 class: `WorkplaceClockEventAttested`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceClockEventAttested`
- IP cross-reference: `WorkplaceClockEventAttested`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0126 — `workplace-integration` — `ASYNC SEND publishWorkplaceDlpTraceSealed`
- Contract file: `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
- Current named class: `WorkplaceDlpTraceSealed`
- Recommended ADR-0263 class: `WorkplaceDlpTraceSealed`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceDlpTraceSealed`
- IP cross-reference: `WorkplaceDlpTraceSealed`
- Enumeration evidence: AsyncAPI publish operation action=send; details: audit_chain_ref present; required=tenant_id,sub_scope_path,event_id,occurred_at,audit_chain_ref

#### ENDPOINT-0127 — `workplace-integration` — `REST POST /workplace/esign/sessions`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current named class: `WorkplaceESignSessionCreated`
- Recommended ADR-0263 class: `WorkplaceESignSessionCreated`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceESignSessionCreated`
- IP cross-reference: `WorkplaceESignSessionCreated`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=WorkplaceESignSessionCreated

#### ENDPOINT-0128 — `workplace-integration` — `REST POST /workplace/esign/sessions/{session_id}/sign`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current named class: `WorkplaceESignSessionCreated`
- Recommended ADR-0263 class: `WorkplaceSignatureCaptured`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceESignSessionCreated,WorkplaceSignatureCaptured`
- IP cross-reference: `WorkplaceESignSessionCreated,WorkplaceSignatureCaptured`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=WorkplaceESignSessionCreated

#### ENDPOINT-0129 — `workplace-integration` — `REST POST /workplace/offer-letters`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current named class: `WorkplaceESignSessionCreated`
- Recommended ADR-0263 class: `WorkplaceOfferGenerated`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceESignSessionCreated,WorkplaceOfferGenerated`
- IP cross-reference: `WorkplaceESignSessionCreated,WorkplaceOfferGenerated`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=WorkplaceESignSessionCreated

#### ENDPOINT-0130 — `workplace-integration` — `REST POST /workplace/engagement-agreements`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current named class: `WorkplaceESignSessionCreated`
- Recommended ADR-0263 class: `WorkplaceAgreementBound`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceAgreementBound,WorkplaceESignSessionCreated`
- IP cross-reference: `WorkplaceAgreementBound,WorkplaceESignSessionCreated`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=WorkplaceESignSessionCreated

#### ENDPOINT-0131 — `workplace-integration` — `REST POST /workplace/roster-bindings`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current named class: `WorkplaceESignSessionCreated`
- Recommended ADR-0263 class: `WorkplaceRosterBindingGranted`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceESignSessionCreated,WorkplaceRosterBindingGranted`
- IP cross-reference: `WorkplaceESignSessionCreated,WorkplaceRosterBindingGranted`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=WorkplaceESignSessionCreated

#### ENDPOINT-0132 — `workplace-integration` — `REST POST /workplace/clock-events`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current named class: `WorkplaceESignSessionCreated`
- Recommended ADR-0263 class: `WorkplaceClockEventAttested`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceClockEventAttested,WorkplaceESignSessionCreated`
- IP cross-reference: `WorkplaceClockEventAttested,WorkplaceESignSessionCreated`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=WorkplaceESignSessionCreated

#### ENDPOINT-0133 — `workplace-integration` — `REST POST /workplace/dlp-traces`
- Contract file: `microservices/workplace-integration/contracts/openapi-v1.yaml`
- Current named class: `WorkplaceESignSessionCreated`
- Recommended ADR-0263 class: `WorkplaceDlpTraceSealed`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceDlpTraceSealed,WorkplaceESignSessionCreated`
- IP cross-reference: `WorkplaceDlpTraceSealed,WorkplaceESignSessionCreated`
- Enumeration evidence: OpenAPI mutation method POST; details: x-audit-event=WorkplaceESignSessionCreated

#### ENDPOINT-0134 — `workplace-integration` — `GRPC RPC SubmitWorkplaceAgreement`
- Contract file: `microservices/workplace-integration/contracts/workplace-integration-v1.proto`
- Current named class: `NONE`
- Recommended ADR-0263 class: `WorkplaceAgreementBound`
- Current class registered in ADR-0263: `false`
- Recommended class already registered in ADR-0263: `false`
- AsyncAPI cross-reference: `WorkplaceAgreementBound`
- IP cross-reference: `WorkplaceAgreementBound`
- Enumeration evidence: Proto rpc not classified as read/query by name; details: no nearby audit-event class comment/option

## §6 — Verdict

Verdict: `PROCEED-WITH-FIXES`

Rationale: the audit sweep completed cleanly and produced a strict-scope endpoint inventory, but the corpus is not ready for `APPROVE` because current state-changing endpoints either lack concrete audit-event classes, reuse classes across distinct transitions, or use concrete class names that are not registered in ADR-0263.

Checkpoint:
- Oya VCS claim: accepted for `docs/architecture` by `codex-audit-event-coverage-sweep` before this file was authored.
- Strict services scanned: 15.
- State-changing endpoints scanned: 134.
- Gaps catalogued: 134.
- Endpoints with current named class: 41.
- Endpoints with current ADR-0263 registered class: 0.
- Source files modified: none.
- Authored artifact: `docs/architecture/audit-event-coverage-sweep-2026-05-20.md`.

Stop condition for this wave: document exists, line count is at least 2,000, gap count is recorded for Oya VCS evidence, and Oya VCS verify/done/promote have been run with `sweep_lines:X gaps_catalogued:Y`.

