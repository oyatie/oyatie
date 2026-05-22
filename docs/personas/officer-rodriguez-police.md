---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 19
persona_slug: officer-rodriguez-police
persona_name: Officer Rodriguez
primary_role: police patrol officer and family member
primary_collar: gray
primary_workspace: field
skill_tier: mid-level
primary_device: vehicle-mount + body-camera + mobile
locale: US
audience_type_primary: LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT
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
journey_range: j001-j150 law enforcement, evidence, emergency, due process
---

# Persona Dossier — Officer Rodriguez

## §A. Archetype

Officer Rodriguez is priority 19 in the 2026-05-21 oyatie persona roster. The active projection is **police patrol officer and family member**. This dossier treats the persona as a projection of one passkey-bound human, not a separate account.

Career arc:
- Entry: enters oyatie through `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`.
- Growth: gains capability tiers without gaining ambient cross-tenant authority.
- Operational: depends on identity, workflow-engine, audit-chain, messenger, incident-mgmt, compliance.
- Cross-context: personal, professional, regulated, family, and side-business contexts remain separate.
- Recovery: ADR-0299 restores identity without reviving revoked tenant roles.
- Precedent: Apple Personal versus Apple Business and Microsoft personal versus work/school, but enforced through Cedar and tenant IDs.
- Count authority: 56 µservices for this brief.
- Layout authority: ADR-0131 flat per-µservice layout.
- Layer authority: ADR-0105 13-layer canonical enum.
- Community authority: `microservices/community/PRD.md`; no `anonymous/` path.

Tenure and universality:
- Collar-color: `gray`.
- Workspace: `field`.
- Skill tier: `mid-level`.
- Device: `vehicle-mount + body-camera + mobile`.
- Locale: `US`.
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.

## §B. Device Profile

- Primary device: `vehicle-mount + body-camera + mobile`.
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
- Audience types: `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`.
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

- `Rodriguez-on-patrol` is the same human under another tenant or role projection.
- `Rodriguez-as-family` is the same human under another tenant or role projection.

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
| 05:30 | Officer Rodriguez acts as police patrol officer and family member. | identity + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Officer Rodriguez acts as police patrol officer and family member. | workflow-engine + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Officer Rodriguez acts as police patrol officer and family member. | audit-chain + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Officer Rodriguez acts as police patrol officer and family member. | messenger + incident-mgmt | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Officer Rodriguez acts as police patrol officer and family member. | incident-mgmt + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Officer Rodriguez acts as police patrol officer and family member. | compliance + drive | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Officer Rodriguez acts as police patrol officer and family member. | drive + notifications | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Officer Rodriguez acts as police patrol officer and family member. | notifications + policy-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Officer Rodriguez acts as police patrol officer and family member. | policy-engine + observability | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Officer Rodriguez acts as police patrol officer and family member. | observability + identity | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Officer Rodriguez acts as police patrol officer and family member. | identity + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Officer Rodriguez acts as police patrol officer and family member. | workflow-engine + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Officer Rodriguez acts as police patrol officer and family member. | audit-chain + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Officer Rodriguez acts as police patrol officer and family member. | messenger + incident-mgmt | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Officer Rodriguez acts as police patrol officer and family member. | incident-mgmt + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Officer Rodriguez acts as police patrol officer and family member. | compliance + drive | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Officer Rodriguez acts as police patrol officer and family member. | drive + notifications | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, workflow-engine, audit-chain, messenger, incident-mgmt, compliance, drive, notifications.
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
- `identity` capability tier: Officer Rodriguez needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: Officer Rodriguez needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: Officer Rodriguez needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: Officer Rodriguez needs tenant-scoped read/write/decide posture with audit evidence.
- `incident-mgmt` capability tier: Officer Rodriguez needs tenant-scoped read/write/decide posture with audit evidence.
- `compliance` capability tier: Officer Rodriguez needs tenant-scoped read/write/decide posture with audit evidence.
- `drive` capability tier: Officer Rodriguez needs tenant-scoped read/write/decide posture with audit evidence.
- `notifications` capability tier: Officer Rodriguez needs tenant-scoped read/write/decide posture with audit evidence.
- `policy-engine` capability tier: Officer Rodriguez needs tenant-scoped read/write/decide posture with audit evidence.
- `observability` capability tier: Officer Rodriguez needs tenant-scoped read/write/decide posture with audit evidence.

