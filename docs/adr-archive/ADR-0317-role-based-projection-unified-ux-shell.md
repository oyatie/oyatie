---
id: ADR-0317
status: Superseded
date: 2026-05-20
doc_class: Architecture-Decision-Record
owners:
  - council-architecture
  - council-design-system
  - council-product
  - council-security
  - council-privacy
  - ops-sre-reliability
  - axis-tenancy
  - axis-identity
  - axis-policy-engine
  - axis-ontology
  - axis-workflow
  - axis-application
supersedes: []
amends:
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0299-account-recovery-resilience.md
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md
superseded_by: [ADR-709]
related:
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0292-minor-user-doctrine-coppa-kosa-eu-age-verification.md
  - ADR-0299-account-recovery-resilience.md
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md
  - ADR-0316 (reserved adjacent doctrine; file absent in current checkout at authoring time)
related_standards:
  - documentation-rigor.md §3.2.3
  - documentation-rigor.md §3.2.5
  - ux-best-practices.md §3
  - ux-best-practices.md §5
related_specs:
  - /specs/tenant-model.json
  - /specs/products/ontology.json
  - /specs/microservices/workflow.json
  - /specs/microservices/ontology.json
  - /specs/design-system/catalog.json
  - /specs/microservice-manifest-schema.json
purpose: >
  Codify role-based projection as the shared doctrine for showing the same
  human different role projections of the same product estate while preserving
  passkey identity, tenant scoping, Cedar authorization, Ontology projections,
  Workflow template selection, and a unified UX shell vocabulary.
enforcement_status: advisory-until-role-projection-registry-lands
enforced_by:
  - oya-governance-role-projection-registry
  - oya-governance-role-context-indicator
  - oya-governance-role-switch-latency
  - oya-governance-role-shell-a11y
  - oya-governance-role-shell-same-training
  - oya-governance-per-microservice-role-adapter
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Role-based projection + unified UX shell

# ADR-0317: Role-Based Projection + Unified UX Shell Doctrine

## Status

Proposed - 2026-05-20.

This ADR is advisory until the shared role-projection registry, the `oya-shared-role-projection` crate, and the per-µservice role-shell adapter manifests land. After those artifacts land, the enforcement lanes named in frontmatter promote to blocker status for user-facing surfaces.

## Date

2026-05-20.

## §A Context

Oyatie already treats tenant_id and sub_scope_path as universal scoping primitives per ADR-0244. ADR-0311 adds the personal-vs-work tenant boundary: the same human can hold a personal tenant and an employer-owned work tenant while the same passkey identity bridges authentication. What remains unresolved is presentation: the same authenticated human must see a different role projection when acting as nurse, parent, side-business-owner, employee, manager, auditor, support agent, tenant admin, developer, or regulator.

The missing doctrine is not a theme switch. A role projection changes authorization, object visibility, workflow templates, navigation density, command vocabulary availability, device treatment, locale copy, accessibility accommodations, and audit context. It does not change the underlying human identity, tenant scoping model, passkey recovery doctrine, canonical Ontology, or Workflow Engine contract.

Yejin is the forcing example. In the morning she acts as a nurse in a hospital tenant with PHI, urgent-care workflows, break-glass semantics, and shift handoff. At lunch she acts as a parent in a personal/family tenant with child-safety consent, school messages, and minor protections per ADR-0292. At night she acts as a side-business owner with invoices, bookings, payroll, and customer messages. The human is the same; the active role projection is not.

The UX-floor in documentation-rigor.md §3.2.3 says defense-in-depth cannot tax legitimate default paths. The critical-path doctrine in §3.2.5 says safety, security, and policy must all hold. Role projection is the positive form of that rule: the user sees the right context with the right affordances before an error, breach, or confusing cross-role action occurs.

### §A.1 Role-switching as a first-class primitive

1. Role switching is an explicit state transition, not a side effect of route changes.
2. The transition changes role_projection_id, permit_set_refs, Ontology projection refs, Workflow template library refs, UX shell refs, locale profile, accessibility profile, and device profile.
3. The transition preserves passkey_subject_id, principal lineage, recovery posture, audit-chain subject continuity, and tenant-boundary invariants.
4. The transition is visible, measured, audited, and reversible unless a policy expiry or revocation event invalidates the previous role.
5. The transition never leaks cached data from the previous role into the next role.
6. The transition never hides which role is active.
7. The transition never makes a user re-learn navigation, keyboard, voice, or gesture primitives unless a documented accommodation requires a different modality.

### §A.2 Yejin multi-context scenario

#### §A.2.1 Nurse shift handoff

Yejin opens the hospital role projection. The UX shell foregrounds patient queue, shift tasks, medication checks, and urgent escalations. Cedar permits are healthcare-role scoped. Ontology projection exposes patient, encounter, care-team, medication, and audit objects with PHI badges. Workflow templates prioritize triage, code-blue, discharge, and handoff templates.

#### §A.2.2 Parent school consent

Yejin switches to the parent role projection. The shell foregrounds family calendar, school forms, child-safety messages, and consent workflows. Cedar permits are family-tenant scoped. Ontology projection exposes child, guardian, consent, school-event, and safety-report objects. ADR-0292 minor protections bind every action.

#### §A.2.3 Side business owner

Yejin switches to the SMB owner role projection. The shell foregrounds invoices, bookings, payroll, payments, customers, and local compliance tasks. Cedar permits are business-tenant scoped. Ontology projection exposes customer, invoice, payroll-run, appointment, product, and tax objects. Workflow templates prioritize quote-to-cash and payroll close.

### §A.3 Hyperscaler precedent summary

| Vendor pattern | Precedent used here | Doctrine consequence |
|---|---|---|
| Apple account-driven enrollment | Managed Apple Account and personal Apple Account can coexist with complete separation of work and personal data. | Role projection must make context explicit without fragmenting core identity, authorization, or object models. |
| Apple Human Interface Guidelines | Primary Apple design-system reference for Apple-family interaction and visual patterns. | Role projection must make context explicit without fragmenting core identity, authorization, or object models. |
| Microsoft account boundary | Microsoft distinguishes personal accounts from work or school accounts and states that user-account information does not synchronize between them. | Role projection must make context explicit without fragmenting core identity, authorization, or object models. |
| Microsoft Fluent Nav | Fluent navigation should be coherent, task- or feature-oriented by context, brief, consistently ordered, and accessible beyond hover. | Role projection must make context explicit without fragmenting core identity, authorization, or object models. |
| Google Workspace account boundary | Google Workspace accounts can behave differently from personal accounts and can be administered by an organization. | Role projection must make context explicit without fragmenting core identity, authorization, or object models. |
| Google Material navigation | Material navigation guidance tailors navigation to important content, tasks, and in-context movement. | Role projection must make context explicit without fragmenting core identity, authorization, or object models. |
| Salesforce Lightning role home page assignment | Salesforce permits assigning Lightning home pages to apps and app/profile combinations. | Role projection must make context explicit without fragmenting core identity, authorization, or object models. |
| ServiceNow Service Portal vs Workspace | ServiceNow separates requester self-service portals from agent or fulfiller workspaces. | Role projection must make context explicit without fragmenting core identity, authorization, or object models. |

### §A.4 Standards and ADR cross-reference map

| Authority | Binding effect on ADR-0317 |
|---|---|
| documentation-rigor.md §3.2.3 | UX floor requires no default-path friction, preserved accessibility, and graceful recovery. |
| documentation-rigor.md §3.2.5 | Critical-path edge cases must name safety, security, and policy handling. |
| ux-best-practices.md §3 | WCAG 2.2 AA is the minimum across shipped surfaces. |
| ux-best-practices.md §5 | Locale, RTL, CJK, date/time, and number formatting require canonical i18n paths. |
| ADR-0244 | tenant_id and sub_scope_path remain universal scoping primitives. |
| ADR-0292 | minor-facing role shells inherit minor protection and accommodation duties. |
| ADR-0299 | same human can bridge contexts through passkey identity and recovery doctrine. |
| ADR-0311 | work and personal tenant boundaries remain non-negotiable. |
| ADR-0316 | adjacent doctrine reference reserved for the preceding Wave-3-G record; missing local file is explicitly called out. |

## §B Decision

Oyatie adopts a role-projection model with same-primitives invariants. A role projection is the typed binding between one authenticated human principal and one active product role inside one tenant/sub-scope context. It selects a Cedar permit set, an Ontology projection, a Workflow template library, a UX shell, device/locale/accessibility profiles, and switch-state telemetry. It never forks the underlying identity, tenant primitive, policy engine, Ontology model, Workflow Engine, or design-token vocabulary.

### §B.1 Same-primitives invariants

| Invariant | Required interpretation | Forbidden interpretation |
|---|---|---|
| Identity continuity | Same passkey_subject_id follows the human across roles per ADR-0299. | Minting separate humans for nurse, parent, and owner contexts. |
| Tenant scoping | Every role projection is tenant_id + sub_scope_path scoped per ADR-0244. | Role logic reads across tenants because the human is the same. |
| Policy unity | Cedar remains the universal gate; role projection selects permit sets. | Client-side nav hiding substitutes for authorization. |
| Ontology unity | Canonical Ontology stays singular; role projection selects allowed views. | Each role invents a private object model. |
| Workflow unity | Workflow Engine stays singular; role projection selects templates. | Each role ships a forked workflow engine. |
| UX vocabulary unity | Nav, sidebar, keyboard, voice, and gestures share primitive names. | Each role teaches unrelated interaction grammar. |
| Accessibility floor | WCAG 2.2 AA applies throughout; accommodations compose by role. | Role-specific UI bypasses accessibility because it is specialized. |
| Switch clarity | Active role is visible and audited. | A route or color hint is the only context signal. |

### §B.2 Role-projection registry shape

```json
{
  "role_projection_id": "rp_<tenant>_<principal>_<role>",
  "tenant_id": "tenant_<id>",
  "sub_scope_path": "org.unit.team-or-family-or-business",
  "principal_id": "principal_<id>",
  "passkey_subject_id": "webauthn_subject_<id>",
  "role_code": "ROLE_CLINICIAN_NURSE",
  "role_display_name": "Nurse",
  "cedar_entity": { "type": "RoleProjection", "id": "rp_<id>" },
  "permit_set_refs": ["cedar/permit-sets/<role>.cedar"],
  "deny_overlay_refs": ["cedar/deny-overlays/<role>.cedar"],
  "ontology_projection_refs": ["ontology/projections/<role>.json"],
  "workflow_template_library_refs": ["workflow/templates/<role>.json"],
  "ux_shell_ref": "ux-shells/<role>.json",
  "role_context_indicator": {
    "required": true,
    "tenant_label": true,
    "role_label": true,
    "data_class_badges": true,
    "switch_affordance": "visible_when_multiple_roles"
  },
  "switching_slo": { "p95_ms": 500, "audit_event": "RoleProjectionSwitched" },
  "device_profiles": ["laptop", "desktop", "tablet", "phone", "wearable", "ar-overlay", "voice-only", "screen-reader-only"],
  "locale_profile_refs": ["i18n/role/<role>/<locale>.json"],
  "accessibility_profile_refs": ["a11y/role/<role>.json"],
  "training_transfer_version": "primitive-vocabulary-v1",
  "version": "1.0.0"
}
```

### §B.3 Role archetype registry

#### §B.3.1 ROLE_CLINICIAN_NURSE

- Display role: Yejin nurse role.
- Tenant context: hospital tenant.
- Primary workflow families: urgent care, shift handoff, EHR break-glass.
- Ontology projection focus: PHI plus clinical workflow objects.
- Compliance and safety obligations: life-safety and healthcare pack obligations.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.2 ROLE_PARENT_GUARDIAN

- Display role: Yejin parent role.
- Tenant context: personal/family tenant.
- Primary workflow families: school forms, child safety, consent, household calendar.
- Ontology projection focus: minor PII plus consent records.
- Compliance and safety obligations: ADR-0292 minor protection obligations.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.3 ROLE_SIDE_BUSINESS_OWNER

- Display role: Yejin side-business-owner role.
- Tenant context: small-business tenant.
- Primary workflow families: invoices, payroll, bookings, customer messages.
- Ontology projection focus: commercial records plus customer PII.
- Compliance and safety obligations: tax, payments, and SMB workflow obligations.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.4 ROLE_EMPLOYEE

- Display role: employee role.
- Tenant context: employer tenant.
- Primary workflow families: mail, messenger, drive, calendar, HR self-service.
- Ontology projection focus: work-owned collaboration data.
- Compliance and safety obligations: ADR-0311 work-vs-personal boundary.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.5 ROLE_MANAGER

- Display role: manager role.
- Tenant context: employer tenant sub-scope.
- Primary workflow families: approvals, reviews, budgets, staffing.
- Ontology projection focus: team objects and approval evidence.
- Compliance and safety obligations: delegated management but not personal inspection.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.6 ROLE_INTERNAL_AUDITOR

- Display role: internal auditor role.
- Tenant context: enterprise tenant audit sub-scope.
- Primary workflow families: SOX, SOC2, evidence pulls, sampling.
- Ontology projection focus: audit evidence and control tests.
- Compliance and safety obligations: scope-bounded read access.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.7 ROLE_3PAO_ASSESSOR

- Display role: third-party assessor role.
- Tenant context: regulated assessment tenant link.
- Primary workflow families: FedRAMP or SOC2 evidence review.
- Ontology projection focus: assessment packages and attestations.
- Compliance and safety obligations: temporary scoped external permit set.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.8 ROLE_TENANT_ADMIN

- Display role: tenant administrator role.
- Tenant context: tenant root or sub-scope.
- Primary workflow families: policy, billing, users, packs, residency.
- Ontology projection focus: configuration and identity metadata.
- Compliance and safety obligations: strong step-up and audit obligations.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.9 ROLE_DEVELOPER

