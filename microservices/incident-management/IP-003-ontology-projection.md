# IP-003 Incident Management ontology projection

Service: incident-management
ChangeSet scope: microservices/incident-management/IP-003-ontology-projection.md
Wave: 15-IP-substance conversion, 2026-05-21
Counterpart anchors: PagerDuty, OpsGenie, xMatters, FireHydrant, ServiceNow, Slack
Binding doctrine: ADR-0324 anti-template-stamping; ADR-0328 D-20 Big-8 P0 elevation

## A. Problem
This IP closes the ontology projection gap for `incident-management`. The previous 55-line shell repeated the same objective/prerequisite/test/rollback labels without proving how `ontology-projection` works for paging, escalation, incident rooms, stakeholder updates, and postmortems are time-critical and must keep evidence when systems are degraded.
The service is in the SRE incident-command Big-8 P0 operational concern lane. A generic platform answer is not enough: `on-call-schedule`, `escalation-policy`, and `postmortem` each carry tenant, principal, cell, data-class, pack, and audit consequences that differ from neighboring services.
The gap matters against PagerDuty and OpsGenie because those products make this capability feel native inside their workflow; Oyatie must match that usability while adding stronger Cedar, audit-chain, cell, and DealSet evidence.
The success condition is an implementation plan a cold engineer can trace from this IP to concrete files such as `microservices/incident-management/contracts/openapi-v1.yaml` without inventing a hidden service boundary.

## B. Approach
Implement `ontology-projection` as a service-local slice, not as a shared suite facility. The technical mechanism is mapping incident resources to shared ontology terms bound to `ontology projection port` and checked before user-visible promotion.
Use `on-call-schedule` as the first fixture path, then prove the same envelope across `escalation-policy` and `postmortem` so the design is not a one-object shortcut.
Every command or event carries `tenant_id`, `principal_id`, `audience_type=ONCALL_RESPONDER`, `home_cell`, `jurisdiction_code`, `data_class`, `traceparent`, `idempotency_key` for mutations, and an audit event class.
The domain layer stays pure in `microservices/incident-management/src/domain/mod.rs`; usecase orchestration lives in `microservices/incident-management/src/usecase/mod.rs`; transport or provider details stay behind adapter/config files.
Cedar fragments under `microservices/incident-management/policy/` and `microservices/incident-management/policies/` are the guard surface. A deny is a signed refusal with operator evidence, not an absent row or swallowed exception.
The approach explicitly covers field lineage, source-system ids, status semantics, and stale projection handling; those are the failure modes that a stamped IP did not name.

## C. Deliverables
- D01: `microservices/incident-management/PRD.md` — modify or bind for `ontology-projection` evidence and contract traceability.
- D02: `microservices/incident-management/ARCHITECTURE.md` — modify or bind for `ontology-projection` evidence and contract traceability.
- D03: `microservices/incident-management/manifest.json` — modify or bind for `ontology-projection` evidence and contract traceability.
- D04: `microservices/incident-management/competitor-parity-matrix.md` — modify or bind for `ontology-projection` evidence and contract traceability.
- D05: `microservices/incident-management/feature-parity-matrix-2026-05-20.md` — modify or bind for `ontology-projection` evidence and contract traceability.
- D06: `microservices/incident-management/contracts/openapi-v1.yaml` — modify or bind for `ontology-projection` evidence and contract traceability.
- D07: `microservices/incident-management/contracts/asyncapi-v1.yaml` — modify or bind for `ontology-projection` evidence and contract traceability.
- D08: `microservices/incident-management/contracts/incident-management-v1.proto` — modify or bind for `ontology-projection` evidence and contract traceability.
- D09: `microservices/incident-management/src/domain/mod.rs` — modify or bind for `ontology-projection` evidence and contract traceability.
- D10: `microservices/incident-management/src/usecase/mod.rs` — modify or bind for `ontology-projection` evidence and contract traceability.
- D11: `microservices/incident-management/src/adapter/http.rs` — modify or bind for `ontology-projection` evidence and contract traceability.
- D12: `microservices/incident-management/src/adapter/asyncapi.rs` — modify or bind for `ontology-projection` evidence and contract traceability.
- D13: `microservices/incident-management/policies/local-page-dispatch-guard.cedar` — modify or bind for `ontology-projection` evidence and contract traceability.
- D14: `microservices/incident-management/policies/local-escalation-policy-control.cedar` — modify or bind for `ontology-projection` evidence and contract traceability.
- D15: `microservices/incident-management/policies/local-war-room-open-approval.cedar` — modify or bind for `ontology-projection` evidence and contract traceability.
- D16: `microservices/incident-management/policy/sre-incident-command-authorization.cedar` — modify or bind for `ontology-projection` evidence and contract traceability.
- D17: `microservices/incident-management/capabilities/page-dispatch.yaml` — modify or bind for `ontology-projection` evidence and contract traceability.
- D18: `microservices/incident-management/capabilities/escalation-evaluate.yaml` — modify or bind for `ontology-projection` evidence and contract traceability.
- D25: `microservices/incident-management/REMEDIATION-NOTES-2026-05-21.md` or tier-scrub equivalent records this Wave 15 conversion outcome.

