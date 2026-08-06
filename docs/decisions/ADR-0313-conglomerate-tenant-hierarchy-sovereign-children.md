---
id: ADR-0313
status: Rejected
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-privacy
  - council-security
  - council-legal-regulatory
  - axis-tenancy
  - axis-identity
  - axis-policy-engine
  - axis-audit-chain
  - axis-finops
  - ops-compliance
  - ops-sre-reliability
supersedes: []
amends:
  - ADR-0244-tenant-as-universal-scoping-primitive.md (§D-3 schema extension — adds `controls_tenants` / `controlled_by_tenants` denormalized index columns; cross-references new `conglomerate_grants` source-of-truth table)
superseded_by: []
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-finops-sustainability-tagging.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0215-multi-context-platform.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0276-backup-portability-format-gdpr-article-20.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0299-cross-pack-data-residency-conflict-arbitration.md
  - ADR-0304-cross-jurisdiction-conflict-resolution.md
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md
  - ADR-0312-court-warrant-scoped-piercing.md
  - ADR-0319-front-middle-back-office-information-barrier.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/tenant-model.json
  - /specs/conglomerate-grant-model.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/identity.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/finops-portal.json
  - /specs/microservices/ops-dashboard-control-center.json
  - /specs/microservice-manifest-schema.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_tenant_as_universal_scoping_primitive
  - feedback_bominal_inheritance_precedence
  - feedback_no_silent_regression
  - feedback_canonical_base_localization
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_autonomous_decision_principles
  - feedback_self_modification_doctrine
  - feedback_compliance_pack_primitive
doc_class: Architecture-Decision-Record
purpose: >
  Establish the conglomerate-tenant hierarchy model: child tenants are
  fully sovereign tenants (per ADR-0244); parent/child relationships
  are expressed entirely through Cedar permits (per ADR-0243) against
  the source-of-truth `conglomerate_grants` table; restructuring
  (spinoff, divestiture, acquisition, IPO, joint venture, bankruptcy)
  is a 1-step Cedar revocation + grant rather than a multi-quarter
  data migration. The controlling-entity grant is bounded by
  per-jurisdiction corporate-governance attestation, cross-jurisdiction
  residency preservation (ADR-0304), the personal/work boundary
  (ADR-0311), and court-warrant scoping (ADR-0312). Audit-chain
  dual-sealing makes every parent action observable to both audit
  streams. This ADR is the conglomerate-layer companion to ADR-0244's
  flat-tenant primitive.
enforcement_status: advisory-until-conglomerate-substrate-lands
enforced_by:
  - oya-governance-conglomerate-grant-attestation-current
  - oya-governance-cross-jurisdiction-residency-preserved
  - oya-governance-conglomerate-grant-dual-sealed
  - oya-governance-conglomerate-grant-personal-tenant-deny
  - oya-governance-conglomerate-information-barrier-coverage
  - oya-governance-conglomerate-grant-transitivity-deny
---

# ADR-0313: Conglomerate-Tenant Hierarchy — Sovereign-Child + Policy-Engine-Mediated Controlling-Entity Grant

## Status

Proposed — 2026-05-20.

This ADR layers atop the 14-ADR foundational keystone bundle of
2026-05-20 (ADR-0242 through ADR-0255 inclusive) and lands as the
conglomerate-layer companion to ADR-0244 (tenant as universal scoping
primitive). Each keystone references the others; this ADR is *not*
itself a keystone (it does not introduce a new substrate primitive
beyond Cedar + tenancy), but its acceptance is required before any
holding-company, multi-subsidiary, joint-venture, or platform-of-
platforms customer can onboard to oyatie at production scale.

Enforcement is `advisory-until-conglomerate-substrate-lands`. The
doctrine is accepted in text now; the CI lanes that enforce it move to
BLOCKER status only after:

1. `microservices/tenancy/` admits the `conglomerate_grants` source-
   of-truth table via migration `0014_conglomerate_grants.sql` (built
   atop the `0002_canonical_tenant_schema.sql` migration from
   ADR-0244).
2. `microservices/tenancy/` adds the two denormalized index columns
   (`controls_tenants`, `controlled_by_tenants`) to the `tenants`
   table via migration `0015_tenant_conglomerate_index_columns.sql`.
3. The Cedar entity-type `ControllingEntity` is loaded into
   `microservices/policy-engine/` (per ADR-0243 + ADR-0246) along
   with the `ParentScope::ReadActions`, `ParentScope::WriteActions`,
   `ParentScope::AuditActions`, `ParentScope::JointVentureActions`,
   and `ParentScope::PaymentFacilitationActions` action namespaces.
4. `microservices/audit-chain/` provisions dual-sealed write paths for
   conglomerate read-actions (every parent read of a child seals in
   *both* parent's and child's audit stream — per §D-4 invariant 6).
5. The new crate `oya-shared-conglomerate-grant-evaluator` lands
   (per §E.1) and is consumed by `ops-dashboard-control-center`,
   `finops-portal`, `audit-chain`, and `tenancy`.
6. The six CI lanes enumerated in §D-8 / §E.4 are wired and producing
   per-grant evidence.

Until those six items land, validators emit findings without failing
CI. Post-substrate, the six lanes promote to BLOCKER per the
2026-07-15 deadline established by the 2026-05-20 keystone bundle.

## Date

2026-05-20.

## §A Context

### §A.1 The conglomerate problem — first-class in real commerce, ignored by most SaaS

Holding companies, parent/subsidiary relationships, multi-brand
groups, joint ventures, sovereign-wealth-fund portfolios,
private-equity rolls, family offices, conglomerate-of-conglomerates
(Berkshire-Hathaway, SoftBank Group, Samsung Group, LG Corp,
SK Holdings, Hanwha Group, Jardine Matheson, Tata Sons, Reliance
Industries, Mitsubishi Group keiretsu, Volkswagen Group, LVMH,
Mondelēz Carve-Outs, Kraft-Heinz / 3G Capital, Dell Technologies +
VMware spinoff, AT&T + WarnerMedia spin/merge, Pfizer + Viatris
spinoff, Disney + 21st Century Fox acquisition) are pervasive
structures in real-world commerce. The Korean *chaebol*, Japanese
*keiretsu*, Indian *industrial-house* and European
*Konzern* structures are first-class economic primitives. A platform
that does not model conglomerate relationships natively forces the
holding company to either:

1. **Run separate tenants per subsidiary with no consolidated view** —
   destroys the entire purpose of the holding structure (consolidated
   financial reporting, group-wide operations dashboards, group-wide
   compliance posture, group-wide procurement, group-wide audit).
2. **Run one tenant for the whole group with embedded sub-divisions**
   — destroys subsidiary sovereignty (a divested subsidiary cannot
   walk out of the platform without a data migration; an acquired
   subsidiary cannot retain its prior platform state; jurisdictional
   ring-fences cannot be enforced; per-subsidiary residency is lost;
   regulators that require subsidiary-level reporting cannot get
   subsidiary-scoped audit streams).

Both approaches fail. Hyperscalers solved this problem a decade ago.
oyatie inherits the solved pattern.

### §A.2 Hyperscaler precedent — the same primitive everywhere

Every named hyperscaler operates with two-layer tenancy: **sovereign
leaf tenants + policy-mediated controlling-entity relationships**.
The pattern is not invented here; this ADR adopts what AWS / Microsoft
/ Google / Stripe / Salesforce / Bloomberg / Apple / Atlassian / Okta
all converged on.

