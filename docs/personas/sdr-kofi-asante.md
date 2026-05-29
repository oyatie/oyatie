---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 51
persona_slug: sdr-kofi-asante
persona_name: SDR Kofi Asante
roster_display_name: SDR Kofi Asante
primary_role: Sales Development Rep
primary_collar: white
primary_workspace: front-office
skill_tier: junior
primary_device: desktop + mobile
locale: GH
audience_type_primary: B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)
microservice_count_authority: 69
community_path: microservices/community/PRD.md
layer_enum_authority: ADR-0105 13-layer canonical enum
flat_layout_authority: ADR-0131 per-microservice flat layout
cross_context_bridge: |
  Kofi-as-SDR / Kofi-as-side-hustle-creator
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

# Persona Dossier — SDR Kofi Asante

## §A. Archetype

SDR Kofi Asante is roster row 51 in the 2026-05-21 oyatie persona graph. The active projection is **Sales Development Rep**. This dossier treats the persona as one passkey-bound human projected through tenant, role, workspace, locale, device, and skill-tier context.

Career arc:
- Entry context: `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)`.
- Collar-color posture: `white`.
- Workspace posture: `front-office`.
- Tenure and skill tier: `junior`.
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

- Primary device: `desktop + mobile`.
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

- Primary locale: `GH`.
- Audience types: `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)`.
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

- `Kofi-as-SDR` is the same human under another tenant or role projection.
- `Kofi-as-side-hustle-creator` is the same human under another tenant or role projection.

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
| 05:30 | SDR Kofi Asante acts as Sales Development Rep in the active context. | identity + tenancy | read within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | SDR Kofi Asante acts as Sales Development Rep in the active context. | tenancy + policy-engine | write within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | policy-engine + audit-chain | decide within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | audit-chain + workflow-engine | delegate within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | workflow-engine + community | read within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | community + messenger | write within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | messenger + mail | decide within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | mail + calendar | delegate within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | calendar + crm | read within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | crm + marketing-automation | write within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | marketing-automation + contact-center | decide within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | contact-center + marketplace | delegate within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | marketplace + analytics | read within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | analytics + identity | write within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | identity + tenancy | decide within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | SDR Kofi Asante acts as Sales Development Rep in the active context. | tenancy + policy-engine | delegate within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | SDR Kofi Asante acts as Sales Development Rep in the active context. | policy-engine + audit-chain | read within `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` | Active tenant banner is visible and Cedar evaluates before mutation. |

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
- `identity` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `tenancy` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `policy-engine` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `community` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `calendar` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `crm` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `marketing-automation` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `contact-center` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `marketplace` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.
- `analytics` capability tier: SDR Kofi Asante needs tenant-scoped read/write/decide posture with audit evidence.

