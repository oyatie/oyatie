---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 65
persona_slug: security-analyst-anna-petrova
persona_name: Security Analyst Anna Petrova
roster_display_name: Security Analyst Anna Petrova
primary_role: Security Analyst
primary_collar: white
primary_workspace: middle-office
skill_tier: mid-level
primary_device: desktop + air-gapped-mobile
locale: RU
audience_type_primary: B2B_EMPLOYEE + B2C_CONSUMER
microservice_count_authority: 69
community_path: microservices/community/PRD.md
layer_enum_authority: ADR-0105 13-layer canonical enum
flat_layout_authority: ADR-0131 per-microservice flat layout
cross_context_bridge: |
  Anna-as-analyst / Anna-as-CTF-player
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

# Persona Dossier — Security Analyst Anna Petrova

## §A. Archetype

Security Analyst Anna Petrova is roster row 65 in the 2026-05-21 oyatie persona graph. The active projection is **Security Analyst**. This dossier treats the persona as one passkey-bound human projected through tenant, role, workspace, locale, device, and skill-tier context.

Career arc:
- Entry context: `B2B_EMPLOYEE + B2C_CONSUMER`.
- Collar-color posture: `white`.
- Workspace posture: `middle-office`.
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

- Primary device: `desktop + air-gapped-mobile`.
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

- Primary locale: `RU`.
- Audience types: `B2B_EMPLOYEE + B2C_CONSUMER`.
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

