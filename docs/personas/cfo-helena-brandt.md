---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 12
persona_slug: cfo-helena-brandt
persona_name: CFO Helena Brandt
primary_role: public-company CFO
primary_collar: white
primary_workspace: executive
skill_tier: executive
primary_device: desktop-primary + hardware-key mobile
locale: DE
audience_type_primary: B2B_CSUITE + B2C_CONSUMER
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
journey_range: j001-j150 finance close, treasury, audit, spend, board
---

# Persona Dossier — CFO Helena Brandt

## §A. Archetype

CFO Helena Brandt is priority 12 in the 2026-05-21 oyatie persona roster. The active projection is **public-company CFO**. This dossier treats the persona as a projection of one passkey-bound human, not a separate account.

Career arc:
- Entry: enters oyatie through `B2B_CSUITE + B2C_CONSUMER`.
- Growth: gains capability tiers without gaining ambient cross-tenant authority.
- Operational: depends on identity, payments, finops-portal, financial-planning, erp-finance, compliance.
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
- Skill tier: `executive`.
- Device: `desktop-primary + hardware-key mobile`.
- Locale: `DE`.
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.

## §B. Device Profile

- Primary device: `desktop-primary + hardware-key mobile`.
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

- Primary locale: `DE`.
- Audience types: `B2B_CSUITE + B2C_CONSUMER`.
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

- `Helena-as-CFO` is the same human under another tenant or role projection.
- `Helena-as-charity-board-director` is the same human under another tenant or role projection.

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
| 05:30 | CFO Helena Brandt acts as public-company CFO. | identity + payments | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | CFO Helena Brandt acts as public-company CFO. | payments + finops-portal | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | CFO Helena Brandt acts as public-company CFO. | finops-portal + financial-planning | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | CFO Helena Brandt acts as public-company CFO. | financial-planning + erp-finance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | CFO Helena Brandt acts as public-company CFO. | erp-finance + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | CFO Helena Brandt acts as public-company CFO. | compliance + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | CFO Helena Brandt acts as public-company CFO. | audit-chain + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | CFO Helena Brandt acts as public-company CFO. | workflow-engine + data-warehouse | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | CFO Helena Brandt acts as public-company CFO. | data-warehouse + marketplace | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | CFO Helena Brandt acts as public-company CFO. | marketplace + identity | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | CFO Helena Brandt acts as public-company CFO. | identity + payments | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | CFO Helena Brandt acts as public-company CFO. | payments + finops-portal | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | CFO Helena Brandt acts as public-company CFO. | finops-portal + financial-planning | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | CFO Helena Brandt acts as public-company CFO. | financial-planning + erp-finance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | CFO Helena Brandt acts as public-company CFO. | erp-finance + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | CFO Helena Brandt acts as public-company CFO. | compliance + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | CFO Helena Brandt acts as public-company CFO. | audit-chain + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, payments, finops-portal, financial-planning, erp-finance, compliance, audit-chain, workflow-engine.
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
- `identity` capability tier: CFO Helena Brandt needs tenant-scoped read/write/decide posture with audit evidence.
- `payments` capability tier: CFO Helena Brandt needs tenant-scoped read/write/decide posture with audit evidence.
- `finops-portal` capability tier: CFO Helena Brandt needs tenant-scoped read/write/decide posture with audit evidence.
- `financial-planning` capability tier: CFO Helena Brandt needs tenant-scoped read/write/decide posture with audit evidence.
- `erp-finance` capability tier: CFO Helena Brandt needs tenant-scoped read/write/decide posture with audit evidence.
- `compliance` capability tier: CFO Helena Brandt needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: CFO Helena Brandt needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: CFO Helena Brandt needs tenant-scoped read/write/decide posture with audit evidence.
- `data-warehouse` capability tier: CFO Helena Brandt needs tenant-scoped read/write/decide posture with audit evidence.
- `marketplace` capability tier: CFO Helena Brandt needs tenant-scoped read/write/decide posture with audit evidence.

