# IP-001 Incident Management tenant scope kernel

Service: incident-management
ChangeSet scope: microservices/incident-management/IP-001-tenant-scope-kernel.md
Wave: 15-IP-substance conversion, 2026-05-21
Counterpart anchors: PagerDuty, OpsGenie, xMatters, FireHydrant, ServiceNow, Slack
Binding doctrine: ADR-0324 anti-template-stamping; ADR-0328 D-20 Big-8 P0 elevation

## A. Problem
This IP replaces a long stamped shell for `tenant scope kernel` with an incident-command implementation slice. The old file repeated `objective`, `prerequisites`, `implementation steps`, `tests`, `rollback`, and `acceptance criteria` labels but never explained how paging, escalation, incident-room opening, status updates, or postmortem sealing move through the service.
Incident Management is T0/SRE critical: during an outage, missing tenant scope or a permissive policy is not a paperwork gap. It can page the wrong responder, leak a war-room timeline, publish the wrong stakeholder update, or lose the postmortem seal that auditors need after recovery.
PagerDuty, OpsGenie, xMatters, FireHydrant, ServiceNow, and Slack set the operator expectation for speed. Oyatie closes the gap by keeping the same fast path while proving tenant, cell, Cedar, workflow, audit-chain, and rollback evidence on every critical transition.

## B. Approach
The technical mechanism is a tenant-scope kernel for incident command identifiers and cell-local state. It is implemented through the existing flat `incident-management` ownership boundary, not a vendor-named suite or a generic platform helper.
Use `policies/local-page-dispatch-guard.cedar` as the first guard surface and confirm every mutation carries `tenant_id`, `principal_id`, `audience_type=ONCALL_RESPONDER`, `home_cell`, `jurisdiction_code`, `data_class`, `traceparent`, `idempotency_key`, and `audit_event_class`.
Domain rules stay in `src/domain/mod.rs`; orchestration stays in `src/usecase/mod.rs`; HTTP, AsyncAPI, and gRPC adapters remain in `src/adapter/`. The policy files deny before the usecase mutates state.
The slice must be proven against `page-dispatch`, `escalation-evaluate`, `incident-room-open`, `stakeholder-update`, `postmortem-seal`, and `statuspage-sync`, not only a single happy-path page event.

## C. Deliverables
- D01: `microservices/incident-management/PRD.md` - bind or verify for this IP.
- D02: `microservices/incident-management/ARCHITECTURE.md` - bind or verify for this IP.
- D03: `microservices/incident-management/manifest.json` - bind or verify for this IP.
- D04: `microservices/incident-management/competitor-parity-matrix.md` - bind or verify for this IP.
- D05: `microservices/incident-management/feature-parity-matrix-2026-05-20.md` - bind or verify for this IP.
- D06: `microservices/incident-management/contracts/openapi-v1.yaml` - bind or verify for this IP.
- D07: `microservices/incident-management/contracts/asyncapi-v1.yaml` - bind or verify for this IP.
- D08: `microservices/incident-management/contracts/incident-management-v1.proto` - bind or verify for this IP.
- D09: `microservices/incident-management/src/domain/mod.rs` - bind or verify for this IP.
- D10: `microservices/incident-management/src/usecase/mod.rs` - bind or verify for this IP.
- D11: `microservices/incident-management/src/adapter/http.rs` - bind or verify for this IP.
- D12: `microservices/incident-management/src/adapter/asyncapi.rs` - bind or verify for this IP.
- D13: `microservices/incident-management/src/adapter/grpc.rs` - bind or verify for this IP.
- D14: `microservices/incident-management/policies/local-page-dispatch-guard.cedar` - bind or verify for this IP.
- D15: `microservices/incident-management/policies/local-escalation-policy-control.cedar` - bind or verify for this IP.
- D16: `microservices/incident-management/policies/local-war-room-open-approval.cedar` - bind or verify for this IP.
- D17: `microservices/incident-management/policies/local-postmortem-seal-required.cedar` - bind or verify for this IP.
- D18: `microservices/incident-management/policy/sre-incident-command-authorization.cedar` - bind or verify for this IP.
- D19: `microservices/incident-management/capabilities/page-dispatch.yaml` - bind or verify for this IP.
- D20: `microservices/incident-management/capabilities/escalation-evaluate.yaml` - bind or verify for this IP.
- D21: `microservices/incident-management/capabilities/incident-room-open.yaml` - bind or verify for this IP.
- D22: `microservices/incident-management/capabilities/stakeholder-update.yaml` - bind or verify for this IP.
- D23: `microservices/incident-management/capabilities/postmortem-seal.yaml` - bind or verify for this IP.
- D24: `microservices/incident-management/capabilities/statuspage-sync.yaml` - bind or verify for this IP.
- D25: `microservices/incident-management/catalog/oya-incident-management-sre-incident-command-domain.yaml` - bind or verify for this IP.
- D26: `microservices/incident-management/catalog/oya-incident-management-sre-incident-command-usecase.yaml` - bind or verify for this IP.