- Display role: developer role.
- Tenant context: developer workspace tenant.
- Primary workflow families: SDK, API keys, app registration, logs.
- Ontology projection focus: developer artifacts and sandbox data.
- Compliance and safety obligations: separate from production tenant authority.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.10 ROLE_SUPPORT_AGENT

- Display role: support agent role.
- Tenant context: provider support tenant.
- Primary workflow families: case triage, safe diagnostics, tenant support.
- Ontology projection focus: support cases and redacted diagnostics.
- Compliance and safety obligations: just-in-time break-glass constraints.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.11 ROLE_REGULATOR

- Display role: regulator role.
- Tenant context: jurisdiction authority scope.
- Primary workflow families: lawful audit, investigation, report review.
- Ontology projection focus: sealed evidence views.
- Compliance and safety obligations: warrant or statute-scoped access.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

#### §B.3.12 ROLE_CONTRACTOR

- Display role: contractor role.
- Tenant context: limited project sub-scope.
- Primary workflow families: project tasks and deliverables.
- Ontology projection focus: least-privilege project objects.
- Compliance and safety obligations: time-bound access and offboarding revoke.
- UX shell invariant: same navigation, command-palette, keyboard, voice, and gesture primitive vocabulary as every other role.
- Cedar invariant: role-specific permits grant only what this role can do in this tenant/sub-scope.
- Switch invariant: role context indicator must be visible before any mutation, export, payment, access grant, or critical-path action.

## §C Consequences

### §C.1 Six engineering-rigor dimensions

| Dimension | Consequence | Acceptance signal |
|---|---|---|
| Maintainability | Role projection localizes variation in registries, not copied client code or forked µservice behavior. | One projection registry row explains each role-specific difference. |
| Observability | Every projection resolution, switch, permit decision, redaction, and shell load emits audit and trace context. | Dashboards can filter by role_projection_id without exposing protected payloads. |
| Scalability | Role registries can be cached per tenant and invalidated by version while services keep flat ownership. | Projection resolution scales independently from product data paths. |
| Performance | Role switching has a 500 ms p95 visible budget and default navigation load preserves critical-path latency. | Switch spans show p95 <= 500 ms and no default-path security friction. |
| Optimization | Cost, cache, search, and command-palette indexes can be tuned per role while sharing core primitives. | Finops sees cost dimensions for projection resolver and per-role shell adapters. |
| Code quality | The shared crate owns type contracts; adapters stay thin and per-µservice; shell primitive names remain canonical. | Adapters declare registries and tests instead of inventing local role logic. |

### §C.2 Maintainability

Doctrine: Role projection localizes variation in registries, not copied client code or forked µservice behavior.
Acceptance signal: One projection registry row explains each role-specific difference.
Required evidence: registry row, adapter manifest, validator output, audit event sample, UX-shell snapshot or accessibility proof, and regression test where applicable.
Risk if omitted: the same human may act in the wrong context, see the wrong object projection, trigger the wrong workflow, or trust a misleading role shell.
Mitigation: default-deny Cedar permits, explicit role-context indicator, cache partitioning by role_projection_id, and shared primitive vocabulary tests.

### §C.3 Observability

Doctrine: Every projection resolution, switch, permit decision, redaction, and shell load emits audit and trace context.
Acceptance signal: Dashboards can filter by role_projection_id without exposing protected payloads.
Required evidence: registry row, adapter manifest, validator output, audit event sample, UX-shell snapshot or accessibility proof, and regression test where applicable.
Risk if omitted: the same human may act in the wrong context, see the wrong object projection, trigger the wrong workflow, or trust a misleading role shell.
Mitigation: default-deny Cedar permits, explicit role-context indicator, cache partitioning by role_projection_id, and shared primitive vocabulary tests.

### §C.4 Scalability

Doctrine: Role registries can be cached per tenant and invalidated by version while services keep flat ownership.
Acceptance signal: Projection resolution scales independently from product data paths.
Required evidence: registry row, adapter manifest, validator output, audit event sample, UX-shell snapshot or accessibility proof, and regression test where applicable.
Risk if omitted: the same human may act in the wrong context, see the wrong object projection, trigger the wrong workflow, or trust a misleading role shell.
Mitigation: default-deny Cedar permits, explicit role-context indicator, cache partitioning by role_projection_id, and shared primitive vocabulary tests.

### §C.5 Performance

Doctrine: Role switching has a 500 ms p95 visible budget and default navigation load preserves critical-path latency.
Acceptance signal: Switch spans show p95 <= 500 ms and no default-path security friction.
Required evidence: registry row, adapter manifest, validator output, audit event sample, UX-shell snapshot or accessibility proof, and regression test where applicable.
Risk if omitted: the same human may act in the wrong context, see the wrong object projection, trigger the wrong workflow, or trust a misleading role shell.
Mitigation: default-deny Cedar permits, explicit role-context indicator, cache partitioning by role_projection_id, and shared primitive vocabulary tests.

### §C.6 Optimization

Doctrine: Cost, cache, search, and command-palette indexes can be tuned per role while sharing core primitives.
Acceptance signal: Finops sees cost dimensions for projection resolver and per-role shell adapters.
Required evidence: registry row, adapter manifest, validator output, audit event sample, UX-shell snapshot or accessibility proof, and regression test where applicable.
Risk if omitted: the same human may act in the wrong context, see the wrong object projection, trigger the wrong workflow, or trust a misleading role shell.
Mitigation: default-deny Cedar permits, explicit role-context indicator, cache partitioning by role_projection_id, and shared primitive vocabulary tests.

### §C.7 Code quality

Doctrine: The shared crate owns type contracts; adapters stay thin and per-µservice; shell primitive names remain canonical.
Acceptance signal: Adapters declare registries and tests instead of inventing local role logic.
Required evidence: registry row, adapter manifest, validator output, audit event sample, UX-shell snapshot or accessibility proof, and regression test where applicable.
Risk if omitted: the same human may act in the wrong context, see the wrong object projection, trigger the wrong workflow, or trust a misleading role shell.
Mitigation: default-deny Cedar permits, explicit role-context indicator, cache partitioning by role_projection_id, and shared primitive vocabulary tests.

## §D Detailed Mechanics

The mechanics below are normative. Each primitive lists registry fields, at least two hyperscaler precedents, acceptance signals, and failure modes.

### §D-1: Role-projection Cedar entity type + permit set

Decision: Every active role context is a Cedar-addressable RoleProjection entity that binds tenant_id, sub_scope_path, principal_id, passkey_subject_id, role_code, and permit_set_refs.

#### Registry fields

- `role_projection_id` is required for this primitive.
- `tenant_id` is required for this primitive.
- `sub_scope_path` is required for this primitive.
- `principal_id` is required for this primitive.
- `passkey_subject_id` is required for this primitive.
- `role_code` is required for this primitive.
- `permit_set_refs` is required for this primitive.
- `deny_overlay_refs` is required for this primitive.
- `expires_at` is required for this primitive.
- `audit_subject_ref` is required for this primitive.

#### Hyperscaler precedents

- Precedent 1: Apple separates Managed Apple Account work data from personal Apple Account data on the same device.
- Precedent 2: Microsoft distinguishes personal Microsoft accounts from work or school accounts and does not synchronize user-account information between them.
- Precedent 3: Google Workspace accounts can be administrator-managed and can expose different product behavior from personal accounts.

#### Acceptance signals

- Acceptance 1: Cedar schema declares RoleProjection as an entity type.
- Acceptance 2: Every permit references role_projection_id or an explicit cross-role grant.
- Acceptance 3: Default-deny is applied when role context is missing, stale, or ambiguous.

#### Failure modes and controls

- Ambiguous role: Refuse mutation; show role selector; audit `RoleProjectionAmbiguous`.
- Stale role registry: Reject cached projection; refetch signed registry version; audit `RoleProjectionCacheInvalidated`.
- Cross-role data bleed: Clear role-local caches; partition search indexes; emit severity-high audit event.
- Missing accessibility path: Block shell publication; attach WCAG 2.2 AA finding; require remediation before rollout.
- Training-transfer exception: Require waiver, owner, expiry, and user-facing help affordance.

#### Role application examples

- ROLE_CLINICIAN_NURSE: this primitive projects PHI plus clinical workflow objects for Yejin nurse role in hospital tenant; workflow families include urgent care, shift handoff, EHR break-glass; obligations include life-safety and healthcare pack obligations.
- ROLE_PARENT_GUARDIAN: this primitive projects minor PII plus consent records for Yejin parent role in personal/family tenant; workflow families include school forms, child safety, consent, household calendar; obligations include ADR-0292 minor protection obligations.
- ROLE_SIDE_BUSINESS_OWNER: this primitive projects commercial records plus customer PII for Yejin side-business-owner role in small-business tenant; workflow families include invoices, payroll, bookings, customer messages; obligations include tax, payments, and SMB workflow obligations.
- ROLE_EMPLOYEE: this primitive projects work-owned collaboration data for employee role in employer tenant; workflow families include mail, messenger, drive, calendar, HR self-service; obligations include ADR-0311 work-vs-personal boundary.
- ROLE_MANAGER: this primitive projects team objects and approval evidence for manager role in employer tenant sub-scope; workflow families include approvals, reviews, budgets, staffing; obligations include delegated management but not personal inspection.
- ROLE_INTERNAL_AUDITOR: this primitive projects audit evidence and control tests for internal auditor role in enterprise tenant audit sub-scope; workflow families include SOX, SOC2, evidence pulls, sampling; obligations include scope-bounded read access.

### §D-2: Per-role Ontology projection

Decision: Every role context exposes a bounded Ontology view: object types, relation types, action affordances, and redaction rules are projected per role without duplicating the underlying object model.

#### Registry fields

- `ontology_projection_id` is required for this primitive.
- `object_type_allowlist` is required for this primitive.
- `link_type_allowlist` is required for this primitive.
- `field_redaction_policy` is required for this primitive.
- `action_affordance_refs` is required for this primitive.
- `query_template_refs` is required for this primitive.
- `explainability_profile` is required for this primitive.

#### Hyperscaler precedents

- Precedent 1: Salesforce Lightning lets admins assign role-appropriate home pages through app/profile combinations.
- Precedent 2: ServiceNow distinguishes requester portals from agent workspaces, with different audiences, purposes, navigation, analytics, and configuration tools.
- Precedent 3: Material navigation guidance supports task-focused and in-context navigation for specific data sets.

#### Acceptance signals

- Acceptance 1: Ontology projection registry lists object types and field-level redaction.
- Acceptance 2: Projection does not mutate canonical Ontology definitions.
- Acceptance 3: Every redacted field has a reason code and audit evidence.

#### Failure modes and controls

- Ambiguous role: Refuse mutation; show role selector; audit `RoleProjectionAmbiguous`.
- Stale role registry: Reject cached projection; refetch signed registry version; audit `RoleProjectionCacheInvalidated`.
- Cross-role data bleed: Clear role-local caches; partition search indexes; emit severity-high audit event.
- Missing accessibility path: Block shell publication; attach WCAG 2.2 AA finding; require remediation before rollout.
- Training-transfer exception: Require waiver, owner, expiry, and user-facing help affordance.

#### Role application examples

- ROLE_CLINICIAN_NURSE: this primitive projects PHI plus clinical workflow objects for Yejin nurse role in hospital tenant; workflow families include urgent care, shift handoff, EHR break-glass; obligations include life-safety and healthcare pack obligations.
- ROLE_PARENT_GUARDIAN: this primitive projects minor PII plus consent records for Yejin parent role in personal/family tenant; workflow families include school forms, child safety, consent, household calendar; obligations include ADR-0292 minor protection obligations.
- ROLE_SIDE_BUSINESS_OWNER: this primitive projects commercial records plus customer PII for Yejin side-business-owner role in small-business tenant; workflow families include invoices, payroll, bookings, customer messages; obligations include tax, payments, and SMB workflow obligations.
- ROLE_EMPLOYEE: this primitive projects work-owned collaboration data for employee role in employer tenant; workflow families include mail, messenger, drive, calendar, HR self-service; obligations include ADR-0311 work-vs-personal boundary.
- ROLE_MANAGER: this primitive projects team objects and approval evidence for manager role in employer tenant sub-scope; workflow families include approvals, reviews, budgets, staffing; obligations include delegated management but not personal inspection.
- ROLE_INTERNAL_AUDITOR: this primitive projects audit evidence and control tests for internal auditor role in enterprise tenant audit sub-scope; workflow families include SOX, SOC2, evidence pulls, sampling; obligations include scope-bounded read access.

### §D-3: Per-role workflow template library

Decision: Workflow templates are selected through role_projection_id so the same Workflow Engine can show nurse handoff, parent consent, SMB invoice, auditor sampling, and developer release templates without forked engines.

#### Registry fields

- `workflow_template_library_id` is required for this primitive.
- `role_code` is required for this primitive.
- `template_refs` is required for this primitive.
- `trigger_refs` is required for this primitive.
- `approval_policy_refs` is required for this primitive.
- `escalation_policy_refs` is required for this primitive.
- `critical_path_rows` is required for this primitive.
- `pack_overlay_refs` is required for this primitive.

#### Hyperscaler precedents

- Precedent 1: ServiceNow Workspaces are task-centric locations for fulfillers to triage, collaborate, resolve records, and view analytics.
- Precedent 2: Salesforce Lightning custom home pages can be assigned to apps and profiles so users start with role-relevant workflow entry points.
- Precedent 3: Microsoft Fluent recommends navigation be task-oriented or feature-oriented depending on product context.

#### Acceptance signals

- Acceptance 1: Template registry is role-indexed and tenant-scoped.
- Acceptance 2: Critical-path workflow templates declare documentation-rigor.md row bindings.
- Acceptance 3: Template drift from the shared Workflow contract is rejected.

#### Failure modes and controls

