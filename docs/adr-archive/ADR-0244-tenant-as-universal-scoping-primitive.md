---
id: ADR-0244
status: Superseded
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-privacy
  - council-security
  - ops-compliance
  - ops-sre-reliability
  - axis-tenancy
  - axis-identity
  - axis-audit-chain
  - axis-finops
supersedes: []
amends:
  - ADR-0220-consumer-intelligence-substrate.md (Audience-as-µservice framing replaced by tenant.audience_type)
  - ADR-0239-amendment-intelligence-internal-scope-clarification-2026-05-18.md (Internal/Consumer µservice split replaced by tenant scoping)
  - ADR-0221-agentic-development-pipeline-hardening.md (§M-04 manifest `audience` field removed; replaced by tenant.audience_type)
superseded_by: [ADR-702]
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-finops-sustainability-tagging.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0215-multi-context-platform.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0292-minor-user-doctrine.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0298-emergency-services-bypass-life-safety.md
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md
  - ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md
  - ADR-0317-role-based-projection-unified-ux-shell.md
  - ADR-0319-front-middle-back-office-information-barrier.md
  - ADR-0321-b2b-saas-industry-leader-coverage.md
related_adrs:
  - ADR-0297
  - ADR-0311
  - ADR-0313
  - ADR-0314
  - ADR-0315
  - ADR-0316
  - ADR-0317
  - ADR-0318
  - ADR-0319
  - ADR-0321
related_specs:
  - /specs/platform-architecture.json
  - /specs/tenant-model.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/identity.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/microservice-manifest-schema.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_bominal_inheritance_precedence
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_flat_product_catalog
  - feedback_canonical_base_localization
  - feedback_no_silent_regression
  - feedback_automate_everything
  - feedback_clean_architecture_requirements
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 3-of-14
purpose: >
  Establish tenant ID + dotted hierarchical sub-scope as the universal
  scoping primitive for every routing, authorization, attribution,
  retention, residency, audit-stream, cost-center, isolation,
  encryption, and compliance decision in the oyatie platform. Replace
  the audience-as-µservice-scope framing inherited from ADR-0220,
  ADR-0239, and ADR-0221 §M-04 with a uniform tenant model where
  audience is a property of the tenant, never of the µservice.
enforcement_status: advisory-until-tenant-substrate-lands
enforced_by:
  - oya gate validate tenant-id-format
  - oya gate validate sub-scope-depth
  - oya gate validate no-audience-on-microservice
  - oya gate validate tenant-schema-coherence
  - oya gate validate reserved-namespace-protection
  - oya gate validate cross-tenant-permit-coverage
---

# ADR-0244: Tenant as Universal Scoping Primitive

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive) landing as a single multispectrum-reviewed PR. Each
keystone references the others; partial acceptance is rejected because
the doctrines are mutually-reinforcing. This is keystone #3 of 14.

Enforcement is `advisory-until-tenant-substrate-lands`: the doctrine is
accepted in text now, but the CI lanes that enforce it move to BLOCKER
status only after:

1. `microservices/tenancy/` admits the canonical `tenants` table schema
   via migration `0002_canonical_tenant_schema.sql` (built atop the
   `0001_create_self_tenant.sql` bootstrap migration from ADR-0242).
2. Cedar entity-types for `Tenant`, `SubScope`, `Principal`, and
   `CrossTenantGrant` are loaded into `microservices/policy-engine/`
   (per ADR-0243 + ADR-0246).
3. Every µservice manifest is rewritten to remove the legacy
   `audience` field (per ADR-0221 §M-04) and gain the new fields
   defined in §D-5.
4. `microservices/audit-chain/` provisions per-sub-scope rollup views
   keyed on `(tenant_id, sub_scope_path)`.

Until those four items land, validators emit findings without failing
CI. Post-substrate, the lanes promote to BLOCKER.

## Date

2026-05-20.

## Context

### Prior portfolio state

The pre-keystone portfolio scoped decisions along three orthogonal
axes that frequently conflated:

1. **µservice audience** (per ADR-0220, ADR-0239, ADR-0221 §M-04):
   each µservice declared `audience: INTERNAL | B2B-tenant |
   B2C-consumer | DEVELOPER`. Routing, policy, and audit decisions
   then keyed on the µservice's declared audience.
2. **Tenant identity** (per ADR-0009, ADR-0218): tenants existed as
   first-class entities for customer-facing operations but were
   excluded from internal-tool decisions.
3. **Cell residency** (per ADR-0009, ADR-0049, ADR-0240): cells were
   tagged by region + pack but tenant→cell binding was per-µservice
   ad-hoc.

This three-axis model produced 23+ distinct policy-class decisions
authored as imperative code, static configuration, or implicit
convention. Each decision was a drift surface. The §Context inventory
in ADR-0243 enumerates the policy-class decisions; this ADR enumerates
the *scoping* decisions that should also have used a uniform primitive
but did not:

| Scoping decision currently in code or static config | Why it should be tenant-scoped |
|---|---|
| **Provider routing in Intelligence** (which LLM provider per data class) | The eligibility set is tenant-specific (provider_credential_mode, compliance pack, jurisdiction) |
| **Cell binding** (which home_cell + dr_cell for a workload) | Tenant determines residency + DR pairing |
| **Audit stream selection** | Tenant determines stream; sub-scope determines sub-stream |
| **Cost attribution** | Tenant + sub-scope determine cost center; rollup follows sub-scope tree |
| **Feature activation** | Tenant tier + compliance packs + sub-scope determine eligibility |
| **Rate limit + quota tier** | Tenant tier + per-sub-scope budget determine quota |
| **Cross-cell traffic permits** | Tenant-pair + cell-pair determine permitted call surface |
| **encryption-BYOK eligibility** | Tenant-level KMS root determines key envelope |
| **Webhook subscription eligibility** | Tenant determines event-class subscription |
| **Retention sunset** | Tenant jurisdiction + sub-scope retention class determine sunset |
| **DSAR cascade scope** | Tenant + sub-scope determine cascade target set |
| **Schema activation gating** | Per-tenant pinning during drain phase |
| **Cross-tenant collaboration permits** | Tenant pair + sub-scope determine sharing surface |
| **Partner-tenant on-behalf-of** | Agency tenant assumes role under customer tenant; sub-scope of customer |
| **Reserved-namespace registration refusal** | Tenant ID format determines refusal (per ADR-0242) |
| **Bulk import/export eligibility** | Tenant data-portability policy determines surface |
| **Plugin Wasmtime capability allowlist** | Tenant Cedar fragment determines plugin capabilities |
| **Sandbox tenant lifetime + budget** | Per-engineer sandbox tenant has scoped lifetime + budget |
| **Preview tenant lifecycle** | Per-PR preview tenant lifecycle bound to PR state |
| **Compliance pack activation** | Tenant adopts packs; pack activation per sub-scope possible |
| **Marketplace surface eligibility** | Tenant + sub-scope determine surface visibility |
| **Encryption key envelope** | Tenant KMS root + sub-scope sub-keys |
| **Observability dashboard scoping** | Tenant + sub-scope filter on every metric label |

That is 23 distinct scoping decisions. Each currently uses some ad-hoc
combination of µservice-audience, tenant-id, cell-id, and team-id. The
result is recurring drift: when a new µservice or feature ships, the
contributor invents a scoping pattern that is *almost* like a prior
pattern but subtly different. Audit-chain emissions, cost-attribution
queries, DSAR cascades, and Cedar fragments all carry the drift.

### Why audience-as-µservice-scope must be dropped

The ADR-0220 / ADR-0239 framing — Foundry is internal-only, Intelligence
is consumer-facing — collapsed under its own weight within twelve days
of being authored. Per ADR-0242 keystone #1 §Context, the framing
produced six distinct failure modes (doubled doctrine surface;
audit-chain stream drift; bypass-path temptation; compliance carve-out
fragility; engineering velocity drag; audit-chain leakage via shared
substrate).

The root cause is that **audience is a property of the *caller*, not
of the *callee***. A µservice that serves both internal CI workflows
and external customer API traffic does not have *an* audience; it has
*every* audience its callers carry. The audience question is answered
by inspecting the principal at the request boundary, not by inspecting
the µservice's manifest declaration.

Naming the property `audience` on a µservice — a callee — is therefore
a category error. It must be retired in favour of putting `audience`
on the tenant (a caller-side property): every principal acts under a
tenant; the tenant declares `audience_type` (per §D-11); every
µservice serves all tenants and resolves audience-aware behaviour
via Cedar policy evaluation against the calling tenant's attributes.

### How `oyatie-is-a-tenant` (ADR-0242) cascades

ADR-0242 establishes `oyatie` as a first-class tenant of its own
platform. The doctrine implicitly requires a uniform tenant model
because otherwise `oyatie` would still need a special path. This
keystone provides the uniform tenant model that ADR-0242 assumes:

- The tenant slug format (D-1) accommodates both `oyatie` and customer
  tenant IDs under one regex.
- The dotted hierarchical sub-scope convention (D-2) accommodates
  `oyatie.foundry.ci-agent` and `tenant-acme.engineering.api-client`
  under one syntax.
- The tenant table schema (D-3) holds the same column set whether the
  row is `oyatie` or `tenant-acme`.
- The Cedar entity-types (D-4) reference `Tenant::"oyatie"` and
  `Tenant::"tenant-acme"` interchangeably.
- The lifecycle (D-7) provisions, suspends, off-boards, and deletes
  the `oyatie` tenant by the same machinery as any other tenant
  (with the caveat that `oyatie` is `locked: true` per ADR-0242 §D-7;
  deletion requires ops-compliance approval).

### Hyperscaler precedent

Every named hyperscaler operates with a uniform tenant primitive at
the substrate layer + per-tenant attribute resolution at the policy
layer:

- **AWS IAM principal path.** AWS principals carry a path
  (`arn:aws:iam::<account-id>:role/<path>/<role-name>`). The path is
  hierarchical, dotted-equivalent (slash-delimited), and supports
  inheritance + rollup. AWS Organizations layers OU → Account → Role
  → Path → Session. The audience question (internal-AWS vs external-
  customer) is answered by the path, not by the service. (Source:
  AWS Identity and Access Management User Guide 2024 edition; AWS
  re:Invent 2024 IAM-403 session "Hierarchical permissions at scale.")
- **GCP IAM resource hierarchy.** Google Cloud Resource Manager
  defines an explicit four-level hierarchy (Organization → Folder →
  Project → Resource). Bindings inherit downward; deny policies
  override at any level. Internal Google teams + Google Cloud
  customers share the same primitive. (Source: Google Cloud
  Documentation "Resource hierarchy" 2024; Google CRE Book ch. 8.)
- **Azure AAD tenant model.** Azure Active Directory tenants are the
  universal scoping primitive across Microsoft 365, Azure, GitHub
  Enterprise, LinkedIn Talent. Microsoft IT operates as one Azure
  AAD tenant; customer organizations as others. Cross-tenant scenarios
  use Azure AD B2B Collaboration (guest accounts) or B2C (consumer
  identity). (Source: Microsoft Build 2024 keynote; Azure AAD docs
  2024-2025 "Tenancy in Azure Active Directory.")
- **Stripe account hierarchy.** Stripe platforms own
  connected accounts; each connected account is a tenant of the
  platform. The hierarchy is one level (platform → connected account)
  but the model is the same: connected accounts inherit defaults from
  the platform, can override, and route capability checks (payouts,
  KYC, payments) per connected-account attributes. (Source: Stripe
  Engineering Blog 2024 "Designing for global platforms";
  Stripe API Reference 2025 "Accounts" section.)
- **Cloudflare account → zone hierarchy.** Cloudflare accounts own
  zones, sub-zones, and DNS records. Account-level settings cascade.
  Cloudflare's own DNS infrastructure is a Cloudflare account.
  (Source: Cloudflare 2024 blog "Building on our own platform";
  Cloudflare API documentation 2024.)
- **Apple Developer team hierarchy.** Apple Developer accounts have
  team membership, role-based access, and per-app capability scoping.
  Apple's own engineering teams use the same team primitive that
  external developers use. (Source: Apple Developer Documentation
  2024; Apple WWDC 2024 "Managing your team in App Store Connect.")

The pattern is **uniform tenant primitive at the substrate; per-tenant
attribute resolution at the policy layer**. This keystone adopts that
pattern verbatim.

### What changes; what stays

**Changes:**

- The `audience` field is removed from every µservice manifest. The
  CI lane that validated audience-on-µservice is retired.
- A new `audience_type` column is added to the `tenants` table. Each
  tenant declares its audience type (see §D-11 enum).
- The 23 scoping decisions enumerated above route through tenant +
  sub-scope evaluation under Cedar (per ADR-0243).
- µservices that previously declared `audience: INTERNAL` now serve
  all tenants. `oyatie.*` principals retain heaviest use; customer
  tenants gain access subject to Cedar permits.

**Stays:**

- ADR-0009 cell architecture (per-tenant per-region) — unchanged.
  Tenants still bind to home_cell + dr_cell.
- ADR-0010 regional pack architecture — unchanged. Tenants still
  declare jurisdiction + sovereign-cloud pack.
- ADR-0145 inter-µservice communication reform (direct gRPC + 3
  invariants) — unchanged. Tenant context now travels in every gRPC
  metadata as a baseline invariant.
- ADR-0150 Cedar policy engine — unchanged in form; expanded in
  scope per ADR-0243.

## Decision

The platform adopts **tenant ID + dotted hierarchical sub-scope** as
the universal scoping primitive. The following twelve decisions are
locked.

### D-1. Tenant ID format

Every tenant has a globally unique slug ID conforming to:

- **Character set.** Lowercase ASCII letters `a-z`, digits `0-9`,
  hyphen `-`, dot `.`.
- **Anchored regex.** `^[a-z][a-z0-9-]{0,62}(\.[a-z0-9-]{1,62}){0,4}$`
  — first segment must begin with a letter; total length capped at
  319 characters (RFC 1035 DNS label compatible per-segment; up to 5
  segments separated by dots).
- **Segment rules.** Each dotted segment is 1-63 characters; hyphens
  may not begin or end a segment; consecutive hyphens are permitted
  but discouraged.
- **Reserved roots.** Per ADR-0242 §D-1 + §D-6, the slugs `oyatie`,
  `oya`, `oyat`, `oyati`, and any `oyatie-*` / `oya-*` / `oyat*` /
  `oyati*` prefix are reserved at platform genesis. Registration of
  any reserved slug is forbidden by the `reserved-tenant-namespace`
  Cedar fragment.