56-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | identity | primary | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 02 | tenancy | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 03 | policy-engine | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 04 | audit-chain | secondary | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 05 | workflow-engine | secondary | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 06 | workflow-studio | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 07 | community | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 08 | messenger | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 09 | mail | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 10 | calendar | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 11 | meet | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 12 | drive | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 13 | notes | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 14 | forms | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 15 | payments | primary | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 16 | finops-portal | primary | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 17 | marketplace | secondary | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 18 | ontology | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 19 | intelligence | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 20 | observability | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 21 | compliance | primary | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 22 | governance | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 23 | ops-dashboard-control-center | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 24 | workplace-integration | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 25 | developer-sdk | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 26 | foundry | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 27 | api-gateway | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 28 | cell | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 29 | cloud-secrets | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 30 | analytics | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 31 | search | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 32 | notifications | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 33 | social | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 34 | shorts | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 35 | ads | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 36 | personal-health-tracker | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 37 | crm | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 38 | marketing-automation | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 39 | contact-center | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 40 | performance-mgmt | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 41 | learning-mgmt | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 42 | itsm | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 43 | incident-mgmt | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 44 | financial-planning | primary | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 45 | data-warehouse | secondary | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 46 | contract-lifecycle-mgmt | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 47 | whiteboard | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 48 | design-collaboration | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 49 | erp-finance | primary | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 50 | erp-procurement | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 51 | erp-inventory | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 52 | erp-manufacturing | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 53 | erp-sales | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 54 | erp-hr | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 55 | erp-projects | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 56 | erp-analytics | ambient | CFO Helena Brandt must preserve tenant scope, audit events, and active-context UX if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"cfo-helena-brandt@active-tenant",
  action in ActionGroup::"cfo-helena-brandt.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2B_CSUITE", "B2C_CONSUMER"] &&
  context.persona_projection == "cfo-helena-brandt" &&
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
| GDPR | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for CFO Helena Brandt. |
| SOX-404 | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for CFO Helena Brandt. |
| DORA | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for CFO Helena Brandt. |
| EU-tax | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for CFO Helena Brandt. |
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
| 1 | Emergency services | watch | CFO Helena Brandt: audit and attest without blocking life-safety; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 2 | Account recovery / lockout | direct | CFO Helena Brandt: passkey backup, recovery code, and trusted contact; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 3 | Financial fraud dispute + chargeback | watch | CFO Helena Brandt: PSP-integrated fast-track dispute; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 4 | Elder financial abuse | direct | CFO Helena Brandt: cooling-off and trusted-contact alert; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 5 | Healthcare urgent care + EHR break-glass | watch | CFO Helena Brandt: post-hoc audit-and-justify; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 6 | Whistleblower + ethics report | direct | CFO Helena Brandt: anonymous sealed chain of custody; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 7 | Press freedom / journalist source | watch | CFO Helena Brandt: metadata-minimized source protection; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 8 | Domestic violence / abuse survivor | direct | CFO Helena Brandt: silent shelter mode; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 9 | Child safety + mandatory reporting | watch | CFO Helena Brandt: mandatory-reporter route cannot be suppressed; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 10 | Deceased-user account | direct | CFO Helena Brandt: legacy contact plus court-order path; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 11 | Custody / shared-account dispute | watch | CFO Helena Brandt: family-court order integration; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 12 | Disability accommodations | direct | CFO Helena Brandt: accessibility profile overrides friction defaults; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 13 | Non-native-language user | watch | CFO Helena Brandt: sensitive translation requires consent; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 14 | Low-bandwidth / disaster-zone / offline-first | direct | CFO Helena Brandt: offline audit retention and sync; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 15 | Banking / financial inclusion | watch | CFO Helena Brandt: low-tier financial inclusion path; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 16 | Activist / dissident | direct | CFO Helena Brandt: Tor-friendly metadata-minimized mode; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 17 | Regulator-deadline outage | watch | CFO Helena Brandt: degraded deadline-preserving workflow; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 18 | Audit / regulator / law-enforcement access | direct | CFO Helena Brandt: lawful-scope read-only evidence; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 19 | Tenant break-glass / dead-account recovery | watch | CFO Helena Brandt: ombudsman quorum and Shamir recovery; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 20 | Cognitive impairment / post-trauma | direct | CFO Helena Brandt: slow-down nudges without autonomy loss; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 21 | Pseudonymous + privacy-by-default | watch | CFO Helena Brandt: public identity separated from compliance identity; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 22 | Disaster-zone surge | direct | CFO Helena Brandt: cell isolation plus emergency rate floor; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 23 | Cross-jurisdiction conflict | watch | CFO Helena Brandt: higher-restriction pack wins; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 24 | Account-hijack victim recovery | direct | CFO Helena Brandt: hardware-key recovery and mutation cool-down; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 25 | Mistaken action / unintended mutation | watch | CFO Helena Brandt: 15s undo and rare high-value confirmation; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 26 | Concurrent-session conflict | direct | CFO Helena Brandt: due-process session conflict handling; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 27 | Bug bounty submitter | watch | CFO Helena Brandt: security-researcher allow-list and safe harbor; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 28 | Delegated agent acting for human | direct | CFO Helena Brandt: attested delegation chain; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 29 | High-net-worth transaction limits | watch | CFO Helena Brandt: KYB-verified transaction tier; applies under `B2B_CSUITE + B2C_CONSUMER`. |
| 30 | Regional outage degradation | direct | CFO Helena Brandt: DR-pair failover within residency boundary; applies under `B2B_CSUITE + B2C_CONSUMER`. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.

## §K. Their Journey Range

- Primary range: j001-j150 finance close, treasury, audit, spend, board.
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

- Place/time: Frankfurt Westend finance war room, 23:30 CET on quarter-close night while sleet hits Bockenheimer Landstrasse.
- Current named tools: SAP S/4HANA Cloud Public Edition 2408 for GL close, Workiva Wdesk 2026.1 for SEC/ESMA reporting, BlackLine 2026.2 for reconciliations, and Kyriba Treasury 2026.1 for liquidity and FX.
- Named pain points: close package `Q4-2025-IFRS16` carried a €3.4m lease reclass error, triggered €48,000 in external-audit rework, and took 16 hours because charity-board expenses appeared in her CFO workspace.
- Jobs-to-be-done: weekly cash forecast and covenant dashboard; quarterly OKR "close D+4 with zero personal/charity bleed"; yearly SOX-404, DORA, tax, and board-compensation review.
- Cedar binding: principal `User::"helena-brandt@publicco-de"` accesses `GeneralLedger::*`, `TreasuryPosition::*`, `ExternalAuditRequest::*`, and `CharityBoardPacket::*`; actions `approve_close_entry`, `release_cash_forecast`, `answer_audit_pbc`, and `deny_charity_cross_read`.
- Cross-context bridges: signs strategic budgets with CEO Aoki Tanaka, reconciles treasury risk with Sven Eriksson, reports SOX evidence to Sam Okafor, and receives LP pressure from Aanya Kapoor only through investor-relations scope.
- Journey IDs: `j94-sox-404-public-company-controls`, `j120-tenant-treasury-multi-currency-fx-hedge`, `j122-vendor-payment-batch-with-tax-withholding`, `j174-sven-eriksson-treasury-eod-position-reconciliation`.

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
