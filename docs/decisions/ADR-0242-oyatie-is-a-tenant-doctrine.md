---
id: ADR-0242
status: Rejected
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
supersedes: []
amends:
  - ADR-0136-intelligence-as-single-microservice.md
  - ADR-0136-amendment (Foundry internal-only carve-out)
  - ADR-0220-consumer-intelligence-substrate.md
  - ADR-0239-amendment-intelligence-internal-scope-clarification-2026-05-18.md
  - ADR-0221-agentic-development-pipeline-hardening.md (§M-04 audience-of-microservice field)
superseded_by: []
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0010-regional-pack-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cursor-pagination-canonical.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0213-ecosystem-as-a-service-architecture.md
  - ADR-0215-multi-context-platform.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/tenant-model.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/policy-engine.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_bominal_inheritance_precedence
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_flat_product_catalog
  - feedback_automate_everything
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
purpose: >
  Establish `oyatie` as the canonical, first-class, reserved-namespace
  org-tenant of its own multi-tenant platform. Eliminate the
  "internal-vs-consumer" µservice audience distinction (ADR-0136-amendment,
  ADR-0220, ADR-0239 audience-as-µservice-scope framings) in favour of a
  uniform tenant model where every workload is a principal under a tenant,
  and `oyatie` is one tenant among many. No internal carve-outs, no
  bypass paths, no special audit-chain streams. Same Cedar gates, same
  DSAR cascade, same compliance machinery, same FinOps cost attribution
  for `oyatie` as for any customer tenant.
enforcement_status: advisory-until-tenant-bootstrap-lands
enforced_by:
  - oya gate validate oyatie-tenant-coherence
  - oya gate validate no-audience-on-microservice
  - oya gate validate reserved-namespace-protection
keystone_position: 1-of-14
---

# ADR-0242: `oyatie`-is-a-tenant doctrine

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive) landing as a single multispectrum-reviewed PR. Each
keystone references the others; partial acceptance is rejected because
the doctrines are mutually-reinforcing and produced together to avoid
the drift pattern that produced ADR-0220 → ADR-0239 amendment within
twelve days.

Enforcement is `advisory-until-tenant-bootstrap-lands`: the doctrine is
accepted in text now, but the CI lanes that enforce it move to BLOCKER
status only after:

1. `microservices/tenancy/` admits `oyatie` as a first-class tenant via
   bootstrap migration `0001_create_self_tenant.sql`.
2. `microservices/identity/` issues OIDC service-principals under
   `oyatie.*` sub-scopes.
3. `microservices/policy-engine/` (per ADR-0246) carries Cedar
   fragments scoped to `oyatie.*` principals.
4. `microservices/audit-chain/` provisions per-stream sealed audit log
   for `oyatie` tenant (and per-sub-scope rollups).

Until those four bootstrap items land, validators emit findings without
failing CI. Post-bootstrap, the lanes promote to BLOCKER.

## Date

2026-05-20.

## Context

### Prior portfolio state (pre-keystone)

The oyatie portfolio inherited from Bominal a doctrine that treated
"internal-platform" use cases as architecturally distinct from
"consumer-facing" use cases:

- **ADR-0136 (Foundry as single µservice, 2026-05-18)** consolidated six
  prior foundry candidates into one `microservices/foundry/` µservice
  but framed the µservice as serving the *internal retired external agent harness agentic
  development pipeline*.
- **ADR-0136-amendment (Foundry internal-only, 2026-05-18)** codified
  that Foundry serves "retired external agent harness agentic development toolchain; CI/CD
  orchestration; internal eval substrate; internal evidence collection"
  — explicitly *not* tenant-facing.
- **ADR-0220 (Consumer Intelligence Substrate, 2026-05-18)** created
  `microservices/intelligence/` as the *consumer-facing* AI substrate
  for B2B tenants and B2C personal users, with the explicit decision
  that its Alternative 2 ("One AI gateway for internal and consumer
  users") was rejected because "gateway unification hides audience
  differences."
- **ADR-0239 (Foundry internal scope clarification, 2026-05-18)** drew
  a sharp boundary: `microservices/foundry/` = INTERNAL only;
  `microservices/intelligence/` = CONSUMER only. Manifest `audience`
  field per ADR-0221 §M-04 was added to make this CI-enforceable.

Layered atop this, the **ADR-0221 §M-04 manifest-`audience` field**
required every µservice to declare `audience: INTERNAL | B2B-tenant |
B2C-consumer | DEVELOPER`. Foundry declared `INTERNAL`. Intelligence
declared `B2C-consumer`. Other µservices declared per their target
audience.

### What "internal carve-out" cost the portfolio

Operating internal use cases as audience-distinct µservices produced
recurring tax:

1. **Doubled doctrine surface.** Each capability (audit chain, cost
   attribution, DSAR, retention, Cedar policy, encryption-at-rest,
   incident response, observability rollup) had to be authored twice
   — once for the "internal" framing and once for the "consumer"
   framing — even when the underlying mechanism was identical.
2. **Audit-chain stream drift.** Foundry's audit chain (build evidence,
   eval receipts, multispectrum review verdicts, ADR drafts) emitted to
   a different stream than Intelligence's audit chain (prompt history,
   DSAR cascades, EU AI Act tier decisions). The streams used the same
   substrate but were schema-divergent in subtle ways. Tamper detection
   coverage was uneven.
3. **Bypass-path temptation.** Internal-only µservices acquired
   gradually a culture of "we don't need the same gates as consumer
   µservices" — an anti-pattern that's how Stripe's 2015 incident
   (since publicly discussed) was caused: an internal-only data
   pipeline that bypassed the Cedar-equivalent gate, written for
   speed during an incident, never reverted.
4. **Compliance carve-out fragility.** Internal-only Foundry was tacitly
   exempt from GDPR DSAR (no tenant data — except, in practice, every
   contributor's personal data sits in commit history, code review
   threads, evidence chains, multispectrum review verdicts). The
   "internal-only" framing made the DSAR question feel solved by
   exemption — but a future EU contributor's right-to-erasure under
   Article 17 would force a one-off engineering scramble, not a
   reusable DSAR cascade.
5. **Engineering velocity drag from policy duplication.** Every new
   Cedar fragment for Intelligence had a near-mirror version for
   Foundry that had to be authored, reviewed, and kept in sync. PR
   #143 close-out audit (`evidence/pr-143-close-out-plan-and-gap-audit-2026-05-18.json`)
   surfaced 9 such pairs already.
6. **Audit-chain leakage risk via co-location of the underlying
   substrate.** Foundry and Intelligence shared Milvus (ADR-0192),
   Wasmtime (ADR-0200), Cedar (ADR-0150), audit-chain (ADR-0028) per
   ADR-0220's "Shared substrate with Foundry" table — but enforced
   separation at the µservice + cell + tenant + Cedar boundary. The
   *audience-as-µservice-scope* framing implied stronger isolation than
   the actual implementation provided. The mismatch is where leaks
   originate.

### What every named hyperscaler reference actually does

The 2026 portfolio's `feedback_quality_performance_scalability_bar`
memory establishes the bar as "industry leaders — Stripe / Palantir /
Linear" plus "hyperscaler-grade." Every reference at that bar operates
as a *tenant of its own platform*, not as an internal carve-out:

