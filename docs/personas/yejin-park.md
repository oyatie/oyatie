---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 1
persona_slug: yejin-park
persona_name: Yejin Park
primary_role: ICU charge nurse, parent, side-business owner, patient, consumer
primary_collar: pink + green
primary_workspace: clinical + field
skill_tier: mid-level
primary_device: mobile-primary + clinical workstation
locale: KR
audience_type_primary: B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT
microservice_count_authority: 56
community_path: microservices/community/PRD.md
layer_enum_authority: ADR-0105 13-layer canonical enum
related_adrs:
  - ADR-0244
  - ADR-0292
  - ADR-0299
  - ADR-0311
  - ADR-0313
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0317
  - ADR-0318
  - ADR-0319
  - ADR-0320
companion_docs:
  - docs/personas/MASTER-ROSTER-2026-05-21.md
  - docs/standards/documentation-rigor.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
  - microservices/community/PRD.md
journey_range: j001-j150 clinical, family, side-business, audit, marketplace, creator-minor
---

# Persona Dossier — Yejin Park

## §A. Archetype

Yejin Park is priority 01 in the 2026-05-21 oyatie persona roster. The active projection is **ICU charge nurse, parent, side-business owner, patient, consumer**. This dossier treats the persona as a projection of one passkey-bound human, not a separate account.

Career arc:
- Entry: enters oyatie through `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`.
- Growth: gains capability tiers without gaining ambient cross-tenant authority.
- Operational: depends on identity, tenancy, policy-engine, audit-chain, workflow-engine, community.
- Cross-context: personal, professional, regulated, family, and side-business contexts remain separate.
- Recovery: ADR-0299 restores identity without reviving revoked tenant roles.
- Precedent: Apple Personal versus Apple Business and Microsoft personal versus work/school, but enforced through Cedar and tenant IDs.
- Count authority: 56 µservices for this brief.
- Layout authority: ADR-0131 flat per-µservice layout.
- Layer authority: ADR-0105 13-layer canonical enum.
- Community authority: `microservices/community/PRD.md`; no `anonymous/` path.

Tenure and universality:
- Collar-color: `pink + green`.
- Workspace: `clinical + field`.
- Skill tier: `mid-level`.
- Device: `mobile-primary + clinical workstation`.
- Locale: `KR`.
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.

## §B. Device Profile

- Primary device: `mobile-primary + clinical workstation`.
- Active tenant indicator is always visible.
- Device trust binds to WebAuthn, hardware key, or managed-device posture.
- Cached state is partitioned by tenant, role, region, and persona projection.
- Offline mode stores minimum-necessary state only.
- Personal notifications never reveal employer-owned data.
- Employer notifications never reveal personal-tenant data.
- Kiosk and shared-device paths require context confirmation.
- Accessibility profile follows the human but reveals no protected status to tenant admins.
- Telemetry is aggregate and never logs raw accessibility settings.
- Sensitive surfaces never auto-translate without explicit consent.
- Regional degradation follows pack and tenant tier.

## §C. Locale + Tenant Context

- Primary locale: `KR`.
- Audience types: `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`.
- Personal tenant owns consumer Mail, Messenger, Drive, Calendar, Notes, Payments, and personal Workflow state.
- Work or institution tenant owns tenant-produced work artifacts.
- Family tenant is explicit and never inferred from surname.
- Side-business tenant is explicit and never inherits employer permissions.
- Regulator, board, counsel, HR, healthcare, and bank scopes are lawful-scope only.
- Marketplace/community tenant scope is facilitator scope, not broad data access.
- Cross-border transfer requires pack alignment.
- Higher-restriction pack wins during conflict.
- Audit events include identity, tenant, audience type, role, data class, and reason code.
- The persona can switch context only through an explicit active-tenant transition.

## §D. Cross-Context Bridge

