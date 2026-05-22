---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 10
persona_slug: chris-volkov
persona_name: Chris Volkov
primary_role: laid-off mid-career engineer rebuilding personal economy
primary_collar: white
primary_workspace: back-office
skill_tier: mid-level
primary_device: personal laptop + mobile
locale: US
audience_type_primary: B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT
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
journey_range: j142-j147 layoff, portfolio import, job pipeline, mutual aid
---

# Persona Dossier — Chris Volkov

## §A. Archetype

Chris Volkov is priority 10 in the 2026-05-21 oyatie persona roster. The active projection is **laid-off mid-career engineer rebuilding personal economy**. This dossier treats the persona as a projection of one passkey-bound human, not a separate account.

Career arc:
- Entry: enters oyatie through `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Growth: gains capability tiers without gaining ambient cross-tenant authority.
- Operational: depends on identity, tenancy, workflow-studio, community, mail, messenger.
- Cross-context: personal, professional, regulated, family, and side-business contexts remain separate.
- Recovery: ADR-0299 restores identity without reviving revoked tenant roles.
- Precedent: Apple Personal versus Apple Business and Microsoft personal versus work/school, but enforced through Cedar and tenant IDs.
- Count authority: 56 µservices for this brief.
- Layout authority: ADR-0131 flat per-µservice layout.
- Layer authority: ADR-0105 13-layer canonical enum.
- Community authority: `microservices/community/PRD.md`; no `anonymous/` path.

Tenure and universality:
- Collar-color: `white`.
- Workspace: `back-office`.
- Skill tier: `mid-level`.
- Device: `personal laptop + mobile`.
- Locale: `US`.
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.

## §B. Device Profile

- Primary device: `personal laptop + mobile`.
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
- Audience types: `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`.
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

- `Chris-pre-layoff` is the same human under another tenant or role projection.
- `Chris-post-layoff` is the same human under another tenant or role projection.
- `Chris-as-family-provider` is the same human under another tenant or role projection.

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
| 05:30 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | identity + tenancy | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | tenancy + workflow-studio | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | workflow-studio + community | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | community + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | mail + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | messenger + drive | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | drive + calendar | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | calendar + payments | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | payments + marketplace | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | marketplace + finops-portal | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | finops-portal + identity | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | identity + tenancy | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | tenancy + workflow-studio | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | workflow-studio + community | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | community + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | mail + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Chris Volkov acts as laid-off mid-career engineer rebuilding personal economy. | messenger + drive | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, tenancy, workflow-studio, community, mail, messenger, drive, calendar.
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
- `identity` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.
- `tenancy` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-studio` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.
- `community` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.
- `drive` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.
- `calendar` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.
- `payments` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.
- `marketplace` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.
- `finops-portal` capability tier: Chris Volkov needs tenant-scoped read/write/decide posture with audit evidence.

