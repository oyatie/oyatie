---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 78
persona_slug: ir-manager-lev-kahn
persona_name: IR Manager Lev Kahn
roster_display_name: IR Manager Lev Kahn
primary_role: Investor Relations Manager
primary_collar: white
primary_workspace: front-office
skill_tier: senior
primary_device: desktop + mobile
locale: IL
audience_type_primary: B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER
microservice_count_authority: 69
community_path: microservices/community/PRD.md
layer_enum_authority: ADR-0105 13-layer canonical enum
flat_layout_authority: ADR-0131 per-microservice flat layout
cross_context_bridge: |
  Lev-as-IR / Lev-as-LP-of-fund
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

# Persona Dossier — IR Manager Lev Kahn

## §A. Archetype

IR Manager Lev Kahn is roster row 78 in the 2026-05-21 oyatie persona graph. The active projection is **Investor Relations Manager**. This dossier treats the persona as one passkey-bound human projected through tenant, role, workspace, locale, device, and skill-tier context.

Career arc:
- Entry context: `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER`.
- Collar-color posture: `white`.
- Workspace posture: `front-office`.
- Tenure and skill tier: `senior`.
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

- Primary locale: `IL`.
- Audience types: `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER`.
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

- `Lev-as-IR` is the same human under another tenant or role projection.
- `Lev-as-LP-of-fund` is the same human under another tenant or role projection.

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
| 05:30 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | identity + tenancy | read within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | tenancy + policy-engine | write within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | policy-engine + audit-chain | decide within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | audit-chain + workflow-engine | delegate within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | workflow-engine + community | read within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | community + messenger | write within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | messenger + mail | decide within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | mail + calendar | delegate within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | calendar + payments | read within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | payments + finops-portal | write within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | finops-portal + treasury | decide within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | treasury + financial-planning | delegate within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | financial-planning + data-warehouse | read within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | data-warehouse + compliance | write within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | compliance + identity | decide within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | identity + tenancy | delegate within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | IR Manager Lev Kahn acts as Investor Relations Manager in the active context. | tenancy + policy-engine | read within `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` | Active tenant banner is visible and Cedar evaluates before mutation. |

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
- `identity` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `tenancy` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `policy-engine` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `community` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `calendar` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `payments` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `finops-portal` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `treasury` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `financial-planning` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `data-warehouse` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.
- `compliance` capability tier: IR Manager Lev Kahn needs tenant-scoped read/write/decide posture with audit evidence.