## D. Implementation steps
1. Confirm manifest criticality, eligible cell tiers, compliance packs, and dependency list before changing contracts.
2. Bind the contract surface in `contracts/openapi-v1.yaml`, `contracts/asyncapi-v1.yaml`, and `contracts/incident-management-v1.proto` so the same invariant names appear in each protocol.
3. Add pure domain checks for incident id, escalation policy id, responder identity, incident-room classification, stakeholder audience, and postmortem seal state.
4. Wire usecase orchestration with idempotency, workflow-engine correlation, audit-chain emission, and typed refusal errors.
5. Evaluate Cedar before adapter work; policy denial returns a signed decision id and safe operator detail.
6. Register or verify catalog rows for domain/usecase/api/rest/worker/sdk layers so ADR-0105 ownership is inspectable.
7. Update SLO, dashboard, and runbook evidence for accepted, denied, duplicate, replayed, and degraded-mode paths.
8. Run focused policy/contract tests and service `cargo check` where the local crate graph permits it.
9. Record the verification command and residual risk in remediation notes.

## E. Acceptance
- Every mutation fails closed when tenant_id, principal_id, purpose, data_class, or audit_event_class is absent.
- A test fixture covers page dispatch, escalation evaluation, incident-room open, stakeholder update, postmortem seal, and statuspage sync.
- Wrong-tenant and stale-cell cases produce Cedar denial evidence, not silent filtering.
- Audit events carry incident id, responder id, escalation policy id, workflow run id, policy decision id, and trace id.
- Rollback can disable the adapter or policy fragment and replay idempotent commands from the backfill ledger.
- A reviewer can trace every claim in this IP to a real path listed in Section F.

## F. Evidence
- `microservices/incident-management/PRD.md`
- `microservices/incident-management/ARCHITECTURE.md`
- `microservices/incident-management/manifest.json`
- `microservices/incident-management/competitor-parity-matrix.md`
- `microservices/incident-management/feature-parity-matrix-2026-05-20.md`
- `microservices/incident-management/contracts/openapi-v1.yaml`
- `microservices/incident-management/contracts/asyncapi-v1.yaml`
- `microservices/incident-management/contracts/incident-management-v1.proto`
- `microservices/incident-management/src/domain/mod.rs`
- `microservices/incident-management/src/usecase/mod.rs`
- `microservices/incident-management/src/adapter/http.rs`
- `microservices/incident-management/src/adapter/asyncapi.rs`
- `microservices/incident-management/src/adapter/grpc.rs`
- `microservices/incident-management/policies/local-page-dispatch-guard.cedar`
- `microservices/incident-management/policies/local-escalation-policy-control.cedar`
- `microservices/incident-management/policies/local-war-room-open-approval.cedar`
- `microservices/incident-management/policies/local-postmortem-seal-required.cedar`
- `microservices/incident-management/policy/sre-incident-command-authorization.cedar`
- `microservices/incident-management/capabilities/page-dispatch.yaml`
- `microservices/incident-management/capabilities/escalation-evaluate.yaml`
- `microservices/incident-management/capabilities/incident-room-open.yaml`
- `microservices/incident-management/capabilities/stakeholder-update.yaml`
- `microservices/incident-management/capabilities/postmortem-seal.yaml`
- `microservices/incident-management/capabilities/statuspage-sync.yaml`
- `microservices/incident-management/catalog/oya-incident-management-sre-incident-command-domain.yaml`
- `microservices/incident-management/catalog/oya-incident-management-sre-incident-command-usecase.yaml`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`

## G. Counterparts
| Counterpart | Gap closed | Oyatie closure |
|---|---|---|
| PagerDuty / OpsGenie | Fast paging and escalation ergonomics. | Same speed target, with tenant-scoped policy decisions and audit-chain records. |
| xMatters / FireHydrant | Incident-room coordination, stakeholder updates, and postmortem workflow. | War-room and postmortem state remain policy-bound, replayable, and inspectable. |
| ServiceNow / Slack | ITSM integration and collaboration expectations. | Oyatie integrates through contracts and workflow/audit evidence without moving incident ownership out of this service. |

## H. Non-goals
- Do not bypass Cedar for emergency or degraded incident paths.
- Do not move incident command into observability, messenger, workflow-engine, or tasks.
- Do not invent Terraform or SDK files that are not in the service path.