- **Amazon (since the 2010-2014 internal-AWS migration).** amazon.com
  retail runs on AWS. The AWS Identity team are AWS IAM principals
  with appropriate scopes; they're audited the same way external
  enterprise tenants are audited. Werner Vogels has discussed this
  publicly (re:Invent 2019 keynote; "10 Lessons from 10 Years of AWS,"
  All Things Distributed blog 2016). The AWS Free Tier is partly
  funded by Amazon's own consumption — Amazon eats the bill.
- **Stripe.** Stripe processes Stripe's own corporate card spend
  through Stripe. Stripe's internal finance team uses Stripe Invoicing,
  Stripe Treasury (internal), and Stripe Payments. Stripe Engineering
  has confirmed this in Quora posts (2013) and in a Brandur Leach
  engineering blog 2014: "We use Stripe to bill Stripe; it's how we
  find bugs first."
- **Google (Borg → Kubernetes lineage).** Borg manages Borg's own
  control plane services. Internal Google teams are Borg-cell tenants
  the same as external Google Cloud customers (modulo earlier history;
  the modern shape since 2018-2020 is unified). Verma et al. "Borg,
  Omega, and Kubernetes" (CACM 2016) describe this convergence.
- **Apple (Apple Intelligence and Apple infrastructure).** Apple uses
  Apple Intelligence internally for Apple operations. Apple's internal
  developer tooling (Xcode-as-a-service, internal CI) runs on the same
  Apple-internal-cloud infrastructure that powers iCloud and Apple
  services. There's no "Apple Foundry" distinct from "Apple
  Intelligence" — Apple has one AI substrate, with audience-aware
  policy at the call boundary, not the µservice boundary.
- **Microsoft (post-One-Microsoft 2014 reorg).** Microsoft runs
  Microsoft 365 + Azure + GitHub + LinkedIn on Azure infrastructure.
  Microsoft IT acts as an Azure tenant. The Azure CTO has discussed
  this in Build 2023 + 2024 keynotes.
- **Palantir.** Palantir runs Palantir Foundry internally for Palantir's
  own data operations + product analytics. Palantir Apollo (their
  deployment system) deploys to Palantir-internal Foundry instances
  the same way it deploys to government and commercial Foundry
  instances. There's no Palantir-internal special path.
- **Cloudflare.** Cloudflare's own DNS, edge proxy, and analytics all
  run on Cloudflare's edge. Cloudflare uses Cloudflare to fight the
  DDoS attacks against Cloudflare. The Pingora blog (2022) and the
  R2 launch blog (2023) make this explicit.

The pattern is unambiguous: **mature platform companies operate as
tenants of their own platform.** Internal carve-outs are a
juvenile-platform symptom, not a hyperscaler shape.

### What "oyatie-is-a-tenant" actually means

The doctrine establishes:

1. The org `oyatie` is registered as a tenant in `microservices/tenancy/`
   the same way `tenant-acme-corp` or `tenant-customer-xyz` is registered.
2. Every internal use case (retired external agent harness / CI / eval / multispectrum review /
   ADR authoring / sovereign-pack regulator evidence emission / FinOps
   internal reporting / security / devrel / customer success / sales /
   marketing / finance) operates as a *principal* under the `oyatie`
   tenant (or under a dotted sub-scope of it).
3. Every µservice serves all tenants. The audience question — internal
   versus B2B versus B2C — is answered by *which tenant is calling*,
   evaluated at the request boundary by Cedar policy (ADR-0243), not by
   µservice identity.
4. There are no internal-only bypass paths. The same Cedar gate, audit
   emission, retention policy, DSAR cascade, encryption requirement, and
   cost-attribution rule applies whether the caller is `oyatie.foundry.ci-agent`
   or `tenant-customer-xyz.user-7421`.
5. The `oyatie` namespace is reserved at platform genesis. No customer
   tenant can be registered with `oyatie` or any case-fold or
   diacritic-normalised variant as their tenant ID.

### Why now (2026-05-20)

Three forcing functions:

- **The PR #143 → ADR-0239 amendment loop took 12 days** (2026-05-18 →
  2026-05-30). That drift cycle is the canonical signal that the prior
  doctrine framing isn't sticking; agents and contributors keep
  applying internal-vs-consumer language inconsistently because the
  underlying model is brittle. The `oyatie-is-a-tenant` doctrine
  removes the ambiguity by removing the distinction.
- **ADR-0240 (sovereign-cloud-per-regional-pack) and ADR-0241 (DR + BC
  portfolio policy)** both landed 2026-05-18 with `oyatie`-as-an-
  ordinary-tenant assumptions baked in. ADR-0240's `prohibited_egress`
  rules and ADR-0241's per-µservice `dr_tier` declaration both presume
  uniform tenant treatment. Without this keystone, ADR-0240's
  sovereign-data-class enforcement against `oyatie`'s internal data
  classes (e.g., source code, eval corpora, internal ADR drafts) is
  ambiguous.
- **The autonomous-masterplan goal (feedback_autonomous_implementation_artifacts).**
  The long-term goal of "Implement the masterplan runs without user
  intervention" requires that Foundry workflows operate as principals
  with deterministic policy gates. With `oyatie-is-a-tenant`, the
  workflows act as `oyatie.foundry.ci-agent` principals; without it,
  the workflows are special and require bespoke policy machinery.

## Decision

### D-1. `oyatie` is the canonical org-tenant slug

The tenant ID `oyatie` (exactly that spelling, lowercase, ASCII) is
**reserved at platform genesis** for the org operating the platform.

Justification for the literal slug:

- Matches AWS's `aws` IAM principal pattern (AWS uses `aws` for its
  internal control-plane principals; AWS docs refer to this in
  `arn:aws:iam::aws:` patterns for AWS-owned managed policies).
- Matches GCP's `google` resource hierarchy convention.
- Matches Azure's `microsoft` partition concept.
- Shorter than `oyatie-internal-ops` or `oyatie-platform-ops` (audit
  log readability + ARN-equivalent length budget).
- Distinguishes by orthography from customer tenant slugs (customer
  tenants use `tenant-` prefix or hash-suffix; `oyatie` is unmistakable
  in any log).

Reserved-namespace protection: the tenancy-substrate admission gate
refuses any registration request where the proposed tenant ID, after
case-fold + Unicode-normalisation (NFKC) + diacritic-strip, equals
`oyatie` or any of the following sub-scope roots:

- `oyatie` (root)
- `oya` (short alias; reserved to prevent typosquatting)
- `oyat`, `oyati`, `oyatie-*` (prefix family; reserved to prevent
  customer tenants from using `oyatie-foo` to imply oyatie affiliation)
- Any IDN-homograph variant of `oyatie` per Unicode TR36 confusables
  table

The reservation is enforced by Cedar fragment
`policy-engine/fragments/reserved-tenant-namespace.cedar` (advisory
during bootstrap; BLOCKER post-bootstrap).

### D-2. Dotted hierarchical sub-scope convention

Sub-scopes within the `oyatie` tenant use dotted-path notation:

