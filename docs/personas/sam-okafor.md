---
doc_class: PersonaDossier
shape: Reference
status: Proposed
date: 2026-05-21
persona_priority: 9
persona_slug: sam-okafor
persona_name: Sam Okafor
primary_role: corporate internal-audit director
primary_collar: white
primary_workspace: middle-office
skill_tier: senior
primary_device: desktop-primary + secure mobile
locale: NG
audience_type_primary: B2B_INTERNAL_AUDIT + B2C_CONSUMER
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
journey_range: j137-j141 SOX, fraud, Cedar misuse, DLP
---

# Persona Dossier — Sam Okafor

## §A. Archetype

Sam Okafor is priority 09 in the 2026-05-21 oyatie persona roster. The active projection is **corporate internal-audit director**. This dossier treats the persona as a projection of one passkey-bound human, not a separate account.

Career arc:
- Entry: enters oyatie through `B2B_INTERNAL_AUDIT + B2C_CONSUMER`.
- Growth: gains capability tiers without gaining ambient cross-tenant authority.
- Operational: depends on identity, audit-chain, compliance, governance, workflow-engine, payments.
- Cross-context: personal, professional, regulated, family, and side-business contexts remain separate.
- Recovery: ADR-0299 restores identity without reviving revoked tenant roles.
- Precedent: Apple Personal versus Apple Business and Microsoft personal versus work/school, but enforced through Cedar and tenant IDs.
- Count authority: 56 µservices for this brief.
- Layout authority: ADR-0131 flat per-µservice layout.
- Layer authority: ADR-0105 13-layer canonical enum.
- Community authority: `microservices/community/PRD.md`; no `anonymous/` path.

Tenure and universality:
- Collar-color: `white`.
- Workspace: `middle-office`.
- Skill tier: `senior`.
- Device: `desktop-primary + secure mobile`.
- Locale: `NG`.
- Same human, many contexts.
- One passkey root, many tenant memberships.
- Audience type belongs to tenant context, not the biological human.
- Cedar default-deny is the starting point.
- Audit-chain evidence is emitted for consequential changes.

## §B. Device Profile

- Primary device: `desktop-primary + secure mobile`.
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

- Primary locale: `NG`.
- Audience types: `B2B_INTERNAL_AUDIT + B2C_CONSUMER`.
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

- `Sam-at-work` is the same human under another tenant or role projection.
- `Sam-as-consumer` is the same human under another tenant or role projection.

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
| 05:30 | Sam Okafor acts as corporate internal-audit director. | identity + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 06:15 | Sam Okafor acts as corporate internal-audit director. | audit-chain + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 07:00 | Sam Okafor acts as corporate internal-audit director. | compliance + governance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 08:00 | Sam Okafor acts as corporate internal-audit director. | governance + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 09:00 | Sam Okafor acts as corporate internal-audit director. | workflow-engine + payments | Active tenant banner is visible and Cedar evaluates before mutation. |
| 10:00 | Sam Okafor acts as corporate internal-audit director. | payments + mail | Active tenant banner is visible and Cedar evaluates before mutation. |
| 11:00 | Sam Okafor acts as corporate internal-audit director. | mail + messenger | Active tenant banner is visible and Cedar evaluates before mutation. |
| 12:00 | Sam Okafor acts as corporate internal-audit director. | messenger + drive | Active tenant banner is visible and Cedar evaluates before mutation. |
| 13:00 | Sam Okafor acts as corporate internal-audit director. | drive + observability | Active tenant banner is visible and Cedar evaluates before mutation. |
| 14:00 | Sam Okafor acts as corporate internal-audit director. | observability + analytics | Active tenant banner is visible and Cedar evaluates before mutation. |
| 15:00 | Sam Okafor acts as corporate internal-audit director. | analytics + identity | Active tenant banner is visible and Cedar evaluates before mutation. |
| 16:00 | Sam Okafor acts as corporate internal-audit director. | identity + audit-chain | Active tenant banner is visible and Cedar evaluates before mutation. |
| 17:00 | Sam Okafor acts as corporate internal-audit director. | audit-chain + compliance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 18:00 | Sam Okafor acts as corporate internal-audit director. | compliance + governance | Active tenant banner is visible and Cedar evaluates before mutation. |
| 19:00 | Sam Okafor acts as corporate internal-audit director. | governance + workflow-engine | Active tenant banner is visible and Cedar evaluates before mutation. |
| 20:30 | Sam Okafor acts as corporate internal-audit director. | workflow-engine + payments | Active tenant banner is visible and Cedar evaluates before mutation. |
| 22:00 | Sam Okafor acts as corporate internal-audit director. | payments + mail | Active tenant banner is visible and Cedar evaluates before mutation. |