- **AWS Organizations + Management Account + Consolidated Billing +
  Cross-Account IAM Roles.** AWS Accounts are fully sovereign tenants
  (their own root, their own IAM, their own audit-chain via
  CloudTrail). An Organization is an AWS account that holds Cedar-
  equivalent permits (Service Control Policies + Cross-Account IAM
  Roles) against other AWS Accounts. Consolidated billing is a
  permit-mediated read against the leaf-account billing data, not a
  data-migration. Spinoff = remove the account from the Organization;
  the leaf account continues unchanged with its own root credentials.
  Acquisition = invite the leaf account into the new Organization
  with a new Service Control Policy + cross-account role. No data
  migration, no identity re-issuance, no resource re-binding.
  (Source: AWS Organizations User Guide 2024 ed. ch. 3 "Account
  management"; AWS re:Invent 2023 SEC305 "Multi-account architectures
  at scale"; AWS Whitepaper "Organizing your AWS environment using
  multiple accounts" Aug 2024 rev.)

- **Microsoft 365 Tenant Hierarchy + Cross-Tenant Synchronization +
  Entitlement Management + Multi-Tenant Organizations (MTO).**
  Each M365 tenant (Entra ID / Azure AD tenant) is fully sovereign:
  its own users, its own licensing, its own audit-stream
  (Microsoft Purview audit log). Cross-tenant relationships are
  expressed via Multi-Tenant Organization (MTO, GA 2024), Cross-Tenant
  Sync, B2B Direct Connect, and Entitlement Management. None of these
  is a data-merge — each is a permit-mediated grant. A divestiture
  exits the MTO; the leaf tenant retains its identity, audit-chain,
  and data residency. (Source: Microsoft Entra ID Docs 2024 "Multi-
  tenant organizations overview"; Microsoft Build 2024 keynote +
  IDN508 session; Microsoft Purview audit-log cross-tenant search
  documentation 2024.)

- **Google Workspace Customer Hierarchy + Reseller Console +
  Google Cloud Organization → Folder → Project.** Workspace customers
  are sovereign; reseller relationships are permit grants from
  reseller to customer with explicit scope (billing-only, full-admin,
  break-glass). Spinoff of a customer from a reseller = revoke the
  grant; the customer retains its own data, identity, and audit.
  Google Cloud's Organization → Folder → Project hierarchy is a
  permit-cascade overlay on independent Projects (each Project is a
  resource container; the hierarchy is an IAM-binding cascade, not a
  data-ownership cascade). (Source: Google Workspace Admin Help
  2024 "Reseller features"; Google Cloud Resource Manager Docs 2024
  "Resource hierarchy"; Google CRE Book ch. 8 "Managing change at
  scale.")

- **Stripe Tenant/RBAC Packaging Account → Connected Account.** The
  platform-facilitator pattern is the single most-cited industry
  reference for this ADR. Stripe platforms (Shopify, Lyft,
  DoorDash, Booking.com, Squarespace) hold permit grants against
  connected accounts (the merchant subsidiaries). Each connected
  account is sovereign (its own Stripe Account ID, its own balance,
  its own KYC, its own bank account, its own 1099-K). The platform's
  control is scoped (payment-facilitation, payout-routing, fee-take,
  refunds). On platform exit, the connected account retains its
  Stripe identity and migrates to direct relationship — no data loss,
  no merchant re-onboarding. (Source: Stripe Engineering Blog 2024
  "Designing for global platforms"; Stripe API Reference
  2025 ed. "Accounts" + "Application Fees" + "Transfers"; Stripe
  Sessions 2024 "platform mechanics" talk.)

- **Salesforce Org Hierarchy + Salesforce Multi-Org Reporting +
  Salesforce Connect.** Each Salesforce Org is a sovereign tenant
  (its own metadata, its own data, its own audit-trail). Multi-Org
  reporting is a permit-mediated federation, not a data merge.
  Customer-controlled Org-to-Org sync uses External Objects +
  Salesforce over OData; the leaf Org retains residency.
  (Source: Salesforce Architects 2024 "Multi-org strategy"; Trailhead
  module "Multi-Org Strategy and Architecture.")

- **Bloomberg Terminal Entitlement Hierarchy.** Bloomberg's
  entitlement model has a corporate-account → business-unit →
  user-seat hierarchy. Each business unit is a sovereign entitlement
  bundle; the corporate account holds *visibility* grants (consumption
  reporting, license attribution, compliance attestation) but does
  not own the leaf user data. Spinoff of a business unit = transfer
  the entitlement bundle to a new corporate account; the user seats
  retain their personalization, watchlists, and message history.
  (Source: Bloomberg Enterprise Solutions 2024 documentation;
  Bloomberg Compliance Center customer briefings 2024.)

- **Apple Business Manager → MDM Tenant Hierarchy.** Apple Business
  Manager (ABM) is the controlling tenant for an organization's
  device fleet. Each managed Apple ID is a sovereign principal under
  the org tenant; on employee offboarding, the principal can be
  detached (the personal Apple ID lives on independently — this is
  exactly the ADR-0311 dual-tenant boundary at Apple's layer).
  (Source: Apple Business Manager User Guide 2024 ed.; WWDC 2024
  "What's new in managing Apple devices" session.)

- **Atlassian Cloud Organization → Site Hierarchy.** Atlassian
  Organizations hold permit grants against sites (Jira, Confluence,
  Bitbucket workspaces). Each site retains its own admin, audit-log,
  and SCIM directory; the Organization layer provides centralized
  identity, billing, and compliance reporting. Site detachment on
  divestiture is a 1-step org-membership revocation. (Source:
  Atlassian Cloud Architecture documentation 2024 "Organizations
  and admin hub"; Atlassian Team '24 conference IDX-101 session.)

- **Okta Org-of-Orgs + Customer Identity Cloud (CIC) Tenants +
  Workforce-Identity Org Hierarchy.** Okta hub-and-spoke tenancy
  treats each spoke as sovereign with the hub holding policy grants
  for SSO, lifecycle management, and admin federation. The
  spoke retains its own user directory and audit; spinoff revokes
  the hub-to-spoke trust without re-issuing user credentials.
  (Source: Okta Architectural Guidance documentation 2024 "Hub-and-
  spoke architecture for federated identity"; Oktane '24 IAM-205
  session.)

- **Slack Enterprise Grid → Workspace Hierarchy.** Slack Enterprise
  Grid Organizations are the controlling tenant; each workspace
  underneath is a sovereign workspace with its own channels, members,
  and integrations. On divestiture, a workspace can be detached from
  the Grid and migrated to a standalone tier; messages and channels
  remain intact. (Source: Slack Enterprise Grid documentation 2024;
  Slack Frontiers 2024 "Designing for multi-org enterprises" session.)

The pattern across all nine is **identical at the architecture
layer**: sovereign leaves + permit-mediated controlling-entity grants.
This ADR adopts that pattern verbatim and grounds it in oyatie's
Cedar substrate (ADR-0243) and tenant substrate (ADR-0244).

### §A.3 The naive (wrong) approach — parent-owns-child as sub-scope

A naive design would model the parent/child relationship by embedding
the child as a sub-scope of the parent (per ADR-0244 §D-2 dotted
hierarchical sub-scope convention) — e.g.,
`snu-hospital-network.gangnam.*` and `snu-hospital-network.busan.*`.
This approach is wrong, and the failure modes were observable in
prior portfolio attempts before ADR-0244 landed:

1. **Divestiture becomes a multi-quarter data migration.** Selling
   `gangnam` to a new parent requires renaming every sub-scope row,
   re-issuing every principal slug, re-binding every Cedar fragment,
   re-sealing every audit-chain entry (because the audit-chain
   includes the principal slug), and re-keying the per-cell KMS
   envelope. The migration touches Postgres, Cedar fragment registry,
   audit-chain, KMS, observability dashboards, finops cost
   attribution, marketplace surface eligibility, plugin manifests,
   webhook subscriptions, and DNS. A 20-µservice platform takes 6-12
   months to migrate one subsidiary. The cost is prohibitive and the
   risk of audit-chain integrity loss is real.

2. **Subsidiary sovereignty is destroyed.** The child has no
   independent KYB, no independent cell binding, no independent
   audit-stream. Regulators that require subsidiary-level reporting
   (e.g., KR-FSS for regulated financial subsidiaries, SEC for
   public-company subsidiaries, EU-EBA for credit-institution
   subsidiaries) cannot get subsidiary-scoped evidence — the audit
   stream is the parent's stream with sub-scope filtering, which
   regulators do not accept as equivalent.

3. **Joint ventures cannot be expressed.** A JV with two parents
   cannot fit a sub-scope tree (sub-scopes are dotted-hierarchical,
   single-parent by construction per ADR-0244 §D-2). The naive
   model forces JVs into one of the two parents, which is wrong
   commercially (the JV is a separate legal entity) and regulatorily
   (the JV has its own KYB, its own audit, its own residency).

4. **Multi-level conglomerates (holding-of-holdings) require
   N-level transitivity rules.** Cedar evaluation is bounded-time
   decidable (per ADR-0150 §performance); transitive permit cascades
   across 5+ levels of holding companies cause super-linear
   evaluation cost and are difficult to reason about. The naive
   model forces the policy-engine to walk transitively up the
   sub-scope tree on every Cedar evaluation.

5. **Per-jurisdiction corporate-governance overlay is impossible.**
   The KR-Commercial-Act-Art-342 *지주회사* (holding company)
   definition imposes specific share-ownership and asset-share
   thresholds; the US Bank Holding Company Act imposes Federal
   Reserve approval requirements; the EU Companies Directive
   2017/1132 imposes different requirements. These overlays cannot
   apply to a sub-scope row — they apply to the controlling-entity
   *relationship*, which is exactly what the naive model loses.

6. **Cross-child information barriers (Glass-Steagall ring-fences,
   FERC ring-fences, MiFID II ring-fences, banking ring-fences,
   insurance ring-fences, accounting/audit independence ring-
   fences) cannot be enforced.** A bank holding company that
   contains both a regulated bank subsidiary and an investment-bank
   subsidiary must enforce a customer-data ring-fence between them.
   The sub-scope model makes the parent the data-owner of both, so
   the ring-fence collapses at the parent level (the parent's
   principals can read both subsidiaries' customer data trivially).

The naive model fails at every commercial and regulatory boundary.
The right model is the **sovereign-child + policy-mediated permit**
pattern, which this ADR adopts.

### §A.4 The right approach — child is sovereign; parent holds Cedar permit

Per user clarification 2026-05-21 (memorialized in the operating
contract): **child tenants are FULLY sovereign tenants per ADR-0244,
not sub-scopes or shadows of the parent.** The parent-child
conglomerate relationship is expressed entirely through Cedar permit
grants against the source-of-truth `conglomerate_grants` table, NOT
through ownership-of-data primitives. This is what makes spinoff /
divestiture / sale / IPO / bankruptcy / joint-venture all tractable:

```
oyatie::Tenant::"snu-hospital-network"      ← parent tenant (holding company)
oyatie::Tenant::"snu-hospital-gangnam"      ← child tenant (full sovereign tenant)
oyatie::Tenant::"snu-hospital-busan"        ← child tenant (full sovereign tenant)
```

Parent's control is a Cedar permit row, NOT an embedded ownership
flag. The illustrative shape:

```cedar
permit (
    principal in oyatie::Tenant::"snu-hospital-network"::Role::"controlling-entity",
    action in ParentScope::ReadActions,
    resource in oyatie::Tenant::"snu-hospital-gangnam"::*
) when {
    principal.kyb_attestation_includes_child("snu-hospital-gangnam")
    && principal.corporate_governance_proof_active("KR-Commercial-Act-Art-342")
    && context.cross_jurisdiction_check_passes(principal.tenant, resource.tenant)
    && context.personal_tenant_boundary_preserved(resource.tenant)
    && context.audit_chain_dual_seal_ready(principal.tenant, resource.tenant)
};
```

This is exactly the Stripe platform-facilitator pattern
generalized: instead of facilitator → connected-account, it is
controlling-entity → controlled-tenant, with a richer scope vocabulary
(read-only-financial / read-only-operational / read-write-board-
decisions / audit-only / cross-jurisdiction-read-only / joint-venture-
partial / payment-facilitation — see §D-3).

### §A.5 Why this is load-bearing for the rest of the portfolio

ADR-0243 (Cedar as universal gate) is the load-bearing primitive that
makes conglomerate restructuring tractable. Without policy-engine-
mediated control, divestiture would be a multi-quarter migration
(see §A.3 failure-mode 1). With Cedar-mediated control, divestiture
is a 1-step Cedar revocation + grant, executed by a Workflow Engine
saga (per ADR-0035) with full audit-chain dual-sealing.

The conglomerate-tenancy doctrine therefore composes with:

- **ADR-0242 (oyatie is a tenant).** oyatie's own tenant is itself a
  conglomerate parent of its sub-tenants (`oyatie.foundry`,
  `oyatie.ops`, `oyatie.engineering`, `oyatie.intelligence`); the
  same machinery applies recursively.
- **ADR-0243 (Cedar as universal gate).** Every parent read /
  write / audit action on a child resource passes through Cedar.
- **ADR-0244 (tenant as universal scoping primitive).** Every child
  is itself a full ADR-0244 tenant row (KYB, cell binding, audit-
  chain, residency, pack overlays).
- **ADR-0245 (substrate vs product).** The `conglomerate_grants`
  table lives in the tenancy substrate µservice, not in any product
  µservice.
- **ADR-0249 (multi-category marketplace).** Marketplace platforms
  (apps, plugins, workflows, agents, models, datasets) are
  conglomerate parents over their seller tenants — the seller
  retains identity and data ownership; the marketplace holds a
  payment-facilitation + listing-mediation Cedar permit.
- **ADR-0263 (observability emission contract).** Every Cedar
  decision for a conglomerate read-action emits a structured audit
  event from the §D-10 emission class registry.
- **ADR-0276 (backup portability format).** Spinoff with simultaneous
  platform-exit uses the GDPR-Art-20 portability format; the leaf
  tenant data is exported in canonical form for the new parent.
- **ADR-0297 (abuse-defence baseline).** Conglomerate permits do not
  bypass abuse-defence — a parent reading a child's data still
  passes through the anti-spoof, anti-scrape, and anti-bot controls
  on the child's surface.
- **ADR-0299 (cross-pack data residency conflict arbitration).**
  Parent reads from a different jurisdictional pack than the child
  pass through the §D-4 invariant 2 residency-preservation guard
  (see §D-4 below).
- **ADR-0304 (cross-jurisdiction conflict resolution).** Cross-
  jurisdiction parent/child relationships obey the per-pack
  conflict-resolution invariants ADR-0304 sets out.
- **ADR-0311 (dual-tenant identity personal-vs-work boundary).** A
  parent's controlling-entity permit cannot pierce the personal
  tenant of a child's employee. The dual-tenant boundary holds
  across parent/child by construction.
- **ADR-0312 (court-warrant-scoped piercing).** A court-warrant
  granting access to a subsidiary does not auto-cascade to the
  parent — and vice versa; piercing remains scope-bounded.

### §A.6 What changes; what stays

**Changes:**

- A new source-of-truth table `conglomerate_grants` is added to
  `microservices/tenancy/` per §D-2.
- Two denormalized index columns (`controls_tenants`,
  `controlled_by_tenants`) are added to the `tenants` table (per
  §D-2 — these are not the source of truth; the policy engine is).
- A new Cedar entity-type `ControllingEntity` and the six action
  namespaces enumerated in §D-1 land in `microservices/policy-
  engine/`.
- A new crate `oya-shared-conglomerate-grant-evaluator` lands in
  `crates/` per §E.1 and is consumed by ops-dashboard-control-center,
  finops-portal, audit-chain, and tenancy.
- Six new CI lanes (§D-8 / §E.4) enforce the invariants.
- The ADR-0244 §D-3 `primary_tenants` field is preserved unchanged
  but cross-referenced from this ADR (Stripe facilitator
  is now a special case of §D-3 scope tier `payment-facilitation`).
- The ADR-0244 §D-3 `can_facilitate_sub_merchants` BOOL is preserved
  unchanged but cross-referenced (it is the binary form of the
  `payment-facilitation` Cedar permit; this ADR generalizes to
  the broader controlling-entity grant).

**Stays:**

- ADR-0244 tenant model (§D-1 ID format, §D-2 sub-scope, §D-3
  schema, §D-4 Cedar entity types, §D-7 lifecycle, §D-11 audience
  type, §D-12 reserved namespaces) — unchanged. Every conglomerate
  tenant is itself an ADR-0244 tenant row.
- ADR-0009 cell architecture (per-tenant per-region) — unchanged.
  Parent and child can be in different cells.
- ADR-0010 regional pack architecture — unchanged. Parent and child
  can be in different jurisdictional packs.
- ADR-0145 inter-µservice communication reform (direct gRPC + 3
  invariants) — unchanged. The conglomerate Cedar evaluator is a
  thin shared crate, not a new µservice.
- ADR-0150 Cedar policy engine — unchanged in form; this ADR extends
  the entity-type registry with `ControllingEntity` and the action
  namespaces.
- ADR-0263 audit-chain emission contract — unchanged; this ADR adds
  six new emission classes (see §D-10).
- ADR-0311 dual-tenant boundary — unchanged; this ADR's permits
  cannot pierce the boundary.
- ADR-0312 court-warrant piercing — unchanged; this ADR's permits
  do not auto-cascade across the warrant scope.

## §B Decision

The conglomerate-tenant hierarchy doctrine establishes:

1. **Every tenant is first-class sovereign per ADR-0244.** No tenant
   is "owned" by another tenant. The conglomerate relationship is
   *purely* a Cedar permit overlay against the source-of-truth
   `conglomerate_grants` table. No data, no audit-chain entry, no
   cell binding, no identity row, no KMS envelope is *owned* by the
   parent. The child is sovereign.

2. **A conglomerate is a tenant that holds Cedar permits against
   other (sovereign) tenants.** The conglomerate relationship is
   represented by:
   - A row in `tenants` for the parent (an ordinary ADR-0244 tenant
     row).
   - A row in `tenants` for each child (each an ordinary ADR-0244
     tenant row with its own KYB, cell, audit-stream, residency).
   - One or more rows in `conglomerate_grants` declaring the
     parent's controlling-entity permits against each child, with
     scope (per §D-3), KYB attestation hash, regulatory citation,
     and audit dual-seal readiness.
   - Cedar fragments in the policy-engine that evaluate the permit
     at call time (per §D-1).

3. **The controlling-entity authority is bounded by per-jurisdiction
   corporate-governance law.** Every grant carries:
   - Parent's KYB attestation that the child is part of the parent's
     corporate group (`kyb_attestation_doc_hash`).
   - A regulatory citation naming the corporate-governance article
     that authorizes the relationship (`regulatory_citation` —
     e.g., `KR-Commercial-Act-Art-342`, `DE-Aktiengesetz-Art-15`,
     `US-DGCL-Title-8-§203`).
   - A corporate-officer principal-id who attested the grant
     (`granted_by_principal_id`).
   - Optional `sunset_at` (for joint ventures with end-dates) and
     `revoked_at` + `revoked_reason` (for spinoff, sale, court-
     ordered receivership, JV end).

4. **Restructuring is a 1-step Cedar revocation + grant.** Spinoff,
   divestiture, sale, IPO, bankruptcy, JV-formation, JV-dissolution,
   acquisition, holding-of-holding insertion, and reseller exit are
   each expressed as a Workflow Engine saga that touches
   `conglomerate_grants` + the Cedar fragment registry + emits
   dual-sealed audit events. No data migration. No identity re-
   issuance. No cell re-binding. No audit-chain re-sealing.

5. **The Cedar permit is bounded by six critical invariants** (per
   §D-4): no transitive auto-include; per-pack residency wins;
   personal-tenant boundary preserved; per-jurisdiction corporate-
   governance attestation required; cross-child information barrier
   enforceable; audit-chain dual-sealing mandatory.

6. **Six CI lanes (per §D-8 / §E.4) enforce the invariants at
   build-time and runtime.** Lanes are advisory until the substrate
   lands; BLOCKER thereafter.

## §C Consequences — across the six engineering-rigor dimensions

Per documentation-rigor.md §1.2, every ADR introducing a primitive
must address all six engineering-rigor dimensions. The conglomerate-
grant primitive is addressed below.

### §C.1 Maintainability dimension

The conglomerate doctrine reduces maintainability cost in three ways:

- **No new substrate µservice.** The doctrine extends three existing
  substrates (tenancy, policy-engine, audit-chain) plus one new
  shared crate (`oya-shared-conglomerate-grant-evaluator`). No new
  µservice means no new deployment topology, no new on-call rotation,
  no new SLO tier, no new dashboard family.
- **No new identity model.** Every child is an ADR-0244 tenant; the
  identity model is unchanged. Maintainers familiar with ADR-0244
  can reason about conglomerate cases without learning a new
  primitive.
- **Source-of-truth in one table.** `conglomerate_grants` is the
  source of truth; the `tenants.controls_tenants` /
  `tenants.controlled_by_tenants` columns are denormalized indexes
  populated by the policy-engine on every grant change. The denorm
  guarantee is documented in §D-2 and validated by the
  `oya-governance-conglomerate-grant-denorm-consistency` lane.

Reverse dependencies enumerated: ops-dashboard-control-center reads
controls_tenants for consolidated views; finops-portal reads
controls_tenants for consolidated billing rollups; audit-chain reads
controls_tenants for dual-seal target resolution; tenancy writes
controls_tenants on grant change.

Versioning policy: the `conglomerate_grants` schema follows
ADR-0258 (canonical schema-evolution policy); new scope tiers require
amendment ADR; new invariants require this ADR's amendment.
Deprecation cadence: scope-tier removal follows the 18-month sunset
window per ADR-0037 (public API stability).

What is hard-coded vs configurable: the six §D-3 scope tiers are
hard-coded in the policy-engine entity-type registry (per ADR-0246
substrate promotion). The seven §D-4 invariants are hard-coded in
the shared crate. Per-pack regulatory citations (§D-9) are
configurable via the pack overlay (per ADR-0010 + ADR-0299).

### §C.2 Observability dimension

Every conglomerate operation emits a structured audit event from the
§D-10 registry. The audit-chain emission contract (ADR-0263) is
extended with six new emission classes:

- `ConglomerateGrantCreated` — emitted on grant creation; carries
  parent-tenant-id, child-tenant-id, scope, KYB doc hash,
  regulatory citation, granted-by principal-id.
- `ConglomerateGrantRevoked` — emitted on revocation; carries the
  prior grant ID, revoke reason, revoked-by principal-id.
- `ConglomerateParentReadAction` — emitted on every parent action
  against child data; carries action surface, scope match, Cedar
  fragment IDs that approved.
- `ConglomerateCrossJurisdictionResidencyEnforced` — emitted when
  ADR-0304 invariant trips and a parent read is residency-preserved
  (data viewable in-cell but not exfiltrated).
- `ConglomerateInformationBarrierCrossingRefused` — emitted when a
  parent attempts to cross a regulator-required information barrier
  between two subsidiary children (FERC ring-fence, MiFID II ring-
  fence, banking ring-fence, accounting independence ring-fence).
- `ConglomeratePersonalTenantBoundaryRefused` — emitted when a
  parent attempts to read a child employee's personal-tenant data;
  the boundary holds per ADR-0311.

Cardinality budget: ConglomerateParentReadAction is the high-cardinality
event class (every dashboard load by an ops-dashboard-control-center
user emits one); rest are low-cardinality (revocation/refusal events
are rare). The per-cell cardinality budget for ConglomerateParentRead
Action is 10,000 events/second sustained, 100,000 events/second peak.
The audit-chain emission backbone is provisioned per ADR-0263 §C.3
for this load.

Metrics: `oyatie_conglomerate_grant_count` (gauge, dimensioned by
parent-tenant-id, child-tenant-id, scope); `oyatie_conglomerate_
parent_read_actions_total` (counter, dimensioned by parent, child,
scope, action); `oyatie_conglomerate_invariant_violations_total`
(counter, dimensioned by invariant number, parent, child);
`oyatie_conglomerate_grant_evaluation_p99_ms` (histogram). All
metrics carry tenant-id labels per ADR-0244 §D-3 invariant.

Traces: every conglomerate Cedar evaluation gets a span
`conglomerate_grant.evaluate` with attributes `parent.tenant_id`,
`child.tenant_id`, `scope`, `decision`, `applied_fragments`. Parent-
child relationship: nested under the originating gRPC span per
ADR-0042 trace tier discipline.

Logs: structured JSON at INFO for grant lifecycle (created, revoked,
sunset-reached); WARN for residency-preservation enforcement;
ERROR for invariant violations.

Dashboards: `dashboards/conglomerate-grants-overview.json` (Grafana)
shipped at substrate-land time, owned by tenancy µservice.

SLO floor: conglomerate-grant evaluation P99 ≤ 1 ms per ADR-0243
§D-1 Cedar evaluation SLO; dual-seal audit write P99 ≤ 5 ms (one
write to parent stream + one to child stream, in parallel).

### §C.3 Scalability dimension

The conglomerate doctrine scales horizontally without architectural
change. Capacity math:

- Each Cedar evaluation requires walking at most 2 entity types
  (parent ControllingEntity + child resource) and one
  `conglomerate_grants` row. The evaluation is O(1) per request
  given the per-cell hot cache (Valkey, 1s TTL — per ADR-0243 §D-7).
- The `conglomerate_grants` table grows linearly with the number of
  grants; at 1 million tenants with average 5 grants per parent
  (across hierarchical levels), the table is 5M rows. Postgres
  index size for `idx_conglomerate_grants_parent` is ~250 MB per
  Little's-Law sizing (5M rows × 50 bytes per index entry). Read
  IOPS per partition tier is well within the per-cell DB tier per
  ADR-0045 database tier strategy.
- Per-cell hot cache pressure: at 10K parent-reads/sec/cell, the
  Valkey hot cache evicts at 10K/sec × 0.5 KB per entry = 5 MB/sec
  churn — within the per-cell Valkey provisioned tier per ADR-0046.

Bottleneck identification + shard strategy:

- **Bottleneck A — denorm consistency lag.** The denormalized
  `tenants.controls_tenants` array can lag the source-of-truth
  `conglomerate_grants` table during high-frequency grant changes
  (e.g., a JV-formation saga with 50 simultaneous grants). Mitigation:
  the denorm is written transactionally with the grant insert (one
  Postgres transaction touches both tables). Lag bound: 0 ms by
  construction.
- **Bottleneck B — Cedar fragment registry contention on parent-
  tenant tags.** A parent with 50 children has 50 active Cedar
  fragments; fragment loading from the registry is per-tenant
  cached (per ADR-0243 §D-4). Mitigation: registry preload at cell
  start; reload via signed change-notification per ADR-0243 §D-3.
- **Bottleneck C — audit-chain dual-seal write parallelism.** Each
  ConglomerateParentReadAction requires writes to two streams.
  Mitigation: parallel async writes; failure to seal in either
  stream fails the operation (no half-sealed states). Per ADR-0263
  §D-2 audit-chain backbone, the per-cell write tier handles 100K
  writes/sec sustained.

Horizontal scale-out path: per-cell sharding (the standard ADR-0009
cell shard). Cross-cell grants follow the ADR-0049 cross-region
replication path; the `conglomerate_grants` table is replicated
to all cells that host either the parent or any child (per ADR-0049
§D-3 selective-replication policy).

Threshold: the system goes red when ConglomerateParentReadAction
cardinality exceeds 100K/sec/cell sustained for >5 minutes (per the
brownout signal per ADR-0176); the brownout response is to shed
ConsolidatedDashboard refresh frequency from 10s to 60s.

### §C.4 Performance dimension

P50/P95/P99 targets for conglomerate-grant evaluation:

- P50: 0.2 ms (cache hit, no Cedar evaluation needed)
- P95: 0.5 ms (cache miss, Cedar evaluation with permit fragments
  pre-loaded)
- P99: 1.0 ms (cache miss + fragment registry round-trip)
- P99.9: 2.0 ms (cold cache + registry round-trip + Postgres lookup
  of the conglomerate_grants row)

Per-region budget split: the conglomerate evaluation happens in the
caller's cell (the parent's cell, since the parent initiated the
read); the only cross-cell hop is the audit-chain dual-seal write to
the child's cell, which is fire-and-forget asynchronous (the response
to the caller is not blocked on the child-cell write).

Tail-latency mitigation: hedged Cedar evaluation (per ADR-0150 §D-5)
covers the P99.9 tail. The hedge fires at 1.0 ms; if the primary
returns first, the hedge is canceled.

Cold-start budget: 50 ms (Cedar fragment registry preload + Valkey
cache warm). Acceptable per ADR-0044 cold-start SLO tier.

No bare percentile claims: all targets are derived from ADR-0243
§D-1 Cedar evaluation SLO (1 ms P99 per gate) plus one additional
hop (the conglomerate gate is one gate per parent-action).

### §C.5 Optimization dimension

Cost-performance trade-offs explicitly named:

- **Lazy vs eager grant materialization.** Lazy: evaluate the Cedar
  permit at every call. Eager: materialize the permit graph at grant-
  creation time and serve from cache. Decision: **lazy**, because the
  permit shape is small (one Cedar evaluation per gate) and the
  cache hit rate is high (>95% in steady state); eager materialization
  would invalidate broadly on every grant change.
- **Cache-invalidation policy.** Per-tenant cache keyed on
  `(parent_tenant_id, child_tenant_id, scope)`. Invalidation on
  grant change is push-based (the policy-engine publishes a
  notification to the per-cell Valkey instance over the bus per
  ADR-0246 §D-5).
- **Cold-vs-warm path latency separation.** Warm path: P99 ≤ 1 ms
  (cache hit). Cold path: P99 ≤ 50 ms (full registry round-trip).
  The cold path is rare (<0.1% of evaluations per the cache hit
  rate model).
- **Per-call cost model.** Each conglomerate evaluation costs
  ~0.5 CPU-µs in cache-hit mode (one cache lookup + one Cedar
  evaluation against pre-compiled fragments) and ~50 CPU-µs in
  cache-miss mode. At 10K parent-reads/sec/cell, the steady-state
  cell CPU cost is 5 ms/sec = 0.5% of one core. Negligible.

Profiling evidence link: per the substrate-promotion plan, profiling
results will land at `evidence/perf/conglomerate-grant-evaluator-
profiling-2026-Q3.json` before BLOCKER promotion.

### §C.6 Code quality dimension

Required test classes for `oya-shared-conglomerate-grant-evaluator`:

- **Unit tests:** ≥85% line coverage, ≥75% branch coverage per
  ADR-0212 buildability doctrine. Every public function carries
  doctests with `Permit` and `Forbid` cases.
- **Property tests (proptest):** invariant-preservation properties
  for the six §D-4 invariants (no transitive auto-include, residency
  preserved, personal-tenant boundary held, attestation required,
  information barrier honored, dual-seal mandatory). Each invariant
  has ≥1 property check.
- **Fuzz tests (cargo-fuzz):** Cedar fragment input fuzzing; Cedar
  decision must be deterministic across all permitted scope
  combinations.
- **Load tests:** 100K evaluations/sec sustained per cell on the
  reference hardware (per ADR-0128 hyperscaler architecture
  invariants).
- **End-to-end tests:** Workflow Engine sagas exercise spinoff,
  acquisition, JV-formation, JV-dissolution, IPO, bankruptcy — each
  end-to-end test verifies the dual-sealed audit trail.

Lint passes named: `oya-check-rust-deny-warnings`,
`oya-check-rust-clippy-bar-A`, `oya-check-cedar-fragment-syntax`,
`oya-check-cedar-fragment-completeness`,
`oya-check-conglomerate-grant-schema-coherence`.

Type strictness: `#![deny(warnings)]`, `#![deny(missing_docs)]`,
`#![deny(unsafe_code)]` at the crate root.

SemVer + ABI policy: per ADR-0037 public API stability; the crate's
public surface (`evaluate`, `grant_create`, `grant_revoke`,
`grant_query`, `denorm_refresh`) is SemVer-stable from 1.0; any
breaking change requires amendment ADR + 18-month sunset.

## §D Detailed mechanics

### §D-1 The controlling-entity Cedar grammar

Per ADR-0150 Cedar v4.2 LTS + ADR-0243 §D-1 evaluation contract. The
conglomerate-grant grammar introduces one new entity type and six
new action namespaces.

#### §D-1.1 Cedar entity types

The new Cedar entity type:

```cedar
entity ControllingEntity in [Tenant] {
    parent_tenant_id:           String,
    kyb_attestation_doc_hash:   String,
    corporate_governance_proof: String,  // regulatory citation (per §D-9)
    grant_scope_tier:           Set<String>,  // subset of §D-3 scope-tier enum
    granted_at:                 Long,    // unix timestamp seconds
    sunset_at:                  Long,    // 0 = no sunset
    information_barrier_set:    Set<String>,  // child-tenant-ids the parent
                                              // cannot cross-disclose between
};
```

`ControllingEntity` is *not* a separate principal class — it is a
Cedar role on a `Tenant` entity. The principal at the call site is
the parent tenant's principal acting under the `controlling-entity`
role.

#### §D-1.2 Action namespaces

Six action namespaces are introduced:

```cedar
namespace ParentScope {
    action ReadActions = [
        ReadFinancialConsolidatedReport,
        ReadOperationalMetric,
        ReadComplianceEvidence,
        ReadAuditEvidence,
        ReadMarketplaceListings,
        ReadWorkflowEngineMetadata,
        ReadCostBudgetAttribution,
        ReadCellTopology,
        ReadDashboardSnapshot
    ];
    action WriteActions = [
        WriteBoardResolutionRecord,
        WriteCapitalStructureChange,
        WriteAppointDirectorAction,
        WriteApproveMergerAction,
        WriteApproveDivestitureAction,
        WriteApproveIPOFiling,
        WriteApproveBudget
    ];
    action AuditActions = [
        ReadComplianceEvidenceOnly,
        ReadAuditChainEvidence,
        ReadRegulatorReportEvidence,
        AttestRegulatorCertification
    ];
    action JointVentureActions = [
        ReadJVPartialFinancialFacet,
        ReadJVPartialOperationalFacet,
        WriteJVCapitalCallApproval,
        WriteJVDissolutionInitiation
    ];
    action PaymentFacilitationActions = [
        FacilitatePayment,
        RouteSettlementToConnectedAccount,
        ApplyApplicationFee,
        RefundFromPlatformBalance,
        AssumeChargebackResponsibility,
        FileFormSubmerchant1099K
    ];
    action CrossJurisdictionReadActions = [
        ReadResidencyPreservedFinancialView,
        ReadResidencyPreservedOperationalView,
        ReadResidencyPreservedAuditView
    ];
}
```

#### §D-1.3 Canonical controlling-entity permit fragment

The canonical permit fragment shape (per ADR-0243 fragment authoring
discipline + cedar-policy-discipline.md):

```cedar
@id("conglomerate-grant-{parent_tenant_id}-controls-{child_tenant_id}")
@version("1.0.0")
@signed_by("policy-engine-bootstrap-tier0-hsm")
@binding_adr("ADR-0313")
permit (
    principal in oyatie::Tenant::"<parent_tenant_id>"::Role::"controlling-entity",
    action in [
        ParentScope::ReadActions,
        ParentScope::AuditActions
    ],
    resource in oyatie::Tenant::"<child_tenant_id>"::*
)
when {
    // Invariant 4: per-jurisdiction corporate-governance attestation
    principal.kyb_attestation_includes_child("<child_tenant_id>")
    && principal.corporate_governance_proof_active("<regulatory-citation>")

    // Invariant 1: no transitive auto-include
    && context.is_direct_permit == true

    // Invariant 2: per-pack residency preserved
    && context.cross_jurisdiction_check_passes(principal.tenant, resource.tenant)

    // Invariant 3: personal-tenant boundary preserved
    && context.personal_tenant_boundary_preserved(resource.tenant)

    // Invariant 5: cross-child information barrier honored
    && context.information_barrier_check_passes(
        principal.tenant,
        resource.tenant,
        principal.information_barrier_set
    )

    // Invariant 6: audit-chain dual-seal ready
    && context.audit_chain_dual_seal_ready(principal.tenant, resource.tenant)

    // Sunset bound (if set)
    && (principal.sunset_at == 0 || context.now < principal.sunset_at)
};

@id("conglomerate-grant-default-deny-{parent_tenant_id}-{child_tenant_id}")
@version("1.0.0")
forbid (
    principal in oyatie::Tenant::"<parent_tenant_id>"::Role::"controlling-entity",
    action,
    resource in oyatie::Tenant::"<child_tenant_id>"::*
)
unless { context.permit_evaluated == true };
```

The default-deny fragment is mandatory per ADR-0243 §D-2 default-
deny coverage CI lane.

#### §D-1.4 Evaluation contract

Per ADR-0243 §D-1 Cedar evaluation contract, the request shape:

```rust
// crates/oya-shared-conglomerate-grant-evaluator/src/api.rs

pub struct ConglomerateEvaluationRequest {
    pub parent_tenant_id: TenantId,
    pub child_tenant_id: TenantId,
    pub action: ParentScopeAction,
    pub resource: ResourceRef,
    pub context: ConglomerateContext,
    pub evaluation_id: Uuid,
}

pub struct ConglomerateContext {
    pub now: UnixTimestamp,
    pub permit_evaluated: bool,
    pub cross_jurisdiction_check_passes: bool,
    pub personal_tenant_boundary_preserved: bool,
    pub information_barrier_check_passes: bool,
    pub audit_chain_dual_seal_ready: bool,
    pub is_direct_permit: bool,
}

pub struct ConglomerateEvaluationResponse {
    pub decision: CedarDecision,        // Permit | Forbid | NotApplicable
    pub applied_fragments: Vec<FragmentId>,
    pub residency_preservation_required: bool,
    pub information_barrier_refused_target: Option<TenantId>,
    pub personal_tenant_refusal_reason: Option<String>,
    pub dual_seal_targets: (TenantId, TenantId),
    pub evaluation_ms: f64,
}
```

### §D-2 Postgres DDL extension

#### §D-2.1 Source-of-truth table

A new table is added to `microservices/tenancy/schemas/conglomerate_
grants.sql`:

```sql
-- Migration: 0014_conglomerate_grants.sql
-- Binding ADR: ADR-0313
-- Author: council-architecture
-- Date: 2026-05-20

CREATE TABLE conglomerate_grants (
    grant_id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_tenant_id                TEXT NOT NULL REFERENCES tenants(tenant_id),
    child_tenant_id                 TEXT NOT NULL REFERENCES tenants(tenant_id),
    scope                           jsonb NOT NULL,
        -- Shape per §D-3:
        -- {
        --   "tiers": ["read-only-financial", "read-only-operational"],
        --   "actions_subset": [...],         // optional Cedar action subset
        --   "information_barrier_set": [...] // optional cross-child barrier
        -- }
    kyb_attestation_doc_hash        TEXT NOT NULL,
        -- SHA-256 of parent's board-resolution + corporate-officer-signature
        -- document chain attesting the controlling-entity relationship
    regulatory_citation             TEXT NOT NULL,
        -- per-jurisdiction corporate-governance law per §D-9
        -- e.g., "KR-Commercial-Act-Art-342"
        --       "DE-Aktiengesetz-Art-15"
        --       "US-DGCL-Title-8-§203"
        --       "JP-Companies-Act-Art-2-§3"
        --       "UK-Companies-Act-2006-§1159"
        --       "CN-PRC-Company-Law-Art-216"
        --       "FR-Code-de-Commerce-Art-L233-3"
        --       "IN-Companies-Act-2013-§2(87)"
    granted_at                      TIMESTAMPTZ NOT NULL DEFAULT now(),
    granted_by_principal_id         TEXT NOT NULL,
        -- the corporate officer at parent who attested the grant
    sunset_at                       TIMESTAMPTZ,
        -- optional sunset (e.g., joint venture with end-date)
    revoked_at                      TIMESTAMPTZ,
    revoked_by_principal_id         TEXT,
    revoked_reason                  TEXT,
        -- e.g., "sold-to-new-parent" | "IPO-spinoff" |
        --       "court-ordered-receivership" | "joint-venture-end" |
        --       "voluntary-divestiture" | "bankruptcy-receiver-takeover" |
        --       "regulator-forced-divestiture" | "share-buyback-deconsolidation"
    audit_chain_seal_parent_id      TEXT NOT NULL,
        -- the audit-chain seal ID for the grant-creation event in
        -- parent's stream (per ADR-0263 dual-seal)
    audit_chain_seal_child_id       TEXT NOT NULL,
        -- the audit-chain seal ID for the grant-creation event in
        -- child's stream

    CONSTRAINT parent_not_self
        CHECK (parent_tenant_id != child_tenant_id),
    CONSTRAINT scope_not_empty
        CHECK (jsonb_array_length(scope->'tiers') > 0),
    CONSTRAINT regulatory_citation_not_empty
        CHECK (length(regulatory_citation) > 0),
    CONSTRAINT kyb_attestation_doc_hash_format
        CHECK (kyb_attestation_doc_hash ~ '^sha256:[a-f0-9]{64}$'),
    CONSTRAINT sunset_after_grant
        CHECK (sunset_at IS NULL OR sunset_at > granted_at),
    CONSTRAINT revoke_consistency
        CHECK ((revoked_at IS NULL AND revoked_by_principal_id IS NULL
                AND revoked_reason IS NULL)
            OR (revoked_at IS NOT NULL AND revoked_by_principal_id IS NOT NULL
                AND revoked_reason IS NOT NULL)),

    UNIQUE (parent_tenant_id, child_tenant_id, granted_at)
        -- multiple grants over time are permitted (revoke + re-grant);
        -- the active grant is the one with revoked_at IS NULL
);

CREATE INDEX idx_conglomerate_grants_parent_active
    ON conglomerate_grants (parent_tenant_id, child_tenant_id)
    WHERE revoked_at IS NULL;

CREATE INDEX idx_conglomerate_grants_child_active
    ON conglomerate_grants (child_tenant_id, parent_tenant_id)
    WHERE revoked_at IS NULL;

CREATE INDEX idx_conglomerate_grants_regulatory_citation
    ON conglomerate_grants (regulatory_citation)
    WHERE revoked_at IS NULL;

CREATE INDEX idx_conglomerate_grants_sunset
    ON conglomerate_grants (sunset_at)
    WHERE sunset_at IS NOT NULL AND revoked_at IS NULL;

-- Row-level security: only the policy-engine substrate may write;
-- only tenants party to the grant (parent OR child) may read.
ALTER TABLE conglomerate_grants ENABLE ROW LEVEL SECURITY;

CREATE POLICY conglomerate_grants_read_policy
    ON conglomerate_grants
    FOR SELECT
    USING (
        current_setting('oyatie.acting_tenant_id', true) = parent_tenant_id
        OR current_setting('oyatie.acting_tenant_id', true) = child_tenant_id
        OR current_setting('oyatie.is_substrate_service', true) = 'true'
    );

CREATE POLICY conglomerate_grants_write_policy
    ON conglomerate_grants
    FOR INSERT
    WITH CHECK (
        current_setting('oyatie.is_substrate_service', true) = 'true'
    );
```

#### §D-2.2 Denormalized index columns on `tenants`

Two columns are added to the `tenants` table (per ADR-0244 §D-3):

```sql
-- Migration: 0015_tenant_conglomerate_index_columns.sql
-- Binding ADR: ADR-0313
-- Author: council-architecture
-- Date: 2026-05-20

ALTER TABLE tenants
    ADD COLUMN controls_tenants TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];
    -- list of child_tenant_ids this tenant has controlling-entity
    -- Cedar permits against;
    -- NOT the source of truth (conglomerate_grants is) — this is a
    -- denormalized index for query performance;
    -- populated transactionally by the policy-engine on every
    -- controlling-entity grant creation / revocation;

ALTER TABLE tenants
    ADD COLUMN controlled_by_tenants TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[];
    -- reverse direction; child tenant rows know who has controlling
    -- permits over them;
    -- enables child-side visibility into parent activity per the
    -- §D-4 invariant 6 audit-chain dual-sealing guarantee;

CREATE INDEX idx_tenants_controls_tenants_gin
    ON tenants USING GIN (controls_tenants);

CREATE INDEX idx_tenants_controlled_by_tenants_gin
    ON tenants USING GIN (controlled_by_tenants);

-- Consistency check function — invoked by the policy-engine on every
-- grant lifecycle event; the CI lane
-- oya-governance-conglomerate-grant-denorm-consistency verifies the
-- denorm matches the source-of-truth.
CREATE OR REPLACE FUNCTION oyatie_assert_conglomerate_denorm_consistent()
    RETURNS TABLE (parent_tenant_id TEXT, child_tenant_id TEXT, drift_kind TEXT)
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Find parent rows whose controls_tenants array is missing an
    -- active grant
    RETURN QUERY
    SELECT cg.parent_tenant_id,
           cg.child_tenant_id,
           'missing-in-controls-tenants-denorm' AS drift_kind
    FROM conglomerate_grants cg
    JOIN tenants t ON t.tenant_id = cg.parent_tenant_id
    WHERE cg.revoked_at IS NULL
      AND NOT (cg.child_tenant_id = ANY(t.controls_tenants));

    -- Find parent rows whose controls_tenants array contains a child
    -- with no active grant
    RETURN QUERY
    SELECT t.tenant_id AS parent_tenant_id,
           c AS child_tenant_id,
           'orphan-in-controls-tenants-denorm' AS drift_kind
    FROM tenants t,
         unnest(t.controls_tenants) AS c
    WHERE NOT EXISTS (
        SELECT 1 FROM conglomerate_grants cg
        WHERE cg.parent_tenant_id = t.tenant_id
          AND cg.child_tenant_id = c
          AND cg.revoked_at IS NULL
    );

    -- Mirror checks for controlled_by_tenants
    RETURN QUERY
    SELECT cg.parent_tenant_id,
           cg.child_tenant_id,
           'missing-in-controlled-by-tenants-denorm' AS drift_kind
    FROM conglomerate_grants cg
    JOIN tenants t ON t.tenant_id = cg.child_tenant_id
    WHERE cg.revoked_at IS NULL
      AND NOT (cg.parent_tenant_id = ANY(t.controlled_by_tenants));

    RETURN QUERY
    SELECT p AS parent_tenant_id,
           t.tenant_id AS child_tenant_id,
           'orphan-in-controlled-by-tenants-denorm' AS drift_kind
    FROM tenants t,
         unnest(t.controlled_by_tenants) AS p
    WHERE NOT EXISTS (
        SELECT 1 FROM conglomerate_grants cg
        WHERE cg.parent_tenant_id = p
          AND cg.child_tenant_id = t.tenant_id
          AND cg.revoked_at IS NULL
    );
END;
$$;
```

#### §D-2.3 Spec — `/specs/conglomerate-grant-model.json`

A new spec is added per the per-primitive spec-binding invariant in
documentation-rigor.md §1.1 completeness invariant #3:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "/specs/conglomerate-grant-model.json",
  "title": "Conglomerate-grant model",
  "version": "1.0.0",
  "_meta": {
    "purpose": "Source-of-truth model for parent/child conglomerate Cedar grants",
    "industry_citations": [
      "AWS Organizations Service Control Policies",
      "Microsoft 365 Multi-Tenant Organization model",
      "Google Workspace reseller console permit model",
      "Stripe platform-facilitator pattern"
    ],
    "related_adrs": ["ADR-0313", "ADR-0244", "ADR-0243", "ADR-0242"],
    "binding_adr": "ADR-0313",
    "status": "advisory-until-conglomerate-substrate-lands",
    "enforcement_status": "advisory-until-2026-07-15-then-blocker"
  },
  "type": "object",
  "required": [
    "grant_id",
    "parent_tenant_id",
    "child_tenant_id",
    "scope",
    "kyb_attestation_doc_hash",
    "regulatory_citation",
    "granted_at",
    "granted_by_principal_id",
    "audit_chain_seal_parent_id",
    "audit_chain_seal_child_id"
  ],
  "properties": {
    "grant_id": {
      "type": "string",
      "format": "uuid",
      "description": "UUID v4 grant identifier; unique per (parent, child, granted_at) triple",
      "examples": ["a7c3e4f9-1b2d-4e5f-8a9b-1c2d3e4f5a6b"]
    },
    "parent_tenant_id": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9-]{2,62}$",
      "description": "Parent (controlling-entity) tenant ID per ADR-0244 §D-1 tenant ID format",
      "examples": ["snu-hospital-network", "samsung-group-holdings"]
    },
    "child_tenant_id": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9-]{2,62}$",
      "description": "Child (controlled) tenant ID per ADR-0244 §D-1 tenant ID format",
      "examples": ["snu-hospital-gangnam", "samsung-electronics"]
    },
    "scope": {
      "type": "object",
      "required": ["tiers"],
      "properties": {
        "tiers": {
          "type": "array",
          "items": {
            "enum": [
              "read-only-financial",
              "read-only-operational",
              "read-write-board-decisions",
              "audit-only",
              "cross-jurisdiction-read-only",
              "joint-venture-partial",
              "payment-facilitation"
            ]
          },
          "minItems": 1,
          "description": "One or more scope tiers per §D-3"
        },
        "actions_subset": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Optional Cedar action subset within the scope tiers"
        },
        "information_barrier_set": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Child-tenant-ids the parent cannot cross-disclose between"
        }
      }
    },
    "kyb_attestation_doc_hash": {
      "type": "string",
      "pattern": "^sha256:[a-f0-9]{64}$",
      "description": "SHA-256 of parent's board-resolution + corporate-officer-signature document chain",
      "examples": ["sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"]
    },
    "regulatory_citation": {
      "type": "string",
      "minLength": 1,
      "description": "Per-jurisdiction corporate-governance law citation per §D-9",
      "examples": [
        "KR-Commercial-Act-Art-342",
        "DE-Aktiengesetz-Art-15",
        "US-DGCL-Title-8-§203",
        "UK-Companies-Act-2006-§1159"
      ]
    },
    "granted_at": {
      "type": "string",
      "format": "date-time",
      "description": "ISO 8601 timestamp of grant creation",
      "examples": ["2026-05-20T10:30:00Z"]
    },
    "granted_by_principal_id": {
      "type": "string",
      "description": "Corporate officer principal-id at parent who attested the grant",
      "examples": ["snu-hospital-network::officer::ceo"]
    },
    "sunset_at": {
      "type": ["string", "null"],
      "format": "date-time",
      "description": "Optional sunset (e.g., joint venture end-date)"
    },
    "revoked_at": {
      "type": ["string", "null"],
      "format": "date-time",
      "description": "Revocation timestamp; null while active"
    },
    "revoked_by_principal_id": {
      "type": ["string", "null"],
      "description": "Principal who revoked the grant"
    },
    "revoked_reason": {
      "type": ["string", "null"],
      "enum": [
        null,
        "sold-to-new-parent",
        "IPO-spinoff",
        "court-ordered-receivership",
        "joint-venture-end",
        "voluntary-divestiture",
        "bankruptcy-receiver-takeover",
        "regulator-forced-divestiture",
        "share-buyback-deconsolidation"
      ]
    },
    "audit_chain_seal_parent_id": {
      "type": "string",
      "description": "Audit-chain seal ID for grant-creation in parent's stream"
    },
    "audit_chain_seal_child_id": {
      "type": "string",
      "description": "Audit-chain seal ID for grant-creation in child's stream"
    }
  }
}
```

### §D-3 Scope tiers

Seven scope tiers are defined. Each tier is a Cedar action namespace
slice; a grant may activate one or more tiers. Tier composition is
union-with-deny-precedence per ADR-0243 §D-1 Cedar evaluation
semantics.

#### §D-3.1 `read-only-financial`

The most common scope tier. Activates `ParentScope::ReadActions`
subset: `ReadFinancialConsolidatedReport`,
`ReadCostBudgetAttribution`. Used by quarterly financial consolidated
reporting (10-K, IFRS-consolidated-statements, K-IFRS-consolidated-
statements). Read-only; cannot modify child financial records. Cross-
jurisdiction residency-preserved per §D-4 invariant 2.

#### §D-3.2 `read-only-operational`

Bloomberg-style operational dashboard scope. Activates
`ParentScope::ReadActions` subset: `ReadOperationalMetric`,
`ReadDashboardSnapshot`, `ReadCellTopology`,
`ReadWorkflowEngineMetadata`. Read-only; aggregated metrics;
per-child sub-totals visible; per-child raw PII NOT visible (the
ADR-0099 data-class registry hides class > 3 from this scope).

#### §D-3.3 `read-write-board-decisions`

For decisions that the parent legitimately makes on behalf of the
group: board appointments, capital structure, M&A approval, budget
approval, dividend declaration. Activates `ParentScope::WriteActions`.
Requires elevated KYB attestation (`kyb_attestation_doc_hash` must
include the per-jurisdiction board-resolution evidence).

#### §D-3.4 `audit-only`

For ring-fenced subsidiaries: regulated banks, healthcare providers,
insurance carriers, accounting/audit firms. Parent sees compliance
evidence (ADR-0263 audit-chain emissions) but NOT operational PII.
Activates `ParentScope::AuditActions`. Used by:

- Bank holding companies whose subsidiary banks are subject to
  Federal Reserve oversight (US Bank Holding Company Act of 1956
  §225.4).
- KR-FSC-regulated *금융지주회사* (financial holding companies)
  whose subsidiaries are subject to FSC oversight (KR-Financial-
  Holding-Companies-Act-Art-15).
- EU-ECB-regulated bank holding under SSM Single Supervisory
  Mechanism.
- Healthcare holding companies whose subsidiaries are HIPAA Covered
  Entities (parent is a Business Associate per HIPAA, NOT a Covered
  Entity; the data-access scope is BA-only per HIPAA §164.504(e)).

#### §D-3.5 `cross-jurisdiction-read-only`

For multi-jurisdictional holding companies (e.g., a US-incorporated
parent with EU-incorporated subsidiaries). The scope is read-only
WITH residency-preservation: the data is viewable in the child's
cell via a read-only proxy but is NEVER exfiltrated across the
jurisdiction boundary. This is the strongest enforcement of the
§D-4 invariant 2 (per-pack residency wins).

Mechanically: the parent's request lands at the child's cell via a
cross-cell gRPC call (per ADR-0049); the cell's egress-filter (per
ADR-0299 cross-pack residency arbitration) rejects any response
payload containing class-3+ data (PII, financial PII, health PII).
Aggregated views (count, sum, avg) pass; row-level data does NOT.

#### §D-3.6 `joint-venture-partial`

For joint ventures with multiple parents. Each parent gets a
partial-scope facet:

- Parent A: `read-only-financial` only.
- Parent B: `read-only-operational` only.
- (Both): `joint-venture-partial` mode with explicit `JointVentureActions`
  enumeration.

Per-parent scope segregation is enforced by the Cedar evaluation
context: when the JV-child's data is requested, the per-parent
fragment evaluates against ONLY that parent's facet.

#### §D-3.7 `payment-facilitation`

The Stripe-platform-facilitator pattern. Cross-references
ADR-0244 §D-3 `can_facilitate_sub_merchants` (which is the binary
form of this scope tier). When the parent is a marketplace platform
(per ADR-0249 multi-category marketplace) and the children are
seller tenants, the platform holds `payment-facilitation` scope
over each seller. Activates `ParentScope::PaymentFacilitationActions`.
The seller retains everything else (KYC identity, customer data,
product catalog, audit-chain); the platform handles the payment
substrate.

The pattern is used by:

- Shopify (each merchant is a sovereign Stripe Connected Account).
- DoorDash, Uber Eats, Lyft (each driver/merchant connected account).
- Squarespace, Wix Stores, Etsy.
- Booking.com (each property partner connected account).
- AWS Marketplace (each ISV seller sovereign with platform-facilitated
  payments).

### §D-4 The six critical invariants

The conglomerate grant primitive is bounded by six invariants. Each
invariant is enforced by Cedar fragments + a dedicated CI lane.

#### §D-4.1 Invariant 1 — no transitive auto-include

A grant from parent A to child B does NOT auto-grant from A to any
grandchild C of B. Each level of the conglomerate hierarchy requires
its own explicit grant.

Rationale: transitivity makes Cedar evaluation unbounded (the policy-
engine would walk the hierarchy on every call); it also makes
restructuring fragile (severing A→B requires re-issuing A→C if A→C
was implicit). Explicit grants make restructuring atomic.

Enforcement: Cedar fragment `context.is_direct_permit == true`;
CI lane `oya-governance-conglomerate-grant-transitivity-deny` walks
the grant graph and refuses any code path that evaluates a transitive
permit.

Counter-example (rejected): "parent A reads child B's child C's data
because A controls B controls C." The right path: A must hold a
direct grant against C (which requires A's KYB to attest the C
relationship — same evidentiary bar as the B grant).

#### §D-4.2 Invariant 2 — per-pack residency wins

When the parent and child reside in different jurisdictional packs
(per ADR-0010), the child's pack residency rules win. A US parent
reading EU-resident child data does not exfiltrate the data — the
ADR-0299 cross-pack residency arbitration determines whether the
read is permitted as a residency-preserved-view (aggregated /
proxied / class-3-redacted) or refused outright.

Enforcement: Cedar fragment
`context.cross_jurisdiction_check_passes(principal.tenant, resource.tenant)`;
CI lane `oya-governance-cross-jurisdiction-residency-preserved`
verifies the cross-jurisdiction permit code path never returns
class-3+ data across the boundary.

#### §D-4.3 Invariant 3 — personal-tenant boundary preserved

A parent's controlling-entity permit cannot pierce the personal
tenant of a child's employee. Per ADR-0311 dual-tenant identity
personal-vs-work boundary: every human principal has a personal
tenant + zero-or-more work tenants. The work tenant is the
employer's tenant (which may itself be a child of a conglomerate).
The personal tenant is fully sovereign, never reachable from any
parent of the work tenant.

Enforcement: Cedar fragment
`context.personal_tenant_boundary_preserved(resource.tenant)`;
CI lane `oya-governance-conglomerate-grant-personal-tenant-deny`
verifies that every conglomerate-grant code path refuses to evaluate
when the target resource carries a `personal-tenant-bound` flag
(set by ADR-0311's principal-resolver on personal-tenant resources).

Counter-example (rejected): "parent reads child employee's calendar
because the calendar app shows both personal and work events." The
right path: only the work-tenant subset is readable; the personal-
tenant subset is opaque even to the child tenant's admins, much
less the parent.

#### §D-4.4 Invariant 4 — per-jurisdiction corporate-governance attestation required

Every grant carries a regulatory citation naming the corporate-
governance article authorizing the relationship. The grant is only
valid while:

- The KYB attestation document is current (re-attested annually per
  the per-jurisdiction filing requirement).
- The regulatory citation is still active (the cited law is not
  repealed; the regulator has not withdrawn its consent if consent
  was required at grant time).

Enforcement: Cedar fragment
`principal.corporate_governance_proof_active("<citation>")`; CI lane
`oya-governance-conglomerate-grant-attestation-current` runs daily
and flags any grant whose attestation is >365 days old, whose cited
regulation is in the retired-citations registry, or whose granted-by
principal has been off-boarded.

#### §D-4.5 Invariant 5 — cross-child information barrier

When the parent holds grants against ≥2 children AND the regulator
requires an information barrier between them (Glass-Steagall ring-
fence for bank holding companies; FERC ring-fence for energy utility
holding companies; SEC analyst-research/investment-banking ring-
fence; MiFID II Article 16(8) ring-fence; HIPAA-§164.514(e) limited-
dataset ring-fence; accounting independence ring-fence for audit
firms), the parent reads each child's data but the policy-engine
refuses to cross-disclose the read across children.

Mechanically: the grant's `information_barrier_set` enumerates the
sibling-child-tenant-ids that this grant cannot cross-disclose to.
The Cedar fragment evaluates: when parent P reads child A's data,
the response carries a *taint* indicating its origin (`origin_tenant_id`);
when parent P then attempts to write that data into child B's
context, the Cedar fragment refuses if A ∈ `information_barrier_set`
of the (P, B) grant.

Enforcement: Cedar fragment
`context.information_barrier_check_passes(principal.tenant,
resource.tenant, principal.information_barrier_set)`; CI lane
`oya-governance-conglomerate-information-barrier-coverage` verifies
every (parent, child-pair) combination where the regulator-list
requires a barrier has a barrier configured.

#### §D-4.6 Invariant 6 — audit-chain dual-sealing

Every parent action against a child seals in BOTH the parent's
audit-chain AND the child's audit-chain. The dual-seal is mandatory;
single-sealed actions are rejected at the audit-emission boundary
per ADR-0263.

Rationale: the parent must have its own evidence trail (for
regulator reporting, for board accountability, for shareholder
disclosure). The child must have its own evidence trail (for
subsidiary-level audit, for regulator-required subsidiary
reporting, for child's own compliance-pack obligations). Single-
sealing favours one party; dual-sealing makes neither party able
to deny the action.

Mechanically: the audit-chain emission backbone (per ADR-0263 §D-2)
accepts a dual-seal request with `(parent_tenant_id, child_tenant_id,
event)` and writes ONE event to each stream with cross-referencing
seal IDs. Failure to write either side fails the action (the parent's
read is not served; the response error is `dual-seal-failed`).

Enforcement: Cedar fragment
`context.audit_chain_dual_seal_ready(principal.tenant, resource.tenant)`
gates the action; CI lane `oya-governance-conglomerate-grant-dual-
sealed` verifies the audit-chain backbone always writes both seals
or neither.

### §D-5 Restructuring scenarios — worked examples

The following nine restructuring scenarios are worked end-to-end to
demonstrate the 1-step Cedar revocation + grant pattern.

#### §D-5.1 Spinoff

**Scenario.** Parent's board votes to spin off child X. The spinoff
is the simplest restructuring: parent's permit is revoked; child
operates standalone immediately.

**Workflow Engine saga** (`microservices/workflow-studio/sagas/
conglomerate-spinoff.yaml`):

1. **Pre-checks** — verify spinoff vote (board resolution document
   hash, corporate officer attestation), verify no court order
   blocking, verify no outstanding regulatory consent required.
2. **Grant revocation** — `UPDATE conglomerate_grants SET revoked_at
   = now(), revoked_by_principal_id = '<corporate-officer>',
   revoked_reason = 'IPO-spinoff' WHERE parent_tenant_id = '<P>'
   AND child_tenant_id = '<X>' AND revoked_at IS NULL;`
3. **Cedar fragment retirement** — emit fragment-registry update
   to mark the controlling-entity fragment as `superseded`; per-cell
   hot cache invalidation propagates within 1s.
4. **Denorm refresh** — remove X from P's `controls_tenants`;
   remove P from X's `controlled_by_tenants`.
5. **Audit-chain dual-seal** — emit `ConglomerateGrantRevoked` to
   both P's and X's audit streams.
6. **Per-pack regulator notification** — file SEC Form 10-12B if
   X is a US public-company subsidiary; file KR-FSS notification
   if X is a regulated financial subsidiary; file EU-ECB
   notification if X is an SSM-regulated bank subsidiary.
7. **Cross-cell notification** — if P and X are in different cells,
   broadcast the revocation event to both cells' policy-engine
   evaluators.
8. **Post-spinoff verification** — verify P's next read of X is
   refused; verify X's data residency and audit-chain are intact;
   verify X's KYB row is unchanged.

**Outcome.** X operates standalone immediately. No data migration.
No identity re-issuance. No cell re-binding. No audit-chain re-
sealing. The total saga duration is ~30 seconds (dominated by the
cross-cell hot-cache invalidation propagation).

**Real-world precedent.** AT&T's 2022 spinoff of WarnerMedia (which
became Warner Bros. Discovery): the spinoff was structurally a
permit transfer; the underlying data systems were already separate
sovereign tenants of the parent's IT environment. The legal
restructuring required SEC filings + tax treatment under IRC §355
but the technical restructuring at the platform layer was permit-
level.

#### §D-5.2 Acquisition

**Scenario.** New parent N acquires child X (previously a subsidiary
of old parent P or previously a standalone tenant).

**Workflow Engine saga** (`microservices/workflow-studio/sagas/
conglomerate-acquisition.yaml`):

1. **New parent KYB attestation** — N submits acquisition documents
   (purchase agreement, board resolutions of both N and X,
   regulatory clearances) — these compose the new KYB-attestation
   document chain.
2. **Pre-acquisition revocation** (if applicable) — if X was a
   subsidiary of old parent P, revoke the (P, X) grant first
   (see §D-5.1 spinoff).
3. **New grant creation** — `INSERT INTO conglomerate_grants
   (parent_tenant_id, child_tenant_id, scope, kyb_attestation_doc_
   hash, regulatory_citation, granted_at, granted_by_principal_id,
   audit_chain_seal_parent_id, audit_chain_seal_child_id) VALUES
   ('<N>', '<X>', '<scope-json>', 'sha256:<...>',
   '<jurisdiction-citation>', now(), '<corporate-officer>',
   '<seal-N>', '<seal-X>');`
4. **Cedar fragment creation** — publish the new controlling-entity
   fragment signed by the policy-engine bootstrap-tier0-HSM.
5. **Denorm refresh** — add X to N's `controls_tenants`; add N to
   X's `controlled_by_tenants`.
6. **Audit-chain dual-seal** — emit `ConglomerateGrantCreated` to
   both N's and X's audit streams.
7. **Per-pack regulator notification** — file HSR Act premerger
   notification if the deal exceeds the US antitrust threshold;
   file EU Merger Regulation notification if EU thresholds met;
   file KR-FTC notification if KR thresholds met.
8. **Post-acquisition verification** — verify N's next read of X
   passes through the new fragment; verify X's identity and data
   are unchanged.

**Real-world precedent.** Microsoft's 2023 acquisition of Activision
Blizzard: the closing was a permit transfer (Activision's M365
tenant came under Microsoft's MTO governance); Activision's data,
identity, and audit-streams remained sovereign.

#### §D-5.3 IPO of subsidiary

**Scenario.** Parent P takes subsidiary X public via IPO. X becomes
a publicly-traded company with its own shareholders. P may retain a
controlling stake, a minority stake, or fully exit.

**Workflow Engine saga** (`microservices/workflow-studio/sagas/
conglomerate-ipo.yaml`):

1. **Pre-IPO disclosure permits** — the new IPO disclosure (S-1
   filing for US, *증권신고서* for KR, prospectus for EU) requires
   permits to be issued to auditors (independent registered public
   accounting firm) and shareholders (via the offering syndicate);
   these are `audit-only` scope grants per §D-3.4.
2. **Determine post-IPO control state**:
   - **Full retention** — P retains controlling stake (>50%); the
     existing controlling-entity grant continues but with an
     amended `scope` reflecting public-company governance overlays.
   - **Partial retention** — P retains a significant minority (e.g.,
     20-50%); the grant is amended to `read-only-financial` +
     `audit-only` only; loss of board-control means
     `read-write-board-decisions` is revoked.
   - **Full exit** — P revokes the grant entirely (see §D-5.1).
3. **Public-disclosure permit creation** — create separate Cedar
   grants for auditor (registered firm), underwriter syndicate
   (for the offering period), and rating agencies (for credit
   ratings on debt).
4. **Audit-chain dual-seal for each permit change.**
5. **Per-pack regulator notification** — SEC Form S-1 + S-3 +
   ongoing 10-K/10-Q regime; KR-DART filing; EU prospectus
   directive filing.

**Real-world precedent.** Pfizer's 2020 spinoff of Upjohn (which
merged with Mylan to form Viatris): a partial spinoff via reverse-
Morris-Trust; the permit graph was amended in two steps (first
spinoff Upjohn, then Upjohn-Mylan merger creates a new parent).

#### §D-5.4 Joint venture

**Scenario.** Two parents A and B create a joint venture child JV
with explicit per-parent scope segregation.

**Workflow Engine saga**:

1. **JV tenant provisioning** — create JV as a new sovereign tenant
   per ADR-0244 §D-7 lifecycle, with its own KYB attestation, cell
   binding, audit-stream, and residency.
2. **Per-parent grant creation** — two grants are created:
   - (A, JV) grant with scope `{"tiers": ["read-only-financial",
     "joint-venture-partial"], "actions_subset":
     ["ReadJVPartialFinancialFacet", "WriteJVCapitalCallApproval"]}`.
   - (B, JV) grant with scope `{"tiers": ["read-only-operational",
     "joint-venture-partial"], "actions_subset":
     ["ReadJVPartialOperationalFacet"]}`.
3. **JV agreement evidence** — the JV agreement document hash is
   stored in both grants' `kyb_attestation_doc_hash`.
4. **Sunset configuration** — if the JV has a contractual end-date,
   both grants set `sunset_at` accordingly.
5. **Cedar fragments** — two fragments published, one per parent;
   each evaluates ONLY that parent's facet.
6. **Cross-parent information barrier** — the (A, JV) grant carries
   `information_barrier_set = ["B"]` and vice versa, so the JV's
   data is partitioned at the Cedar layer.

**Real-world precedent.** Sony-Ericsson (2001-2012) prior to Sony's
buyout — a 50/50 JV with explicit per-parent facets in the operating
agreement.

#### §D-5.5 Bankruptcy / receivership

**Scenario.** A court orders X into Chapter 11 (US) / 회생절차 (KR) /
EU-Insolvency-Regulation receivership. A court-appointed receiver
tenant R is granted temporary controlling-entity authority over X.
Original parent P's permit is suspended (not revoked — restored if
bankruptcy emerges with P retaining control).

**Workflow Engine saga**:

1. **Court warrant ingestion** — the court order document hash is
   the new KYB-equivalent attestation (per ADR-0312 court-warrant
   piercing pattern).
2. **Suspend (not revoke) parent's grant** — `UPDATE conglomerate_
   grants SET sunset_at = '<court-receivership-end-date>' WHERE
   parent_tenant_id = '<P>' AND child_tenant_id = '<X>' AND
   revoked_at IS NULL;` — the grant is now sunset-bound to the
   receivership period.
3. **Create receiver's grant** — (R, X) grant with full
   `read-write-board-decisions` scope, `regulatory_citation` set
   to the court order docket number, `granted_by_principal_id` set
   to the court-appointed receiver.
4. **Information barrier between P and R** — both grants carry each
   other in their `information_barrier_set` to prevent data
   bleeding between the original parent (now sidelined) and the
   receiver.
5. **Emergence handling** — on bankruptcy emergence:
   - If P retains control: P's grant `sunset_at` is cleared
     (UPDATE); R's grant revoked.
   - If new parent N acquires X via §363 sale: R revokes; (N, X)
     grant created (see §D-5.2 acquisition).
   - If X is liquidated: (R, X) grant revoked; X tenant lifecycle
     transitions to off-boarding per ADR-0244 §D-7.

**Real-world precedent.** Lehman Brothers 2008 bankruptcy — the court-
appointed trustee (Anton Valukas, then Bryan Marsal) had explicit
scope-limited authority that did not extend to Lehman's parent
ownership of its subsidiaries; this exact pattern is replicated here
via Cedar.

#### §D-5.6 Holding-of-holding (multi-level conglomerate)

**Scenario.** Conglomerate of conglomerates — e.g.,
`samsung-group-holdings` (top-level) controls
`samsung-electronics-holdings` (mid-level) controls
`samsung-electronics-mobile-division` (leaf). Each level requires
its own explicit grant per §D-4 invariant 1 (no transitive auto-
include).

**Permit chain (three explicit grants)**:

1. (samsung-group-holdings, samsung-electronics-holdings) — scope:
   `read-only-financial` + `audit-only` + `read-write-board-
   decisions`; regulatory citation:
   `KR-Commercial-Act-Art-342-지주회사`.
2. (samsung-electronics-holdings, samsung-electronics-mobile-
   division) — scope: `read-only-operational` +
   `read-write-board-decisions`; regulatory citation:
   `KR-Commercial-Act-Art-342-지주회사`.
3. (samsung-group-holdings, samsung-electronics-mobile-division)
   — **NOT auto-created**. If samsung-group-holdings wants to read
   samsung-electronics-mobile-division directly, it must hold its
   own grant against the leaf. This requires its own KYB attestation
   covering the leaf; in practice, top-level conglomerates rarely
   read at the leaf level (they read at the mid-level which then
   reads at the leaf).

**Cedar evaluation cost** — bounded O(1) per call (one grant lookup);
not O(N) on hierarchy depth.

**Real-world precedent.** Samsung Group's chaebol structure has
multiple levels (Samsung C&T → Samsung Electronics → Samsung
Electronics divisions); each cross-share-holding is governed by
KR-Commercial-Act-Art-342 *지주회사* rules.

#### §D-5.7 Reseller / channel-partner relationship

**Scenario.** A reseller R sells oyatie to end customers but retains
no ownership of the customer's data — R is the controlling-entity
for billing/lifecycle/admin-federation only.

**Grant shape**:

- (R, customer-tenant) grant with scope: `audit-only` + (limited
  facet of `payment-facilitation` covering invoice issuance, not
  payment-routing); `regulatory_citation`:
  `oyatie-reseller-program-terms-v3.2`.

The customer retains full sovereignty; the reseller cannot read
customer data beyond the billing-and-licensing surface.

**Real-world precedent.** Google Workspace reseller program: resellers
have administrative privileges scoped to billing, licensing, and
break-glass; customer data residency stays with the customer.

#### §D-5.8 Conglomerate dissolution

**Scenario.** Conglomerate parent P dissolves entirely; all children
revert to standalone sovereign tenants.

**Workflow Engine saga**:

1. **Enumerate all active grants where parent_tenant_id = P.**
2. **Per-child revocation** — revoke each (P, child) grant with
   `revoked_reason = 'parent-dissolution'`.
3. **Audit-chain dual-seal per revocation.**
4. **Per-pack regulator notification per child** — per the
   per-jurisdiction reporting obligation.
5. **P tenant off-boarding** — P transitions to ADR-0244 §D-7
   off-boarding lifecycle.

**Real-world precedent.** Tyco International's 2007 three-way split
(into Tyco Electronics, Tyco Healthcare, and Tyco International)
followed by Tyco International's later dissolution and merger with
Johnson Controls: each step was an explicit permit-graph rewrite.

#### §D-5.9 Cross-conglomerate transfer (corporate restructuring under PE)

**Scenario.** A private-equity firm PE holds a portfolio of children;
PE sells child X to another conglomerate N as part of portfolio
rebalancing.

This is a composite of §D-5.1 (spinoff from PE) + §D-5.2 (acquisition
by N), executed as a single Workflow Engine saga to ensure atomicity:

1. **Pre-checks** — verify sale agreement, both parties' board
   resolutions, regulatory consents (antitrust, sector-specific).
2. **Atomic revoke + grant** — within one Postgres transaction:
   - Revoke (PE, X) grant.
   - Insert (N, X) grant.
3. **Cedar fragment retirement + creation in lockstep.**
4. **Denorm refresh** — atomic update of `controls_tenants` on PE
   and N; atomic update of `controlled_by_tenants` on X.
5. **Dual-seal** — `ConglomerateGrantRevoked` to PE+X, `Conglomerate
   GrantCreated` to N+X.

**Real-world precedent.** KKR's portfolio rebalancing in 2024-2025
— multiple sub-portfolio transfers between KKR-controlled
conglomerates and external acquirers; each transaction is a permit
graph edit.

### §D-6 Joint venture + cross-child information barrier — worked example with regulatory ring-fences

**Scenario.** A holding company `bigbank-holdings` owns:

- `bigbank-retail-bank` (regulated retail bank subsidiary; Federal
  Reserve oversight; FDIC insured; subject to Bank Holding Company
  Act §225).
- `bigbank-investment-bank` (broker-dealer subsidiary; SEC + FINRA
  oversight; subject to SEC Reg M, Rule 105, MiFID II equivalent).

**Regulator-required ring-fence.** Per the Volcker Rule (12 CFR
§248) + Glass-Steagall-era information-barrier doctrine (which
survives in Sec 23A/23B Federal Reserve Act for affiliate
transactions), customer data from the retail bank may not be
transmitted to the investment bank for use in trading decisions.

**Grant shape**:

- (bigbank-holdings, bigbank-retail-bank): scope `read-only-financial`
  + `audit-only`; `information_barrier_set = ["bigbank-investment-bank"]`.
- (bigbank-holdings, bigbank-investment-bank): scope `read-only-
  financial` + `audit-only`; `information_barrier_set =
  ["bigbank-retail-bank"]`.

**Enforcement at runtime**:

- bigbank-holdings' principal P reads bigbank-retail-bank's consolidated
  P&L: permit. Audit-chain dual-seal.
- P reads bigbank-retail-bank's customer deposit ledger: permit (it
  is audit-only scope, aggregated). Audit-chain dual-seal.
- P then attempts to write that data into a workflow that touches
  bigbank-investment-bank's trading-decision system: **forbid**.
  The Cedar fragment evaluates the `information_barrier_check_passes`
  condition; the response data carries an `origin_tenant_id =
  bigbank-retail-bank` taint; the (P, bigbank-investment-bank) grant's
  `information_barrier_set` contains bigbank-retail-bank; the cross-
  disclosure is refused. Audit-chain emits
  `ConglomerateInformationBarrierCrossingRefused`.

**Worked precedent.** This pattern was the proximate cause of the
*Volcker Rule* (Dodd-Frank §619) which forbids proprietary trading
on customer data within bank holding companies; the regulatory
fix in 2010-2014 codified what Cedar now enforces at the platform
layer.

### §D-7 Stripe platform-facilitator pattern integration

Cross-reference ADR-0244 §D-3 `can_facilitate_sub_merchants` BOOL.
The Stripe platform-facilitator pattern is a SPECIAL CASE of
ADR-0313 where the controlling-entity scope is `payment-facilitation`
ONLY, not full read/write.

**Grant shape (Stripe-equivalent)**:

```sql
INSERT INTO conglomerate_grants (
    parent_tenant_id,
    child_tenant_id,
    scope,
    kyb_attestation_doc_hash,
    regulatory_citation,
    granted_at,
    granted_by_principal_id,
    audit_chain_seal_parent_id,
    audit_chain_seal_child_id
) VALUES (
    'shopify-platform-tenant',                  -- the marketplace platform
    'merchant-acme-shop',                       -- the connected merchant
    '{
        "tiers": ["payment-facilitation"],
        "actions_subset": [
            "FacilitatePayment",
            "RouteSettlementToConnectedAccount",
            "ApplyApplicationFee",
            "RefundFromPlatformBalance",
            "AssumeChargebackResponsibility"
        ]
    }'::jsonb,
    'sha256:<merchant-onboarding-form-attestation>',
    'US-PSP-EUDR-2023-§Title-III-Art-25-payment-facilitator',  -- and equivalents
    now(),
    'shopify-platform-tenant::officer::merchant-services-vp',
    '<seal-platform>',
    '<seal-merchant>'
);
```

The grant authorizes Shopify to route payments on behalf of
merchant-acme-shop but does NOT authorize Shopify to read
merchant-acme-shop's product catalog, customer PII, or business
operational data. The merchant retains full sovereignty over
non-payment surfaces.

The pattern is structurally identical to AWS Marketplace's seller-
agreement, Apple App Store's developer-agreement, Google Play's
developer-agreement, and Mercado Livre / Coupang / Lazada / Shopee
seller-onboarding flows — all of which are payment-facilitation
permits that do not bleed into operational ownership.

### §D-8 Implementation footprint — CI lanes

Six CI lanes are introduced:

| Lane name | Verifies | Promotion to BLOCKER |
|---|---|---|
| `oya-governance-conglomerate-grant-attestation-current` | Every `conglomerate_grants` row's `kyb_attestation_doc_hash` is ≤365 days old AND `regulatory_citation` is in the active-citations registry | 2026-07-16 |
| `oya-governance-cross-jurisdiction-residency-preserved` | Every cross-pack parent-read code path is residency-preserving per §D-4 invariant 2 | 2026-07-16 |
| `oya-governance-conglomerate-grant-dual-sealed` | Every conglomerate audit-event is dual-sealed; no half-sealed writes | 2026-07-16 |
| `oya-governance-conglomerate-grant-personal-tenant-deny` | Every conglomerate-grant code path refuses to evaluate when target is personal-tenant per ADR-0311 | 2026-07-16 |
| `oya-governance-conglomerate-information-barrier-coverage` | Every (parent, child-pair) requiring a regulator-mandated barrier has one configured | 2026-07-16 |
| `oya-governance-conglomerate-grant-transitivity-deny` | No Cedar evaluation walks the grant graph transitively | 2026-07-16 |

Plus a denorm-consistency lane:

| `oya-governance-conglomerate-grant-denorm-consistency` | The `tenants.controls_tenants` / `tenants.controlled_by_tenants` denorm matches the `conglomerate_grants` source-of-truth | 2026-07-16 |

### §D-9 Per-jurisdiction corporate-governance overlay

Each pack (per ADR-0010 + ADR-0299) carries its corporate-governance
regulatory anchor. The conglomerate-grant `regulatory_citation` MUST
be from the active-citations registry of at least one of the parent's
and child's packs.

#### §D-9.1 US

- **Delaware General Corporation Law (DGCL) Title 8 §203** —
  controlling-shareholder definition and business combination
  restrictions.
- **Sarbanes-Oxley Act §404** — internal-control attestation for
  subsidiaries of public companies.
- **Bank Holding Company Act (1956) §225.4** — Federal Reserve
  oversight of bank holding companies.
- **Federal Energy Regulatory Commission (FERC) §366.1** — energy-
  utility holding company information-barrier requirements.
- **SEC Regulation S-K Item 601** — subsidiaries-of-registrant
  disclosure required in 10-K filings.
- **Hart-Scott-Rodino Antitrust Improvements Act §7A** — premerger
  notification for transactions exceeding the size-of-transaction
  threshold.
- **SEC Rule 405 (Securities Act of 1933)** — definition of
  "controlled company."

#### §D-9.2 EU

- **EU Companies Directive 2017/1132** — uniform company-law
  directive on group structures.
- **SCE Statute (Regulation 1435/2003)** — European Cooperative
  Society holding structure.
- **ECB Single Supervisory Mechanism (SSM)** — bank holding company
  oversight at EU level.
- **EU Merger Regulation 139/2004** — premerger notification for
  EU-wide thresholds.
- **EBA Guidelines on internal governance (EBA/GL/2021/05)** —
  governance arrangements for institutions under CRR/CRD scope.

#### §D-9.3 KR (Korea)

- **KR-Commercial-Act-Art-342** (상법 제342조) — *지주회사*
  (holding company) definition and asset-share thresholds.
- **KR-Financial-Holding-Companies-Act** (금융지주회사법) — FSC
  oversight of financial holding companies.
- **KR-Monopoly-Regulation-and-Fair-Trade-Act-Art-14** —
  *대규모기업집단* (large business group / chaebol) regulation.
- **KR-Capital-Markets-Act-Art-9** — public-company subsidiary
  disclosure regime.

#### §D-9.4 JP (Japan)

- **Companies Act of Japan (会社法) Art 2 §3** — *親会社・子会社*
  (parent-subsidiary) definition.
- **Japanese SOX (J-SOX)** — internal-control attestation.
- **Anti-Monopoly Act (独占禁止法)** — corporate combination
  notification to the Japan Fair Trade Commission (JFTC).
- **Financial Instruments and Exchange Act** — Article 27 mandatory
  bid rules for parent acquisitions.

#### §D-9.5 UK

- **Companies Act 2006 §1159** — definition of subsidiary
  undertaking and parent undertaking.
- **UK Corporate Governance Code (2024 ed.)** — board governance of
  group structures.
- **UK Takeover Code (City Code)** — mandatory bid thresholds.
- **Financial Conduct Authority Listing Rules** — controlling-
  shareholder disclosure.

#### §D-9.6 CN (China)

- **PRC Company Law (公司法) Art 216** — definition of *控股股东*
  (controlling shareholder) and *实际控制人* (actual controller).
- **PRC Anti-Monopoly Law (反垄断法) Art 21** — concentration of
  undertakings declaration to SAMR.
- **PRC Foreign Investment Law** — foreign-controlled-entity
  restrictions for negative-list industries.
- **Cyberspace Administration of China (CAC) Data Outbound Security
  Assessment** — cross-border data transfer requirements that bound
  cross-jurisdiction conglomerate reads.

#### §D-9.7 IN (India)

- **Companies Act 2013 §2(87)** — subsidiary company definition.
- **SEBI (Listing Obligations and Disclosure Requirements)
  Regulations** — subsidiary disclosure for listed entities.
- **Competition Act 2002 §6** — combination notification to CCI.

#### §D-9.8 FR (France)

- **Code de Commerce Art L233-3** — *sociétés contrôlées*
  (controlled companies) definition.
- **AMF Réglement Général** — controlling-shareholder disclosure.

#### §D-9.9 DE (Germany)

- **Aktiengesetz §15** — *verbundene Unternehmen* (affiliated
  undertakings) framework.
- **Handelsgesetzbuch §271** — group-financial-statement obligations.
- **Kartellgesetz (GWB)** — merger control notification to
  Bundeskartellamt.

The active-citations registry is maintained at
`microservices/governance/data/regulatory-citations-registry.yaml`
per ADR-0299 cross-pack regulatory authoring discipline. Citations
that are repealed or withdrawn transition to the retired-citations
list; grants citing them must be re-attested under a successor
citation within the per-pack sunset window (typically 90 days).

### §D-10 Observability — emitted audit-event classes

Per ADR-0263 audit-chain emission contract, the following six event
classes are added to the canonical registry:

#### §D-10.1 `ConglomerateGrantCreated`

```yaml
event_class: ConglomerateGrantCreated
binding_adr: ADR-0313
emit_on: grant insertion to conglomerate_grants
schema:
  parent_tenant_id: string
  child_tenant_id: string
  scope: object (per §D-3 shape)
  kyb_attestation_doc_hash: string (sha256)
  regulatory_citation: string
  granted_at: timestamp
  granted_by_principal_id: string
  audit_chain_seal_parent_id: string
  audit_chain_seal_child_id: string
