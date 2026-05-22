# IP-021 Learning Management SLO-gated promotion

Service: learning-management
ChangeSet scope: microservices/learning-management/IP-021-slo-gated-promotion.md
Wave: 15-IP-substance conversion, 2026-05-21
Counterpart anchors: Workday Learning, Cornerstone, Degreed, LinkedIn Learning, Udemy Business, Salesforce Trailhead
Binding doctrine: ADR-0324 anti-template-stamping; ADR-0328 D-20 Big-8 P0 elevation

## A. Problem
This IP closes the SLO-gated promotion gap for `learning-management`. The previous 55-line shell repeated the same objective/prerequisite/test/rollback labels without proving how `slo-gated-promotion` works for course catalogs, enrollment, learning paths, assessments, credential assertions, and regulated attestations need an auditable learning substrate.
The service is in the HR learning and credentialing Big-8 P0 adjacency lane. A generic platform answer is not enough: `course-catalog`, `enrollment`, and `credential` each carry tenant, principal, cell, data-class, pack, and audit consequences that differ from neighboring services.
The gap matters against Workday Learning and Cornerstone because those products make this capability feel native inside their workflow; Oyatie must match that usability while adding stronger Cedar, audit-chain, cell, and DealSet evidence.
The success condition is an implementation plan a cold engineer can trace from this IP to concrete files such as `microservices/learning-management/slos/*.openslo.yaml` without inventing a hidden service boundary.

## B. Approach
Implement `slo-gated-promotion` as a service-local slice, not as a shared suite facility. The technical mechanism is release gate on error budget and audit completeness bound to `promotion gate` and checked before user-visible promotion.
Use `course-catalog` as the first fixture path, then prove the same envelope across `enrollment` and `credential` so the design is not a one-object shortcut.
Every command or event carries `tenant_id`, `principal_id`, `audience_type=LEARNING_ADMIN`, `home_cell`, `jurisdiction_code`, `data_class`, `traceparent`, `idempotency_key` for mutations, and an audit event class.
The domain layer stays pure in `microservices/learning-management/src/domain/mod.rs`; usecase orchestration lives in `microservices/learning-management/src/usecase/mod.rs`; transport or provider details stay behind adapter/config files.
Cedar fragments under `microservices/learning-management/policy/` and `microservices/learning-management/policies/` are the guard surface. A deny is a signed refusal with operator evidence, not an absent row or swallowed exception.
The approach explicitly covers burn-rate windows, rollback trigger, evidence bundle; those are the failure modes that a stamped IP did not name.

## C. Deliverables
- D01: `microservices/learning-management/PRD.md` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D02: `microservices/learning-management/ARCHITECTURE.md` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D03: `microservices/learning-management/manifest.json` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D04: `microservices/learning-management/competitor-parity-matrix.md` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D05: `microservices/learning-management/feature-parity-matrix-2026-05-20.md` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D06: `microservices/learning-management/slos/*.openslo.yaml` — create or reconcile for `slo-gated-promotion` evidence and contract traceability.
- D07: `microservices/learning-management/contracts/openapi-v1.yaml` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D08: `microservices/learning-management/contracts/asyncapi-v1.yaml` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D09: `microservices/learning-management/contracts/learning-management-v1.proto` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D10: `microservices/learning-management/src/domain/mod.rs` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D11: `microservices/learning-management/src/usecase/mod.rs` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D12: `microservices/learning-management/src/adapter/mod.rs` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D13: `microservices/learning-management/src/config.rs` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D14: `microservices/learning-management/policies/local-course-publish-approval.cedar` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D15: `microservices/learning-management/policies/local-cohort-enrollment-scope.cedar` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D16: `microservices/learning-management/policies/local-certificate-issue-gate.cedar` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D17: `microservices/learning-management/policy/credential-training-authorization.cedar` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D18: `microservices/learning-management/capabilities/course-enroll.yaml` — modify or bind for `slo-gated-promotion` evidence and contract traceability.
- D26: `microservices/learning-management/REMEDIATION-NOTES-2026-05-21.md` or tier-scrub equivalent records this Wave 15 conversion outcome.

