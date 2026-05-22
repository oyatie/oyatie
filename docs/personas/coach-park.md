---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 18
persona_slug: coach-park
persona_name: Coach Park
roster_display_name: Coach Park
primary_role: Youth soccer coach
primary_collar: pink
primary_workspace: front-office
skill_tier: mid-level
primary_device: mobile-primary
locale: KR
audience_type_primary: EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT
microservice_count_authority: 69
community_path: microservices/community/PRD.md
layer_enum_authority: ADR-0105 13-layer canonical enum
flat_layout_authority: ADR-0131 per-microservice flat layout
cross_context_bridge: |
  Park-as-coach / Park-as-day-job-engineer / Park-as-father
related_adrs:
  - ADR-0244
  - ADR-0292
  - ADR-0299
  - ADR-0311
  - ADR-0313
  - ADR-0316
  - ADR-0317
  - ADR-0318
  - ADR-0319
  - ADR-0320
  - ADR-0321
companion_docs:
  - docs/personas/MASTER-ROSTER-2026-05-21.md
  - docs/standards/documentation-rigor.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
  - microservices/community/PRD.md
journey_range: j001-j150 plus future j151+
---

# Persona Dossier — Coach Park

## §A. Archetype

Coach Park is roster row 18 in the 2026-05-21 oyatie persona graph. The active projection is **Youth soccer coach**. This dossier treats the persona as one passkey-bound human projected through tenant, role, workspace, locale, device, and skill-tier context.

Career arc:
- Entry context: `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Collar-color posture: `pink`.
- Workspace posture: `front-office`.
- Tenure and skill tier: `mid-level`.
- The person can gain new capability tiers without gaining ambient cross-tenant authority.
- The career arc is modeled as role projection, not account multiplication.
- Personal, professional, regulated, family, and side-business contexts remain tenant-separated.
- ADR-0299 restores the identity root without reviving revoked role grants.
- ADR-0311 blocks work/personal data collapse even when UX is unified.
- ADR-0317 makes role projection the explicit doctrine for this dossier.
- ADR-0318 makes collar-color universality part of the persona model.
- ADR-0319 binds front/middle/back/field/clinical/executive workspace semantics.
- ADR-0320 binds apprentice, intern, resident, fellow, junior, senior, and executive tiers.
- ADR-0321 keeps B2B leader scaffolds as capability tiers over the same substrate.
- Precedent: Apple personal versus Apple Business and Microsoft personal versus work/school, but enforced through Cedar and tenant IDs.
- Count authority: 69 µservices after B2B-leader scaffolds.
- Layout authority: ADR-0131 flat per-µservice layout.
- Community authority: `microservices/community/PRD.md`; no `anonymous/` path.

Tenure and universality:
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to the active tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.
- The active tenant indicator is part of the product contract.
- Role revocation must not damage personal tenant continuity.
- Cross-context bridge data supports continuity, not co-mingling.

## §B. Device Profile

- Primary device: `mobile-primary`.
- Secondary device: mobile-primary fallback with passkey and tenant-aware notification gating.
- Accessibility profile follows the human but is not disclosed to tenant admins as protected status.
- Device trust binds to WebAuthn, hardware key, managed-device posture, or delegated trusted-contact recovery.
- Cached state is partitioned by tenant, role, region, and persona projection.
- Offline mode stores minimum-necessary state only and emits reconciliation evidence on sync.
- Personal notifications never reveal employer-owned data.
- Employer notifications never reveal personal-tenant data.
- Kiosk, vehicle, rugged, or shared-device paths require explicit context confirmation when applicable.
- Screen-reader, voice, switch-control, large-text, and low-bandwidth modes are first-class profile flags.
- Sensitive surfaces never auto-translate without explicit consent.
- Regional degradation follows pack and tenant tier.
- Device loss triggers tenant-scoped revocation, not global identity destruction.
- Every device switch reaffirms the active-tenant banner.

## §C. Locale + Tenant Context

- Primary locale: `KR`.
- Audience types: `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Personal tenant owns consumer Mail, Messenger, Drive, Calendar, Notes, Payments, and personal Workflow state.
- Work or institution tenant owns tenant-produced work artifacts.
- Family tenant is explicit and never inferred from surname, device sharing, or social graph proximity.
- Side-business tenant is explicit and never inherits employer permissions.
- Regulator, board, counsel, HR, healthcare, education, bank, and field scopes are lawful-scope only.
- Marketplace and community tenant scope is facilitator scope, not broad data access.
- Cross-border transfer requires pack alignment.
- Higher-restriction pack wins during conflict.
- Audit events include identity, tenant, audience type, role, data class, and reason code.
- The persona can switch context only through an explicit active-tenant transition.
- Tenant membership is revocable without deleting the root identity.
- Tenant grants are observable, expire where policy requires, and are never inferred from contact lists.