Day summary:
- Foreground surfaces: identity, audit-chain, compliance, governance, workflow-engine, payments, mail, messenger.
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
- `identity` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.
- `audit-chain` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.
- `compliance` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.
- `governance` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.
- `workflow-engine` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.
- `payments` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.
- `mail` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.
- `messenger` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.
- `drive` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.
- `observability` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.
- `analytics` capability tier: Sam Okafor needs tenant-scoped read/write/decide posture with audit evidence.

56-µservice adjacency matrix:

| # | µservice | Persona stance | Why it matters |
|---:|---|---|---|
| 01 | identity | primary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 02 | tenancy | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 03 | policy-engine | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 04 | audit-chain | primary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 05 | workflow-engine | primary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 06 | workflow-studio | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 07 | community | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 08 | messenger | secondary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 09 | mail | secondary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 10 | calendar | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 11 | meet | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 12 | drive | secondary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 13 | notes | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 14 | forms | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 15 | payments | primary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 16 | finops-portal | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 17 | marketplace | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 18 | ontology | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 19 | intelligence | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 20 | observability | secondary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 21 | compliance | primary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 22 | governance | primary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 23 | ops-dashboard-control-center | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 24 | workplace-integration | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 25 | developer-sdk | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 26 | foundry | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 27 | api-gateway | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 28 | cell | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 29 | cloud-secrets | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 30 | analytics | secondary | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 31 | search | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 32 | notifications | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 33 | social | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 34 | shorts | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 35 | ads | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 36 | personal-health-tracker | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 37 | crm | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 38 | marketing-automation | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 39 | contact-center | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 40 | performance-mgmt | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 41 | learning-mgmt | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 42 | itsm | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 43 | incident-mgmt | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 44 | financial-planning | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 45 | data-warehouse | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 46 | contract-lifecycle-mgmt | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 47 | whiteboard | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 48 | design-collaboration | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 49 | erp-finance | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 50 | erp-procurement | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 51 | erp-inventory | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 52 | erp-manufacturing | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 53 | erp-sales | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 54 | erp-hr | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 55 | erp-projects | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |
| 56 | erp-analytics | ambient | Sam Okafor must preserve tenant scope, audit events, and active-context UX if this surface is reached. |

## §H. Cedar Permit Shape