- `Yejin-as-nurse` is the same human under another tenant or role projection.
- `Yejin-as-parent` is the same human under another tenant or role projection.
- `Yejin-as-side-business-owner` is the same human under another tenant or role projection.
- `Yejin-as-patient` is the same human under another tenant or role projection.
- `Yejin-as-consumer` is the same human under another tenant or role projection.

Bridge invariants:
- Personal tenant survives work-role revocation.
- Personal account recovery does not revive revoked work grants.
- Work tenant cannot inspect personal surfaces.
- Personal tenant cannot mutate work evidence without explicit grant.
- Search indexes stay context-partitioned.
- Relevance models stay context-partitioned.
- Community membership never backdoors personal data.
- Context switch emits audit evidence.
- Context switch updates the active-tenant indicator.
- Cross-context bridge data supports continuity, not co-mingling.

## §E. Typical Day on oyatie

| Time | Narrative beat | µservices touched | Identity invariant |
|---|---|---|---|
| 05:30 | Yejin Park acts as ICU charge nurse. | identity + tenancy | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Yejin Park acts as ICU charge nurse. | tenancy + policy-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Yejin Park acts as ICU charge nurse. | policy-engine + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Yejin Park acts as ICU charge nurse. | audit-chain + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Yejin Park acts as ICU charge nurse. | workflow-engine + community | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Yejin Park acts as ICU charge nurse. | community + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Yejin Park acts as ICU charge nurse. | messenger + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Yejin Park acts as ICU charge nurse. | mail + calendar | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Yejin Park acts as ICU charge nurse. | calendar + payments | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Yejin Park acts as ICU charge nurse. | payments + marketplace | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Yejin Park acts as ICU charge nurse. | marketplace + finops-portal | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Yejin Park acts as ICU charge nurse. | finops-portal + personal-health-tracker | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Yejin Park acts as ICU charge nurse. | personal-health-tracker + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Yejin Park acts as ICU charge nurse. | compliance + identity | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Yejin Park acts as ICU charge nurse. | identity + tenancy | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Yejin Park acts as ICU charge nurse. | tenancy + policy-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Yejin Park acts as ICU charge nurse. | policy-engine + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, tenancy, policy-engine, audit-chain, workflow-engine, community, messenger, mail.
- Every mutation carries `tenant_id`, `principal_id`, `audience_type`, `data_class`, and `audit_event_class`.
- Every privileged read is server-side Cedar-filtered.
- Delegated agents use `delegated_agent_token`.
- Undo and cool-down apply to high-consequence actions.
- Region outage behavior is pack-bound and tenant-tier-bound.
- No personal/work cache bleed is acceptable.
- The same human can finish the day without re-registering identity.

## §F. Their Needs

- Needs identity continuity without context collapse.
- Needs the shortest safe path through the active workflow.
- Needs clear recovery from denial, lockout, accident, or outage.
- Needs notifications to arrive in the right context.
- Needs pack overlays to apply without architecture knowledge.
- Needs no repeated account setup across tenants.
- Good day: passkey works, correct tenant is obvious, permits are present, audit is silent.
- Bad day: stale grants remain, personal data leaks, lawful scope is too broad, or critical path is over-blocked.
- Worst day: emergency, recovery, audit, legal deadline, child safety, survivor safety, or healthcare urgency fails because of generic friction.
- Product invariant: safety, security, and policy must hold together.

## §G. Useful Adjacencies

Core capability tiers:
- `identity` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `tenancy` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `policy-engine` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `community` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `calendar` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `payments` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `marketplace` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `finops-portal` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `personal-health-tracker` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.
- `compliance` capability tier: Yejin Park needs tenant-scoped read/write/decide posture with audit evidence.