## §D. Cross-Context Bridge

- `Park-as-coach` is the same human under another tenant or role projection.
- `Park-as-day-job-engineer` is the same human under another tenant or role projection.
- `Park-as-father` is the same human under another tenant or role projection.

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
- Any bridge expansion must update both source and target dossiers.
- Cross-tenant reads require explicit lawful basis and Cedar permit scope.

## §E. Typical Day on oyatie

| Time | Narrative beat | µservices touched | Capability tiers active | Identity invariant |
|---|---|---|---|---|
| 05:30 | Coach Park acts as Youth soccer coach in the active context. | identity + tenancy | read within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Coach Park acts as Youth soccer coach in the active context. | tenancy + policy-engine | write within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Coach Park acts as Youth soccer coach in the active context. | policy-engine + audit-chain | decide within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Coach Park acts as Youth soccer coach in the active context. | audit-chain + workflow-engine | delegate within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Coach Park acts as Youth soccer coach in the active context. | workflow-engine + community | read within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Coach Park acts as Youth soccer coach in the active context. | community + messenger | write within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Coach Park acts as Youth soccer coach in the active context. | messenger + mail | decide within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Coach Park acts as Youth soccer coach in the active context. | mail + calendar | delegate within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Coach Park acts as Youth soccer coach in the active context. | calendar + learning-management | read within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Coach Park acts as Youth soccer coach in the active context. | learning-management + forms | write within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Coach Park acts as Youth soccer coach in the active context. | forms + drive | decide within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Coach Park acts as Youth soccer coach in the active context. | drive + meet | delegate within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Coach Park acts as Youth soccer coach in the active context. | meet + identity | read within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Coach Park acts as Youth soccer coach in the active context. | identity + tenancy | write within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Coach Park acts as Youth soccer coach in the active context. | tenancy + policy-engine | decide within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Coach Park acts as Youth soccer coach in the active context. | policy-engine + audit-chain | delegate within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Coach Park acts as Youth soccer coach in the active context. | audit-chain + workflow-engine | read within `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, tenancy, policy-engine, audit-chain, workflow-engine, community, messenger, mail.
- Every mutation carries `tenant_id`, `principal_id`, `audience_type`, `data_class`, and `audit_event_class`.
- Every privileged read is server-side Cedar-filtered.
- Delegated agents use `delegated_agent_token` and trace back to the authorizing human.
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
- Needs lawful-scope data access that is explainable to non-specialists.
- Needs screen-reader, low-bandwidth, mobile, kiosk, rugged, or desktop parity according to the device profile.
- Needs fast denial recovery that names the missing permit rather than leaking protected data.
- Needs tenant switching that is obvious enough to prevent accidental disclosure.
- Needs audit evidence that helps compliance without creating surveillance sprawl.
- Needs delegated-agent behavior that stays inside the human grant.
- Needs good-day flow: passkey works, correct tenant is obvious, permits are present, audit is silent.
- Needs bad-day flow: stale grants remain, personal data leaks, lawful scope is too broad, or critical path is over-blocked.
- Needs worst-day flow: emergency, recovery, audit, legal deadline, child safety, survivor safety, or healthcare urgency fails because of generic friction.
- Product invariant: safety, security, and policy must hold together.

## §G. Useful Adjacencies

Core capability tiers:
- `identity` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `tenancy` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `policy-engine` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `community` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `calendar` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `learning-management` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `forms` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `drive` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.
- `meet` capability tier: Coach Park needs tenant-scoped read/write/decide posture with audit evidence.

69-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | analytics | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 02 | api-gateway | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 03 | application | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 04 | audit-chain | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 05 | calendar | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 06 | cell | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 07 | cloud-iac | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 08 | cloud-k8s | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 09 | cloud-secrets | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 10 | comms-email | secondary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 11 | community | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 12 | compliance | secondary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 13 | connect | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 14 | consent-graph | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 15 | contact-center | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 16 | contract-lifecycle-management | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 17 | crm | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 18 | data-pipeline | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 19 | data-warehouse | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 20 | design-collaboration | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 21 | detection | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 22 | developer-sdk | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 23 | docs | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 24 | drive | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 25 | feature-flags | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 26 | financial-planning | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 27 | finops-portal | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 28 | forms | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 29 | foundry | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 30 | global-trade | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 31 | governance | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 32 | healthcare-integration | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 33 | identity | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 34 | incident-management | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 35 | intelligence | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 36 | itsm | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 37 | learning-management | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 38 | mail | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 39 | marketing-automation | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 40 | marketplace | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 41 | meet | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 42 | messenger | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 43 | network | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 44 | notes | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 45 | observability | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 46 | ontology | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 47 | ops-dashboard-control-center | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 48 | payments | secondary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 49 | performance-management | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 50 | plant-maintenance | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 51 | plugin-app-store | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 52 | production-planning | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 53 | quality-management | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 54 | real-estate | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 55 | recordings | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 56 | sheets | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 57 | shorts | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 58 | sites | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 59 | slides | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 60 | social | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 61 | supply-chain-planning | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 62 | tasks | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 63 | tenancy | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 64 | translate | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 65 | treasury | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 66 | warehouse | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 67 | whiteboard | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 68 | workflow-engine | primary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 69 | workflow-studio | secondary | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 70 | workplace-integration | ambient | Coach Park must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"coach-park@active-tenant",
  action in ActionGroup::"coach-park.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["EDU_TEACHER", "B2C_CONSUMER", "B2C_FAMILY_PARENT"] &&
  context.persona_projection == "coach-park" &&
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
- Regulator, board, counsel, HR, audit, bank, healthcare, education, field, and law-enforcement scopes are minimum necessary.
- High-value mutation uses undo or cool-down.
- Critical-path exception is audited, not silent.
- Delegated agent traces back to authorizing human.
- Revocation must meet tenant policy budget.
- Cedar fragment publication observes soak requirements.
- Denial copy includes recovery path.
- Community flows use `microservices/community/PRD.md`.
- The deleted anonymous path remains unused.

## §I. Per-Pack Overlay Applicable

| Pack | Activation | Effect |
|---|---|---|
| KR-CSAP | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Coach Park. |
| KR-PIPA | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Coach Park. |
| KR-Labor | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Coach Park. |
| DPDP-2023 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Coach Park. |
| FERPA / education pack | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Coach Park. |
| reserved-pack-06 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Coach Park. |
| reserved-pack-07 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Coach Park. |
| reserved-pack-08 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Coach Park. |
| reserved-pack-09 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Coach Park. |
| reserved-pack-10 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Coach Park. |

Pack invariants:
- Higher-restriction pack wins.
- Overlay cannot weaken tenant scoping.
- Overlay cannot weaken personal/work separation.
- Overlay can add evidence, retention, data-residency, co-sign, or lawful-basis rules.
- Overlay must be observable.
- Overlay must have rollback behavior.
- Overlay expansion must cite its binding ADR or registry row.
- Pack conflict resolution emits audit evidence.

## §J. Critical-Path Edge Cases Applicable (per documentation-rigor.md §3.2.5)

| Row | Critical path | Applicability | Persona handling |
|---:|---|---|---|
| 1 | Emergency services | direct | Coach Park: audit-and-attest without blocking life safety; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 2 | Account recovery / lockout | direct | Coach Park: passkey backup, recovery code, delegated trusted contact, and ombudsman path; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 3 | Financial fraud dispute + chargeback | direct | Coach Park: PSP-timed fast-track dispute and audit evidence; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 4 | Elder financial abuse | direct | Coach Park: cooling-off and trusted-contact alert without removing autonomy; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 5 | Healthcare urgent care + EHR break-glass | direct | Coach Park: post-hoc audit-and-justify for treatment urgency; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 6 | Whistleblower + ethics report | watch | Coach Park: sealed chain of custody and tenant-admin blindness; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 7 | Press freedom / journalist source | direct | Coach Park: metadata-minimized protected-source mode; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 8 | Domestic violence / abuse survivor | direct | Coach Park: silent shelter mode and safe-device handling; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 9 | Child safety + mandatory reporting | direct | Coach Park: mandatory-reporter route cannot be suppressed; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 10 | Deceased-user account | direct | Coach Park: legacy-contact and court-order ingress; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 11 | Custody / shared-account dispute | direct | Coach Park: family-court order integration and child-best-interest default; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 12 | Disability accommodations | direct | Coach Park: accessibility profile overrides friction defaults; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 13 | Non-native-language user | direct | Coach Park: consented translation on sensitive surfaces; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 14 | Low-bandwidth / disaster-zone / offline-first | direct | Coach Park: offline audit retention and conflict-safe sync; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 15 | Banking / financial inclusion | direct | Coach Park: low-tier inclusion path with regulator floor; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 16 | Activist / dissident | direct | Coach Park: Tor-friendly metadata-minimized mode; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 17 | Regulator-deadline outage | watch | Coach Park: deadline-preserving degraded workflow; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 18 | Audit / regulator / law-enforcement access | watch | Coach Park: lawful-scope read-only evidence; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 19 | Tenant break-glass / dead-account recovery | watch | Coach Park: ombudsman quorum and Shamir recovery; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 20 | Cognitive impairment / post-trauma | direct | Coach Park: slow-down nudges without autonomy loss; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 21 | Pseudonymous + privacy-by-default | direct | Coach Park: public identity separated from compliance identity; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 22 | Disaster-zone surge | direct | Coach Park: cell isolation plus emergency rate floor; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 23 | Cross-jurisdiction conflict | direct | Coach Park: higher-restriction pack wins; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 24 | Account-hijack victim recovery | direct | Coach Park: hardware-key recovery and mutation cool-down; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 25 | Mistaken action / unintended mutation | direct | Coach Park: undo and rare high-value confirmation; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 26 | Concurrent-session conflict | direct | Coach Park: due-process conflict handling; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 27 | Bug bounty submitter | watch | Coach Park: safe-harbor submission path; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 28 | Delegated agent acting for human | direct | Coach Park: attested delegation chain; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 29 | High-net-worth transaction limits | direct | Coach Park: KYB-verified transaction tier; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |
| 30 | Regional outage degradation | direct | Coach Park: DR-pair failover within residency boundary; applies under `EDU_TEACHER + B2C_CONSUMER + B2C_FAMILY_PARENT` with active tenant scope. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.
- Regional failover respects data-residency hard stops.
- Delegated agents inherit only the explicit human grant.
- Every critical-path row names a recovery or rollback posture.

## §K. Their Journey Range

- Primary range: j001-j150 plus future j151+ capability-tier expansion.
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
- Future j151+: ERP, B2B leader scaffolds, and capability-tier expansion over the 69-µservice catalog.
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

- Place/time: Seoul World Cup Stadium auxiliary field, Sangam-dong, 19:05 KST on a windy Tuesday after middle-school practice.
- Current named tools: SportsEngine HQ Premium 2026.1 for roster/waivers, KakaoTalk Channel Manager 3.5 for parent notices, Naver Calendar 2026.04 for field bookings, and Samsung Galaxy S23 FE Knox 3.11 for photo-consent capture.
- Named pain points: spring tournament `U13-HanRiver-2026` used an expired photo waiver for 11 players, cost ₩1.2m in refund credits, and took 6h to separate parent chats from school-owned records.
- Jobs-to-be-done: weekly practice attendance and injury-note review; quarterly child-safety OKR "100% current waivers before travel"; yearly league registration, insurance, and parent-volunteer audit.
- Cedar binding: principal `User::"coach-park@hanriver-youth-soccer-kr"` accesses `PlayerRoster::*`, `GuardianConsent::*`, `InjuryNote::*`, and `FieldBooking::*`; actions `mark_attendance`, `request_parent_consent`, `file_injury_note`, and `reserve_field`.
- Cross-context bridges: child-safety escalation borrows Yejin Park's mandatory-reporter pattern, school coordination aligns with Ms. Patel, and cafeteria-allergy travel meals can route through Soyeon Kim's food-service controls.
- Journey IDs: `j18-child-safety-mandatory-reporter`, `j27-calendar-cross-context-family-and-work`, `j161-cafeteria-soyeon-kim-allergen-recall-and-school-coordination`.

## §L. References

- ADR-0244 — audience_type and tenant/user boundary authority.
- ADR-0292 — minor and vulnerable-user doctrine as applicable.
- ADR-0299 — account recovery and passkey-bound identity.
- ADR-0311 — dual personal/work tenant boundary.
- ADR-0313 — conglomerate tenant hierarchy and child-tenant sovereignty.
- ADR-0316 — capability-tier activation over shared substrates.
- ADR-0317 — role-projection doctrine.
- ADR-0318 — collar-color universality.
- ADR-0319 — front/middle/back/field/clinical/executive workspace axis.
- ADR-0320 — apprentice, intern, resident, fellow, and tenure-tier model.
- ADR-0321 — B2B leader scaffolds and post-B2B-leader catalog expansion.
- ADR-0131 — flat per-µservice layout.
- ADR-0105 — 13-layer canonical enum.
- documentation-rigor.md §1.1, §1.2, §2, and §3.2.5.
- enterprise-software-coverage-matrix-2026-05-21.md.
- CATALOG-j126-j150-ecosystem.md.
- microservices/community/PRD.md after anonymous fold deletion.

## §M. Buildability Ledger

- A cold intern can identify who the persona is, what tenant context is active, and why the same human may appear in multiple projections.
- The device section states primary, secondary, accessibility, and recovery implications.
- The locale section states audience type, tenant membership, and cross-border pack behavior.
- The bridge section enumerates same-human projections from the roster.
- The day-in-life section names µservices and capability tiers by hour.
- The needs section distinguishes good day, bad day, and worst day outcomes.
- The adjacency section maps this persona onto all 69 µservices.
- The Cedar section gives permit and forbid shapes instead of generic auth prose.
- The pack section declares jurisdictional and sector overlays.
- The critical-path section covers documentation-rigor.md §3.2.5 rows 1-30.
- The journey section spans j001-j150 and future j151+.
- The dossier cites the active ADR bundle required by the brief.
- The dossier uses `microservices/community/PRD.md` for community context.
- The dossier does not recreate the deleted anonymous path.
- The dossier assumes flat per-µservice layout per ADR-0131.
- The dossier keeps audience type contextual rather than biological.
- The dossier keeps role revocation separate from identity recovery.
- The dossier treats delegated agents as scoped grants.
- The dossier makes denial, recovery, rollback, and audit visible.
- The dossier is additive and does not modify ADRs, standards, synthesis, or existing dossiers.