- `Anna-as-analyst` is the same human under another tenant or role projection.
- `Anna-as-CTF-player` is the same human under another tenant or role projection.

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
| 05:30 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | identity + tenancy | read within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | tenancy + policy-engine | write within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | policy-engine + audit-chain | decide within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | audit-chain + workflow-engine | delegate within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | workflow-engine + community | read within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | community + messenger | write within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | messenger + mail | decide within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | mail + calendar | delegate within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | calendar + developer-sdk | read within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | developer-sdk + foundry | write within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | foundry + observability | decide within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | observability + data-pipeline | delegate within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | data-pipeline + intelligence | read within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | intelligence + identity | write within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | identity + tenancy | decide within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | tenancy + policy-engine | delegate within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Security Analyst Anna Petrova acts as Security Analyst in the active context. | policy-engine + audit-chain | read within `B2B_EMPLOYEE + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |

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
- `identity` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `tenancy` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `policy-engine` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `community` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `calendar` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `developer-sdk` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `foundry` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `observability` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `data-pipeline` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.
- `intelligence` capability tier: Security Analyst Anna Petrova needs tenant-scoped read/write/decide posture with audit evidence.

69-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | analytics | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 02 | api-gateway | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 03 | application | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 04 | audit-chain | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 05 | calendar | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 06 | cell | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 07 | cloud-iac | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 08 | cloud-k8s | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 09 | cloud-secrets | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 10 | comms-email | secondary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 11 | community | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 12 | compliance | secondary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 13 | connect | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 14 | consent-graph | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 15 | contact-center | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 16 | contract-lifecycle-management | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 17 | crm | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 18 | data-pipeline | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 19 | data-warehouse | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 20 | design-collaboration | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 21 | detection | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 22 | developer-sdk | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 23 | docs | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 24 | drive | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 25 | feature-flags | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 26 | financial-planning | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 27 | finops-portal | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 28 | forms | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 29 | foundry | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 30 | global-trade | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 31 | governance | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 32 | healthcare-integration | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 33 | identity | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 34 | incident-management | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 35 | intelligence | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 36 | itsm | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 37 | learning-management | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 38 | mail | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 39 | marketing-automation | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 40 | marketplace | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 41 | meet | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 42 | messenger | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 43 | network | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 44 | notes | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 45 | observability | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 46 | ontology | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 47 | ops-dashboard-control-center | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 48 | payments | secondary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 49 | performance-management | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 50 | plant-maintenance | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 51 | plugin-app-store | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 52 | production-planning | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 53 | quality-management | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 54 | real-estate | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 55 | recordings | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 56 | sheets | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 57 | shorts | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 58 | sites | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 59 | slides | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 60 | social | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 61 | supply-chain-planning | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 62 | tasks | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 63 | tenancy | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 64 | translate | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 65 | treasury | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 66 | warehouse | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 67 | whiteboard | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 68 | workflow-engine | primary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 69 | workflow-studio | secondary | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 70 | workplace-integration | ambient | Security Analyst Anna Petrova must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"security-analyst-anna-petrova@active-tenant",
  action in ActionGroup::"security-analyst-anna-petrova.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2B_EMPLOYEE", "B2C_CONSUMER"] &&
  context.persona_projection == "security-analyst-anna-petrova" &&
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
| GDPR | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Security Analyst Anna Petrova. |
| EU-AI-Act | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Security Analyst Anna Petrova. |
| DORA-where-financial | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Security Analyst Anna Petrova. |
| reserved-pack-04 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Security Analyst Anna Petrova. |
| reserved-pack-05 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Security Analyst Anna Petrova. |
| reserved-pack-06 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Security Analyst Anna Petrova. |
| reserved-pack-07 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Security Analyst Anna Petrova. |
| reserved-pack-08 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Security Analyst Anna Petrova. |
| reserved-pack-09 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Security Analyst Anna Petrova. |
| reserved-pack-10 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for Security Analyst Anna Petrova. |

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
| 1 | Emergency services | watch | Security Analyst Anna Petrova: audit-and-attest without blocking life safety; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 2 | Account recovery / lockout | direct | Security Analyst Anna Petrova: passkey backup, recovery code, delegated trusted contact, and ombudsman path; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 3 | Financial fraud dispute + chargeback | direct | Security Analyst Anna Petrova: PSP-timed fast-track dispute and audit evidence; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 4 | Elder financial abuse | direct | Security Analyst Anna Petrova: cooling-off and trusted-contact alert without removing autonomy; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 5 | Healthcare urgent care + EHR break-glass | watch | Security Analyst Anna Petrova: post-hoc audit-and-justify for treatment urgency; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 6 | Whistleblower + ethics report | direct | Security Analyst Anna Petrova: sealed chain of custody and tenant-admin blindness; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 7 | Press freedom / journalist source | direct | Security Analyst Anna Petrova: metadata-minimized protected-source mode; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 8 | Domestic violence / abuse survivor | direct | Security Analyst Anna Petrova: silent shelter mode and safe-device handling; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 9 | Child safety + mandatory reporting | watch | Security Analyst Anna Petrova: mandatory-reporter route cannot be suppressed; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 10 | Deceased-user account | watch | Security Analyst Anna Petrova: legacy-contact and court-order ingress; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 11 | Custody / shared-account dispute | watch | Security Analyst Anna Petrova: family-court order integration and child-best-interest default; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 12 | Disability accommodations | direct | Security Analyst Anna Petrova: accessibility profile overrides friction defaults; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 13 | Non-native-language user | direct | Security Analyst Anna Petrova: consented translation on sensitive surfaces; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 14 | Low-bandwidth / disaster-zone / offline-first | direct | Security Analyst Anna Petrova: offline audit retention and conflict-safe sync; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 15 | Banking / financial inclusion | direct | Security Analyst Anna Petrova: low-tier inclusion path with regulator floor; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 16 | Activist / dissident | direct | Security Analyst Anna Petrova: Tor-friendly metadata-minimized mode; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 17 | Regulator-deadline outage | direct | Security Analyst Anna Petrova: deadline-preserving degraded workflow; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 18 | Audit / regulator / law-enforcement access | direct | Security Analyst Anna Petrova: lawful-scope read-only evidence; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 19 | Tenant break-glass / dead-account recovery | direct | Security Analyst Anna Petrova: ombudsman quorum and Shamir recovery; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 20 | Cognitive impairment / post-trauma | watch | Security Analyst Anna Petrova: slow-down nudges without autonomy loss; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 21 | Pseudonymous + privacy-by-default | direct | Security Analyst Anna Petrova: public identity separated from compliance identity; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 22 | Disaster-zone surge | watch | Security Analyst Anna Petrova: cell isolation plus emergency rate floor; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 23 | Cross-jurisdiction conflict | direct | Security Analyst Anna Petrova: higher-restriction pack wins; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 24 | Account-hijack victim recovery | direct | Security Analyst Anna Petrova: hardware-key recovery and mutation cool-down; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 25 | Mistaken action / unintended mutation | direct | Security Analyst Anna Petrova: undo and rare high-value confirmation; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 26 | Concurrent-session conflict | watch | Security Analyst Anna Petrova: due-process conflict handling; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 27 | Bug bounty submitter | direct | Security Analyst Anna Petrova: safe-harbor submission path; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 28 | Delegated agent acting for human | direct | Security Analyst Anna Petrova: attested delegation chain; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 29 | High-net-worth transaction limits | direct | Security Analyst Anna Petrova: KYB-verified transaction tier; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |
| 30 | Regional outage degradation | direct | Security Analyst Anna Petrova: DR-pair failover within residency boundary; applies under `B2B_EMPLOYEE + B2C_CONSUMER` with active tenant scope. |

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

- Place/time: Riga SOC overflow room, Latvia, 02:18 EET in freezing fog during a credential-stuffing surge.
- Current named tools: Splunk Enterprise Security 8.0 notable events, CrowdStrike Falcon Complete 2026.05 detections, Microsoft Defender XDR 2026.04 incidents, Okta Workforce Identity 2026.1 risk signals, Tenable.io Enterprise 2026.2 exposure exports, and oyatie `audit-chain` sealed incident timeline.
- Named pain points: the "Falcon Case 7741" alert owner changed after containment, hid a $118,000 customer-credit exposure, and cost Anna 9h reconciling Splunk, CrowdStrike, and Okta timestamps.
- Jobs-to-be-done: nightly SOC triage, monthly OKR SEC-SOC-05 "100% high-severity incidents sealed within 30 minutes", Project Aurora phishing drill, and yearly ISO 27001 evidence review with Jakub Nowak.
- Cedar binding: principal `User::"anna-petrova@soc-ru-eu"` accesses `SecurityIncident::*`, `EndpointDetection::*`, `IdentityRiskSignal::*`, and `VulnerabilityFinding::*`; actions `triage_alert`, `contain_endpoint`, `escalate_incident`, and `deny_personal_device_lookup`.
- Cross-context bridges: supports CISO Yuki Park, DevOps Manager Pavel Korsak, Software Engineer Hugo Tanaka, and keeps Anna-as-CTF-player tooling separate from production investigations.
- Journey IDs: `j87-fedramp-high-il5-air-gap-deployment`, `j95-iso-27001-soc-2-annual-audit`, `j117-api-customer-tenant-incident-response`, `j140-internal-audit-data-loss-prevention-egress-trip`.
