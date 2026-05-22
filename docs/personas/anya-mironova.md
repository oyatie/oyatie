---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 6
persona_slug: anya-mironova
persona_name: Anya Mironova
primary_role: investigative journalist, activist, parent
primary_collar: white
primary_workspace: field
skill_tier: senior
primary_device: secure laptop + burner mobile + Tor ingress
locale: EU
audience_type_primary: B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER
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
journey_range: j001-j150 journalist-source, privacy, parent, high-risk user
---

# Persona Dossier — Anya Mironova

## §A. Archetype

Anya Mironova is priority 06 in the 2026-05-21 oyatie persona roster. The active projection is **investigative journalist, activist, parent**. This dossier treats the persona as a projection of one passkey-bound human, not a separate account.

Career arc:
- Entry: enters oyatie through `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`.
- Growth: gains capability tiers without gaining ambient cross-tenant authority.
- Operational: depends on identity, messenger, mail, drive, community, compliance.
- Cross-context: personal, professional, regulated, family, and side-business contexts remain separate.
- Recovery: ADR-0299 restores identity without reviving revoked tenant roles.
- Precedent: Apple Personal versus Apple Business and Microsoft personal versus work/school, but enforced through Cedar and tenant IDs.
- Count authority: 56 µservices for this brief.
- Layout authority: ADR-0131 flat per-µservice layout.
- Layer authority: ADR-0105 13-layer canonical enum.
- Community authority: `microservices/community/PRD.md`; no `anonymous/` path.

Tenure and universality:
- Collar-color: `white`.
- Workspace: `field`.
- Skill tier: `senior`.
- Device: `secure laptop + burner mobile + Tor ingress`.
- Locale: `EU`.
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.

## §B. Device Profile

- Primary device: `secure laptop + burner mobile + Tor ingress`.
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

- Primary locale: `EU`.
- Audience types: `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`.
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