## D. Implementation steps
1. Read `microservices/incident-management/manifest.json` and confirm the Big-8 family, audience, compliance packs, cell eligibility, and benchmark list before editing code.
2. Add or update the `ontology-projection` contract shape in `contracts/openapi-v1.yaml` and keep request/event/proto field names aligned with ADR-0105 layer naming.
3. Add domain invariants in `microservices/incident-management/src/domain/mod.rs` for `page_event` and `escalation_policy` so tenant scope and immutable evidence are checked before adapters run.
4. Add usecase orchestration in `microservices/incident-management/src/usecase/mod.rs` with idempotency, trace context, and audit-chain emission before external side effects.
5. Bind adapter behavior through `ontology projection port` and `microservices/incident-management/src/adapter/mod.rs`, using typed errors from `microservices/incident-management/src/error.rs` instead of stringly provider failures.
6. Update the Cedar policy files listed above so `page-dispatch` and `escalation-evaluate` deny cross-tenant, stale-pack, and missing-purpose requests.
7. Update catalog rows under `microservices/incident-management/catalog/` so the layer registry names the owning crate, layer, capability id, and contract version.
8. Update dashboards/SLOs or the service operating bar to expose success, refusal, replay, latency, and audit completeness for `ontology-projection`.
9. Run focused contract/policy checks, then a service-level `cargo check` or equivalent if the crate graph is present.
10. Attach verification evidence to the remediation notes with the exact commands and changed IP list.

## E. Acceptance
- A reviewer can trace `ontology-projection` from this IP to at least one real contract, one real policy file, one real source file, and one capability/catalog artifact.
- The contract tests include accepted, duplicate-idempotency, Cedar-denied, stale-pack, wrong-tenant, and replay/backfill cases for `on-call-schedule`.
- The policy tests prove default deny for missing `tenant_id`, missing `principal_id`, wrong `audience_type`, stale `home_cell`, and data-class mismatch.
- The observability evidence includes metric, trace, structured log, audit event id, policy decision id, and low-cardinality labels for `ontology-projection`.
- The counterpart row below explains what Oyatie displaces from PagerDuty / OpsGenie / xMatters without claiming suite ownership or hiding substrate dependencies.
- Rollback is documented as contract version retreat, Cedar fragment pointer rollback, adapter feature flag off, and replay of idempotent commands from the backfill ledger.

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
- `microservices/incident-management/policies/local-page-dispatch-guard.cedar`
- `microservices/incident-management/policies/local-escalation-policy-control.cedar`
- `microservices/incident-management/policies/local-war-room-open-approval.cedar`
- `microservices/incident-management/policy/sre-incident-command-authorization.cedar`
- `microservices/incident-management/capabilities/page-dispatch.yaml`
- `microservices/incident-management/capabilities/escalation-evaluate.yaml`
- `microservices/incident-management/capabilities/incident-room-open.yaml`
- `microservices/incident-management/capabilities/stakeholder-update.yaml`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`

## G. Counterparts
| Counterpart | Gap closed by this IP | Oyatie substance requirement |
|---|---|---|
| PagerDuty | Provides a native `ontology projection` experience inside its product boundary. | Oyatie keeps the same operator fluency while binding `ontology-projection` to tenant scope, Cedar deny evidence, audit-chain records, and flat `incident-management` ownership. |
| OpsGenie | Competes on workflow speed and admin ergonomics. | Oyatie must prove the workflow through `page-dispatch` / `escalation-evaluate` with explicit policy and replay paths. |
| xMatters | Sets buyer expectation for enterprise reporting and integration. | Oyatie closes the gap through contracts, catalog rows, SLOs, and remediation evidence instead of a stamped parity claim. |
| FireHydrant | Pressures adjacent analytics or collaboration expectations. | Oyatie accepts the benchmark only where it maps to `postmortem` and does not weaken residency, audit, or data-class controls. |

## H. Non-goals and deletion check
- Do not move `ontology-projection` into a sibling service such as `observability` or `messenger` unless a later ADR changes ownership.
- Do not add Terraform, Cedar, or SDK claims for files that are not present; missing IaC remains a follow-up rather than fake HCL in this IP.
- No duplicative IP was deleted in this pass because this slice has a distinct contract/policy/evidence concern from its siblings.