```
oyatie                                  # root tenant
oyatie.platform-ops                     # platform operations team
oyatie.platform-ops.sre                 # SRE sub-team principals
oyatie.platform-ops.compliance          # compliance team principals
oyatie.foundry                          # internal dev pipeline scope
oyatie.foundry.ci-agent                 # the CI agent service principal
oyatie.foundry.eval-runner              # multispectrum review runner
oyatie.foundry.adr-drafter              # ADR-drafting workflow principal
oyatie.foundry.merge-queue              # merge queue controller
oyatie.security                         # security team
oyatie.security.incident-response       # incident response team
oyatie.devrel                           # developer relations
oyatie.finance                          # finance ops
oyatie.legal                            # legal ops
oyatie.dev.<engineer-id>                # per-engineer sandbox tenant
oyatie.preview.<pr-number>              # per-PR ephemeral preview tenant
oyatie.cell.<cell-id>                   # per-cell internal operations
```

Properties of dotted sub-scopes:

- **Inherit by default.** `oyatie.foundry.ci-agent` inherits `oyatie`'s
  jurisdiction, home_cell, capability flags, and Cedar policy unless
  the sub-scope explicitly overrides.
- **Roll up by default.** Audit events on `oyatie.foundry.ci-agent` are
  visible in `oyatie.foundry`'s audit stream (which rolls into
  `oyatie`'s).
- **Cost attribute by default.** Per-action cost attribution defaults
  to the deepest declared sub-scope; FinOps portal rolls up.
- **Cedar fragments may be sub-scope-specific.** A Cedar permit may
  apply to `oyatie.foundry.*` (glob) or to `oyatie.foundry.ci-agent`
  (exact).
- **Principals can carry only one sub-scope at a time.** A service
  principal's identity is its scope; cross-scope action requires an
  assume-role flow (per ADR-0244).

### D-3. No internal-only µservices

Every µservice (substrate or product) serves all tenants. The
internal-vs-consumer distinction is not a µservice property; it is a
*tenant* property and a *Cedar policy* result.

Concretely:

- The manifest `audience` field per ADR-0221 §M-04 is **removed**. The
  field is moved to the tenant model (per ADR-0244) and renamed
  `tenant.audience_type` to indicate intent. CI lane
  `oya-check-audience-coherence` is removed; replaced by
  `oya-check-tenant-audience-coherence` operating against the tenant
  table, not against µservice manifests.
- Foundry's role of "internal retired external agent harness pipeline" is preserved
  *operationally* — `oyatie.foundry.*` principals are the heaviest
  consumers — but Foundry as a *µservice with internal-only audience*
  dissolves (per ADR-0247 self-modification doctrine + the marketplace
  ADR which folds Foundry's BCs into Workflow Engine + Intelligence +
  audit-chain).
- Intelligence's role of "consumer AI substrate" is preserved
  *operationally* — `tenant-<customer>.*` principals are the heaviest
  consumers of the Consumer Brand Surface layer — but Intelligence's
  AI Substrate layer (per ADR-0255 rewrite) serves all tenants
  including `oyatie`.
- Every other µservice (Mail, Drive, Calendar, Messenger, Workflow
  Studio, HR, Payroll, Plugin App Store, Marketplace, Community, etc.)
  serves all tenants under Cedar policy.

### D-4. Uniform compliance machinery applied to `oyatie`

Every compliance and operational machinery applies equally to the
`oyatie` tenant:

- **DSAR (Data Subject Access Request) cascade.** If an oyatie
  engineer's personal data (e.g., commit history, code review
  comments, internal Slack-equivalent threads in `microservices/messenger/`)
  is subject to an erasure request under GDPR Article 17 / KR PIPA
  Article 36 / state-level laws, the cascade is the same primitive as
  for any tenant. The `oyatie` tenant has a DSAR contact (legal counsel
  on record) and a DSAR-response SLA.
- **Audit-chain emission.** Every state-changing action by an
  `oyatie.*` principal emits to the `oyatie` tenant's audit chain
  stream (per-sub-scope sub-streams; Merkle-sealed per ADR-0028
  inheritance + per-period cadence per Ontology PRD §"Audit + Compliance").
- **Retention policy.** `oyatie` tenant data follows the same per-
  jurisdiction retention as any tenant. Source code retention follows
  legal-hold-on-open-litigation conventions (per FRCP 37(e) +
  Sedona Conference Working Group 1). Audit emissions follow per-
  jurisdiction minimums (HIPAA 6+y if PHI ever touched; SOX 7y if
  pre-IPO finance; KR-FSS 3+y if financial-services data; etc.).
- **Cost attribution.** Every action's cost (LLM tokens, compute-
  hours, storage, network egress, third-party API call fees) bills
  to the deepest `oyatie.*` sub-scope. The internal FinOps portal
  shows per-sub-scope spend; budgets and chargebacks (between sub-
  scopes if the org chooses) work like any tenant.
- **Cedar policy evaluation.** Every action's authorisation goes
  through Cedar (per ADR-0243); `oyatie.*` principals have policy
  fragments granting their work; no bypass.
- **Data-class enforcement.** `oyatie` data has data classes (per
  ADR-0240 sovereign-cloud-overlay). Source code is data class
  `SOURCE_CODE_INTERNAL`; CI secrets are `CREDENTIAL_INTERNAL`; eval
  corpora are `EVAL_CORPUS_INTERNAL`; etc. Data residency rules
  per ADR-0240 + ADR-0049 apply.
- **DR tier declaration.** `oyatie.*` operational services declare
  `dr_tier` per ADR-0241. The `oyatie.foundry.ci-agent` workflow is
  T2 (< 1h RTO; some throughput degradation tolerable). The
  `oyatie.security.incident-response` toolchain is T1 (< 5min RTO;
  zero data loss). FinOps tag (ADR-0174) sustainability metrics
  per-cell apply equally.
- **Encryption.** `oyatie` data at rest is encrypted with `oyatie`-
  tenant-scoped KMS keys held in OpenBao per `microservices/cloud-secrets/`.
  provider-BYOK for `oyatie` is the same code path as provider-BYOK for any tenant
  (per the provider-BYOK SecretReference model documented in ADR-0255 Intelligence
  substrate rewrite).

### D-5. Bootstrap sequence

Bootstrap (the first deployment from zero hardware) follows a strict
ordering, with `oyatie`-tenant creation as one of the earliest steps:

| Step | Component | Bootstrap action | Required for next step? |
|---|---|---|---|
| 0 | Hardware + DNS + git host + container registry (Tier 0) | External setup (cloud provider account, GitHub Enterprise org, registry) | Yes |
| 1 | Bootstrap cell (Tier 1 — minimal K8s cluster) | `kubeadm init` + Cilium CNI + minimal etcd | Yes |
| 2 | `microservices/cloud-secrets/` (OpenBao) | Shamir-shared root unseal + service-account credentials seeded | Yes |
| 3 | `microservices/identity/` (Zitadel) | Initial admin + OIDC client for `oyatie` org | Yes |
| 4 | `microservices/tenancy/` | Migration `0001_create_self_tenant.sql` creates `oyatie` tenant row (hard-coded ID, reserved namespace) | Yes |
| 5 | `microservices/policy-engine/` | Bootstrap Cedar fragment set signed by org root key (per ADR-0246 bootstrap) | Yes |
| 6 | `microservices/audit-chain/` | Provisions `oyatie` tenant audit stream + Ed25519 signing key in OpenBao | Yes |
| 7 | `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` | Registers Tier 1 cell, marks it as bootstrap-class | Yes |
| 8 | `microservices/workflow-engine/` | Deploys minimal Workflow Engine; loads bootstrap workflows | Yes |
| 9 | First Foundry-equivalent workflow runs | `oyatie.foundry.ci-agent` principal performs first build of next-stage components | Bootstraps Tier 2 |
| 10 | Bootstrap → steady-state handoff | Bootstrap cell self-retires (per ADR-0247); Tier 2 control plane cell takes over | Bootstrap complete |