- **Unicode normalisation.** Submitted tenant IDs are normalised via
  NFKC + lowercase + diacritic-strip + Unicode-confusable replacement
  (per Unicode Technical Standard #39) before regex validation. The
  normalised form is what is stored.
- **IDN-homograph defence.** TR#36 confusables table is consulted at
  admission. Any normalisation that collapses to a reserved root is
  refused.
- **Customer-tenant prefix convention.** Customer tenants registered
  via self-service onboarding receive the `tenant-` prefix (e.g.,
  `tenant-acme-corp`). Enterprise tenants negotiated via sales may
  receive non-prefixed slugs subject to reserved-namespace check.
  Partner and reseller tenants receive `partner-` or `reseller-`
  prefix.
- **Case-fold uniqueness.** Two tenant IDs are duplicate if their
  NFKC-normalised lowercase forms are equal.

**Tenant ID format examples (valid):**

```
oyatie                          # platform owner; reserved
oyatie.foundry                  # sub-scope; not a tenant ID itself
tenant-acme-corp                # B2B customer
tenant-acme-corp.eu             # explicit EU subsidiary tenant
partner-bigfour-consulting      # partner agency tenant
reseller-emea-reseller-7        # reseller tenant
b2c-7f3a9c2e                    # B2C user (hash-suffix; per privacy doctrine)
b2c-user.<opaque-id>            # B2C user; opaque slug; one segment only
```

**Tenant ID format examples (invalid):**

```
Acme-Corp                       # uppercase
1-acme                          # starts with digit
acme--corp                      # not invalid by regex but discouraged
-acme                           # starts with hyphen
.acme                           # starts with dot
acme.                           # ends with dot
oyatie-corp                     # reserved prefix
oyаtie                          # Cyrillic 'а' confusable → normalises to "oyatie" → reserved
tenant.acme.corp.eu.subsidiary.us  # 6 segments; exceeds depth 5
```

### D-2. Dotted hierarchical sub-scope convention

Within a tenant, principals and resources address themselves by
**sub-scope path**:

- **Syntax.** `<tenant_id>.<sub-scope-segment-1>.<sub-scope-segment-2>.…`
- **Depth.** Maximum 5 sub-scope segments beyond the tenant ID
  (total 5 levels including tenant root; 6 segments total including
  the tenant ID itself). Deeper paths are refused at issuance.
- **Segment rules.** Each sub-scope segment matches one of two forms:
  - **Platform-reserved segments:** `^_[a-z0-9-]{0,61}$` — starts
    with `_`; only segments in the reserved-segment list below are
    valid; tenant admins MAY NOT author `_`-prefixed segments.
  - **Tenant-authored segments:** `^[a-z0-9][a-z0-9-]{0,62}$` —
    starts with alphanumeric, 1-63 chars; underscore NOT permitted as
    a leading character for tenant-authored segments.

  The combined regex that covers both forms is:
  `^((_[a-z0-9-]{0,61})|([a-z0-9][a-z0-9-]{0,62}))$`

  The `tenant_sub_scopes.sub_scope_path` CHECK constraint MUST use
  this combined regex. The admission-gate Rust code and the Cedar
  `SubScope` entity-type `sub_scope_path` attribute apply the same
  pattern. A separate admission-level guard ensures tenant admins
  cannot supply segments matching `^_` (reserved-platform-only).

  **Regex validation examples:**

  | Segment | Valid? | Reason |
  |---|---|---|
  | `foundry` | Yes | Tenant-authored: `[a-z0-9][a-z0-9-]*` |
  | `ci-agent` | Yes | Tenant-authored: hyphen in body |
  | `_system` | Yes | Platform-reserved: in reserved list |
  | `_audit` | Yes | Platform-reserved: in reserved list |
  | `_custom` | No | Starts with `_` but NOT in reserved list |
  | `-bad` | No | Starts with hyphen |
  | `BAD` | No | Uppercase |

- **Inheritance.** Default inheritance from parent: jurisdiction,
  home_cell, dr_cell, audience_type, capability flags, retention
  class, audit-stream root, FinOps cost center root, Cedar baseline
  policy.
- **Override.** Sub-scope may override an inherited attribute via
  explicit declaration in the tenant_sub_scopes table (D-3). Override
  is restrictive-only by default: a sub-scope may *narrow* the parent
  attribute (smaller capability flag set, shorter retention, narrower
  jurisdiction) but cannot *broaden* it.
- **Rollup semantics.** Audit emissions, cost attribution, observability
  metrics, and DSAR cascades roll up the sub-scope tree by default:
  events on `oyatie.foundry.ci-agent` appear in
  `oyatie.foundry`'s rollup view, which appears in `oyatie`'s rollup
  view. Rollup is non-destructive (per-level views are queryable).
- **Authorisation.** A principal carries exactly one sub-scope at a
  time (its identity scope). Acting under a different sub-scope of
  the same tenant requires an assume-role flow (D-6). Acting under
  a different tenant requires cross-tenant authorisation (D-6).
- **Cedar glob syntax.** Cedar fragments may apply to a sub-scope
  family via the suffix `.*` (e.g., `oyatie.foundry.*` matches every
  principal whose sub-scope path is `oyatie.foundry` or any descendant).
  Glob applies only at sub-scope segment boundaries.
- **Maximum number of distinct sub-scopes per tenant.** Soft limit
  10,000; hard limit 100,000. Beyond hard limit, the tenant must
  shard into multiple tenants (a partner-tenant cluster) or its sub-
  scopes must compress (e.g., reuse a hashed pool of principal-
  resource IDs rather than authoring per-principal sub-scopes).
- **Reserved sub-scope segments.** Within any tenant, the following
  sub-scope segments are reserved for platform use and cannot be
  authored by tenant admins: `_system`, `_bootstrap`, `_migration`,
  `_audit`, `_observability`, `_policy`, `_admin`, `_root`. Tenant
  admins may author any non-reserved segment.

### D-3. Tenant table schema (Postgres DDL)

The canonical `tenants` table lives in `microservices/tenancy/`,
Postgres 16+ via Citus shard on `(tenant_id)`. Migration
`0002_canonical_tenant_schema.sql`:

```sql
-- microservices/tenancy/migrations/0002_canonical_tenant_schema.sql
-- Canonical tenant schema per ADR-0244 §D-3.
-- Lands after 0001_create_self_tenant.sql (ADR-0242 bootstrap).

CREATE TYPE audience_type AS ENUM (
    -- Base set (foundational; cited by ADR-0220 / ADR-0221 retirements):
    'PLATFORM_OWNER',
    'B2B_TENANT',
    'B2C_CONSUMER',
    'DEVELOPER',
    'SANDBOX',
    'PREVIEW',
    'PARTNER_AGENCY',
    'RESELLER',
    -- Wave-3-G extensions (cited by ADR-0292 / ADR-0297..ADR-0306 / ADR-0311..ADR-0321):
    'B2C_FAMILY_PARENT',         -- Parent/guardian principal acting on minor (per ADR-0292 minor-user doctrine)
    'B2C_JOB_SEEKER_ACTIVE',     -- Cross-tenant job-seeking principal (per ADR-0317 role-projection persona profile)
    'B2B_HR_ADMIN',              -- B2B employer HR administrator scope (per ADR-0311 dual-tenant; ADR-0321 HCM dossiers)
    'B2B_INTERNAL_AUDIT',        -- B2B internal-audit organisational unit (per ADR-0313 conglomerate roll-up; ADR-0319 MIDDLE office)
    'INTERNAL_AUDITOR_3PAO',     -- External 3PAO/CPA/QA-firm tenant performing FedRAMP/SOC2/ISO audits (per ADR-0251 §D-10 + ADR-0319 MIDDLE clearance)
    'INTERNAL_DEV_TOOLS',        -- oyatie-internal dev/CI surfaces under dev-tools cells (per ADR-0247 + ADR-0297 §D-6.3)
    'EMERGENCY_SERVICES',        -- Life-safety bypass principal (per ADR-0298 emergency-services-bypass-life-safety)
    'FRIENDLY_CRAWLER_PARTNER',  -- Registered search/index/research crawler (per ADR-0297 §D-6.3 anti-bot exemptions)
    'MINOR_TARGETED'             -- Tenant whose surfaces serve minors (per ADR-0292 minor-user doctrine + ADR-0297 KOSA tier)
);

CREATE TYPE merchant_status AS ENUM (
    'NONE',
    'PENDING_KYB',
    'KYB_APPROVED',
    'PAYMENT_FACILITATOR',
    'PLATFORM_FACILITATOR',
    'SUSPENDED',
    'TERMINATED'
);

CREATE TYPE payout_method AS ENUM (
    'INTERNAL',
    'BANK_TRANSFER',
    'CARD_PAYOUT',
    'WALLET_PAYOUT',
    'CHECK',
    'NOT_APPLICABLE'
);

CREATE TYPE tenant_lifecycle_state AS ENUM (
    'PROVISIONING',
    'ACTIVE',
    'SUSPENDED',
    'OFFBOARDING',
    'SOFT_DELETED',
    'HARD_DELETED'
);

CREATE TYPE dr_pair_strategy AS ENUM (
    'ACTIVE_ACTIVE',
    'ACTIVE_PASSIVE',
    'PILOT_LIGHT',
    'BACKUP_RESTORE',
    'COLD_STANDBY',
    'NONE'
);

CREATE TYPE bootstrap_tier AS ENUM (
    'TIER_0_HARDWARE',
    'TIER_1_BOOTSTRAP_CELL',
    'TIER_2_CONTROL_PLANE',
    'TIER_3_DATA_PLANE',
    'TIER_4_TENANT_DATA',
    'NOT_APPLICABLE'
);

CREATE TYPE provider_credential_mode_t AS ENUM (
    'platform_default',         -- oyatie owns default LLM/provider credentials; B2C personal-use default
    'byok',                     -- tenant brings own LLM/provider API key or subscription (opt-in)
    'byok_required_by_pack'     -- compliance pack forces provider-BYOK (HIPAA/PCI/FedRAMP/IL5-6/KR-FSS/EU-AI-Act high-risk)
);

-- Note: This enum covers LLM/provider API credentials only (ADR-0255 §D-4).
-- encryption-BYOK (tenant-supplied KMS root / HSM partition) is a separate
-- concern tracked by the `byok_enabled` BOOLEAN column above and lives under
-- the encryption substrate (ADR-0251 §D-10 + cloud-secrets µservice).

-- Library-first policy evaluation mode per ADR-0246-amendment + ADR-0257-amendment.
CREATE TYPE policy_evaluation_mode_t AS ENUM (
    'library_first',                        -- in-process library; network on miss (default)
    'network_only',                         -- every evaluation is a network call
    'library_first_with_attested_fallback'  -- library_first; fallback requires meta-trust-root attestation
);

-- Library-first ontology read mode per ADR-0257-amendment.
CREATE TYPE ontology_read_mode_t AS ENUM (
    'library_first',                          -- in-process library cache; network on miss (default)
    'network_only',                           -- every read is a network call
    'library_first_with_freshness_floor'      -- library_first; cached values past freshness_floor trigger refresh
);
-- encryption-BYOK (tenant-supplied KMS root / HSM partition) is a separate
-- concern tracked by the `byok_enabled` BOOLEAN column above and lives under
-- the encryption substrate (ADR-0251 §D-10 + cloud-secrets µservice).

CREATE TABLE tenants (
    -- Identity
    tenant_id              TEXT        PRIMARY KEY
                                       CHECK (tenant_id ~ '^[a-z][a-z0-9-]{0,62}(\.[a-z0-9-]{1,62}){0,4}$'),
                                       -- Canonical slug; format per ADR-0244 §D-1.

    normalized_id          TEXT        NOT NULL UNIQUE,
                                       -- NFKC + lowercase + diacritic-strip + confusable-collapse form;
                                       -- used for reserved-namespace check; see microservices/tenancy/src/reserved_namespace.rs.

    display_name           TEXT        NOT NULL
                                       CHECK (char_length(display_name) BETWEEN 1 AND 256),
                                       -- Human-readable name shown in UIs; not unique.

    -- Audience classification (replaces ADR-0221 §M-04 manifest audience)
    audience_type          audience_type NOT NULL,
                                       -- Drives Cedar resolution; see §D-11 for enum semantics.

    -- Lifecycle
    lifecycle_state        tenant_lifecycle_state NOT NULL DEFAULT 'PROVISIONING',
                                       -- Active iff state = ACTIVE; see §D-7.

    locked                 BOOLEAN     NOT NULL DEFAULT FALSE,
                                       -- If TRUE, deletion requires ops-compliance approval per ADR-0242 §D-7.

    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by             TEXT        NOT NULL,
                                       -- Principal that created this tenant (sub-scope path of creator).

    activated_at           TIMESTAMPTZ,
                                       -- NULL until lifecycle transitions PROVISIONING → ACTIVE.

    suspended_at           TIMESTAMPTZ,
                                       -- NULL unless SUSPENDED; reason captured in audit chain.

    offboarding_initiated_at TIMESTAMPTZ,
                                       -- NULL unless OFFBOARDING; 30/90-day grace per §D-7.

    soft_deleted_at        TIMESTAMPTZ,
                                       -- NULL unless SOFT_DELETED; entry into recovery window.

    hard_deleted_at        TIMESTAMPTZ,
                                       -- NULL unless HARD_DELETED; cascade complete; reserved for tombstone marker.

    -- Cell binding (per ADR-0009)
    home_cell              TEXT        NOT NULL
                                       CHECK (home_cell ~ '^[a-z][a-z0-9-]{0,62}$'),
                                       -- Primary cell hosting tenant data; see microservices/tenancy/ARCHITECTURE.md#cell-assignment and microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning.

    dr_cell                TEXT
                                       CHECK (dr_cell IS NULL OR dr_cell ~ '^[a-z][a-z0-9-]{0,62}$'),
                                       -- Paired DR cell (per ADR-0241); NULL for SANDBOX/PREVIEW tenants on T3/T4.

    dr_pair_strategy       dr_pair_strategy NOT NULL DEFAULT 'ACTIVE_PASSIVE',
                                       -- DR strategy per ADR-0241 portfolio.

    cellular_deployment_pattern TEXT   NOT NULL DEFAULT 'standard',
                                       -- Deployment pattern hint for control-plane: standard | dedicated | shared | edge.

    -- Jurisdiction + residency (per ADR-0240)
    jurisdiction_code      TEXT        NOT NULL
                                       CHECK (jurisdiction_code ~ '^[A-Z]{2}(-[A-Z]{1,3})?$'),
                                       -- ISO 3166-1 alpha-2 + optional subdivision (e.g., 'US-DE', 'EU', 'KR', 'JP').

    data_residency_allowed TEXT[]      NOT NULL DEFAULT ARRAY[]::TEXT[],
                                       -- Permitted residency regions; subset of ADR-0240 pack matrix.

    sovereign_cloud_pack   TEXT,
                                       -- Pack ID per ADR-0240; NULL → uses cell-default pack.

    -- Hierarchy (D-2)
    parent_tenant_id       TEXT        REFERENCES tenants(tenant_id) ON DELETE RESTRICT,
                                       -- NULL for root tenants; non-NULL for partner/subsidiary tenants.

    -- Capabilities (drive Cedar evaluation; see §D-4 entity-type)
    can_pay                BOOLEAN     NOT NULL DEFAULT FALSE,
                                       -- Tenant may initiate payments via cloud-billing µservice.

    can_receive            BOOLEAN     NOT NULL DEFAULT FALSE,
                                       -- Tenant may receive payouts (post-KYB).

    can_settle             BOOLEAN     NOT NULL DEFAULT FALSE,
                                       -- Tenant may settle to bank account / external rail.

    can_facilitate_sub_merchants BOOLEAN NOT NULL DEFAULT FALSE,
                                       -- Tenant may onboard sub-merchants (Stripe platform analogue).

    -- Payments + tax
    merchant_status        merchant_status NOT NULL DEFAULT 'NONE',
    payout_method          payout_method   NOT NULL DEFAULT 'NOT_APPLICABLE',
    tax_registrations      JSONB       NOT NULL DEFAULT '[]'::JSONB,
                                       -- Array of {jurisdiction, tax_id, tax_type, registered_at, expiry_at, evidence_uri}.

    -- Tier (replaces audience-tier-on-µservice)
    tier                   TEXT        NOT NULL DEFAULT 'standard'
                                       CHECK (tier IN ('free', 'standard', 'pro', 'enterprise',
                                                       'platform-internal', 'sandbox', 'preview')),
                                       -- Drives quota + feature gates via Cedar.

    -- Compliance + privacy
    dsar_contact           JSONB       NOT NULL DEFAULT '{}'::JSONB,
                                       -- {email, phone, legal_owner, response_sla_days, escalation_chain[]}.

    compliance_packs       TEXT[]      NOT NULL DEFAULT ARRAY[]::TEXT[],
                                       -- Active compliance packs (per ADR-0251); e.g., ['soc2-t2', 'hipaa-baa', 'gdpr-eu'].

    legal_holds            JSONB       NOT NULL DEFAULT '[]'::JSONB,
                                       -- Active legal-hold entries (suppresses retention sunset + DSAR erasure).

    -- Audit (per ADR-0028 + ADR-0242 §D-7)
    audit_streams          TEXT[]      NOT NULL DEFAULT ARRAY[]::TEXT[],
                                       -- Named audit-stream identifiers owned by this tenant.

    -- FinOps (per ADR-0174)
    finops_cost_center     TEXT        NOT NULL
                                       CHECK (finops_cost_center ~ '^[a-z][a-z0-9-]{0,127}$'),
                                       -- Root cost-center slug for this tenant's spend.

    -- Cross-tenant relationships
    primary_tenants        TEXT[]      NOT NULL DEFAULT ARRAY[]::TEXT[],
                                       -- For partner / reseller tenants: tenant IDs they primarily serve.

    serves_oyatie_internal_ops BOOLEAN NOT NULL DEFAULT FALSE,
                                       -- TRUE iff this tenant is a sub-scope of `oyatie` or operates oyatie-internal workloads.

    bootstrap_tier         bootstrap_tier NOT NULL DEFAULT 'NOT_APPLICABLE',
                                       -- For oyatie-internal sub-tenants; identifies bootstrap-sequence step from ADR-0242.

    -- Encryption + secrets
    kms_root_key_handle    TEXT,
                                       -- Tenant-scoped KMS root key handle in OpenBao (microservices/cloud-secrets/).
                                       -- NULL until provisioning step 4 completes.

    byok_enabled           BOOLEAN     NOT NULL DEFAULT FALSE,
                                       -- encryption-BYOK: tenant brings own KMS root / HSM partition
                                       -- for at-rest encryption per ADR-0251 §D-10. Distinct from
                                       -- `provider_credential_mode` below, which covers LLM/provider API keys.

    provider_credential_mode provider_credential_mode_t NOT NULL DEFAULT 'platform_default',
                                       -- Per ADR-0255 §D-4. LLM/provider API credentials only. Enum:
                                       --   'platform_default'     — oyatie-owned provider credentials (B2C default)
                                       --   'byok'                 — tenant brings own provider key/subscription (opt-in)
                                       --   'byok_required_by_pack'— at least one active compliance pack
                                       --                            forces provider-BYOK (HIPAA/PCI/FedRAMP/
                                       --                            IL5-6/KR-FSS/EU-AI-Act high-risk).
                                       -- Disjoint from `byok_enabled` above, which is encryption-BYOK.

    -- Library-first policy evaluation mode (per ADR-0246-amendment + ADR-0257-amendment)
    policy_evaluation_mode  policy_evaluation_mode_t NOT NULL DEFAULT 'library_first',
                                       -- Per ADR-0246-amendment §D-2. Controls how the tenant's Cedar policy
                                       -- evaluation reaches the policy-engine substrate:
                                       --   'library_first'                       — evaluator library evaluated
                                       --                                            in-process; network call only on
                                       --                                            cache miss (default; lowest latency)
                                       --   'network_only'                        — every evaluation is a network call
                                       --                                            to the cell-local policy-engine;
                                       --                                            use when in-process library cannot
                                       --                                            be updated quickly enough
                                       --   'library_first_with_attested_fallback'— library_first; on cache miss,
                                       --                                            fallback is a network call that
                                       --                                            MUST carry meta-trust-root
                                       --                                            attestation (per ADR-0293)

    attested_fallback_threshold INTERVAL NOT NULL DEFAULT '24 hours',
                                       -- Per ADR-0246-amendment §D-2. Only relevant when
                                       -- `policy_evaluation_mode = 'library_first_with_attested_fallback'`.
                                       -- Maximum age of a cached policy decision before the attested
                                       -- network fallback is mandatory. Prevents stale library-cached
                                       -- decisions from persisting beyond the compliance window.

    -- Library-first ontology read mode (per ADR-0257-amendment)
    ontology_read_mode      ontology_read_mode_t NOT NULL DEFAULT 'library_first',
                                       -- Per ADR-0257-amendment §D-2. Controls how the tenant's Ontology
                                       -- reads are dispatched:
                                       --   'library_first'                         — read from in-process
                                       --                                              library cache first;
                                       --                                              network call on miss
                                       --   'network_only'                           — every read is a network
                                       --                                              call to the Ontology µservice
                                       --   'library_first_with_freshness_floor'     — library_first; but rejects
                                       --                                              any cached value older than
                                       --                                              `freshness_floor`

    freshness_floor         INTERVAL    NOT NULL DEFAULT '5 seconds',
                                       -- Per ADR-0257-amendment §D-2. Only relevant when
                                       -- `ontology_read_mode = 'library_first_with_freshness_floor'`.
                                       -- Minimum data-freshness guarantee for library-cached Ontology reads.
                                       -- Cached values older than this trigger a network refresh before
                                       -- returning to the caller.

    -- Resource budget (sandboxes + previews + quotas)
    resource_budget        JSONB       NOT NULL DEFAULT '{}'::JSONB,
                                       -- {compute_cpu_max, memory_max, storage_max, llm_tokens_per_day, …}.

    -- Sandbox / preview metadata
    parent_engineer_id     TEXT,
                                       -- NON-NULL iff tenant_id matches `oyatie.dev.<engineer-id>` pattern.

    parent_pr_number       INT,
                                       -- NON-NULL iff tenant_id matches `oyatie.preview.<pr-number>` pattern.

    auto_teardown_at       TIMESTAMPTZ,
                                       -- Scheduled teardown time for ephemeral tenants; NULL for non-ephemeral.

    -- Migration / versioning
    schema_version         INT         NOT NULL DEFAULT 1,
                                       -- Bumped when tenant attributes get migrated.

    last_migrated_at       TIMESTAMPTZ,

    -- Audit trail on the row itself
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by             TEXT        NOT NULL DEFAULT 'system:bootstrap',
    revision               BIGINT      NOT NULL DEFAULT 1
);

-- Sub-scope catalog (sparse; only sub-scopes with overrides are stored)
CREATE TABLE tenant_sub_scopes (
    tenant_id              TEXT        NOT NULL REFERENCES tenants(tenant_id) ON DELETE CASCADE,
    sub_scope_path         TEXT        NOT NULL
                                       CHECK (sub_scope_path ~ '^((_[a-z0-9-]{0,61})|([a-z0-9][a-z0-9-]{0,62}))(\.(_[a-z0-9-]{0,61}|[a-z0-9][a-z0-9-]{0,62})){0,4}$'),
                                       -- Dotted path relative to tenant_id; max 5 segments per ADR-0244 §D-2.
                                       -- Each segment is either a platform-reserved segment (^_[a-z0-9-]{0,61}$)
                                       -- or a tenant-authored segment (^[a-z0-9][a-z0-9-]{0,62}$).
                                       -- Admission gate additionally rejects tenant-authored segments that start with '_'.

    -- Optional overrides (NULL → inherit from parent)
    home_cell              TEXT,
    dr_cell                TEXT,
    jurisdiction_code      TEXT,
    audience_type          audience_type,
    tier                   TEXT,
    audit_streams          TEXT[],
    finops_cost_center     TEXT,
    retention_class        TEXT,

    -- Sub-scope-specific resource budget (overrides tenant default)
    resource_budget        JSONB       NOT NULL DEFAULT '{}'::JSONB,

    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by             TEXT        NOT NULL,

    PRIMARY KEY (tenant_id, sub_scope_path)
);

CREATE INDEX idx_tenants_lifecycle_state    ON tenants (lifecycle_state);
CREATE INDEX idx_tenants_parent             ON tenants (parent_tenant_id) WHERE parent_tenant_id IS NOT NULL;
CREATE INDEX idx_tenants_home_cell          ON tenants (home_cell);
CREATE INDEX idx_tenants_jurisdiction       ON tenants (jurisdiction_code);
CREATE INDEX idx_tenants_audience_type      ON tenants (audience_type);
CREATE INDEX idx_tenants_serves_oyatie      ON tenants (serves_oyatie_internal_ops) WHERE serves_oyatie_internal_ops;
CREATE INDEX idx_tenants_auto_teardown      ON tenants (auto_teardown_at) WHERE auto_teardown_at IS NOT NULL;
CREATE INDEX idx_tenant_sub_scopes_path     ON tenant_sub_scopes (tenant_id, sub_scope_path);

-- Update-trigger (revision++ + updated_at + updated_by on every mutation)
CREATE OR REPLACE FUNCTION tenants_update_revision() RETURNS TRIGGER AS $$
BEGIN
    NEW.revision := OLD.revision + 1;
    NEW.updated_at := now();
    -- updated_by must be set by caller; ENV-default 'system:unknown' is a CI-detected smell.
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_tenants_update_revision
    BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION tenants_update_revision();

-- Citus shard distribution
SELECT create_distributed_table('tenants',           'tenant_id');
SELECT create_distributed_table('tenant_sub_scopes', 'tenant_id', colocate_with => 'tenants');
```

**Field documentation summary** (every field has a comment in the DDL
above; this table reinforces the policy framing):

| Field | Type | Nullable | Drives |
|---|---|---|---|
| `tenant_id` | TEXT PK | NO | Universal scoping primitive |
| `normalized_id` | TEXT UNIQUE | NO | Reserved-namespace + duplicate detection |
| `display_name` | TEXT | NO | UI label only |
| `audience_type` | enum | NO | Cedar audience resolution; replaces µservice manifest field |
| `lifecycle_state` | enum | NO | Per §D-7 state machine |
| `locked` | BOOL | NO | Hard deletion requires ops-compliance |
| `home_cell` | TEXT | NO | Cell binding per ADR-0009 |
| `dr_cell` | TEXT | YES | Paired DR cell per ADR-0241 |
| `dr_pair_strategy` | enum | NO | DR strategy |
| `cellular_deployment_pattern` | TEXT | NO | Control-plane hint |
| `jurisdiction_code` | TEXT | NO | ADR-0240 residency overlay |
| `data_residency_allowed` | TEXT[] | NO | Multi-region residency |
| `sovereign_cloud_pack` | TEXT | YES | Per-pack overlay |
| `parent_tenant_id` | TEXT FK | YES | Partner / subsidiary hierarchy |
| `can_pay` | BOOL | NO | Payments capability gate |
| `can_receive` | BOOL | NO | Payout capability gate |
| `can_settle` | BOOL | NO | Settlement capability gate |
| `can_facilitate_sub_merchants` | BOOL | NO | Stripe Connect-style facilitation |
| `merchant_status` | enum | NO | KYB + payments lifecycle |
| `payout_method` | enum | NO | Payout mechanism |
| `tax_registrations` | JSONB | NO | Per-jurisdiction tax nexus |
| `tier` | TEXT | NO | Quota + feature tier |
| `dsar_contact` | JSONB | NO | DSAR routing |
| `compliance_packs` | TEXT[] | NO | Active packs per ADR-0251 |
| `legal_holds` | JSONB | NO | Suppresses retention + DSAR erasure |
| `audit_streams` | TEXT[] | NO | Per ADR-0028 + ADR-0242 §D-7 |
| `finops_cost_center` | TEXT | NO | Per ADR-0174 |
| `primary_tenants` | TEXT[] | NO | Partner / reseller relationships |
| `serves_oyatie_internal_ops` | BOOL | NO | Replaces `audience: INTERNAL` on µservices |
| `bootstrap_tier` | enum | NO | Bootstrap sequence position |
| `kms_root_key_handle` | TEXT | YES | OpenBao handle |
| `byok_enabled` | BOOL | NO | encryption-BYOK (KMS / HSM root) per ADR-0251 §D-10 |
| `provider_credential_mode` | enum | NO | LLM/provider-BYOK per ADR-0255 §D-4: `platform_default` (B2C) / `byok` (opt-in) / `byok_required_by_pack` |
| `policy_evaluation_mode` | enum (`policy_evaluation_mode_t`) | NO | Library-first Cedar evaluation dispatch per ADR-0246-amendment: `library_first` (default) / `network_only` / `library_first_with_attested_fallback` (requires meta-trust-root witness) |
| `attested_fallback_threshold` | INTERVAL | NO | Max age of cached Cedar decision before attested network fallback is mandatory; relevant when `policy_evaluation_mode = 'library_first_with_attested_fallback'`; default 24 hours |
| `ontology_read_mode` | enum (`ontology_read_mode_t`) | NO | Library-first Ontology read dispatch per ADR-0257-amendment: `library_first` (default) / `network_only` / `library_first_with_freshness_floor` |
| `freshness_floor` | INTERVAL | NO | Minimum Ontology data freshness for library-cached reads; cached values older than this trigger a network refresh; default 5 seconds; relevant when `ontology_read_mode = 'library_first_with_freshness_floor'` |
| `resource_budget` | JSONB | NO | Quota envelope |
| `parent_engineer_id` | TEXT | YES | Sandbox tenant parent |
| `parent_pr_number` | INT | YES | Preview tenant parent |
| `auto_teardown_at` | TIMESTAMPTZ | YES | Ephemeral lifecycle |
| `schema_version` | INT | NO | Per-row schema evolution |
| `created_at` / `created_by` / `updated_at` / `updated_by` / `revision` | various | NO | Row audit trail |

### D-4. Cedar entity types for Tenant

`microservices/policy-engine/schemas/tenant.cedarschema` declares the
Cedar v4.2 entity-types referenced by every fragment that gates a
tenant-scoped action:

```cedar
// microservices/policy-engine/schemas/tenant.cedarschema
// Per ADR-0244 §D-4. Cedar v4.2 grammar.

namespace Tenancy {

    entity Tenant = {
        "tenant_id":                        String,
        "normalized_id":                    String,
        "audience_type":                    String,
        "lifecycle_state":                  String,
        "locked":                           Bool,
        "home_cell":                        String,
        "dr_cell":                          String,
        "dr_pair_strategy":                 String,
        "cellular_deployment_pattern":      String,
        "jurisdiction_code":                String,
        "data_residency_allowed":           Set<String>,
        "sovereign_cloud_pack":             String,
        "parent_tenant_id":                 String,
        "can_pay":                          Bool,
        "can_receive":                      Bool,
        "can_settle":                       Bool,
        "can_facilitate_sub_merchants":     Bool,
        "merchant_status":                  String,
        "payout_method":                    String,
        "tier":                             String,
        "compliance_packs":                 Set<String>,
        "audit_streams":                    Set<String>,
        "finops_cost_center":               String,
        "primary_tenants":                  Set<String>,
        "serves_oyatie_internal_ops":       Bool,
        "bootstrap_tier":                   String,
        "byok_enabled":                     Bool,
        "provider_credential_mode":         String,
        "schema_version":                   Long
    };

    entity SubScope in [Tenant] = {
        "tenant_id":                        String,
        "sub_scope_path":                   String,
        "home_cell":                        String,
        "dr_cell":                          String,
        "jurisdiction_code":                String,
        "audience_type":                    String,
        "tier":                             String,
        "audit_streams":                    Set<String>,
        "finops_cost_center":               String,
        "retention_class":                  String
    };

    entity Principal in [SubScope, Tenant] = {
        "principal_id":                     String,
        "principal_kind":                   String,    // human | service | agent | workflow | webhook
        "tenant_id":                        String,
        "sub_scope_path":                   String,
        "issued_at":                        Long,
        "expires_at":                       Long,
        "mfa_strength":                     String,    // none | totp | webauthn | hardware-key
        "consent_grants":                   Set<String>,
        "labels":                           Set<String>
    };

    entity CrossTenantGrant = {
        "grant_id":                         String,
        "from_tenant":                      String,
        "to_tenant":                        String,
        "from_sub_scope":                   String,
        "to_sub_scope":                     String,
        "grant_kind":                       String,    // assume_role | share | partner_obo | reseller_obo
        "actions_permitted":                Set<String>,
        "resources_permitted":              Set<String>,
        "issued_at":                        Long,
        "expires_at":                       Long,
        "revoked":                          Bool,
        "approver_principal":               String,
        "evidence_uri":                     String
    };

    entity Resource = {
        "resource_id":                      String,
        "resource_kind":                    String,
        "owner_tenant":                     String,
        "owner_sub_scope":                  String,
        "data_class":                       String,
        "labels":                           Set<String>
    };

    action "RegisterTenant"                 appliesTo { principal: [Principal], resource: [Tenant] };
    action "ActivateTenant"                 appliesTo { principal: [Principal], resource: [Tenant] };
    action "SuspendTenant"                  appliesTo { principal: [Principal], resource: [Tenant] };
    action "OffboardTenant"                 appliesTo { principal: [Principal], resource: [Tenant] };
    action "SoftDeleteTenant"               appliesTo { principal: [Principal], resource: [Tenant] };
    action "HardDeleteTenant"               appliesTo { principal: [Principal], resource: [Tenant] };
    action "RecoverTenant"                  appliesTo { principal: [Principal], resource: [Tenant] };

    action "ReadInScope"                    appliesTo { principal: [Principal], resource: [Resource] };
    action "WriteInScope"                   appliesTo { principal: [Principal], resource: [Resource] };
    action "DeleteInScope"                  appliesTo { principal: [Principal], resource: [Resource] };

    action "AssumeSubScope"                 appliesTo { principal: [Principal], resource: [SubScope] };
    action "AcrossTenantAct"                appliesTo { principal: [Principal], resource: [CrossTenantGrant] };

    action "IssueCrossTenantGrant"          appliesTo { principal: [Principal], resource: [Tenant] };
    action "RevokeCrossTenantGrant"         appliesTo { principal: [Principal], resource: [CrossTenantGrant] };
}
```

Baseline Cedar fragment for tenant-scope authorisation
(`microservices/policy-engine/fragments/baseline/tenant-scope-baseline.cedar`):

```cedar
// Baseline: principal may act on a resource owned by the same tenant
// + same sub-scope (or a descendant sub-scope). All other access
// requires explicit grant.

permit (
    principal,
    action in [Tenancy::Action::"ReadInScope",
               Tenancy::Action::"WriteInScope",
               Tenancy::Action::"DeleteInScope"],
    resource is Tenancy::Resource
)
when {
    principal.tenant_id == resource.owner_tenant
    && (principal.sub_scope_path == resource.owner_sub_scope
        || principal.sub_scope_path like (resource.owner_sub_scope + ".*"))
    && principal.expires_at > context.now
};

forbid (
    principal,
    action in [Tenancy::Action::"ReadInScope",
               Tenancy::Action::"WriteInScope",
               Tenancy::Action::"DeleteInScope"],
    resource is Tenancy::Resource
)
when {
    principal.tenant_id != resource.owner_tenant
    && !context.cross_tenant_grant_present
};
```

### D-5. Manifest schema changes (replacing ADR-0221 §M-04)

Every µservice manifest at `microservices/<name>/manifest.yaml` is
rewritten:

**Removed fields:**
- `audience` (was `INTERNAL | B2B-tenant | B2C-consumer | DEVELOPER`)

**Added fields:**
- `tier` (substrate tier; see ADR-0245)
- `primary_tenants` (which tenant audience_types this µservice
  primarily serves, used for capacity planning + observability
  labels only; NOT for policy gating)
- `serves_oyatie_internal_ops` (boolean; TRUE iff µservice carries
  oyatie-internal-only data classes)
- `cellular_deployment_pattern` (deployment pattern hint)
- `dr_pair_strategy` (per-µservice DR strategy per ADR-0241)
- `bootstrap_tier` (bootstrap-sequence position)

**Canonical JSON schema** (`/specs/microservice-manifest-schema.json`):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://oyatie/specs/microservice-manifest-schema.json",
  "title": "Microservice Manifest Schema",
  "type": "object",
  "required": [
    "name",
    "tier",
    "owners",
    "primary_tenants",
    "serves_oyatie_internal_ops",
    "cellular_deployment_pattern",
    "dr_pair_strategy",
    "bootstrap_tier",
    "data_classes",
    "actions",
    "calls"
  ],
  "additionalProperties": false,
  "properties": {
    "name": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9-]{2,62}$",
      "description": "µservice slug; matches directory under microservices/."
    },
    "tier": {
      "type": "string",
      "enum": ["substrate", "kernel", "product", "studio", "marketplace", "edge"],
      "description": "Substrate-vs-product position per ADR-0245."
    },
    "owners": {
      "type": "array",
      "items": { "type": "string" },
      "minItems": 1,
      "description": "Council / axis / ops team slugs responsible for this µservice."
    },
    "primary_tenants": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "PLATFORM_OWNER",
          "B2B_TENANT",
          "B2C_CONSUMER",
          "DEVELOPER",
          "SANDBOX",
          "PREVIEW",
          "PARTNER_AGENCY",
          "RESELLER"
        ]
      },
      "minItems": 1,
      "description": "Tenant audience_types this µservice primarily serves; used for capacity planning + observability labels ONLY. NOT a policy gate; policy gating is Cedar-only (per ADR-0243)."
    },
    "serves_oyatie_internal_ops": {
      "type": "boolean",
      "description": "TRUE iff this µservice carries oyatie-internal-only data classes (e.g., SOURCE_CODE_INTERNAL, CREDENTIAL_INTERNAL, EVAL_CORPUS_INTERNAL). Drives sovereign-cloud overlay + Cedar fragments that scope reads to oyatie.* principals. NOT a replacement for the old `audience` field; this is one tenant-property among many."
    },
    "cellular_deployment_pattern": {
      "type": "string",
      "enum": ["standard", "dedicated", "shared", "edge", "bootstrap"],
      "description": "Deployment pattern hint for control-plane: standard = one Deployment per cell; dedicated = one Deployment per tenant per cell; shared = one Deployment across multiple cells (rare; substrate-only); edge = deployed at edge cells; bootstrap = bootstrap-only (Tier 1 cell)."
    },
    "dr_pair_strategy": {
      "type": "string",
      "enum": ["ACTIVE_ACTIVE", "ACTIVE_PASSIVE", "PILOT_LIGHT", "BACKUP_RESTORE", "COLD_STANDBY", "NONE"],
      "description": "DR strategy per ADR-0241; defines RPO + RTO targets."
    },
    "bootstrap_tier": {
      "type": "string",
      "enum": ["TIER_0_HARDWARE", "TIER_1_BOOTSTRAP_CELL", "TIER_2_CONTROL_PLANE", "TIER_3_DATA_PLANE", "TIER_4_TENANT_DATA", "NOT_APPLICABLE"],
      "description": "Bootstrap-sequence position per ADR-0242 §D-5."
    },
    "data_classes": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Data classes this µservice handles; per ADR-0099 registry."
    },
    "actions": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "cedar_permit_fragment", "cedar_default_deny_fragment"],
        "properties": {
          "name": { "type": "string" },
          "cedar_permit_fragment": { "type": "string" },
          "cedar_default_deny_fragment": { "type": "string" }
        }
      },
      "description": "Actions declared by this µservice; every action requires Cedar permit + default-deny fragments per ADR-0243 §D-3."
    },
    "calls": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["to_microservice", "actions_called"],
        "properties": {
          "to_microservice": { "type": "string" },
          "actions_called": {
            "type": "array",
            "items": { "type": "string" }
          }
        }
      },
      "description": "Cross-µservice calls; per ADR-0145 + ADR-0243 §D-9."
    }
  }
}
```

**Migration of existing manifests:** Every existing manifest with an
`audience` field is rewritten in a single sweep ChangeSet:

| Legacy `audience` value | Rewrite |
|---|---|
| `INTERNAL` | `serves_oyatie_internal_ops: true`, `primary_tenants: [PLATFORM_OWNER, DEVELOPER]` |
| `B2B-tenant` | `serves_oyatie_internal_ops: false`, `primary_tenants: [B2B_TENANT, PARTNER_AGENCY, RESELLER]` |
| `B2C-consumer` | `serves_oyatie_internal_ops: false`, `primary_tenants: [B2C_CONSUMER]` |
| `DEVELOPER` | `serves_oyatie_internal_ops: false`, `primary_tenants: [DEVELOPER, B2B_TENANT]` |

Note that `serves_oyatie_internal_ops: true` does NOT prevent other
tenants from being served — it only means this µservice handles oyatie-
internal data classes. Per-tenant access is gated by Cedar, not by this
field.

### D-6. Cross-tenant operations

Four cross-tenant operation patterns, each backed by a `CrossTenantGrant`
entity (D-4) and Cedar permits:

**D-6.1 Assume-role (control-plane-mediated).** A principal in tenant
A's sub-scope `oyatie.security.incident-response` may, for the
duration of an incident, assume a role in tenant B's sub-scope
`tenant-acme.audit-readonly`. The grant is:

- Issued by the target tenant's admin via the admin console (Cedar
  `IssueCrossTenantGrant` permit), with `grant_kind: assume_role`.
- Time-bounded (`expires_at` mandatory; max 24h initial; max 7d with
  ops-compliance approval).
- Audit-emitted on issue, on each assumption, and on revocation.
- Reversible: revocation is immediate; in-flight grants survive only
  if the principal has cached an active session token (TTL 60s).

**D-6.2 Sharing (shadow projection across cells).** Tenant A may
share a specific Resource (e.g., a Workflow Studio template, a
catalog record, a directory entry) with tenant B. The mechanism:

- Source tenant's resource is projected as a *shadow* into target
  tenant's cell via the inter-cell projection service (per ADR-0049
  + ADR-0240 residency overlay).
- The shadow is read-only by default; write capability requires an
  additional Cedar `WriteShadow` permit.
- The shadow lifecycle binds to the source: if the source is
  deleted, the shadow is tombstoned.
- DSAR cascade traverses shadows: an erasure request on the source
  cascades to tombstone all shadows.
- Audit emissions on the shadow carry both `owner_tenant` (source)
  and `viewer_tenant` (shadow consumer); both audit streams receive
  the event.

**D-6.3 Partner-tenant on-behalf-of (OBO).** A partner agency
(`partner-bigfour-consulting`) acts on behalf of a customer tenant
(`tenant-acme`). The flow:

- Customer tenant admin pre-authorises the partner tenant via Cedar
  `IssueCrossTenantGrant` with `grant_kind: partner_obo`.
- The grant specifies `actions_permitted` (allowlist) and
  `resources_permitted` (scoped resource glob).
- Partner agency's principals (`partner-bigfour-consulting.*`) may
  obtain an OBO session token by presenting the grant ID + MFA.
- OBO session sees customer's data; every read/write emits to BOTH
  the partner audit stream and the customer audit stream.
- Customer admin may revoke at any time; in-flight sessions expire
  within 60s.

**D-6.4 Reseller on-behalf-of (OBO).** Identical mechanism to D-6.3
with `grant_kind: reseller_obo`. Reseller may manage their downstream
customer tenants' lifecycle (create / suspend / terminate) subject
to the grant scope.

**Cross-tenant Cedar permits** are required for every cross-tenant
operation; absence defaults to deny. Example permit fragment:

```cedar
// microservices/policy-engine/fragments/baseline/cross-tenant-baseline.cedar

