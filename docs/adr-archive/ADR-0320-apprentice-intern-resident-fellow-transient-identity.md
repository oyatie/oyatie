---
id: ADR-0320
title: Apprentice, Intern, Resident, and Fellow Transient Identity Doctrine
status: Rejected
date: 2026-05-20
doc_class: architecture_decision_record
owners:
  - identity-platform
  - community-platform
  - workplace-integration
  - audit-chain
  - payments-platform
amends:
  - ADR-0244
  - ADR-0311
  - ADR-0313
related_adrs:
  - ADR-0244
  - ADR-0292
  - ADR-0299
  - ADR-0305
  - ADR-0311
  - ADR-0313
related_specs:
  - /specs/root-hub-pointers.json
  - /specs/master-plan-sequencing.json
  - /specs/tenant-model.json
implementation_units:
  - crates/oya-shared-program-transient-identity
  - services/community
  - services/identity
  - services/workplace-integration
  - services/payments
  - services/audit-chain
program_type_enum:
  - APPRENTICE
  - INTERN
  - RESIDENT
  - FELLOW
  - COOP
  - EXTERN
verification_expectations:
  - line_count_at_least_1500
  - no_todo_tbd_placeholders
  - cross_refs_present
  - regulatory_articles_exactly_named
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Keep Rejected: Apprentice/intern transient identity — niche; reopen with IAM pack
# ADR-0320: Apprentice, Intern, Resident, and Fellow Transient Identity Doctrine

Status: Proposed

Date: 2026-05-20

Decision owner: identity-platform with community, workplace-integration, payments, and audit-chain consumers.

Scope: this ADR defines the canonical doctrine for short-term, cross-tenant program identities that participate in education, work, credentialing, supervision, and compensation workflows without collapsing a person's personal tenant into the host organization tenant.

Non-scope: this ADR does not replace the universal tenant model from ADR-0244, the minor doctrine from ADR-0292, the fairness doctrine from ADR-0299, the mentor doctrine from ADR-0305, the dual personal/work boundary from ADR-0311, or the conglomerate hierarchy model from ADR-0313.

## A. Context

A.1 Oyatie models tenant as the universal scoping primitive. ADR-0244 already allows a tenant to represent an institution, employer, household, school, cohort, program, marketplace, or audience-defined operating cell. Apprentices, interns, residents, fellows, cooperative education students, and externs need that primitive to act as more than an employment label.

A.2 These identities are transient because a person joins a host program for a bounded interval, gains scoped capability while enrolled, and must lose that capability when the program ends. The bounded interval is not merely a UI field; it is an authorization boundary, a payroll boundary, a labor-law boundary, an audit boundary, and a portfolio-retention boundary.

A.3 These identities are multi-tenant because one person commonly has at least three relevant scopes: a personal tenant, a source or school tenant, and a host or employer tenant. Healthcare residents often add a hospital system, a graduate medical education program, and a supervising attending. Apprentices can add unions, government workforce agencies, and credentialing bodies.

A.4 ADR-0311 separates personal identity from work identity. ADR-0320 applies that separation to program participants so the host tenant receives operational control only over the host-facing role, while the personal tenant retains portable artifacts such as verified hours, program completion claims, attestations, and shareable portfolio entries.

A.5 ADR-0313 permits sovereign child tenants inside a conglomerate. Program tenants use that capability when a parent corporation, university system, or hospital network sponsors many child program tenants. Each child tenant can enforce local policy and jurisdictional overlays while remaining visible to the parent governance plane.

A.6 ADR-0292 matters because many program participants are minors or near-minors. A high-school extern, youth apprentice, or early college intern can be under age-of-majority rules. The minor doctrine controls guardian consent, age gates, data minimization, and restricted messaging regardless of host-tenant convenience.

A.7 ADR-0299 matters because program identity systems are high-risk fairness systems. Candidate assignment, mentor pairing, evaluation, pay eligibility, and conversion-to-employee recommendations can create disparate impact. The doctrine therefore requires audit events and reviewable explanation records.

A.8 ADR-0305 matters because apprenticeships, internships, residencies, and fellowships are supervised relationships. Mentor, preceptor, attending, manager, sponsor, and reviewer identities must be first-class policy subjects, not informal profile links.

A.9 The common product mistake is to treat apprentice or intern as an HR title under a company account. That loses school sponsorship, personal continuity, regulated hours, mentor obligations, and jurisdiction-specific pay rules. Oyatie instead treats the program as a transient cross-tenant identity relationship.

A.10 The common security mistake is to leave permissions on an employee-like group and rely on HR offboarding. ADR-0320 forbids that pattern. Every program capability is time-bound, tenant-scoped, event-backed, and auto-revoked when the program ends or when an eligibility predicate fails.

A.11 The common compliance mistake is to assume one internship doctrine globally. The US FLSA, EU predictable working conditions regime, Korean labor standards, ACGME duty-hour rules, and NLRA protected activity rules create different overlays. Oyatie stores those overlays as policy facts rather than prose exceptions.

A.12 The common data-model mistake is to make a student identity disappear at program end. ADR-0320 keeps operational host capabilities revocable while letting the personal tenant retain portable completion, work sample, credential hour, evaluation, and recommendation artifacts according to consent and retention policy.

### A.13 External precedent summary

A.13.1 LinkedIn student and early-talent recruiting surfaces: show that a durable personal professional profile can be connected to temporary student and employer recruiting contexts without making the employer own the whole person record.
A.13.2 Handshake university and employer networks: show the three-party student, school, and employer pattern that program tenants must represent natively.
A.13.3 Workday early-career and learning programs: show that internships and apprenticeship-style pipelines need HR, learning, skills, and manager workflows tied to a bounded program interval.
A.13.4 SAP SuccessFactors career development and learning programs: show that competency progression, goals, and assessments are product primitives rather than free-form notes.
A.13.5 Epic and clinical training workflows: show that healthcare trainees require tightly scoped clinical-system access and supervision records.
A.13.6 ACGME graduate medical education rules: show that residents have exact duty-hour and supervision obligations that must be enforceable across clinical schedules.
A.13.7 US Department of Labor FLSA internship guidance: shows that unpaid intern classification is fact-sensitive and must not be hidden behind a generic program label.
A.13.8 EU Directive 2019/1152 and the EU Quality Framework for Traineeships: show that written terms, information duties, predictability, duration, and working conditions are program facts.
A.13.9 Korean Labor Standards Act and Minimum Wage Act: show that written terms, work-hour limits, rest, leave, and wage floor facts apply when the relationship is employee-like.
A.13.10 NLRA Section 7 protections: show that protected concerted activity can attach to student-worker and trainee relationships in covered contexts.

## B. Decision

B.1 Oyatie SHALL introduce `program_type` as a closed enum with the initial values `APPRENTICE`, `INTERN`, `RESIDENT`, `FELLOW`, `COOP`, and `EXTERN`.

B.2 Oyatie SHALL model each program participation as a `program_tenant_membership` that connects `person_id`, `personal_tenant_id`, `source_tenant_id`, `host_tenant_id`, and `program_tenant_id`.

B.3 Oyatie SHALL treat the program tenant as the policy anchor for transient capabilities, not as a decorative grouping under an employer account.

B.4 Oyatie SHALL keep host-operational claims separate from personal-portfolio claims. Host claims end with the program unless explicitly converted; personal portfolio claims survive if consent, retention, and evidence predicates pass.

B.5 Oyatie SHALL express program capabilities as time-bound Cedar `permit` policies with `when` clauses over program dates, status, supervision, jurisdiction, minor status, and evidence facts.

B.6 Oyatie SHALL auto-revoke active program capabilities at `program_end_at`, at early termination, at source-tenant withdrawal, at loss of guardian consent for a minor participant, or at labor-overlay disqualification.

B.7 Oyatie SHALL publish shared domain contracts in a new crate named `oya-shared-program-transient-identity` and SHALL avoid duplicating program-type rules in consumer microservices.

B.8 Oyatie SHALL require consumers in community, identity, workplace-integration, payments, and audit-chain to use the shared crate for program membership validation, event names, capability tier checks, and portfolio-survival semantics.

B.9 Oyatie SHALL attach jurisdiction overlays by exact legal reference. The overlay stores statute, article, section, jurisdiction, effective date, affected program types, classification condition, and enforcement action.

B.10 Oyatie SHALL support per-program capability tiers: observer, contributor, supervised-operator, compensated-worker, clinical-trainee, research-trainee, and portfolio-only.

B.11 Oyatie SHALL record hours and competency evidence against both host operational context and personal portable context, with explicit redaction rules for confidential employer, patient, or school data.

B.12 Oyatie SHALL emit audit-chain events for creation, approval, capability grant, mentor assignment, jurisdiction-overlay attachment, hour certification, evaluation, revocation, conversion, and portfolio export.