The chicken-and-egg problems are resolved by:

- **Step 4 hard-codes the `oyatie` tenant ID.** No need for tenancy to
  query itself; the migration writes a row with the literal value.
- **Step 5 bootstraps Cedar via signed-by-org-root-key fragments.**
  Org root key lives in a tier-0 HSM (e.g., a YubiKey HSM cluster
  held by the founding team). Subsequent Cedar fragment changes are
  signed by intermediate keys, which were themselves signed by the
  root key at step 5.
- **Step 6 provisions the audit-chain key.** Audit chain for steps 0-5
  is replayed from a bootstrap log file (signed at the time, ingested
  into the chain at step 6). This makes the bootstrap process itself
  auditable retroactively.

### D-6. Reserved-namespace enforcement detail

The reserved-namespace check (D-1) is implemented as:

```rust
// microservices/tenancy/src/reserved_namespace.rs

pub fn is_reserved(proposed_id: &str) -> ReservedResult {
    // Normalize: NFKC, lowercase, strip diacritics, replace
    // Unicode-confusable characters with canonical ASCII forms
    // per Unicode Technical Standard #39 (Unicode Security Mechanisms).
    let normalised = unicode_normalize_nfkc(proposed_id)
        .to_lowercase();
    let stripped = strip_diacritics(&normalised);
    let deconfused = remove_unicode_confusables(&stripped);

    let reserved_roots = ["oyatie", "oya", "oyat", "oyati"];

    for root in reserved_roots {
        if deconfused == *root
            || deconfused.starts_with(&format!("{}-", root))
            || deconfused.starts_with(&format!("{}.", root))
            || deconfused.starts_with(&format!("{}_", root))
        {
            return ReservedResult::Reserved {
                root: root.to_string(),
                normalised: deconfused,
            };
        }
    }

    ReservedResult::Available
}
```

The Cedar fragment that gates tenancy admission references this check:

```cedar
// policy-engine/fragments/reserved-tenant-namespace.cedar

forbid (
  principal,
  action == TenancyAction::"RegisterTenant",
  resource is Tenant
)
when {
  resource.id matches /^oyatie[-_.]/i
  || resource.id == "oyatie"
  || resource.normalized_id == "oyatie"  // post-NFKC
};
```

### D-7. The `oyatie` tenant's properties

The `oyatie` tenant row carries:

```yaml
tenant_id: "oyatie"
audience_type: "PLATFORM_OWNER"        # new enum value per ADR-0244
home_cell: "<bootstrap-or-control-plane-cell-id>"
dr_cell: "<paired-cell-id>"
jurisdiction:
  primary: "US-DE"                      # Delaware C-corp; adjust per actual incorporation
  data_residency_allowed:
    - "US"
    - "EU"                              # operates in EU subject to EU pack overlay
    - "KR"                              # operates in KR subject to KR pack overlay
sovereign_cloud_pack: null              # uses default pack matrix per cell
parent_tenant_id: null                  # root tenant
capabilities:
  can_pay: true                         # reserved: post-payments-certification
  can_receive: true                     # reserved: post-payments-certification
  can_settle: true                      # reserved: post-payments-certification
  can_facilitate_sub_merchants: true    # reserved: oyatie IS the platform facilitator
merchant_status: "platform_facilitator" # post-certification
payout_method: "internal"               # not applicable
tax_registrations: []                   # populated as oyatie's own tax nexus expands
dsar_contact:
  legal_owner: "council-legal"
  email: "dsar@oyatie.com"              # or equivalent
  response_sla_days: 30                 # GDPR Article 12 default
audit_streams:
  - "oyatie.root"                       # default rollup
  - "oyatie.foundry"                    # CI/dev sub-stream
  - "oyatie.security"                   # security sub-stream
  - "oyatie.finance"                    # finance sub-stream
  - "oyatie.platform-ops"               # platform-ops sub-stream
finops_cost_center: "oyatie-platform"
created_at: "<bootstrap-timestamp>"
created_by: "system:bootstrap-migration"
locked: true                            # cannot be deleted without ops-compliance approval
```

### D-8. Sandbox + preview tenants

For developer experience and CI:

- **Per-engineer sandbox tenant.** Every oyatie engineer has a personal
  sandbox tenant under `oyatie.dev.<engineer-id>` (e.g.,
  `oyatie.dev.jasonlee`). Properties: full platform shape, bounded
  resources, auto-teardown after 24h of inactivity. Used for local
  development against the real platform with isolated data.
- **Per-PR preview tenant.** Each PR triggers automatic creation of
  `oyatie.preview.<pr-number>` (e.g., `oyatie.preview.123`). A
  lightweight cell deploys for the PR's changes; the preview tenant
  lives there; reviewers can test the actual PR'd code against this
  preview. Auto-teardown on PR close (merge or abandon).
- **Per-CI-run ephemeral tenant.** Long-running CI jobs (e.g.,
  multispectrum review on a substantial change) get
  `oyatie.ci.<run-id>` for the duration of the job. Disposed on job
  completion.

These ephemeral tenants are subject to the same Cedar gates as the
parent `oyatie.*` tenants — bypass paths are prohibited.

## Alternatives considered

### Alt-1. Keep audience-as-µservice-scope (status quo from ADR-0136-amendment + ADR-0220 + ADR-0239)

Continue treating Foundry as "internal-only µservice" and Intelligence
as "consumer-only µservice"; each µservice declares its `audience` per
ADR-0221 §M-04.

**Pros:**

- Zero migration cost (already in place).
- Sharp visual separation in code review of "is this internal or
  consumer code."
- Familiar mental model for contributors coming from products that
  treat their internal tooling as off-the-shelf or external (e.g.,
  GitHub-as-customer-of-itself for issue tracking).

**Cons:**

- **Drift loop is already evidenced.** ADR-0220 → ADR-0239 amendment
  in 12 days shows the framing isn't sticking.
- **Doubled doctrine surface.** Cedar fragments, audit-chain schemas,
  DSAR cascades, retention rules, encryption policies all authored
  twice in mirror form.
- **Bypass-path temptation cultural.** "Internal-only" implicitly
  invites bypass under deadline pressure.
- **Contradicts every named industry reference.** Stripe / Palantir /
  Amazon / Google / Apple / Cloudflare / Microsoft all operate as
  tenants of their own platform.
- **Compliance carve-out fragility.** When an EU contributor under
  GDPR Article 17 requests erasure, a one-off engineering scramble
  is needed because the DSAR cascade was authored for "tenants" not
  for "the platform team itself."
- **Sovereign-cloud (ADR-0240) ambiguity.** ADR-0240's
  `prohibited_egress` for `oyatie`'s own data classes is undefined
  under audience-as-µservice-scope (is `SOURCE_CODE_INTERNAL` a
  Foundry concern with no sovereign-cloud override? Or a `oyatie`
  tenant concern with full sovereign-cloud machinery?).