- Ambiguous role: Refuse mutation; show role selector; audit `RoleProjectionAmbiguous`.
- Stale role registry: Reject cached projection; refetch signed registry version; audit `RoleProjectionCacheInvalidated`.
- Cross-role data bleed: Clear role-local caches; partition search indexes; emit severity-high audit event.
- Missing accessibility path: Block shell publication; attach WCAG 2.2 AA finding; require remediation before rollout.
- Training-transfer exception: Require waiver, owner, expiry, and user-facing help affordance.

#### Role application examples

- ROLE_CLINICIAN_NURSE: this primitive projects PHI plus clinical workflow objects for Yejin nurse role in hospital tenant; workflow families include urgent care, shift handoff, EHR break-glass; obligations include life-safety and healthcare pack obligations.
- ROLE_PARENT_GUARDIAN: this primitive projects minor PII plus consent records for Yejin parent role in personal/family tenant; workflow families include school forms, child safety, consent, household calendar; obligations include ADR-0292 minor protection obligations.
- ROLE_SIDE_BUSINESS_OWNER: this primitive projects commercial records plus customer PII for Yejin side-business-owner role in small-business tenant; workflow families include invoices, payroll, bookings, customer messages; obligations include tax, payments, and SMB workflow obligations.
- ROLE_EMPLOYEE: this primitive projects work-owned collaboration data for employee role in employer tenant; workflow families include mail, messenger, drive, calendar, HR self-service; obligations include ADR-0311 work-vs-personal boundary.
- ROLE_MANAGER: this primitive projects team objects and approval evidence for manager role in employer tenant sub-scope; workflow families include approvals, reviews, budgets, staffing; obligations include delegated management but not personal inspection.
- ROLE_INTERNAL_AUDITOR: this primitive projects audit evidence and control tests for internal auditor role in enterprise tenant audit sub-scope; workflow families include SOX, SOC2, evidence pulls, sampling; obligations include scope-bounded read access.

### §D-4: Per-role UX shell primitives

Decision: NavBar, Sidebar, KeyboardShortcut, VoiceCommand, and GestureSet are role-projected from the same primitive vocabulary, not hand-built per product surface.

#### Registry fields

- `ux_shell_id` is required for this primitive.
- `navbar_ref` is required for this primitive.
- `sidebar_ref` is required for this primitive.
- `keyboard_shortcut_map_ref` is required for this primitive.
- `voice_command_set_ref` is required for this primitive.
- `gesture_set_ref` is required for this primitive.
- `command_palette_ref` is required for this primitive.
- `density_tier` is required for this primitive.
- `motion_profile` is required for this primitive.

#### Hyperscaler precedents

- Precedent 1: Apple Human Interface Guidelines provide common system interaction expectations across Apple devices.
- Precedent 2: Microsoft Fluent Nav requires coherent, short, consistently ordered navigation and accessible secondary actions.
- Precedent 3: Google Material navigation focuses attention on important content and tasks while supporting in-context movement.

#### Acceptance signals

- Acceptance 1: Every role shell declares the five primitive refs.
- Acceptance 2: Every visible command has keyboard and screen-reader equivalents.
- Acceptance 3: Tenant branding may skin but not redefine the primitive vocabulary.

#### Failure modes and controls

- Ambiguous role: Refuse mutation; show role selector; audit `RoleProjectionAmbiguous`.
- Stale role registry: Reject cached projection; refetch signed registry version; audit `RoleProjectionCacheInvalidated`.
- Cross-role data bleed: Clear role-local caches; partition search indexes; emit severity-high audit event.
- Missing accessibility path: Block shell publication; attach WCAG 2.2 AA finding; require remediation before rollout.
- Training-transfer exception: Require waiver, owner, expiry, and user-facing help affordance.

#### Role application examples

- ROLE_CLINICIAN_NURSE: this primitive projects PHI plus clinical workflow objects for Yejin nurse role in hospital tenant; workflow families include urgent care, shift handoff, EHR break-glass; obligations include life-safety and healthcare pack obligations.
- ROLE_PARENT_GUARDIAN: this primitive projects minor PII plus consent records for Yejin parent role in personal/family tenant; workflow families include school forms, child safety, consent, household calendar; obligations include ADR-0292 minor protection obligations.
- ROLE_SIDE_BUSINESS_OWNER: this primitive projects commercial records plus customer PII for Yejin side-business-owner role in small-business tenant; workflow families include invoices, payroll, bookings, customer messages; obligations include tax, payments, and SMB workflow obligations.
- ROLE_EMPLOYEE: this primitive projects work-owned collaboration data for employee role in employer tenant; workflow families include mail, messenger, drive, calendar, HR self-service; obligations include ADR-0311 work-vs-personal boundary.
- ROLE_MANAGER: this primitive projects team objects and approval evidence for manager role in employer tenant sub-scope; workflow families include approvals, reviews, budgets, staffing; obligations include delegated management but not personal inspection.
- ROLE_INTERNAL_AUDITOR: this primitive projects audit evidence and control tests for internal auditor role in enterprise tenant audit sub-scope; workflow families include SOX, SOC2, evidence pulls, sampling; obligations include scope-bounded read access.

### §D-5: Role-context indicator

Decision: Every user-facing surface renders an unambiguous current role-context indicator showing role label, tenant label, sub-scope label, data-class badge when applicable, and switch affordance state.

#### Registry fields

- `indicator_id` is required for this primitive.
- `role_label` is required for this primitive.
- `tenant_display_name` is required for this primitive.
- `sub_scope_display_name` is required for this primitive.
- `data_class_badge_policy` is required for this primitive.
- `switch_affordance_state` is required for this primitive.
- `last_switched_at` is required for this primitive.
- `audit_context_ref` is required for this primitive.

#### Hyperscaler precedents

- Precedent 1: Apple account-driven enrollment shows the Managed Apple Account prominently in Settings after enrollment.
- Precedent 2: Microsoft sign-in flows route users to the correct service based on account type.
- Precedent 3: Google Workspace documentation warns that organization accounts may work differently from personal accounts.

#### Acceptance signals

- Acceptance 1: Indicator is present on every default and critical path.
- Acceptance 2: Indicator survives narrow viewports and assistive technologies.
- Acceptance 3: No destructive action is allowed when the role indicator is absent.

#### Failure modes and controls

- Ambiguous role: Refuse mutation; show role selector; audit `RoleProjectionAmbiguous`.
- Stale role registry: Reject cached projection; refetch signed registry version; audit `RoleProjectionCacheInvalidated`.
- Cross-role data bleed: Clear role-local caches; partition search indexes; emit severity-high audit event.
- Missing accessibility path: Block shell publication; attach WCAG 2.2 AA finding; require remediation before rollout.
- Training-transfer exception: Require waiver, owner, expiry, and user-facing help affordance.

#### Role application examples

- ROLE_CLINICIAN_NURSE: this primitive projects PHI plus clinical workflow objects for Yejin nurse role in hospital tenant; workflow families include urgent care, shift handoff, EHR break-glass; obligations include life-safety and healthcare pack obligations.
- ROLE_PARENT_GUARDIAN: this primitive projects minor PII plus consent records for Yejin parent role in personal/family tenant; workflow families include school forms, child safety, consent, household calendar; obligations include ADR-0292 minor protection obligations.
- ROLE_SIDE_BUSINESS_OWNER: this primitive projects commercial records plus customer PII for Yejin side-business-owner role in small-business tenant; workflow families include invoices, payroll, bookings, customer messages; obligations include tax, payments, and SMB workflow obligations.
- ROLE_EMPLOYEE: this primitive projects work-owned collaboration data for employee role in employer tenant; workflow families include mail, messenger, drive, calendar, HR self-service; obligations include ADR-0311 work-vs-personal boundary.
- ROLE_MANAGER: this primitive projects team objects and approval evidence for manager role in employer tenant sub-scope; workflow families include approvals, reviews, budgets, staffing; obligations include delegated management but not personal inspection.
- ROLE_INTERNAL_AUDITOR: this primitive projects audit evidence and control tests for internal auditor role in enterprise tenant audit sub-scope; workflow families include SOX, SOC2, evidence pulls, sampling; obligations include scope-bounded read access.

### §D-6: Cross-role-switch friction floor

Decision: Role switching is a first-class primitive with p95 visible switch latency at or below 500 ms, clear active-role visualization, no data bleed, and no hidden re-auth unless a step-up policy explicitly requires it.

#### Registry fields

- `switch_slo_profile_id` is required for this primitive.
- `p95_latency_ms` is required for this primitive.
- `visual_transition_policy` is required for this primitive.
- `cache_partition_policy` is required for this primitive.
- `data_isolation_policy_ref` is required for this primitive.
- `step_up_policy_ref` is required for this primitive.
- `rollback_state_ref` is required for this primitive.

#### Hyperscaler precedents

- Precedent 1: Microsoft separates personal and work/school account resources so the correct account context matters at sign-in.
- Precedent 2: Apple supports coexisting managed and personal accounts while preserving data separation.
- Precedent 3: Google Workspace account behavior can differ under administrator control, making explicit switching semantics necessary.

#### Acceptance signals

- Acceptance 1: p95 role switch latency <= 500 ms on supported profiles.
- Acceptance 2: Cache keys include tenant_id and role_projection_id.
- Acceptance 3: Switch trace includes before/after role ids and no protected payload fields.

#### Failure modes and controls

- Ambiguous role: Refuse mutation; show role selector; audit `RoleProjectionAmbiguous`.
- Stale role registry: Reject cached projection; refetch signed registry version; audit `RoleProjectionCacheInvalidated`.
- Cross-role data bleed: Clear role-local caches; partition search indexes; emit severity-high audit event.
- Missing accessibility path: Block shell publication; attach WCAG 2.2 AA finding; require remediation before rollout.
- Training-transfer exception: Require waiver, owner, expiry, and user-facing help affordance.

#### Role application examples

- ROLE_CLINICIAN_NURSE: this primitive projects PHI plus clinical workflow objects for Yejin nurse role in hospital tenant; workflow families include urgent care, shift handoff, EHR break-glass; obligations include life-safety and healthcare pack obligations.
- ROLE_PARENT_GUARDIAN: this primitive projects minor PII plus consent records for Yejin parent role in personal/family tenant; workflow families include school forms, child safety, consent, household calendar; obligations include ADR-0292 minor protection obligations.
- ROLE_SIDE_BUSINESS_OWNER: this primitive projects commercial records plus customer PII for Yejin side-business-owner role in small-business tenant; workflow families include invoices, payroll, bookings, customer messages; obligations include tax, payments, and SMB workflow obligations.
- ROLE_EMPLOYEE: this primitive projects work-owned collaboration data for employee role in employer tenant; workflow families include mail, messenger, drive, calendar, HR self-service; obligations include ADR-0311 work-vs-personal boundary.
- ROLE_MANAGER: this primitive projects team objects and approval evidence for manager role in employer tenant sub-scope; workflow families include approvals, reviews, budgets, staffing; obligations include delegated management but not personal inspection.
- ROLE_INTERNAL_AUDITOR: this primitive projects audit evidence and control tests for internal auditor role in enterprise tenant audit sub-scope; workflow families include SOX, SOC2, evidence pulls, sampling; obligations include scope-bounded read access.

### §D-7: Per-device profile per role

Decision: Each role projection declares device-specific affordances for laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only contexts.

#### Registry fields

- `device_profile_id` is required for this primitive.
- `supported_device_classes` is required for this primitive.
- `input_modes` is required for this primitive.
- `layout_density` is required for this primitive.
- `offline_policy` is required for this primitive.
- `notification_policy` is required for this primitive.
- `hardware_attestation_policy` is required for this primitive.
- `continuity_policy` is required for this primitive.

#### Hyperscaler precedents

- Precedent 1: Apple account-driven enrollment spans iPhone, iPad, Mac, and Apple Vision Pro device-management flows.
- Precedent 2: Google Workspace mobile controls include managed work-profile behavior on Android devices.
- Precedent 3: Material navigation guidance changes structure according to screen and navigation context.

#### Acceptance signals

- Acceptance 1: Every role declares at least laptop, phone, and screen-reader-only behavior.
- Acceptance 2: Wearable and voice-only profiles expose only safe one-tap or spoken commands.
- Acceptance 3: Device profile changes do not alter Cedar authority.

#### Failure modes and controls

- Ambiguous role: Refuse mutation; show role selector; audit `RoleProjectionAmbiguous`.
- Stale role registry: Reject cached projection; refetch signed registry version; audit `RoleProjectionCacheInvalidated`.
- Cross-role data bleed: Clear role-local caches; partition search indexes; emit severity-high audit event.
- Missing accessibility path: Block shell publication; attach WCAG 2.2 AA finding; require remediation before rollout.
- Training-transfer exception: Require waiver, owner, expiry, and user-facing help affordance.

#### Role application examples

- ROLE_CLINICIAN_NURSE: this primitive projects PHI plus clinical workflow objects for Yejin nurse role in hospital tenant; workflow families include urgent care, shift handoff, EHR break-glass; obligations include life-safety and healthcare pack obligations.
- ROLE_PARENT_GUARDIAN: this primitive projects minor PII plus consent records for Yejin parent role in personal/family tenant; workflow families include school forms, child safety, consent, household calendar; obligations include ADR-0292 minor protection obligations.
- ROLE_SIDE_BUSINESS_OWNER: this primitive projects commercial records plus customer PII for Yejin side-business-owner role in small-business tenant; workflow families include invoices, payroll, bookings, customer messages; obligations include tax, payments, and SMB workflow obligations.
- ROLE_EMPLOYEE: this primitive projects work-owned collaboration data for employee role in employer tenant; workflow families include mail, messenger, drive, calendar, HR self-service; obligations include ADR-0311 work-vs-personal boundary.
- ROLE_MANAGER: this primitive projects team objects and approval evidence for manager role in employer tenant sub-scope; workflow families include approvals, reviews, budgets, staffing; obligations include delegated management but not personal inspection.
- ROLE_INTERNAL_AUDITOR: this primitive projects audit evidence and control tests for internal auditor role in enterprise tenant audit sub-scope; workflow families include SOX, SOC2, evidence pulls, sampling; obligations include scope-bounded read access.