69-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | analytics | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 02 | api-gateway | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 03 | application | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 04 | audit-chain | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 05 | calendar | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 06 | cell | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 07 | cloud-iac | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 08 | cloud-k8s | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 09 | cloud-secrets | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 10 | comms-email | secondary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 11 | community | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 12 | compliance | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 13 | connector | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 14 | consent-graph | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 15 | contact-center | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 16 | contract-lifecycle-management | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 17 | crm | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 18 | data-pipeline | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 19 | data-warehouse | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 20 | design-collaboration | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 21 | detection | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 22 | developer-sdk | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 23 | docs | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 24 | drive | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 25 | feature-flags | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 26 | financial-planning | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 27 | finops-portal | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 28 | forms | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 29 | foundry | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 30 | global-trade | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 31 | governance | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 32 | healthcare-integration | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 33 | identity | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 34 | incident-management | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 35 | intelligence | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 36 | itsm | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 37 | learning-management | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 38 | mail | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 39 | marketing-automation | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 40 | marketplace | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 41 | meet | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 42 | messenger | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 43 | network | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 44 | notes | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 45 | observability | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 46 | ontology | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 47 | ops-dashboard-control-center | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 48 | payments | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 49 | performance-management | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 50 | plant-maintenance | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 51 | plugin-app-store | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 52 | production-planning | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 53 | quality-management | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 54 | real-estate | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 55 | recordings | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 56 | sheets | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 57 | shorts | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 58 | sites | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 59 | slides | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 60 | social | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 61 | supply-chain-planning | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 62 | tasks | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 63 | tenancy | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 64 | translate | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 65 | treasury | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 66 | warehouse | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 67 | whiteboard | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 68 | workflow-engine | primary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 69 | workflow-studio | secondary | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |
| 70 | workplace-integration | ambient | IR Manager Lev Kahn must preserve tenant scope, audit events, active-context UX, and pack overlays if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"ir-manager-lev-kahn@active-tenant",
  action in ActionGroup::"ir-manager-lev-kahn.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2B_EMPLOYEE", "B2B_INVESTOR_LP (other-cap)", "B2C_CONSUMER"] &&
  context.persona_projection == "ir-manager-lev-kahn" &&
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
| GDPR | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for IR Manager Lev Kahn. |
| EU-AI-Act | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for IR Manager Lev Kahn. |
| DORA-where-financial | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for IR Manager Lev Kahn. |
| DPDP-2023 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for IR Manager Lev Kahn. |
| FINRA/SEC/Reg-NMS or local banking pack | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for IR Manager Lev Kahn. |
| reserved-pack-06 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for IR Manager Lev Kahn. |
| reserved-pack-07 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for IR Manager Lev Kahn. |
| reserved-pack-08 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for IR Manager Lev Kahn. |
| reserved-pack-09 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for IR Manager Lev Kahn. |
| reserved-pack-10 | Locale, role, tenant, data class, or audience type activates it. | Adds retention, evidence, lawful-basis, Cedar reason-code, residency, or co-sign rules for IR Manager Lev Kahn. |

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
| 1 | Emergency services | watch | IR Manager Lev Kahn: audit-and-attest without blocking life safety; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 2 | Account recovery / lockout | direct | IR Manager Lev Kahn: passkey backup, recovery code, delegated trusted contact, and ombudsman path; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 3 | Financial fraud dispute + chargeback | direct | IR Manager Lev Kahn: PSP-timed fast-track dispute and audit evidence; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 4 | Elder financial abuse | direct | IR Manager Lev Kahn: cooling-off and trusted-contact alert without removing autonomy; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 5 | Healthcare urgent care + EHR break-glass | watch | IR Manager Lev Kahn: post-hoc audit-and-justify for treatment urgency; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 6 | Whistleblower + ethics report | watch | IR Manager Lev Kahn: sealed chain of custody and tenant-admin blindness; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 7 | Press freedom / journalist source | direct | IR Manager Lev Kahn: metadata-minimized protected-source mode; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 8 | Domestic violence / abuse survivor | direct | IR Manager Lev Kahn: silent shelter mode and safe-device handling; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 9 | Child safety + mandatory reporting | watch | IR Manager Lev Kahn: mandatory-reporter route cannot be suppressed; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 10 | Deceased-user account | watch | IR Manager Lev Kahn: legacy-contact and court-order ingress; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 11 | Custody / shared-account dispute | watch | IR Manager Lev Kahn: family-court order integration and child-best-interest default; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 12 | Disability accommodations | direct | IR Manager Lev Kahn: accessibility profile overrides friction defaults; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 13 | Non-native-language user | direct | IR Manager Lev Kahn: consented translation on sensitive surfaces; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 14 | Low-bandwidth / disaster-zone / offline-first | direct | IR Manager Lev Kahn: offline audit retention and conflict-safe sync; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 15 | Banking / financial inclusion | direct | IR Manager Lev Kahn: low-tier inclusion path with regulator floor; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 16 | Activist / dissident | direct | IR Manager Lev Kahn: Tor-friendly metadata-minimized mode; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 17 | Regulator-deadline outage | watch | IR Manager Lev Kahn: deadline-preserving degraded workflow; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 18 | Audit / regulator / law-enforcement access | watch | IR Manager Lev Kahn: lawful-scope read-only evidence; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 19 | Tenant break-glass / dead-account recovery | watch | IR Manager Lev Kahn: ombudsman quorum and Shamir recovery; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 20 | Cognitive impairment / post-trauma | watch | IR Manager Lev Kahn: slow-down nudges without autonomy loss; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 21 | Pseudonymous + privacy-by-default | direct | IR Manager Lev Kahn: public identity separated from compliance identity; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 22 | Disaster-zone surge | watch | IR Manager Lev Kahn: cell isolation plus emergency rate floor; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 23 | Cross-jurisdiction conflict | direct | IR Manager Lev Kahn: higher-restriction pack wins; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 24 | Account-hijack victim recovery | direct | IR Manager Lev Kahn: hardware-key recovery and mutation cool-down; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 25 | Mistaken action / unintended mutation | direct | IR Manager Lev Kahn: undo and rare high-value confirmation; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 26 | Concurrent-session conflict | watch | IR Manager Lev Kahn: due-process conflict handling; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 27 | Bug bounty submitter | watch | IR Manager Lev Kahn: safe-harbor submission path; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 28 | Delegated agent acting for human | direct | IR Manager Lev Kahn: attested delegation chain; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 29 | High-net-worth transaction limits | direct | IR Manager Lev Kahn: KYB-verified transaction tier; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |
| 30 | Regional outage degradation | direct | IR Manager Lev Kahn: DR-pair failover within residency boundary; applies under `B2B_EMPLOYEE + B2B_INVESTOR_LP (other-cap) + B2C_CONSUMER` with active tenant scope. |

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

- Place/time: Tel Aviv IR studio, 16:00 IDT, 90 minutes before annual shareholder-meeting livestream opens.
- Current named tools: Nasdaq IR Insight, Q4 Platform, Diligent Boards, Zoom Webinars, Broadridge proxy feed, and oyatie `recordings`.
- Named pain points: Reg FD selective-disclosure risk when analyst Q&A arrives by private email; livestream captions lag legal disclaimers; proxy-vote CSVs arrive after the board packet is sealed.
- Jobs-to-be-done: weekly investor-question triage; quarterly earnings and shareholder webcast; yearly AGM, proxy, and annual-report evidence archive.
- Cedar binding: principal `User::"ir-manager-lev-kahn@issuer-ir"` accesses `InvestorQuestion::*`, `WebcastRecording::*`, `ProxyVote::*`, and `DisclosureLog::*`; actions `publish_webcast`, `redact_selective_detail`, `seal_qna_log`, and `export_proxy_result`.
- Cross-context bridges: works with CFO Helena Brandt, Board Secretary Florence Akinsanya, PR Manager Helena Sato, and External Auditor Dimitri Volkov.
- Journey IDs: `j172-lev-kahn-investor-relations-shareholder-meeting-livestream`, `j94-sox-404-public-company-controls`.

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