permit (
    principal,
    action in [Tenancy::Action::"ReadInScope",
               Tenancy::Action::"WriteInScope"],
    resource is Tenancy::Resource
)
when {
    principal.tenant_id != resource.owner_tenant
    && context has "cross_tenant_grant_id"
    && context.cross_tenant_grant_active == true
    && context.cross_tenant_grant_expires_at > context.now
    && resource.resource_kind in context.cross_tenant_grant_resources_permitted
    && action.name in context.cross_tenant_grant_actions_permitted
};
```

### D-7. Tenant lifecycle

The `lifecycle_state` enum has six states forming a state machine:

```
                ┌───────────────┐
                │ PROVISIONING  │
                └───────┬───────┘
                        │ activate (Cedar permit + KYB if merchant)
                        ▼
                ┌───────────────┐         ┌──────────────┐
                │    ACTIVE     │◄────────┤  SUSPENDED   │
                └───┬───────┬───┘         └──────┬───────┘
        suspend     │       │ offboard           │
                    ▼       ▼                    │
            ┌──────────────────┐                 │
            │   SUSPENDED      │                 │ recover
            └────────┬─────────┘                 │
                     │ offboard                  ▼
                     │             ┌──────────────────┐
                     └────────────►│   OFFBOARDING    │
                                   └────────┬─────────┘
                                            │ 30d grace
                                            ▼
                                   ┌──────────────────┐
                                   │  SOFT_DELETED    │
                                   └────────┬─────────┘
                                            │ 90d window + DSAR cascade
                                            ▼
                                   ┌──────────────────┐
                                   │  HARD_DELETED    │
                                   └──────────────────┘