### §D-8: Per-locale UX adaptation per role

Decision: Locale is part of the role shell profile: labels, sort order, date/time, number, currency, RTL, CJK line-height, legal copy, and voice-command grammar adapt per role and locale.

#### Registry fields

- `locale_profile_id` is required for this primitive.
- `locale` is required for this primitive.
- `rtl_policy` is required for this primitive.
- `cjk_rendering_policy` is required for this primitive.
- `voice_grammar_ref` is required for this primitive.
- `legal_copy_ref` is required for this primitive.
- `formatting_profile_ref` is required for this primitive.
- `fallback_chain` is required for this primitive.

#### Hyperscaler precedents

- Precedent 1: Google Workspace organizational accounts can expose different product settings, requiring locale behavior to respect organizational context.
- Precedent 2: Salesforce Lightning pages and apps are admin-configurable and commonly localized for enterprise roles.
- Precedent 3: Apple design guidance treats device and locale conventions as part of system-native interaction quality.

#### Acceptance signals

- Acceptance 1: Every role shell declares fallback locale chain.
- Acceptance 2: RTL icons flip only when directionality semantics require it.
- Acceptance 3: Legal and consent language follows pack overlays without role-specific hardcoding.

#### Failure modes and controls

- Ambiguous role: Refuse mutation; show role selector; audit `RoleProjectionAmbiguous`.
- Stale role registry: Reject cached projection; refetch signed registry version; audit `RoleProjectionCacheInvalidated`.
- Cross-role data bleed: Clear role-local caches; partition search indexes; emit severity-high audit event.
- Missing accessibility path: Block shell publication; attach WCAG 2.2 AA finding; require remediation before rollout.
- Training-transfer exception: Require waiver, owner, expiry, and user-facing help affordance.

#### Role application examples

- ROLE_CLINICIAN_NURSE: this primitive projects PHI plus clinical workflow objects for Yejin nurse role in hospital tenant; workflow families include urgent care, shift handoff, EHR break-glass; obligations include life-safety and healthcare pack obligations.
- ROLE_PARENT_GUARDIAN: this primitive projects minor PII plus consent records for Yejin parent role in personal/family tenant; workflow families include school forms, child safety, consent, household calendar; obligations include ADR-0292 minor protection obligations.
- ROLE_SIDE_BUSINESS_OWNER: this primitive projects commercial records plus customer PII for Yejin side-business-owner role in small-business tenant; workflow families include invoices, payroll, bookings, customer messages; obligations include tax, payments, and SMB workflow obligations.
- ROLE_EMPLOYEE: this primitive projects work-owned collaboration data for employee role in employer tenant; workflow families include mail, messenger, drive, calendar, HR self-service; obligations include ADR-0311 work-vs-personal boundary.
- ROLE_MANAGER: this primitive projects team objects and approval evidence for manager role in employer tenant sub-scope; workflow families include approvals, reviews, budgets, staffing; obligations include delegated management but not personal inspection.
- ROLE_INTERNAL_AUDITOR: this primitive projects audit evidence and control tests for internal auditor role in enterprise tenant audit sub-scope; workflow families include SOX, SOC2, evidence pulls, sampling; obligations include scope-bounded read access.

### §D-9: Per-accessibility-profile UX adaptation per role

Decision: Role shells must satisfy WCAG 2.2 AA and declare accommodations for minors, low vision, screen-reader-only users, voice-control-only users, switch users, reduced motion, cognitive impairment, and post-trauma modes.

#### Registry fields

- `accessibility_profile_id` is required for this primitive.
- `wcag_level` is required for this primitive.
- `screen_reader_policy` is required for this primitive.
- `keyboard_policy` is required for this primitive.
- `voice_control_policy` is required for this primitive.
- `switch_control_policy` is required for this primitive.
- `motion_policy` is required for this primitive.
- `cognitive_load_policy` is required for this primitive.
- `minor_safety_policy` is required for this primitive.

#### Hyperscaler precedents

- Precedent 1: W3C WCAG 2.2 is the shared accessibility baseline.
- Precedent 2: Microsoft Fluent Nav requires hover-only secondary actions to remain reachable for screen-reader, voice-control, eye-gaze, and switch users.
- Precedent 3: Apple Human Interface Guidelines provide system-native accessibility expectations across Apple devices.

#### Acceptance signals

- Acceptance 1: WCAG 2.2 AA is the minimum; regulated paths may require AAA.
- Acceptance 2: Every gesture has keyboard and assistive-technology alternatives.
- Acceptance 3: Minor profiles inherit ADR-0292 safety constraints without weakening accessibility.

#### Failure modes and controls

- Ambiguous role: Refuse mutation; show role selector; audit `RoleProjectionAmbiguous`.
- Stale role registry: Reject cached projection; refetch signed registry version; audit `RoleProjectionCacheInvalidated`.
- Cross-role data bleed: Clear role-local caches; partition search indexes; emit severity-high audit event.
- Missing accessibility path: Block shell publication; attach WCAG 2.2 AA finding; require remediation before rollout.
- Training-transfer exception: Require waiver, owner, expiry, and user-facing help affordance.

#### Role application examples

- ROLE_CLINICIAN_NURSE: this primitive projects PHI plus clinical workflow objects for Yejin nurse role in hospital tenant; workflow families include urgent care, shift handoff, EHR break-glass; obligations include life-safety and healthcare pack obligations.
- ROLE_PARENT_GUARDIAN: this primitive projects minor PII plus consent records for Yejin parent role in personal/family tenant; workflow families include school forms, child safety, consent, household calendar; obligations include ADR-0292 minor protection obligations.
- ROLE_SIDE_BUSINESS_OWNER: this primitive projects commercial records plus customer PII for Yejin side-business-owner role in small-business tenant; workflow families include invoices, payroll, bookings, customer messages; obligations include tax, payments, and SMB workflow obligations.
- ROLE_EMPLOYEE: this primitive projects work-owned collaboration data for employee role in employer tenant; workflow families include mail, messenger, drive, calendar, HR self-service; obligations include ADR-0311 work-vs-personal boundary.
- ROLE_MANAGER: this primitive projects team objects and approval evidence for manager role in employer tenant sub-scope; workflow families include approvals, reviews, budgets, staffing; obligations include delegated management but not personal inspection.
- ROLE_INTERNAL_AUDITOR: this primitive projects audit evidence and control tests for internal auditor role in enterprise tenant audit sub-scope; workflow families include SOX, SOC2, evidence pulls, sampling; obligations include scope-bounded read access.

### §D-10: Same-training-transfers invariant

Decision: Users learn the role-switching, navigation, command, keyboard, voice, and gesture vocabulary once; roles change content and authority, not the primitive interaction grammar.

#### Registry fields

- `training_transfer_id` is required for this primitive.
- `primitive_vocabulary_version` is required for this primitive.
- `gesture_vocabulary_ref` is required for this primitive.
- `keyboard_vocabulary_ref` is required for this primitive.
- `voice_vocabulary_ref` is required for this primitive.
- `nav_order_policy` is required for this primitive.
- `exception_registry_ref` is required for this primitive.

#### Hyperscaler precedents

- Precedent 1: Microsoft Fluent recommends consistent navigation order across products and surfaces to increase predictability.
- Precedent 2: Google Material uses common navigation patterns such as drawers, tabs, up navigation, and in-context navigation.
- Precedent 3: Salesforce Lightning emphasizes familiar navigation while allowing app/profile customization.

#### Acceptance signals

- Acceptance 1: New role-specific commands must map to existing primitive verbs where possible.
- Acceptance 2: Exceptions require explicit training-transfer waiver.
- Acceptance 3: Command names are stable across roles even when target objects differ.

#### Failure modes and controls

- Ambiguous role: Refuse mutation; show role selector; audit `RoleProjectionAmbiguous`.
- Stale role registry: Reject cached projection; refetch signed registry version; audit `RoleProjectionCacheInvalidated`.
- Cross-role data bleed: Clear role-local caches; partition search indexes; emit severity-high audit event.
- Missing accessibility path: Block shell publication; attach WCAG 2.2 AA finding; require remediation before rollout.
- Training-transfer exception: Require waiver, owner, expiry, and user-facing help affordance.

#### Role application examples

- ROLE_CLINICIAN_NURSE: this primitive projects PHI plus clinical workflow objects for Yejin nurse role in hospital tenant; workflow families include urgent care, shift handoff, EHR break-glass; obligations include life-safety and healthcare pack obligations.
- ROLE_PARENT_GUARDIAN: this primitive projects minor PII plus consent records for Yejin parent role in personal/family tenant; workflow families include school forms, child safety, consent, household calendar; obligations include ADR-0292 minor protection obligations.
- ROLE_SIDE_BUSINESS_OWNER: this primitive projects commercial records plus customer PII for Yejin side-business-owner role in small-business tenant; workflow families include invoices, payroll, bookings, customer messages; obligations include tax, payments, and SMB workflow obligations.
- ROLE_EMPLOYEE: this primitive projects work-owned collaboration data for employee role in employer tenant; workflow families include mail, messenger, drive, calendar, HR self-service; obligations include ADR-0311 work-vs-personal boundary.
- ROLE_MANAGER: this primitive projects team objects and approval evidence for manager role in employer tenant sub-scope; workflow families include approvals, reviews, budgets, staffing; obligations include delegated management but not personal inspection.
- ROLE_INTERNAL_AUDITOR: this primitive projects audit evidence and control tests for internal auditor role in enterprise tenant audit sub-scope; workflow families include SOX, SOC2, evidence pulls, sampling; obligations include scope-bounded read access.

## §E Implementation Footprint

### §E.1 Shared crate

Create `crates/oya-shared-role-projection/` as the shared substrate crate that owns role-projection types, validation, registry parsing, cache keys, telemetry event builders, and adapter traits. The crate does not own product data, does not render UI, and does not grant authorization by itself; it gives every µservice the same typed way to ask for a role projection and to bind local adapters to it.

#### Naming justification: `oya-shared-role-projection`

- `oya` prefix: product namespace used by current flat crates.
- `shared`: the crate is a cross-µservice substrate contract, not a product-owned service.
- `role-projection`: exact concern owned by the crate: typed role context resolution and projection registry validation.
- Rejected name `oya-role-shell`: too UI-specific and would hide Cedar, Ontology, and Workflow obligations.
- Rejected name `oya-identity-role-projection`: too identity-owned; the doctrine spans tenancy, policy, Ontology, Workflow, and UX shell adapters.

### §E.2 Rust contract sketch

```rust
pub struct RoleProjectionId(String);
pub struct RoleProjection {
    pub id: RoleProjectionId,
    pub tenant_id: TenantId,
    pub sub_scope_path: SubScopePath,
    pub principal_id: PrincipalId,
    pub passkey_subject_id: PasskeySubjectId,
    pub role_code: RoleCode,
    pub permit_set_refs: Vec<CedarPermitSetRef>,
    pub ontology_projection_refs: Vec<OntologyProjectionRef>,
    pub workflow_template_library_refs: Vec<WorkflowTemplateLibraryRef>,
    pub ux_shell_ref: UxShellRef,
    pub device_profiles: Vec<DeviceProfileRef>,
    pub locale_profiles: Vec<LocaleProfileRef>,
    pub accessibility_profiles: Vec<AccessibilityProfileRef>,
    pub training_transfer_version: PrimitiveVocabularyVersion,
}
pub trait RoleProjectionAdapter {
    fn declare_registry(&self) -> RoleProjectionRegistryDeclaration;
    fn resolve_projection(&self, request: RoleProjectionRequest) -> Result<RoleProjection, RoleProjectionError>;
    fn emit_projection_event(&self, event: RoleProjectionAuditEvent) -> Result<(), RoleProjectionError>;
}
```

### §E.3 Per-µservice role-shell adapters

Every µservice with a user-facing or workflow-facing surface declares an adapter manifest. The manifest is local to that µservice, but the schema is shared. Adapter code may map local object types and commands, but it cannot invent global primitive names or bypass Cedar.

| Adapter field | Meaning |
|---|---|
| `service_id` | µservice identifier that owns the local projection adapter. |
| `role_codes_supported` | List of role codes supported by this µservice. |
| `cedar_action_namespace` | Action namespace mapped to RoleProjection permits. |
| `ontology_projection_bindings` | Object and link types exposed by role. |
| `workflow_template_bindings` | Workflow templates made available by role. |
| `ux_shell_bindings` | Navigation, sidebar, command, keyboard, voice, and gesture refs. |
| `critical_path_rows` | documentation-rigor.md §3.2.5 rows that this adapter touches. |
| `a11y_evidence_refs` | WCAG 2.2 AA evidence for shell paths. |
| `switch_slo_evidence_refs` | p95 switch latency evidence. |

### §E.4 Per-primitive implementation matrix

| Primitive | Shared crate owns | Adapter owns | Validator owns |
|---|---|---|---|
| D-1 Role-projection Cedar entity type + permit set | typed refs, schema, cache key, event builder | local mappings and supported roles | registry completeness, drift, latency, a11y, no cross-role bleed |
| D-2 Per-role Ontology projection | typed refs, schema, cache key, event builder | local mappings and supported roles | registry completeness, drift, latency, a11y, no cross-role bleed |
| D-3 Per-role workflow template library | typed refs, schema, cache key, event builder | local mappings and supported roles | registry completeness, drift, latency, a11y, no cross-role bleed |
| D-4 Per-role UX shell primitives | typed refs, schema, cache key, event builder | local mappings and supported roles | registry completeness, drift, latency, a11y, no cross-role bleed |
| D-5 Role-context indicator | typed refs, schema, cache key, event builder | local mappings and supported roles | registry completeness, drift, latency, a11y, no cross-role bleed |
| D-6 Cross-role-switch friction floor | typed refs, schema, cache key, event builder | local mappings and supported roles | registry completeness, drift, latency, a11y, no cross-role bleed |
| D-7 Per-device profile per role | typed refs, schema, cache key, event builder | local mappings and supported roles | registry completeness, drift, latency, a11y, no cross-role bleed |
| D-8 Per-locale UX adaptation per role | typed refs, schema, cache key, event builder | local mappings and supported roles | registry completeness, drift, latency, a11y, no cross-role bleed |
| D-9 Per-accessibility-profile UX adaptation per role | typed refs, schema, cache key, event builder | local mappings and supported roles | registry completeness, drift, latency, a11y, no cross-role bleed |
| D-10 Same-training-transfers invariant | typed refs, schema, cache key, event builder | local mappings and supported roles | registry completeness, drift, latency, a11y, no cross-role bleed |

