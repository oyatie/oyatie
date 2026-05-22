---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-012-cedar-policies-and-data-residency
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + ops-security
acceptance_lanes: [cedar-test, residency-test, oya-governance-pack-policy]
---

# IP-012: Cedar policies and data residency

## A. Problem
Calendar's strongest differentiator is policy-bounded scheduling, but that is only credible if Cedar fragments and residency rules cover event details, free/busy projection, public reads, auditor reads, and CI synthetic tenants.

## B. Approach
Validate and complete the existing calendar policy corpus without introducing cross-service policy shortcuts. Cedar denies raw event disclosure by default, allows scoped free/busy projection only through explicit grants, and binds pack residency rules to home cell and jurisdiction.

## C. Deliverables
| Artifact | Role |
|---|---|
| `policy/tenant-scope.cedar` | Tenant caller guard. |
| `policy/public-read.cedar` | Public projection guard. |
| `policy/auditor-scope.cedar` | Evidence-only auditor guard. |
| `policy/ci-scope.cedar` | Synthetic tenant CI guard. |
| `policy/event-isolation.md` | Personal/professional context rule. |
| `policy/data-residency.md` | Pack and region residency rule. |

## D. Ordered implementation steps
1. Parse every Cedar file and fail closed on syntax or schema mismatch.
2. Add allow/deny fixtures for event author, attendee, external invitee, auditor, CI, and public projection.
3. Test cross-tenant free/busy with and without invite grants.
4. Test personal/professional context refusal for raw metadata.
5. Bind residency examples to KR, EU, US healthcare, and default packs.
6. Confirm OpenAPI/AsyncAPI expose policy results without leaking denial internals.
7. Record policy version in audit/event payload examples.

## E. Acceptance
- Cedar parser/test command passes for all files under `microservices/calendar/policy/`.
- `cargo run -p oya-dev-cli -- gate validate data-residency --microservice calendar` passes.
- `cargo run -p oya-dev-cli -- gate validate dual-context-correctness --microservice calendar` passes.
- Public read tests expose only allowed free/busy/projection fields.
- Compliance references in `compliance.md` and `dpia.md` remain consistent.

## F. Evidence
- Policy corpus: `microservices/calendar/policy/`.
- Compliance: `compliance.md`, `dpia.md`, `threat-model.md`.
- Contracts: `contracts/openapi/calendar.yaml`, `contracts/asyncapi/calendar-events.yaml`.
- Packs: `packs/GDPR.md`, `packs/HIPAA.md`, `packs/KR-PIPA.md`, `packs/SOC2.md`.

## G. Counterpart comparison
Google and Outlook provide calendar permissions, but not Cedar-like tenant policy as a product artifact. Proton emphasizes privacy, and Calendly/Cal.com emphasize booking convenience. Oyatie's counterpart advantage is a testable policy corpus with residency and dual-context isolation attached to every calendar action.

## H. Foundation delivery expansion
- Deliverable detail: Cedar fixtures cover organizer, attendee, external invitee, auditor, CI, and anonymous public projection.
- Deliverable detail: policy version is included in free/busy, event mutation, RSVP, and import/export evidence.
- Deliverable detail: residency examples cover KR, EU, US healthcare, SOC2, and default packs.
- Deliverable detail: public read rules expose only intended projection fields.
- Deliverable detail: auditor rules expose evidence and suppress raw event bodies.
- Deliverable detail: CI rules use synthetic tenants and cannot become production allow rules.
- Deliverable detail: contracts surface policy result safely without leaking denial internals.
- Deliverable detail: Slack shared workspaces create comparison pressure for external participant boundaries.

## I. Acceptance expansion
- Acceptance detail: Cedar parser tests must include every file under `policy/`.
- Acceptance detail: allow/deny fixtures must include both positive and negative cases for every actor class.
- Acceptance detail: dual-context tests must deny raw event reads across personal/work boundaries.
- Acceptance detail: residency tests must reject writes to disallowed home-cell/pack combinations.
- Acceptance detail: public projection tests must prove free/busy minimality.
- Acceptance detail: contract examples must include policy result fields.
- Acceptance detail: compliance and DPIA links must resolve for every pack cited.
- Acceptance detail: Slack/Google/Outlook comparisons must stay grounded in permission and residency evidence.

## J. Evidence expansion
- Evidence detail: capture Cedar parser/test output for the calendar policy directory.
- Evidence detail: capture data-residency and dual-context gate outputs.
- Evidence detail: capture OpenAPI/AsyncAPI policy-result example validation.
- Evidence detail: cite `compliance.md`, `dpia.md`, and `threat-model.md`.
- Evidence detail: cite pack files for GDPR, HIPAA, KR-PIPA, and SOC2.
- Evidence detail: cite `policy/event-isolation.md` for personal/professional boundaries.
- Evidence detail: cite Slack as shared-collaboration pressure requiring explicit external participant policy.