```

**Transitions and SLOs:**

| Transition | Trigger | SLO | Audit emission |
|---|---|---|---|
| `(none) → PROVISIONING` | Registration request | < 5s admission | `TenantProvisioning` |
| `PROVISIONING → ACTIVE` | Bootstrap steps complete (KMS root key, audit stream, Cedar permits, home_cell assignment) | < 5 min | `TenantActivated` |
| `ACTIVE → SUSPENDED` | Operator action (billing, abuse, compliance investigation) | < 1 min | `TenantSuspended` (with reason) |
| `SUSPENDED → ACTIVE` | Recover action; operator override | < 1 min | `TenantRecovered` |
| `ACTIVE → OFFBOARDING` | Tenant admin offboarding request OR operator decision | Immediate | `TenantOffboardingInitiated` |
| `OFFBOARDING → SOFT_DELETED` | 30-day grace expires (configurable per pack) | At grace expiry | `TenantSoftDeleted` |
| `SOFT_DELETED → ACTIVE` | Recovery request within 90-day window | < 5 min | `TenantRecovered` (recovery flag) |
| `SOFT_DELETED → HARD_DELETED` | 90-day window expires + DSAR cascade complete | At window expiry + cascade | `TenantHardDeleted` |

**DSAR Article 17 deletion (GDPR / KR PIPA Article 36 equivalent):**
A subject's right-to-erasure within an active tenant follows the
DSAR cascade as documented in ADR-0242 §Appendix B. Tenant-level
hard deletion (lifecycle transition to HARD_DELETED) executes the
same cascade for ALL principals + ALL resources owned by the tenant.

**Recovery from soft-delete.** During the SOFT_DELETED window (90
days post-OFFBOARDING grace), the tenant's data remains recoverable.
Recovery:

- Requires re-authentication of a tenant admin who was authorised
  before offboarding (or a higher-tier ops-compliance override).
- Restores `lifecycle_state` to ACTIVE; all sub-scopes restored.
- Audit-emits `TenantRecovered` event with reason.
- Legal holds active during the soft-delete window are preserved;
  retention sunsets that fired during the window are *not* undone
  (data already deleted under per-jurisdiction retention rules stays
  deleted; only data under hold or within retention is recoverable).

**Hard deletion** is irreversible. Triggered by:

- 90-day soft-delete window expiry, OR
- Explicit operator-initiated hard delete with ops-compliance
  approval (overrides `locked: true`).

The hard-delete cascade:

1. Cedar `HardDeleteTenant` permit evaluated; legal hold must NOT
   be active; ops-compliance approval recorded.
2. DSAR cascade enumerates every resource owned by the tenant.
3. Per-resource erasure plan (hard-delete / pseudonymise / tombstone)
   per the type of resource.
4. Cascade execution emits per-step audit rows.
5. Audit-chain entries themselves are NOT deleted (Merkle-sealed);
   tenant identifier is replaced with hash; original Merkle proof
   retained.
6. KMS root key destroyed (cryptographic erasure for any data not
   per-record erased).
7. `tenants` row updated to `HARD_DELETED` with `hard_deleted_at`
   timestamp; row retained as tombstone for tenant-ID-reservation
   purposes (prevents re-use of the slug for 7 years per regulatory
   alignment with FRCP 37(e) + relevant retention rules).

### D-8. Sandbox tenants

**Pattern.** `oyatie.dev.<engineer-id>` — per-engineer personal
sandbox tenant under the `oyatie` root.

**Provisioning.** Auto-provisioned on first engineer login to the
internal developer portal. Engineer ID derived from corporate SSO
subject (Zitadel claim).

**Lifecycle:**

- `auto_teardown_at = max(last_activity + 24h, last_activity + 7d if pinned)`.
- Engineer may pin the tenant to extend lifetime to 7 days idle.
- Beyond 7 days idle without pin, tenant moves to SOFT_DELETED with
  7-day recovery window (shorter than production 90 days).

**Resource budget** (enforced via Cedar `WriteInScope` permit
attribute checks):

| Resource | Limit |
|---|---|
| CPU | 2 vCPU max per cell-side workload |
| Memory | 4 GiB |
| Storage (Postgres + SeaweedFS) | 10 GiB |
| LLM tokens (Intelligence µservice) | 10,000 per day |
| Workflow runs (Workflow Engine) | 1,000 per day |
| Outbound network egress | 1 GiB per day |
| Audit-chain rows | 100,000 per day |

**Cell placement.** Sandbox tenants live in a dedicated
`bootstrap-sandbox` cell tier (deploys at the engineer's nearest
region pack; defaults to `us-west-2-sandbox` for North America,
`eu-central-1-sandbox` for EU, `ap-northeast-2-sandbox` for KR).

**Isolation.** Sandbox tenants share underlying substrate with
production but Cedar permits scope them to `bootstrap-sandbox` cells;
cross-cell traffic from sandbox to production is forbidden by Cedar
fragment `oyatie-sandbox-cross-cell-forbid.cedar`.

**Compliance.** Sandbox tenants ARE subject to Cedar gates, audit
emission, DSAR cascade, encryption — full uniformity per ADR-0242.
Sandbox tenants are NOT subject to SOC 2 / HIPAA / FedRAMP pack
overlays unless the engineer explicitly opts in for testing.

### D-9. Preview tenants

**Pattern.** `oyatie.preview.<pr-number>` — per-PR ephemeral preview
tenant.

**Provisioning.** Auto-provisioned by GitHub Actions / Foundry
workflow on PR open. Trigger: `pull_request` event of kind `opened`
or `reopened` against the `dev` branch.

**Cell placement.** Preview tenants deploy to a `preview-` cell
tier; preview cells share substrate but are isolated by Cedar +
network policy.

**Lifecycle:**

- Created on PR open.
- Refreshed on each commit to the PR branch (workflow re-runs
  bootstrap migrations against the preview cell).
- Auto-teardown on PR close (merge OR abandon) within 60 minutes.
- `auto_teardown_at` updated on every commit push to `commit_pushed_at + 7d`
  as a safety net.

**Resource budget** (smaller than sandbox; bounded to limit cost
per PR):

| Resource | Limit |
|---|---|
| CPU | 4 vCPU total |
| Memory | 8 GiB |
| Storage | 20 GiB |
| LLM tokens | 50,000 per PR lifetime |
| Workflow runs | 5,000 per PR lifetime |
| Audit-chain rows | 500,000 per PR lifetime |

**Tenant data.** Preview tenants start from a *frozen seed dataset*
(checked into `microservices/fixtures/preview-seed/`); they do not
copy production tenant data. Tests against preview tenants exercise
the PR'd code path; PII never lands in preview cells.

**Reviewer access.** PR reviewers (council, axis, ops, agent) may
log into the preview tenant via OIDC-mediated SSO. Reviewer access
is audit-emitted to both the reviewer's principal stream and the
preview tenant's stream.

**Cleanup.** Auto-teardown:

1. Cedar `HardDeleteTenant` permit evaluated under the workflow
   principal (`oyatie.foundry.preview-lifecycle`).
2. DSAR cascade executes within the preview cell (small surface;
   typically completes < 30s).
3. Cell-local Postgres rows + SeaweedFS objects deleted.
4. Tenant row moves to HARD_DELETED.
5. PR number is freed for reuse (slug `oyatie.preview.<pr-number>`
   may be re-provisioned for a future PR with the same number; rare
   given GitHub's monotonic PR numbering).

### D-10. Cross-cell tenant migration

A planned, rare operation: relocate a tenant's home_cell (e.g.,
because the original cell is being decommissioned, or the tenant's
jurisdiction has changed, or load-balancing requires re-binding).

**Preconditions:**

- Tenant is in lifecycle_state ACTIVE.
- Operator-initiated; requires ops-compliance approval recorded
  on the migration request.
- Migration window scheduled in advance (default 7 days notice for
  paying tenants; 24h for SANDBOX / PREVIEW tenants).
- Tenant admin notified by email + in-app banner.

**Migration steps:**

1. **Signed migration ledger entry** created in
   `microservices/tenancy/migration_ledger/`. Schema:

   ```sql
   CREATE TABLE tenant_migration_ledger (
       migration_id           UUID        PRIMARY KEY,
       tenant_id              TEXT        NOT NULL REFERENCES tenants(tenant_id),
       from_cell              TEXT        NOT NULL,
       to_cell                TEXT        NOT NULL,
       initiated_at           TIMESTAMPTZ NOT NULL,
       scheduled_for          TIMESTAMPTZ NOT NULL,
       completed_at           TIMESTAMPTZ,
       rolled_back_at         TIMESTAMPTZ,
       downtime_budget_seconds INT        NOT NULL,
       downtime_used_seconds  INT,
       reason                 TEXT        NOT NULL,
       approver_principal     TEXT        NOT NULL,
       signed_by              TEXT        NOT NULL,
       signature              BYTEA       NOT NULL,
       evidence_uri           TEXT        NOT NULL
   );
   ```

2. **Pre-migration drain.** Tenant traffic to the source cell is
   gradually drained over the migration window (10% / 25% / 50% /
   100% steps).
3. **Read-only flip.** Source cell tenant traffic is paused (Cedar
   `WriteInScope` forbid emergency-fragment); read traffic continues.
4. **Bulk replication.** Tenant data replicated from source cell
   storage to target cell storage. Replication uses per-table
   integrity hashing.
5. **Tail-replication.** Audit emissions during replication are
   tail-replicated to target.
6. **Cutover.** Tenants row `home_cell` updated atomically;
   `dr_cell` updated to the source cell's pair (or to a new pair
   per ADR-0241 portfolio).
7. **Read-write resume.** Cedar emergency-fragment retired; tenant
   traffic resumes on target cell.
8. **Source cell cleanup.** Source-cell tenant data retained for
   7 days as recovery option (per tier); then erased on a sweeper
   cycle.

**Downtime budget.** Per ADR-0241 DR tier:

| Tenant tier | Downtime budget |
|---|---|
| `enterprise` | < 60 seconds (read-only flip → cutover) |
| `pro` | < 5 minutes |
| `standard` | < 30 minutes |
| `free` | < 4 hours |
| `sandbox` / `preview` | not applicable (re-provision instead) |

**Rollback.** If cutover fails (target cell can't accept writes,
data integrity mismatch, Cedar fragment misconfiguration):

- `rolled_back_at` recorded in ledger.
- Cedar emergency-fragment retired on source cell.
- Tenant traffic resumes on source cell.
- Audit emission of `TenantMigrationRolledBack`.

### D-11. Audience-type enum

The `audience_type` column on `tenants` (D-3) takes one of seventeen
values; the first eight form the base set inherited from the
ADR-0220/0221 retirement, and the remaining nine were added during
the Wave-3-G doctrine cluster (ADR-0292 / ADR-0297-ADR-0306 / ADR-0311-
ADR-0321). The semantics drive Cedar resolution + observability labels:

#### D-11.1 Base set (foundational)

| Enum value | Semantics | Examples |
|---|---|---|
| `PLATFORM_OWNER` | The org operating the platform itself. Exactly one tenant carries this: `oyatie` (per ADR-0242). | `oyatie` |
| `B2B_TENANT` | Customer organisations using the platform commercially. | `tenant-acme-corp`, `tenant-stripe-internal`, `tenant-bigbank-eu` |
| `B2C_CONSUMER` | Individual consumer users (private accounts, not organisations). | `b2c-7f3a9c2e`, `b2c-user.<opaque>` |
| `DEVELOPER` | Individual developer accounts (e.g., plugin authors, integration developers, sandbox accounts not under `oyatie.dev.*`). | `dev-john-doe`, `dev-org-acme-developer` |
| `SANDBOX` | Per-engineer or per-team sandbox tenants used for development/testing. | `oyatie.dev.jasonlee` (technically a sub-scope; sandboxes outside `oyatie.*` are first-class tenants with this type) |
| `PREVIEW` | Per-PR or per-environment ephemeral preview tenants. | `oyatie.preview.123` (sub-scope) or first-class preview tenants for external collaborators |
| `PARTNER_AGENCY` | Partner organisations acting on behalf of B2B customer tenants (consulting, integration, services agencies). | `partner-bigfour-consulting`, `partner-implementation-co` |
| `RESELLER` | Reseller organisations managing portfolios of downstream tenants. | `reseller-emea-7`, `reseller-apac-channel` |

#### D-11.2 Wave-3-G extensions (cross-ADR-referenced; landed 2026-05-21)

These extensions resolve the corpus-rigor-audit-2026-05-21 finding that
ADR-0297, ADR-0298, ADR-0301, ADR-0303, ADR-0305, ADR-0306, ADR-0311,
ADR-0312, and the persona-journey-microservice-cross-coverage-matrix
referenced enum values that were never landed in ADR-0244 §D-11.

| Enum value | Semantics | Originating ADR | Examples |
|---|---|---|---|
| `B2C_FAMILY_PARENT` | Parent/guardian principal acting on behalf of a minor. Drives consent-graph routing + COPPA/KOSA refusals. | ADR-0292 (minor-user doctrine) | `b2c-parent-of-7f3a9c2e` |
| `B2C_JOB_SEEKER_ACTIVE` | Consumer principal in an active job-search cross-tenant context. Drives ADR-0317 role-projection (active job seekers see a different unified-shell projection than passive professional profiles). | ADR-0311 (dual-tenant) + ADR-0317 (role projection) | `b2c-7f3a9c2e` with `job_seeker_active = true` |
| `B2B_HR_ADMIN` | B2B employer HR administrator scope tenant (HR-only subsidiary tenant or HR organisational unit inside an existing B2B_TENANT). | ADR-0311 (dual-tenant) + ADR-0321 (HCM/Workday dossier) | `tenant-acme-corp.hr` |
| `B2B_INTERNAL_AUDIT` | B2B internal-audit organisational unit. Roll-up scope for ADR-0313 conglomerate audit-chain shards; MIDDLE-office scope per ADR-0319. | ADR-0313 + ADR-0319 | `tenant-acme-corp.internal-audit` |
| `INTERNAL_AUDITOR_3PAO` | External Third-Party Assessment Organization (3PAO/CPA/QA firm) tenant performing FedRAMP / SOC2 / ISO 27001 / PCI QSA / HIPAA OCR assessments. Time-bounded Cedar grant + MIDDLE-office clearance per ADR-0319. | ADR-0251 §D-10 + ADR-0319 | `tenant-coalfire-fedramp-3pao`, `tenant-pwc-soc2-auditor` |
| `INTERNAL_DEV_TOOLS` | oyatie-internal dev/CI/test surfaces hosted in dev-tools cells per ADR-0247. Distinct from `SANDBOX` because dev-tools cells carry production-grade observability but no customer data. | ADR-0247 + ADR-0297 §D-6.3 | `oyatie.dev-tools.cell-1` |
| `EMERGENCY_SERVICES` | Life-safety bypass principal (verified emergency-services origin, e.g., 911/112/119 PSAP, AMBER-alert relay, hospital code-blue). Drives ADR-0298 emergency-services-bypass refusal-override. | ADR-0298 (emergency-services-bypass-life-safety) | `emergency-svc-911-king-county`, `emergency-svc-112-eu-lifeline` |
| `FRIENDLY_CRAWLER_PARTNER` | Registered search/index/academic-research crawler tenant. Exempt from default-deny scrape-pattern detection per ADR-0297 §D-6.3. | ADR-0297 (abuse-defence baseline) | `crawler-google-search`, `crawler-archive-org`, `crawler-academic-aclu-research` |
| `MINOR_TARGETED` | Tenant whose surfaces predominantly serve users under 18 (COPPA refusal for <13, KOSA tier for 14-17). Drives aggressive abuse-defence + ADR-0292 consent-graph refusals. | ADR-0292 + ADR-0297 KOSA tier | `tenant-edu-k12-district-7`, `tenant-kids-creative-platform` |

The Wave-3-G extensions are additive only; no enum value was removed
or renamed. Migration of existing rows is unnecessary because no row
currently carries any extension value (extensions were authored as
"reserved literals" in upstream ADRs without column-population paths;
this ADR now ratifies the literals as canonical).

#### D-11.3 Binding + audit emission contract for requested extensions

The following contract is normative for the eight Wave-3-G extension
values referenced by ADR-0297, ADR-0313, and ADR-0319. Each value is
emitted into observability as the tenant's `audience_type` label on
tenant-registration, tenant-mutation, Cedar evaluation, and state-
changing audit emissions. Tenant creation MUST record
`audience_type`, `originating_adr`, `assigned_by_principal_id`,
`assignment_reason`, `jurisdiction_code`, and `cedar_policy_version`.
Audience-type mutation MUST additionally record `prior_audience_type`,
`new_audience_type`, `approval_ticket_id`, and `approved_by_principal_id`.

| Enum value | Definition | When assigned / emitted | Cedar binding example | Audit emission contract |
|---|---|---|---|---|
| `INTERNAL_AUDITOR_3PAO` | External auditor tenant performing 3PAO, CPA, QSA, SOC2, ISO, or comparable assessment work. | Assigned only after engagement, independence, and evidence-scope approval; emitted on time-bounded audit grants and every assessed-resource read. | `principal.tenant.audience_type == "INTERNAL_AUDITOR_3PAO" && context.engagement_id != ""` | Include `engagement_id`, `assessed_tenant_id`, `assessment_framework`, `scope_hash`, `expires_at`, and dual-seal audit stream IDs. |
| `B2B_HR_ADMIN` | Employer-owned HR administration tenant or HR organisational sub-scope. | Assigned during B2B workforce tenant provisioning; emitted on HR dossier, payroll, benefit, onboarding, offboarding, and role-projection decisions. | `principal.tenant.audience_type == "B2B_HR_ADMIN" && action == Tenant::Action::"ManageWorkforceRecord"` | Include `work_tenant_id`, redacted `employee_principal_ref`, `hr_surface`, `lawful_basis`, and `jurisdiction_code`. |
| `B2B_INTERNAL_AUDIT` | Customer internal-audit organisational tenant or sub-scope. | Assigned by a B2B tenant admin with ops-compliance approval; emitted on internal-control reads, sampled evidence reads, and cross-office audit workflows. | `principal.tenant.audience_type == "B2B_INTERNAL_AUDIT" && context.audit_purpose_id != ""` | Include `audit_purpose_id`, `control_framework`, `sample_set_hash`, `evidence_window`, and information-barrier outcome. |
| `B2C_JOB_SEEKER_ACTIVE` | Consumer principal actively seeking work across employer tenants while retaining personal-tenant ownership. | Assigned while an explicit job-seeking state is active; emitted on employer discovery, application, interview, portability, and role-projection actions. | `principal.tenant.audience_type == "B2C_JOB_SEEKER_ACTIVE" && context.job_search_consent_id != ""` | Include `job_search_consent_id`, `target_work_tenant_id`, redacted `candidate_profile_ref`, and consent expiry. |
| `EMERGENCY_SERVICES` | Verified life-safety responder or emergency relay tenant. | Assigned only through life-safety verification; emitted on emergency bypass, break-glass, and refusal-override decisions. | `principal.tenant.audience_type == "EMERGENCY_SERVICES" && context.life_safety_incident_id != ""` | Include `life_safety_incident_id`, `responder_agency_id`, `bypass_reason`, `post_event_review_due_at`, and notified tenant stream IDs. |
| `FRIENDLY_CRAWLER_PARTNER` | Registered crawler, search, archive, or approved research tenant. | Assigned after crawler agreement, identity attestation, and robots-policy binding; emitted on crawler allow, throttle, and scrape-defense decisions. | `principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER" && context.crawler_agreement_id != ""` | Include `crawler_agreement_id`, `crawler_user_agent_hash`, `route_class`, `rate_limit_policy_id`, and abuse-defence decision. |
| `MINOR_TARGETED` | Tenant surface intentionally serving minors or likely to be used by minors. | Assigned during tenant onboarding or compliance-pack activation; emitted on age-gated reads, content, contact, consent, and abuse-defense decisions. | `resource.tenant.audience_type == "MINOR_TARGETED" && (context.age_band == "under13" || context.age_band == "minor14_17")` | Include `age_band`, `guardian_consent_ref` when available, `minor_safety_policy_id`, and refusal or permit reason. |
| `INTERNAL_DEV_TOOLS` | Oyatie-internal dev, CI, staging, and tool tenant with production-grade telemetry and no customer-data entitlement. | Assigned only under the `oyatie` root; emitted on build, test, staging, policy-pack rehearsal, and self-modification dry-run decisions. | `principal.tenant.audience_type == "INTERNAL_DEV_TOOLS" && resource.cell_class == "dev-tools"` | Include `tool_surface`, `change_id`, `test_cell_id`, `customer_data_access=false`, and policy-pack rehearsal result. |

