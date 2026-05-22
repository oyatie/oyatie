---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 29
persona_slug: board-director-patrick-oreilly
persona_name: Board director Patrick O'Reilly
primary_role: independent board director across three boards
primary_collar: white
primary_workspace: executive
skill_tier: principal
primary_device: desktop + secure mobile
locale: IE
audience_type_primary: B2B_BOARD_DIRECTOR + B2C_CONSUMER
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
journey_range: j001-j150 board resolutions, conflicts, audit, private boundary
---

# Persona Dossier — Board director Patrick O'Reilly

## §A. Archetype

Board director Patrick O'Reilly is priority 29 in the 2026-05-21 oyatie persona roster. The active projection is **independent board director across three boards**. This dossier treats the persona as a projection of one passkey-bound human, not a separate account.

Career arc:
- Entry: enters oyatie through `B2B_BOARD_DIRECTOR + B2C_CONSUMER`.
- Growth: gains capability tiers without gaining ambient cross-tenant authority.
- Operational: depends on identity, governance, mail, calendar, drive, workflow-engine.
- Cross-context: personal, professional, regulated, family, and side-business contexts remain separate.
- Recovery: ADR-0299 restores identity without reviving revoked tenant roles.
- Precedent: Apple Personal versus Apple Business and Microsoft personal versus work/school, but enforced through Cedar and tenant IDs.
- Count authority: 56 µservices for this brief.
- Layout authority: ADR-0131 flat per-µservice layout.
- Layer authority: ADR-0105 13-layer canonical enum.
- Community authority: `microservices/community/PRD.md`; no `anonymous/` path.

Tenure and universality:
- Collar-color: `white`.
- Workspace: `executive`.
- Skill tier: `principal`.
- Device: `desktop + secure mobile`.
- Locale: `IE`.
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.

## §B. Device Profile

- Primary device: `desktop + secure mobile`.
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

- Primary locale: `IE`.
- Audience types: `B2B_BOARD_DIRECTOR + B2C_CONSUMER`.
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

- `O'Reilly-on-Board-A` is the same human under another tenant or role projection.
- `O'Reilly-on-Board-B` is the same human under another tenant or role projection.
- `O'Reilly-on-Board-C` is the same human under another tenant or role projection.

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
| 05:30 | Board director Patrick O'Reilly acts as independent board director across three boards. | identity + governance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Board director Patrick O'Reilly acts as independent board director across three boards. | governance + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | mail + calendar | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | calendar + drive | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | drive + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | workflow-engine + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | audit-chain + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | compliance + financial-planning | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | financial-planning + ops-dashboard-control-center | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | ops-dashboard-control-center + identity | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | identity + governance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | governance + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | mail + calendar | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | calendar + drive | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | drive + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Board director Patrick O'Reilly acts as independent board director across three boards. | workflow-engine + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Board director Patrick O'Reilly acts as independent board director across three boards. | audit-chain + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, governance, mail, calendar, drive, workflow-engine, audit-chain, compliance.
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
- `identity` capability tier: Board director Patrick O'Reilly needs tenant-scoped read/write/decide posture with audit evidence.
- `governance` capability tier: Board director Patrick O'Reilly needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: Board director Patrick O'Reilly needs tenant-scoped read/write/decide posture with audit evidence.
- `calendar` capability tier: Board director Patrick O'Reilly needs tenant-scoped read/write/decide posture with audit evidence.
- `drive` capability tier: Board director Patrick O'Reilly needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: Board director Patrick O'Reilly needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: Board director Patrick O'Reilly needs tenant-scoped read/write/decide posture with audit evidence.
- `compliance` capability tier: Board director Patrick O'Reilly needs tenant-scoped read/write/decide posture with audit evidence.
- `financial-planning` capability tier: Board director Patrick O'Reilly needs tenant-scoped read/write/decide posture with audit evidence.
- `ops-dashboard-control-center` capability tier: Board director Patrick O'Reilly needs tenant-scoped read/write/decide posture with audit evidence.