### B.13 Canonical enum shape

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProgramType {
    Apprentice,
    Intern,
    Resident,
    Fellow,
    Coop,
    Extern,
}
```

### B.14 Capability tier rule

A capability tier is not a rank, job title, or payroll class. It is a policy bundle that answers four questions: what the participant may do, under whose supervision, for which tenant, and until which revocation boundary.

B.14.observer: may view approved learning materials and observe limited workflow artifacts without production mutation rights.
B.14.contributor: may submit artifacts, reflections, work samples, and issue-linked deliverables for review.
B.14.supervised_operator: may perform bounded operational tasks only under assigned mentor or supervisor attestation.
B.14.compensated_worker: may receive wage, stipend, or payroll-eligible records when labor overlay permits or requires it.
B.14.clinical_trainee: may access clinical training workflows under supervision, duty-hour, patient-safety, and privacy constraints.
B.14.research_trainee: may access approved research tasks and datasets under protocol, consent, and publication controls.
B.14.portfolio_only: may retain personal-tenant artifacts after host permissions are revoked.

## C. Consequences

### C.1 Maintainability

C.1.1 The doctrine centralizes program-type and transient-identity semantics in one shared crate.
C.1.2 Consumer services lose freedom to define their own internship or residency rules.
C.1.3 The cost is stricter migration work when a program type changes.
C.1.4 The benefit is fewer hidden authorization variants and fewer divergent compliance interpretations.
C.1.5 ADR-0244 remains the tenant primitive source of truth, so the new model does not fork tenancy.

### C.2 Observability

C.2.1 Every grant, revocation, overlay decision, mentor assignment, and portfolio export becomes audit-chain visible.
C.2.2 The model requires correlation identifiers across source, host, personal, and program tenants.
C.2.3 The cost is more event volume.
C.2.4 The benefit is a defensible evidence trail for labor, supervision, safety, and fairness reviews.
C.2.5 ADR-0299 fairness checks can consume standardized program events rather than per-service logs.

### C.3 Scalability

C.3.1 Program tenants scale horizontally across schools, cohorts, employers, clinics, unions, and conglomerate children.
C.3.2 The model avoids one mega-tenant for all interns or all residents.
C.3.3 The cost is more tenant relationships.
C.3.4 The benefit is clean sharding, clean sovereignty, and clean revocation by bounded membership.
C.3.5 ADR-0313 parent-child tenant mechanics can aggregate without centralizing all policy decisions.

### C.4 Performance

C.4.1 Authorization checks become predicate-rich because they evaluate dates, jurisdiction facts, status, minor facts, and supervision facts.
C.4.2 The policy engine therefore needs cached normalized membership facts.
C.4.3 The cost is cache invalidation on program-end and overlay updates.
C.4.4 The benefit is deterministic policy evaluation without making services re-query every source tenant.
C.4.5 Program-ending revocation events are batchable because they are scheduled from canonical dates.

### C.5 Optimization

C.5.1 The model creates a path for optimized placement, mentoring, schedule, and credential-hour workflows.
C.5.2 Optimization must be constrained by fairness and labor overlays.
C.5.3 The cost is less freedom for ranking algorithms.
C.5.4 The benefit is that recommendations can be inspected against exact program facts and protected-class safeguards.
C.5.5 Optimization outputs must remain advisory unless a downstream service records human approval.

### C.6 Code quality

C.6.1 The shared crate prevents stringly typed program roles and duplicated date-window checks.
C.6.2 Typed events reduce accidental contract drift across community, identity, workplace-integration, payments, and audit-chain.
C.6.3 The cost is that consumers must depend on a domain package rather than local enums.
C.6.4 The benefit is simpler review: forbidden behavior can be searched in one policy vocabulary.
C.6.5 Tests can fixture the shared crate once and reuse the same invariants across services.

## D. Detailed mechanics

The mechanics below are normative. A consumer may add narrower local policy only when it preserves these invariants and records the narrower rule as an overlay fact.

### D-1. Program tenant membership schema

D-1.1 This primitive is required for ADR-0320 conformance and is enforced at service boundary, event boundary, and policy boundary.
D-1.2 The primitive derives its tenancy semantics from ADR-0244 and its personal/work separation from ADR-0311.
D-1.3 The primitive must be compatible with minor controls from ADR-0292, fairness controls from ADR-0299, mentor controls from ADR-0305, and hierarchy controls from ADR-0313.
D-1.4 Hyperscaler precedent one: enterprise identity systems separate organization membership from personal identity so that access can be revoked without deleting a person profile.
D-1.5 Hyperscaler precedent two: cloud IAM systems model temporary credentials and time-bounded sessions as normal primitives, not exception paths.
D-1.6 Regulatory precedent one: labor and training regimes attach duties to the relationship interval, so the model stores start and end dates as policy inputs.
D-1.7 Regulatory precedent two: education, clinical, and workforce programs rely on supervised authorization, so the model stores supervisor and mentor claims as policy inputs.
D-1.8 Audit rule: every state transition for this primitive emits an audit-chain event with actor, subject, tenants, source evidence, and before/after status.
D-1.9 Revocation rule: active privileges granted through this primitive become invalid when the membership status is not active or the current time is outside the allowed interval.
D-1.10 Privacy rule: personal-tenant retention stores only portable evidence and redacts host-confidential, patient-confidential, school-confidential, and trade-secret content.
D-1.11 Fairness rule: ranking, evaluation, placement, conversion, and mentor assignment using this primitive must expose reviewable inputs for ADR-0299 checks.
D-1.12 Test rule: the shared crate must include positive, negative, boundary-date, early-revocation, minor, and jurisdiction-overlay fixtures for this primitive.

### D-2. Cedar entity model

D-2.1 This primitive is required for ADR-0320 conformance and is enforced at service boundary, event boundary, and policy boundary.
D-2.2 The primitive derives its tenancy semantics from ADR-0244 and its personal/work separation from ADR-0311.
D-2.3 The primitive must be compatible with minor controls from ADR-0292, fairness controls from ADR-0299, mentor controls from ADR-0305, and hierarchy controls from ADR-0313.
D-2.4 Hyperscaler precedent one: enterprise identity systems separate organization membership from personal identity so that access can be revoked without deleting a person profile.
D-2.5 Hyperscaler precedent two: cloud IAM systems model temporary credentials and time-bounded sessions as normal primitives, not exception paths.
D-2.6 Regulatory precedent one: labor and training regimes attach duties to the relationship interval, so the model stores start and end dates as policy inputs.
D-2.7 Regulatory precedent two: education, clinical, and workforce programs rely on supervised authorization, so the model stores supervisor and mentor claims as policy inputs.
D-2.8 Audit rule: every state transition for this primitive emits an audit-chain event with actor, subject, tenants, source evidence, and before/after status.
D-2.9 Revocation rule: active privileges granted through this primitive become invalid when the membership status is not active or the current time is outside the allowed interval.
D-2.10 Privacy rule: personal-tenant retention stores only portable evidence and redacts host-confidential, patient-confidential, school-confidential, and trade-secret content.
D-2.11 Fairness rule: ranking, evaluation, placement, conversion, and mentor assignment using this primitive must expose reviewable inputs for ADR-0299 checks.
D-2.12 Test rule: the shared crate must include positive, negative, boundary-date, early-revocation, minor, and jurisdiction-overlay fixtures for this primitive.

### D-3. Cross-tenant identity attestation

D-3.1 This primitive is required for ADR-0320 conformance and is enforced at service boundary, event boundary, and policy boundary.
D-3.2 The primitive derives its tenancy semantics from ADR-0244 and its personal/work separation from ADR-0311.
D-3.3 The primitive must be compatible with minor controls from ADR-0292, fairness controls from ADR-0299, mentor controls from ADR-0305, and hierarchy controls from ADR-0313.
D-3.4 Hyperscaler precedent one: enterprise identity systems separate organization membership from personal identity so that access can be revoked without deleting a person profile.
D-3.5 Hyperscaler precedent two: cloud IAM systems model temporary credentials and time-bounded sessions as normal primitives, not exception paths.
D-3.6 Regulatory precedent one: labor and training regimes attach duties to the relationship interval, so the model stores start and end dates as policy inputs.
D-3.7 Regulatory precedent two: education, clinical, and workforce programs rely on supervised authorization, so the model stores supervisor and mentor claims as policy inputs.
D-3.8 Audit rule: every state transition for this primitive emits an audit-chain event with actor, subject, tenants, source evidence, and before/after status.
D-3.9 Revocation rule: active privileges granted through this primitive become invalid when the membership status is not active or the current time is outside the allowed interval.
D-3.10 Privacy rule: personal-tenant retention stores only portable evidence and redacts host-confidential, patient-confidential, school-confidential, and trade-secret content.
D-3.11 Fairness rule: ranking, evaluation, placement, conversion, and mentor assignment using this primitive must expose reviewable inputs for ADR-0299 checks.
D-3.12 Test rule: the shared crate must include positive, negative, boundary-date, early-revocation, minor, and jurisdiction-overlay fixtures for this primitive.

### D-4. Per-program capability tiers

D-4.1 This primitive is required for ADR-0320 conformance and is enforced at service boundary, event boundary, and policy boundary.
D-4.2 The primitive derives its tenancy semantics from ADR-0244 and its personal/work separation from ADR-0311.
D-4.3 The primitive must be compatible with minor controls from ADR-0292, fairness controls from ADR-0299, mentor controls from ADR-0305, and hierarchy controls from ADR-0313.
D-4.4 Hyperscaler precedent one: enterprise identity systems separate organization membership from personal identity so that access can be revoked without deleting a person profile.
D-4.5 Hyperscaler precedent two: cloud IAM systems model temporary credentials and time-bounded sessions as normal primitives, not exception paths.
D-4.6 Regulatory precedent one: labor and training regimes attach duties to the relationship interval, so the model stores start and end dates as policy inputs.
D-4.7 Regulatory precedent two: education, clinical, and workforce programs rely on supervised authorization, so the model stores supervisor and mentor claims as policy inputs.
D-4.8 Audit rule: every state transition for this primitive emits an audit-chain event with actor, subject, tenants, source evidence, and before/after status.
D-4.9 Revocation rule: active privileges granted through this primitive become invalid when the membership status is not active or the current time is outside the allowed interval.
D-4.10 Privacy rule: personal-tenant retention stores only portable evidence and redacts host-confidential, patient-confidential, school-confidential, and trade-secret content.
D-4.11 Fairness rule: ranking, evaluation, placement, conversion, and mentor assignment using this primitive must expose reviewable inputs for ADR-0299 checks.
D-4.12 Test rule: the shared crate must include positive, negative, boundary-date, early-revocation, minor, and jurisdiction-overlay fixtures for this primitive.

### D-5. Labor-law and training-law overlays

D-5.1 This primitive is required for ADR-0320 conformance and is enforced at service boundary, event boundary, and policy boundary.
D-5.2 The primitive derives its tenancy semantics from ADR-0244 and its personal/work separation from ADR-0311.
D-5.3 The primitive must be compatible with minor controls from ADR-0292, fairness controls from ADR-0299, mentor controls from ADR-0305, and hierarchy controls from ADR-0313.
D-5.4 Hyperscaler precedent one: enterprise identity systems separate organization membership from personal identity so that access can be revoked without deleting a person profile.
D-5.5 Hyperscaler precedent two: cloud IAM systems model temporary credentials and time-bounded sessions as normal primitives, not exception paths.
D-5.6 Regulatory precedent one: labor and training regimes attach duties to the relationship interval, so the model stores start and end dates as policy inputs.
D-5.7 Regulatory precedent two: education, clinical, and workforce programs rely on supervised authorization, so the model stores supervisor and mentor claims as policy inputs.
D-5.8 Audit rule: every state transition for this primitive emits an audit-chain event with actor, subject, tenants, source evidence, and before/after status.
D-5.9 Revocation rule: active privileges granted through this primitive become invalid when the membership status is not active or the current time is outside the allowed interval.
D-5.10 Privacy rule: personal-tenant retention stores only portable evidence and redacts host-confidential, patient-confidential, school-confidential, and trade-secret content.
D-5.11 Fairness rule: ranking, evaluation, placement, conversion, and mentor assignment using this primitive must expose reviewable inputs for ADR-0299 checks.
D-5.12 Test rule: the shared crate must include positive, negative, boundary-date, early-revocation, minor, and jurisdiction-overlay fixtures for this primitive.

### D-6. Credential hour and competency tracking

D-6.1 This primitive is required for ADR-0320 conformance and is enforced at service boundary, event boundary, and policy boundary.
D-6.2 The primitive derives its tenancy semantics from ADR-0244 and its personal/work separation from ADR-0311.
D-6.3 The primitive must be compatible with minor controls from ADR-0292, fairness controls from ADR-0299, mentor controls from ADR-0305, and hierarchy controls from ADR-0313.
D-6.4 Hyperscaler precedent one: enterprise identity systems separate organization membership from personal identity so that access can be revoked without deleting a person profile.
D-6.5 Hyperscaler precedent two: cloud IAM systems model temporary credentials and time-bounded sessions as normal primitives, not exception paths.
D-6.6 Regulatory precedent one: labor and training regimes attach duties to the relationship interval, so the model stores start and end dates as policy inputs.
D-6.7 Regulatory precedent two: education, clinical, and workforce programs rely on supervised authorization, so the model stores supervisor and mentor claims as policy inputs.
D-6.8 Audit rule: every state transition for this primitive emits an audit-chain event with actor, subject, tenants, source evidence, and before/after status.
D-6.9 Revocation rule: active privileges granted through this primitive become invalid when the membership status is not active or the current time is outside the allowed interval.
D-6.10 Privacy rule: personal-tenant retention stores only portable evidence and redacts host-confidential, patient-confidential, school-confidential, and trade-secret content.
D-6.11 Fairness rule: ranking, evaluation, placement, conversion, and mentor assignment using this primitive must expose reviewable inputs for ADR-0299 checks.
D-6.12 Test rule: the shared crate must include positive, negative, boundary-date, early-revocation, minor, and jurisdiction-overlay fixtures for this primitive.

### D-7. Personal-tenant portfolio survival

D-7.1 This primitive is required for ADR-0320 conformance and is enforced at service boundary, event boundary, and policy boundary.
D-7.2 The primitive derives its tenancy semantics from ADR-0244 and its personal/work separation from ADR-0311.
D-7.3 The primitive must be compatible with minor controls from ADR-0292, fairness controls from ADR-0299, mentor controls from ADR-0305, and hierarchy controls from ADR-0313.
D-7.4 Hyperscaler precedent one: enterprise identity systems separate organization membership from personal identity so that access can be revoked without deleting a person profile.
D-7.5 Hyperscaler precedent two: cloud IAM systems model temporary credentials and time-bounded sessions as normal primitives, not exception paths.
D-7.6 Regulatory precedent one: labor and training regimes attach duties to the relationship interval, so the model stores start and end dates as policy inputs.
D-7.7 Regulatory precedent two: education, clinical, and workforce programs rely on supervised authorization, so the model stores supervisor and mentor claims as policy inputs.
D-7.8 Audit rule: every state transition for this primitive emits an audit-chain event with actor, subject, tenants, source evidence, and before/after status.
D-7.9 Revocation rule: active privileges granted through this primitive become invalid when the membership status is not active or the current time is outside the allowed interval.
D-7.10 Privacy rule: personal-tenant retention stores only portable evidence and redacts host-confidential, patient-confidential, school-confidential, and trade-secret content.
D-7.11 Fairness rule: ranking, evaluation, placement, conversion, and mentor assignment using this primitive must expose reviewable inputs for ADR-0299 checks.
D-7.12 Test rule: the shared crate must include positive, negative, boundary-date, early-revocation, minor, and jurisdiction-overlay fixtures for this primitive.

### D-8. Cross-program continuity

D-8.1 This primitive is required for ADR-0320 conformance and is enforced at service boundary, event boundary, and policy boundary.
D-8.2 The primitive derives its tenancy semantics from ADR-0244 and its personal/work separation from ADR-0311.
D-8.3 The primitive must be compatible with minor controls from ADR-0292, fairness controls from ADR-0299, mentor controls from ADR-0305, and hierarchy controls from ADR-0313.
D-8.4 Hyperscaler precedent one: enterprise identity systems separate organization membership from personal identity so that access can be revoked without deleting a person profile.
D-8.5 Hyperscaler precedent two: cloud IAM systems model temporary credentials and time-bounded sessions as normal primitives, not exception paths.
D-8.6 Regulatory precedent one: labor and training regimes attach duties to the relationship interval, so the model stores start and end dates as policy inputs.
D-8.7 Regulatory precedent two: education, clinical, and workforce programs rely on supervised authorization, so the model stores supervisor and mentor claims as policy inputs.
D-8.8 Audit rule: every state transition for this primitive emits an audit-chain event with actor, subject, tenants, source evidence, and before/after status.
D-8.9 Revocation rule: active privileges granted through this primitive become invalid when the membership status is not active or the current time is outside the allowed interval.
D-8.10 Privacy rule: personal-tenant retention stores only portable evidence and redacts host-confidential, patient-confidential, school-confidential, and trade-secret content.
D-8.11 Fairness rule: ranking, evaluation, placement, conversion, and mentor assignment using this primitive must expose reviewable inputs for ADR-0299 checks.
D-8.12 Test rule: the shared crate must include positive, negative, boundary-date, early-revocation, minor, and jurisdiction-overlay fixtures for this primitive.

### D-9. Mentor, sponsor, preceptor, and attending pairing

D-9.1 This primitive is required for ADR-0320 conformance and is enforced at service boundary, event boundary, and policy boundary.
D-9.2 The primitive derives its tenancy semantics from ADR-0244 and its personal/work separation from ADR-0311.
D-9.3 The primitive must be compatible with minor controls from ADR-0292, fairness controls from ADR-0299, mentor controls from ADR-0305, and hierarchy controls from ADR-0313.
D-9.4 Hyperscaler precedent one: enterprise identity systems separate organization membership from personal identity so that access can be revoked without deleting a person profile.
D-9.5 Hyperscaler precedent two: cloud IAM systems model temporary credentials and time-bounded sessions as normal primitives, not exception paths.
D-9.6 Regulatory precedent one: labor and training regimes attach duties to the relationship interval, so the model stores start and end dates as policy inputs.
D-9.7 Regulatory precedent two: education, clinical, and workforce programs rely on supervised authorization, so the model stores supervisor and mentor claims as policy inputs.
D-9.8 Audit rule: every state transition for this primitive emits an audit-chain event with actor, subject, tenants, source evidence, and before/after status.
D-9.9 Revocation rule: active privileges granted through this primitive become invalid when the membership status is not active or the current time is outside the allowed interval.
D-9.10 Privacy rule: personal-tenant retention stores only portable evidence and redacts host-confidential, patient-confidential, school-confidential, and trade-secret content.
D-9.11 Fairness rule: ranking, evaluation, placement, conversion, and mentor assignment using this primitive must expose reviewable inputs for ADR-0299 checks.
D-9.12 Test rule: the shared crate must include positive, negative, boundary-date, early-revocation, minor, and jurisdiction-overlay fixtures for this primitive.

### D-10. Program-end handoff and revocation

D-10.1 This primitive is required for ADR-0320 conformance and is enforced at service boundary, event boundary, and policy boundary.
D-10.2 The primitive derives its tenancy semantics from ADR-0244 and its personal/work separation from ADR-0311.
D-10.3 The primitive must be compatible with minor controls from ADR-0292, fairness controls from ADR-0299, mentor controls from ADR-0305, and hierarchy controls from ADR-0313.
D-10.4 Hyperscaler precedent one: enterprise identity systems separate organization membership from personal identity so that access can be revoked without deleting a person profile.
D-10.5 Hyperscaler precedent two: cloud IAM systems model temporary credentials and time-bounded sessions as normal primitives, not exception paths.
D-10.6 Regulatory precedent one: labor and training regimes attach duties to the relationship interval, so the model stores start and end dates as policy inputs.
D-10.7 Regulatory precedent two: education, clinical, and workforce programs rely on supervised authorization, so the model stores supervisor and mentor claims as policy inputs.
D-10.8 Audit rule: every state transition for this primitive emits an audit-chain event with actor, subject, tenants, source evidence, and before/after status.
D-10.9 Revocation rule: active privileges granted through this primitive become invalid when the membership status is not active or the current time is outside the allowed interval.
D-10.10 Privacy rule: personal-tenant retention stores only portable evidence and redacts host-confidential, patient-confidential, school-confidential, and trade-secret content.
D-10.11 Fairness rule: ranking, evaluation, placement, conversion, and mentor assignment using this primitive must expose reviewable inputs for ADR-0299 checks.
D-10.12 Test rule: the shared crate must include positive, negative, boundary-date, early-revocation, minor, and jurisdiction-overlay fixtures for this primitive.

#### D-1.13 Canonical DDL

```sql
CREATE TYPE program_type AS ENUM (
  'APPRENTICE',
  'INTERN',
  'RESIDENT',
  'FELLOW',
  'COOP',
  'EXTERN'
);