**Effect on Cedar resolution.** Cedar fragments may reference
`principal.tenant.audience_type` to scope permits. Example:

```cedar
// Marketplace publisher action is permitted for B2B and DEVELOPER
// tenants only; PARTNER_AGENCY can publish ON BEHALF OF B2B tenants
// via cross-tenant grant.

permit (
    principal,
    action == Marketplace::Action::"PublishPlugin",
    resource is Marketplace::Plugin
)
when {
    principal.tenant.audience_type in ["B2B_TENANT", "DEVELOPER"]
    && resource.owner_tenant == principal.tenant_id
};

permit (
    principal,
    action == Marketplace::Action::"PublishPlugin",
    resource is Marketplace::Plugin
)
when {
    principal.tenant.audience_type == "PARTNER_AGENCY"
    && context has "cross_tenant_grant_id"
    && context.cross_tenant_grant_actions_permitted.contains("Marketplace::PublishPlugin")
};
```

**Audience_type is mutable** only under ops-compliance approval +
audit emission. Common mutations:

- `B2B_TENANT → ENTERPRISE B2B_TENANT (tier change)` — handled via
  `tier` column, not `audience_type`.
- `DEVELOPER → B2B_TENANT (org upgrade)` — mutation requires KYB +
  audit emission.
- `PARTNER_AGENCY → RESELLER (business model change)` — mutation
  requires re-signing of partner agreement + audit emission.