**Rejected** because the cons are unbounded coordination cost + every
named industry reference disagrees + the drift loop demonstrates the
model is brittle. The pros are all preserved (separation, mental
model, comprehensibility) under the chosen Alt-5 with tenant + sub-
scope conventions.

### Alt-2. Make tenant model first-class but keep `oyatie` distinct in special-case logic

Adopt a tenant model where customer tenants are first-class but
`oyatie` is hard-coded as a special-case "platform owner" outside the
tenant model. Internal tooling continues to bypass the tenant gates.

**Pros:**

- Smaller migration cost than full uniformity (Alt-5).
- Familiar from many SaaS codebases (where `*_admin` user accounts
  exist outside the regular user model).

**Cons:**

- **Same drift loop.** Special-case logic for `oyatie` is exactly the
  internal carve-out we want to eliminate.
- **Audit-chain stream divergence.** `oyatie` would need its own
  schema, separate from tenant audit streams; tamper-detection coverage
  uneven.
- **DSAR machinery duplicated.** GDPR / KR PIPA cascade has to be
  authored twice — once for tenants, once for the `oyatie` special
  case.
- **Compliance evidence non-uniform.** Regulators (CSAP, SOC 2, ISO
  22301) prefer uniform evidence packets; "the platform owner is
  outside the system" is a regulator red flag.
- **Sovereign-cloud violation risk.** Special-case `oyatie` data may
  egress to unsanctioned providers because the data-class enforcement
  was authored for "tenant data, not platform-owner data."

**Rejected** because the special-case carve-out reproduces the original
problem at a deeper layer.

### Alt-3. Merge Foundry and Intelligence into one consumer-facing brand (Apple Intelligence model)

Dissolve the Foundry / Intelligence distinction by merging both into a
single consumer-facing µservice ("Apple Intelligence"-style), and treat
internal use cases as one of the audience-tags the merged µservice
supports.

**Pros:**

- Eliminates the Foundry / Intelligence distinction outright.
- Matches Apple Intelligence shape closely.
- Reduces µservice count by one.

**Cons:**

- **Doesn't address audience-as-µservice-scope at the portfolio
  level.** Other µservices (Mail, Drive, etc.) still face the
  internal-vs-consumer question; this Alt only addresses two.
- **Apple Intelligence has its own internal-team consumers but doesn't
  conflate them with brand-surface end-users.** Apple's internal teams
  use Apple Intelligence via internal-only product surfaces (Xcode
  intelligence, Apple-internal Slack equivalent); externally, Apple
  Intelligence is the consumer brand. Apple operates as a tenant of
  its own platform with audience-aware policy at the call boundary,
  not by merging surfaces.
- **Loses the substrate-vs-product separation** that ADR-0245 will
  formalise.

**Rejected** because it solves a narrower problem than the keystone
needs to solve.

### Alt-4. Flat per-team tenants (oyatie-ci, oyatie-eval, oyatie-finance, etc., no hierarchy)

Adopt the `oyatie-is-a-tenant` doctrine but use flat sub-tenants
(`oyatie-ci`, `oyatie-eval`, `oyatie-finance`, ...) rather than dotted
hierarchical sub-scopes.

**Pros:**

- Simpler than hierarchical sub-scope inheritance + rollup logic.
- Each team's tenant is independent; no inheritance bugs.

**Cons:**

- **Loses inheritance default.** Every sub-team has to redeclare
  jurisdiction, home_cell, capability flags, Cedar policy. Drift
  guaranteed.
- **Loses rollup default.** Audit streams + cost attribution would
  need explicit cross-tenant rollup queries; FinOps portal more
  complex.
- **Loses cross-team coordination.** A flat model can't express
  "anyone in `oyatie.*` can perform X" — has to enumerate every
  sub-tenant.
- **Doesn't match AWS IAM principal paths** which are hierarchical for
  exactly these reasons (AWS Organizations + OU + Account + Role
  path hierarchy).

**Rejected** because flat hierarchy gives up the inheritance and
rollup that make the doctrine practical at scale.

### Alt-5. `oyatie-is-a-tenant` with dotted hierarchical sub-scopes ← **CHOSEN**

The selected alternative, fully specified in §Decision.

**Pros:**

- **Matches every named industry reference** (Amazon / Stripe /
  Google / Apple / Microsoft / Palantir / Cloudflare).
- **Eliminates audience-as-µservice-scope.** Audience moves to
  tenant; µservices serve all tenants.
- **Inheritance + rollup defaults make practical operation tractable.**
- **Reserved namespace** prevents typosquatting / impersonation.
- **Bootstrap sequence is well-defined** with audited replay of
  pre-audit-chain steps.
- **Compliance machinery is dogfooded** — the same DSAR, retention,
  audit, encryption applied to `oyatie` is applied to every tenant,
  forcing the machinery to be production-grade.
- **Closes the drift loop** of ADR-0220 → ADR-0239 by removing the
  framing that drift was anchored to.

**Cons:**

- **Bounded one-time migration cost.** Existing `audience` fields must
  be removed from manifests (~46 µservices). Cedar fragments authored
  for "internal-only" and "consumer-only" must be reauthored as
  tenant-scoped. Audit-chain stream schemas must be unified.
- **Reserved-namespace logic is non-trivial.** Unicode confusables +
  NFKC normalisation + diacritic stripping must be implemented
  correctly. Mitigation: well-tested library (`unicode-security`
  crate); Unicode Technical Standard #39 conformance.
- **Bootstrap sequence is longer than the prior model.** Tradeoff
  accepted because the deterministic sequence is a small one-time
  cost.

**Accepted** as the foundational keystone. The cons are bounded one-
time costs; the pros include closing a drift loop and matching every
named industry reference.

## Consequences

### Positive

1. **Drift loop closed.** No more "is this µservice internal or
   consumer?" because the question is malformed. The question becomes
   "which tenant is this principal acting under?" — answerable by
   inspecting the principal's sub-scope.
2. **Unified compliance machinery.** DSAR / retention / encryption /
   audit-chain / cost-attribution / Cedar gates are authored once and
   applied uniformly. No `oyatie` carve-out.
3. **Dogfooding at the platform level.** The same machinery that
   serves customer tenants serves `oyatie`. Bugs surface during oyatie
   internal use before they reach customer tenants.
4. **Sovereign-cloud (ADR-0240) cleanly applies.** `oyatie`'s own data
   classes (source code, eval corpora, internal docs) are subject to
   per-pack sovereign-cloud-overlay enforcement. KR-resident
   contributors' personal data stays in the KR pack; EU GDPR Article
   17 erasure of an EU contributor's data is mechanically the same as
   a customer's DSAR.
5. **DR portfolio (ADR-0241) cleanly applies.** `oyatie.*` services
   declare `dr_tier`; `oyatie.security.incident-response` is T1;
   `oyatie.foundry.ci-agent` is T2. Per-µservice drill cadence
   applies uniformly.
6. **Hyperscaler-shape achieved.** Matches Amazon / Stripe / Google /
   Apple / Microsoft / Palantir / Cloudflare. Closes the
   feedback_quality_performance_scalability_bar requirement.
7. **Autonomous-masterplan-execution unlocked.** Foundry workflows
   acting as `oyatie.foundry.ci-agent` principals can modify the
   platform under deterministic Cedar policy. Per ADR-0247.
