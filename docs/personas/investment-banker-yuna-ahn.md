---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 23
persona_slug: investment-banker-yuna-ahn
persona_name: Investment Banker Yuna Ahn
primary_role: M&A investment banker
primary_collar: white
primary_workspace: front-office
skill_tier: senior
primary_device: regulated desktop + secure mobile
locale: KR
audience_type_primary: B2B_BANK_INTERNAL + B2C_CONSUMER
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
journey_range: j001-j150 banking, deal room, personal MBA, audit
---

# Persona Dossier — Investment Banker Yuna Ahn

## §A. Archetype

Investment Banker Yuna Ahn is priority 23 in the 2026-05-21 oyatie persona roster. The active projection is **M&A investment banker**. This dossier treats the persona as a projection of one passkey-bound human, not a separate account.

Career arc:
- Entry: enters oyatie through `B2B_BANK_INTERNAL + B2C_CONSUMER`.
- Growth: gains capability tiers without gaining ambient cross-tenant authority.
- Operational: depends on identity, mail, messenger, drive, workflow-engine, payments.
- Cross-context: personal, professional, regulated, family, and side-business contexts remain separate.
- Recovery: ADR-0299 restores identity without reviving revoked tenant roles.
- Precedent: Apple Personal versus Apple Business and Microsoft personal versus work/school, but enforced through Cedar and tenant IDs.
- Count authority: 56 µservices for this brief.
- Layout authority: ADR-0131 flat per-µservice layout.
- Layer authority: ADR-0105 13-layer canonical enum.
- Community authority: `microservices/community/PRD.md`; no `anonymous/` path.

Tenure and universality:
- Collar-color: `white`.
- Workspace: `front-office`.
- Skill tier: `senior`.
- Device: `regulated desktop + secure mobile`.
- Locale: `KR`.
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.

## §B. Device Profile

- Primary device: `regulated desktop + secure mobile`.
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

- Primary locale: `KR`.
- Audience types: `B2B_BANK_INTERNAL + B2C_CONSUMER`.
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

- `Yuna-as-IB` is the same human under another tenant or role projection.
- `Yuna-as-MBA-applicant` is the same human under another tenant or role projection.

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
| 05:30 | Investment Banker Yuna Ahn acts as M&A investment banker. | identity + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Investment Banker Yuna Ahn acts as M&A investment banker. | mail + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | messenger + drive | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | drive + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | workflow-engine + payments | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | payments + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | audit-chain + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | compliance + data-warehouse | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | data-warehouse + calendar | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | calendar + identity | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | identity + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | mail + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | messenger + drive | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | drive + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | workflow-engine + payments | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Investment Banker Yuna Ahn acts as M&A investment banker. | payments + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Investment Banker Yuna Ahn acts as M&A investment banker. | audit-chain + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, mail, messenger, drive, workflow-engine, payments, audit-chain, compliance.
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
- `identity` capability tier: Investment Banker Yuna Ahn needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: Investment Banker Yuna Ahn needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: Investment Banker Yuna Ahn needs tenant-scoped read/write/decide posture with audit evidence.
- `drive` capability tier: Investment Banker Yuna Ahn needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: Investment Banker Yuna Ahn needs tenant-scoped read/write/decide posture with audit evidence.
- `payments` capability tier: Investment Banker Yuna Ahn needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: Investment Banker Yuna Ahn needs tenant-scoped read/write/decide posture with audit evidence.
- `compliance` capability tier: Investment Banker Yuna Ahn needs tenant-scoped read/write/decide posture with audit evidence.
- `data-warehouse` capability tier: Investment Banker Yuna Ahn needs tenant-scoped read/write/decide posture with audit evidence.
- `calendar` capability tier: Investment Banker Yuna Ahn needs tenant-scoped read/write/decide posture with audit evidence.