56-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | identity | primary | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 02 | tenancy | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 03 | policy-engine | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 04 | audit-chain | secondary | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 05 | workflow-engine | primary | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 06 | workflow-studio | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 07 | community | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 08 | messenger | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 09 | mail | primary | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 10 | calendar | primary | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 11 | meet | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 12 | drive | primary | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 13 | notes | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 14 | forms | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 15 | payments | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 16 | finops-portal | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 17 | marketplace | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 18 | ontology | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 19 | intelligence | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 20 | observability | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 21 | compliance | secondary | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 22 | governance | primary | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 23 | ops-dashboard-control-center | secondary | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 24 | workplace-integration | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 25 | developer-sdk | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 26 | foundry | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 27 | api-gateway | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 28 | cell | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 29 | cloud-secrets | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 30 | analytics | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 31 | search | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 32 | notifications | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 33 | social | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 34 | shorts | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 35 | ads | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 36 | personal-health-tracker | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 37 | crm | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 38 | marketing-automation | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 39 | contact-center | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 40 | performance-mgmt | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 41 | learning-mgmt | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 42 | itsm | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 43 | incident-mgmt | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 44 | financial-planning | secondary | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 45 | data-warehouse | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 46 | contract-lifecycle-mgmt | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 47 | whiteboard | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 48 | design-collaboration | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 49 | erp-finance | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 50 | erp-procurement | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 51 | erp-inventory | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 52 | erp-manufacturing | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 53 | erp-sales | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 54 | erp-hr | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 55 | erp-projects | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 56 | erp-analytics | ambient | Board director Patrick O'Reilly must preserve tenant scope, audit events, and active-context UX if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"board-director-patrick-oreilly@active-tenant",
  action in ActionGroup::"board-director-patrick-oreilly.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2B_BOARD_DIRECTOR", "B2C_CONSUMER"] &&
  context.persona_projection == "board-director-patrick-oreilly" &&
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
| GDPR | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Board director Patrick O'Reilly. |
| board-governance | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Board director Patrick O'Reilly. |
| market-abuse | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Board director Patrick O'Reilly. |
| privileged-comms | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Board director Patrick O'Reilly. |
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
| 1 | Emergency services | direct | Board director Patrick O'Reilly: audit and attest without blocking life-safety; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 2 | Account recovery / lockout | watch | Board director Patrick O'Reilly: passkey backup, recovery code, and trusted contact; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 3 | Financial fraud dispute + chargeback | direct | Board director Patrick O'Reilly: PSP-integrated fast-track dispute; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 4 | Elder financial abuse | watch | Board director Patrick O'Reilly: cooling-off and trusted-contact alert; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 5 | Healthcare urgent care + EHR break-glass | direct | Board director Patrick O'Reilly: post-hoc audit-and-justify; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 6 | Whistleblower + ethics report | watch | Board director Patrick O'Reilly: anonymous sealed chain of custody; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 7 | Press freedom / journalist source | direct | Board director Patrick O'Reilly: metadata-minimized source protection; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 8 | Domestic violence / abuse survivor | watch | Board director Patrick O'Reilly: silent shelter mode; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 9 | Child safety + mandatory reporting | direct | Board director Patrick O'Reilly: mandatory-reporter route cannot be suppressed; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 10 | Deceased-user account | watch | Board director Patrick O'Reilly: legacy contact plus court-order path; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 11 | Custody / shared-account dispute | direct | Board director Patrick O'Reilly: family-court order integration; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 12 | Disability accommodations | watch | Board director Patrick O'Reilly: accessibility profile overrides friction defaults; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 13 | Non-native-language user | direct | Board director Patrick O'Reilly: sensitive translation requires consent; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 14 | Low-bandwidth / disaster-zone / offline-first | watch | Board director Patrick O'Reilly: offline audit retention and sync; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 15 | Banking / financial inclusion | direct | Board director Patrick O'Reilly: low-tier financial inclusion path; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 16 | Activist / dissident | watch | Board director Patrick O'Reilly: Tor-friendly metadata-minimized mode; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 17 | Regulator-deadline outage | direct | Board director Patrick O'Reilly: degraded deadline-preserving workflow; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 18 | Audit / regulator / law-enforcement access | watch | Board director Patrick O'Reilly: lawful-scope read-only evidence; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 19 | Tenant break-glass / dead-account recovery | direct | Board director Patrick O'Reilly: ombudsman quorum and Shamir recovery; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 20 | Cognitive impairment / post-trauma | watch | Board director Patrick O'Reilly: slow-down nudges without autonomy loss; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 21 | Pseudonymous + privacy-by-default | direct | Board director Patrick O'Reilly: public identity separated from compliance identity; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 22 | Disaster-zone surge | watch | Board director Patrick O'Reilly: cell isolation plus emergency rate floor; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 23 | Cross-jurisdiction conflict | direct | Board director Patrick O'Reilly: higher-restriction pack wins; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 24 | Account-hijack victim recovery | watch | Board director Patrick O'Reilly: hardware-key recovery and mutation cool-down; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 25 | Mistaken action / unintended mutation | direct | Board director Patrick O'Reilly: 15s undo and rare high-value confirmation; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 26 | Concurrent-session conflict | watch | Board director Patrick O'Reilly: due-process session conflict handling; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 27 | Bug bounty submitter | direct | Board director Patrick O'Reilly: security-researcher allow-list and safe harbor; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 28 | Delegated agent acting for human | watch | Board director Patrick O'Reilly: attested delegation chain; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 29 | High-net-worth transaction limits | direct | Board director Patrick O'Reilly: KYB-verified transaction tier; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |
| 30 | Regional outage degradation | watch | Board director Patrick O'Reilly: DR-pair failover within residency boundary; applies under `B2B_BOARD_DIRECTOR + B2C_CONSUMER`. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.

## §K. Their Journey Range

- Primary range: j001-j150 board resolutions, conflicts, audit, private boundary.
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

- Place/time: IFSC Dublin boardroom, North Wall Quay, 06:40 GMT on a foggy Tuesday before three consecutive audit-committee calls.
- Current named tools: Diligent Boards 2026.1 for board books, Nasdaq Boardvantage 8.9 for committee workflows, DocuSign CLM 2026.02 for resolutions, and iManage Work 10.8 for privileged board materials.
- Named pain points: resolution packet `B-2026-ACQ-07` copied a Company A diligence appendix into Company B's board portal, created €96,000 in counsel remediation, and postponed a vote by 48 hours.
- Jobs-to-be-done: weekly committee pre-read separation across three boards; quarterly OKR "zero cross-board evidence bleed"; yearly independence certification, D&O insurance, and conflict-of-interest attestation.
- Cedar binding: principal `User::"patrick-oreilly@independent-board-ie"` accesses `BoardRoom::*`, `CommitteePacket::*`, `ConflictRegister::*`, and `PersonalAdvisorNote::*`; actions `vote_resolution`, `annotate_board_book`, `declare_conflict`, and `deny_cross_board_copy`.
- Cross-context bridges: reviews Marcus Chen's board materials only in that board tenant, receives CFO Helena Brandt's audit pack, checks CEO Aoki Tanaka's subsidiary governance, and escalates compliance language to CCO Naveen Iyer.
- Journey IDs: `j94-sox-404-public-company-controls`, `j123-multi-tenant-coordinated-product-launch`, `j165-cco-naveen-iyer-board-quarterly-compliance-report`.

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