56-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | identity | primary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 02 | tenancy | primary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 03 | policy-engine | primary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 04 | audit-chain | primary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 05 | workflow-engine | primary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 06 | workflow-studio | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 07 | community | primary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 08 | messenger | secondary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 09 | mail | secondary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 10 | calendar | secondary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 11 | meet | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 12 | drive | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 13 | notes | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 14 | forms | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 15 | payments | secondary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 16 | finops-portal | secondary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 17 | marketplace | secondary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 18 | ontology | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 19 | intelligence | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 20 | observability | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 21 | compliance | secondary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 22 | governance | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 23 | ops-dashboard-control-center | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 24 | workplace-integration | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 25 | developer-sdk | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 26 | foundry | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 27 | api-gateway | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 28 | cell | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 29 | cloud-secrets | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 30 | analytics | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 31 | search | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 32 | notifications | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 33 | social | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 34 | shorts | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 35 | ads | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 36 | personal-health-tracker | secondary | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 37 | crm | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 38 | marketing-automation | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 39 | contact-center | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 40 | performance-mgmt | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 41 | learning-mgmt | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 42 | itsm | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 43 | incident-mgmt | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 44 | financial-planning | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 45 | data-warehouse | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 46 | contract-lifecycle-mgmt | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 47 | whiteboard | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 48 | design-collaboration | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 49 | erp-finance | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 50 | erp-procurement | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 51 | erp-inventory | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 52 | erp-manufacturing | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 53 | erp-sales | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 54 | erp-hr | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 55 | erp-projects | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 56 | erp-analytics | ambient | Yejin Park must preserve tenant scope, audit events, and active-context UX if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"yejin-park@active-tenant",
  action in ActionGroup::"yejin-park.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2B_HEALTHCARE_PROVIDER", "B2C_FAMILY_PARENT", "B2B_TENANT_ADMIN", "B2B_HEALTHCARE_PATIENT"] &&
  context.persona_projection == "yejin-park" &&
  context.active_tenant_confirmed == true
};
```

```cedar
forbid (
  principal,
  action,
  resource in Tenant::"personal"
)
unless {
  context.explicit_cross_tenant_grant == true &&
  context.grant_scope_covers_resource == true &&
  context.audit_reason_code != ""
};
```

Permit requirements:
- Audience type derives from ADR-0244.
- Cross-context bridge never implies cross-tenant read.
- Privileged operation requires reason code.
- Regulator, board, counsel, HR, audit, bank, healthcare, education, and law-enforcement scopes are minimum necessary.
- High-value mutation uses undo or cool-down.
- Critical-path exception is audited, not silent.
- Delegated agent traces back to authorizing human.
- Revocation must meet tenant policy budget.
- Cedar fragment publication observes soak requirements.
- Denial copy includes recovery path.
- Community flows use `microservices/community/PRD.md`.
- The deleted `anonymous/` path remains unused.

## §I. Per-Pack Overlay

| Pack | Activation | Effect |
|---|---|---|
| KR-CSAP | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Yejin Park. |
| KR-PIPA | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Yejin Park. |
| KR-Health-Privacy | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Yejin Park. |
| KR-Labor | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Yejin Park. |
| KR-VAT | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Yejin Park. |
| HIPAA-equivalent | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Yejin Park. |
| reserved-pack-07 | Future tenant or vertical overlay. | Cannot weaken ADR-0244 or ADR-0311. |
| reserved-pack-08 | Future tenant or vertical overlay. | Cannot weaken ADR-0244 or ADR-0311. |
| reserved-pack-09 | Future tenant or vertical overlay. | Cannot weaken ADR-0244 or ADR-0311. |
| reserved-pack-10 | Future tenant or vertical overlay. | Cannot weaken ADR-0244 or ADR-0311. |
| reserved-pack-11 | Future tenant or vertical overlay. | Cannot weaken ADR-0244 or ADR-0311. |
| reserved-pack-12 | Future tenant or vertical overlay. | Cannot weaken ADR-0244 or ADR-0311. |

Pack invariants:
- Higher-restriction pack wins.
- Overlay cannot weaken tenant scoping.
- Overlay cannot weaken personal/work separation.
- Overlay can add evidence, retention, data-residency, co-sign, or lawful-basis rules.
- Overlay must be observable.
- Overlay must have rollback behavior.

## §J. Critical-Path Edge Cases (per documentation-rigor.md §3.2.5)

| Row | Critical path | Applicability | Persona handling |
|---:|---|---|---|
| 1 | Emergency services | direct | Yejin Park: audit and attest without blocking life-safety; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 2 | Account recovery / lockout | watch | Yejin Park: passkey backup, recovery code, and trusted contact; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 3 | Financial fraud dispute + chargeback | direct | Yejin Park: PSP-integrated fast-track dispute; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 4 | Elder financial abuse | watch | Yejin Park: cooling-off and trusted-contact alert; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 5 | Healthcare urgent care + EHR break-glass | direct | Yejin Park: post-hoc audit-and-justify; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 6 | Whistleblower + ethics report | watch | Yejin Park: anonymous sealed chain of custody; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 7 | Press freedom / journalist source | direct | Yejin Park: metadata-minimized source protection; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 8 | Domestic violence / abuse survivor | watch | Yejin Park: silent shelter mode; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 9 | Child safety + mandatory reporting | direct | Yejin Park: mandatory-reporter route cannot be suppressed; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 10 | Deceased-user account | watch | Yejin Park: legacy contact plus court-order path; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 11 | Custody / shared-account dispute | direct | Yejin Park: family-court order integration; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 12 | Disability accommodations | watch | Yejin Park: accessibility profile overrides friction defaults; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 13 | Non-native-language user | direct | Yejin Park: sensitive translation requires consent; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 14 | Low-bandwidth / disaster-zone / offline-first | watch | Yejin Park: offline audit retention and sync; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 15 | Banking / financial inclusion | direct | Yejin Park: low-tier financial inclusion path; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 16 | Activist / dissident | watch | Yejin Park: Tor-friendly metadata-minimized mode; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 17 | Regulator-deadline outage | direct | Yejin Park: degraded deadline-preserving workflow; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 18 | Audit / regulator / law-enforcement access | watch | Yejin Park: lawful-scope read-only evidence; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 19 | Tenant break-glass / dead-account recovery | direct | Yejin Park: ombudsman quorum and Shamir recovery; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 20 | Cognitive impairment / post-trauma | watch | Yejin Park: slow-down nudges without autonomy loss; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 21 | Pseudonymous + privacy-by-default | direct | Yejin Park: public identity separated from compliance identity; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 22 | Disaster-zone surge | watch | Yejin Park: cell isolation plus emergency rate floor; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 23 | Cross-jurisdiction conflict | direct | Yejin Park: higher-restriction pack wins; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 24 | Account-hijack victim recovery | watch | Yejin Park: hardware-key recovery and mutation cool-down; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 25 | Mistaken action / unintended mutation | direct | Yejin Park: 15s undo and rare high-value confirmation; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 26 | Concurrent-session conflict | watch | Yejin Park: due-process session conflict handling; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 27 | Bug bounty submitter | direct | Yejin Park: security-researcher allow-list and safe harbor; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 28 | Delegated agent acting for human | watch | Yejin Park: attested delegation chain; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 29 | High-net-worth transaction limits | direct | Yejin Park: KYB-verified transaction tier; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |
| 30 | Regional outage degradation | watch | Yejin Park: DR-pair failover within residency boundary; applies under `B2B_HEALTHCARE_PROVIDER + B2C_FAMILY_PARENT + B2B_TENANT_ADMIN + B2B_HEALTHCARE_PATIENT`. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.

## §K. Their Journey Range

- Primary range: j001-j150 clinical, family, side-business, audit, marketplace, creator-minor.
- j001-j025: onboarding, passkey continuity, personal tenant bootstrap.
- j026-j050: workplace, family, payment, community base flows.
- j051-j075: job, hiring, education, marketplace handshakes.
- j076-j100: healthcare, regulated evidence, compliance, boundary proofs.
- j101-j125: localization, pack overlays, account recovery, region failover.
- j126-j131: government auditor and lawful-scope audit.
- j132-j136: HR, benefits, layoff, staffing, harassment boundary.
- j137-j141: internal audit, fraud, DLP, Cedar misuse.
- j142-j147: layoff employee-side, portfolio import, job pipeline, mutual aid.
- j148-j150: circular economy, gig economy, creator monetization, minor safety.
- Future j151+: ERP migration and capability-tier expansion.
- Each journey must state active tenant.
- Each journey must state audience type.
- Each journey must state Cedar permit class.
- Each journey must state µservices touched.
- Each journey must state pack overlays.
- Each journey must state critical-path rows.
- Each journey must preserve personal/work separation.
- Each journey must state recovery or rollback.
- Each journey must emit audit event class.
- Each journey must be intern-buildable from linked docs.

## §K.1 Substance Anchors — 2026-05-20 Pass

- Place/time: Seoul National University Hospital ICU nurses' station, 101 Daehak-ro, 03:22 KST during monsoon rain after a code-blue handoff.
- Current named tools: Epic Hyperspace May 2026 for MAR and handoff notes, Samsung Galaxy S24 Enterprise with Knox 3.11 for shift alerts, KakaoWork Enterprise 4.8 for ward coordination, and Toss Payments Business 2.1 for soap-order settlement.
- Named pain points: a 2025 code-blue handoff lost 19 minutes when allergy notes lived in a free-text field, a private caregiver deposit was double-charged ₩380,000, and her soap side-business VAT export took 6 hours to separate from family purchases.
- Jobs-to-be-done: weekly ICU shift handoff and family calendar reconciliation; quarterly KR-PIPA/HIPAA chart-access audit for the SNU ward; yearly "SoapCo VAT + family care plan" closeout.
- Cedar binding: principal `User::"yejin-park@snu-hospital-kr"` accesses `PatientChart::*`, `ShiftRoster::*`, `FamilyCalendar::*`, and `SoapOrder::*`; actions `update_handoff`, `request_break_glass_attest`, `schedule_family_care`, and `settle_side_business_order`.
- Cross-context bridges: clinical escalations route to Dr. Tanaka, HR/payroll questions route to Priya Krishnan's tenant-owned HR scope, and family-care exceptions stay separate from Hiroshi Tanaka's elder-care journey.
- Journey IDs: `j01-emergency-911-dispatch`, `j02-healthcare-code-blue-ehr-break-glass`, `j07-deceased-user-inheritance-handoff`, `j09-account-recovery-phishing-resistant`, `j10-account-takeover-SIM-swap-detected`, `j11-disaster-zone-offline-first-sync`, `j14-delegated-llm-agent-acting-for-yejin`, `j18-child-safety-mandatory-reporter`, `j43-healthcare-nurse-patient-handoff`, `j44-healthcare-telemedicine-consultation`, `j46-healthcare-prescription-renewal-workflow`, `j47-healthcare-billing-and-insurance`, `j69-llm-agent-managing-yejins-week`, `j91-us-state-money-transmitter-licensing`.

## §L. References

- ADR-0244 — binding persona roster authority.
- ADR-0292 — binding persona roster authority.
- ADR-0299 — binding persona roster authority.
- ADR-0311 — binding persona roster authority.
- ADR-0313 — binding persona roster authority.
- ADR-0314 — binding persona roster authority.
- ADR-0315 — binding persona roster authority.
- ADR-0316 — binding persona roster authority.
- ADR-0317 — binding persona roster authority.
- ADR-0318 — binding persona roster authority.
- ADR-0319 — binding persona roster authority.
- ADR-0320 — binding persona roster authority.
- documentation-rigor.md §1.1, §1.2, §2, and §3.2.5.
- enterprise-software-coverage-matrix-2026-05-21.md.
- CATALOG-j126-j150-ecosystem.md.
- microservices/community/PRD.md after anonymous fold deletion.
- ADR-0105 13-layer canonical enum.
- ADR-0131 flat per-µservice layout.

## §M. Buildability Ledger