56-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | identity | primary | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 02 | tenancy | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 03 | policy-engine | secondary | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 04 | audit-chain | primary | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 05 | workflow-engine | primary | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 06 | workflow-studio | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 07 | community | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 08 | messenger | primary | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 09 | mail | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 10 | calendar | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 11 | meet | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 12 | drive | secondary | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 13 | notes | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 14 | forms | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 15 | payments | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 16 | finops-portal | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 17 | marketplace | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 18 | ontology | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 19 | intelligence | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 20 | observability | secondary | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 21 | compliance | primary | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 22 | governance | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 23 | ops-dashboard-control-center | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 24 | workplace-integration | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 25 | developer-sdk | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 26 | foundry | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 27 | api-gateway | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 28 | cell | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 29 | cloud-secrets | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 30 | analytics | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 31 | search | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 32 | notifications | secondary | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 33 | social | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 34 | shorts | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 35 | ads | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 36 | personal-health-tracker | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 37 | crm | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 38 | marketing-automation | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 39 | contact-center | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 40 | performance-mgmt | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 41 | learning-mgmt | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 42 | itsm | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 43 | incident-mgmt | primary | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 44 | financial-planning | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 45 | data-warehouse | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 46 | contract-lifecycle-mgmt | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 47 | whiteboard | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 48 | design-collaboration | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 49 | erp-finance | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 50 | erp-procurement | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 51 | erp-inventory | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 52 | erp-manufacturing | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 53 | erp-sales | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 54 | erp-hr | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 55 | erp-projects | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 56 | erp-analytics | ambient | Officer Rodriguez must preserve tenant scope, audit events, and active-context UX if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"officer-rodriguez-police@active-tenant",
  action in ActionGroup::"officer-rodriguez-police.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["LAW_ENFORCEMENT", "B2C_CONSUMER", "B2C_FAMILY_PARENT"] &&
  context.persona_projection == "officer-rodriguez-police" &&
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
| CJIS | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Officer Rodriguez. |
| US-privacy | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Officer Rodriguez. |
| court-order | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Officer Rodriguez. |
| family-boundary | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Officer Rodriguez. |
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
| 1 | Emergency services | direct | Officer Rodriguez: audit and attest without blocking life-safety; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 2 | Account recovery / lockout | watch | Officer Rodriguez: passkey backup, recovery code, and trusted contact; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 3 | Financial fraud dispute + chargeback | direct | Officer Rodriguez: PSP-integrated fast-track dispute; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 4 | Elder financial abuse | watch | Officer Rodriguez: cooling-off and trusted-contact alert; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 5 | Healthcare urgent care + EHR break-glass | direct | Officer Rodriguez: post-hoc audit-and-justify; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 6 | Whistleblower + ethics report | watch | Officer Rodriguez: anonymous sealed chain of custody; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 7 | Press freedom / journalist source | direct | Officer Rodriguez: metadata-minimized source protection; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 8 | Domestic violence / abuse survivor | watch | Officer Rodriguez: silent shelter mode; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 9 | Child safety + mandatory reporting | direct | Officer Rodriguez: mandatory-reporter route cannot be suppressed; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 10 | Deceased-user account | watch | Officer Rodriguez: legacy contact plus court-order path; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 11 | Custody / shared-account dispute | direct | Officer Rodriguez: family-court order integration; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 12 | Disability accommodations | watch | Officer Rodriguez: accessibility profile overrides friction defaults; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 13 | Non-native-language user | direct | Officer Rodriguez: sensitive translation requires consent; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 14 | Low-bandwidth / disaster-zone / offline-first | watch | Officer Rodriguez: offline audit retention and sync; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 15 | Banking / financial inclusion | direct | Officer Rodriguez: low-tier financial inclusion path; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 16 | Activist / dissident | watch | Officer Rodriguez: Tor-friendly metadata-minimized mode; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 17 | Regulator-deadline outage | direct | Officer Rodriguez: degraded deadline-preserving workflow; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 18 | Audit / regulator / law-enforcement access | watch | Officer Rodriguez: lawful-scope read-only evidence; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 19 | Tenant break-glass / dead-account recovery | direct | Officer Rodriguez: ombudsman quorum and Shamir recovery; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 20 | Cognitive impairment / post-trauma | watch | Officer Rodriguez: slow-down nudges without autonomy loss; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 21 | Pseudonymous + privacy-by-default | direct | Officer Rodriguez: public identity separated from compliance identity; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 22 | Disaster-zone surge | watch | Officer Rodriguez: cell isolation plus emergency rate floor; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 23 | Cross-jurisdiction conflict | direct | Officer Rodriguez: higher-restriction pack wins; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 24 | Account-hijack victim recovery | watch | Officer Rodriguez: hardware-key recovery and mutation cool-down; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 25 | Mistaken action / unintended mutation | direct | Officer Rodriguez: 15s undo and rare high-value confirmation; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 26 | Concurrent-session conflict | watch | Officer Rodriguez: due-process session conflict handling; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 27 | Bug bounty submitter | direct | Officer Rodriguez: security-researcher allow-list and safe harbor; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 28 | Delegated agent acting for human | watch | Officer Rodriguez: attested delegation chain; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 29 | High-net-worth transaction limits | direct | Officer Rodriguez: KYB-verified transaction tier; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |
| 30 | Regional outage degradation | watch | Officer Rodriguez: DR-pair failover within residency boundary; applies under `LAW_ENFORCEMENT + B2C_CONSUMER + B2C_FAMILY_PARENT`. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.

## §K. Their Journey Range

- Primary range: j001-j150 law enforcement, evidence, emergency, due process.
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

- Place/time: LAPD Olympic Division roll-call room, Koreatown, Los Angeles, 06:05 PST on a smoky wildfire-morning air-quality alert.
- Current named tools: Axon Evidence.com 2026.1 for bodycam evidence, Motorola PremierOne CAD 4.7 for dispatch, Mark43 RMS 2026.02 for reports, and iPhone 15 FirstNet profile for field messaging.
- Named pain points: incident `OLY-2025-1192` attached a family-contact SMS to a CAD note, forced a $6,500 discovery review, and delayed report approval 13 hours.
- Jobs-to-be-done: weekly bodycam/report evidence closeout; quarterly CJIS and use-of-force audit; yearly POST recertification, family privacy review, and disaster-response drill.
- Cedar binding: principal `User::"officer-rodriguez@lapd-olympic-us"` accesses `CADIncident::*`, `BodycamEvidence::*`, `CJISRecord::*`, and `FamilyMessenger::*`; actions `accept_dispatch`, `upload_bodycam`, `file_incident_report`, and `deny_family_message_discovery`.
- Cross-context bridges: lawful requests from Anya Mironova or Father Lopez require warrant/legal scope, school safety handoffs can involve Ms. Patel, and high-risk emergency patterns mirror Yejin Park's dispatch journeys.
- Journey IDs: `j01-emergency-911-dispatch`, `j12-mass-casualty-incident-10x-traffic`, `j129-court-warrant-pierces-personal-tenant-with-judicial-oversight`.

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