- `Anya-as-journalist` is the same human under another tenant or role projection.
- `Anya-as-parent` is the same human under another tenant or role projection.
- `Anya-as-activist` is the same human under another tenant or role projection.

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
| 05:30 | Anya Mironova acts as investigative journalist. | identity + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Anya Mironova acts as investigative journalist. | messenger + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Anya Mironova acts as investigative journalist. | mail + drive | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Anya Mironova acts as investigative journalist. | drive + community | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Anya Mironova acts as investigative journalist. | community + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Anya Mironova acts as investigative journalist. | compliance + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Anya Mironova acts as investigative journalist. | audit-chain + policy-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Anya Mironova acts as investigative journalist. | policy-engine + workflow-studio | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Anya Mironova acts as investigative journalist. | workflow-studio + notes | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Anya Mironova acts as investigative journalist. | notes + shorts | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Anya Mironova acts as investigative journalist. | shorts + identity | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Anya Mironova acts as investigative journalist. | identity + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Anya Mironova acts as investigative journalist. | messenger + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Anya Mironova acts as investigative journalist. | mail + drive | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Anya Mironova acts as investigative journalist. | drive + community | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Anya Mironova acts as investigative journalist. | community + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Anya Mironova acts as investigative journalist. | compliance + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, messenger, mail, drive, community, compliance, audit-chain, policy-engine.
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
- `identity` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.
- `drive` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.
- `community` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.
- `compliance` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.
- `policy-engine` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-studio` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.
- `notes` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.
- `shorts` capability tier: Anya Mironova needs tenant-scoped read/write/decide posture with audit evidence.

56-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | identity | primary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 02 | tenancy | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 03 | policy-engine | secondary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 04 | audit-chain | secondary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 05 | workflow-engine | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 06 | workflow-studio | secondary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 07 | community | primary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 08 | messenger | primary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 09 | mail | primary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 10 | calendar | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 11 | meet | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 12 | drive | primary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 13 | notes | secondary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 14 | forms | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 15 | payments | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 16 | finops-portal | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 17 | marketplace | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 18 | ontology | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 19 | intelligence | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 20 | observability | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 21 | compliance | primary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 22 | governance | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 23 | ops-dashboard-control-center | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 24 | workplace-integration | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 25 | developer-sdk | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 26 | foundry | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 27 | api-gateway | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 28 | cell | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 29 | cloud-secrets | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 30 | analytics | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 31 | search | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 32 | notifications | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 33 | social | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 34 | shorts | secondary | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 35 | ads | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 36 | personal-health-tracker | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 37 | crm | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 38 | marketing-automation | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 39 | contact-center | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 40 | performance-mgmt | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 41 | learning-mgmt | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 42 | itsm | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 43 | incident-mgmt | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 44 | financial-planning | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 45 | data-warehouse | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 46 | contract-lifecycle-mgmt | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 47 | whiteboard | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 48 | design-collaboration | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 49 | erp-finance | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 50 | erp-procurement | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 51 | erp-inventory | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 52 | erp-manufacturing | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 53 | erp-sales | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 54 | erp-hr | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 55 | erp-projects | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 56 | erp-analytics | ambient | Anya Mironova must preserve tenant scope, audit events, and active-context UX if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"anya-mironova@active-tenant",
  action in ActionGroup::"anya-mironova.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2C_CONSUMER", "B2C_FAMILY_PARENT", "HIGH_RISK_USER"] &&
  context.persona_projection == "anya-mironova" &&
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
| GDPR | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Anya Mironova. |
| publisher-source-protection | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Anya Mironova. |
| DSA | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Anya Mironova. |
| EU-whistleblower | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Anya Mironova. |
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
| 1 | Emergency services | watch | Anya Mironova: audit and attest without blocking life-safety; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 2 | Account recovery / lockout | direct | Anya Mironova: passkey backup, recovery code, and trusted contact; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 3 | Financial fraud dispute + chargeback | watch | Anya Mironova: PSP-integrated fast-track dispute; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 4 | Elder financial abuse | direct | Anya Mironova: cooling-off and trusted-contact alert; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 5 | Healthcare urgent care + EHR break-glass | watch | Anya Mironova: post-hoc audit-and-justify; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 6 | Whistleblower + ethics report | direct | Anya Mironova: anonymous sealed chain of custody; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 7 | Press freedom / journalist source | watch | Anya Mironova: metadata-minimized source protection; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 8 | Domestic violence / abuse survivor | direct | Anya Mironova: silent shelter mode; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 9 | Child safety + mandatory reporting | watch | Anya Mironova: mandatory-reporter route cannot be suppressed; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 10 | Deceased-user account | direct | Anya Mironova: legacy contact plus court-order path; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 11 | Custody / shared-account dispute | watch | Anya Mironova: family-court order integration; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 12 | Disability accommodations | direct | Anya Mironova: accessibility profile overrides friction defaults; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 13 | Non-native-language user | watch | Anya Mironova: sensitive translation requires consent; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 14 | Low-bandwidth / disaster-zone / offline-first | direct | Anya Mironova: offline audit retention and sync; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 15 | Banking / financial inclusion | watch | Anya Mironova: low-tier financial inclusion path; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 16 | Activist / dissident | direct | Anya Mironova: Tor-friendly metadata-minimized mode; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 17 | Regulator-deadline outage | watch | Anya Mironova: degraded deadline-preserving workflow; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 18 | Audit / regulator / law-enforcement access | direct | Anya Mironova: lawful-scope read-only evidence; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 19 | Tenant break-glass / dead-account recovery | watch | Anya Mironova: ombudsman quorum and Shamir recovery; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 20 | Cognitive impairment / post-trauma | direct | Anya Mironova: slow-down nudges without autonomy loss; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 21 | Pseudonymous + privacy-by-default | watch | Anya Mironova: public identity separated from compliance identity; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 22 | Disaster-zone surge | direct | Anya Mironova: cell isolation plus emergency rate floor; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 23 | Cross-jurisdiction conflict | watch | Anya Mironova: higher-restriction pack wins; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 24 | Account-hijack victim recovery | direct | Anya Mironova: hardware-key recovery and mutation cool-down; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 25 | Mistaken action / unintended mutation | watch | Anya Mironova: 15s undo and rare high-value confirmation; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 26 | Concurrent-session conflict | direct | Anya Mironova: due-process session conflict handling; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 27 | Bug bounty submitter | watch | Anya Mironova: security-researcher allow-list and safe harbor; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 28 | Delegated agent acting for human | direct | Anya Mironova: attested delegation chain; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 29 | High-net-worth transaction limits | watch | Anya Mironova: KYB-verified transaction tier; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |
| 30 | Regional outage degradation | direct | Anya Mironova: DR-pair failover within residency boundary; applies under `B2C_CONSUMER + B2C_FAMILY_PARENT + HIGH_RISK_USER`. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.

## §K. Their Journey Range

- Primary range: j001-j150 journalist-source, privacy, parent, high-risk user.
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

- Place/time: Cafe St. Oberholz at Rosenthaler Platz, Berlin-Mitte, 18:45 CET during icy rain while a protected-source upload is still hashing.
- Current named tools: Signal Desktop 7.8 with sealed sender, Proton Mail Visionary 5.0 with address aliases, SecureDrop Workstation 2.11 for source intake, and Obsidian 1.7 Sync for encrypted notes.
- Named pain points: the 2025 "Danube Ledger" draft preserved EXIF timezone metadata, triggered €7,800 in emergency counsel review, and delayed publication 36 hours while source-risk redaction was rebuilt.
- Jobs-to-be-done: weekly protected-source triage and publication risk board; quarterly "Source Safety OKR" reducing metadata-bearing drafts to zero; yearly press-freedom grant audit and cross-border legal review.
- Cedar binding: principal `User::"anya-mironova@press-eu"` accesses `SourceDrop::*`, `DraftArticle::*`, `LegalHold::*`, and `FamilyCalendar::*`; actions `decrypt_source_packet`, `request_counsel_review`, `publish_redacted_story`, and `mute_family_notifications`.
- Cross-context bridges: obtains privileged review from Outside Counsel Wei-Yi Chen, coordinates lawful-public-safety questions with Officer Rodriguez only through warrant scope, and keeps child-school records separate from Ms. Patel's education tenant.
- Journey IDs: `j06-press-source-securedrop-class`, `j17-activist-dissident-high-risk-mode`.

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