## §F Migration

### §F.1 Migration phases

#### §F-1 Inventory

Every existing µservice declares whether it has user-facing, workflow-facing, policy-facing, or background-only role relevance.
Evidence required: manifest row, validator output, audit event sample, and ownership assignment.

#### §F-2 Registry declaration

Every relevant µservice adds a role-projection registry declaration with supported roles and primitive refs.
Evidence required: manifest row, validator output, audit event sample, and ownership assignment.

#### §F-3 Shared crate adoption

`oya-shared-role-projection` becomes the sole typed parser and validator for registry declarations.
Evidence required: manifest row, validator output, audit event sample, and ownership assignment.

#### §F-4 Cedar binding

Policy-engine adapters bind RoleProjection entity types and per-role permit sets.
Evidence required: manifest row, validator output, audit event sample, and ownership assignment.

#### §F-5 Ontology binding

Ontology projection refs declare object, link, field-redaction, and query templates.
Evidence required: manifest row, validator output, audit event sample, and ownership assignment.

#### §F-6 Workflow binding

Workflow template library refs declare role-appropriate templates and critical-path rows.
Evidence required: manifest row, validator output, audit event sample, and ownership assignment.

#### §F-7 UX shell binding

Application-facing adapters bind NavBar, Sidebar, KeyboardShortcut, VoiceCommand, and GestureSet refs.
Evidence required: manifest row, validator output, audit event sample, and ownership assignment.

#### §F-8 Device/locale/a11y profiles

Every role shell declares device, locale, and accessibility profiles with WCAG 2.2 AA evidence.
Evidence required: manifest row, validator output, audit event sample, and ownership assignment.

#### §F-9 Switch SLO enforcement

Role switching p95 <= 500 ms becomes a blocker for supported clients.
Evidence required: manifest row, validator output, audit event sample, and ownership assignment.

#### §F-10 Blocker promotion

Governance lanes promote from advisory to blocker after first full registry pass.
Evidence required: manifest row, validator output, audit event sample, and ownership assignment.

### §F.2 Per-existing-µservice registry declaration plan

#### §F.2.1 `accounting`

- Registry id: `role_projection_registry.accounting.v1`.
- Source spec: `specs/microservices/accounting.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.2 `anonymous`

- Registry id: `role_projection_registry.anonymous.v1`.
- Source spec: `specs/microservices/anonymous.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.3 `calendar`

- Registry id: `role_projection_registry.calendar.v1`.
- Source spec: `specs/microservices/calendar.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.4 `tenant-rbac-packaging`

- Registry id: `role_projection_registry.tenant_rbac_packaging.v1`.
- Source spec: `specs/tenant-rbac-packaging.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.5 `tenant-rbac`

- Registry id: `role_projection_registry.tenant_rbac.v1`.
- Source spec: `specs/microservices/tenant-rbac.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.6 `foundry`

- Registry id: `role_projection_registry.foundry.v1`.
- Source spec: `specs/microservices/foundry.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.7 `hr`

- Registry id: `role_projection_registry.hr.v1`.
- Source spec: `specs/microservices/hr.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.8 `mail`

- Registry id: `role_projection_registry.mail.v1`.
- Source spec: `specs/microservices/mail.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.9 `messenger`

- Registry id: `role_projection_registry.messenger.v1`.
- Source spec: `specs/microservices/messenger.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.10 `network`

- Registry id: `role_projection_registry.network.v1`.
- Source spec: `microservices/community/PRD.md` after Wave 15K network-to-community merge.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.11 `ontology`

- Registry id: `role_projection_registry.ontology.v1`.
- Source spec: `specs/microservices/ontology.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.12 `payroll`

- Registry id: `role_projection_registry.payroll.v1`.
- Source spec: `specs/microservices/payroll.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.13 `shorts`

- Registry id: `role_projection_registry.shorts.v1`.
- Source spec: `specs/microservices/shorts.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.14 `social`

- Registry id: `role_projection_registry.social.v1`.
- Source spec: `specs/microservices/social.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.15 `workflow-studio`

- Registry id: `role_projection_registry.workflow_studio.v1`.
- Source spec: `specs/microservices/workflow-studio.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

#### §F.2.16 `workflow`

- Registry id: `role_projection_registry.workflow.v1`.
- Source spec: `specs/microservices/workflow.json`.
- Required role set: declare supported role codes explicitly; unsupported roles must render no local shell routes.
- Cedar binding: local action namespace maps to RoleProjection permit sets; default-deny when projection is missing.
- Ontology binding: local objects and links list field-redaction behavior per role.
- Workflow binding: local workflow templates declare role visibility and critical-path rows.
- UX shell binding: local navigation, sidebar, command palette, keyboard shortcuts, voice commands, and gesture set refs must use shared primitive names.
- Device binding: laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only behavior is declared or explicitly refused with rationale.
- Locale binding: locale fallback chain, RTL handling, CJK rendering, and legal-copy pack overlay are declared.
- Accessibility binding: WCAG 2.2 AA evidence, reduced-motion behavior, keyboard parity, screen-reader labels, and voice/switch alternatives are declared.
- Switch-SLO binding: p95 visible role switch latency target is 500 ms or lower; cache partitions include tenant_id and role_projection_id.
- Audit binding: projection resolve, switch, denial, redaction, and shell-load events are emitted with no protected payload leakage.
- Migration risk: existing local role flags may conflict with the shared registry; local flags must become adapter inputs or be deleted.
- Stop condition: registry validates and local adapter passes no-cross-role-bleed tests.

### §F.3 Role-to-primitive declaration grid

