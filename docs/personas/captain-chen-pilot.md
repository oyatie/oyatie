---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 17
persona_slug: captain-chen-pilot
persona_name: Captain Chen
primary_role: long-haul airline pilot and father
primary_collar: gold
primary_workspace: field
skill_tier: senior
primary_device: vehicle-mount + tablet + mobile
locale: SG
audience_type_primary: B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT
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
journey_range: j001-j150 roster, safety, family, travel disruption
---

# Persona Dossier — Captain Chen

## §A. Archetype

Captain Chen is priority 17 in the 2026-05-21 oyatie persona roster. The active projection is **long-haul airline pilot and father**. This dossier treats the persona as a projection of one passkey-bound human, not a separate account.

Career arc:
- Entry: enters oyatie through `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Growth: gains capability tiers without gaining ambient cross-tenant authority.
- Operational: depends on identity, calendar, workflow-engine, messenger, mail, incident-mgmt.
- Cross-context: personal, professional, regulated, family, and side-business contexts remain separate.
- Recovery: ADR-0299 restores identity without reviving revoked tenant roles.
- Precedent: Apple Personal versus Apple Business and Microsoft personal versus work/school, but enforced through Cedar and tenant IDs.
- Count authority: 56 µservices for this brief.
- Layout authority: ADR-0131 flat per-µservice layout.
- Layer authority: ADR-0105 13-layer canonical enum.
- Community authority: `microservices/community/PRD.md`; no `anonymous/` path.

Tenure and universality:
- Collar-color: `gold`.
- Workspace: `field`.
- Skill tier: `senior`.
- Device: `vehicle-mount + tablet + mobile`.
- Locale: `SG`.
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.

## §B. Device Profile

- Primary device: `vehicle-mount + tablet + mobile`.
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

- Primary locale: `SG`.
- Audience types: `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`.
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

- `Chen-as-pilot` is the same human under another tenant or role projection.
- `Chen-as-father` is the same human under another tenant or role projection.

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
| 05:30 | Captain Chen acts as long-haul airline pilot and father. | identity + calendar | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Captain Chen acts as long-haul airline pilot and father. | calendar + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Captain Chen acts as long-haul airline pilot and father. | workflow-engine + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Captain Chen acts as long-haul airline pilot and father. | messenger + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Captain Chen acts as long-haul airline pilot and father. | mail + incident-mgmt | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Captain Chen acts as long-haul airline pilot and father. | incident-mgmt + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Captain Chen acts as long-haul airline pilot and father. | compliance + personal-health-tracker | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Captain Chen acts as long-haul airline pilot and father. | personal-health-tracker + observability | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Captain Chen acts as long-haul airline pilot and father. | observability + notifications | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Captain Chen acts as long-haul airline pilot and father. | notifications + identity | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Captain Chen acts as long-haul airline pilot and father. | identity + calendar | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Captain Chen acts as long-haul airline pilot and father. | calendar + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Captain Chen acts as long-haul airline pilot and father. | workflow-engine + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Captain Chen acts as long-haul airline pilot and father. | messenger + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Captain Chen acts as long-haul airline pilot and father. | mail + incident-mgmt | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Captain Chen acts as long-haul airline pilot and father. | incident-mgmt + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Captain Chen acts as long-haul airline pilot and father. | compliance + personal-health-tracker | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, calendar, workflow-engine, messenger, mail, incident-mgmt, compliance, personal-health-tracker.
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
- `identity` capability tier: Captain Chen needs tenant-scoped read/write/decide posture with audit evidence.
- `calendar` capability tier: Captain Chen needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: Captain Chen needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: Captain Chen needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: Captain Chen needs tenant-scoped read/write/decide posture with audit evidence.
- `incident-mgmt` capability tier: Captain Chen needs tenant-scoped read/write/decide posture with audit evidence.
- `compliance` capability tier: Captain Chen needs tenant-scoped read/write/decide posture with audit evidence.
- `personal-health-tracker` capability tier: Captain Chen needs tenant-scoped read/write/decide posture with audit evidence.
- `observability` capability tier: Captain Chen needs tenant-scoped read/write/decide posture with audit evidence.
- `notifications` capability tier: Captain Chen needs tenant-scoped read/write/decide posture with audit evidence.

56-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | identity | primary | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 02 | tenancy | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 03 | policy-engine | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 04 | audit-chain | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 05 | workflow-engine | primary | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 06 | workflow-studio | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 07 | community | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 08 | messenger | primary | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 09 | mail | primary | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 10 | calendar | primary | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 11 | meet | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 12 | drive | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 13 | notes | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 14 | forms | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 15 | payments | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 16 | finops-portal | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 17 | marketplace | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 18 | ontology | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 19 | intelligence | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 20 | observability | secondary | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 21 | compliance | secondary | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 22 | governance | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 23 | ops-dashboard-control-center | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 24 | workplace-integration | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 25 | developer-sdk | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 26 | foundry | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 27 | api-gateway | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 28 | cell | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 29 | cloud-secrets | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 30 | analytics | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 31 | search | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 32 | notifications | secondary | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 33 | social | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 34 | shorts | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 35 | ads | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 36 | personal-health-tracker | secondary | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 37 | crm | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 38 | marketing-automation | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 39 | contact-center | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 40 | performance-mgmt | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 41 | learning-mgmt | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 42 | itsm | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 43 | incident-mgmt | primary | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 44 | financial-planning | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 45 | data-warehouse | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 46 | contract-lifecycle-mgmt | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 47 | whiteboard | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 48 | design-collaboration | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 49 | erp-finance | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 50 | erp-procurement | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 51 | erp-inventory | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 52 | erp-manufacturing | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 53 | erp-sales | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 54 | erp-hr | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 55 | erp-projects | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 56 | erp-analytics | ambient | Captain Chen must preserve tenant scope, audit events, and active-context UX if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"captain-chen-pilot@active-tenant",
  action in ActionGroup::"captain-chen-pilot.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2B_EMPLOYEE", "B2C_CONSUMER", "B2C_FAMILY_PARENT"] &&
  context.persona_projection == "captain-chen-pilot" &&
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
| aviation-safety | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Captain Chen. |
| Singapore-PDPA | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Captain Chen. |
| fatigue-risk | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Captain Chen. |
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
| 1 | Emergency services | direct | Captain Chen: audit and attest without blocking life-safety; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 2 | Account recovery / lockout | watch | Captain Chen: passkey backup, recovery code, and trusted contact; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 3 | Financial fraud dispute + chargeback | direct | Captain Chen: PSP-integrated fast-track dispute; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 4 | Elder financial abuse | watch | Captain Chen: cooling-off and trusted-contact alert; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 5 | Healthcare urgent care + EHR break-glass | direct | Captain Chen: post-hoc audit-and-justify; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 6 | Whistleblower + ethics report | watch | Captain Chen: anonymous sealed chain of custody; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 7 | Press freedom / journalist source | direct | Captain Chen: metadata-minimized source protection; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 8 | Domestic violence / abuse survivor | watch | Captain Chen: silent shelter mode; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 9 | Child safety + mandatory reporting | direct | Captain Chen: mandatory-reporter route cannot be suppressed; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 10 | Deceased-user account | watch | Captain Chen: legacy contact plus court-order path; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 11 | Custody / shared-account dispute | direct | Captain Chen: family-court order integration; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 12 | Disability accommodations | watch | Captain Chen: accessibility profile overrides friction defaults; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 13 | Non-native-language user | direct | Captain Chen: sensitive translation requires consent; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 14 | Low-bandwidth / disaster-zone / offline-first | watch | Captain Chen: offline audit retention and sync; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 15 | Banking / financial inclusion | direct | Captain Chen: low-tier financial inclusion path; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 16 | Activist / dissident | watch | Captain Chen: Tor-friendly metadata-minimized mode; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 17 | Regulator-deadline outage | direct | Captain Chen: degraded deadline-preserving workflow; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 18 | Audit / regulator / law-enforcement access | watch | Captain Chen: lawful-scope read-only evidence; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 19 | Tenant break-glass / dead-account recovery | direct | Captain Chen: ombudsman quorum and Shamir recovery; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 20 | Cognitive impairment / post-trauma | watch | Captain Chen: slow-down nudges without autonomy loss; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 21 | Pseudonymous + privacy-by-default | direct | Captain Chen: public identity separated from compliance identity; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 22 | Disaster-zone surge | watch | Captain Chen: cell isolation plus emergency rate floor; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 23 | Cross-jurisdiction conflict | direct | Captain Chen: higher-restriction pack wins; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 24 | Account-hijack victim recovery | watch | Captain Chen: hardware-key recovery and mutation cool-down; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 25 | Mistaken action / unintended mutation | direct | Captain Chen: 15s undo and rare high-value confirmation; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 26 | Concurrent-session conflict | watch | Captain Chen: due-process session conflict handling; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 27 | Bug bounty submitter | direct | Captain Chen: security-researcher allow-list and safe harbor; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 28 | Delegated agent acting for human | watch | Captain Chen: attested delegation chain; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 29 | High-net-worth transaction limits | direct | Captain Chen: KYB-verified transaction tier; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 30 | Regional outage degradation | watch | Captain Chen: DR-pair failover within residency boundary; applies under `B2B_EMPLOYEE + B2C_CONSUMER + B2C_FAMILY_PARENT`. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.

## §K. Their Journey Range

- Primary range: j001-j150 roster, safety, family, travel disruption.
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

- Place/time: Singapore Changi Terminal 3 crew briefing room, 23:40 SGT before SQ26 redeye departure under a lightning-alert ground stop.
- Current named tools: Jeppesen FliteDeck Pro 4.7.2 for charts, Sabre AirCentre Crew Manager 2026.1 for roster, Apple iPad Pro M4 with EFB MDM profile, and WhatsApp Business 2.24 for family travel logistics.
- Named pain points: 2025 Narita diversion `SQ26-NRT-1125` left hotel duty-time evidence in email screenshots, cost S$18,600 in crew-delay penalties, and took 9h to prove personal family travel was outside airline scope.
- Jobs-to-be-done: weekly duty-time and EFB chart currency check; quarterly safety OKR "zero unsealed diversion evidence"; yearly simulator, medical certificate, and family travel entitlement review.
- Cedar binding: principal `User::"captain-chen@singapore-airline-crew"` accesses `FlightPlan::*`, `CrewRoster::*`, `DutyTimeRecord::*`, and `FamilyTravelPlan::*`; actions `acknowledge_release`, `file_diversion_report`, `attest_rest_period`, and `view_family_itinerary`.
- Cross-context bridges: outage/offline patterns borrow from Sarah Kim's mobile route sync, regulated evidence boundaries align with Diana Reyes, and family context mirrors Marcus Chen's parent/employee separation.
- Journey IDs: `j11-disaster-zone-offline-first-sync`, `j27-calendar-cross-context-family-and-work`, `j97-sg-pdpa-mas-singapore-tenant`.

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