8. **Sub-scope cost attribution.** FinOps portal shows
   `oyatie.foundry` vs `oyatie.platform-ops` vs `oyatie.security`
   spend without bespoke reporting. Per-team budgets enforceable.

### Negative

1. **One-time migration cost.** ~46 µservice manifests change
   (`audience` field removed). Bounded; one ChangeSet executes it.
2. **Bootstrap sequence is rigorous.** 10 steps from zero hardware
   to steady state. Each step audited. Drift potential during
   bootstrap is non-zero; mitigated by retroactive audit replay
   (step 6 ingests the bootstrap log).
3. **Reserved-namespace check requires Unicode discipline.**
   Mitigation: vendored `unicode-security` crate; TR#39 conformance
   test set; clearly-named module
   `microservices/tenancy/src/reserved_namespace.rs`.
4. **Sub-scope hierarchy depth bounded.** Cedar policy evaluation
   complexity grows with depth; recommend max 4 levels
   (`oyatie.foundry.eval-runner.shard-3` is legal but discouraged).

### Operational

1. **New CI lanes (advisory until bootstrap; BLOCKER post-bootstrap):**
   - `oya-check-oyatie-tenant-coherence` — verifies `oyatie` tenant
     row exists post-bootstrap; verifies sub-scope hierarchy validity.
   - `oya-check-reserved-namespace-protection` — verifies the
     reserved-namespace Cedar fragment is loaded.
   - `oya-check-no-audience-on-microservice` — verifies no µservice
     manifest declares an `audience` field.
2. **Tenancy substrate** adds migration `0001_create_self_tenant.sql`
   + reserved-namespace admission gate.
3. **Identity substrate** issues OIDC service-principals under
   `oyatie.*` sub-scopes (Zitadel project + service-account schema).
4. **Cedar fragment library** seeds `oyatie.*` policy fragments at
   bootstrap; subsequent fragments evolve under the self-modification
   doctrine (ADR-0247).
5. **Audit-chain substrate** provisions per-sub-scope streams +
   rollup views.
6. **FinOps portal** adds `oyatie.*` cost-center hierarchy.
7. **Observability dashboards** add `oyatie.*` tenant filters; pre-
   existing dashboards continue to work because the underlying
   metrics now carry tenant labels.

### Sustainability

- No direct sustainability impact. Indirect benefit: per-sub-scope
  FinOps visibility lets `oyatie` track its own carbon footprint
  per-team, enabling self-imposed sustainability budgets matching
  the sustainability tag per ADR-0174.

### Compliance

- **GDPR / KR PIPA / state-level privacy laws** apply to `oyatie`
  contributor data the same as to tenant data. Mitigates risk of
  one-off scrambles when erasure requests arrive.
- **SOC 2 / ISO 27001 / ISO 22301** evidence is uniform across
  tenants including `oyatie`. Auditor cannot ask "is the platform
  owner outside scope?" — answer is "no, oyatie is in scope as a
  tenant."
- **EU AI Act Article 17 high-risk classification** applies to
  `oyatie`'s use of AI for code generation, eval, multispectrum
  review if those workflows touch tenant-data-class material. Tier
  declarations follow ADR-0144.

## Implementation surface

The following artifacts are required for this keystone to be considered
implemented:

| Artifact | Status |
|---|---|
| `/specs/tenant-model.json` | NEW — derived from §D-7 |
| `/specs/platform-architecture.json` (this keystone's `platform.tenancy` section) | NEW — derived from §D |
| `microservices/tenancy/migrations/0001_create_self_tenant.sql` | NEW |
| `microservices/tenancy/src/reserved_namespace.rs` | NEW |
| `microservices/identity/src/oyatie_service_principals.rs` | NEW |
| `microservices/policy-engine/fragments/reserved-tenant-namespace.cedar` | NEW |
| `microservices/policy-engine/fragments/oyatie-foundry-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/oyatie-platform-ops-permits.cedar` | NEW |
| `microservices/policy-engine/fragments/oyatie-security-permits.cedar` | NEW |
| `microservices/audit-chain/src/oyatie_stream_provisioner.rs` | NEW |
| `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` | UPDATE — bootstrap cell lifecycle owner after ADR-0333 |
| `microservices/finops-portal/src/oyatie_cost_center.rs` | NEW |
| `microservices/observability/dashboards/oyatie-tenant.md` | NEW |
| Removal of `audience` field from ~46 µservice manifests | SWEEP |
| Removal of CI lane `oya-check-audience-coherence` | SWEEP |
| Addition of CI lanes `oya-check-oyatie-tenant-coherence` + `oya-check-reserved-namespace-protection` + `oya-check-no-audience-on-microservice` | NEW |
| `docs/standards/oyatie-tenant-bootstrap.md` | NEW — full runbook |
| `docs/runbooks/oyatie-dsar-cascade.md` | NEW — DSAR procedure for `oyatie` contributor data |
| `docs/runbooks/oyatie-bootstrap-recovery.md` | NEW — bootstrap replay procedure for catastrophic loss |

## Verification

- [ ] `microservices/tenancy/` has migration `0001_create_self_tenant.sql`; running it on an empty database yields a row with `tenant_id = "oyatie"` and the §D-7 properties.
- [ ] `microservices/tenancy/src/reserved_namespace.rs` exists and passes test cases:
  - `is_reserved("oyatie") == Reserved`
  - `is_reserved("OYATIE") == Reserved` (case-fold)
  - `is_reserved("оyatie") == Reserved` (Cyrillic 'о' confusable; TR#39)
  - `is_reserved("oyatie-corp") == Reserved` (prefix family)
  - `is_reserved("oyatie.com") == Reserved` (dot is also a reserved separator)
  - `is_reserved("oyat") == Reserved` (short alias)
  - `is_reserved("acme-corp") == Available`
  - `is_reserved("tenant-acme") == Available`
- [ ] `oya gate validate oyatie-tenant-coherence` exits 0 on a bootstrapped instance.
- [ ] `oya gate validate no-audience-on-microservice` exits 0 (no µservice manifest declares `audience`).
- [ ] `oya gate validate reserved-namespace-protection` exits 0 (Cedar fragment loaded).
- [ ] Audit-chain emits a `OyatieTenantBootstrapEvidence` event at bootstrap completion, signed by the bootstrap key.
- [ ] FinOps portal shows `oyatie.*` cost center hierarchy.
- [ ] DSAR cascade test against an oyatie contributor's audit-chain entries returns expected erasure plan within 30 days SLO.
- [ ] DR drill for `oyatie.security.incident-response` (T1) completes within 5 minutes.
- [ ] ADR-0136-amendment + ADR-0220 + ADR-0239 frontmatter updated with `superseded_by: [ADR-0242, ADR-0244]` (per the amendment pattern in ADR-0145).

## References

### Industry sources

- **Werner Vogels, "10 Lessons from 10 Years of AWS" (All Things Distributed, 2016).** Documents Amazon's transition to using AWS as a customer of itself.
- **AWS Builders' Library — "Building dashboards for operational visibility."** Documents the "use your own product" principle.
- **AWS re:Invent 2019 keynote (Werner Vogels).** "Amazon.com runs on AWS" architectural review.
- **Stripe Engineering — "How Stripe Uses Stripe."** Internal-blog → public discussions on Quora 2013; reaffirmed in Brandur Leach posts 2014-2018.
- **Pat Helland, "Life Beyond Distributed Transactions" (2007).** Theoretical foundation for tenant-as-primitive.
- **Eric Brewer, "Towards Robust Distributed Systems" (PODC 2000).** CAP theorem context.
- **Google CRE blog series on "tenant isolation in multi-tenant systems."** Google's perspective on internal vs external tenancy.
- **Apple WWDC 2024 — Apple Intelligence keynote.** Documents Apple's "one substrate, audience-aware brand surface" architecture.
- **Palantir Apollo product page (palantir.com/platforms/apollo).** Internal use of Apollo to deploy Apollo.
- **Cloudflare Engineering blog — "Building Pingora" (2022).** Cloudflare's edge runs on Cloudflare.
- **Microsoft Build 2024 keynote.** Microsoft IT as Azure tenant.
- **Salesforce Trailhead — "Multi-tenant architecture."** Salesforce as a tenant of Salesforce Trust Cloud.
- **Verma et al., "Borg, Omega, and Kubernetes" (CACM 2016, vol. 59 no. 5).** Documents Google's internal-team tenancy in Borg/Omega/Kubernetes.
- **Unicode Technical Standard #39 (UTS #39) — Unicode Security Mechanisms.** Confusable detection + identifier normalisation.
- **Unicode Technical Report #36 (UTR #36) — Unicode Security Considerations.** IDN homograph attacks.

### Regulatory sources

- **GDPR Article 17 (Right to Erasure).** Applies to `oyatie` contributor personal data.
- **GDPR Article 12 (Modalities for the exercise of the rights of the data subject).** DSAR response SLA.
- **KR PIPA Article 36 (정보주체의 권리 — Information Subject's Rights).** Erasure equivalent.
- **CSAP (Cloud Security Assurance Program) v3.1.** Korean regulator framework requiring uniform tenant treatment.
- **ISO 22301:2019 — Security and resilience — Business continuity management systems.** Tenant inclusion in business continuity scope.
- **SOC 2 Type II Trust Service Criteria — CC1.4.** "The entity demonstrates a commitment to attract, develop, and retain competent individuals in alignment with objectives" — interpreted by auditors as requiring uniform compliance treatment.
- **FRCP 37(e) — Failure to Preserve Electronically Stored Information.** Legal hold supersedes retention sunset.
- **Sedona Conference Working Group 1 — "The Sedona Principles" (3rd ed.).** eDiscovery + legal hold authority.

### Internal portfolio ADRs

- **ADR-0009 — Cell architecture per-tenant per-region.** Cell-level isolation primitive for `oyatie` tenant.
- **ADR-0010 — Regional pack architecture.** `oyatie` operates across packs.
- **ADR-0049 — Cross-region replication + residency.** Applies to `oyatie` data.
- **ADR-0105 — Thirteen-layer canonical enum.** Layer rules unchanged.
- **ADR-0128 — Hyperscaler architecture invariants.** Doctrine alignment.
- **ADR-0131 — Per-microservice flat layout.** Layout unchanged.
- **ADR-0132 — No-grouping forward policy.** No grouping µservices created.
- **ADR-0136 — Foundry as single µservice.** Foundry's internal-only framing is amended (Foundry dissolves per ADR-0247).
- **ADR-0136 amendment — Foundry internal-only.** Amended.
- **ADR-0145 — Inter-microservice communication reform.** Direct gRPC + 3 invariants pattern continues; `oyatie.*` principal calls follow same rules.
- **ADR-0150 — Cedar policy engine.** `oyatie` policy fragments live in policy-engine.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Both gate `oyatie` actions.
- **ADR-0211 — In-house Rust-primary tech stack.** Tenancy substrate is Rust.
- **ADR-0212 — Buildability doctrine.** This ADR is itself a deliverable artifact.
- **ADR-0213 — Ecosystem-as-a-service architecture.** Ecosystem includes oyatie as a participant.
- **ADR-0215 — Multi-context platform.** `oyatie` tenant is one context; customer tenants are others.
- **ADR-0218 — Tenant granular control surface.** `oyatie` admin surface mirrors tenant admin surface.
- **ADR-0220 — Consumer Intelligence Substrate.** Amended.
- **ADR-0221 — Agentic development pipeline hardening.** §M-04 audience-of-µservice field amended.
- **ADR-0239 — Foundry internal scope clarification.** Amended.
- **ADR-0240 — Sovereign cloud per regional pack.** Applies to `oyatie` data classes.
- **ADR-0241 — DR + business-continuity portfolio policy.** Applies to `oyatie.*` services.
- **ADR-0243 — Cedar as universal gate (keystone #2 — companion).**
- **ADR-0244 — Tenant as universal scoping primitive (keystone #3 — companion).**
- **ADR-0245 — Substrate vs Product layering (keystone #4 — companion).**
- **ADR-0246 — Policy-engine substrate promotion (keystone #5 — companion).**
- **ADR-0247 — Self-hosting / self-modification doctrine (keystone #6 — companion).**
- **ADR-0248 — Amazon-shape cellular architecture (keystone #7 — companion).**

### Auto-memory feedback

- `feedback_oyatie_is_a_tenant_doctrine` — NEW, captures this keystone for future agent context.
- `feedback_bominal_inheritance_precedence` — applies; this ADR overrides Bominal's audience-as-µservice-scope inheritance.
- `feedback_quality_performance_scalability_bar` — reinforced; hyperscaler-grade.
- `feedback_autonomous_implementation_artifacts` — reinforced; enables autonomous masterplan execution.
- `feedback_flat_product_catalog` — preserved.
- `feedback_automate_everything` — reinforced.
- `feedback_workflow_objectgraph_adapter_layer` — retired per ADR-0145; not relevant here.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in the pre-keystone exploration
(2026-05-20 session record), every architectural decision in this ADR
is attributed to a named hyperscaler pattern + source + anti-pattern
avoided. Required appendix.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (oyatie as canonical org-tenant slug) | "Eat-Your-Own-Dogfood at Platform Level" | Vogels 2016 "10 Lessons"; Stripe Engineering Quora 2013; Apple WWDC 2024 keynote; Palantir Apollo product docs | "Internal Carve-Out" — bypass paths for platform-owner ops |
| D-1 reserved-namespace protection | "Reserved Identifier Namespace" + "IDN Homograph Defence" | AWS `arn:aws:iam::aws:` reserved partition; UTS#39 Unicode Security; UTR#36 Security Considerations | "Typosquatting Tenant Impersonation" — third-party registers `oyatie-foo` to imply affiliation |
| D-2 (dotted hierarchical sub-scopes) | "Hierarchical Principal Path" | AWS IAM principal ARN paths; GCP IAM resource hierarchy; Azure RBAC scope hierarchy | "Flat Namespace Drift" — inheritance + rollup require explicit cross-namespace queries |
| D-3 (no internal-only µservices) | "Unified Multi-Tenant Substrate" | Salesforce multi-tenant architecture; AWS shared-substrate model; Microsoft 365 multi-tenant Exchange Online | "Audience-As-Service-Scope" — explicitly retired by every named hyperscaler reference |
| D-4 (uniform compliance machinery) | "Dogfooded Compliance Pipeline" | Stripe SOC 2 includes Stripe's internal use; AWS Audit Manager covers AWS-on-AWS; Microsoft 365 includes Microsoft IT | "Compliance Carve-Out" — platform owner outside audit scope (regulator red flag) |
| D-5 (bootstrap sequence) | "Audited Bootstrap Replay" | rustc stage0 bootstrap; Kubernetes kubeadm certificate chain; Certificate Transparency log bootstrap | "Untraceable Genesis" — original deployment lacks audit trail |
| D-6 (reserved-namespace enforcement) | "Defence-in-Depth via Cedar Fragment" | AWS Service Control Policy enforcing partition; GCP Org Policy constraints | "Application-Layer-Only Check" — bypass via direct database write |
| D-7 (`oyatie` tenant properties) | "First-Class Platform-Owner Account" | AWS `aws` system account; GCP `google` system project; Microsoft "First-Party Tenant" pattern in Azure AD | "Implicit Platform Account" — undocumented service-principal sprawl |
| D-8 (sandbox + preview tenants) | "Ephemeral Tenant Pattern" | Vercel preview deployments; Stripe test mode; Heroku review apps | "Production-Only Testing" — risk to live tenants from CI tests |

---

## Appendix B: Worked example — DSAR for an EU-resident oyatie contributor

To illustrate that the compliance machinery is genuinely uniform (not
just claimed), here is a worked example.

**Scenario:** An oyatie engineer based in Berlin (`oyatie.foundry.engineer.<id>`),
who leaves the company and exercises GDPR Article 17 right-to-erasure
six months later. They request erasure of all personally-identifying
data the company holds about them.

**DSAR cascade under `oyatie-is-a-tenant` doctrine:**

1. **Intake.** DSAR submission lands at the standard tenant DSAR
   intake endpoint (`microservices/governance/src/dsar_intake.rs`).
   The endpoint accepts the request, validates the requester's
   identity (per ADR-0188 passkey/WebAuthn), and records the request
   in the audit chain as `DsarRequest` event under the requester's
   sub-scope.
2. **Cedar gate.** Policy fragment evaluates: is this request from a
   subject permitted to erasure (yes — Article 17 right)? Are there
   overriding holds (e.g., legal hold for ongoing litigation, SEC
   17a-4 retention for financial records)? If holds, response is
   acknowledged-but-held; if no holds, cascade proceeds.
3. **Cascade enumeration.** The DSAR cascade engine (per Ontology
   IP-013) enumerates every `ObjectType` row across every µservice
   that may contain the subject's identifier. For a Berlin engineer,
   this includes:
   - `microservices/identity/`: account record, OIDC tokens, login
     history.
   - `microservices/audit-chain/`: every emission with the subject as
     actor or principal.
   - `microservices/observability/`: trace + log entries.
   - `microservices/finops-portal/`: cost-center attribution rows.
   - Workflow Engine: workflow execution history where subject was a
     human-in-loop approver.
   - Source control: commit history, PR comments, review threads.
   - `microservices/messenger/`: internal team chat history.
   - `microservices/mail/`: internal email records.
   - `microservices/calendar/`: internal calendar entries.
   - `microservices/recordings/`: meeting recordings the subject
     participated in.
4. **Per-source erasure plan.** For each source, the cascade
   determines:
   - Hard-delete? (e.g., OIDC tokens) — yes.
   - Pseudonymise? (e.g., commit author retained but personal email +
     name replaced with hash) — yes per FRCP 37(e) preservation
     compatibility.
   - Tombstone? (e.g., audit-chain entries — Merkle-sealed, cannot be
     deleted; subject's identifier replaced with hash; original Merkle
     proof retained for tamper detection) — yes.
   - Retain under legal hold? (e.g., commits under active litigation
     hold) — yes, with subject notification of hold reason.
5. **Execution.** Workflow Engine durably executes the plan; each
   step emits to audit chain (under a per-DSAR sub-stream).
6. **Confirmation.** Within 30 days of intake (GDPR Article 12), the
   subject receives a confirmation listing what was erased, what was
   pseudonymised, what was tombstoned, and what was held (with hold
   reason).

**Why this is uniform:** the entire flow above is identical to what
happens when a customer tenant's user exercises Article 17. The only
difference is the `tenant_id` carried through the cascade:
`oyatie.foundry.engineer.<id>` vs `tenant-customer-xyz.user-<id>`.
Same machinery, same SLO, same audit trail.

Under the prior internal-vs-consumer model, this cascade would have
required:

- A one-off engineering scramble to figure out which "internal" data
  stores hold the subject's data.
- Bespoke erasure scripts (untested, written under time pressure).
- A separate compliance report (since the standard tenant DSAR machinery
  didn't apply to internal contributors).
- Risk of missing a data store (e.g., the eval-corpus µservice tagged
  internal-only that the cascade didn't enumerate).

The keystone closes that risk by construction.

## Naming justification

Per `feedback_naming_justification`: every new name introduced by this ADR carries a one-line BNF v4.1 + ADR-0105 13-layer conformance justification.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|---|---|---|---|
| `oyatie` (tenant slug) | N/A (reserved root) | N/A | Platform-owner reserved tenant slug; top of reserved-namespace family per §D-1 + §D-6; registered at platform genesis; locked against deletion. Not a crate name. |
| `oya` (reserved root) | N/A | N/A | Short-form reserved root prefix; `oya-*` crates are platform-owned per ADR-0056; reserved against tenant registration. |
| `oyat` (reserved root) | N/A | N/A | Prefix variant reserved to prevent near-miss homograph registration. |
| `oyati` (reserved root) | N/A | N/A | Prefix variant reserved to prevent near-miss homograph registration. |
| `oya-check-oyatie-tenant-coherence` | N/A (check-family) | `check`.`oyatie-tenant-coherence` | CI fitness-check per ADR-0105 Amendment 2 `oya-check-*` flat namespace; verifies `oyatie` tenant row properties per §D-7. |
| `oya-check-reserved-namespace-protection` | N/A (check-family) | `check`.`reserved-namespace-protection` | CI fitness-check; verifies the reserved-namespace Cedar fragment is active and blocks registration of `oyatie`/`oya`/`oyat`/`oyati` roots. |
| `oya-check-no-audience-on-microservice` | N/A (check-family) | `check`.`no-audience-on-microservice` | CI fitness-check; verifies no µservice manifest carries the retired `audience` field (retired by ADR-0244). |
| `microservices/tenancy/migrations/0001_create_self_tenant.sql` | N/A (migration file) | N/A | Postgres migration; per-µservice counter convention `^[0-9]{4}_[a-z][a-z0-9_]*\.sql$` per ADR-0131; creates `oyatie` bootstrap tenant row. |
| `microservices/tenancy/src/reserved_namespace.rs` | N/A (Rust module) | N/A | Rust module; snake_case file per Rust module-naming idiom; implements UTS#39 reserved-namespace admission logic. |
| `microservices/policy-engine/fragments/reserved-tenant-namespace.cedar` | N/A (Cedar fragment) | N/A | Cedar fragment file; kebab-case + `.cedar` extension per Cedar v4.2 convention; enforces reserved-namespace reservation at admission. |
| `oyatie.root` / `oyatie.foundry` / `oyatie.security` / `oyatie.finance` / `oyatie.platform-ops` | N/A (audit stream names) | N/A | Audit stream name family; `oyatie.` prefix marks platform-owned streams per ADR-0242 reserved-namespace doctrine; dotted sub-scope path per ADR-0244 §D-2. |

---

*End of ADR-0242.*