## D. Implementation steps
1. Read `microservices/learning-management/manifest.json` and confirm the Big-8 family, audience, compliance packs, cell eligibility, and benchmark list before editing code.
2. Add or update the `slo-gated-promotion` contract shape in `slos/*.openslo.yaml` and keep request/event/proto field names aligned with ADR-0105 layer naming.
3. Add domain invariants in `microservices/learning-management/src/domain/mod.rs` for `course_enrollment` and `completion_evidence` so tenant scope and immutable evidence are checked before adapters run.
4. Add usecase orchestration in `microservices/learning-management/src/usecase/mod.rs` with idempotency, trace context, and audit-chain emission before external side effects.
5. Bind adapter behavior through `promotion gate` and `microservices/learning-management/src/adapter/mod.rs`, using typed errors from `microservices/learning-management/src/error.rs` instead of stringly provider failures.
6. Update the Cedar policy files listed above so `course-enroll` and `completion-seal` deny cross-tenant, stale-pack, and missing-purpose requests.
7. Update catalog rows under `microservices/learning-management/catalog/` so the layer registry names the owning crate, layer, capability id, and contract version.
8. Update dashboards/SLOs or the service operating bar to expose success, refusal, replay, latency, and audit completeness for `slo-gated-promotion`.
9. Run focused contract/policy checks, then a service-level `cargo check` or equivalent if the crate graph is present.
10. Attach verification evidence to the remediation notes with the exact commands and changed IP list.

## E. Acceptance
- A reviewer can trace `slo-gated-promotion` from this IP to at least one real contract, one real policy file, one real source file, and one capability/catalog artifact.
- The contract tests include accepted, duplicate-idempotency, Cedar-denied, stale-pack, wrong-tenant, and replay/backfill cases for `course-catalog`.
- The policy tests prove default deny for missing `tenant_id`, missing `principal_id`, wrong `audience_type`, stale `home_cell`, and data-class mismatch.
- The observability evidence includes metric, trace, structured log, audit event id, policy decision id, and low-cardinality labels for `slo-gated-promotion`.
- The counterpart row below explains what Oyatie displaces from Workday Learning / Cornerstone / Degreed without claiming suite ownership or hiding substrate dependencies.
- Rollback is documented as contract version retreat, Cedar fragment pointer rollback, adapter feature flag off, and replay of idempotent commands from the backfill ledger.

## F. Evidence
- `microservices/learning-management/PRD.md`
- `microservices/learning-management/ARCHITECTURE.md`
- `microservices/learning-management/manifest.json`
- `microservices/learning-management/competitor-parity-matrix.md`
- `microservices/learning-management/feature-parity-matrix-2026-05-20.md`
- `microservices/learning-management/slos/*.openslo.yaml`
- `microservices/learning-management/contracts/openapi-v1.yaml`
- `microservices/learning-management/contracts/asyncapi-v1.yaml`
- `microservices/learning-management/contracts/learning-management-v1.proto`
- `microservices/learning-management/src/domain/mod.rs`
- `microservices/learning-management/src/usecase/mod.rs`
- `microservices/learning-management/src/adapter/mod.rs`
- `microservices/learning-management/src/config.rs`
- `microservices/learning-management/policies/local-course-publish-approval.cedar`
- `microservices/learning-management/policies/local-cohort-enrollment-scope.cedar`
- `microservices/learning-management/policies/local-certificate-issue-gate.cedar`
- `microservices/learning-management/policy/credential-training-authorization.cedar`
- `microservices/learning-management/capabilities/course-enroll.yaml`
- `microservices/learning-management/capabilities/completion-seal.yaml`
- `microservices/learning-management/capabilities/credential-issue.yaml`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`

## G. Counterparts
| Counterpart | Gap closed by this IP | Oyatie substance requirement |
|---|---|---|
| Workday Learning | Provides a native `SLO-gated promotion` experience inside its product boundary. | Oyatie keeps the same operator fluency while binding `slo-gated-promotion` to tenant scope, Cedar deny evidence, audit-chain records, and flat `learning-management` ownership. |
| Cornerstone | Competes on workflow speed and admin ergonomics. | Oyatie must prove the workflow through `course-enroll` / `completion-seal` with explicit policy and replay paths. |
| Degreed | Sets buyer expectation for enterprise reporting and integration. | Oyatie closes the gap through contracts, catalog rows, SLOs, and remediation evidence instead of a stamped parity claim. |
| LinkedIn Learning | Pressures adjacent analytics or collaboration expectations. | Oyatie accepts the benchmark only where it maps to `credential` and does not weaken residency, audit, or data-class controls. |

## H. Non-goals and deletion check
- Do not move `slo-gated-promotion` into a sibling service such as `community` or `workflow-engine` unless a later ADR changes ownership.
- Do not add Terraform, Cedar, or SDK claims for files that are not present; missing IaC remains a follow-up rather than fake HCL in this IP.
- No duplicative IP was deleted in this pass because this slice has a distinct contract/policy/evidence concern from its siblings.