| Role code | Primitive | Required declaration |
|---|---|---|
| ROLE_CLINICIAN_NURSE | D-1 | Yejin nurse role declares Role-projection Cedar entity type + permit set for hospital tenant; workflows: urgent care, shift handoff, EHR break-glass; obligations: life-safety and healthcare pack obligations. |
| ROLE_CLINICIAN_NURSE | D-2 | Yejin nurse role declares Per-role Ontology projection for hospital tenant; workflows: urgent care, shift handoff, EHR break-glass; obligations: life-safety and healthcare pack obligations. |
| ROLE_CLINICIAN_NURSE | D-3 | Yejin nurse role declares Per-role workflow template library for hospital tenant; workflows: urgent care, shift handoff, EHR break-glass; obligations: life-safety and healthcare pack obligations. |
| ROLE_CLINICIAN_NURSE | D-4 | Yejin nurse role declares Per-role UX shell primitives for hospital tenant; workflows: urgent care, shift handoff, EHR break-glass; obligations: life-safety and healthcare pack obligations. |
| ROLE_CLINICIAN_NURSE | D-5 | Yejin nurse role declares Role-context indicator for hospital tenant; workflows: urgent care, shift handoff, EHR break-glass; obligations: life-safety and healthcare pack obligations. |
| ROLE_CLINICIAN_NURSE | D-6 | Yejin nurse role declares Cross-role-switch friction floor for hospital tenant; workflows: urgent care, shift handoff, EHR break-glass; obligations: life-safety and healthcare pack obligations. |
| ROLE_CLINICIAN_NURSE | D-7 | Yejin nurse role declares Per-device profile per role for hospital tenant; workflows: urgent care, shift handoff, EHR break-glass; obligations: life-safety and healthcare pack obligations. |
| ROLE_CLINICIAN_NURSE | D-8 | Yejin nurse role declares Per-locale UX adaptation per role for hospital tenant; workflows: urgent care, shift handoff, EHR break-glass; obligations: life-safety and healthcare pack obligations. |
| ROLE_CLINICIAN_NURSE | D-9 | Yejin nurse role declares Per-accessibility-profile UX adaptation per role for hospital tenant; workflows: urgent care, shift handoff, EHR break-glass; obligations: life-safety and healthcare pack obligations. |
| ROLE_CLINICIAN_NURSE | D-10 | Yejin nurse role declares Same-training-transfers invariant for hospital tenant; workflows: urgent care, shift handoff, EHR break-glass; obligations: life-safety and healthcare pack obligations. |
| ROLE_PARENT_GUARDIAN | D-1 | Yejin parent role declares Role-projection Cedar entity type + permit set for personal/family tenant; workflows: school forms, child safety, consent, household calendar; obligations: ADR-0292 minor protection obligations. |
| ROLE_PARENT_GUARDIAN | D-2 | Yejin parent role declares Per-role Ontology projection for personal/family tenant; workflows: school forms, child safety, consent, household calendar; obligations: ADR-0292 minor protection obligations. |
| ROLE_PARENT_GUARDIAN | D-3 | Yejin parent role declares Per-role workflow template library for personal/family tenant; workflows: school forms, child safety, consent, household calendar; obligations: ADR-0292 minor protection obligations. |
| ROLE_PARENT_GUARDIAN | D-4 | Yejin parent role declares Per-role UX shell primitives for personal/family tenant; workflows: school forms, child safety, consent, household calendar; obligations: ADR-0292 minor protection obligations. |
| ROLE_PARENT_GUARDIAN | D-5 | Yejin parent role declares Role-context indicator for personal/family tenant; workflows: school forms, child safety, consent, household calendar; obligations: ADR-0292 minor protection obligations. |
| ROLE_PARENT_GUARDIAN | D-6 | Yejin parent role declares Cross-role-switch friction floor for personal/family tenant; workflows: school forms, child safety, consent, household calendar; obligations: ADR-0292 minor protection obligations. |
| ROLE_PARENT_GUARDIAN | D-7 | Yejin parent role declares Per-device profile per role for personal/family tenant; workflows: school forms, child safety, consent, household calendar; obligations: ADR-0292 minor protection obligations. |
| ROLE_PARENT_GUARDIAN | D-8 | Yejin parent role declares Per-locale UX adaptation per role for personal/family tenant; workflows: school forms, child safety, consent, household calendar; obligations: ADR-0292 minor protection obligations. |
| ROLE_PARENT_GUARDIAN | D-9 | Yejin parent role declares Per-accessibility-profile UX adaptation per role for personal/family tenant; workflows: school forms, child safety, consent, household calendar; obligations: ADR-0292 minor protection obligations. |
| ROLE_PARENT_GUARDIAN | D-10 | Yejin parent role declares Same-training-transfers invariant for personal/family tenant; workflows: school forms, child safety, consent, household calendar; obligations: ADR-0292 minor protection obligations. |
| ROLE_SIDE_BUSINESS_OWNER | D-1 | Yejin side-business-owner role declares Role-projection Cedar entity type + permit set for small-business tenant; workflows: invoices, payroll, bookings, customer messages; obligations: tax, payments, and SMB workflow obligations. |
| ROLE_SIDE_BUSINESS_OWNER | D-2 | Yejin side-business-owner role declares Per-role Ontology projection for small-business tenant; workflows: invoices, payroll, bookings, customer messages; obligations: tax, payments, and SMB workflow obligations. |
| ROLE_SIDE_BUSINESS_OWNER | D-3 | Yejin side-business-owner role declares Per-role workflow template library for small-business tenant; workflows: invoices, payroll, bookings, customer messages; obligations: tax, payments, and SMB workflow obligations. |
| ROLE_SIDE_BUSINESS_OWNER | D-4 | Yejin side-business-owner role declares Per-role UX shell primitives for small-business tenant; workflows: invoices, payroll, bookings, customer messages; obligations: tax, payments, and SMB workflow obligations. |
| ROLE_SIDE_BUSINESS_OWNER | D-5 | Yejin side-business-owner role declares Role-context indicator for small-business tenant; workflows: invoices, payroll, bookings, customer messages; obligations: tax, payments, and SMB workflow obligations. |
| ROLE_SIDE_BUSINESS_OWNER | D-6 | Yejin side-business-owner role declares Cross-role-switch friction floor for small-business tenant; workflows: invoices, payroll, bookings, customer messages; obligations: tax, payments, and SMB workflow obligations. |
| ROLE_SIDE_BUSINESS_OWNER | D-7 | Yejin side-business-owner role declares Per-device profile per role for small-business tenant; workflows: invoices, payroll, bookings, customer messages; obligations: tax, payments, and SMB workflow obligations. |
| ROLE_SIDE_BUSINESS_OWNER | D-8 | Yejin side-business-owner role declares Per-locale UX adaptation per role for small-business tenant; workflows: invoices, payroll, bookings, customer messages; obligations: tax, payments, and SMB workflow obligations. |
| ROLE_SIDE_BUSINESS_OWNER | D-9 | Yejin side-business-owner role declares Per-accessibility-profile UX adaptation per role for small-business tenant; workflows: invoices, payroll, bookings, customer messages; obligations: tax, payments, and SMB workflow obligations. |
| ROLE_SIDE_BUSINESS_OWNER | D-10 | Yejin side-business-owner role declares Same-training-transfers invariant for small-business tenant; workflows: invoices, payroll, bookings, customer messages; obligations: tax, payments, and SMB workflow obligations. |
| ROLE_EMPLOYEE | D-1 | employee role declares Role-projection Cedar entity type + permit set for employer tenant; workflows: mail, messenger, drive, calendar, HR self-service; obligations: ADR-0311 work-vs-personal boundary. |
| ROLE_EMPLOYEE | D-2 | employee role declares Per-role Ontology projection for employer tenant; workflows: mail, messenger, drive, calendar, HR self-service; obligations: ADR-0311 work-vs-personal boundary. |
| ROLE_EMPLOYEE | D-3 | employee role declares Per-role workflow template library for employer tenant; workflows: mail, messenger, drive, calendar, HR self-service; obligations: ADR-0311 work-vs-personal boundary. |
| ROLE_EMPLOYEE | D-4 | employee role declares Per-role UX shell primitives for employer tenant; workflows: mail, messenger, drive, calendar, HR self-service; obligations: ADR-0311 work-vs-personal boundary. |
| ROLE_EMPLOYEE | D-5 | employee role declares Role-context indicator for employer tenant; workflows: mail, messenger, drive, calendar, HR self-service; obligations: ADR-0311 work-vs-personal boundary. |
| ROLE_EMPLOYEE | D-6 | employee role declares Cross-role-switch friction floor for employer tenant; workflows: mail, messenger, drive, calendar, HR self-service; obligations: ADR-0311 work-vs-personal boundary. |
| ROLE_EMPLOYEE | D-7 | employee role declares Per-device profile per role for employer tenant; workflows: mail, messenger, drive, calendar, HR self-service; obligations: ADR-0311 work-vs-personal boundary. |
| ROLE_EMPLOYEE | D-8 | employee role declares Per-locale UX adaptation per role for employer tenant; workflows: mail, messenger, drive, calendar, HR self-service; obligations: ADR-0311 work-vs-personal boundary. |
| ROLE_EMPLOYEE | D-9 | employee role declares Per-accessibility-profile UX adaptation per role for employer tenant; workflows: mail, messenger, drive, calendar, HR self-service; obligations: ADR-0311 work-vs-personal boundary. |
| ROLE_EMPLOYEE | D-10 | employee role declares Same-training-transfers invariant for employer tenant; workflows: mail, messenger, drive, calendar, HR self-service; obligations: ADR-0311 work-vs-personal boundary. |
| ROLE_MANAGER | D-1 | manager role declares Role-projection Cedar entity type + permit set for employer tenant sub-scope; workflows: approvals, reviews, budgets, staffing; obligations: delegated management but not personal inspection. |
| ROLE_MANAGER | D-2 | manager role declares Per-role Ontology projection for employer tenant sub-scope; workflows: approvals, reviews, budgets, staffing; obligations: delegated management but not personal inspection. |
| ROLE_MANAGER | D-3 | manager role declares Per-role workflow template library for employer tenant sub-scope; workflows: approvals, reviews, budgets, staffing; obligations: delegated management but not personal inspection. |
| ROLE_MANAGER | D-4 | manager role declares Per-role UX shell primitives for employer tenant sub-scope; workflows: approvals, reviews, budgets, staffing; obligations: delegated management but not personal inspection. |
| ROLE_MANAGER | D-5 | manager role declares Role-context indicator for employer tenant sub-scope; workflows: approvals, reviews, budgets, staffing; obligations: delegated management but not personal inspection. |
| ROLE_MANAGER | D-6 | manager role declares Cross-role-switch friction floor for employer tenant sub-scope; workflows: approvals, reviews, budgets, staffing; obligations: delegated management but not personal inspection. |
| ROLE_MANAGER | D-7 | manager role declares Per-device profile per role for employer tenant sub-scope; workflows: approvals, reviews, budgets, staffing; obligations: delegated management but not personal inspection. |
| ROLE_MANAGER | D-8 | manager role declares Per-locale UX adaptation per role for employer tenant sub-scope; workflows: approvals, reviews, budgets, staffing; obligations: delegated management but not personal inspection. |
| ROLE_MANAGER | D-9 | manager role declares Per-accessibility-profile UX adaptation per role for employer tenant sub-scope; workflows: approvals, reviews, budgets, staffing; obligations: delegated management but not personal inspection. |
| ROLE_MANAGER | D-10 | manager role declares Same-training-transfers invariant for employer tenant sub-scope; workflows: approvals, reviews, budgets, staffing; obligations: delegated management but not personal inspection. |
| ROLE_INTERNAL_AUDITOR | D-1 | internal auditor role declares Role-projection Cedar entity type + permit set for enterprise tenant audit sub-scope; workflows: SOX, SOC2, evidence pulls, sampling; obligations: scope-bounded read access. |
| ROLE_INTERNAL_AUDITOR | D-2 | internal auditor role declares Per-role Ontology projection for enterprise tenant audit sub-scope; workflows: SOX, SOC2, evidence pulls, sampling; obligations: scope-bounded read access. |
| ROLE_INTERNAL_AUDITOR | D-3 | internal auditor role declares Per-role workflow template library for enterprise tenant audit sub-scope; workflows: SOX, SOC2, evidence pulls, sampling; obligations: scope-bounded read access. |
| ROLE_INTERNAL_AUDITOR | D-4 | internal auditor role declares Per-role UX shell primitives for enterprise tenant audit sub-scope; workflows: SOX, SOC2, evidence pulls, sampling; obligations: scope-bounded read access. |
| ROLE_INTERNAL_AUDITOR | D-5 | internal auditor role declares Role-context indicator for enterprise tenant audit sub-scope; workflows: SOX, SOC2, evidence pulls, sampling; obligations: scope-bounded read access. |
| ROLE_INTERNAL_AUDITOR | D-6 | internal auditor role declares Cross-role-switch friction floor for enterprise tenant audit sub-scope; workflows: SOX, SOC2, evidence pulls, sampling; obligations: scope-bounded read access. |
| ROLE_INTERNAL_AUDITOR | D-7 | internal auditor role declares Per-device profile per role for enterprise tenant audit sub-scope; workflows: SOX, SOC2, evidence pulls, sampling; obligations: scope-bounded read access. |
| ROLE_INTERNAL_AUDITOR | D-8 | internal auditor role declares Per-locale UX adaptation per role for enterprise tenant audit sub-scope; workflows: SOX, SOC2, evidence pulls, sampling; obligations: scope-bounded read access. |
| ROLE_INTERNAL_AUDITOR | D-9 | internal auditor role declares Per-accessibility-profile UX adaptation per role for enterprise tenant audit sub-scope; workflows: SOX, SOC2, evidence pulls, sampling; obligations: scope-bounded read access. |
| ROLE_INTERNAL_AUDITOR | D-10 | internal auditor role declares Same-training-transfers invariant for enterprise tenant audit sub-scope; workflows: SOX, SOC2, evidence pulls, sampling; obligations: scope-bounded read access. |
| ROLE_3PAO_ASSESSOR | D-1 | third-party assessor role declares Role-projection Cedar entity type + permit set for regulated assessment tenant link; workflows: FedRAMP or SOC2 evidence review; obligations: temporary scoped external permit set. |
| ROLE_3PAO_ASSESSOR | D-2 | third-party assessor role declares Per-role Ontology projection for regulated assessment tenant link; workflows: FedRAMP or SOC2 evidence review; obligations: temporary scoped external permit set. |
| ROLE_3PAO_ASSESSOR | D-3 | third-party assessor role declares Per-role workflow template library for regulated assessment tenant link; workflows: FedRAMP or SOC2 evidence review; obligations: temporary scoped external permit set. |
| ROLE_3PAO_ASSESSOR | D-4 | third-party assessor role declares Per-role UX shell primitives for regulated assessment tenant link; workflows: FedRAMP or SOC2 evidence review; obligations: temporary scoped external permit set. |
| ROLE_3PAO_ASSESSOR | D-5 | third-party assessor role declares Role-context indicator for regulated assessment tenant link; workflows: FedRAMP or SOC2 evidence review; obligations: temporary scoped external permit set. |
| ROLE_3PAO_ASSESSOR | D-6 | third-party assessor role declares Cross-role-switch friction floor for regulated assessment tenant link; workflows: FedRAMP or SOC2 evidence review; obligations: temporary scoped external permit set. |
| ROLE_3PAO_ASSESSOR | D-7 | third-party assessor role declares Per-device profile per role for regulated assessment tenant link; workflows: FedRAMP or SOC2 evidence review; obligations: temporary scoped external permit set. |
| ROLE_3PAO_ASSESSOR | D-8 | third-party assessor role declares Per-locale UX adaptation per role for regulated assessment tenant link; workflows: FedRAMP or SOC2 evidence review; obligations: temporary scoped external permit set. |
| ROLE_3PAO_ASSESSOR | D-9 | third-party assessor role declares Per-accessibility-profile UX adaptation per role for regulated assessment tenant link; workflows: FedRAMP or SOC2 evidence review; obligations: temporary scoped external permit set. |
| ROLE_3PAO_ASSESSOR | D-10 | third-party assessor role declares Same-training-transfers invariant for regulated assessment tenant link; workflows: FedRAMP or SOC2 evidence review; obligations: temporary scoped external permit set. |
| ROLE_TENANT_ADMIN | D-1 | tenant administrator role declares Role-projection Cedar entity type + permit set for tenant root or sub-scope; workflows: policy, billing, users, packs, residency; obligations: strong step-up and audit obligations. |
| ROLE_TENANT_ADMIN | D-2 | tenant administrator role declares Per-role Ontology projection for tenant root or sub-scope; workflows: policy, billing, users, packs, residency; obligations: strong step-up and audit obligations. |
| ROLE_TENANT_ADMIN | D-3 | tenant administrator role declares Per-role workflow template library for tenant root or sub-scope; workflows: policy, billing, users, packs, residency; obligations: strong step-up and audit obligations. |
| ROLE_TENANT_ADMIN | D-4 | tenant administrator role declares Per-role UX shell primitives for tenant root or sub-scope; workflows: policy, billing, users, packs, residency; obligations: strong step-up and audit obligations. |
| ROLE_TENANT_ADMIN | D-5 | tenant administrator role declares Role-context indicator for tenant root or sub-scope; workflows: policy, billing, users, packs, residency; obligations: strong step-up and audit obligations. |
| ROLE_TENANT_ADMIN | D-6 | tenant administrator role declares Cross-role-switch friction floor for tenant root or sub-scope; workflows: policy, billing, users, packs, residency; obligations: strong step-up and audit obligations. |
| ROLE_TENANT_ADMIN | D-7 | tenant administrator role declares Per-device profile per role for tenant root or sub-scope; workflows: policy, billing, users, packs, residency; obligations: strong step-up and audit obligations. |
| ROLE_TENANT_ADMIN | D-8 | tenant administrator role declares Per-locale UX adaptation per role for tenant root or sub-scope; workflows: policy, billing, users, packs, residency; obligations: strong step-up and audit obligations. |
| ROLE_TENANT_ADMIN | D-9 | tenant administrator role declares Per-accessibility-profile UX adaptation per role for tenant root or sub-scope; workflows: policy, billing, users, packs, residency; obligations: strong step-up and audit obligations. |
| ROLE_TENANT_ADMIN | D-10 | tenant administrator role declares Same-training-transfers invariant for tenant root or sub-scope; workflows: policy, billing, users, packs, residency; obligations: strong step-up and audit obligations. |
| ROLE_DEVELOPER | D-1 | developer role declares Role-projection Cedar entity type + permit set for developer workspace tenant; workflows: SDK, API keys, app registration, logs; obligations: separate from production tenant authority. |
| ROLE_DEVELOPER | D-2 | developer role declares Per-role Ontology projection for developer workspace tenant; workflows: SDK, API keys, app registration, logs; obligations: separate from production tenant authority. |
| ROLE_DEVELOPER | D-3 | developer role declares Per-role workflow template library for developer workspace tenant; workflows: SDK, API keys, app registration, logs; obligations: separate from production tenant authority. |
| ROLE_DEVELOPER | D-4 | developer role declares Per-role UX shell primitives for developer workspace tenant; workflows: SDK, API keys, app registration, logs; obligations: separate from production tenant authority. |
| ROLE_DEVELOPER | D-5 | developer role declares Role-context indicator for developer workspace tenant; workflows: SDK, API keys, app registration, logs; obligations: separate from production tenant authority. |
| ROLE_DEVELOPER | D-6 | developer role declares Cross-role-switch friction floor for developer workspace tenant; workflows: SDK, API keys, app registration, logs; obligations: separate from production tenant authority. |
| ROLE_DEVELOPER | D-7 | developer role declares Per-device profile per role for developer workspace tenant; workflows: SDK, API keys, app registration, logs; obligations: separate from production tenant authority. |
| ROLE_DEVELOPER | D-8 | developer role declares Per-locale UX adaptation per role for developer workspace tenant; workflows: SDK, API keys, app registration, logs; obligations: separate from production tenant authority. |
| ROLE_DEVELOPER | D-9 | developer role declares Per-accessibility-profile UX adaptation per role for developer workspace tenant; workflows: SDK, API keys, app registration, logs; obligations: separate from production tenant authority. |
| ROLE_DEVELOPER | D-10 | developer role declares Same-training-transfers invariant for developer workspace tenant; workflows: SDK, API keys, app registration, logs; obligations: separate from production tenant authority. |
| ROLE_SUPPORT_AGENT | D-1 | support agent role declares Role-projection Cedar entity type + permit set for provider support tenant; workflows: case triage, safe diagnostics, tenant support; obligations: just-in-time break-glass constraints. |
| ROLE_SUPPORT_AGENT | D-2 | support agent role declares Per-role Ontology projection for provider support tenant; workflows: case triage, safe diagnostics, tenant support; obligations: just-in-time break-glass constraints. |
| ROLE_SUPPORT_AGENT | D-3 | support agent role declares Per-role workflow template library for provider support tenant; workflows: case triage, safe diagnostics, tenant support; obligations: just-in-time break-glass constraints. |
| ROLE_SUPPORT_AGENT | D-4 | support agent role declares Per-role UX shell primitives for provider support tenant; workflows: case triage, safe diagnostics, tenant support; obligations: just-in-time break-glass constraints. |
| ROLE_SUPPORT_AGENT | D-5 | support agent role declares Role-context indicator for provider support tenant; workflows: case triage, safe diagnostics, tenant support; obligations: just-in-time break-glass constraints. |
| ROLE_SUPPORT_AGENT | D-6 | support agent role declares Cross-role-switch friction floor for provider support tenant; workflows: case triage, safe diagnostics, tenant support; obligations: just-in-time break-glass constraints. |
| ROLE_SUPPORT_AGENT | D-7 | support agent role declares Per-device profile per role for provider support tenant; workflows: case triage, safe diagnostics, tenant support; obligations: just-in-time break-glass constraints. |
| ROLE_SUPPORT_AGENT | D-8 | support agent role declares Per-locale UX adaptation per role for provider support tenant; workflows: case triage, safe diagnostics, tenant support; obligations: just-in-time break-glass constraints. |
| ROLE_SUPPORT_AGENT | D-9 | support agent role declares Per-accessibility-profile UX adaptation per role for provider support tenant; workflows: case triage, safe diagnostics, tenant support; obligations: just-in-time break-glass constraints. |
| ROLE_SUPPORT_AGENT | D-10 | support agent role declares Same-training-transfers invariant for provider support tenant; workflows: case triage, safe diagnostics, tenant support; obligations: just-in-time break-glass constraints. |
| ROLE_REGULATOR | D-1 | regulator role declares Role-projection Cedar entity type + permit set for jurisdiction authority scope; workflows: lawful audit, investigation, report review; obligations: warrant or statute-scoped access. |
| ROLE_REGULATOR | D-2 | regulator role declares Per-role Ontology projection for jurisdiction authority scope; workflows: lawful audit, investigation, report review; obligations: warrant or statute-scoped access. |
| ROLE_REGULATOR | D-3 | regulator role declares Per-role workflow template library for jurisdiction authority scope; workflows: lawful audit, investigation, report review; obligations: warrant or statute-scoped access. |
| ROLE_REGULATOR | D-4 | regulator role declares Per-role UX shell primitives for jurisdiction authority scope; workflows: lawful audit, investigation, report review; obligations: warrant or statute-scoped access. |
| ROLE_REGULATOR | D-5 | regulator role declares Role-context indicator for jurisdiction authority scope; workflows: lawful audit, investigation, report review; obligations: warrant or statute-scoped access. |
| ROLE_REGULATOR | D-6 | regulator role declares Cross-role-switch friction floor for jurisdiction authority scope; workflows: lawful audit, investigation, report review; obligations: warrant or statute-scoped access. |
| ROLE_REGULATOR | D-7 | regulator role declares Per-device profile per role for jurisdiction authority scope; workflows: lawful audit, investigation, report review; obligations: warrant or statute-scoped access. |
| ROLE_REGULATOR | D-8 | regulator role declares Per-locale UX adaptation per role for jurisdiction authority scope; workflows: lawful audit, investigation, report review; obligations: warrant or statute-scoped access. |
| ROLE_REGULATOR | D-9 | regulator role declares Per-accessibility-profile UX adaptation per role for jurisdiction authority scope; workflows: lawful audit, investigation, report review; obligations: warrant or statute-scoped access. |
| ROLE_REGULATOR | D-10 | regulator role declares Same-training-transfers invariant for jurisdiction authority scope; workflows: lawful audit, investigation, report review; obligations: warrant or statute-scoped access. |
| ROLE_CONTRACTOR | D-1 | contractor role declares Role-projection Cedar entity type + permit set for limited project sub-scope; workflows: project tasks and deliverables; obligations: time-bound access and offboarding revoke. |
| ROLE_CONTRACTOR | D-2 | contractor role declares Per-role Ontology projection for limited project sub-scope; workflows: project tasks and deliverables; obligations: time-bound access and offboarding revoke. |
| ROLE_CONTRACTOR | D-3 | contractor role declares Per-role workflow template library for limited project sub-scope; workflows: project tasks and deliverables; obligations: time-bound access and offboarding revoke. |
| ROLE_CONTRACTOR | D-4 | contractor role declares Per-role UX shell primitives for limited project sub-scope; workflows: project tasks and deliverables; obligations: time-bound access and offboarding revoke. |
| ROLE_CONTRACTOR | D-5 | contractor role declares Role-context indicator for limited project sub-scope; workflows: project tasks and deliverables; obligations: time-bound access and offboarding revoke. |
| ROLE_CONTRACTOR | D-6 | contractor role declares Cross-role-switch friction floor for limited project sub-scope; workflows: project tasks and deliverables; obligations: time-bound access and offboarding revoke. |
| ROLE_CONTRACTOR | D-7 | contractor role declares Per-device profile per role for limited project sub-scope; workflows: project tasks and deliverables; obligations: time-bound access and offboarding revoke. |
| ROLE_CONTRACTOR | D-8 | contractor role declares Per-locale UX adaptation per role for limited project sub-scope; workflows: project tasks and deliverables; obligations: time-bound access and offboarding revoke. |
| ROLE_CONTRACTOR | D-9 | contractor role declares Per-accessibility-profile UX adaptation per role for limited project sub-scope; workflows: project tasks and deliverables; obligations: time-bound access and offboarding revoke. |
| ROLE_CONTRACTOR | D-10 | contractor role declares Same-training-transfers invariant for limited project sub-scope; workflows: project tasks and deliverables; obligations: time-bound access and offboarding revoke. |