**Inheritance.** Sub-scopes inherit `audience_type` from parent
tenant by default. Sub-scope override (in `tenant_sub_scopes` table)
is permitted only for ROOT tenants creating internal staging
sub-scopes (e.g., a B2B tenant creating `tenant-acme.preview` sub-
scope inherits `PREVIEW` audience_type for that branch).

### D-12. Reserved-namespace enforcement (re-cite from ADR-0242)

Reserved-namespace check (per ADR-0242 §D-6) is enforced by the
tenancy substrate admission gate before any row is inserted in the
`tenants` table. The check is a Cedar `forbid` fragment evaluated
against the proposed Tenant resource:

```cedar
// microservices/policy-engine/fragments/baseline/reserved-tenant-namespace.cedar
// Per ADR-0242 §D-6 + ADR-0244 §D-1 + §D-12.

forbid (
    principal,
    action == Tenancy::Action::"RegisterTenant",
    resource is Tenancy::Tenant
)
when {
    resource.normalized_id == "oyatie"
    || resource.normalized_id == "oya"
    || resource.normalized_id == "oyat"
    || resource.normalized_id == "oyati"
    || resource.normalized_id like "oyatie.*"
    || resource.normalized_id like "oyatie-*"
    || resource.normalized_id like "oyatie_*"
    || resource.normalized_id like "oya.*"
    || resource.normalized_id like "oya-*"
    || resource.normalized_id like "oya_*"
};
```

The normalisation step (NFKC + lowercase + diacritic-strip +
confusable-collapse) happens in
`microservices/tenancy/src/reserved_namespace.rs` (per ADR-0242
§D-6); the Cedar fragment evaluates against the normalised form, not
the user-submitted form.

**Additional reservation cases:**