retention_class: permanent
emission_targets:
  - parent.audit_stream
  - child.audit_stream
cardinality_per_cell_per_second: 10  (low; grant lifecycle is rare)
```

#### §D-10.2 `ConglomerateGrantRevoked`

```yaml
event_class: ConglomerateGrantRevoked
binding_adr: ADR-0313
emit_on: revocation of conglomerate_grants row
schema:
  grant_id: string (uuid)
  parent_tenant_id: string
  child_tenant_id: string
  revoked_at: timestamp
  revoked_by_principal_id: string
  revoked_reason: string (enum per §D-2 schema)
  audit_chain_seal_parent_id: string
  audit_chain_seal_child_id: string
retention_class: permanent
emission_targets:
  - parent.audit_stream
  - child.audit_stream
cardinality_per_cell_per_second: 1  (very low)
```

#### §D-10.3 `ConglomerateParentReadAction`

```yaml
event_class: ConglomerateParentReadAction
binding_adr: ADR-0313
emit_on: parent's principal reads child resource via the grant
schema:
  parent_tenant_id: string
  child_tenant_id: string
  action: string (Cedar action namespace)
  resource_ref: string
  scope_match: string (which §D-3 tier matched)
  applied_fragments: array of string
  evaluation_ms: float
  evaluation_id: uuid