### §F.4 Device profile requirements

#### §F.4.1 `laptop`

- Constraints: full keyboard, pointer, command palette, side-by-side panels.
- Required behavior: dense or comfortable layout with complete shortcut map.
- Cedar authority: unchanged by device profile; device profile may only remove unsafe affordances or require step-up.
- UX shell: role-context indicator remains perceivable through the device modality.
- Validation: snapshot, interaction, or assistive-technology proof exists for every supported role.

#### §F.4.2 `desktop`

- Constraints: large screen, full keyboard, multi-window, high-density data review.
- Required behavior: persistent sidebar and advanced table controls.
- Cedar authority: unchanged by device profile; device profile may only remove unsafe affordances or require step-up.
- UX shell: role-context indicator remains perceivable through the device modality.
- Validation: snapshot, interaction, or assistive-technology proof exists for every supported role.

#### §F.4.3 `tablet`

- Constraints: touch-first, optional keyboard, split-view capable.
- Required behavior: larger targets and no hover-only actions.
- Cedar authority: unchanged by device profile; device profile may only remove unsafe affordances or require step-up.
- UX shell: role-context indicator remains perceivable through the device modality.
- Validation: snapshot, interaction, or assistive-technology proof exists for every supported role.

#### §F.4.4 `phone`

- Constraints: one-handed paths, constrained screen, intermittent connectivity.
- Required behavior: bottom navigation or compact role switcher with clear active context.
- Cedar authority: unchanged by device profile; device profile may only remove unsafe affordances or require step-up.
- UX shell: role-context indicator remains perceivable through the device modality.
- Validation: snapshot, interaction, or assistive-technology proof exists for every supported role.

#### §F.4.5 `wearable`

- Constraints: glanceable notifications and one-tap safe actions.
- Required behavior: no high-risk mutation without paired device or explicit safe shortcut.
- Cedar authority: unchanged by device profile; device profile may only remove unsafe affordances or require step-up.
- UX shell: role-context indicator remains perceivable through the device modality.
- Validation: snapshot, interaction, or assistive-technology proof exists for every supported role.

#### §F.4.6 `ar-overlay`

- Constraints: hands-free overlays, spatial anchoring, constrained privacy.
- Required behavior: minimal data exposure and explicit role badge in overlay.
- Cedar authority: unchanged by device profile; device profile may only remove unsafe affordances or require step-up.
- UX shell: role-context indicator remains perceivable through the device modality.
- Validation: snapshot, interaction, or assistive-technology proof exists for every supported role.

#### §F.4.7 `voice-only`

- Constraints: spoken commands, confirmations, interruption handling.
- Required behavior: command grammar maps to same primitive verbs.
- Cedar authority: unchanged by device profile; device profile may only remove unsafe affordances or require step-up.
- UX shell: role-context indicator remains perceivable through the device modality.
- Validation: snapshot, interaction, or assistive-technology proof exists for every supported role.

#### §F.4.8 `screen-reader-only`

- Constraints: semantic landmarks, focus order, clear labels.
- Required behavior: full parity with visual shell and no hidden role context.
- Cedar authority: unchanged by device profile; device profile may only remove unsafe affordances or require step-up.
- UX shell: role-context indicator remains perceivable through the device modality.
- Validation: snapshot, interaction, or assistive-technology proof exists for every supported role.

### §F.5 Locale adaptation requirements

#### §F.5.1 `en-US`

- Locale profile `en-US` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.2 `ko-KR`

- Locale profile `ko-KR` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.3 `ja-JP`

- Locale profile `ja-JP` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.4 `zh-Hans-CN`

- Locale profile `zh-Hans-CN` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.5 `zh-Hant-TW`

- Locale profile `zh-Hant-TW` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.6 `es-MX`

- Locale profile `es-MX` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.7 `pt-BR`

- Locale profile `pt-BR` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.8 `de-DE`

- Locale profile `de-DE` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.9 `fr-FR`

- Locale profile `fr-FR` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.10 `ar-SA`

- Locale profile `ar-SA` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.11 `he-IL`

- Locale profile `he-IL` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

#### §F.5.12 `hi-IN`

- Locale profile `hi-IN` declares labels, command grammar, date/time, number, currency, pluralization, and legal-copy fallback.
- Role labels must remain understandable without relying on color or icon shape.
- Critical-path and consent copy must use pack-approved translations; machine translation alone is insufficient.
- Directionality, CJK sizing, and regional formatting follow ux-best-practices.md §5.

### §F.6 Accessibility profile requirements

#### §F.6.1 `screen-reader-only`

- Requirement: semantic landmarks, role-context indicator announced, switch events announced politely.
- WCAG 2.2 AA minimum applies to every shipped role shell path.
- Regulated or critical-path flows may require stronger evidence and manual audit.
- The accommodation changes presentation and pacing, not unauthorized data scope.

#### §F.6.2 `keyboard-only`

- Requirement: complete traversal, skip links, no positive tabindex, focus returns after modal role picker.
- WCAG 2.2 AA minimum applies to every shipped role shell path.
- Regulated or critical-path flows may require stronger evidence and manual audit.
- The accommodation changes presentation and pacing, not unauthorized data scope.

#### §F.6.3 `voice-control-only`

- Requirement: stable command names, visible labels match spoken grammar, no hover-only controls.
- WCAG 2.2 AA minimum applies to every shipped role shell path.
- Regulated or critical-path flows may require stronger evidence and manual audit.
- The accommodation changes presentation and pacing, not unauthorized data scope.

#### §F.6.4 `single-switch`

- Requirement: linear scan order, timeout extension, no irreversible action on accidental dwell.
- WCAG 2.2 AA minimum applies to every shipped role shell path.
- Regulated or critical-path flows may require stronger evidence and manual audit.
- The accommodation changes presentation and pacing, not unauthorized data scope.

#### §F.6.5 `low-vision`

- Requirement: contrast, zoom, focus visibility, no clipped role badge at 200% text resize.
- WCAG 2.2 AA minimum applies to every shipped role shell path.
- Regulated or critical-path flows may require stronger evidence and manual audit.
- The accommodation changes presentation and pacing, not unauthorized data scope.

#### §F.6.6 `reduced-motion`

- Requirement: switch animation can be reduced to instant state change without losing orientation.
- WCAG 2.2 AA minimum applies to every shipped role shell path.
- Regulated or critical-path flows may require stronger evidence and manual audit.
- The accommodation changes presentation and pacing, not unauthorized data scope.

#### §F.6.7 `cognitive-support`

- Requirement: clear role name, confirmation only for high-value mutations, no jargon-only role codes.
- WCAG 2.2 AA minimum applies to every shipped role shell path.
- Regulated or critical-path flows may require stronger evidence and manual audit.
- The accommodation changes presentation and pacing, not unauthorized data scope.

#### §F.6.8 `post-trauma-safety`

- Requirement: safe exit, privacy-preserving previews, no surprise notification on sensitive roles.
- WCAG 2.2 AA minimum applies to every shipped role shell path.
- Regulated or critical-path flows may require stronger evidence and manual audit.
- The accommodation changes presentation and pacing, not unauthorized data scope.

#### §F.6.9 `minor-safe`

- Requirement: ADR-0292 protections, guardian/child context clarity, no dark patterns.
- WCAG 2.2 AA minimum applies to every shipped role shell path.
- Regulated or critical-path flows may require stronger evidence and manual audit.
- The accommodation changes presentation and pacing, not unauthorized data scope.

#### §F.6.10 `language-support`

- Requirement: simple language fallback, locale-aware examples, no untranslated legal blockers.
- WCAG 2.2 AA minimum applies to every shipped role shell path.
- Regulated or critical-path flows may require stronger evidence and manual audit.
- The accommodation changes presentation and pacing, not unauthorized data scope.

## §G References

### §G.1 Official external references

1. Apple account-driven enrollment: https://support.apple.com/en-mt/guide/deployment/dep4d9e9cd26/web
   - Used for: Managed Apple Account and personal Apple Account can coexist with complete separation of work and personal data.
2. Apple Human Interface Guidelines: https://developer.apple.com/design/human-interface-guidelines/
   - Used for: Primary Apple design-system reference for Apple-family interaction and visual patterns.
3. Microsoft account boundary: https://support.microsoft.com/en-us/accounts-billing/manage/what-s-the-difference-between-a-microsoft-account-and-a-work-or-school-account
   - Used for: Microsoft distinguishes personal accounts from work or school accounts and states that user-account information does not synchronize between them.
4. Microsoft Fluent Nav: https://fluent2.microsoft.design/components/web/react/core/nav/usage
   - Used for: Fluent navigation should be coherent, task- or feature-oriented by context, brief, consistently ordered, and accessible beyond hover.