56-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | identity | primary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 02 | tenancy | primary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 03 | policy-engine | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 04 | audit-chain | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 05 | workflow-engine | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 06 | workflow-studio | primary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 07 | community | primary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 08 | messenger | primary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 09 | mail | primary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 10 | calendar | secondary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 11 | meet | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 12 | drive | secondary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 13 | notes | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 14 | forms | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 15 | payments | secondary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 16 | finops-portal | secondary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 17 | marketplace | secondary | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 18 | ontology | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 19 | intelligence | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 20 | observability | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 21 | compliance | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 22 | governance | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 23 | ops-dashboard-control-center | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 24 | workplace-integration | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 25 | developer-sdk | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 26 | foundry | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 27 | api-gateway | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 28 | cell | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 29 | cloud-secrets | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 30 | analytics | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 31 | search | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 32 | notifications | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 33 | social | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 34 | shorts | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 35 | ads | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 36 | personal-health-tracker | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 37 | crm | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 38 | marketing-automation | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 39 | contact-center | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 40 | performance-mgmt | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 41 | learning-mgmt | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 42 | itsm | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 43 | incident-mgmt | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 44 | financial-planning | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 45 | data-warehouse | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 46 | contract-lifecycle-mgmt | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 47 | whiteboard | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 48 | design-collaboration | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 49 | erp-finance | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 50 | erp-procurement | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 51 | erp-inventory | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 52 | erp-manufacturing | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 53 | erp-sales | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 54 | erp-hr | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 55 | erp-projects | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 56 | erp-analytics | ambient | Chris Volkov must preserve tenant scope, audit events, and active-context UX if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"chris-volkov@active-tenant",
  action in ActionGroup::"chris-volkov.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2C_JOB_SEEKER_ACTIVE", "B2C_CONSUMER", "B2C_FAMILY_PARENT"] &&
  context.persona_projection == "chris-volkov" &&
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
| US-labor | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Chris Volkov. |
| COBRA | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Chris Volkov. |
| account-recovery | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Chris Volkov. |
| financial-inclusion | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Chris Volkov. |
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
| 1 | Emergency services | watch | Chris Volkov: audit and attest without blocking life-safety; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 2 | Account recovery / lockout | direct | Chris Volkov: passkey backup, recovery code, and trusted contact; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 3 | Financial fraud dispute + chargeback | watch | Chris Volkov: PSP-integrated fast-track dispute; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 4 | Elder financial abuse | direct | Chris Volkov: cooling-off and trusted-contact alert; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 5 | Healthcare urgent care + EHR break-glass | watch | Chris Volkov: post-hoc audit-and-justify; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 6 | Whistleblower + ethics report | direct | Chris Volkov: anonymous sealed chain of custody; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 7 | Press freedom / journalist source | watch | Chris Volkov: metadata-minimized source protection; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 8 | Domestic violence / abuse survivor | direct | Chris Volkov: silent shelter mode; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 9 | Child safety + mandatory reporting | watch | Chris Volkov: mandatory-reporter route cannot be suppressed; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 10 | Deceased-user account | direct | Chris Volkov: legacy contact plus court-order path; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 11 | Custody / shared-account dispute | watch | Chris Volkov: family-court order integration; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 12 | Disability accommodations | direct | Chris Volkov: accessibility profile overrides friction defaults; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 13 | Non-native-language user | watch | Chris Volkov: sensitive translation requires consent; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 14 | Low-bandwidth / disaster-zone / offline-first | direct | Chris Volkov: offline audit retention and sync; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 15 | Banking / financial inclusion | watch | Chris Volkov: low-tier financial inclusion path; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 16 | Activist / dissident | direct | Chris Volkov: Tor-friendly metadata-minimized mode; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 17 | Regulator-deadline outage | watch | Chris Volkov: degraded deadline-preserving workflow; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 18 | Audit / regulator / law-enforcement access | direct | Chris Volkov: lawful-scope read-only evidence; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 19 | Tenant break-glass / dead-account recovery | watch | Chris Volkov: ombudsman quorum and Shamir recovery; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 20 | Cognitive impairment / post-trauma | direct | Chris Volkov: slow-down nudges without autonomy loss; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 21 | Pseudonymous + privacy-by-default | watch | Chris Volkov: public identity separated from compliance identity; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 22 | Disaster-zone surge | direct | Chris Volkov: cell isolation plus emergency rate floor; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 23 | Cross-jurisdiction conflict | watch | Chris Volkov: higher-restriction pack wins; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 24 | Account-hijack victim recovery | direct | Chris Volkov: hardware-key recovery and mutation cool-down; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 25 | Mistaken action / unintended mutation | watch | Chris Volkov: 15s undo and rare high-value confirmation; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 26 | Concurrent-session conflict | direct | Chris Volkov: due-process session conflict handling; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 27 | Bug bounty submitter | watch | Chris Volkov: security-researcher allow-list and safe harbor; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 28 | Delegated agent acting for human | direct | Chris Volkov: attested delegation chain; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 29 | High-net-worth transaction limits | watch | Chris Volkov: KYB-verified transaction tier; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 30 | Regional outage degradation | direct | Chris Volkov: DR-pair failover within residency boundary; applies under `B2C_JOB_SEEKER_ACTIVE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.

## §K. Their Journey Range

- Primary range: j142-j147 layoff, portfolio import, job pipeline, mutual aid.
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

- Place/time: Detroit Public Library Skillman Branch, 121 Gratiot Avenue, 10:25 EST in February slush while Chris rebuilds his job pipeline from a borrowed study-room monitor.
- Current named tools: LinkedIn Premium Career 2026 for recruiter reachout, Teal Job Tracker Pro 3.9 for applications, Google Takeout 2026 export for portfolio salvage, and YNAB 5.4 for layoff-budget planning.
- Named pain points: layoff-day export scrub blocked 74 portfolio files, cost him a $3,200 contract interview window, and took 18 hours to prove his personal tenant was not former-employer property.
- Jobs-to-be-done: weekly 25-application pipeline and unemployment-benefits checklist; quarterly OKR "reemployed by Q2 without personal/work leakage"; yearly tax, COBRA, retirement rollover, and marketplace income closeout.
- Cedar binding: principal `User::"chris-volkov@personal-us"` accesses `JobPipeline::*`, `PortfolioExport::*`, `FormerEmployerArchive::*`, and `MarketplaceGig::*`; actions `import_scrubbed_portfolio`, `apply_to_role`, `accept_marketplace_gig`, and `appeal_export_denial`.
- Cross-context bridges: receives offboarding notices from Priya Krishnan, is protected from Sam Okafor's work-surface audit scope, and may ask Benefits Specialist Aoife Murphy for benefits continuation without reopening employer access.
- Journey IDs: `j142-layoff-day-zero-from-employees-side`, `j143-laid-off-imports-work-portfolio-into-personal-tenant`, `j144-laid-off-builds-job-search-pipeline-in-workflow-studio`, `j145-laid-off-applies-via-community-handshake-linkedin-mode`, `j146-laid-off-uses-marketplace-as-temporary-income`, `j147-laid-off-cohort-mutual-aid-community-channel`.

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