retention_class: 7-years-or-jurisdiction-floor
emission_targets:
  - parent.audit_stream
  - child.audit_stream
cardinality_per_cell_per_second: 10000  (steady-state)
                                  100000 (peak)
```

#### §D-10.4 `ConglomerateCrossJurisdictionResidencyEnforced`

```yaml
event_class: ConglomerateCrossJurisdictionResidencyEnforced
binding_adr: ADR-0313
emit_on: ADR-0304 invariant trips on a conglomerate parent-read;
         data is viewable in-cell but not exfiltrated
schema:
  parent_tenant_id: string
  child_tenant_id: string
  parent_pack: string
  child_pack: string
  data_class_max: integer (per ADR-0099)
  redaction_applied: array of string (fields redacted)
retention_class: 7-years-or-jurisdiction-floor
emission_targets:
  - parent.audit_stream
  - child.audit_stream
cardinality_per_cell_per_second: 100  (moderate; cross-jurisdiction
                                       conglomerates are common)
```

#### §D-10.5 `ConglomerateInformationBarrierCrossingRefused`

```yaml
event_class: ConglomerateInformationBarrierCrossingRefused
binding_adr: ADR-0313
emit_on: parent attempts to cross a regulator-required information
         barrier between two children (§D-4 invariant 5)
