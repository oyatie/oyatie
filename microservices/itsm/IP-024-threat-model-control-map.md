---
doc_class: IP
ip_id: IP-024-threat-model-control-map
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + security
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/policy/service-management-authorization.cedar
  - microservices/itsm/policy/abuse-defence.cedar
  - microservices/itsm/src/domain/mod.rs
  - microservices/itsm/contracts/openapi-v1.yaml
---

# IP-024 ITSM Threat Model Control Map

## A. Problem
ITSM is a high-value target: attackers can use it to exfiltrate incident notes, change production systems, poison CMDB relations, phish through status updates, or abuse service-catalog fulfillment. The stamped IP did not map threats to controls.

This IP creates a control map for the concrete ITSM surfaces already in the repo.

## B. Approach
Map threats to controls and evidence:

| Threat | Control |
|---|---|
| cross-tenant ticket read | Cedar tenant-scope policy |
| unauthorized change approval | change freeze/risk policy |
| CMDB relation poisoning | relation write policy + audit |
| KB/RAG data leakage | data residency + article policy |
| status update phishing | audience scope and template review |
| credential theft | OpenBao sidecar references |
| portal abuse | WAF + abuse Cedar |

## C. Deliverables
- Threat/control matrix covering REST, gRPC, AsyncAPI, policy, credentials, and dashboards.
- Tests or planned tests for top misuse cases.
- Control owners for each policy file.
- Link to chaos drill pack for failure behavior.
- Counterpart risk comparison against ServiceNow/Jira/Freshservice administrative surfaces.

## D. Implementation
1. Inventory public entrypoints in `contracts/openapi-v1.yaml` and internal entrypoints in `contracts/itsm-v1.proto`.
2. Inventory policy files in `policy/` and `policies/`.
3. For each `Capability`, document abuse case, control, audit event, and rollback.
4. Add misuse tests for cross-tenant access, unauthorized change approval, and CMDB write.
5. Add status-update phishing controls: template allowlist, audience scope, and audit review.
6. Add credential theft control reference to IP-009.
7. Add chaos drill references for audit outage, Cedar mismatch, and regional outage.
8. Keep residual risks explicit rather than hiding them behind "standard controls."

## E. Acceptance
- Every ITSM capability has at least one named threat and control.
- Cross-tenant, unauthorized change, and CMDB poisoning paths have tests or explicit follow-up.
- Control map cites real policy files and code types.
- Residual risks are documented with owner and next evidence.

## F. Evidence
- `policy/service-management-authorization.cedar` and `policy/abuse-defence.cedar` exist.
- Local policy fragments under `policies/` cover incident, change, CMDB, SLA, problem, and KB flows.
- `src/domain/mod.rs` defines capabilities and audit event kinds.
- ADR-0243, ADR-0244, and ADR-0263 govern Cedar, tenant, and audit controls.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow security controls | Threats mapped to Oyatie Cedar/audit evidence |
| Jira Service Management admin/project risks | Project-style authority cannot cross tenant gates |
| Freshservice portal and workflow risks | Abuse, credential, and status-update threats are explicit |

## H. Cold-start buildability notes
- Inventory entrypoints before naming threats.
- Map every capability to at least one misuse case.
- Start tests with cross-tenant read and unauthorized change approval.
- Keep status-update phishing controls separate from portal abuse.
- Link each threat to a specific policy file.
- Record residual risks with an owner.
- Do not use generic "standard controls" as evidence.
- Include credential theft controls from IP-009.
- Include chaos drill references from IP-022.
- Keep counterpart comparison at risk-surface level.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`, `.proto`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
