---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 15
persona_slug: carlos-martinez-forklift
persona_name: Carlos Martinez
primary_role: forklift driver and warehouse parent
primary_collar: blue
primary_workspace: field
skill_tier: mid-level
primary_device: handheld-rugged + kiosk + mobile
locale: US
audience_type_primary: B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT
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
journey_range: j001-j150 field shift, safety incident, family, benefits
---

# Persona Dossier — Carlos Martinez

## §A. Archetype

Carlos Martinez is priority 15 in the 2026-05-21 oyatie persona roster. The active projection is **forklift driver and warehouse parent**. This dossier treats the persona as a projection of one passkey-bound human, not a separate account.

Career arc:
- Entry: enters oyatie through `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Growth: gains capability tiers without gaining ambient cross-tenant authority.
- Operational: depends on identity, workplace-integration, workflow-engine, messenger, calendar, payments.
- Cross-context: personal, professional, regulated, family, and side-business contexts remain separate.
- Recovery: ADR-0299 restores identity without reviving revoked tenant roles.
- Precedent: Apple Personal versus Apple Business and Microsoft personal versus work/school, but enforced through Cedar and tenant IDs.
- Count authority: 56 µservices for this brief.
- Layout authority: ADR-0131 flat per-µservice layout.
- Layer authority: ADR-0105 13-layer canonical enum.
- Community authority: `microservices/community/PRD.md`; no `anonymous/` path.

Tenure and universality:
- Collar-color: `blue`.
- Workspace: `field`.
- Skill tier: `mid-level`.
- Device: `handheld-rugged + kiosk + mobile`.
- Locale: `US`.
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.

## §B. Device Profile

- Primary device: `handheld-rugged + kiosk + mobile`.
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

- Primary locale: `US`.
- Audience types: `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`.
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

- `Carlos-at-work` is the same human under another tenant or role projection.
- `Carlos-as-father` is the same human under another tenant or role projection.

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
| 05:30 | Carlos Martinez acts as forklift driver and warehouse parent. | identity + workplace-integration | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Carlos Martinez acts as forklift driver and warehouse parent. | workplace-integration + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Carlos Martinez acts as forklift driver and warehouse parent. | workflow-engine + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Carlos Martinez acts as forklift driver and warehouse parent. | messenger + calendar | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Carlos Martinez acts as forklift driver and warehouse parent. | calendar + payments | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Carlos Martinez acts as forklift driver and warehouse parent. | payments + community | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Carlos Martinez acts as forklift driver and warehouse parent. | community + incident-mgmt | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Carlos Martinez acts as forklift driver and warehouse parent. | incident-mgmt + erp-inventory | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Carlos Martinez acts as forklift driver and warehouse parent. | erp-inventory + notifications | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Carlos Martinez acts as forklift driver and warehouse parent. | notifications + identity | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Carlos Martinez acts as forklift driver and warehouse parent. | identity + workplace-integration | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Carlos Martinez acts as forklift driver and warehouse parent. | workplace-integration + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Carlos Martinez acts as forklift driver and warehouse parent. | workflow-engine + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Carlos Martinez acts as forklift driver and warehouse parent. | messenger + calendar | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Carlos Martinez acts as forklift driver and warehouse parent. | calendar + payments | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Carlos Martinez acts as forklift driver and warehouse parent. | payments + community | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Carlos Martinez acts as forklift driver and warehouse parent. | community + incident-mgmt | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, workplace-integration, workflow-engine, messenger, calendar, payments, community, incident-mgmt.
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
- `identity` capability tier: Carlos Martinez needs tenant-scoped read/write/decide posture with audit evidence.
- `workplace-integration` capability tier: Carlos Martinez needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: Carlos Martinez needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: Carlos Martinez needs tenant-scoped read/write/decide posture with audit evidence.
- `calendar` capability tier: Carlos Martinez needs tenant-scoped read/write/decide posture with audit evidence.
- `payments` capability tier: Carlos Martinez needs tenant-scoped read/write/decide posture with audit evidence.
- `community` capability tier: Carlos Martinez needs tenant-scoped read/write/decide posture with audit evidence.
- `incident-mgmt` capability tier: Carlos Martinez needs tenant-scoped read/write/decide posture with audit evidence.
- `erp-inventory` capability tier: Carlos Martinez needs tenant-scoped read/write/decide posture with audit evidence.
- `notifications` capability tier: Carlos Martinez needs tenant-scoped read/write/decide posture with audit evidence.

56-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | identity | primary | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 02 | tenancy | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 03 | policy-engine | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 04 | audit-chain | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 05 | workflow-engine | primary | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 06 | workflow-studio | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 07 | community | secondary | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 08 | messenger | primary | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 09 | mail | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 10 | calendar | primary | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 11 | meet | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 12 | drive | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 13 | notes | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 14 | forms | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 15 | payments | primary | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 16 | finops-portal | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 17 | marketplace | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 18 | ontology | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 19 | intelligence | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 20 | observability | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 21 | compliance | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 22 | governance | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 23 | ops-dashboard-control-center | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 24 | workplace-integration | primary | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 25 | developer-sdk | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 26 | foundry | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 27 | api-gateway | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 28 | cell | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 29 | cloud-secrets | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 30 | analytics | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 31 | search | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 32 | notifications | secondary | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 33 | social | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 34 | shorts | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 35 | ads | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 36 | personal-health-tracker | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 37 | crm | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 38 | marketing-automation | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 39 | contact-center | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 40 | performance-mgmt | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 41 | learning-mgmt | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 42 | itsm | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 43 | incident-mgmt | secondary | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 44 | financial-planning | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 45 | data-warehouse | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 46 | contract-lifecycle-mgmt | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 47 | whiteboard | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 48 | design-collaboration | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 49 | erp-finance | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 50 | erp-procurement | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 51 | erp-inventory | secondary | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 52 | erp-manufacturing | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 53 | erp-sales | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 54 | erp-hr | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 55 | erp-projects | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 56 | erp-analytics | ambient | Carlos Martinez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"carlos-martinez-forklift@active-tenant",
  action in ActionGroup::"carlos-martinez-forklift.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2B_FIELD_WORKER", "B2C_CONSUMER", "B2C_FAMILY_PARENT"] &&
  context.persona_projection == "carlos-martinez-forklift" &&
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
| OSHA | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Carlos Martinez. |
| US-labor | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Carlos Martinez. |
| KOSA-parent | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Carlos Martinez. |
| reserved-pack-04 | Future tenant or vertical overlay. | Cannot weaken ADR-0244 or ADR-0311. |
| reserved-pack-05 | Future tenant or vertical overlay. | Cannot weaken ADR-0244 or ADR-0311. |
| reserved-pack-06 | Future tenant or vertical overlay. | Cannot weaken ADR-0244 or ADR-0311. |
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
| 1 | Emergency services | direct | Carlos Martinez: audit and attest without blocking life-safety; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 2 | Account recovery / lockout | watch | Carlos Martinez: passkey backup, recovery code, and trusted contact; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 3 | Financial fraud dispute + chargeback | direct | Carlos Martinez: PSP-integrated fast-track dispute; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 4 | Elder financial abuse | watch | Carlos Martinez: cooling-off and trusted-contact alert; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 5 | Healthcare urgent care + EHR break-glass | direct | Carlos Martinez: post-hoc audit-and-justify; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 6 | Whistleblower + ethics report | watch | Carlos Martinez: anonymous sealed chain of custody; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 7 | Press freedom / journalist source | direct | Carlos Martinez: metadata-minimized source protection; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 8 | Domestic violence / abuse survivor | watch | Carlos Martinez: silent shelter mode; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 9 | Child safety + mandatory reporting | direct | Carlos Martinez: mandatory-reporter route cannot be suppressed; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 10 | Deceased-user account | watch | Carlos Martinez: legacy contact plus court-order path; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 11 | Custody / shared-account dispute | direct | Carlos Martinez: family-court order integration; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 12 | Disability accommodations | watch | Carlos Martinez: accessibility profile overrides friction defaults; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 13 | Non-native-language user | direct | Carlos Martinez: sensitive translation requires consent; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 14 | Low-bandwidth / disaster-zone / offline-first | watch | Carlos Martinez: offline audit retention and sync; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 15 | Banking / financial inclusion | direct | Carlos Martinez: low-tier financial inclusion path; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 16 | Activist / dissident | watch | Carlos Martinez: Tor-friendly metadata-minimized mode; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 17 | Regulator-deadline outage | direct | Carlos Martinez: degraded deadline-preserving workflow; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 18 | Audit / regulator / law-enforcement access | watch | Carlos Martinez: lawful-scope read-only evidence; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 19 | Tenant break-glass / dead-account recovery | direct | Carlos Martinez: ombudsman quorum and Shamir recovery; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 20 | Cognitive impairment / post-trauma | watch | Carlos Martinez: slow-down nudges without autonomy loss; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 21 | Pseudonymous + privacy-by-default | direct | Carlos Martinez: public identity separated from compliance identity; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 22 | Disaster-zone surge | watch | Carlos Martinez: cell isolation plus emergency rate floor; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 23 | Cross-jurisdiction conflict | direct | Carlos Martinez: higher-restriction pack wins; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 24 | Account-hijack victim recovery | watch | Carlos Martinez: hardware-key recovery and mutation cool-down; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 25 | Mistaken action / unintended mutation | direct | Carlos Martinez: 15s undo and rare high-value confirmation; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 26 | Concurrent-session conflict | watch | Carlos Martinez: due-process session conflict handling; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 27 | Bug bounty submitter | direct | Carlos Martinez: security-researcher allow-list and safe harbor; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 28 | Delegated agent acting for human | watch | Carlos Martinez: attested delegation chain; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 29 | High-net-worth transaction limits | direct | Carlos Martinez: KYB-verified transaction tier; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 30 | Regional outage degradation | watch | Carlos Martinez: DR-pair failover within residency boundary; applies under `B2B_FIELD_WORKER + B2C_CONSUMER + B2C_FAMILY_PARENT`. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.

## §K. Their Journey Range

- Primary range: j001-j150 field shift, safety incident, family, benefits.
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

- Place/time: Lineage Logistics cold-storage dock 14, South Gate, Los Angeles, 04:55 PST in 41°F dock fog before the first Costco pallet wave.
- Current named tools: Zebra MC9300 Android Enterprise scanner with Ivanti Velocity 2.1, Blue Yonder WMS 2026.1 for slotting, Kronos Workforce Dimensions 9.2 for clocking, and Toyota I_Site 4.8 forklift telemetry.
- Named pain points: the "Dock 14 mis-pick" on 2026-02-11 sent 96 frozen pallets to the wrong route, cost $28,400 in spoilage risk credits, and forced Carlos to stay 2h40m after shift for evidence reconciliation.
- Jobs-to-be-done: weekly shift-clock proof and pallet exception closeout; quarterly OSHA forklift recertification and cold-chain audit; yearly family tax/childcare budget review after wage and overtime changes.
- Cedar binding: principal `User::"carlos-martinez@lineage-southgate-us"` accesses `PalletMove::*`, `ForkliftTelemetry::*`, `ShiftClock::*`, and `FamilyBudget::*`; actions `confirm_pick`, `open_mispick_exception`, `attest_forklift_inspection`, and `export_w2_summary`.
- Cross-context bridges: safety incidents can route to Ahmad Hassan's construction-style bilingual incident pattern, maintenance faults route to Carlos Reyes II, and HR wage questions route to Priya Krishnan without exposing family data.
- Journey IDs: `j37-b2b-clocking-and-attendance`, `j40-b2b-marketplace-vendor-billing`, `j52-order-to-cash-marketplace-to-fulfillment`.

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