schema:
  parent_tenant_id: string
  origin_child_tenant_id: string  (the tenant the data came from)
  target_child_tenant_id: string  (the tenant the data was being
                                   written into)
  information_barrier_set: array of string
  regulator_citation: string  (which regulator requires the barrier)
retention_class: permanent
emission_targets:
  - parent.audit_stream
  - origin_child.audit_stream
  - target_child.audit_stream    (triple-sealed for this class)
  - regulator-evidence-stream    (per ADR-0263 regulator emission)
cardinality_per_cell_per_second: 1  (very low; refusal is expected
                                     to be rare in well-architected
                                     systems)
```

#### §D-10.6 `ConglomeratePersonalTenantBoundaryRefused`

```yaml
event_class: ConglomeratePersonalTenantBoundaryRefused
binding_adr: ADR-0313
emit_on: parent attempts to read a child employee's personal-tenant
         data (§D-4 invariant 3 + ADR-0311)
schema:
  parent_tenant_id: string
  child_tenant_id: string
  personal_tenant_id: string  (the boundary that was preserved)
  attempted_resource_ref: string
  refusal_reason: string
retention_class: permanent
emission_targets:
  - parent.audit_stream
  - child.audit_stream
  - personal_tenant.audit_stream (triple-sealed; the personal tenant
                                   has visibility into refused
                                   attempts per ADR-0311 §D-5)