```cedar
permit (
  principal == User::"sam-okafor@active-tenant",
  action in ActionGroup::"sam-okafor.allowed_actions",
  resource in Tenant::"active-tenant"
)
when {
  context.audience_type in ["B2B_INTERNAL_AUDIT", "B2C_CONSUMER"] &&
  context.persona_projection == "sam-okafor" &&
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
| SOX-404 | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Sam Okafor. |
| Nigeria-NDPA | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Sam Okafor. |
| insider-risk | Locale, role, tenant, or data class activates it. | Adds retention, evidence, lawful-basis, or Cedar reason-code rules for Sam Okafor. |
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
| 1 | Emergency services | direct | Sam Okafor: audit and attest without blocking life-safety; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 2 | Account recovery / lockout | watch | Sam Okafor: passkey backup, recovery code, and trusted contact; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 3 | Financial fraud dispute + chargeback | direct | Sam Okafor: PSP-integrated fast-track dispute; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 4 | Elder financial abuse | watch | Sam Okafor: cooling-off and trusted-contact alert; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 5 | Healthcare urgent care + EHR break-glass | direct | Sam Okafor: post-hoc audit-and-justify; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 6 | Whistleblower + ethics report | watch | Sam Okafor: anonymous sealed chain of custody; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 7 | Press freedom / journalist source | direct | Sam Okafor: metadata-minimized source protection; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 8 | Domestic violence / abuse survivor | watch | Sam Okafor: silent shelter mode; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 9 | Child safety + mandatory reporting | direct | Sam Okafor: mandatory-reporter route cannot be suppressed; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 10 | Deceased-user account | watch | Sam Okafor: legacy contact plus court-order path; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 11 | Custody / shared-account dispute | direct | Sam Okafor: family-court order integration; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 12 | Disability accommodations | watch | Sam Okafor: accessibility profile overrides friction defaults; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 13 | Non-native-language user | direct | Sam Okafor: sensitive translation requires consent; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 14 | Low-bandwidth / disaster-zone / offline-first | watch | Sam Okafor: offline audit retention and sync; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 15 | Banking / financial inclusion | direct | Sam Okafor: low-tier financial inclusion path; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 16 | Activist / dissident | watch | Sam Okafor: Tor-friendly metadata-minimized mode; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 17 | Regulator-deadline outage | direct | Sam Okafor: degraded deadline-preserving workflow; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 18 | Audit / regulator / law-enforcement access | watch | Sam Okafor: lawful-scope read-only evidence; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 19 | Tenant break-glass / dead-account recovery | direct | Sam Okafor: ombudsman quorum and Shamir recovery; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 20 | Cognitive impairment / post-trauma | watch | Sam Okafor: slow-down nudges without autonomy loss; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 21 | Pseudonymous + privacy-by-default | direct | Sam Okafor: public identity separated from compliance identity; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 22 | Disaster-zone surge | watch | Sam Okafor: cell isolation plus emergency rate floor; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 23 | Cross-jurisdiction conflict | direct | Sam Okafor: higher-restriction pack wins; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 24 | Account-hijack victim recovery | watch | Sam Okafor: hardware-key recovery and mutation cool-down; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 25 | Mistaken action / unintended mutation | direct | Sam Okafor: 15s undo and rare high-value confirmation; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 26 | Concurrent-session conflict | watch | Sam Okafor: due-process session conflict handling; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 27 | Bug bounty submitter | direct | Sam Okafor: security-researcher allow-list and safe harbor; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 28 | Delegated agent acting for human | watch | Sam Okafor: attested delegation chain; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 29 | High-net-worth transaction limits | direct | Sam Okafor: KYB-verified transaction tier; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |
| 30 | Regional outage degradation | watch | Sam Okafor: DR-pair failover within residency boundary; applies under `B2B_INTERNAL_AUDIT + B2C_CONSUMER`. |

Edge-case synthesis:
- Safety cannot defeat security.
- Security cannot defeat safety.
- Policy cannot be ignored.
- Critical paths use audited exceptions.
- Generic CAPTCHA on recovery is forbidden.
- Anonymous whistleblower binding to caller identity is forbidden.
- Survivor audit visibility cannot be shared with an abuser or tenant admin.

## §K. Their Journey Range

- Primary range: j137-j141 SOX, fraud, Cedar misuse, DLP.
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

- Place/time: Eko Atlantic Tower B, Victoria Island, Lagos, 20:30 WAT during rainy-season traffic while the SOX sample close meeting runs late.
- Current named tools: SAP GRC Access Control 12 SP24 for SoD review, Splunk Enterprise Security 8.1 for DLP signals, AuditBoard SOXHUB 2026.1 for workpapers, and Microsoft Purview DLP E5 build 2026.03.
- Named pain points: payment chain `AP-2026-0447` missed signer evidence on a $640,000 vendor batch, required 21 hours of reconstruction, and almost pulled an employee personal Drive link into the workpaper set.
- Jobs-to-be-done: weekly SOX control sample and exception aging; quarterly fraud-pattern investigation against payments and workflow logs; yearly audit-committee packet with personal-tenant denial evidence.
- Cedar binding: principal `User::"sam-okafor@krampuscorp-audit"` accesses `WorkMessengerArchive::*`, `WorkMailArchive::*`, `PaymentApproval::*`, and `AuditFinding::*`; actions `sample_work_surface`, `open_fraud_case`, `request_cedar_scope_review`, and `seal_audit_finding`.
- Cross-context bridges: receives HR lawful-basis attestations from Priya Krishnan, reports material weakness summaries to Marcus Chen, and coordinates external sampling boundaries with Diana Reyes.
- Journey IDs: `j137-corporate-internal-audit-sox-controls-test`, `j138-corporate-audit-fraud-investigation-via-pattern-detection`, `j139-internal-audit-policy-violation-cedar-permit-misuse`, `j140-internal-audit-data-loss-prevention-egress-trip`, `j141-internal-audit-respects-employee-personal-tenant-boundary`.

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