- IDN-homograph variants of `oyatie` (Cyrillic 'а', Greek 'ο',
  Armenian 'ɢ' replacements per UTS#39 confusables) — refused.
- Substrings of `oyatie` (`oyat`, `oyati`, `yatie`, etc.) — refused
  for the listed reserved roots.
- Internal-system slugs: `system`, `_system`, `admin`, `root`,
  `bootstrap`, `migration` — refused as tenant IDs.

## Alternatives considered

### Alt-1. Per-µservice audience field retained (status quo)

Keep `audience: INTERNAL | B2B-tenant | B2C-consumer | DEVELOPER`
on each µservice manifest; route scoping decisions per the µservice's
declared audience.

**Pros:**

- Zero migration cost (already in place from ADR-0221 §M-04).
- Sharp visual separator in code review.

**Cons:**

- **Category error**: audience is a caller property, not a callee
  property. A µservice serving multiple audiences cannot honestly
  declare a single `audience`.
- **Drift loop documented**: ADR-0220 → ADR-0239 in 12 days; per
  ADR-0242 §Context, 6 distinct failure modes traced to this
  framing.
- **Cross-tenant operations impossible to model**: partner agency
  acting on customer's behalf has no clean expression under
  audience-on-µservice.
- **Every named hyperscaler reference disagrees**: AWS / GCP / Azure
  / Stripe / Cloudflare / Apple all use tenant-as-primitive + per-
  tenant attribute resolution.

**Rejected** because the model is structurally incoherent; carrying
it forward compounds the drift.

### Alt-2. Per-µservice tenancy with no shared tenant table (each µservice owns its tenant view)

Allow each µservice to declare its own concept of "tenant" suitable
to its needs (e.g., billing's notion of tenant ≠ identity's notion ≠
intelligence's notion).

**Pros:**

- Each µservice fully autonomous on tenancy concerns.
- No tenancy-substrate single-point-of-failure or coupling.

**Cons:**

- **Cross-µservice tenant identity becomes a translation problem.**
  Every call needs to map tenant IDs between µservices; drift
  guaranteed.
- **Audit-chain incoherent.** Per-µservice tenant IDs mean audit
  emissions can't be cross-µservice-joined without per-source-target
  mapping tables.
- **DSAR cascade impossible.** An erasure request would need to
  ask every µservice "what's your view of this tenant?" before
  cascading.
- **Compliance Pack abstraction breaks.** Packs presume a single
  tenant identity to attach to.
- **Cost attribution incoherent.** FinOps portal can't aggregate
  per-tenant spend if each µservice has its own tenant view.
- **No hyperscaler reference does this.** Even highly federated
  systems like Kubernetes carry a single namespace identifier across
  controllers.

**Rejected** because tenancy is a foundational primitive; per-µservice
divergence reproduces the audience-on-µservice problem at a
different level.

### Alt-3. Flat tenant model (no sub-scope hierarchy)

Tenant IDs only; no sub-scope hierarchy. Each team or workload that
would have been a sub-scope becomes its own first-class tenant.

**Pros:**

- Simpler schema (no `tenant_sub_scopes` table; no inheritance
  resolution).
- No depth limits to enforce.

**Cons:**

- **Inheritance default lost.** Every sub-team has to redeclare
  jurisdiction, home_cell, audience_type, Cedar policy. Drift
  guaranteed at scale.
- **Rollup default lost.** Audit, FinOps, observability per-team
  rollup requires explicit cross-tenant join queries.
- **Cross-tenant glob impossible.** "Permit any oyatie team member
  to read X" must enumerate every sub-tenant.
- **Tenant table explodes.** A typical org has hundreds of teams +
  sub-teams + role-based principals; a flat model makes the tenants
  table millions of rows where hierarchical would be thousands.
- **Doesn't match AWS / GCP / Azure hierarchical models.**

**Rejected** because flat models give up the inheritance and rollup
that make the primitive practical at scale.

### Alt-4. Five-axis scoping (tenant × cell × team × project × environment)

Adopt a richer scoping primitive: each entity carries (tenant_id,
cell_id, team_id, project_id, environment) and policy evaluates
across all five axes.

**Pros:**

- Expresses every conceivable scoping dimension explicitly.
- Each axis can be reasoned about independently.

**Cons:**

- **N×M×K×L×P combinatorial explosion.** Cedar fragments would need
  cross-axis guards; coverage CI lane grows exponentially.
- **Maps poorly to identity tokens.** OIDC + JWT carry sub-id; five
  axes need five claims; token bloat.
- **No hyperscaler reference uses five axes.** AWS uses path
  hierarchy (effectively one axis with structure); GCP uses
  resource hierarchy (one axis); Azure uses tenant + resource group
  + resource (effectively two-three axes with strong hierarchy).
- **Tenant + sub-scope expresses everything we need.** Cell is a
  per-tenant attribute (D-3 `home_cell`); team is a sub-scope
  segment; project is a sub-scope segment; environment is either
  a sub-scope (preview / staging / prod) or a separate tenant
  (preview tenants in §D-9).

**Rejected** because the same expressiveness is available under
tenant + sub-scope without the combinatorial cost.

## Consequences

### Positive

1. **Audience-as-µservice-scope retired.** The category error from
   ADR-0220 / ADR-0239 / ADR-0221 §M-04 is structurally eliminated.
2. **Single primitive for 23 scoping decisions.** Cell binding, audit
   stream, cost center, retention, residency, DSAR cascade, provider_credential_mode, encryption-BYOK,
   quota tier, feature gates, etc. — all parameterised on (tenant_id,
   sub_scope_path).
3. **Cross-tenant operations cleanly expressible.** Partner agency
   OBO, reseller hierarchy, sharing, assume-role all use the same
   `CrossTenantGrant` Cedar entity.
4. **`oyatie-is-a-tenant` (ADR-0242) cascades cleanly.** Every
   decision that distinguishes `oyatie` from `tenant-acme` is now
   answered by the tenant table, not by µservice carve-out.
5. **Cedar fragment coverage uniform.** Per-action permits +
   default-deny per-tenant-attribute-condition; no per-µservice
   audience branching.
6. **Hyperscaler shape.** Matches AWS IAM principal hierarchy, GCP
   resource hierarchy, Azure AAD, Stripe Connect.
7. **Sandbox + preview tenants first-class.** Engineer DX +
   per-PR review flow have a native model rather than being bolted
   on.
8. **Cross-cell migration tractable.** Schema + ledger + signed
   evidence support planned tenant relocation.
9. **DSAR + retention + legal-hold uniformly applied.** Same
   cascade machinery for any tenant (including `oyatie`); same
   GDPR / KR PIPA / state-law compliance.

### Negative

1. **One-time manifest sweep.** ~46 µservice manifests rewritten to
   remove `audience` + add new fields. Bounded; one ChangeSet
   executes it.
2. **Tenant table is on the hot path of every request.** Per-request
   tenant lookup (Citus shard on tenant_id; in-cell cache). Latency
   budget: < 0.5ms p99 on hot path. Mitigation: per-cell Postgres
   read replicas + Valkey cache + tenant row pinned in evaluator
   process memory for active tenants.
3. **Sub-scope depth limit (5) is opinionated.** Some real-world
   orgs might want deeper. Mitigation: 5 is sufficient for AWS IAM
   path patterns; deeper hierarchies should compress via sharding
   into separate tenants.
4. **Cross-tenant grant complexity.** Four grant kinds + Cedar
   permit per kind + audit emission on every assumption. Mitigation:
   well-defined patterns; documented.
5. **Reserved-namespace logic carries Unicode complexity.**
   Mitigation: `unicode-security` crate; TR#39 conformance test
   suite (per ADR-0242 §D-6).

### Operational

1. **New CI lanes (advisory until substrate lands; BLOCKER post-
   substrate):**
   - `oya-check-tenant-id-format` — verifies every tenant ID matches
     the §D-1 regex + reserved-namespace check.
   - `oya-check-sub-scope-depth` — verifies every sub-scope path is
     ≤ 5 segments.
   - `oya-check-no-audience-on-microservice` — verifies no µservice
     manifest declares the legacy `audience` field.
   - `oya-check-tenant-schema-coherence` — verifies every `tenants`
     row satisfies declared constraints (FK to parent_tenant_id,
     home_cell exists in `cells`, jurisdiction_code matches ISO,
     etc.).
   - `oya-check-reserved-namespace-protection` (inherited from
     ADR-0242 verification).
   - `oya-check-cross-tenant-permit-coverage` — verifies every
     declared cross-tenant action has a Cedar permit + default-deny.
2. **New µservice surfaces:**
   - `microservices/tenancy/migrations/0002_canonical_tenant_schema.sql`
   - `microservices/tenancy/src/lifecycle.rs` (state machine)
   - `microservices/tenancy/src/sub_scope_resolver.rs` (inheritance +
     rollup)
   - `microservices/tenancy/src/cross_tenant_grants.rs`
   - `microservices/tenancy/src/migration_ledger.rs`
   - `microservices/tenancy/src/sandbox_lifecycle.rs`
   - `microservices/tenancy/src/preview_lifecycle.rs`
3. **Observability:**
   - Every metric, log, trace carries `tenant_id` + `sub_scope_path`
     labels.
   - Per-tenant dashboards default to filtering on the calling
     tenant; cross-tenant operators get aggregate views via Cedar
     `ObservabilityAdmin` permits.
4. **Identity substrate integration:** Zitadel projects are scoped
   per tenant; service-principals carry sub_scope_path as a custom
   claim.
5. **Audit-chain integration:** every audit row carries `tenant_id`
   + `sub_scope_path`; rollup views computed per-period.
6. **FinOps portal integration:** cost centers track tenant +
   sub-scope; rollups follow sub-scope tree.
7. **DSAR runbook update:** `docs/runbooks/dsar-cascade.md` updated
   to reference the canonical tenant model.

### Sustainability

- Per-tenant + per-sub-scope FinOps visibility enables sustainability
  budgets at any granularity. Carbon-tag (per ADR-0174) inherits sub-
  scope tree.
- Sandbox + preview tenant resource budgets (D-8, D-9) bound the
  per-engineer + per-PR carbon footprint.

### Compliance

- **GDPR Article 17 (right to erasure).** Tenant + sub-scope DSAR
  cascade per §D-7 handles erasure uniformly.
- **GDPR Article 30 (records of processing).** Tenant table is the
  canonical processing record source.
- **KR PIPA Article 36 (information subject rights).** Same cascade.
- **SOC 2 CC6.1 (logical access).** Cedar + tenant scoping provides
  evidence.
- **ISO 27001 A.9 (access control).** Same.
- **HIPAA Security Rule §164.312.** Per-tenant + per-sub-scope
  access control.
- **EU AI Act Article 26 (high-risk system deployer obligations).**
  Per-tenant AI usage records keyed on tenant + sub-scope.
- **FRCP 37(e) (preservation of ESI).** Legal-holds column on
  `tenants` (D-3) suppresses retention sunset.
- **CSAP v3.1 (Korean cloud assurance).** Uniform tenant treatment
  satisfies multi-tenancy controls.
- **FedRAMP Moderate AC-2 (Account Management).** Tenant lifecycle
  (§D-7) maps to account-management controls.

## Implementation surface

| Artifact | Status |
|---|---|
| `/specs/tenant-model.json` | NEW — canonical tenant schema spec |
| `/specs/microservices/tenancy.json` | UPDATE — references this ADR |
| `/specs/microservices/identity.json` | UPDATE — sub-scope claim |
| `/specs/microservices/audit-chain.json` | UPDATE — per-tenant + per-sub-scope rollup views |
| `/specs/microservice-manifest-schema.json` | NEW — canonical manifest schema (replaces ad-hoc per-µservice schema) |
| `microservices/tenancy/migrations/0002_canonical_tenant_schema.sql` | NEW — full DDL per §D-3 |
| `microservices/tenancy/migrations/0003_tenant_sub_scopes.sql` | NEW — sub-scope table |
| `microservices/tenancy/migrations/0004_tenant_migration_ledger.sql` | NEW — cross-cell migration ledger |
| `microservices/tenancy/migrations/0005_cross_tenant_grants.sql` | NEW — grants table |
| `microservices/tenancy/src/lifecycle.rs` | NEW — state machine implementation |
| `microservices/tenancy/src/sub_scope_resolver.rs` | NEW — inheritance + rollup |
| `microservices/tenancy/src/cross_tenant_grants.rs` | NEW — grant lifecycle |
| `microservices/tenancy/src/migration_ledger.rs` | NEW — signed cross-cell ledger |
| `microservices/tenancy/src/sandbox_lifecycle.rs` | NEW — sandbox provisioning |
| `microservices/tenancy/src/preview_lifecycle.rs` | NEW — preview provisioning |
| `microservices/policy-engine/schemas/tenant.cedarschema` | NEW — Cedar entity-types |
| `microservices/policy-engine/fragments/baseline/tenant-scope-baseline.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/cross-tenant-baseline.cedar` | NEW |
| `microservices/policy-engine/fragments/baseline/reserved-tenant-namespace.cedar` | UPDATE — extends ADR-0242 fragment |
| `microservices/policy-engine/fragments/baseline/sub-scope-inheritance.cedar` | NEW |
| `microservices/identity/src/service_principal_sub_scope_claim.rs` | UPDATE — emits sub_scope_path claim |
| `microservices/audit-chain/src/rollup_view_provisioner.rs` | UPDATE — per-sub-scope rollup |
| `microservices/finops-portal/src/sub_scope_cost_attribution.rs` | UPDATE — per-sub-scope cost attribution |
| `microservices/tenancy/ARCHITECTURE.md#cell-assignment` | UPDATE — home_cell + dr_cell binding |
| `crates/oya-shared-tenancy-client/` | NEW — per-µservice SDK |
| `tools/oya-check-tenant-id-format/` | NEW |
| `tools/oya-check-sub-scope-depth/` | NEW |
| `tools/oya-check-no-audience-on-microservice/` | NEW |
| `tools/oya-check-tenant-schema-coherence/` | NEW |
| `tools/oya-check-cross-tenant-permit-coverage/` | NEW |
| `docs/standards/tenant-model-authoring.md` | NEW — full standards doc |
| `docs/runbooks/tenant-lifecycle.md` | NEW — provisioning / suspension / offboarding / recovery |
| `docs/runbooks/sandbox-tenant-management.md` | NEW |
| `docs/runbooks/preview-tenant-management.md` | NEW |
| `docs/runbooks/cross-cell-tenant-migration.md` | NEW |
| `docs/runbooks/dsar-cascade.md` | UPDATE — references the canonical tenant model |
| `docs/runbooks/cross-tenant-grant-lifecycle.md` | NEW |
| Sweep: removal of `audience` field from ~46 µservice manifests | SWEEP |
| Sweep: removal of CI lane `oya-check-audience-coherence` | SWEEP |
| Sweep: addition of `tier`, `primary_tenants`, `serves_oyatie_internal_ops`, `cellular_deployment_pattern`, `dr_pair_strategy`, `bootstrap_tier` to every manifest | SWEEP |

## Verification

- [ ] `/specs/tenant-model.json` validates against the JSON Schema; `/specs/microservice-manifest-schema.json` validates against itself.
- [ ] `microservices/tenancy/migrations/0002_canonical_tenant_schema.sql` runs on a clean Postgres 16 + Citus instance with zero errors; creates `tenants` + `tenant_sub_scopes` tables; creates all required enum types.
- [ ] Inserting a row with `tenant_id = "oyatie"` post-bootstrap succeeds (reserved-namespace check is bypassed for the bootstrap migration only).
- [ ] Inserting a row with `tenant_id = "Oyatie-Corp"` is refused (case-fold + reserved-prefix violation).
- [ ] Inserting a row with `tenant_id = "оyatie"` (Cyrillic 'о') is refused (NFKC normalisation collapses to "oyatie").
- [ ] Inserting a row with `tenant_id = "tenant-acme-corp"` succeeds.
- [ ] Inserting a sub-scope row with `sub_scope_path = "a.b.c.d.e.f"` is refused (depth > 5).
- [ ] Cedar fragment `tenant-scope-baseline.cedar` validates against `tenant.cedarschema`.
- [ ] Cedar fragment `cross-tenant-baseline.cedar` validates against the schema; cross-tenant Read returns Permit when grant active, Forbid otherwise.
- [ ] State-machine integration test: a tenant transitions PROVISIONING → ACTIVE → SUSPENDED → ACTIVE → OFFBOARDING → SOFT_DELETED → ACTIVE (recovery) → OFFBOARDING → SOFT_DELETED → HARD_DELETED; each transition emits a correctly-shaped audit row.
- [ ] DSAR cascade test: erasure request on a tenant's principal triggers cascade across identity, audit-chain, observability, finops, workflow-engine, source-control; cascade completes within 30 days SLO.
- [ ] Sandbox lifecycle test: provisioning a sandbox via `oyatie.dev.<id>` succeeds; auto-teardown fires after 24h idle; resource budget enforced via Cedar.
- [ ] Preview lifecycle test: PR open triggers preview provisioning within 60s; PR close triggers teardown within 60min.
- [ ] Cross-cell migration test: signed ledger entry created; replication completes; cutover within downtime budget; rollback path exercised on simulated failure.
- [ ] `oya gate validate tenant-id-format` exits 0 over the canonical fixture set.
- [ ] `oya gate validate sub-scope-depth` exits 0 over the canonical fixture set.
- [ ] `oya gate validate no-audience-on-microservice` exits 0 (no µservice manifest declares `audience`).
- [ ] `oya gate validate tenant-schema-coherence` exits 0.
- [ ] `oya gate validate reserved-namespace-protection` exits 0.
- [ ] `oya gate validate cross-tenant-permit-coverage` reports 100% coverage.
- [ ] ADR-0220, ADR-0239, ADR-0221 frontmatter updated to reference this ADR as amender.
- [ ] Hot-path tenant lookup p99 < 0.5ms measured at 10k QPS per cell.

## References

### Industry sources

- **AWS Identity and Access Management User Guide (2024 ed.).** IAM principal path + AWS Organizations hierarchy.
- **AWS re:Invent 2024 IAM-403 — "Hierarchical permissions at scale."** Multi-account + multi-OU permission patterns.
- **AWS Verified Permissions documentation (GA 2024-Q1).** Cedar entity hierarchy + cross-account grants.
- **AWS Builder's Library — "Workload Isolation Using Shuffle Sharding" (Colm MacCárthaigh).** Tenant isolation patterns at scale.
- **Google Cloud Documentation — "Resource hierarchy" (2024).** Organization → Folder → Project → Resource model.
- **Google Cloud — "Cross-organization sharing" documentation (2024).** Cross-tenant grants pattern.
- **Google CRE Book (2024 ed.), ch. 8 — "Multi-tenancy patterns".** Tenancy as substrate primitive.
- **Microsoft Build 2024 keynote — Azure AAD multi-tenancy.** Tenant model across Microsoft 365 + Azure + GitHub.
- **Azure Active Directory Documentation (2024-2025) — "Tenancy in Azure Active Directory".** Tenant + directory + subscription hierarchy.
- **Azure AD B2B Collaboration documentation (2024).** Cross-tenant guest patterns.
- **Stripe Engineering Blog 2024 — "Designing for global platforms".** Platform + connected account hierarchy.
- **Stripe API Reference 2025 — Accounts section.** account capability flags.
- **Stripe Engineering Blog 2025 — "Lifecycle of a Stripe account".** Provisioning + KYB + suspension + offboarding patterns.
- **Cloudflare Blog 2024 — "Building on our own platform".** Cloudflare-tenant-of-Cloudflare; account → zone hierarchy.
- **Cloudflare API Documentation (2024).** Account + zone + sub-zone scoping.
- **Apple WWDC 2024 — "Managing your team in App Store Connect".** Team membership + role hierarchy.
- **Apple Developer Documentation (2024) — Team Management.** Per-app capability scoping.
- **Salesforce Trailhead — "Multi-tenant architecture" (2024 update).** Salesforce-as-tenant-of-Trust-Cloud.
- **Pat Helland — "Life Beyond Distributed Transactions" (CACM 2017 re-issue).** Tenant-as-primitive foundational theory.
- **Werner Vogels — "10 Lessons from 10 Years of AWS" (All Things Distributed, 2016).** Amazon-as-tenant-of-AWS framing.
- **Verma et al. — "Borg, Omega, and Kubernetes" (CACM 2016 vol. 59 no. 5).** Internal Google teams as Borg/K8s tenants.
- **Brandur Leach — Stripe Engineering posts (2014-2018).** Stripe internal use of Stripe.
- **Unicode Technical Standard #39 — Unicode Security Mechanisms.** Confusable detection for tenant ID validation.
- **Unicode Technical Report #36 — Unicode Security Considerations.** IDN homograph attacks.
- **NIST SP 800-162 — Attribute Based Access Control (ABAC).** Tenant attributes as ABAC inputs.
- **NIST SP 800-207 — Zero Trust Architecture.** Per-request tenant evaluation.
- **GitHub Engineering Blog 2024 — "Organizations + Enterprise Accounts at GitHub".** Three-level hierarchy + cross-organization workflows.

### Regulatory sources

- **GDPR (Regulation (EU) 2016/679) Article 17 — Right to erasure.** DSAR cascade per §D-7.
- **GDPR Article 12 — Modalities for exercising data-subject rights.** Response SLA.
- **GDPR Article 30 — Records of processing activities.** Tenant table as canonical processing record.
- **KR PIPA (Personal Information Protection Act) Article 36 — Information Subject Rights.** Erasure equivalent.
- **EU AI Act 2024/1689 Article 26 — Deployer obligations.** Per-tenant usage records.
- **CSAP v3.1 — Korean Cloud Security Assurance Program.** Multi-tenancy controls.
- **SOC 2 Type II — Trust Service Criteria CC6.1, CC7.2.** Logical access + audit.
- **ISO 27001:2022 Annex A.9 — Access Control.** Standard access-control evidence.
- **ISO 22301:2019 — Business continuity management systems.** Tenant inclusion in BC scope.
- **HIPAA Security Rule §164.312 — Technical safeguards.** Per-tenant + per-PHI scoping.
- **FRCP 37(e) — Failure to Preserve Electronically Stored Information.** Legal hold suppression of retention sunset.
- **Sedona Conference Working Group 1 — "The Sedona Principles" (3rd ed.).** eDiscovery legal hold authority.
- **FedRAMP Moderate AC-2 — Account Management.** Tenant lifecycle controls.
- **NYDFS Cybersecurity Regulation 23 NYCRR 500.** Cross-tenant access control evidence.

### Internal portfolio ADRs

- **ADR-0009 — Cell architecture per-tenant per-region.** Tenant binds to home_cell + dr_cell (D-3).
- **ADR-0010 — Regional pack architecture.** Sovereign-cloud-pack column (D-3).
- **ADR-0028 — Cloud microservice architecture.** Audit-chain emission per tenant.
- **ADR-0049 — Cross-region replication + residency.** data_residency_allowed (D-3).
- **ADR-0099 — Data class registry.** Data classes used in tenant attribute resolution.
- **ADR-0105 — Thirteen-layer canonical enum.** Layer rules unchanged.
- **ADR-0128 — Hyperscaler architecture invariants.** Doctrine alignment.
- **ADR-0131 — Per-microservice flat layout.** Layout unchanged.
- **ADR-0132 — No-grouping forward policy.** No grouping µservices created.
- **ADR-0144 — EU AI Act graduated-risk tier model.** Per-tenant AI tier (D-3 compliance_packs).
- **ADR-0145 — Inter-microservice communication reform.** Tenant context in gRPC metadata.
- **ADR-0150 — Cedar policy engine.** Tenant entity-types live in policy-engine.
- **ADR-0174 — FinOps sustainability tagging.** finops_cost_center inheritance per sub-scope.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Tenant in both layers.
- **ADR-0211 — In-house tech stack preference.** Tenancy substrate is Rust.
- **ADR-0212 — Buildability doctrine.** This ADR is a deliverable artifact.
- **ADR-0215 — Multi-context platform.** Tenant is context root.
- **ADR-0218 — Tenant granular control surface.** Admin console per tenant + sub-scope.
- **ADR-0220 — Consumer Intelligence Substrate.** AMENDED — audience-as-µservice framing replaced.
- **ADR-0221 — Agentic development pipeline hardening.** AMENDED — §M-04 manifest `audience` field removed.
- **ADR-0239 — Foundry internal scope clarification.** AMENDED — internal/consumer split replaced by tenant scoping.
- **ADR-0240 — Sovereign cloud per regional pack.** Applies via tenant jurisdiction + pack.
- **ADR-0241 — DR + BC portfolio policy.** dr_pair_strategy column (D-3).
- **ADR-0242 — `oyatie`-is-a-tenant doctrine (keystone #1).** Cascades into this primitive.
- **ADR-0243 — Cedar as universal gate (keystone #2).** Tenant attributes drive Cedar.
- **ADR-0245 — Substrate vs product layering (keystone #4 — companion).**
- **ADR-0246 — Policy-engine substrate promotion (keystone #5 — companion).**
- **ADR-0247 — Self-hosting / self-modification doctrine (keystone #6 — companion).**
- **ADR-0248 — Amazon-shape cellular architecture (keystone #7 — companion).**
- **ADR-0251 — Compliance Pack + Cell Certification Levels (companion).** Packs activate per tenant.

### Auto-memory feedback

- `feedback_oyatie_is_a_tenant_doctrine` — keystone #1 dependency.
- `feedback_bominal_inheritance_precedence` — overrides Bominal's audience-as-µservice framing.
- `feedback_quality_performance_scalability_bar` — hyperscaler-grade tenant primitive.
- `feedback_autonomous_implementation_artifacts` — enables autonomous tenant lifecycle workflows.
- `feedback_flat_product_catalog` — preserved.
- `feedback_canonical_base_localization` — pack overlays compose at evaluation time.
- `feedback_no_silent_regression` — tenant schema is versioned; sub-scope additions go through ADR + migration.
- `feedback_automate_everything` — tenant lifecycle is fully automatable.
- `feedback_clean_architecture_requirements` — tenant primitive lives at kernel layer; per-µservice tenant views eliminated.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in ADR-0242 + ADR-0243, every
architectural decision in this ADR is attributed to a named
hyperscaler pattern + source + anti-pattern avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (Tenant ID format) | "Globally Unique Slug + DNS-Compatible Segments" | RFC 1035; AWS account-alias rules; Stripe account ID conventions | "Auto-Incrementing Integer Tenant ID" — leaks customer count + ordering |
| D-1 (Reserved namespace) | "Reserved Identifier Namespace + IDN Homograph Defence" | AWS `arn:aws:iam::aws:`; UTS #39; UTR #36 | "Typosquatting Tenant Impersonation" — partner registers `oyatie-fake` to imply affiliation |
| D-2 (Dotted hierarchical sub-scope) | "Hierarchical Principal Path" | AWS IAM principal paths; GCP resource hierarchy; Azure RBAC scope; Kubernetes namespace hierarchy | "Flat Namespace Drift" — inheritance requires explicit cross-namespace queries at scale |
| D-2 (Max depth 5) | "Bounded-Depth Hierarchy" | AWS IAM path limit; Azure subscription nesting limit; GCP folder depth limit (10 in practice; 5 recommended) | "Unbounded Tree Depth" — policy evaluation exponential in depth |
| D-3 (Tenant table schema) | "Single Source of Truth Tenant Registry" | AWS Organizations master account table; GCP Resource Manager hierarchy table; Stripe Accounts table | "Per-µservice Tenant View Drift" — each µservice rolls its own tenant concept |
| D-3 (Capability flags) | "Capability-Based Authorization" | Stripe account capabilities; AWS IAM permission boundaries; Linux capabilities(7) | "Role-Based-Only" — coarse role assignment misses per-capability gating |
| D-3 (DR pair strategy enum) | "Tier-Aware DR Strategy" | AWS Resilience Hub tiers; Azure Site Recovery patterns | "One-Size-Fits-All DR" — premium tier RTO applied to every tenant |
| D-4 (Cedar entity-types) | "Typed Entity Policy Schema" | AWS Verified Permissions Cedar entity schema; OPA structured-data policies | "Untyped String Match Policy" — fragile per-string conditions |
| D-5 (Manifest schema; drop audience) | "Caller-Side Attribute Resolution" | AWS principal-attribute policy conditions; Azure AAD claims-based; Stripe webhook tenant_id in payload | "Callee-Side Audience Declaration" — category error retired |
| D-6 (Cross-tenant grants) | "Time-Bounded Cross-Tenant Grant" | AWS STS AssumeRole cross-account; Azure AAD B2B Collaboration; Stripe platform-on-behalf-of | "Permanent Cross-Tenant Trust" — perpetual elevation; bypass-path acquisition |
| D-6.3 (Partner OBO) | "Platform-on-Behalf-Of Pattern" | Stripe Connect; Salesforce Partner Portal; AWS Marketplace partner accounts | "Direct Customer Credential Sharing" — partner holds customer secrets |
| D-7 (Tenant lifecycle state machine) | "Multi-State Tenant Lifecycle with Soft-Delete Window" | AWS Organizations account close (90-day grace); GCP Project delete (30-day soft-delete); Azure AD tenant delete (30-day recovery) | "Hard-Delete-Only Lifecycle" — accidental deletes irrecoverable |
| D-7 (Hard delete cascade + tombstone) | "Cascade-Plus-Tombstone Deletion" | AWS Organizations CLOSED account preserves audit; GCP Project SOFT_DELETED preserves logs | "Total Erasure Including Audit" — regulatory violation; tamper detection broken |
| D-8 (Sandbox tenants) | "Per-Engineer Sandbox Tenant" | AWS Cloud9 + AWS Sandboxes; Stripe Test Mode; Heroku development apps | "Shared Development Tenant" — engineers step on each other's data |
| D-9 (Preview tenants) | "Per-PR Ephemeral Tenant" | Vercel preview deployments; Heroku Review Apps; Render preview environments | "Manual Pre-Production Promotion" — slow review cycle |
| D-10 (Cross-cell migration) | "Signed Migration Ledger + Drain + Cutover" | AWS Database Migration Service patterns; Google Spanner re-shard; Cassandra token migration | "Big-Bang Migration" — irreversible if cutover fails |
| D-11 (Audience-type enum) | "Closed-Enum Tenant Classification" | Stripe account type enum; Salesforce customer-vs-partner-vs-internal; Azure AAD tenant type | "Free-Form Audience Tags" — drift across tenants |
| D-12 (Reserved namespace enforcement) | "Defence-in-Depth via Cedar Fragment" | AWS Service Control Policy; GCP Org Policy constraints; Kubernetes admission controller | "Application-Layer-Only Check" — bypass via direct DB write |

---

## Appendix B: Worked example — cross-tenant collaboration between two B2B tenants in different cells

To illustrate that the cross-tenant primitive (D-6) works concretely
under residency + cell + audit constraints, here is a worked example.

**Scenario.** Two customer tenants collaborate on a shared workflow:

- **Tenant A:** `tenant-acme-corp` — Berlin-based; jurisdiction
  `EU`; home_cell `data-plane-cell-eu-central-1-a`; compliance_packs
  `[gdpr-eu, soc2-t2, iso-27001]`; audience_type `B2B_TENANT`; tier
  `enterprise`.
- **Tenant B:** `tenant-globalbank-us` — Delaware-based; jurisdiction
  `US-DE`; home_cell `data-plane-cell-us-east-1-c`; compliance_packs
  `[soc2-t2, pci-dss-l1, sox]`; audience_type `B2B_TENANT`; tier
  `enterprise`.

The two tenants want to collaborate on a Workflow Studio template
that Acme has authored (`workflow-template:acme-invoice-validation-v3`).
Acme wants to share this template with GlobalBank — but only the
template definition, not Acme's tenant data; and only for use within
GlobalBank's own audit trail.

**Step 1 — Acme issues a CrossTenantGrant.**

Acme admin (`tenant-acme-corp.admin.olivia`) opens the admin console.
She selects "Share workflow template" + target tenant `tenant-globalbank-us` +
resource `workflow-template:acme-invoice-validation-v3` + actions
`[Read, Execute]` + expiry `2026-08-20`.

The admin console submits to `microservices/tenancy/`. Cedar evaluates
`Tenancy::Action::IssueCrossTenantGrant`:

```cedar
permit (
    principal,
    action == Tenancy::Action::"IssueCrossTenantGrant",
    resource is Tenancy::Tenant
)
when {
    principal.tenant_id == resource.tenant_id
    && principal.sub_scope_path like "*.admin.*"
    && principal.mfa_strength in ["webauthn", "hardware-key"]
    && resource.lifecycle_state == "ACTIVE"
};
```

Permit fires. A new row in `cross_tenant_grants`:

```
grant_id                = grant-7f3a9c2e
from_tenant             = tenant-acme-corp
to_tenant               = tenant-globalbank-us
from_sub_scope          = tenant-acme-corp.workflows.invoice
to_sub_scope            = tenant-globalbank-us
grant_kind              = share
actions_permitted       = ["WorkflowEngine::Read", "WorkflowEngine::Execute"]
resources_permitted     = ["workflow-template:acme-invoice-validation-v3"]
issued_at               = 2026-05-20T14:32:00Z
expires_at              = 2026-08-20T14:32:00Z
revoked                 = false
approver_principal      = tenant-acme-corp.admin.olivia
evidence_uri            = s3://oyatie-evidence/grants/grant-7f3a9c2e/issuance.json
```

Audit emission lands in both tenants' streams: `tenant-acme-corp.root`
+ `tenant-globalbank-us.root`.

**Step 2 — GlobalBank accepts (optional, per cross-tenant doctrine).**

GlobalBank admin (`tenant-globalbank-us.admin.michael`) sees the
incoming grant in the admin console. He clicks "Accept." This emits
`CrossTenantGrantAccepted` to both streams.

**Step 3 — Shadow projection across cells.**

The cross-cell projection service (per ADR-0049) creates a *shadow*
of `workflow-template:acme-invoice-validation-v3` in GlobalBank's
home_cell (`data-plane-cell-us-east-1-c`). The shadow is marked
`owner_tenant = tenant-acme-corp` + `viewer_tenant = tenant-globalbank-us`.

Residency check: the shadow projection respects GDPR + US-DE residency
rules. The workflow *template definition* (no PII) projects without
issue. If the template referenced Acme's tenant data, the data would
NOT project — only the template definition does.

**Step 4 — GlobalBank's principals execute the template.**

GlobalBank engineer (`tenant-globalbank-us.engineering.alice`) opens
Workflow Studio. She sees the shared template available. She clicks
"Execute against my tenant data."

Workflow Engine submits to `microservices/policy-engine/`. Cedar
evaluates `WorkflowEngine::Action::Execute`:

```cedar
permit (
    principal,
    action == WorkflowEngine::Action::"Execute",
    resource is WorkflowEngine::Template
)
when {
    // Same-tenant execute is fine
    (principal.tenant_id == resource.owner_tenant)
    ||
    // Cross-tenant: grant must be active
    (context has "cross_tenant_grant_id"
     && context.cross_tenant_grant_active == true
     && context.cross_tenant_grant_expires_at > context.now
     && context.cross_tenant_grant_actions_permitted.contains("WorkflowEngine::Execute")
     && resource.resource_id in context.cross_tenant_grant_resources_permitted)
};
```

Permit fires under the cross-tenant branch (grant `grant-7f3a9c2e`
is active + Execute permitted + resource matches).

Workflow Engine runs the template against `tenant-globalbank-us`'s
data (NOT Acme's data). Audit emissions:

- `tenant-globalbank-us.workflows`: workflow execution event with
  Acme's template as input; GlobalBank's data as data input.
- `tenant-acme-corp.shares.audit`: cross-tenant template usage event;
  notes GlobalBank executed the template; does NOT reveal GlobalBank's
  data contents.

**Step 5 — Cross-cell audit chain rollup.**

Both tenants' audit streams receive the event. Acme can query their
`shares.audit` view to see "GlobalBank executed
acme-invoice-validation-v3 at 2026-05-20T14:45:11Z" — useful for
licensing / usage analytics. GlobalBank can query their `workflows`
view to see the full execution context (data they ran it against,
results, etc.) — Acme has no visibility into GlobalBank's data.

**Step 6 — Grant revocation.**

On 2026-07-15, Acme decides to update the template and revoke the
old grant. Olivia clicks "Revoke" in the admin console. Cedar
evaluates `Tenancy::Action::RevokeCrossTenantGrant`. Permit fires
(same conditions as issuance + grant must exist + caller must be in
grant's `from_tenant`).

`cross_tenant_grants.revoked = true` + audit emission to both
streams. Within 60 seconds:

- Cedar evaluations for the grant now return `Forbid` (the `revoked`
  flag is checked).
- In-flight sessions complete their current operations but no new
  Execute calls succeed.
- The shadow remains projected (read-only, no longer executable);
  GlobalBank admin sees a banner that the template is no longer
  available.

**Step 7 — GDPR Article 17 erasure scenario.**

Suppose a GlobalBank user exercises right-to-erasure six months
later. The DSAR cascade:

- Enumerates GlobalBank tenant resources owned by the subject.
- The workflow executions (Step 4) are subject's actions; the data
  is GlobalBank's.
- Erasure plan: hard-delete the subject's identifier in workflow
  executions; pseudonymise the execution audit row in
  `tenant-globalbank-us.workflows`; tombstone the cross-tenant
  audit row in `tenant-acme-corp.shares.audit` (subject identifier
  replaced with hash; Merkle proof retained).
- Acme is NOT notified individually of GlobalBank user erasures (no
  data sharing) but the tombstone preserves tamper-detection.

**Why this works.** Under the universal tenant + sub-scope + Cedar
primitive:

- Cross-tenant operation is explicit (grant entity; signed; audit-
  emitted; revocable).
- Residency is automatically enforced (shadow projects only what
  residency permits).
- Audit is automatically emitted to both sides (no manual
  cross-tenant reporting).
- DSAR cascades correctly (tombstone preserves audit tamper-detection).
- Revocation is immediate (Cedar fragment checks `revoked` flag).

Under the prior audience-as-µservice-scope model:

- Cross-tenant operations had no clean expression; Acme would have
  had to grant GlobalBank "B2B-tenant access" to all Acme's data, or
  invent a one-off workflow.
- Shadow projection was per-µservice ad-hoc.
- Audit emission required cross-µservice joins to attribute the
  event to both tenants.
- DSAR cascades risked missing cross-tenant audit rows.
- Revocation was per-µservice ad-hoc rather than centralised.

The keystone unifies these mechanisms behind one primitive.

## Naming justification

Per `feedback_naming_justification`: every new name introduced by this ADR carries a one-line BNF v4.1 + ADR-0105 13-layer conformance justification.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|---|---|---|---|
| `audience_type` | N/A (Postgres ENUM) | N/A | Postgres ENUM type; snake_case per Postgres convention; replaces µservice manifest `audience` field; semantically accurate (type of audience the tenant represents). Not a crate name. |
| `merchant_status` | N/A (Postgres ENUM) | N/A | Postgres ENUM type; snake_case; tracks KYB + payments lifecycle state for merchant-capable tenants. |
| `payout_method` | N/A (Postgres ENUM) | N/A | Postgres ENUM type; snake_case; identifies the payout rail for a merchant tenant. |
| `tenant_lifecycle_state` | N/A (Postgres ENUM) | N/A | Postgres ENUM type; snake_case; canonical lifecycle state machine per §D-7. |
| `dr_pair_strategy` | N/A (Postgres ENUM) | N/A | Postgres ENUM type; snake_case; DR pairing strategy per ADR-0241 portfolio. |
| `bootstrap_tier` | N/A (Postgres ENUM) | N/A | Postgres ENUM type; snake_case; bootstrap-sequence position per ADR-0242 §D-5. |
| `provider_credential_mode_t` | N/A (Postgres ENUM) | N/A | Postgres ENUM type; snake_case + `_t` suffix per Postgres type-naming convention; scoped to LLM/provider API credentials per ADR-0255 §D-4. `_t` suffix disambiguates type from column name `provider_credential_mode`. |
| `Tenancy::Tenant` | N/A (Cedar entity) | N/A | Cedar v4.2 entity type; `Tenancy` namespace + PascalCase type name per Cedar v4.2 grammar. Represents a platform tenant. |
| `Tenancy::SubScope` | N/A (Cedar entity) | N/A | Cedar entity type; sub-scope within a tenant hierarchy per §D-2. |
| `Tenancy::Principal` | N/A (Cedar entity) | N/A | Cedar entity type; authenticated caller carrying a tenant + sub-scope. |
| `Tenancy::CrossTenantGrant` | N/A (Cedar entity) | N/A | Cedar entity type; explicit cross-tenant authorization grant per §D-6. |
| `Tenancy::Resource` | N/A (Cedar entity) | N/A | Cedar entity type; tenant-owned resource subject to authorization. |
| `oya-check-tenant-id-format` | N/A (check-family) | `check`.`tenant-id-format` | CI fitness-check per ADR-0105 Amendment 2 `oya-check-*` flat namespace; verifies tenant ID matches §D-1 regex at registration. |
| `oya-check-sub-scope-depth` | N/A (check-family) | `check`.`sub-scope-depth` | CI fitness-check; verifies sub-scope paths do not exceed 5-segment depth per §D-2. |
| `oya-check-no-audience-on-microservice` | N/A (check-family) | `check`.`no-audience-on-microservice` | CI fitness-check; verifies no µservice manifest carries the retired `audience` field per ADR-0244 §Context. |
| `oya-check-tenant-audience-coherence` | N/A (check-family) | `check`.`tenant-audience-coherence` | CI fitness-check; verifies `audience_type` column values match known enum variants; replaces retired `oya-check-audience-coherence`. |

---

## Change log

- **2026-05-20 (Wave-3-A cross-reference wiring):** Applied §D-3 DDL extension per ADR-0246-amendment + ADR-0257-amendment:
  - Added `policy_evaluation_mode_t` ENUM type (values: `library_first` | `network_only` | `library_first_with_attested_fallback`; default `library_first`).
  - Added `ontology_read_mode_t` ENUM type (values: `library_first` | `network_only` | `library_first_with_freshness_floor`; default `library_first`).
  - Added columns to `tenants` table: `policy_evaluation_mode` (enum, default `library_first`), `attested_fallback_threshold` (INTERVAL, default `'24 hours'`), `ontology_read_mode` (enum, default `library_first`), `freshness_floor` (INTERVAL, default `'5 seconds'`).
  - Added `policy_evaluation_network_opt_in` BOOLEAN + `policy_evaluation_network_opt_in_reasons` TEXT[] columns to `cloud_secrets.secret_references` via migration `microservices/cloud-secrets/migrations/0001_secret_references_policy_eval_opt_in.sql`.
  - Updated field documentation summary table with all new columns.

*End of ADR-0244.*