cardinality_per_cell_per_second: 1  (very low)
```

## §E Implementation footprint

### §E.1 New crate

A new shared crate is introduced per the substrate-promotion discipline:

```
crates/oya-shared-conglomerate-grant-evaluator/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── api.rs                  # ConglomerateEvaluationRequest/Response
│   ├── evaluator.rs            # Cedar evaluation against grant table
│   ├── grant_store.rs          # Postgres CRUD for conglomerate_grants
│   ├── denorm.rs               # tenants.controls_tenants denorm maintenance
│   ├── invariants.rs           # §D-4 six invariants as Rust validators
│   ├── audit_seal.rs           # dual-seal write coordinator
│   ├── information_barrier.rs  # §D-4 invariant 5 enforcement
│   ├── personal_tenant.rs      # §D-4 invariant 3 enforcement (delegates to ADR-0311 crate)
│   ├── residency.rs            # §D-4 invariant 2 enforcement (delegates to ADR-0299 crate)
│   ├── transitivity.rs         # §D-4 invariant 1 enforcement
│   ├── attestation.rs          # §D-4 invariant 4 (KYB + regulatory citation freshness)
│   ├── cache.rs                # per-cell Valkey hot-cache adapter
│   ├── saga.rs                 # Workflow Engine saga adapter for §D-5 restructuring
│   └── error.rs
├── tests/
│   ├── unit_evaluator.rs
│   ├── prop_invariants.rs       # proptest properties for §D-4
│   ├── e2e_spinoff.rs
│   ├── e2e_acquisition.rs
│   ├── e2e_ipo.rs
│   ├── e2e_joint_venture.rs
│   ├── e2e_bankruptcy.rs
│   ├── e2e_dissolution.rs
│   └── load_evaluations.rs      # 100K eval/sec sustained
├── benches/
│   ├── evaluate_cache_hit.rs
│   └── evaluate_cache_miss.rs
├── fuzz/
│   └── fuzz_targets/
│       └── cedar_fragment_input.rs
└── README.md
```

Crate name: `oya-shared-conglomerate-grant-evaluator` (per ADR-0017
crate-naming convention: `oya-<layer>-<concern>-<modifier>` where
`shared` is the layer, `conglomerate-grant` is the concern, and
`evaluator` is the modifier).

### §E.2 Per-µservice consumers

The following existing µservices consume the new crate:

#### §E.2.1 `microservices/ops-dashboard-control-center/`

The primary consumer. The consolidated-dashboard surface (group-
wide ops dashboard, finops rollup view, audit-evidence aggregator)
is exactly the parent-reads-child use case. Routes:

- `GET /api/v1/conglomerate/{parent_tenant_id}/dashboard/consolidated`
- `GET /api/v1/conglomerate/{parent_tenant_id}/dashboard/per-child/{child_tenant_id}`
- `GET /api/v1/conglomerate/{parent_tenant_id}/audit-evidence`

Each route invokes the conglomerate evaluator before serving any
child data; refusals surface as HTTP 403 with structured Cedar
denial reason.

#### §E.2.2 `microservices/finops-portal/`

Consolidated billing rollups for conglomerate parents. Routes:

- `GET /api/v1/finops/conglomerate/{parent_tenant_id}/quarterly-rollup`
- `GET /api/v1/finops/conglomerate/{parent_tenant_id}/per-child-cost/{child_tenant_id}`
- `POST /api/v1/finops/conglomerate/{parent_tenant_id}/budget/allocate`

Cross-jurisdiction billing rollups invoke the §D-3.5 residency-
preserved scope.

#### §E.2.3 `microservices/audit-chain/`

The dual-seal write path. The audit-chain emission backbone is
extended with a `dual_seal_write(parent_event, child_event)` API
that writes both events transactionally — both succeed or both
fail (no half-sealed states).

#### §E.2.4 `microservices/tenancy/`

The lifecycle owner of the `conglomerate_grants` table. Implements:

- `POST /admin/conglomerate-grant` (create)
- `DELETE /admin/conglomerate-grant/{grant_id}` (revoke)
- `GET /admin/conglomerate-grants?parent={...}&child={...}`
- `POST /admin/conglomerate-grant/{grant_id}/re-attest` (KYB
  re-attestation)

#### §E.2.5 `microservices/policy-engine/`

The Cedar fragment registry owner. Implements:

- Bootstrap fragment for the `ControllingEntity` entity type.
- Per-grant fragment generation (Workflow Engine saga publishes
  fragments here on grant creation).
- Hot-reload on grant revocation/sunset.

#### §E.2.6 `microservices/workflow-studio/`

The orchestrator for the eight restructuring sagas (§D-5):

- `conglomerate-spinoff.yaml`
- `conglomerate-acquisition.yaml`
- `conglomerate-ipo.yaml`
- `conglomerate-joint-venture-formation.yaml`
- `conglomerate-joint-venture-dissolution.yaml`
- `conglomerate-bankruptcy-receivership.yaml`
- `conglomerate-dissolution.yaml`
- `conglomerate-cross-conglomerate-transfer.yaml`

Each saga is idempotent and resumable per ADR-0035 workflow engine
state-machine + DAG hybrid.

### §E.3 New runbooks

Six new runbooks are introduced at `docs/runbooks/`:

- `conglomerate-grant-creation.md` — operator procedure for new
  grant creation, including KYB doc-hash computation and regulator-
  citation verification.
- `conglomerate-grant-revocation.md` — operator procedure for
  voluntary revocation; differs from spinoff saga by skipping the
  workflow-engine orchestration (intended for emergency manual
  revocation).
- `conglomerate-grant-attestation-renewal.md` — annual KYB
  re-attestation procedure.
- `conglomerate-grant-denorm-drift-recovery.md` — recovery procedure
  when the `tenants.controls_tenants` denorm drifts from
  `conglomerate_grants` source-of-truth.
- `conglomerate-information-barrier-incident.md` — incident response
  when `ConglomerateInformationBarrierCrossingRefused` event fires
  (typically requires regulator notification within 72 hours).
- `conglomerate-cross-jurisdiction-residency-breach.md` — incident
  response when a residency-preservation invariant is violated (this
  should NOT happen at runtime per the §D-4 invariants; but the
  runbook covers the case where a Cedar fragment bug allowed
  exfiltration and the corpus needs remediation per ADR-0049 §D-7
  cross-region replication recovery).

### §E.4 New CI lanes

Seven CI lanes (six §D-8 invariant lanes + one denorm-consistency
lane) are introduced under `tools/ci/lanes/`:

- `oya-governance-conglomerate-grant-attestation-current` —
  scheduled daily; emits `INFO`/`WARN`/`FAIL` per grant row.
- `oya-governance-cross-jurisdiction-residency-preserved` —
  on-commit; greps for cross-jurisdiction parent-read code paths
  and verifies residency-preserving wrappers.
- `oya-governance-conglomerate-grant-dual-sealed` — on-commit;
  verifies every audit-chain emission for conglomerate events uses
  the `dual_seal_write` API.
- `oya-governance-conglomerate-grant-personal-tenant-deny` —
  on-commit; verifies the personal-tenant boundary check is wired.
- `oya-governance-conglomerate-information-barrier-coverage` —
  scheduled daily; cross-references the regulator-required-barrier
  registry with active grants and flags any (parent, child-pair)
  combination requiring a barrier that lacks one.
- `oya-governance-conglomerate-grant-transitivity-deny` —
  on-commit; static-analysis pass on the evaluator crate to verify
  no transitive walk path exists.
- `oya-governance-conglomerate-grant-denorm-consistency` —
  scheduled hourly; runs
  `SELECT * FROM oyatie_assert_conglomerate_denorm_consistent()`
  and flags any drift rows.

All seven lanes promote to BLOCKER on 2026-07-16 per the
2026-05-20 keystone bundle deadline.

### §E.5 Vendor selection rationale

No new vendor introduced. The conglomerate evaluator is in-house
Rust code on top of Cedar v4.2 LTS (already in the portfolio per
ADR-0150). The Postgres tables run on the existing per-cell DB tier
(per ADR-0045). The Valkey hot cache is already provisioned for
Cedar evaluation (per ADR-0046).

Per ADR-0211 in-house tech stack preference: building the conglomerate
evaluator in-house is correct because:

- The primitive is differentiating (no off-the-shelf product
  provides this exact composition of tenant + Cedar + dual-seal
  + per-jurisdiction overlay).
- The performance budget (1 ms P99 per evaluation) is tight; an
  external vendor adds network hop latency.
- The audit-chain dual-seal coupling is platform-specific; no
  external vendor would coordinate writes to two of our streams.
- The seven invariants are policy-discipline that we MUST own.

## §F Migration

The migration is staged in three waves.

### §F.1 Wave A — substrate land (2026-Q3)

1. Land migration `0014_conglomerate_grants.sql` in tenancy µservice.
2. Land migration `0015_tenant_conglomerate_index_columns.sql`.
3. Publish the new spec at `/specs/conglomerate-grant-model.json`.
4. Publish the new crate `oya-shared-conglomerate-grant-evaluator`
   at version 0.1.0 (pre-release).
5. Publish the bootstrap Cedar fragments (entity type +
   default-deny + skeleton permits) into the policy-engine fragment
   registry signed by tier-0 HSM.
6. Wire the six new audit-event classes into the audit-chain
   emission contract (per ADR-0263 §D-3 registry extension).
7. Add the seven CI lanes in advisory mode.

### §F.2 Wave B — identify existing conglomerate-relationship candidates (2026-Q3 → 2026-Q4)

The tenancy µservice runs a one-time scan of the existing tenant
corpus to identify candidates:

- Tenants whose `primary_tenants` field (per ADR-0244 §D-3) lists
  `PARTNER_AGENCY` or `RESELLER` audience types are candidates for
  reseller grants per §D-5.7.
- Tenants whose `can_facilitate_sub_merchants = TRUE` (per ADR-0244
  §D-3) are candidates for Stripe-Connect-style payment-facilitation
  grants per §D-3.7 / §D-7.
- Tenants whose KYB records list a parent organization are
  candidates for controlling-entity grants per §D-3.1-§D-3.6.

For each candidate, the tenancy µservice surfaces an "operator
prompt" in the ops-dashboard-control-center for the corporate
officer to confirm the grant shape, regulatory citation, and
audit-chain dual-seal targets.

### §F.3 Wave C — grant-creation workflow + CI promotion (2026-Q4)

1. For each confirmed candidate, the operator runs the
   `conglomerate-grant-creation.md` runbook (per §E.3) which
   invokes the Workflow Engine saga to create the grant
   transactionally.
2. The seven CI lanes promote from advisory to BLOCKER on 2026-07-16
   per the keystone bundle deadline.
3. Any tenant that meets the candidate criteria but has not had a
   grant confirmed by 2026-07-16 surfaces in the
   `oya-governance-conglomerate-grant-attestation-current` lane as
   a FAIL — the operator must resolve before the lane's daily
   evidence sweep completes.

### §F.4 What is NOT migrated

The following items are explicitly NOT changed:

- The `tenants` table primary key column (`tenant_id`) — unchanged.
- The Cedar entity type `Tenant` — unchanged; `ControllingEntity` is
  a Role on Tenant, not a replacement.
- The `audience_type` enum (per ADR-0244 §D-11) — unchanged.
- The cell-binding model (per ADR-0009) — unchanged. Parent and
  child stay in their respective cells.
- The pack-overlay model (per ADR-0010) — unchanged.

### §F.5 Rollback path

If the conglomerate substrate must be rolled back (e.g., a critical
bug surfaces post-substrate-land):

1. Disable the six CI lanes (move back to advisory).
2. Revoke all active `conglomerate_grants` (their Cedar fragments
   are unloaded; parents lose access to children).
3. Drop the `controls_tenants` and `controlled_by_tenants` columns
   from `tenants` (DDL rollback).
4. Drop the `conglomerate_grants` table (DDL rollback).
5. Audit-chain events for `ConglomerateGrantCreated` and friends
   remain in the audit-stream (they are permanent per their
   retention class) — they document the rolled-back state.

Roll-back time: ~30 minutes for the DDL + ~1 hour for the Cedar
fragment unload propagation across cells. Total operator time: 2
hours.

### §F.6 Risk register

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Denorm drift causes consolidated-view to omit a child | Medium | Medium | Hourly consistency-check lane; denorm written transactionally with source-of-truth |
| Cedar fragment signature key compromise (tier-0 HSM) | Very Low | Catastrophic | Shamir-shared key per ADR-0150 §security; rotation per ADR-0043 secrets management |
| Information barrier crossing not caught by static analysis | Low | High | Triple-sealed audit emission + regulator-notification runbook; lane verifies coverage |
| Per-pack residency check false-positive (legitimate read refused) | Low | Medium | Per-pack overlay configurable; refusal carries actionable error |
| Workflow saga partial-fail leaves grant + Cedar fragment desynced | Low | High | Saga is idempotent + resumable; dual-seal ensures atomicity at audit layer |
| Operator misconfigures `information_barrier_set` and a regulator-required barrier is missed | Medium | High | Coverage lane cross-references regulator-required-barrier registry; surfaces missing barriers as FAIL |

## §G References

### §G.1 Hyperscaler precedents

- AWS Organizations User Guide 2024 ed. ch. 3 "Account management";
  AWS re:Invent 2023 SEC305 "Multi-account architectures at scale";
  AWS Whitepaper "Organizing your AWS environment using multiple
  accounts" Aug 2024 rev.
- Microsoft Entra ID Docs 2024 "Multi-tenant organizations overview";
  Microsoft Build 2024 keynote + IDN508 session; Microsoft Purview
  audit-log cross-tenant search documentation 2024.
- Google Workspace Admin Help 2024 "Reseller features"; Google
  Cloud Resource Manager Docs 2024 "Resource hierarchy"; Google CRE
  Book ch. 8 "Managing change at scale."
- Stripe Engineering Blog 2024 "Designing for global
  platforms"; Stripe API Reference 2025 ed. "Accounts" + "Application
  Fees" + "Transfers"; Stripe Sessions 2024 "platform
  mechanics."
- Salesforce Architects 2024 "Multi-org strategy"; Trailhead module
  "Multi-Org Strategy and Architecture."
- Bloomberg Enterprise Solutions 2024 documentation; Bloomberg
  Compliance Center customer briefings 2024.
- Apple Business Manager User Guide 2024 ed.; WWDC 2024 "What's new
  in managing Apple devices."
- Atlassian Cloud Architecture documentation 2024 "Organizations and
  admin hub"; Atlassian Team '24 IDX-101 session.
- Okta Architectural Guidance 2024 "Hub-and-spoke architecture for
  federated identity"; Oktane '24 IAM-205.
- Slack Enterprise Grid documentation 2024; Slack Frontiers 2024
  "Designing for multi-org enterprises."

### §G.2 Standards + RFCs

- Cedar Policy Language v4.2 LTS specification (AWS, 2024).
- RFC 8259 (JSON) for the `scope` jsonb shape.
- RFC 4122 (UUID) for `grant_id`.
- RFC 6234 (SHA-256) for `kyb_attestation_doc_hash`.

### §G.3 Regulatory citations (per §D-9)

#### US

- Delaware General Corporation Law Title 8 §203.
- Sarbanes-Oxley Act §404.
- Bank Holding Company Act (1956) §225.4.
- Federal Energy Regulatory Commission Order §366.1.
- SEC Regulation S-K Item 601.
- Hart-Scott-Rodino Antitrust Improvements Act §7A.
- SEC Rule 405 (Securities Act of 1933).
- Volcker Rule (Dodd-Frank Act §619) + 12 CFR §248.
- Federal Reserve Act §23A / §23B (affiliate transactions).
- IRC §355 (tax-free spinoff treatment).
- HIPAA §164.504(e) (Business Associate scope).
- HIPAA §164.514(e) (limited dataset).

#### EU

- EU Companies Directive 2017/1132.
- SCE Statute (Regulation 1435/2003).
- ECB SSM Regulation 1024/2013.
- EU Merger Regulation 139/2004.
- EBA Guidelines on internal governance EBA/GL/2021/05.
- GDPR Article 6 (lawful basis) and Article 28 (processor).
- MiFID II Article 16(8) (information barriers in investment
  firms).

#### KR

- KR-Commercial-Act-Art-342 (상법 제342조).
- KR-Financial-Holding-Companies-Act (금융지주회사법).
- KR-Monopoly-Regulation-and-Fair-Trade-Act-Art-14.
- KR-Capital-Markets-Act-Art-9.

#### JP

- Companies Act of Japan (会社法) Art 2 §3.
- Japanese SOX (J-SOX) Financial Instruments and Exchange Act Art
  24-4-4.
- Anti-Monopoly Act (独占禁止法) Art 9.
- Financial Instruments and Exchange Act Art 27.

#### UK

- Companies Act 2006 §1159.
- UK Corporate Governance Code 2024 ed.
- UK Takeover Code (City Code).
- FCA Listing Rules LR 14.

#### CN

- PRC Company Law (公司法) Art 216.
- PRC Anti-Monopoly Law (反垄断法) Art 21.
- PRC Foreign Investment Law.
- CAC Data Outbound Security Assessment Measures (2023).

#### IN

- Companies Act 2013 §2(87).
- SEBI (LODR) Regulations.
- Competition Act 2002 §6.

#### FR

- Code de Commerce Art L233-3.
- AMF Règlement Général.

#### DE

- Aktiengesetz §15.
- Handelsgesetzbuch §271.
- Kartellgesetz (GWB).

### §G.4 Internal portfolio ADRs

- ADR-0009 Cell architecture (per-tenant per-region).
- ADR-0010 Regional pack architecture.
- ADR-0035 Workflow engine state-machine and DAG hybrid.
- ADR-0037 Public API stability tiers and deprecation.
- ADR-0042 Observability stack OTel.
- ADR-0043 Secrets management OpenBao and HSM per cell.
- ADR-0044 Service mesh Istio ambient and Envoy gateway.
- ADR-0045 Database tier strategy.
- ADR-0046 Vector store strategy (cache tier crossover).
- ADR-0049 Cross-region replication and residency.
- ADR-0099 Data class registry.
- ADR-0105 Thirteen-layer canonical enum.
- ADR-0128 Hyperscaler architecture invariants.
- ADR-0145 Inter-microservice communication reform.
- ADR-0150 Cedar policy engine.
- ADR-0174 FinOps sustainability tagging.
- ADR-0176 Brown-out degradation signal.
- ADR-0183 Policy engine separation Cedar app-authz Kyverno
  admission.
- ADR-0211 In-house tech stack preference.
- ADR-0212 Buildability doctrine.
- ADR-0215 Multi-context platform.
- ADR-0218 Tenant-granular control surface.
- ADR-0240 Sovereign cloud per regional pack.
- ADR-0241 DR business continuity portfolio policy.
- **ADR-0242 oyatie-is-a-tenant doctrine** (keystone #1).
- **ADR-0243 Cedar as universal gate** (keystone #2; the load-
  bearing primitive).
- **ADR-0244 Tenant as universal scoping primitive** (keystone #3;
  this ADR's flat-tenant base).
- ADR-0245 Substrate vs product layering (keystone #4).
- ADR-0246 Policy engine substrate promotion.
- ADR-0247 Self-hosting / self-modification doctrine.
- ADR-0248 Amazon-shape cellular architecture.
- ADR-0249 Multi-category marketplace doctrine.
- ADR-0251 Compliance pack cell certification levels.
- ADR-0258 Canonical schema-evolution policy.
- ADR-0263 Observability emission contract.
- ADR-0276 Backup portability format (GDPR Article 20).
- ADR-0284 Reserved-namespace registry.
- ADR-0297 Abuse-defence baseline (anti-bot, anti-spoof, anti-scrape).
- ADR-0299 Cross-pack data residency conflict arbitration.
- ADR-0304 Cross-jurisdiction conflict resolution.
- ADR-0311 Dual-tenant identity personal-vs-work boundary (in-flight
  by parallel agent — cite as authoritative).
- ADR-0312 Court-warrant-scoped piercing (in-flight by parallel
  agent — cite as authoritative).

### §G.5 Standards docs

- docs/standards/documentation-rigor.md §1.1 + §1.2 + §2 ADR-row
  + §3.2.1 + §3.2.5.
- docs/standards/cedar-policy-discipline.md.
- docs/standards/tenant-lifecycle.md.
- docs/standards/regulatory-pack-authzpolicy-overlays.md.
- docs/standards/sovereign-cloud-overlay.md.
- docs/standards/observability-slo.md.

### §G.6 Auto-memory feedback (related)

- feedback_oyatie_is_a_tenant_doctrine.
- feedback_cedar_as_universal_gate.
- feedback_tenant_as_universal_scoping_primitive.
- feedback_bominal_inheritance_precedence.
- feedback_no_silent_regression.
- feedback_canonical_base_localization.
- feedback_quality_performance_scalability_bar.
- feedback_clean_architecture_requirements.
- feedback_autonomous_decision_principles.
- feedback_self_modification_doctrine.
- feedback_compliance_pack_primitive.

## §H Change log + naming-justifications

### §H.1 Change log

| Date | Author | Change |
|---|---|---|
| 2026-05-20 | council-architecture | Initial draft authored as conglomerate-layer companion to ADR-0244; lands in keystone-companion cadence per the 2026-05-20 bundle |

### §H.2 Naming-justifications

Per the project rule that every new name introduced carries a one-
line justification proving conformance to v4 BNF + 12-layer-enum
+ canonical naming conventions:

#### Entity types

- **`ControllingEntity`** — Cedar role on `Tenant`; named after the
  Companies-Act-2006-§1159 / KR-Commercial-Act-Art-342 /
  US-DGCL-§203 universally-recognized term *controlling entity*;
  not a separate principal class (kept Tenant-rooted per ADR-0244
  §D-4); BNF: `<role>` = `controlling-entity`.

#### Crates

- **`oya-shared-conglomerate-grant-evaluator`** — `oya-` prefix +
  `shared` layer (per the 12-layer enum / ADR-0105 13-layer canonical
  enum) + `conglomerate-grant` concern + `evaluator` modifier; the
  `shared` layer is correct because the crate is a thin in-process
  evaluator consumed by multiple µservices, not a new µservice itself
  (which would be `oya-microservice-*`); BNF compliant per ADR-0017
  crate-naming convention.

#### Audit-event classes

- **`ConglomerateGrantCreated`** — `<Subject><Verb>` CamelCase per
  ADR-0263 audit-event-class naming; subject `ConglomerateGrant`
  is the noun the event is about; verb `Created` is the past-tense
  state-transition.
- **`ConglomerateGrantRevoked`** — same shape; verb `Revoked` is
  the inverse of `Created`.
- **`ConglomerateParentReadAction`** — `<Subject><Object><Action>`
  CamelCase; subject is `ConglomerateParent`, object is the
  read-action surface; emitted on every parent action against a
  child resource.
- **`ConglomerateCrossJurisdictionResidencyEnforced`** —
  `<Subject><Condition><State>` CamelCase; subject is
  `ConglomerateCrossJurisdiction`, condition is `Residency`, state
  is `Enforced`; emitted when ADR-0304 invariant trips during
  conglomerate parent-read.
- **`ConglomerateInformationBarrierCrossingRefused`** —
  `<Subject><Surface><Refused>` CamelCase; subject is
  `ConglomerateInformationBarrier`, surface is `Crossing`, refusal
  flag `Refused`; emitted when §D-4 invariant 5 trips.
- **`ConglomeratePersonalTenantBoundaryRefused`** — same shape;
  emitted when §D-4 invariant 3 trips per ADR-0311.

#### Postgres tables

- **`conglomerate_grants`** — `<noun-plural>` lower_snake_case per
  ADR-0058 postgres-table-naming; noun is `conglomerate_grant` (the
  Cedar permit row); plural is conventional for collection tables;
  binding-ADR `ADR-0313` cited in DDL header comment.

#### Postgres columns (new on `tenants` table)

- **`controls_tenants`** — `<verb-plural><object>` lower_snake_case;
  verb `controls` (third-person singular present of `to control`),
  object `tenants` (plural because the column is a TEXT[]); the
  column reads naturally: "this tenant controls these tenants"; the
  parallel reverse-direction column follows the same shape.
- **`controlled_by_tenants`** — `<past-participle>_<by>_<object>`
  lower_snake_case; reverse direction of `controls_tenants`; reads
  naturally as "this tenant is controlled-by these tenants."

#### Cedar action namespaces

- **`ParentScope::ReadActions`** — `<ParentNoun>::<VerbCategoryActions>`
  per Cedar-namespace convention (`Action::ReadProducts`,
  `Action::WriteOrders` in AWS Verified Permissions examples are the
  canonical shape); `ParentScope` is the namespace common to
  conglomerate scopes; `ReadActions` is the action-group.
- **`ParentScope::WriteActions`** — same shape; group of mutating
  parent-actions.
- **`ParentScope::AuditActions`** — same shape; group of audit-only
  parent-actions.
- **`ParentScope::JointVentureActions`** — same shape; group of
  JV-specific parent-actions.
- **`ParentScope::PaymentFacilitationActions`** — same shape; group
  of Stripe-Connect-equivalent payment-facilitation actions.
- **`ParentScope::CrossJurisdictionReadActions`** — same shape;
  group of cross-jurisdiction read-actions with residency
  preservation.

#### CI lane names

- **`oya-governance-conglomerate-grant-attestation-current`** —
  `oya-<layer>-<concern>-<sub-concern>-<state>` per ADR-0123
  CI-lane-naming convention; `governance` is the layer (per ADR-0132
  oya-governance-* prefix); `conglomerate-grant` is the concern;
  `attestation` is the sub-concern; `current` is the desired-state
  (the lane verifies attestations are not stale).
- **`oya-governance-cross-jurisdiction-residency-preserved`** — same
  shape; `cross-jurisdiction-residency` is the sub-concern;
  `preserved` is the desired-state.
- **`oya-governance-conglomerate-grant-dual-sealed`** — same shape;
  `dual-sealed` is the desired-state (every emission is dual-sealed).
- **`oya-governance-conglomerate-grant-personal-tenant-deny`** — same
  shape; `personal-tenant-deny` is the desired-state (the boundary
  is always denied).
- **`oya-governance-conglomerate-information-barrier-coverage`** —
  same shape; `coverage` is the desired-state (every regulator-
  required barrier is configured).
- **`oya-governance-conglomerate-grant-transitivity-deny`** — same
  shape; `transitivity-deny` is the desired-state (transitive walks
  are denied at static-analysis time).
- **`oya-governance-conglomerate-grant-denorm-consistency`** — same
  shape; `denorm-consistency` is the desired-state (the denorm
  matches the source-of-truth).

#### Workflow saga names

- **`conglomerate-spinoff.yaml`** — `<concern>-<verb>.yaml` per
  Workflow-Studio saga-naming convention; concern is `conglomerate`;
  verb is `spinoff`; the saga is the canonical implementation of
  §D-5.1.
- **`conglomerate-acquisition.yaml`** — same shape; verb is
  `acquisition`; implements §D-5.2.
- **`conglomerate-ipo.yaml`** — same shape; verb is `ipo`;
  implements §D-5.3.
- **`conglomerate-joint-venture-formation.yaml`** — same shape;
  verb is the compound `joint-venture-formation`; implements §D-5.4.
- **`conglomerate-joint-venture-dissolution.yaml`** — same shape;
  inverse of the formation saga.
- **`conglomerate-bankruptcy-receivership.yaml`** — same shape;
  verb is the compound `bankruptcy-receivership`; implements §D-5.5.
- **`conglomerate-dissolution.yaml`** — same shape; verb is
  `dissolution`; implements §D-5.8.
- **`conglomerate-cross-conglomerate-transfer.yaml`** — same shape;
  verb is the compound `cross-conglomerate-transfer`; implements
  §D-5.9.

#### Scope-tier names

- **`read-only-financial`** — `<access-class>-<domain>` kebab-case
  per the per-tier scope-naming convention; `read-only` is the
  access class; `financial` is the domain.
- **`read-only-operational`** — same shape; domain is `operational`.
- **`read-write-board-decisions`** — `<access-class>-<compound-
  domain>` kebab-case; access class is `read-write`; compound-domain
  is `board-decisions`.
- **`audit-only`** — `<access-class>` kebab-case; access class is
  `audit-only`; no domain qualifier needed (it is its own scope).
- **`cross-jurisdiction-read-only`** — `<modifier>-<access-class>`
  kebab-case; modifier is `cross-jurisdiction`; access class is
  `read-only`.
- **`joint-venture-partial`** — `<context>-<modifier>` kebab-case;
  context is `joint-venture`; modifier is `partial` (per-parent
  facet).
- **`payment-facilitation`** — `<concern>-<modifier>` kebab-case;
  concern is `payment`; modifier is `facilitation`; chosen to match
  Stripe Connect's vocabulary verbatim per the §A.2 hyperscaler
  precedent (precedent-naming-fidelity per the doctrine of citing
  established industry terms).

#### Runbook names

- **`conglomerate-grant-creation.md`** — `<concern>-<verb>.md` per
  runbook-naming convention.
- **`conglomerate-grant-revocation.md`** — same shape.
- **`conglomerate-grant-attestation-renewal.md`** — same shape with
  compound verb.
- **`conglomerate-grant-denorm-drift-recovery.md`** — same shape
  with compound verb; the recovery-runbook naming pattern is
  consistent with ADR-0049 cross-region-replication runbook family.
- **`conglomerate-information-barrier-incident.md`** — incident-
  runbook naming pattern per ADR-0042 observability stack.
- **`conglomerate-cross-jurisdiction-residency-breach.md`** —
  incident-runbook with compound concern.

#### Spec name

- **`/specs/conglomerate-grant-model.json`** — `<concern>-<model>.json`
  per JSON-spec-naming convention; concern is `conglomerate-grant`;
  artifact is `model`.

### §H.3 Cross-back-pointer follow-up flags

The following parallel ADRs need amendment to add cross-back-pointers
to this ADR. Each is filed as a follow-up:

- **ADR-0244 §D-3** — needs amendment to cross-reference
  `conglomerate_grants` as the source-of-truth table for
  `controls_tenants` / `controlled_by_tenants` denorm columns;
  also needs amendment to note that `can_facilitate_sub_merchants`
  is now the binary form of the §D-3.7 scope tier; also needs the
  ADR-0244 §D-3 schema diagram updated to show the two new columns.
  *Follow-up filed as: F-ADR-0244-conglomerate-grant-back-reference.*
- **ADR-0311 §D-5** — needs cross-reference to §D-4 invariant 3
  (personal-tenant boundary preserved across conglomerate
  parent/child by construction); needs amendment to note that the
  `ConglomeratePersonalTenantBoundaryRefused` audit-event triple-
  seals to the personal-tenant audit-stream.
  *Follow-up filed as: F-ADR-0311-conglomerate-boundary-preservation-
  back-reference.*
- **ADR-0312 §D-3** — needs cross-reference to §D-5.5 bankruptcy-
  receivership scenario showing how court-warrant-scoped piercing
  interacts with the receiver's grant; needs amendment to clarify
  that the receiver's grant does NOT auto-cascade to the parent's
  prior controlling-entity scope (the §D-4 invariant 1 no-
  transitive-auto-include applies).
  *Follow-up filed as: F-ADR-0312-conglomerate-bankruptcy-back-
  reference.*
- **ADR-0263 §D-3** — needs the six new audit-event classes added
  to the canonical event-class registry; the dual-seal write API
  needs documentation per §E.2.3.
  *Follow-up filed as: F-ADR-0263-conglomerate-audit-event-registry-
  extension.*
- **ADR-0249 §D-4** — needs cross-reference to §D-3.7
  payment-facilitation scope tier; marketplace seller relationships
  per ADR-0249 should explicitly reference this ADR as the substrate
  primitive.
  *Follow-up filed as: F-ADR-0249-marketplace-payment-facilitation-
  back-reference.*
- **ADR-0299 §D-3** — needs cross-reference to §D-4 invariant 2;
  the cross-pack residency-preservation invariant applies at the
  conglomerate-grant evaluation boundary.
  *Follow-up filed as: F-ADR-0299-conglomerate-residency-preservation-
  back-reference.*
- **ADR-0304 §D-2** — needs cross-reference to §D-4 invariant 2 and
  to §D-10.4 `ConglomerateCrossJurisdictionResidencyEnforced` event;
  the cross-jurisdiction conflict resolution applies inside the
  conglomerate-grant evaluator.
  *Follow-up filed as: F-ADR-0304-conglomerate-cross-jurisdiction-
  back-reference.*