56-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | identity | primary | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 02 | tenancy | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 03 | policy-engine | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 04 | audit-chain | secondary | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 05 | workflow-engine | primary | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 06 | workflow-studio | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 07 | community | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 08 | messenger | primary | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 09 | mail | primary | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 10 | calendar | secondary | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 11 | meet | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 12 | drive | primary | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 13 | notes | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 14 | forms | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 15 | payments | primary | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 16 | finops-portal | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 17 | marketplace | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 18 | ontology | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 19 | intelligence | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 20 | observability | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 21 | compliance | secondary | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 22 | governance | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 23 | ops-dashboard-control-center | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 24 | workplace-integration | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 25 | developer-sdk | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 26 | foundry | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 27 | api-gateway | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 28 | cell | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 29 | cloud-secrets | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 30 | analytics | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 31 | search | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 32 | notifications | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 33 | social | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 34 | shorts | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 35 | ads | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 36 | personal-health-tracker | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 37 | crm | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 38 | marketing-automation | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 39 | contact-center | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 40 | performance-mgmt | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 41 | learning-mgmt | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 42 | itsm | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 43 | incident-mgmt | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 44 | financial-planning | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 45 | data-warehouse | secondary | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 46 | contract-lifecycle-mgmt | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 47 | whiteboard | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 48 | design-collaboration | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 49 | erp-finance | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 50 | erp-procurement | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 51 | erp-inventory | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 52 | erp-manufacturing | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 53 | erp-sales | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 54 | erp-hr | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 55 | erp-projects | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 56 | erp-analytics | ambient | Investment Banker Yuna Ahn must preserve tenant scope, audit events, and active-context UX if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"investment-banker-yuna-ahn@active-tenant",
  action in ActionGroup::"investment-banker-yuna-ahn.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2B_BANK_INTERNAL", "B2C_CONSUMER"] &&
  context.persona_projection == "investment-banker-yuna-ahn" &&
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
| KR-FSS | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Investment Banker Yuna Ahn. |
| FINRA-equivalent | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Investment Banker Yuna Ahn. |
| insider-info | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Investment Banker Yuna Ahn. |
| deal-room | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Investment Banker Yuna Ahn. |
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
| 1 | Emergency services | direct | Investment Banker Yuna Ahn: audit and attest without blocking life-safety; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 2 | Account recovery / lockout | watch | Investment Banker Yuna Ahn: passkey backup, recovery code, and trusted contact; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 3 | Financial fraud dispute + chargeback | direct | Investment Banker Yuna Ahn: PSP-integrated fast-track dispute; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 4 | Elder financial abuse | watch | Investment Banker Yuna Ahn: cooling-off and trusted-contact alert; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 5 | Healthcare urgent care + EHR break-glass | direct | Investment Banker Yuna Ahn: post-hoc audit-and-justify; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 6 | Whistleblower + ethics report | watch | Investment Banker Yuna Ahn: anonymous sealed chain of custody; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 7 | Press freedom / journalist source | direct | Investment Banker Yuna Ahn: metadata-minimized source protection; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 8 | Domestic violence / abuse survivor | watch | Investment Banker Yuna Ahn: silent shelter mode; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 9 | Child safety + mandatory reporting | direct | Investment Banker Yuna Ahn: mandatory-reporter route cannot be suppressed; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 10 | Deceased-user account | watch | Investment Banker Yuna Ahn: legacy contact plus court-order path; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 11 | Custody / shared-account dispute | direct | Investment Banker Yuna Ahn: family-court order integration; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 12 | Disability accommodations | watch | Investment Banker Yuna Ahn: accessibility profile overrides friction defaults; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 13 | Non-native-language user | direct | Investment Banker Yuna Ahn: sensitive translation requires consent; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 14 | Low-bandwidth / disaster-zone / offline-first | watch | Investment Banker Yuna Ahn: offline audit retention and sync; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 15 | Banking / financial inclusion | direct | Investment Banker Yuna Ahn: low-tier financial inclusion path; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 16 | Activist / dissident | watch | Investment Banker Yuna Ahn: Tor-friendly metadata-minimized mode; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 17 | Regulator-deadline outage | direct | Investment Banker Yuna Ahn: degraded deadline-preserving workflow; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 18 | Audit / regulator / law-enforcement access | watch | Investment Banker Yuna Ahn: lawful-scope read-only evidence; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 19 | Tenant break-glass / dead-account recovery | direct | Investment Banker Yuna Ahn: ombudsman quorum and Shamir recovery; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 20 | Cognitive impairment / post-trauma | watch | Investment Banker Yuna Ahn: slow-down nudges without autonomy loss; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 21 | Pseudonymous + privacy-by-default | direct | Investment Banker Yuna Ahn: public identity separated from compliance identity; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 22 | Disaster-zone surge | watch | Investment Banker Yuna Ahn: cell isolation plus emergency rate floor; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 23 | Cross-jurisdiction conflict | direct | Investment Banker Yuna Ahn: higher-restriction pack wins; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 24 | Account-hijack victim recovery | watch | Investment Banker Yuna Ahn: hardware-key recovery and mutation cool-down; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 25 | Mistaken action / unintended mutation | direct | Investment Banker Yuna Ahn: 15s undo and rare high-value confirmation; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 26 | Concurrent-session conflict | watch | Investment Banker Yuna Ahn: due-process session conflict handling; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 27 | Bug bounty submitter | direct | Investment Banker Yuna Ahn: security-researcher allow-list and safe harbor; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 28 | Delegated agent acting for human | watch | Investment Banker Yuna Ahn: attested delegation chain; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 29 | High-net-worth transaction limits | direct | Investment Banker Yuna Ahn: KYB-verified transaction tier; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |
| 30 | Regional outage degradation | watch | Investment Banker Yuna Ahn: DR-pair failover within residency boundary; applies under `B2B_BANK_INTERNAL + B2C_CONSUMER`. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.

## §K. Their Journey Range

- Primary range: j001-j150 banking, deal room, personal MBA, audit.
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

- Place/time: Yeouido M&A data room, Seoul, 01:20 KST after a late investor call while the KRX pre-open watchlist is being refreshed.
- Current named tools: DealCloud 2026.1 for pipeline, Intralinks VIA Pro 2026.02 for data-room permissions, Bloomberg Terminal 2026 with B-PIPE entitlement, and Symphony 2026.1 for regulated chat.
- Named pain points: deal `Project HanRiver` exposed a buyer-side model to an unclean team for 11 minutes, risked a ₩2.1b fee clawback, and took 9h to reconstruct MNPI access logs.
- Jobs-to-be-done: weekly data-room access review and buyer update; quarterly compliance OKR "zero unclean-team exports"; yearly FINRA/SEC/KR-FSC training, banker licensing, and MBA-applicant personal boundary audit.
- Cedar binding: principal `User::"yuna-ahn@ibank-kr"` accesses `DealRoom::*`, `MNPIModel::*`, `CleanTeamLog::*`, and `MBAApplication::*`; actions `grant_data_room_access`, `quarantine_mnpi_export`, `send_buyer_update`, and `deny_personal_app_read`.
- Cross-context bridges: works with CFO Helena Brandt on acquisition financing, separates investor messaging from Lev Kahn's IR tenant, and keeps Aanya Kapoor's LP requests in sponsor-approved scope.
- Journey IDs: `j120-tenant-treasury-multi-currency-fx-hedge`, `j121-business-loan-application-from-bank-tenant`, `j159-saanvi-mehta-mba-application-spans-personal-and-work`.

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