CREATE TYPE program_membership_status AS ENUM (
  'DRAFT',
  'PENDING_SOURCE_APPROVAL',
  'PENDING_HOST_APPROVAL',
  'ACTIVE',
  'SUSPENDED',
  'COMPLETED',
  'TERMINATED',
  'REVOKED'
);

CREATE TABLE program_tenant_membership (
  membership_id uuid PRIMARY KEY,
  person_id uuid NOT NULL,
  personal_tenant_id uuid NOT NULL,
  source_tenant_id uuid,
  host_tenant_id uuid NOT NULL,
  program_tenant_id uuid NOT NULL,
  program_type program_type NOT NULL,
  status program_membership_status NOT NULL,
  capability_tier text NOT NULL,
  jurisdiction_code text NOT NULL,
  program_start_at timestamptz NOT NULL,
  program_end_at timestamptz NOT NULL,
  early_revoke_at timestamptz,
  source_approval_event_id uuid,
  host_approval_event_id uuid,
  mentor_assignment_id uuid,
  labor_overlay_id uuid NOT NULL,
  minor_profile_id uuid,
  portfolio_retention_profile_id uuid NOT NULL,
  created_at timestamptz NOT NULL,
  updated_at timestamptz NOT NULL,
  CHECK (program_start_at < program_end_at),
  CHECK (capability_tier IN ('observer','contributor','supervised_operator','compensated_worker','clinical_trainee','research_trainee','portfolio_only'))
);
```

#### D-2.13 Cedar policy sketch

```cedar
permit(
  principal in Oyatie::ProgramParticipant::"{membership_id}",
  action in Oyatie::ProgramAction::"submit_supervised_artifact",
  resource in Oyatie::ProgramTenant::"{program_tenant_id}"
)
when {
  context.program.status == "ACTIVE" &&
  context.now >= context.program.program_start_at &&
  context.now < context.program.program_end_at &&
  context.supervision.assigned == true &&
  context.labor_overlay.allows_action == true &&
  context.revocation.active == false
};
```

#### D-5.13 Jurisdiction overlay requirements

D-5.13.1 Overlay `US-FLSA-INTERN` applies to United States.
D-5.13.1.a Summary: Unpaid intern classification requires a primary-beneficiary analysis before the host can suppress wage obligations.
D-5.13.1.ref1: 29 U.S.C. § 203(e).
D-5.13.1.ref2: 29 U.S.C. § 206.
D-5.13.1.ref3: 29 U.S.C. § 207.
D-5.13.1.ref4: 29 C.F.R. § 785.27.
D-5.13.1.ref5: DOL Fact Sheet #71 primary beneficiary test.
D-5.13.1.enforcement: the overlay can require wage eligibility, block unpaid classification, require written terms, require hours review, block schedule publication, block capability grants, or require compliance review before activation.
D-5.13.1.audit: every overlay evaluation stores the exact reference set, evaluated facts, decision, actor, and timestamp.
D-5.13.2 Overlay `US-NLRA-STUDENT-WORKER` applies to United States.
D-5.13.2.a Summary: Covered student-worker and trainee relationships must not suppress protected concerted activity.
D-5.13.2.ref1: 29 U.S.C. § 157.
D-5.13.2.ref2: 29 U.S.C. § 158(a)(1).
D-5.13.2.enforcement: the overlay can require wage eligibility, block unpaid classification, require written terms, require hours review, block schedule publication, block capability grants, or require compliance review before activation.
D-5.13.2.audit: every overlay evaluation stores the exact reference set, evaluated facts, decision, actor, and timestamp.
D-5.13.3 Overlay `EU-TRANSPARENT-PREDICTABLE-WORK` applies to European Union.
D-5.13.3.a Summary: Written information, timing, predictability, probation, and adverse-treatment protections become program facts.
D-5.13.3.ref1: Directive (EU) 2019/1152 Article 1.
D-5.13.3.ref2: Directive (EU) 2019/1152 Article 3.
D-5.13.3.ref3: Directive (EU) 2019/1152 Article 4.
D-5.13.3.ref4: Directive (EU) 2019/1152 Article 5.
D-5.13.3.ref5: Directive (EU) 2019/1152 Article 8.
D-5.13.3.ref6: Directive (EU) 2019/1152 Article 17.
D-5.13.3.enforcement: the overlay can require wage eligibility, block unpaid classification, require written terms, require hours review, block schedule publication, block capability grants, or require compliance review before activation.
D-5.13.3.audit: every overlay evaluation stores the exact reference set, evaluated facts, decision, actor, and timestamp.
D-5.13.4 Overlay `EU-QUALITY-FRAMEWORK-TRAINEESHIPS` applies to European Union.
D-5.13.4.a Summary: Traineeship quality requirements become host-program obligations.
D-5.13.4.ref1: Council Recommendation 2014/C 88/01 principles on written traineeship agreement.
D-5.13.4.ref2: Council Recommendation 2014/C 88/01 principles on learning and training objectives.
D-5.13.4.ref3: Council Recommendation 2014/C 88/01 principles on working conditions.
D-5.13.4.ref4: Council Recommendation 2014/C 88/01 principles on reasonable duration.
D-5.13.4.ref5: Council Recommendation 2014/C 88/01 principles on recognition.
D-5.13.4.enforcement: the overlay can require wage eligibility, block unpaid classification, require written terms, require hours review, block schedule publication, block capability grants, or require compliance review before activation.
D-5.13.4.audit: every overlay evaluation stores the exact reference set, evaluated facts, decision, actor, and timestamp.
D-5.13.5 Overlay `KR-APPRENTICE-WORKER-BASELINE` applies to Republic of Korea.
D-5.13.5.a Summary: Written terms, working-hour limits, extended work, recess, weekly holiday, paid annual leave, and minimum-wage floors become enforcement predicates.
D-5.13.5.ref1: Labor Standards Act Article 17.
D-5.13.5.ref2: Labor Standards Act Article 50.
D-5.13.5.ref3: Labor Standards Act Article 53.
D-5.13.5.ref4: Labor Standards Act Article 54.
D-5.13.5.ref5: Labor Standards Act Article 55.
D-5.13.5.ref6: Labor Standards Act Article 60.
D-5.13.5.ref7: Minimum Wage Act Article 6.
D-5.13.5.enforcement: the overlay can require wage eligibility, block unpaid classification, require written terms, require hours review, block schedule publication, block capability grants, or require compliance review before activation.
D-5.13.5.audit: every overlay evaluation stores the exact reference set, evaluated facts, decision, actor, and timestamp.
D-5.13.6 Overlay `KR-VOCATIONAL-TRAINING` applies to Republic of Korea.
D-5.13.6.a Summary: On-site vocational training facts must be separated from ordinary employment claims while preserving safety and documentation duties.
D-5.13.6.ref1: Vocational Education and Training Promotion Act Article 7.
D-5.13.6.ref2: Vocational Education and Training Promotion Act Article 9.
D-5.13.6.ref3: Vocational Education and Training Promotion Act Article 24.
D-5.13.6.ref4: Vocational Education and Training Promotion Act Article 25.
D-5.13.6.enforcement: the overlay can require wage eligibility, block unpaid classification, require written terms, require hours review, block schedule publication, block capability grants, or require compliance review before activation.
D-5.13.6.audit: every overlay evaluation stores the exact reference set, evaluated facts, decision, actor, and timestamp.
D-5.13.7 Overlay `US-ACGME-RESIDENT-DUTY-HOUR` applies to United States.
D-5.13.7.a Summary: Residents require duty-hour counting, moonlighting inclusion, and rest-period evidence as authorization facts.
D-5.13.7.ref1: ACGME Common Program Requirements 6.20.
D-5.13.7.ref2: ACGME Common Program Requirements 6.21.b.
D-5.13.7.ref3: ACGME Common Program Requirements 6.25.a.
D-5.13.7.ref4: ACGME Common Program Requirements 6.28.
D-5.13.7.enforcement: the overlay can require wage eligibility, block unpaid classification, require written terms, require hours review, block schedule publication, block capability grants, or require compliance review before activation.
D-5.13.7.audit: every overlay evaluation stores the exact reference set, evaluated facts, decision, actor, and timestamp.

## E. Implementation footprint

### E.1 Shared crate

E.1.1 Create `crates/oya-shared-program-transient-identity` as the canonical domain package for this doctrine.
E.1.2 The crate owns `ProgramType`, `ProgramMembershipStatus`, `ProgramCapabilityTier`, `ProgramTenantMembership`, `ProgramJurisdictionOverlay`, `ProgramPortfolioRetentionProfile`, and `ProgramAuditEvent`.
E.1.3 The crate exposes validation functions for date windows, enum parsing, status transitions, minor gates, labor overlay gates, capability tier compatibility, and portfolio survival predicates.
E.1.4 The crate exposes Cedar context builders so consumer services cannot construct partial or inconsistent authorization facts.
E.1.5 The crate exposes event names as constants and typed payload structs so audit-chain receives stable names.
E.1.6 The crate MUST NOT call databases, HTTP clients, queues, or policy engines directly; it is a clean domain package used by adapters.
E.1.7 The crate includes tests for every program type, capability tier, jurisdiction overlay, revocation path, and cross-ADR compatibility path.

### E.2 `community` consumer

E.2.1 `community` consumes `oya-shared-program-transient-identity` for program cohorts, mentor channels, peer groups, public/private work sample visibility, and safe messaging boundaries.
E.2.2 `community` must reject unknown `program_type` values rather than silently downgrade them to generic member roles.
E.2.3 `community` must record the `membership_id`, `program_tenant_id`, `host_tenant_id`, and `personal_tenant_id` on every program-scoped event it emits.
E.2.4 `community` must treat program-end revocation as a hard authorization boundary and may only keep personal portfolio artifacts through the shared retention predicate.
E.2.5 `community` must expose test fixtures covering `APPRENTICE`, `INTERN`, `RESIDENT`, and `FELLOW` at minimum, because those four labels carry the highest labor, clinical, and education risk.
E.2.6 `community` must include ADR-0320 in its service-contract documentation once it consumes the crate.

### E.3 `identity` consumer

E.3.1 `identity` consumes `oya-shared-program-transient-identity` for person binding, tenant membership, source-host attestation, personal/work separation, and age/minor facts.
E.3.2 `identity` must reject unknown `program_type` values rather than silently downgrade them to generic member roles.
E.3.3 `identity` must record the `membership_id`, `program_tenant_id`, `host_tenant_id`, and `personal_tenant_id` on every program-scoped event it emits.
E.3.4 `identity` must treat program-end revocation as a hard authorization boundary and may only keep personal portfolio artifacts through the shared retention predicate.
E.3.5 `identity` must expose test fixtures covering `APPRENTICE`, `INTERN`, `RESIDENT`, and `FELLOW` at minimum, because those four labels carry the highest labor, clinical, and education risk.
E.3.6 `identity` must include ADR-0320 in its service-contract documentation once it consumes the crate.

### E.4 `workplace-integration` consumer

E.4.1 `workplace-integration` consumes `oya-shared-program-transient-identity` for host roster import, HRIS mapping, schedule import, supervisor mapping, work assignment, and conversion-to-employee handoff.
E.4.2 `workplace-integration` must reject unknown `program_type` values rather than silently downgrade them to generic member roles.
E.4.3 `workplace-integration` must record the `membership_id`, `program_tenant_id`, `host_tenant_id`, and `personal_tenant_id` on every program-scoped event it emits.
E.4.4 `workplace-integration` must treat program-end revocation as a hard authorization boundary and may only keep personal portfolio artifacts through the shared retention predicate.
E.4.5 `workplace-integration` must expose test fixtures covering `APPRENTICE`, `INTERN`, `RESIDENT`, and `FELLOW` at minimum, because those four labels carry the highest labor, clinical, and education risk.
E.4.6 `workplace-integration` must include ADR-0320 in its service-contract documentation once it consumes the crate.

### E.5 `payments` consumer

E.5.1 `payments` consumes `oya-shared-program-transient-identity` for stipend, wage, reimbursement, scholarship, payroll eligibility, tax classification hinting, and unpaid classification block decisions.
E.5.2 `payments` must reject unknown `program_type` values rather than silently downgrade them to generic member roles.
E.5.3 `payments` must record the `membership_id`, `program_tenant_id`, `host_tenant_id`, and `personal_tenant_id` on every program-scoped event it emits.
E.5.4 `payments` must treat program-end revocation as a hard authorization boundary and may only keep personal portfolio artifacts through the shared retention predicate.
E.5.5 `payments` must expose test fixtures covering `APPRENTICE`, `INTERN`, `RESIDENT`, and `FELLOW` at minimum, because those four labels carry the highest labor, clinical, and education risk.
E.5.6 `payments` must include ADR-0320 in its service-contract documentation once it consumes the crate.

### E.6 `audit-chain` consumer

E.6.1 `audit-chain` consumes `oya-shared-program-transient-identity` for append-only events, overlay decisions, evidence hash linking, portfolio export proofs, and revocation evidence.
E.6.2 `audit-chain` must reject unknown `program_type` values rather than silently downgrade them to generic member roles.
E.6.3 `audit-chain` must record the `membership_id`, `program_tenant_id`, `host_tenant_id`, and `personal_tenant_id` on every program-scoped event it emits.
E.6.4 `audit-chain` must treat program-end revocation as a hard authorization boundary and may only keep personal portfolio artifacts through the shared retention predicate.
E.6.5 `audit-chain` must expose test fixtures covering `APPRENTICE`, `INTERN`, `RESIDENT`, and `FELLOW` at minimum, because those four labels carry the highest labor, clinical, and education risk.
E.6.6 `audit-chain` must include ADR-0320 in its service-contract documentation once it consumes the crate.

## F. Migration

F.1 Inventory existing service fields that use role names such as intern, apprentice, resident, fellow, student, trainee, cohort member, junior associate, extern, and co-op.

F.2 Map each existing field to `program_type`, `capability_tier`, `program_tenant_id`, and `program_membership_status`.

F.3 Create program tenants for active source-host programs and bind them to source, host, and parent tenants according to ADR-0244 and ADR-0313.

F.4 Backfill personal-tenant portfolio retention records only after consent, confidentiality, and evidence predicates pass.

F.5 Backfill labor overlays by jurisdiction and program type. Unknown overlay mappings default to blocked activation, not permissive activation.

F.6 Convert service-local authorization checks to Cedar context produced by the shared crate.

F.7 Emit synthetic audit-chain migration events for each backfilled membership, overlay, portfolio retention profile, and revocation schedule.

F.8 Re-run fairness checks from ADR-0299 on placement, evaluation, pay, and conversion data after migration.

F.9 Re-run minor checks from ADR-0292 on every participant below the relevant age threshold or with missing date-of-birth evidence.

F.10 Roll out in read-only mirror mode, then policy-shadow mode, then enforcement mode per service.

## G. References

G.1 ADR-0244: Tenant as Universal Scoping Primitive. Source: docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md
G.2 ADR-0292: Minor User Doctrine for COPPA, KOSA, EU Age Verification, and Guardian Boundaries. Source: docs/decisions/ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
G.3 ADR-0299: Fairness and Explainability Controls. Source: docs/decisions/ADR-0299-generational-abuse-detection-and-family-privacy-boundary.md
G.4 ADR-0305: Mentor Relationship Doctrine. Source: docs/decisions/ADR-0305-non-familial-mentor-guardian-trust-boundaries.md
G.5 ADR-0311: Dual Tenant Identity Personal vs Work Boundary. Source: docs/decisions/ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md
G.6 ADR-0313: Conglomerate Tenant Hierarchy with Sovereign Children. Source: docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md
G.7 US FLSA: 29 U.S.C. § 203(e), § 206, and § 207. Source: https://www.law.cornell.edu/uscode/text/29/chapter-8
G.8 US FLSA trainee rule: 29 C.F.R. § 785.27. Source: https://www.ecfr.gov/current/title-29/subtitle-B/chapter-V/subchapter-B/part-785/subpart-C/section-785.27
G.9 DOL Fact Sheet #71: Internship Programs Under the Fair Labor Standards Act. Source: https://www.dol.gov/agencies/whd/fact-sheets/71-flsa-internships
G.10 NLRA: 29 U.S.C. § 157 and § 158(a)(1). Source: https://www.nlrb.gov/guidance/key-reference-materials/national-labor-relations-act
G.11 EU Directive 2019/1152: Transparent and Predictable Working Conditions Articles 1, 3, 4, 5, 8, and 17. Source: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX%3A32019L1152
G.12 EU Quality Framework for Traineeships: Council Recommendation 2014/C 88/01. Source: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX%3A32014H0327%2801%29
G.13 Korean Labor Standards Act: Articles 17, 50, 53, 54, 55, and 60. Source: https://elaw.klri.re.kr/eng_service/lawTwoView.do?hseq=19119
G.14 Korean Minimum Wage Act: Article 6. Source: https://elaw.klri.re.kr/eng_mobile/viewer.do?hseq=70535&key=6&type=sogan
G.15 Korean Vocational Education and Training Promotion Act: Articles 7, 9, 24, and 25. Source: https://elaw.klri.re.kr/eng_service/lawViewTitle.do?hseq=51980
G.16 ACGME Common Program Requirements: Requirements 6.20, 6.21.b, 6.25.a, and 6.28 for clinical and educational work-hour controls. Source: https://www.acgme.org/globalassets/pfassets/programrequirements/2026-prs/cprresidency_2026_feb_revision.pdf
G.17 Cedar policy syntax: permit, forbid, when clauses, context, and schema concepts. Source: https://docs.cedarpolicy.com/policies/syntax-policy.html
G.18 LinkedIn Talent Solutions early talent: student talent pool and recruiting context. Source: https://business.linkedin.com/talent-solutions/c/15/6/student-talent-pool/studentrecruitment
G.19 Handshake: student, university, and employer network pattern. Source: https://joinhandshake.com/
G.20 Workday early career: internship and early-career program context. Source: https://www.workday.com/en-us/company/careers/early-career.html

## H. Change log and naming justifications

| Date | Change | Reason |
| --- | --- | --- |
| 2026-05-20 | Created ADR-0320 | Establishes a multi-tenant transient identity doctrine for apprentices, interns, residents, fellows, co-ops, and externs. |

### H.1 Naming justifications

H.1.1 `program_type`: plain domain term that covers education, training, clinical, and workforce programs without implying employment status.
H.1.2 `program_tenant_membership`: names the relationship as a tenant-scoped membership instead of a role string.
H.1.3 `oya-shared-program-transient-identity`: states the crate owns shared domain logic for program identities that expire or convert.
H.1.4 `APPRENTICE`: covers vocational and workforce training patterns with strong supervision and credential obligations.
H.1.5 `INTERN`: covers short-term educational work placements and FLSA primary-beneficiary risk.
H.1.6 `RESIDENT`: covers graduate clinical training with ACGME duty-hour and supervision obligations.
H.1.7 `FELLOW`: covers advanced clinical, research, policy, or professional training beyond entry-level placement.
H.1.8 `COOP`: covers cooperative education rotations that alternate or combine school and host work.
H.1.9 `EXTERN`: covers shorter observation or shadowing placements with reduced production authority.

## I. Primitive compliance matrix

I.1.0 Primitive `program_type` is part of the ADR-0320 conformance surface.
I.1.1 Source of truth: `program_type` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.1.2 Tenant rule: `program_type` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.1.3 ADR-0244 rule: `program_type` cannot introduce a new tenancy primitive outside the universal tenant model.
I.1.4 ADR-0311 rule: `program_type` cannot merge personal and work identities into one host-owned record.
I.1.5 ADR-0313 rule: `program_type` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.1.6 ADR-0292 rule: `program_type` must block unsafe minor flows when age, guardian, or school facts are missing.
I.1.7 ADR-0299 rule: `program_type` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.1.8 ADR-0305 rule: `program_type` must expose mentor or supervisor context when capability depends on supervision.
I.1.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.1.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.1.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.1.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.1.13 Failure mode: if `program_type` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.1.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `program_type`.
I.1.15 Audit fixture: include creation, mutation, denial, and revocation events for `program_type`.

I.2.0 Primitive `program_membership_status` is part of the ADR-0320 conformance surface.
I.2.1 Source of truth: `program_membership_status` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.2.2 Tenant rule: `program_membership_status` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.2.3 ADR-0244 rule: `program_membership_status` cannot introduce a new tenancy primitive outside the universal tenant model.
I.2.4 ADR-0311 rule: `program_membership_status` cannot merge personal and work identities into one host-owned record.
I.2.5 ADR-0313 rule: `program_membership_status` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.2.6 ADR-0292 rule: `program_membership_status` must block unsafe minor flows when age, guardian, or school facts are missing.
I.2.7 ADR-0299 rule: `program_membership_status` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.2.8 ADR-0305 rule: `program_membership_status` must expose mentor or supervisor context when capability depends on supervision.
I.2.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.2.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.2.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.2.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.2.13 Failure mode: if `program_membership_status` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.2.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `program_membership_status`.
I.2.15 Audit fixture: include creation, mutation, denial, and revocation events for `program_membership_status`.

I.3.0 Primitive `program_tenant_id` is part of the ADR-0320 conformance surface.
I.3.1 Source of truth: `program_tenant_id` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.3.2 Tenant rule: `program_tenant_id` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.3.3 ADR-0244 rule: `program_tenant_id` cannot introduce a new tenancy primitive outside the universal tenant model.
I.3.4 ADR-0311 rule: `program_tenant_id` cannot merge personal and work identities into one host-owned record.
I.3.5 ADR-0313 rule: `program_tenant_id` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.3.6 ADR-0292 rule: `program_tenant_id` must block unsafe minor flows when age, guardian, or school facts are missing.
I.3.7 ADR-0299 rule: `program_tenant_id` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.3.8 ADR-0305 rule: `program_tenant_id` must expose mentor or supervisor context when capability depends on supervision.
I.3.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.3.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.3.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.3.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.3.13 Failure mode: if `program_tenant_id` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.3.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `program_tenant_id`.
I.3.15 Audit fixture: include creation, mutation, denial, and revocation events for `program_tenant_id`.

I.4.0 Primitive `personal_tenant_id` is part of the ADR-0320 conformance surface.
I.4.1 Source of truth: `personal_tenant_id` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.4.2 Tenant rule: `personal_tenant_id` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.4.3 ADR-0244 rule: `personal_tenant_id` cannot introduce a new tenancy primitive outside the universal tenant model.
I.4.4 ADR-0311 rule: `personal_tenant_id` cannot merge personal and work identities into one host-owned record.
I.4.5 ADR-0313 rule: `personal_tenant_id` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.4.6 ADR-0292 rule: `personal_tenant_id` must block unsafe minor flows when age, guardian, or school facts are missing.
I.4.7 ADR-0299 rule: `personal_tenant_id` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.4.8 ADR-0305 rule: `personal_tenant_id` must expose mentor or supervisor context when capability depends on supervision.
I.4.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.4.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.4.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.4.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.4.13 Failure mode: if `personal_tenant_id` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.4.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `personal_tenant_id`.
I.4.15 Audit fixture: include creation, mutation, denial, and revocation events for `personal_tenant_id`.

I.5.0 Primitive `source_tenant_id` is part of the ADR-0320 conformance surface.
I.5.1 Source of truth: `source_tenant_id` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.5.2 Tenant rule: `source_tenant_id` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.5.3 ADR-0244 rule: `source_tenant_id` cannot introduce a new tenancy primitive outside the universal tenant model.
I.5.4 ADR-0311 rule: `source_tenant_id` cannot merge personal and work identities into one host-owned record.
I.5.5 ADR-0313 rule: `source_tenant_id` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.5.6 ADR-0292 rule: `source_tenant_id` must block unsafe minor flows when age, guardian, or school facts are missing.
I.5.7 ADR-0299 rule: `source_tenant_id` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.5.8 ADR-0305 rule: `source_tenant_id` must expose mentor or supervisor context when capability depends on supervision.
I.5.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.5.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.5.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.5.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.5.13 Failure mode: if `source_tenant_id` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.5.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `source_tenant_id`.
I.5.15 Audit fixture: include creation, mutation, denial, and revocation events for `source_tenant_id`.

I.6.0 Primitive `host_tenant_id` is part of the ADR-0320 conformance surface.
I.6.1 Source of truth: `host_tenant_id` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.6.2 Tenant rule: `host_tenant_id` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.6.3 ADR-0244 rule: `host_tenant_id` cannot introduce a new tenancy primitive outside the universal tenant model.
I.6.4 ADR-0311 rule: `host_tenant_id` cannot merge personal and work identities into one host-owned record.
I.6.5 ADR-0313 rule: `host_tenant_id` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.6.6 ADR-0292 rule: `host_tenant_id` must block unsafe minor flows when age, guardian, or school facts are missing.
I.6.7 ADR-0299 rule: `host_tenant_id` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.6.8 ADR-0305 rule: `host_tenant_id` must expose mentor or supervisor context when capability depends on supervision.
I.6.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.6.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.6.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.6.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.6.13 Failure mode: if `host_tenant_id` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.6.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `host_tenant_id`.
I.6.15 Audit fixture: include creation, mutation, denial, and revocation events for `host_tenant_id`.

I.7.0 Primitive `program_start_at` is part of the ADR-0320 conformance surface.
I.7.1 Source of truth: `program_start_at` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.7.2 Tenant rule: `program_start_at` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.7.3 ADR-0244 rule: `program_start_at` cannot introduce a new tenancy primitive outside the universal tenant model.
I.7.4 ADR-0311 rule: `program_start_at` cannot merge personal and work identities into one host-owned record.
I.7.5 ADR-0313 rule: `program_start_at` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.7.6 ADR-0292 rule: `program_start_at` must block unsafe minor flows when age, guardian, or school facts are missing.
I.7.7 ADR-0299 rule: `program_start_at` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.7.8 ADR-0305 rule: `program_start_at` must expose mentor or supervisor context when capability depends on supervision.
I.7.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.7.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.7.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.7.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.7.13 Failure mode: if `program_start_at` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.7.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `program_start_at`.
I.7.15 Audit fixture: include creation, mutation, denial, and revocation events for `program_start_at`.

I.8.0 Primitive `program_end_at` is part of the ADR-0320 conformance surface.
I.8.1 Source of truth: `program_end_at` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.8.2 Tenant rule: `program_end_at` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.8.3 ADR-0244 rule: `program_end_at` cannot introduce a new tenancy primitive outside the universal tenant model.
I.8.4 ADR-0311 rule: `program_end_at` cannot merge personal and work identities into one host-owned record.
I.8.5 ADR-0313 rule: `program_end_at` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.8.6 ADR-0292 rule: `program_end_at` must block unsafe minor flows when age, guardian, or school facts are missing.
I.8.7 ADR-0299 rule: `program_end_at` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.8.8 ADR-0305 rule: `program_end_at` must expose mentor or supervisor context when capability depends on supervision.
I.8.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.8.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.8.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.8.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.8.13 Failure mode: if `program_end_at` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.8.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `program_end_at`.
I.8.15 Audit fixture: include creation, mutation, denial, and revocation events for `program_end_at`.

I.9.0 Primitive `capability_tier` is part of the ADR-0320 conformance surface.
I.9.1 Source of truth: `capability_tier` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.9.2 Tenant rule: `capability_tier` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.9.3 ADR-0244 rule: `capability_tier` cannot introduce a new tenancy primitive outside the universal tenant model.
I.9.4 ADR-0311 rule: `capability_tier` cannot merge personal and work identities into one host-owned record.
I.9.5 ADR-0313 rule: `capability_tier` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.9.6 ADR-0292 rule: `capability_tier` must block unsafe minor flows when age, guardian, or school facts are missing.
I.9.7 ADR-0299 rule: `capability_tier` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.9.8 ADR-0305 rule: `capability_tier` must expose mentor or supervisor context when capability depends on supervision.
I.9.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.9.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.9.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.9.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.9.13 Failure mode: if `capability_tier` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.9.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `capability_tier`.
I.9.15 Audit fixture: include creation, mutation, denial, and revocation events for `capability_tier`.

I.10.0 Primitive `labor_overlay_id` is part of the ADR-0320 conformance surface.
I.10.1 Source of truth: `labor_overlay_id` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.10.2 Tenant rule: `labor_overlay_id` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.10.3 ADR-0244 rule: `labor_overlay_id` cannot introduce a new tenancy primitive outside the universal tenant model.
I.10.4 ADR-0311 rule: `labor_overlay_id` cannot merge personal and work identities into one host-owned record.
I.10.5 ADR-0313 rule: `labor_overlay_id` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.10.6 ADR-0292 rule: `labor_overlay_id` must block unsafe minor flows when age, guardian, or school facts are missing.
I.10.7 ADR-0299 rule: `labor_overlay_id` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.10.8 ADR-0305 rule: `labor_overlay_id` must expose mentor or supervisor context when capability depends on supervision.
I.10.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.10.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.10.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.10.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.10.13 Failure mode: if `labor_overlay_id` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.10.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `labor_overlay_id`.
I.10.15 Audit fixture: include creation, mutation, denial, and revocation events for `labor_overlay_id`.

I.11.0 Primitive `minor_profile_id` is part of the ADR-0320 conformance surface.
I.11.1 Source of truth: `minor_profile_id` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.11.2 Tenant rule: `minor_profile_id` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.11.3 ADR-0244 rule: `minor_profile_id` cannot introduce a new tenancy primitive outside the universal tenant model.
I.11.4 ADR-0311 rule: `minor_profile_id` cannot merge personal and work identities into one host-owned record.
I.11.5 ADR-0313 rule: `minor_profile_id` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.11.6 ADR-0292 rule: `minor_profile_id` must block unsafe minor flows when age, guardian, or school facts are missing.
I.11.7 ADR-0299 rule: `minor_profile_id` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.11.8 ADR-0305 rule: `minor_profile_id` must expose mentor or supervisor context when capability depends on supervision.
I.11.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.11.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.11.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.11.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.11.13 Failure mode: if `minor_profile_id` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.11.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `minor_profile_id`.
I.11.15 Audit fixture: include creation, mutation, denial, and revocation events for `minor_profile_id`.

I.12.0 Primitive `mentor_assignment_id` is part of the ADR-0320 conformance surface.
I.12.1 Source of truth: `mentor_assignment_id` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.12.2 Tenant rule: `mentor_assignment_id` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.12.3 ADR-0244 rule: `mentor_assignment_id` cannot introduce a new tenancy primitive outside the universal tenant model.
I.12.4 ADR-0311 rule: `mentor_assignment_id` cannot merge personal and work identities into one host-owned record.
I.12.5 ADR-0313 rule: `mentor_assignment_id` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.12.6 ADR-0292 rule: `mentor_assignment_id` must block unsafe minor flows when age, guardian, or school facts are missing.
I.12.7 ADR-0299 rule: `mentor_assignment_id` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.12.8 ADR-0305 rule: `mentor_assignment_id` must expose mentor or supervisor context when capability depends on supervision.
I.12.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.12.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.12.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.12.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.12.13 Failure mode: if `mentor_assignment_id` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.12.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `mentor_assignment_id`.
I.12.15 Audit fixture: include creation, mutation, denial, and revocation events for `mentor_assignment_id`.

I.13.0 Primitive `portfolio_retention_profile_id` is part of the ADR-0320 conformance surface.
I.13.1 Source of truth: `portfolio_retention_profile_id` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.13.2 Tenant rule: `portfolio_retention_profile_id` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.13.3 ADR-0244 rule: `portfolio_retention_profile_id` cannot introduce a new tenancy primitive outside the universal tenant model.
I.13.4 ADR-0311 rule: `portfolio_retention_profile_id` cannot merge personal and work identities into one host-owned record.
I.13.5 ADR-0313 rule: `portfolio_retention_profile_id` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.13.6 ADR-0292 rule: `portfolio_retention_profile_id` must block unsafe minor flows when age, guardian, or school facts are missing.
I.13.7 ADR-0299 rule: `portfolio_retention_profile_id` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.13.8 ADR-0305 rule: `portfolio_retention_profile_id` must expose mentor or supervisor context when capability depends on supervision.
I.13.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.13.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.13.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.13.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.13.13 Failure mode: if `portfolio_retention_profile_id` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.13.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `portfolio_retention_profile_id`.
I.13.15 Audit fixture: include creation, mutation, denial, and revocation events for `portfolio_retention_profile_id`.

I.14.0 Primitive `hours_record` is part of the ADR-0320 conformance surface.
I.14.1 Source of truth: `hours_record` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.14.2 Tenant rule: `hours_record` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.14.3 ADR-0244 rule: `hours_record` cannot introduce a new tenancy primitive outside the universal tenant model.
I.14.4 ADR-0311 rule: `hours_record` cannot merge personal and work identities into one host-owned record.
I.14.5 ADR-0313 rule: `hours_record` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.14.6 ADR-0292 rule: `hours_record` must block unsafe minor flows when age, guardian, or school facts are missing.
I.14.7 ADR-0299 rule: `hours_record` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.14.8 ADR-0305 rule: `hours_record` must expose mentor or supervisor context when capability depends on supervision.
I.14.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.14.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.14.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.14.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.14.13 Failure mode: if `hours_record` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.14.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `hours_record`.
I.14.15 Audit fixture: include creation, mutation, denial, and revocation events for `hours_record`.

I.15.0 Primitive `competency_claim` is part of the ADR-0320 conformance surface.
I.15.1 Source of truth: `competency_claim` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.15.2 Tenant rule: `competency_claim` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.15.3 ADR-0244 rule: `competency_claim` cannot introduce a new tenancy primitive outside the universal tenant model.
I.15.4 ADR-0311 rule: `competency_claim` cannot merge personal and work identities into one host-owned record.
I.15.5 ADR-0313 rule: `competency_claim` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.15.6 ADR-0292 rule: `competency_claim` must block unsafe minor flows when age, guardian, or school facts are missing.
I.15.7 ADR-0299 rule: `competency_claim` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.15.8 ADR-0305 rule: `competency_claim` must expose mentor or supervisor context when capability depends on supervision.
I.15.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.15.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.15.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.15.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.15.13 Failure mode: if `competency_claim` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.15.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `competency_claim`.
I.15.15 Audit fixture: include creation, mutation, denial, and revocation events for `competency_claim`.

I.16.0 Primitive `cedar_context` is part of the ADR-0320 conformance surface.
I.16.1 Source of truth: `cedar_context` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.16.2 Tenant rule: `cedar_context` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.16.3 ADR-0244 rule: `cedar_context` cannot introduce a new tenancy primitive outside the universal tenant model.
I.16.4 ADR-0311 rule: `cedar_context` cannot merge personal and work identities into one host-owned record.
I.16.5 ADR-0313 rule: `cedar_context` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.16.6 ADR-0292 rule: `cedar_context` must block unsafe minor flows when age, guardian, or school facts are missing.
I.16.7 ADR-0299 rule: `cedar_context` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.16.8 ADR-0305 rule: `cedar_context` must expose mentor or supervisor context when capability depends on supervision.
I.16.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.16.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.16.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.16.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.16.13 Failure mode: if `cedar_context` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.16.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `cedar_context`.
I.16.15 Audit fixture: include creation, mutation, denial, and revocation events for `cedar_context`.

I.17.0 Primitive `revocation_event` is part of the ADR-0320 conformance surface.
I.17.1 Source of truth: `revocation_event` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.17.2 Tenant rule: `revocation_event` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.17.3 ADR-0244 rule: `revocation_event` cannot introduce a new tenancy primitive outside the universal tenant model.
I.17.4 ADR-0311 rule: `revocation_event` cannot merge personal and work identities into one host-owned record.
I.17.5 ADR-0313 rule: `revocation_event` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.17.6 ADR-0292 rule: `revocation_event` must block unsafe minor flows when age, guardian, or school facts are missing.
I.17.7 ADR-0299 rule: `revocation_event` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.17.8 ADR-0305 rule: `revocation_event` must expose mentor or supervisor context when capability depends on supervision.
I.17.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.17.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.17.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.17.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.17.13 Failure mode: if `revocation_event` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.17.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `revocation_event`.
I.17.15 Audit fixture: include creation, mutation, denial, and revocation events for `revocation_event`.

I.18.0 Primitive `conversion_event` is part of the ADR-0320 conformance surface.
I.18.1 Source of truth: `conversion_event` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.18.2 Tenant rule: `conversion_event` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.18.3 ADR-0244 rule: `conversion_event` cannot introduce a new tenancy primitive outside the universal tenant model.
I.18.4 ADR-0311 rule: `conversion_event` cannot merge personal and work identities into one host-owned record.
I.18.5 ADR-0313 rule: `conversion_event` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.18.6 ADR-0292 rule: `conversion_event` must block unsafe minor flows when age, guardian, or school facts are missing.
I.18.7 ADR-0299 rule: `conversion_event` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.18.8 ADR-0305 rule: `conversion_event` must expose mentor or supervisor context when capability depends on supervision.
I.18.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.18.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.18.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.18.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.18.13 Failure mode: if `conversion_event` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.18.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `conversion_event`.
I.18.15 Audit fixture: include creation, mutation, denial, and revocation events for `conversion_event`.

I.19.0 Primitive `source_attestation` is part of the ADR-0320 conformance surface.
I.19.1 Source of truth: `source_attestation` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.19.2 Tenant rule: `source_attestation` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.19.3 ADR-0244 rule: `source_attestation` cannot introduce a new tenancy primitive outside the universal tenant model.
I.19.4 ADR-0311 rule: `source_attestation` cannot merge personal and work identities into one host-owned record.
I.19.5 ADR-0313 rule: `source_attestation` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.19.6 ADR-0292 rule: `source_attestation` must block unsafe minor flows when age, guardian, or school facts are missing.
I.19.7 ADR-0299 rule: `source_attestation` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.19.8 ADR-0305 rule: `source_attestation` must expose mentor or supervisor context when capability depends on supervision.
I.19.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.19.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.19.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.19.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.19.13 Failure mode: if `source_attestation` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.19.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `source_attestation`.
I.19.15 Audit fixture: include creation, mutation, denial, and revocation events for `source_attestation`.

I.20.0 Primitive `host_attestation` is part of the ADR-0320 conformance surface.
I.20.1 Source of truth: `host_attestation` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.20.2 Tenant rule: `host_attestation` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.20.3 ADR-0244 rule: `host_attestation` cannot introduce a new tenancy primitive outside the universal tenant model.
I.20.4 ADR-0311 rule: `host_attestation` cannot merge personal and work identities into one host-owned record.
I.20.5 ADR-0313 rule: `host_attestation` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.20.6 ADR-0292 rule: `host_attestation` must block unsafe minor flows when age, guardian, or school facts are missing.
I.20.7 ADR-0299 rule: `host_attestation` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.20.8 ADR-0305 rule: `host_attestation` must expose mentor or supervisor context when capability depends on supervision.
I.20.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.20.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.20.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.20.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.20.13 Failure mode: if `host_attestation` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.20.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `host_attestation`.
I.20.15 Audit fixture: include creation, mutation, denial, and revocation events for `host_attestation`.

I.21.0 Primitive `guardian_consent_fact` is part of the ADR-0320 conformance surface.
I.21.1 Source of truth: `guardian_consent_fact` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.21.2 Tenant rule: `guardian_consent_fact` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.21.3 ADR-0244 rule: `guardian_consent_fact` cannot introduce a new tenancy primitive outside the universal tenant model.
I.21.4 ADR-0311 rule: `guardian_consent_fact` cannot merge personal and work identities into one host-owned record.
I.21.5 ADR-0313 rule: `guardian_consent_fact` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.21.6 ADR-0292 rule: `guardian_consent_fact` must block unsafe minor flows when age, guardian, or school facts are missing.
I.21.7 ADR-0299 rule: `guardian_consent_fact` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.21.8 ADR-0305 rule: `guardian_consent_fact` must expose mentor or supervisor context when capability depends on supervision.
I.21.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.21.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.21.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.21.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.21.13 Failure mode: if `guardian_consent_fact` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.21.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `guardian_consent_fact`.
I.21.15 Audit fixture: include creation, mutation, denial, and revocation events for `guardian_consent_fact`.

I.22.0 Primitive `protected_activity_flag` is part of the ADR-0320 conformance surface.
I.22.1 Source of truth: `protected_activity_flag` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.22.2 Tenant rule: `protected_activity_flag` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.22.3 ADR-0244 rule: `protected_activity_flag` cannot introduce a new tenancy primitive outside the universal tenant model.
I.22.4 ADR-0311 rule: `protected_activity_flag` cannot merge personal and work identities into one host-owned record.
I.22.5 ADR-0313 rule: `protected_activity_flag` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.22.6 ADR-0292 rule: `protected_activity_flag` must block unsafe minor flows when age, guardian, or school facts are missing.
I.22.7 ADR-0299 rule: `protected_activity_flag` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.22.8 ADR-0305 rule: `protected_activity_flag` must expose mentor or supervisor context when capability depends on supervision.
I.22.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.22.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.22.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.22.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.22.13 Failure mode: if `protected_activity_flag` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.22.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `protected_activity_flag`.
I.22.15 Audit fixture: include creation, mutation, denial, and revocation events for `protected_activity_flag`.

I.23.0 Primitive `duty_hour_window` is part of the ADR-0320 conformance surface.
I.23.1 Source of truth: `duty_hour_window` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.23.2 Tenant rule: `duty_hour_window` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.23.3 ADR-0244 rule: `duty_hour_window` cannot introduce a new tenancy primitive outside the universal tenant model.
I.23.4 ADR-0311 rule: `duty_hour_window` cannot merge personal and work identities into one host-owned record.
I.23.5 ADR-0313 rule: `duty_hour_window` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.23.6 ADR-0292 rule: `duty_hour_window` must block unsafe minor flows when age, guardian, or school facts are missing.
I.23.7 ADR-0299 rule: `duty_hour_window` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.23.8 ADR-0305 rule: `duty_hour_window` must expose mentor or supervisor context when capability depends on supervision.
I.23.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.23.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.23.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.23.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.23.13 Failure mode: if `duty_hour_window` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.23.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `duty_hour_window`.
I.23.15 Audit fixture: include creation, mutation, denial, and revocation events for `duty_hour_window`.

I.24.0 Primitive `wage_eligibility_decision` is part of the ADR-0320 conformance surface.
I.24.1 Source of truth: `wage_eligibility_decision` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.24.2 Tenant rule: `wage_eligibility_decision` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.24.3 ADR-0244 rule: `wage_eligibility_decision` cannot introduce a new tenancy primitive outside the universal tenant model.
I.24.4 ADR-0311 rule: `wage_eligibility_decision` cannot merge personal and work identities into one host-owned record.
I.24.5 ADR-0313 rule: `wage_eligibility_decision` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.24.6 ADR-0292 rule: `wage_eligibility_decision` must block unsafe minor flows when age, guardian, or school facts are missing.
I.24.7 ADR-0299 rule: `wage_eligibility_decision` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.24.8 ADR-0305 rule: `wage_eligibility_decision` must expose mentor or supervisor context when capability depends on supervision.
I.24.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.24.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.24.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.24.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.24.13 Failure mode: if `wage_eligibility_decision` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.24.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `wage_eligibility_decision`.
I.24.15 Audit fixture: include creation, mutation, denial, and revocation events for `wage_eligibility_decision`.

I.25.0 Primitive `portfolio_export_event` is part of the ADR-0320 conformance surface.
I.25.1 Source of truth: `portfolio_export_event` is defined by `oya-shared-program-transient-identity` or by a table generated from that crate contract.
I.25.2 Tenant rule: `portfolio_export_event` must be evaluated against personal, source, host, and program tenant context when those tenants are present.
I.25.3 ADR-0244 rule: `portfolio_export_event` cannot introduce a new tenancy primitive outside the universal tenant model.
I.25.4 ADR-0311 rule: `portfolio_export_event` cannot merge personal and work identities into one host-owned record.
I.25.5 ADR-0313 rule: `portfolio_export_event` must preserve child-tenant sovereignty when the host belongs to a conglomerate.
I.25.6 ADR-0292 rule: `portfolio_export_event` must block unsafe minor flows when age, guardian, or school facts are missing.
I.25.7 ADR-0299 rule: `portfolio_export_event` must be available for fairness audit when used in evaluation, ranking, pay, or conversion.
I.25.8 ADR-0305 rule: `portfolio_export_event` must expose mentor or supervisor context when capability depends on supervision.
I.25.9 Hyperscaler precedent A: temporary credentials show that scoped and expiring authority is safer than standing membership.
I.25.10 Hyperscaler precedent B: enterprise directory groups separate membership, role, and session state for operational durability.
I.25.11 Regulatory precedent A: written-term regimes require durable evidence for start date, role, pay, hours, and supervising party.
I.25.12 Regulatory precedent B: training and clinical regimes require reviewable supervision and hour boundaries.
I.25.13 Failure mode: if `portfolio_export_event` is missing, services may treat a participant as a generic employee, generic student, or generic guest.
I.25.14 Test fixture: include active, expired, revoked, suspended, minor, cross-border, and portfolio-only cases for `portfolio_export_event`.
I.25.15 Audit fixture: include creation, mutation, denial, and revocation events for `portfolio_export_event`.

## J. Program-type operating rules

J.1.0 `APPRENTICE` means vocational or workforce training with structured supervision and often wage or credential obligations.
J.1.1 Default capability tier: `supervised_operator` unless a stricter local overlay applies.
J.1.2 Membership start: activation requires source approval when a source tenant exists and host approval in all cases.
J.1.3 Membership end: scheduled revocation fires at program end, early termination, compliance block, or consent withdrawal.
J.1.4 Portfolio rule: personal tenant can retain verified completion, hours, competencies, and attestations after host redaction.
J.1.5 Pay rule: payments service must consult labor overlay before unpaid, stipend, reimbursement, wage, or payroll treatment.
J.1.6 Supervision rule: community and workplace-integration must bind at least one mentor, supervisor, preceptor, attending, or sponsor when capability exceeds observer.
J.1.7 Minor rule: identity service must evaluate ADR-0292 before messaging, public display, guardian consent, or workplace participation.
J.1.8 Fairness rule: placement, task allocation, evaluation, and conversion recommendations enter ADR-0299 audit scope.
J.1.9 Hierarchy rule: program tenant can be a child of school, employer, clinic, or parent conglomerate tenant under ADR-0313.
J.1.10 Precedent one: early-talent recruiting products separate candidate profile from temporary host program state.
J.1.11 Precedent two: learning-management and HR systems track cohorts and competencies over a bounded program interval.
J.1.12 Regulatory anchor one: written terms and predictable conditions require the program interval to be explicit.
J.1.13 Regulatory anchor two: wage, hour, safety, and supervision duties can differ by program type and jurisdiction.
J.1.14 Forbidden downgrade: `APPRENTICE` cannot be stored only as free-text title, team name, badge, or community tag.
J.1.15 Required event: `program.apprentice.membership.created` records source, host, personal, and program tenant identifiers.
J.1.16 Required event: `program.apprentice.capability.granted` records capability tier, policy version, and expiration boundary.
J.1.17 Required event: `program.apprentice.capability.revoked` records revocation cause and portfolio-retention result.
J.1.18 Required test: activation denies when host approval is missing.
J.1.19 Required test: activation denies when current time is outside the program window.
J.1.20 Required test: participant retains only allowed portable artifacts after program end.
J.1.21 Required test: cross-tenant source withdrawal blocks host capability without deleting personal tenant history.
J.1.22 Required test: unknown jurisdiction overlay blocks activation until classified.
J.1.23 Required test: minor participant without required guardian or school fact cannot activate unsafe flows.
J.1.24 Required test: mentor-dependent capability denies when mentor assignment is absent or inactive.
J.1.25 Required test: audit-chain receives immutable grant and revoke evidence for `APPRENTICE`.

J.2.0 `INTERN` means short-term educational placement with primary-beneficiary, pay, and school-host boundary risk.
J.2.1 Default capability tier: `contributor` unless a stricter local overlay applies.
J.2.2 Membership start: activation requires source approval when a source tenant exists and host approval in all cases.
J.2.3 Membership end: scheduled revocation fires at program end, early termination, compliance block, or consent withdrawal.
J.2.4 Portfolio rule: personal tenant can retain verified completion, hours, competencies, and attestations after host redaction.
J.2.5 Pay rule: payments service must consult labor overlay before unpaid, stipend, reimbursement, wage, or payroll treatment.
J.2.6 Supervision rule: community and workplace-integration must bind at least one mentor, supervisor, preceptor, attending, or sponsor when capability exceeds observer.
J.2.7 Minor rule: identity service must evaluate ADR-0292 before messaging, public display, guardian consent, or workplace participation.
J.2.8 Fairness rule: placement, task allocation, evaluation, and conversion recommendations enter ADR-0299 audit scope.
J.2.9 Hierarchy rule: program tenant can be a child of school, employer, clinic, or parent conglomerate tenant under ADR-0313.
J.2.10 Precedent one: early-talent recruiting products separate candidate profile from temporary host program state.
J.2.11 Precedent two: learning-management and HR systems track cohorts and competencies over a bounded program interval.
J.2.12 Regulatory anchor one: written terms and predictable conditions require the program interval to be explicit.
J.2.13 Regulatory anchor two: wage, hour, safety, and supervision duties can differ by program type and jurisdiction.
J.2.14 Forbidden downgrade: `INTERN` cannot be stored only as free-text title, team name, badge, or community tag.
J.2.15 Required event: `program.intern.membership.created` records source, host, personal, and program tenant identifiers.
J.2.16 Required event: `program.intern.capability.granted` records capability tier, policy version, and expiration boundary.
J.2.17 Required event: `program.intern.capability.revoked` records revocation cause and portfolio-retention result.
J.2.18 Required test: activation denies when host approval is missing.
J.2.19 Required test: activation denies when current time is outside the program window.
J.2.20 Required test: participant retains only allowed portable artifacts after program end.
J.2.21 Required test: cross-tenant source withdrawal blocks host capability without deleting personal tenant history.
J.2.22 Required test: unknown jurisdiction overlay blocks activation until classified.
J.2.23 Required test: minor participant without required guardian or school fact cannot activate unsafe flows.
J.2.24 Required test: mentor-dependent capability denies when mentor assignment is absent or inactive.
J.2.25 Required test: audit-chain receives immutable grant and revoke evidence for `INTERN`.

J.3.0 `RESIDENT` means graduate clinical training with duty-hour, supervision, patient-safety, and clinical-access constraints.
J.3.1 Default capability tier: `clinical_trainee` unless a stricter local overlay applies.
J.3.2 Membership start: activation requires source approval when a source tenant exists and host approval in all cases.
J.3.3 Membership end: scheduled revocation fires at program end, early termination, compliance block, or consent withdrawal.
J.3.4 Portfolio rule: personal tenant can retain verified completion, hours, competencies, and attestations after host redaction.
J.3.5 Pay rule: payments service must consult labor overlay before unpaid, stipend, reimbursement, wage, or payroll treatment.
J.3.6 Supervision rule: community and workplace-integration must bind at least one mentor, supervisor, preceptor, attending, or sponsor when capability exceeds observer.
J.3.7 Minor rule: identity service must evaluate ADR-0292 before messaging, public display, guardian consent, or workplace participation.
J.3.8 Fairness rule: placement, task allocation, evaluation, and conversion recommendations enter ADR-0299 audit scope.
J.3.9 Hierarchy rule: program tenant can be a child of school, employer, clinic, or parent conglomerate tenant under ADR-0313.
J.3.10 Precedent one: early-talent recruiting products separate candidate profile from temporary host program state.
J.3.11 Precedent two: learning-management and HR systems track cohorts and competencies over a bounded program interval.
J.3.12 Regulatory anchor one: written terms and predictable conditions require the program interval to be explicit.
J.3.13 Regulatory anchor two: wage, hour, safety, and supervision duties can differ by program type and jurisdiction.
J.3.14 Forbidden downgrade: `RESIDENT` cannot be stored only as free-text title, team name, badge, or community tag.
J.3.15 Required event: `program.resident.membership.created` records source, host, personal, and program tenant identifiers.
J.3.16 Required event: `program.resident.capability.granted` records capability tier, policy version, and expiration boundary.
J.3.17 Required event: `program.resident.capability.revoked` records revocation cause and portfolio-retention result.
J.3.18 Required test: activation denies when host approval is missing.
J.3.19 Required test: activation denies when current time is outside the program window.
J.3.20 Required test: participant retains only allowed portable artifacts after program end.
J.3.21 Required test: cross-tenant source withdrawal blocks host capability without deleting personal tenant history.
J.3.22 Required test: unknown jurisdiction overlay blocks activation until classified.
J.3.23 Required test: minor participant without required guardian or school fact cannot activate unsafe flows.
J.3.24 Required test: mentor-dependent capability denies when mentor assignment is absent or inactive.
J.3.25 Required test: audit-chain receives immutable grant and revoke evidence for `RESIDENT`.

J.4.0 `FELLOW` means advanced professional, clinical, research, policy, or leadership training with bounded host authority.
J.4.1 Default capability tier: `research_trainee` unless a stricter local overlay applies.
J.4.2 Membership start: activation requires source approval when a source tenant exists and host approval in all cases.
J.4.3 Membership end: scheduled revocation fires at program end, early termination, compliance block, or consent withdrawal.
J.4.4 Portfolio rule: personal tenant can retain verified completion, hours, competencies, and attestations after host redaction.
J.4.5 Pay rule: payments service must consult labor overlay before unpaid, stipend, reimbursement, wage, or payroll treatment.
J.4.6 Supervision rule: community and workplace-integration must bind at least one mentor, supervisor, preceptor, attending, or sponsor when capability exceeds observer.
J.4.7 Minor rule: identity service must evaluate ADR-0292 before messaging, public display, guardian consent, or workplace participation.
J.4.8 Fairness rule: placement, task allocation, evaluation, and conversion recommendations enter ADR-0299 audit scope.
J.4.9 Hierarchy rule: program tenant can be a child of school, employer, clinic, or parent conglomerate tenant under ADR-0313.
J.4.10 Precedent one: early-talent recruiting products separate candidate profile from temporary host program state.
J.4.11 Precedent two: learning-management and HR systems track cohorts and competencies over a bounded program interval.
J.4.12 Regulatory anchor one: written terms and predictable conditions require the program interval to be explicit.
J.4.13 Regulatory anchor two: wage, hour, safety, and supervision duties can differ by program type and jurisdiction.
J.4.14 Forbidden downgrade: `FELLOW` cannot be stored only as free-text title, team name, badge, or community tag.
J.4.15 Required event: `program.fellow.membership.created` records source, host, personal, and program tenant identifiers.
J.4.16 Required event: `program.fellow.capability.granted` records capability tier, policy version, and expiration boundary.
J.4.17 Required event: `program.fellow.capability.revoked` records revocation cause and portfolio-retention result.
J.4.18 Required test: activation denies when host approval is missing.
J.4.19 Required test: activation denies when current time is outside the program window.
J.4.20 Required test: participant retains only allowed portable artifacts after program end.
J.4.21 Required test: cross-tenant source withdrawal blocks host capability without deleting personal tenant history.
J.4.22 Required test: unknown jurisdiction overlay blocks activation until classified.
J.4.23 Required test: minor participant without required guardian or school fact cannot activate unsafe flows.
J.4.24 Required test: mentor-dependent capability denies when mentor assignment is absent or inactive.
J.4.25 Required test: audit-chain receives immutable grant and revoke evidence for `FELLOW`.

J.5.0 `COOP` means cooperative education placement that may alternate academic and host work terms.
J.5.1 Default capability tier: `compensated_worker` unless a stricter local overlay applies.
J.5.2 Membership start: activation requires source approval when a source tenant exists and host approval in all cases.
J.5.3 Membership end: scheduled revocation fires at program end, early termination, compliance block, or consent withdrawal.
J.5.4 Portfolio rule: personal tenant can retain verified completion, hours, competencies, and attestations after host redaction.
J.5.5 Pay rule: payments service must consult labor overlay before unpaid, stipend, reimbursement, wage, or payroll treatment.
J.5.6 Supervision rule: community and workplace-integration must bind at least one mentor, supervisor, preceptor, attending, or sponsor when capability exceeds observer.
J.5.7 Minor rule: identity service must evaluate ADR-0292 before messaging, public display, guardian consent, or workplace participation.
J.5.8 Fairness rule: placement, task allocation, evaluation, and conversion recommendations enter ADR-0299 audit scope.
J.5.9 Hierarchy rule: program tenant can be a child of school, employer, clinic, or parent conglomerate tenant under ADR-0313.
J.5.10 Precedent one: early-talent recruiting products separate candidate profile from temporary host program state.
J.5.11 Precedent two: learning-management and HR systems track cohorts and competencies over a bounded program interval.
J.5.12 Regulatory anchor one: written terms and predictable conditions require the program interval to be explicit.
J.5.13 Regulatory anchor two: wage, hour, safety, and supervision duties can differ by program type and jurisdiction.
J.5.14 Forbidden downgrade: `COOP` cannot be stored only as free-text title, team name, badge, or community tag.
J.5.15 Required event: `program.coop.membership.created` records source, host, personal, and program tenant identifiers.
J.5.16 Required event: `program.coop.capability.granted` records capability tier, policy version, and expiration boundary.
J.5.17 Required event: `program.coop.capability.revoked` records revocation cause and portfolio-retention result.
J.5.18 Required test: activation denies when host approval is missing.
J.5.19 Required test: activation denies when current time is outside the program window.
J.5.20 Required test: participant retains only allowed portable artifacts after program end.
J.5.21 Required test: cross-tenant source withdrawal blocks host capability without deleting personal tenant history.
J.5.22 Required test: unknown jurisdiction overlay blocks activation until classified.
J.5.23 Required test: minor participant without required guardian or school fact cannot activate unsafe flows.
J.5.24 Required test: mentor-dependent capability denies when mentor assignment is absent or inactive.
J.5.25 Required test: audit-chain receives immutable grant and revoke evidence for `COOP`.

J.6.0 `EXTERN` means observation or shadowing placement with strongly limited production permissions.
J.6.1 Default capability tier: `observer` unless a stricter local overlay applies.
J.6.2 Membership start: activation requires source approval when a source tenant exists and host approval in all cases.
J.6.3 Membership end: scheduled revocation fires at program end, early termination, compliance block, or consent withdrawal.
J.6.4 Portfolio rule: personal tenant can retain verified completion, hours, competencies, and attestations after host redaction.
J.6.5 Pay rule: payments service must consult labor overlay before unpaid, stipend, reimbursement, wage, or payroll treatment.
J.6.6 Supervision rule: community and workplace-integration must bind at least one mentor, supervisor, preceptor, attending, or sponsor when capability exceeds observer.
J.6.7 Minor rule: identity service must evaluate ADR-0292 before messaging, public display, guardian consent, or workplace participation.
J.6.8 Fairness rule: placement, task allocation, evaluation, and conversion recommendations enter ADR-0299 audit scope.
J.6.9 Hierarchy rule: program tenant can be a child of school, employer, clinic, or parent conglomerate tenant under ADR-0313.
J.6.10 Precedent one: early-talent recruiting products separate candidate profile from temporary host program state.
J.6.11 Precedent two: learning-management and HR systems track cohorts and competencies over a bounded program interval.
J.6.12 Regulatory anchor one: written terms and predictable conditions require the program interval to be explicit.
J.6.13 Regulatory anchor two: wage, hour, safety, and supervision duties can differ by program type and jurisdiction.
J.6.14 Forbidden downgrade: `EXTERN` cannot be stored only as free-text title, team name, badge, or community tag.
J.6.15 Required event: `program.extern.membership.created` records source, host, personal, and program tenant identifiers.
J.6.16 Required event: `program.extern.capability.granted` records capability tier, policy version, and expiration boundary.
J.6.17 Required event: `program.extern.capability.revoked` records revocation cause and portfolio-retention result.
J.6.18 Required test: activation denies when host approval is missing.
J.6.19 Required test: activation denies when current time is outside the program window.
J.6.20 Required test: participant retains only allowed portable artifacts after program end.
J.6.21 Required test: cross-tenant source withdrawal blocks host capability without deleting personal tenant history.
J.6.22 Required test: unknown jurisdiction overlay blocks activation until classified.
J.6.23 Required test: minor participant without required guardian or school fact cannot activate unsafe flows.
J.6.24 Required test: mentor-dependent capability denies when mentor assignment is absent or inactive.
J.6.25 Required test: audit-chain receives immutable grant and revoke evidence for `EXTERN`.

## K. Service contract matrix

K.1.0 Service `community` consumes ADR-0320 through the shared crate and must not fork the doctrine.
K.1.1.1 Surface `cohort_channel` records membership id, program type, capability tier, tenant tuple, and policy version.
K.1.1.2 Surface `cohort_channel` rejects expired, revoked, suspended, or unknown program memberships.
K.1.1.3 Surface `cohort_channel` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.1.1.4 Surface `cohort_channel` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.1.2.1 Surface `mentor_thread` records membership id, program type, capability tier, tenant tuple, and policy version.
K.1.2.2 Surface `mentor_thread` rejects expired, revoked, suspended, or unknown program memberships.
K.1.2.3 Surface `mentor_thread` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.1.2.4 Surface `mentor_thread` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.1.3.1 Surface `peer_group` records membership id, program type, capability tier, tenant tuple, and policy version.
K.1.3.2 Surface `peer_group` rejects expired, revoked, suspended, or unknown program memberships.
K.1.3.3 Surface `peer_group` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.1.3.4 Surface `peer_group` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.1.4.1 Surface `portfolio_comment` records membership id, program type, capability tier, tenant tuple, and policy version.
K.1.4.2 Surface `portfolio_comment` rejects expired, revoked, suspended, or unknown program memberships.
K.1.4.3 Surface `portfolio_comment` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.1.4.4 Surface `portfolio_comment` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.1.5.1 Surface `work_sample_visibility` records membership id, program type, capability tier, tenant tuple, and policy version.
K.1.5.2 Surface `work_sample_visibility` rejects expired, revoked, suspended, or unknown program memberships.
K.1.5.3 Surface `work_sample_visibility` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.1.5.4 Surface `work_sample_visibility` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.1.6.1 Surface `guardian_safe_messaging` records membership id, program type, capability tier, tenant tuple, and policy version.
K.1.6.2 Surface `guardian_safe_messaging` rejects expired, revoked, suspended, or unknown program memberships.
K.1.6.3 Surface `guardian_safe_messaging` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.1.6.4 Surface `guardian_safe_messaging` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.

K.2.0 Service `identity` consumes ADR-0320 through the shared crate and must not fork the doctrine.
K.2.1.1 Surface `person_binding` records membership id, program type, capability tier, tenant tuple, and policy version.
K.2.1.2 Surface `person_binding` rejects expired, revoked, suspended, or unknown program memberships.
K.2.1.3 Surface `person_binding` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.2.1.4 Surface `person_binding` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.2.2.1 Surface `tenant_membership` records membership id, program type, capability tier, tenant tuple, and policy version.
K.2.2.2 Surface `tenant_membership` rejects expired, revoked, suspended, or unknown program memberships.
K.2.2.3 Surface `tenant_membership` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.2.2.4 Surface `tenant_membership` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.2.3.1 Surface `source_attestation` records membership id, program type, capability tier, tenant tuple, and policy version.
K.2.3.2 Surface `source_attestation` rejects expired, revoked, suspended, or unknown program memberships.
K.2.3.3 Surface `source_attestation` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.2.3.4 Surface `source_attestation` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.2.4.1 Surface `host_attestation` records membership id, program type, capability tier, tenant tuple, and policy version.
K.2.4.2 Surface `host_attestation` rejects expired, revoked, suspended, or unknown program memberships.
K.2.4.3 Surface `host_attestation` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.2.4.4 Surface `host_attestation` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.2.5.1 Surface `age_gate` records membership id, program type, capability tier, tenant tuple, and policy version.
K.2.5.2 Surface `age_gate` rejects expired, revoked, suspended, or unknown program memberships.
K.2.5.3 Surface `age_gate` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.2.5.4 Surface `age_gate` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.2.6.1 Surface `personal_work_boundary` records membership id, program type, capability tier, tenant tuple, and policy version.
K.2.6.2 Surface `personal_work_boundary` rejects expired, revoked, suspended, or unknown program memberships.
K.2.6.3 Surface `personal_work_boundary` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.2.6.4 Surface `personal_work_boundary` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.

K.3.0 Service `workplace-integration` consumes ADR-0320 through the shared crate and must not fork the doctrine.
K.3.1.1 Surface `hris_roster` records membership id, program type, capability tier, tenant tuple, and policy version.
K.3.1.2 Surface `hris_roster` rejects expired, revoked, suspended, or unknown program memberships.
K.3.1.3 Surface `hris_roster` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.3.1.4 Surface `hris_roster` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.3.2.1 Surface `schedule_import` records membership id, program type, capability tier, tenant tuple, and policy version.
K.3.2.2 Surface `schedule_import` rejects expired, revoked, suspended, or unknown program memberships.
K.3.2.3 Surface `schedule_import` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.3.2.4 Surface `schedule_import` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.3.3.1 Surface `supervisor_map` records membership id, program type, capability tier, tenant tuple, and policy version.
K.3.3.2 Surface `supervisor_map` rejects expired, revoked, suspended, or unknown program memberships.
K.3.3.3 Surface `supervisor_map` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.3.3.4 Surface `supervisor_map` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.3.4.1 Surface `assignment_sync` records membership id, program type, capability tier, tenant tuple, and policy version.
K.3.4.2 Surface `assignment_sync` rejects expired, revoked, suspended, or unknown program memberships.
K.3.4.3 Surface `assignment_sync` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.3.4.4 Surface `assignment_sync` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.3.5.1 Surface `conversion_handoff` records membership id, program type, capability tier, tenant tuple, and policy version.
K.3.5.2 Surface `conversion_handoff` rejects expired, revoked, suspended, or unknown program memberships.
K.3.5.3 Surface `conversion_handoff` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.3.5.4 Surface `conversion_handoff` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.3.6.1 Surface `offboarding_signal` records membership id, program type, capability tier, tenant tuple, and policy version.
K.3.6.2 Surface `offboarding_signal` rejects expired, revoked, suspended, or unknown program memberships.
K.3.6.3 Surface `offboarding_signal` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.3.6.4 Surface `offboarding_signal` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.

K.4.0 Service `payments` consumes ADR-0320 through the shared crate and must not fork the doctrine.
K.4.1.1 Surface `wage_decision` records membership id, program type, capability tier, tenant tuple, and policy version.
K.4.1.2 Surface `wage_decision` rejects expired, revoked, suspended, or unknown program memberships.
K.4.1.3 Surface `wage_decision` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.4.1.4 Surface `wage_decision` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.4.2.1 Surface `stipend_decision` records membership id, program type, capability tier, tenant tuple, and policy version.
K.4.2.2 Surface `stipend_decision` rejects expired, revoked, suspended, or unknown program memberships.
K.4.2.3 Surface `stipend_decision` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.4.2.4 Surface `stipend_decision` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.4.3.1 Surface `reimbursement_decision` records membership id, program type, capability tier, tenant tuple, and policy version.
K.4.3.2 Surface `reimbursement_decision` rejects expired, revoked, suspended, or unknown program memberships.
K.4.3.3 Surface `reimbursement_decision` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.4.3.4 Surface `reimbursement_decision` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.4.4.1 Surface `unpaid_block` records membership id, program type, capability tier, tenant tuple, and policy version.
K.4.4.2 Surface `unpaid_block` rejects expired, revoked, suspended, or unknown program memberships.
K.4.4.3 Surface `unpaid_block` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.4.4.4 Surface `unpaid_block` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.4.5.1 Surface `tax_hint` records membership id, program type, capability tier, tenant tuple, and policy version.
K.4.5.2 Surface `tax_hint` rejects expired, revoked, suspended, or unknown program memberships.
K.4.5.3 Surface `tax_hint` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.4.5.4 Surface `tax_hint` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.4.6.1 Surface `payroll_export` records membership id, program type, capability tier, tenant tuple, and policy version.
K.4.6.2 Surface `payroll_export` rejects expired, revoked, suspended, or unknown program memberships.
K.4.6.3 Surface `payroll_export` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.4.6.4 Surface `payroll_export` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.

K.5.0 Service `audit-chain` consumes ADR-0320 through the shared crate and must not fork the doctrine.
K.5.1.1 Surface `grant_event` records membership id, program type, capability tier, tenant tuple, and policy version.
K.5.1.2 Surface `grant_event` rejects expired, revoked, suspended, or unknown program memberships.
K.5.1.3 Surface `grant_event` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.5.1.4 Surface `grant_event` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.5.2.1 Surface `deny_event` records membership id, program type, capability tier, tenant tuple, and policy version.
K.5.2.2 Surface `deny_event` rejects expired, revoked, suspended, or unknown program memberships.
K.5.2.3 Surface `deny_event` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.5.2.4 Surface `deny_event` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.5.3.1 Surface `overlay_event` records membership id, program type, capability tier, tenant tuple, and policy version.
K.5.3.2 Surface `overlay_event` rejects expired, revoked, suspended, or unknown program memberships.
K.5.3.3 Surface `overlay_event` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.5.3.4 Surface `overlay_event` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.5.4.1 Surface `hour_cert_event` records membership id, program type, capability tier, tenant tuple, and policy version.
K.5.4.2 Surface `hour_cert_event` rejects expired, revoked, suspended, or unknown program memberships.
K.5.4.3 Surface `hour_cert_event` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.5.4.4 Surface `hour_cert_event` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.5.5.1 Surface `revocation_event` records membership id, program type, capability tier, tenant tuple, and policy version.
K.5.5.2 Surface `revocation_event` rejects expired, revoked, suspended, or unknown program memberships.
K.5.5.3 Surface `revocation_event` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.5.5.4 Surface `revocation_event` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.
K.5.6.1 Surface `portfolio_export_event` records membership id, program type, capability tier, tenant tuple, and policy version.
K.5.6.2 Surface `portfolio_export_event` rejects expired, revoked, suspended, or unknown program memberships.
K.5.6.3 Surface `portfolio_export_event` emits audit evidence when it grants, denies, exports, or mutates program-scoped state.
K.5.6.4 Surface `portfolio_export_event` includes a fixture for at least one apprentice, one intern, one resident, and one fellow.

## L. Event catalog

L.1.1 Event `program.membership.drafted` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.1.2 Event `program.membership.drafted` is append-only and cannot be rewritten by consumer service retries.
L.1.3 Event `program.membership.drafted` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.2.1 Event `program.membership.source_approved` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.2.2 Event `program.membership.source_approved` is append-only and cannot be rewritten by consumer service retries.
L.2.3 Event `program.membership.source_approved` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.3.1 Event `program.membership.host_approved` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.3.2 Event `program.membership.host_approved` is append-only and cannot be rewritten by consumer service retries.
L.3.3 Event `program.membership.host_approved` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.4.1 Event `program.membership.activated` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.4.2 Event `program.membership.activated` is append-only and cannot be rewritten by consumer service retries.
L.4.3 Event `program.membership.activated` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.5.1 Event `program.membership.suspended` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.5.2 Event `program.membership.suspended` is append-only and cannot be rewritten by consumer service retries.
L.5.3 Event `program.membership.suspended` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.6.1 Event `program.membership.completed` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.6.2 Event `program.membership.completed` is append-only and cannot be rewritten by consumer service retries.
L.6.3 Event `program.membership.completed` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.7.1 Event `program.membership.terminated` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.7.2 Event `program.membership.terminated` is append-only and cannot be rewritten by consumer service retries.
L.7.3 Event `program.membership.terminated` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.8.1 Event `program.membership.revoked` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.8.2 Event `program.membership.revoked` is append-only and cannot be rewritten by consumer service retries.
L.8.3 Event `program.membership.revoked` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.9.1 Event `program.capability.requested` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.9.2 Event `program.capability.requested` is append-only and cannot be rewritten by consumer service retries.
L.9.3 Event `program.capability.requested` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.10.1 Event `program.capability.granted` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.10.2 Event `program.capability.granted` is append-only and cannot be rewritten by consumer service retries.
L.10.3 Event `program.capability.granted` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.11.1 Event `program.capability.denied` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.11.2 Event `program.capability.denied` is append-only and cannot be rewritten by consumer service retries.
L.11.3 Event `program.capability.denied` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.12.1 Event `program.capability.revoked` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.12.2 Event `program.capability.revoked` is append-only and cannot be rewritten by consumer service retries.
L.12.3 Event `program.capability.revoked` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.13.1 Event `program.overlay.attached` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.13.2 Event `program.overlay.attached` is append-only and cannot be rewritten by consumer service retries.
L.13.3 Event `program.overlay.attached` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.14.1 Event `program.overlay.evaluated` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.14.2 Event `program.overlay.evaluated` is append-only and cannot be rewritten by consumer service retries.
L.14.3 Event `program.overlay.evaluated` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.15.1 Event `program.overlay.blocked` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.15.2 Event `program.overlay.blocked` is append-only and cannot be rewritten by consumer service retries.
L.15.3 Event `program.overlay.blocked` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.16.1 Event `program.mentor.assigned` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.16.2 Event `program.mentor.assigned` is append-only and cannot be rewritten by consumer service retries.
L.16.3 Event `program.mentor.assigned` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.17.1 Event `program.mentor.reassigned` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.17.2 Event `program.mentor.reassigned` is append-only and cannot be rewritten by consumer service retries.
L.17.3 Event `program.mentor.reassigned` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.18.1 Event `program.mentor.removed` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.18.2 Event `program.mentor.removed` is append-only and cannot be rewritten by consumer service retries.
L.18.3 Event `program.mentor.removed` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.19.1 Event `program.hours.submitted` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.19.2 Event `program.hours.submitted` is append-only and cannot be rewritten by consumer service retries.
L.19.3 Event `program.hours.submitted` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.20.1 Event `program.hours.certified` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.20.2 Event `program.hours.certified` is append-only and cannot be rewritten by consumer service retries.
L.20.3 Event `program.hours.certified` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.21.1 Event `program.hours.disputed` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.21.2 Event `program.hours.disputed` is append-only and cannot be rewritten by consumer service retries.
L.21.3 Event `program.hours.disputed` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.22.1 Event `program.competency.claimed` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.22.2 Event `program.competency.claimed` is append-only and cannot be rewritten by consumer service retries.
L.22.3 Event `program.competency.claimed` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.23.1 Event `program.competency.verified` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.23.2 Event `program.competency.verified` is append-only and cannot be rewritten by consumer service retries.
L.23.3 Event `program.competency.verified` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.24.1 Event `program.competency.rejected` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.24.2 Event `program.competency.rejected` is append-only and cannot be rewritten by consumer service retries.
L.24.3 Event `program.competency.rejected` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.25.1 Event `program.payment.classification_requested` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.25.2 Event `program.payment.classification_requested` is append-only and cannot be rewritten by consumer service retries.
L.25.3 Event `program.payment.classification_requested` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.26.1 Event `program.payment.classification_decided` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.26.2 Event `program.payment.classification_decided` is append-only and cannot be rewritten by consumer service retries.
L.26.3 Event `program.payment.classification_decided` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.27.1 Event `program.payment.exported` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.27.2 Event `program.payment.exported` is append-only and cannot be rewritten by consumer service retries.
L.27.3 Event `program.payment.exported` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.28.1 Event `program.portfolio.retention_evaluated` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.28.2 Event `program.portfolio.retention_evaluated` is append-only and cannot be rewritten by consumer service retries.
L.28.3 Event `program.portfolio.retention_evaluated` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.29.1 Event `program.portfolio.exported` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.29.2 Event `program.portfolio.exported` is append-only and cannot be rewritten by consumer service retries.
L.29.3 Event `program.portfolio.exported` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.30.1 Event `program.portfolio.redacted` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.30.2 Event `program.portfolio.redacted` is append-only and cannot be rewritten by consumer service retries.
L.30.3 Event `program.portfolio.redacted` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.31.1 Event `program.conversion.requested` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.31.2 Event `program.conversion.requested` is append-only and cannot be rewritten by consumer service retries.
L.31.3 Event `program.conversion.requested` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.32.1 Event `program.conversion.approved` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.32.2 Event `program.conversion.approved` is append-only and cannot be rewritten by consumer service retries.
L.32.3 Event `program.conversion.approved` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.
L.33.1 Event `program.conversion.denied` includes `event_id`, `membership_id`, `program_type`, `program_tenant_id`, `host_tenant_id`, `personal_tenant_id`, `actor_id`, `occurred_at`, `policy_version`, and `evidence_hash`.
L.33.2 Event `program.conversion.denied` is append-only and cannot be rewritten by consumer service retries.
L.33.3 Event `program.conversion.denied` is eligible for audit-chain proof export when a participant, school, host, regulator, or court-request workflow needs evidence.

## M. Test and verification matrix

M.1.1 Test `activation_denies_without_host_tenant` is required for ADR-0320 implementation acceptance.
M.1.2 Test `activation_denies_without_host_tenant` must include the tenant tuple and policy version in its fixture.
M.1.3 Test `activation_denies_without_host_tenant` must run against the shared crate contract before any consumer service claims conformance.
M.2.1 Test `activation_denies_without_program_tenant` is required for ADR-0320 implementation acceptance.
M.2.2 Test `activation_denies_without_program_tenant` must include the tenant tuple and policy version in its fixture.
M.2.3 Test `activation_denies_without_program_tenant` must run against the shared crate contract before any consumer service claims conformance.
M.3.1 Test `activation_denies_when_end_before_start` is required for ADR-0320 implementation acceptance.
M.3.2 Test `activation_denies_when_end_before_start` must include the tenant tuple and policy version in its fixture.
M.3.3 Test `activation_denies_when_end_before_start` must run against the shared crate contract before any consumer service claims conformance.
M.4.1 Test `activation_denies_unknown_program_type` is required for ADR-0320 implementation acceptance.
M.4.2 Test `activation_denies_unknown_program_type` must include the tenant tuple and policy version in its fixture.
M.4.3 Test `activation_denies_unknown_program_type` must run against the shared crate contract before any consumer service claims conformance.
M.5.1 Test `activation_denies_unknown_capability_tier` is required for ADR-0320 implementation acceptance.
M.5.2 Test `activation_denies_unknown_capability_tier` must include the tenant tuple and policy version in its fixture.
M.5.3 Test `activation_denies_unknown_capability_tier` must run against the shared crate contract before any consumer service claims conformance.
M.6.1 Test `activation_denies_missing_labor_overlay` is required for ADR-0320 implementation acceptance.
M.6.2 Test `activation_denies_missing_labor_overlay` must include the tenant tuple and policy version in its fixture.
M.6.3 Test `activation_denies_missing_labor_overlay` must run against the shared crate contract before any consumer service claims conformance.
M.7.1 Test `activation_denies_minor_without_required_guardian_fact` is required for ADR-0320 implementation acceptance.
M.7.2 Test `activation_denies_minor_without_required_guardian_fact` must include the tenant tuple and policy version in its fixture.
M.7.3 Test `activation_denies_minor_without_required_guardian_fact` must run against the shared crate contract before any consumer service claims conformance.
M.8.1 Test `activation_denies_resident_without_supervisor` is required for ADR-0320 implementation acceptance.
M.8.2 Test `activation_denies_resident_without_supervisor` must include the tenant tuple and policy version in its fixture.
M.8.3 Test `activation_denies_resident_without_supervisor` must run against the shared crate contract before any consumer service claims conformance.
M.9.1 Test `activation_denies_intern_unpaid_without_primary_beneficiary_record` is required for ADR-0320 implementation acceptance.
M.9.2 Test `activation_denies_intern_unpaid_without_primary_beneficiary_record` must include the tenant tuple and policy version in its fixture.
M.9.3 Test `activation_denies_intern_unpaid_without_primary_beneficiary_record` must run against the shared crate contract before any consumer service claims conformance.
M.10.1 Test `activation_denies_kr_worker_without_written_terms` is required for ADR-0320 implementation acceptance.
M.10.2 Test `activation_denies_kr_worker_without_written_terms` must include the tenant tuple and policy version in its fixture.
M.10.3 Test `activation_denies_kr_worker_without_written_terms` must run against the shared crate contract before any consumer service claims conformance.
M.11.1 Test `activation_denies_eu_trainee_without_written_information` is required for ADR-0320 implementation acceptance.
M.11.2 Test `activation_denies_eu_trainee_without_written_information` must include the tenant tuple and policy version in its fixture.
M.11.3 Test `activation_denies_eu_trainee_without_written_information` must run against the shared crate contract before any consumer service claims conformance.
M.12.1 Test `revocation_runs_at_program_end` is required for ADR-0320 implementation acceptance.
M.12.2 Test `revocation_runs_at_program_end` must include the tenant tuple and policy version in its fixture.
M.12.3 Test `revocation_runs_at_program_end` must run against the shared crate contract before any consumer service claims conformance.
M.13.1 Test `early_revoke_blocks_all_host_capabilities` is required for ADR-0320 implementation acceptance.
M.13.2 Test `early_revoke_blocks_all_host_capabilities` must include the tenant tuple and policy version in its fixture.
M.13.3 Test `early_revoke_blocks_all_host_capabilities` must run against the shared crate contract before any consumer service claims conformance.
M.14.1 Test `source_withdrawal_blocks_host_capabilities` is required for ADR-0320 implementation acceptance.
M.14.2 Test `source_withdrawal_blocks_host_capabilities` must include the tenant tuple and policy version in its fixture.
M.14.3 Test `source_withdrawal_blocks_host_capabilities` must run against the shared crate contract before any consumer service claims conformance.
M.15.1 Test `portfolio_retains_verified_completion` is required for ADR-0320 implementation acceptance.
M.15.2 Test `portfolio_retains_verified_completion` must include the tenant tuple and policy version in its fixture.
M.15.3 Test `portfolio_retains_verified_completion` must run against the shared crate contract before any consumer service claims conformance.
M.16.1 Test `portfolio_redacts_host_confidential_content` is required for ADR-0320 implementation acceptance.
M.16.2 Test `portfolio_redacts_host_confidential_content` must include the tenant tuple and policy version in its fixture.
M.16.3 Test `portfolio_redacts_host_confidential_content` must run against the shared crate contract before any consumer service claims conformance.
M.17.1 Test `portfolio_redacts_patient_content` is required for ADR-0320 implementation acceptance.
M.17.2 Test `portfolio_redacts_patient_content` must include the tenant tuple and policy version in its fixture.
M.17.3 Test `portfolio_redacts_patient_content` must run against the shared crate contract before any consumer service claims conformance.
M.18.1 Test `portfolio_redacts_school_confidential_content` is required for ADR-0320 implementation acceptance.
M.18.2 Test `portfolio_redacts_school_confidential_content` must include the tenant tuple and policy version in its fixture.
M.18.3 Test `portfolio_redacts_school_confidential_content` must run against the shared crate contract before any consumer service claims conformance.
M.19.1 Test `payments_blocks_unpaid_when_overlay_requires_wage` is required for ADR-0320 implementation acceptance.
M.19.2 Test `payments_blocks_unpaid_when_overlay_requires_wage` must include the tenant tuple and policy version in its fixture.
M.19.3 Test `payments_blocks_unpaid_when_overlay_requires_wage` must run against the shared crate contract before any consumer service claims conformance.
M.20.1 Test `payments_allows_reimbursement_when_overlay_permits` is required for ADR-0320 implementation acceptance.
M.20.2 Test `payments_allows_reimbursement_when_overlay_permits` must include the tenant tuple and policy version in its fixture.
M.20.3 Test `payments_allows_reimbursement_when_overlay_permits` must run against the shared crate contract before any consumer service claims conformance.
M.21.1 Test `payments_records_decision_source` is required for ADR-0320 implementation acceptance.
M.21.2 Test `payments_records_decision_source` must include the tenant tuple and policy version in its fixture.
M.21.3 Test `payments_records_decision_source` must run against the shared crate contract before any consumer service claims conformance.
M.22.1 Test `community_blocks_unsafe_minor_messaging` is required for ADR-0320 implementation acceptance.
M.22.2 Test `community_blocks_unsafe_minor_messaging` must include the tenant tuple and policy version in its fixture.
M.22.3 Test `community_blocks_unsafe_minor_messaging` must run against the shared crate contract before any consumer service claims conformance.
M.23.1 Test `community_allows_supervised_peer_channel` is required for ADR-0320 implementation acceptance.
M.23.2 Test `community_allows_supervised_peer_channel` must include the tenant tuple and policy version in its fixture.
M.23.3 Test `community_allows_supervised_peer_channel` must run against the shared crate contract before any consumer service claims conformance.
M.24.1 Test `identity_preserves_personal_work_boundary` is required for ADR-0320 implementation acceptance.
M.24.2 Test `identity_preserves_personal_work_boundary` must include the tenant tuple and policy version in its fixture.
M.24.3 Test `identity_preserves_personal_work_boundary` must run against the shared crate contract before any consumer service claims conformance.
M.25.1 Test `workplace_import_maps_supervisor` is required for ADR-0320 implementation acceptance.
M.25.2 Test `workplace_import_maps_supervisor` must include the tenant tuple and policy version in its fixture.
M.25.3 Test `workplace_import_maps_supervisor` must run against the shared crate contract before any consumer service claims conformance.
M.26.1 Test `workplace_import_rejects_unknown_status` is required for ADR-0320 implementation acceptance.
M.26.2 Test `workplace_import_rejects_unknown_status` must include the tenant tuple and policy version in its fixture.
M.26.3 Test `workplace_import_rejects_unknown_status` must run against the shared crate contract before any consumer service claims conformance.
M.27.1 Test `audit_chain_receives_grant_event` is required for ADR-0320 implementation acceptance.
M.27.2 Test `audit_chain_receives_grant_event` must include the tenant tuple and policy version in its fixture.
M.27.3 Test `audit_chain_receives_grant_event` must run against the shared crate contract before any consumer service claims conformance.
M.28.1 Test `audit_chain_receives_revoke_event` is required for ADR-0320 implementation acceptance.
M.28.2 Test `audit_chain_receives_revoke_event` must include the tenant tuple and policy version in its fixture.
M.28.3 Test `audit_chain_receives_revoke_event` must run against the shared crate contract before any consumer service claims conformance.
M.29.1 Test `fairness_export_contains_conversion_inputs` is required for ADR-0320 implementation acceptance.
M.29.2 Test `fairness_export_contains_conversion_inputs` must include the tenant tuple and policy version in its fixture.
M.29.3 Test `fairness_export_contains_conversion_inputs` must run against the shared crate contract before any consumer service claims conformance.
M.30.1 Test `mentor_assignment_required_for_supervised_operator` is required for ADR-0320 implementation acceptance.
M.30.2 Test `mentor_assignment_required_for_supervised_operator` must include the tenant tuple and policy version in its fixture.
M.30.3 Test `mentor_assignment_required_for_supervised_operator` must run against the shared crate contract before any consumer service claims conformance.

## N. Open implementation sequence

N.1 Land the shared crate with enums, structs, validators, event constants, and Cedar context builders.

N.2 Add schema migrations for program memberships, overlays, portfolio retention profiles, hour records, competency claims, and audit event links.

N.3 Wire identity as the first consumer because person binding and tenant tuple validation sit upstream of all other consumers.

N.4 Wire audit-chain second so later consumers emit evidence from the first enabled path.

N.5 Wire community for mentor channels, cohort membership, and safe program messaging.

N.6 Wire workplace-integration for HRIS, school roster, schedule, supervisor, and conversion handoff flows.

N.7 Wire payments only after labor overlays and wage/stipend/unpaid classification fixtures pass.

N.8 Run shadow-mode policy checks for existing internship, apprenticeship, residency, fellowship, co-op, and extern labels.

N.9 Enable enforcement per tenant cohort after mismatches are reviewed and backfilled.

N.10 Publish service-contract documentation and add ADR-0320 to the canonical doc index when the documentation automation lane is active.

## O. Doctrine summary

O.1 Apprentices, interns, residents, fellows, co-ops, and externs are not weak employees, weak students, or community badges.

O.2 They are cross-tenant, time-bounded program identities whose capabilities must be granted, supervised, audited, and revoked through policy.

O.3 Their host authority expires or converts, while their personal tenant can retain verified portable evidence under consent, confidentiality, and retention rules.

O.4 Their legal treatment depends on jurisdiction and facts, so the platform stores exact labor, training, clinical, and protected-activity overlays.

O.5 Their implementation belongs in one shared crate and five consumer services, with typed events and policy facts rather than string labels.