69-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | analytics | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 02 | api-gateway | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 03 | application | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 04 | audit-chain | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 05 | calendar | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 06 | cell | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 07 | cloud-iac | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 08 | cloud-k8s | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 09 | cloud-secrets | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 10 | comms-email | secondary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 11 | community | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 12 | compliance | secondary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 13 | connector | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 14 | consent-graph | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 15 | contact-center | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 16 | contract-lifecycle-management | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 17 | crm | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 18 | data-pipeline | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 19 | data-warehouse | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 20 | design-collaboration | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 21 | detection | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 22 | developer-sdk | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 23 | docs | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 24 | drive | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 25 | feature-flags | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 26 | financial-planning | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 27 | finops-portal | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 28 | forms | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 29 | foundry | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 30 | global-trade | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 31 | governance | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 32 | healthcare-integration | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 33 | identity | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 34 | incident-management | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 35 | intelligence | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 36 | itsm | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 37 | learning-management | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 38 | mail | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 39 | marketing-automation | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 40 | marketplace | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 41 | meet | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 42 | messenger | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 43 | network | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 44 | notes | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 45 | observability | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 46 | ontology | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 47 | ops-dashboard-control-center | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 48 | payments | secondary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 49 | performance-management | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 50 | plant-maintenance | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 51 | plugin-app-store | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 52 | production-planning | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 53 | quality-management | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 54 | real-estate | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 55 | recordings | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 56 | sheets | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 57 | shorts | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 58 | sites | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 59 | slides | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 60 | social | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 61 | supply-chain-planning | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 62 | tasks | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 63 | tenancy | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 64 | translate | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 65 | treasury | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 66 | warehouse | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 67 | whiteboard | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 68 | workflow-engine | primary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 69 | workflow-studio | secondary | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 70 | workplace-integration | ambient | SDR Kofi Asante must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"sdr-kofi-asante@active-tenant",
  action in ActionGroup::"sdr-kofi-asante.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2B_EMPLOYEE", "B2C_CONSUMER", "B2C_JOB_SEEKER_ACTIVE (passive)"] &&
  context.persona_projection == "sdr-kofi-asante" &&
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
| GDPR | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for SDR Kofi Asante. |
| EU-AI-Act | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for SDR Kofi Asante. |
| DORA-where-financial | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for SDR Kofi Asante. |
| NDPR / Africa regional privacy | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for SDR Kofi Asante. |
| reserved-pack-05 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for SDR Kofi Asante. |
| reserved-pack-06 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for SDR Kofi Asante. |
| reserved-pack-07 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for SDR Kofi Asante. |
| reserved-pack-08 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for SDR Kofi Asante. |
| reserved-pack-09 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for SDR Kofi Asante. |
| reserved-pack-10 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for SDR Kofi Asante. |

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
| 1 | Emergency services | watch | SDR Kofi Asante: audit-and-attest without blocking life safety; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 2 | Account recovery / lockout | direct | SDR Kofi Asante: passkey backup, recovery code, delegated trusted contact, and ombudsman path; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 3 | Financial fraud dispute + chargeback | direct | SDR Kofi Asante: PSP-timed fast-track dispute and audit evidence; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 4 | Elder financial abuse | direct | SDR Kofi Asante: cooling-off and trusted-contact alert without removing autonomy; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 5 | Healthcare urgent care + EHR break-glass | watch | SDR Kofi Asante: post-hoc audit-and-justify for treatment urgency; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 6 | Whistleblower + ethics report | watch | SDR Kofi Asante: sealed chain of custody and tenant-admin blindness; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 7 | Press freedom / journalist source | direct | SDR Kofi Asante: metadata-minimized protected-source mode; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 8 | Domestic violence / abuse survivor | direct | SDR Kofi Asante: silent shelter mode and safe-device handling; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 9 | Child safety + mandatory reporting | watch | SDR Kofi Asante: mandatory-reporter route cannot be suppressed; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 10 | Deceased-user account | watch | SDR Kofi Asante: legacy-contact and court-order ingress; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 11 | Custody / shared-account dispute | watch | SDR Kofi Asante: family-court order integration and child-best-interest default; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 12 | Disability accommodations | direct | SDR Kofi Asante: accessibility profile overrides friction defaults; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 13 | Non-native-language user | direct | SDR Kofi Asante: consented translation on sensitive surfaces; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 14 | Low-bandwidth / disaster-zone / offline-first | direct | SDR Kofi Asante: offline audit retention and conflict-safe sync; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 15 | Banking / financial inclusion | direct | SDR Kofi Asante: low-tier inclusion path with regulator floor; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 16 | Activist / dissident | direct | SDR Kofi Asante: Tor-friendly metadata-minimized mode; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 17 | Regulator-deadline outage | watch | SDR Kofi Asante: deadline-preserving degraded workflow; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 18 | Audit / regulator / law-enforcement access | watch | SDR Kofi Asante: lawful-scope read-only evidence; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 19 | Tenant break-glass / dead-account recovery | watch | SDR Kofi Asante: ombudsman quorum and Shamir recovery; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 20 | Cognitive impairment / post-trauma | watch | SDR Kofi Asante: slow-down nudges without autonomy loss; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 21 | Pseudonymous + privacy-by-default | direct | SDR Kofi Asante: public identity separated from compliance identity; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 22 | Disaster-zone surge | watch | SDR Kofi Asante: cell isolation plus emergency rate floor; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 23 | Cross-jurisdiction conflict | direct | SDR Kofi Asante: higher-restriction pack wins; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 24 | Account-hijack victim recovery | direct | SDR Kofi Asante: hardware-key recovery and mutation cool-down; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 25 | Mistaken action / unintended mutation | direct | SDR Kofi Asante: undo and rare high-value confirmation; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 26 | Concurrent-session conflict | watch | SDR Kofi Asante: due-process conflict handling; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 27 | Bug bounty submitter | watch | SDR Kofi Asante: safe-harbor submission path; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 28 | Delegated agent acting for human | direct | SDR Kofi Asante: attested delegation chain; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 29 | High-net-worth transaction limits | direct | SDR Kofi Asante: KYB-verified transaction tier; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |
| 30 | Regional outage degradation | direct | SDR Kofi Asante: DR-pair failover within residency boundary; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_JOB_SEEKER_ACTIVE (passive)` with active tenant scope. |

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

## §K.1 Substance Anchors — 2026-05-20 Pass

- Place/time: Osu sales pod, Accra, 15:10 GMT under Harmattan haze before the EMEA pipeline standup.
- Current named tools: Salesforce Sales Cloud Unlimited Spring '26 leads, Outreach Enterprise 2026.1 sequences, ZoomInfo Elite 2026.2 enrichment, LinkedIn Sales Navigator Advanced Plus, Gong Engage 2026.05 call coaching, and oyatie `crm` prospect-consent ledger.
- Named pain points: the "Fintech 400" sequence double-emailed 1,920 suppressed prospects, wasted $14,800 in SDR credits, and made Kofi spend 6h reconciling Outreach opt-outs with Salesforce campaign members.
- Jobs-to-be-done: daily account research blocks, weekly OKR SDR-GH-01 "25 qualified meetings with zero suppressed sends", Project Cocoa outbound sprint, and quarterly handoff hygiene review with Sales Manager Anthony Costa.
- Cedar binding: principal `User::"kofi-asante@sales-gh"` accesses `Lead::*`, `SequenceStep::*`, `CallRecording::*`, and `SuppressionList::*`; actions `enroll_prospect`, `log_call`, `request_ae_handoff`, and `deny_suppressed_contact_outreach`.
- Cross-context bridges: reports to Anthony Costa, feeds Sales AE Maya Lindqvist, consumes Akemi Sato customer proof, and keeps Kofi-as-side-hustle-creator separate from employer prospect lists.
- Journey IDs: `j100-pack-rollout-from-tenant-onboarding-to-first-action`, `j115-saas-vendor-sells-api-to-multiple-tenant-customers`, `j123-multi-tenant-coordinated-product-launch`, `j145-laid-off-applies-via-community-handshake-linkedin-mode`.