5. Google Workspace account boundary: https://support.google.com/a/answer/6250166?hl=en
   - Used for: Google Workspace accounts can behave differently from personal accounts and can be administered by an organization.
6. Google Material navigation: https://m1.material.io/patterns/navigation.html
   - Used for: Material navigation guidance tailors navigation to important content, tasks, and in-context movement.
7. Salesforce Lightning role home page assignment: https://help.salesforce.com/s/articleView?id=xcloud.admin_home_lex_intro.htm&language=en_US&type=5
   - Used for: Salesforce permits assigning Lightning home pages to apps and app/profile combinations.
8. ServiceNow Service Portal vs Workspace: https://www.servicenow.com/docs/r/application-development/dev-get-start-service-portal-vs-workspace.html
   - Used for: ServiceNow separates requester self-service portals from agent or fulfiller workspaces.
9. W3C WCAG 2.2: https://www.w3.org/TR/WCAG22/
   - Used for: W3C Recommendation baseline for WCAG 2.2 conformance.

### §G.2 Internal references

1. documentation-rigor.md §3.2.3: UX floor requires no default-path friction, preserved accessibility, and graceful recovery.
2. documentation-rigor.md §3.2.5: Critical-path edge cases must name safety, security, and policy handling.
3. ux-best-practices.md §3: WCAG 2.2 AA is the minimum across shipped surfaces.
4. ux-best-practices.md §5: Locale, RTL, CJK, date/time, and number formatting require canonical i18n paths.
5. ADR-0244: tenant_id and sub_scope_path remain universal scoping primitives.
6. ADR-0292: minor-facing role shells inherit minor protection and accommodation duties.
7. ADR-0299: same human can bridge contexts through passkey identity and recovery doctrine.
8. ADR-0311: work and personal tenant boundaries remain non-negotiable.
9. ADR-0316: adjacent doctrine reference reserved for the preceding Wave-3-G record; missing local file is explicitly called out.

### §G.3 Reference interpretation boundaries

- External design systems are precedent evidence, not authority over Oyatie tenant, Cedar, Ontology, or Workflow doctrine.
- Where external UX precedent conflicts with documentation-rigor.md critical-path requirements, documentation-rigor.md wins.
- Where a role projection needs stronger privacy or accessibility behavior than a cited vendor pattern, Oyatie must choose the stricter behavior.
- Salesforce and ServiceNow prove that role-shaped experiences are normal; they do not authorize suite-style ownership boundaries.
- Apple, Microsoft, and Google prove account-context separation; they do not replace ADR-0244 tenant scoping or ADR-0311 personal-vs-work boundaries.

## §H Change Log + Naming Justifications

### §H.1 Change log

| Date | Change | Authoring context |
|---|---|---|
| 2026-05-20 | Initial ADR-0317 proposed. | Authored from `/tmp/codex-brief-adr-0317-role-projection.md`; grounded in documentation-rigor.md, ux-best-practices.md, ADR-0244, ADR-0292, ADR-0299, ADR-0311, and external official design/account-context precedents. |

### §H.2 Naming justifications

#### §H.2.1 `RoleProjection`

- Meaning: Cedar and Rust entity name for an active role-bound projection of one principal in one tenant/sub-scope.
- Justification: It names the active projection, not the human, tenant, or UI shell alone.
- Rejection rule: future modifiers must not rename this without proving registry migration, audit compatibility, documentation updates, and user-training impact.

#### §H.2.2 `role_projection_id`

- Meaning: Stable registry id used across Cedar, Ontology, Workflow, UX shell, cache, and audit events.
- Justification: It prevents each layer from inventing incompatible role context keys.
- Rejection rule: future modifiers must not rename this without proving registry migration, audit compatibility, documentation updates, and user-training impact.

#### §H.2.3 `oya-shared-role-projection`

- Meaning: Shared crate for projection registry types and adapter traits.
- Justification: It is shared substrate because every µservice and user-facing shell needs the same primitive.
- Rejection rule: future modifiers must not rename this without proving registry migration, audit compatibility, documentation updates, and user-training impact.

#### §H.2.4 `RoleProjectionSwitched`

- Meaning: Audit event for successful active role transition.
- Justification: It is explicit enough for traceability and avoids generic navigation telemetry.
- Rejection rule: future modifiers must not rename this without proving registry migration, audit compatibility, documentation updates, and user-training impact.

#### §H.2.5 `RoleProjectionDenied`

- Meaning: Audit event for missing, stale, or unauthorized role context.
- Justification: It distinguishes authorization refusal from shell rendering or routing failures.
- Rejection rule: future modifiers must not rename this without proving registry migration, audit compatibility, documentation updates, and user-training impact.

#### §H.2.6 `role_context_indicator`

- Meaning: UX shell component contract for visible active role and tenant context.
- Justification: It states function rather than visual treatment so each device profile can render appropriately.
- Rejection rule: future modifiers must not rename this without proving registry migration, audit compatibility, documentation updates, and user-training impact.

#### §H.2.7 `primitive_vocabulary_version`

- Meaning: Version marker for shared nav, command, keyboard, voice, and gesture vocabulary.
- Justification: It protects same-training-transfers across roles and surfaces.
- Rejection rule: future modifiers must not rename this without proving registry migration, audit compatibility, documentation updates, and user-training impact.

### §H.3 Validators authorized by this ADR

- `oya-governance-role-projection-registry`: Validates required registry fields, signatures, versions, and role-to-primitive completeness.
- `oya-governance-role-context-indicator`: Validates visible and assistive-technology-perceivable active-role indicators.
- `oya-governance-role-switch-latency`: Validates p95 visible role switch latency <= 500 ms for supported clients.
- `oya-governance-role-shell-a11y`: Validates WCAG 2.2 AA evidence and per-accommodation declarations.
- `oya-governance-role-shell-same-training`: Validates primitive vocabulary reuse and exception waivers.
- `oya-governance-per-microservice-role-adapter`: Validates each µservice adapter declaration against shared schema and local surfaces.
- `oya-governance-role-cache-partition`: Validates cache keys include tenant_id, sub_scope_path, and role_projection_id.
- `oya-governance-role-ontology-redaction`: Validates field redaction reasons and object/link allowlists.
- `oya-governance-role-workflow-template-scope`: Validates workflow templates are role-visible only when Cedar and critical-path rules allow.
- `oya-governance-role-device-profile`: Validates every supported device class has a declared behavior or explicit refusal.

### §H.4 Final doctrine statement

- The same human may hold many legitimate roles.
- Each role sees a different projection.
- The underlying identity, tenant scoping, Cedar gate, Ontology model, Workflow Engine, and UX primitive vocabulary remain unified.
- Role switching is visible, fast, audited, accessible, and cache-isolated.
- The user learns the vocabulary once and carries it across roles.
- The role-context indicator is not decoration; it is a safety, privacy, and policy control.
- A missing role projection is a default-deny condition, not an invitation to guess.

## Appendix A: Projection Acceptance Checklist

### Appendix A.1 `accounting` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.2 `anonymous` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.3 `calendar` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.4 `tenant-rbac-packaging` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.5 `tenant-rbac` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.6 `foundry` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.7 `hr` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.8 `mail` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.9 `messenger` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.10 `network` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.11 `ontology` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.12 `payroll` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.13 `shorts` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.14 `social` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.15 `workflow-studio` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

### Appendix A.16 `workflow` acceptance checklist

- [ ] Registry row exists and is versioned.
- [ ] Role codes are explicit and unsupported roles are refused.
- [ ] Cedar RoleProjection entity binding is present.
- [ ] Permit set refs and deny overlay refs are listed.
- [ ] Ontology object allowlist is listed.
- [ ] Ontology link allowlist is listed.
- [ ] Field redaction policy is listed.
- [ ] Workflow template library refs are listed.
- [ ] Critical-path rows are listed or explicitly not applicable.
- [ ] NavBar ref uses shared primitive names.
- [ ] Sidebar ref uses shared primitive names.
- [ ] KeyboardShortcut map has role-specific commands only as extensions.
- [ ] VoiceCommand set maps to stable command vocabulary.
- [ ] GestureSet has keyboard and assistive alternatives.
- [ ] Role-context indicator renders in visual shell.
- [ ] Role-context indicator is announced to screen readers.
- [ ] Device profiles cover laptop, desktop, tablet, phone, wearable, AR-overlay, voice-only, and screen-reader-only.
- [ ] Locale profiles include fallback chain and RTL/CJK behavior.
- [ ] Accessibility profiles satisfy WCAG 2.2 AA minimum.
- [ ] Switch SLO proof shows p95 <= 500 ms or the surface is blocked.
- [ ] Cache partition includes tenant_id, sub_scope_path, and role_projection_id.
- [ ] Audit events avoid protected payload leakage.
- [ ] Same-training-transfer exceptions have owner and expiry.
- [ ] Adapter tests prove no cross-role data bleed.

## Appendix B: Role Projection Event Catalog

| Event | Trigger | Severity | Payload boundary |
|---|---|---|---|
| `RoleProjectionResolved` | Projection resolver returns a valid active projection. | info | Must include ids, versions, and reason codes; must not include PHI/PII/content payloads. |
| `RoleProjectionSwitched` | Human changes active role context. | info | Must include ids, versions, and reason codes; must not include PHI/PII/content payloads. |
| `RoleProjectionDenied` | Projection resolver refuses missing, stale, or unauthorized role. | warning | Must include ids, versions, and reason codes; must not include PHI/PII/content payloads. |
| `RoleProjectionAmbiguous` | Multiple valid projections exist and no explicit user choice is present. | warning | Must include ids, versions, and reason codes; must not include PHI/PII/content payloads. |
| `RoleProjectionCacheInvalidated` | Registry version or permit version invalidates a cached projection. | info | Must include ids, versions, and reason codes; must not include PHI/PII/content payloads. |
| `RoleProjectionRedactionApplied` | Ontology field redaction is applied for a role. | info | Must include ids, versions, and reason codes; must not include PHI/PII/content payloads. |
| `RoleProjectionCrossRoleBleedPrevented` | Cache/search/session guard prevents prior-role data exposure. | high | Must include ids, versions, and reason codes; must not include PHI/PII/content payloads. |
| `RoleProjectionA11yEvidenceMissing` | Role shell lacks required accessibility evidence. | high | Must include ids, versions, and reason codes; must not include PHI/PII/content payloads. |
| `RoleProjectionSwitchSloBreached` | Role switch p95 exceeds 500 ms on supported profile. | medium | Must include ids, versions, and reason codes; must not include PHI/PII/content payloads. |
| `RoleProjectionTrainingExceptionUsed` | Role shell uses approved vocabulary exception. | info | Must include ids, versions, and reason codes; must not include PHI/PII/content payloads. |

## Appendix C: Hyperscaler Precedent Coverage by Primitive

| Primitive | Precedent count | Evidence notes |
|---|---:|---|
| D-1 Role-projection Cedar entity type + permit set | 3 | Apple separates Managed Apple Account work data from personal Apple Account data on the same device.; Microsoft distinguishes personal Microsoft accounts from work or school accounts and does not synchronize user-account information between them.; Google Workspace accounts can be administrator-managed and can expose different product behavior from personal accounts. |
| D-2 Per-role Ontology projection | 3 | Salesforce Lightning lets admins assign role-appropriate home pages through app/profile combinations.; ServiceNow distinguishes requester portals from agent workspaces, with different audiences, purposes, navigation, analytics, and configuration tools.; Material navigation guidance supports task-focused and in-context navigation for specific data sets. |
| D-3 Per-role workflow template library | 3 | ServiceNow Workspaces are task-centric locations for fulfillers to triage, collaborate, resolve records, and view analytics.; Salesforce Lightning custom home pages can be assigned to apps and profiles so users start with role-relevant workflow entry points.; Microsoft Fluent recommends navigation be task-oriented or feature-oriented depending on product context. |
| D-4 Per-role UX shell primitives | 3 | Apple Human Interface Guidelines provide common system interaction expectations across Apple devices.; Microsoft Fluent Nav requires coherent, short, consistently ordered navigation and accessible secondary actions.; Google Material navigation focuses attention on important content and tasks while supporting in-context movement. |
| D-5 Role-context indicator | 3 | Apple account-driven enrollment shows the Managed Apple Account prominently in Settings after enrollment.; Microsoft sign-in flows route users to the correct service based on account type.; Google Workspace documentation warns that organization accounts may work differently from personal accounts. |
| D-6 Cross-role-switch friction floor | 3 | Microsoft separates personal and work/school account resources so the correct account context matters at sign-in.; Apple supports coexisting managed and personal accounts while preserving data separation.; Google Workspace account behavior can differ under administrator control, making explicit switching semantics necessary. |
| D-7 Per-device profile per role | 3 | Apple account-driven enrollment spans iPhone, iPad, Mac, and Apple Vision Pro device-management flows.; Google Workspace mobile controls include managed work-profile behavior on Android devices.; Material navigation guidance changes structure according to screen and navigation context. |
| D-8 Per-locale UX adaptation per role | 3 | Google Workspace organizational accounts can expose different product settings, requiring locale behavior to respect organizational context.; Salesforce Lightning pages and apps are admin-configurable and commonly localized for enterprise roles.; Apple design guidance treats device and locale conventions as part of system-native interaction quality. |
| D-9 Per-accessibility-profile UX adaptation per role | 3 | W3C WCAG 2.2 is the shared accessibility baseline.; Microsoft Fluent Nav requires hover-only secondary actions to remain reachable for screen-reader, voice-control, eye-gaze, and switch users.; Apple Human Interface Guidelines provide system-native accessibility expectations across Apple devices. |
| D-10 Same-training-transfers invariant | 3 | Microsoft Fluent recommends consistent navigation order across products and surfaces to increase predictability.; Google Material uses common navigation patterns such as drawers, tabs, up navigation, and in-context navigation.; Salesforce Lightning emphasizes familiar navigation while allowing app/profile customization. |
