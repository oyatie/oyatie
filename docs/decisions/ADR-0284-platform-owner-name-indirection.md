---
id: ADR-0284
status: Rejected
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-brand
  - council-legal
  - council-security
  - ops-compliance
  - ops-sre-reliability
  - axis-tenancy
  - axis-identity
  - axis-audit-chain
  - axis-i18n
supersedes: []
amends:
  - ADR-0242-oyatie-is-a-tenant-doctrine.md (§D-1 reserved literal `oyatie` slug — now sourced from a single named constant)
superseded_by: []
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0063-doc-coverage-enforcement.md
  - ADR-0064-canonical-base-plus-localization-overlay.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0206-i18n-substrate-fluent-icu.md
  - ADR-0211-in-house-tech-stack-policy.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0216-open-integration-and-migration-out-policy.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0249-multi-category-marketplace-doctrine.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/platform-constants.json
  - /specs/tenant-model.json
  - /specs/microservices/tenancy.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/audit-chain.json
  - /specs/markdown-retirement-policy.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_bominal_inheritance_precedence
  - feedback_no_silent_regression
  - feedback_quality_performance_scalability_bar
  - feedback_canonical_base_localization
  - feedback_naming_justification
  - feedback_automate_everything
  - feedback_autonomous_implementation_artifacts
doc_class: Architecture-Decision-Record
enforcement_status: advisory-until-constant-crate-lands
enforced_by:
  - oya gate validate platform-owner-constant-indirection
  - oya gate validate no-hardcoded-platform-owner-slug
  - oya gate validate platform-owner-brand-vs-slug-separation
keystone_position: tier-1-lockdown
purpose: >
  Introduce a single named source of truth (`PLATFORM_OWNER_TENANT_SLUG`)
  for the platform-owner tenant identifier that ADR-0242 hardcoded as the
  literal string `oyatie` across Cedar fragments, audit-chain code,
  tenancy bootstrap, identity provisioning, FinOps cost-center hierarchy,
  observability dashboards, and migration ledgers. Without this
  indirection, any rebrand of the platform owner becomes a multi-day
  full-portfolio search-and-replace operation crossing hundreds of files,
  Cedar policies, sealed audit-chain rows, signed Merkle roots, and
  deployment manifests — a catastrophe pattern the keystone bundle
  cannot tolerate. The decision also separates the platform-owner brand
  *display name* (i18n-able, surface-localised, marketing-controlled)
  from the platform-owner *slug* (ASCII, lowercase, stable, reserved),
  matching how Apple Intelligence brand surfaces vary while the
  underlying tenant slug stays fixed.
---

> **Disposition light-edit (2026-08-06):** Keep Rejected: Platform-owner-name indirection — thin; fold into tenant doctrine if needed

# ADR-0284: Platform-Owner-Name Indirection

## Status

Proposed — 2026-05-20.

Tier-1 lockdown ADR. Filed as a follow-on hardening of ADR-0242's
`oyatie`-is-a-tenant doctrine. ADR-0242 §D-1 reserved the literal slug
`oyatie` and threaded that literal directly into Cedar fragments,
migrations, Rust modules, audit-chain stream names, FinOps cost-center
IDs, and reserved-namespace checks. This ADR introduces the single-
source-of-truth indirection that ADR-0242 deferred. ADR-0242 remains
authoritative for the *value* of the slug; ADR-0284 is authoritative
for *how that value is referenced*.

Enforcement is `advisory-until-constant-crate-lands`: the
`oya-shared-platform-constants-kernel` crate must exist, expose the
constant, and pass its own CI lane before this ADR's `oya-check-no-
hardcoded-platform-owner-slug` lane is promoted to BLOCKER. Bootstrap
ordering is documented in §Implementation surface.

## Date

2026-05-20.

## Context

### Why ADR-0242 left a hardcoded slug behind

ADR-0242 was authored under the keystone-bundle constraint that 14
foundational ADRs land together as a mutually-reinforcing set. The
authors' explicit framing was "establish doctrine in text first;
enforce mechanically once bootstrap lands." Within that scope, the
literal string `oyatie` appears in ADR-0242 in roughly 60 places
across the body, code examples, Cedar fragments, YAML schemas, and the
worked example. The literal also propagates into the artifacts the ADR
declares as the implementation surface:

- `microservices/tenancy/migrations/0001_create_self_tenant.sql`
  (writes `tenant_id = 'oyatie'`).
- `microservices/tenancy/src/reserved_namespace.rs` (the
  `reserved_roots = ["oyatie", "oya", "oyat", "oyati"]` array).
- `microservices/identity/src/oyatie_service_principals.rs` (the file
  name itself bakes the slug).
- `microservices/policy-engine/fragments/reserved-tenant-namespace.cedar`
  (the Cedar regex `/^oyatie[-_.]/i` literal).
- `microservices/policy-engine/fragments/oyatie-foundry-permits.cedar`,
  `oyatie-platform-ops-permits.cedar`, `oyatie-security-permits.cedar`
  (file names + body principal patterns).
- `microservices/audit-chain/src/oyatie_stream_provisioner.rs` (stream
  names `oyatie.root`, `oyatie.foundry`, `oyatie.security`,
  `oyatie.finance`, `oyatie.platform-ops`).
- `microservices/finops-portal/src/oyatie_cost_center.rs` (cost
  center IDs and rollup keys).
- `microservices/observability/dashboards/oyatie-tenant.md` (dashboard
  tenant filter literal).
- Cedar fragments at `oyatie.foundry.ci-agent`,
  `oyatie.foundry.eval-runner`, `oyatie.foundry.adr-drafter`,
  `oyatie.foundry.merge-queue` principal paths.
- DSAR cascade enumerator paths
  (`oyatie.foundry.engineer.<id>`).
- Bootstrap admin email `dsar@oyatie.com` in the tenant row YAML.
- 46 µservice manifests (per ADR-0242 §Implementation surface) where
  the `audience` field was being removed; the replacement
  `tenant.audience_type` references via tenant slug do not yet exist
  but will inevitably reference `oyatie` if the indirection isn't
  established first.

That literal also threads into:

- ADR-0240 sovereign-cloud `prohibited_egress` rules referencing
  `oyatie` data classes.
- ADR-0241 DR-tier declarations under `oyatie.security.incident-response`.
- ADR-0247 self-modification doctrine principal pattern
  `oyatie.foundry.ci-agent`.
- ADR-0248 cellular shape cell-class assignments to `oyatie` tenant.
- ADR-0255 Intelligence substrate provider-BYOK key path
  `oyatie/<tenant>/...`.
- The DSAR contact email pattern.
- Reserved namespace prefix family (`oya`, `oyat`, `oyati`,
  `oyatie-*`) — five distinct literals.

The result: the literal string `oyatie` is referenced from somewhere
between several hundred and several thousand sites across the
portfolio as the keystone bundle lands. Every Cedar fragment signed
during bootstrap (per ADR-0242 §D-5 step 5) cryptographically commits
to the literal. Every audit-chain entry sealed after bootstrap (per
ADR-0028 inheritance + Merkle sealing) cryptographically commits to
the stream names. Re-signing those artifacts after a slug change is
not a textual sed operation; it requires re-running cryptographic
ceremonies under the org root key.

### Why a rebrand is plausibly necessary at some future point

The portfolio's expected lifetime is decade-scale (per the autonomous-
masterplan goal in
`feedback_autonomous_implementation_artifacts`). Over a decade, the
following rebrand drivers are observed in industry:

- **Trademark conflict.** A larger company files a trademark in a
  conflicting class (e.g., `Oyatie` matching a registered mark in a
  jurisdiction the platform expands into). Resolution: rename or pay
  for license. Stripe rebranded one product line in 2018 after a
  conflicting registration in the EU; Atlassian's "Stride" was killed
  partly due to a trademark conflict overlap with another vendor.
- **Corporate restructuring.** Holding-company formation forces a slug
  change to match the new parent's branding. Facebook → Meta (2021)
  is the canonical example: a parent rebrand without changing the
  Facebook product slug, but operating-account and internal-tooling
  slugs migrated. Google → Alphabet (2015) is the structural model;
  Google retained its slug but new sub-products under Alphabet
  acquired their own slug spaces.
- **Geographic expansion conflict.** The slug means something
  offensive or trademarked in a target market's language. Mitsubishi
  Pajero is sold as Montero in Spanish-speaking markets. Slug
  changes for the same reason are documented across many SaaS expansions.
- **Acquisition.** The platform is acquired and the acquirer rebrands.
  Whether the slug also changes depends on integration strategy;
  acquihires routinely change slugs; strategic acquisitions sometimes
  keep them (Slack kept its slug after Salesforce acquisition; Github
  kept its slug after Microsoft acquisition).
- **Reputational reset.** The slug acquires negative association
  through an incident or scandal. Twitter → X (2023) was a forced
  rebrand driven by ownership preference plus a strategy reset; the
  slug change cost (per public press reporting) hundreds of millions
  of dollars in legal + technical + brand work.
- **Language modernization or neutrality.** The slug uses outdated
  terminology, jargon, or a culturally-loaded term. Open-source
  projects routinely retire terms like "master/slave" or "blacklist";
  product names face the same pressure on different time horizons.
- **Regulatory mandate.** A regulator mandates a slug change as part
  of a settlement or operating license. This is rare but documented
  (e.g., financial-services rebranding after enforcement actions).

Probability of *some* rebrand over decade-scale lifetime: high.
Probability of rebrand within the next 24 months: low. The asymmetry
favours preparing now: the cost of preparing the indirection is
bounded one-time work (this ADR + the constant crate); the cost of
not preparing is unbounded if and when rebrand becomes necessary.

### What "catastrophe without indirection" actually looks like

Without indirection, a rebrand from `oyatie` to (hypothetical)
`something-else` requires:

1. **Search-and-replace across hundreds of files.** Risk: missed
   sites; risk: false-positive replacements where the literal `oyatie`
   appears in test fixtures meant to remain as historical evidence.
2. **Re-sign every Cedar fragment under the org root key.** The
   re-signing ceremony itself emits audit-chain events; those events
   reference the old slug as the audit-stream-name; chicken-and-egg
   ordering must be choreographed carefully.
3. **Migrate the tenancy table.** A migration `000N_rename_self_tenant.sql`
   updates `tenant_id` from `oyatie` to the new slug. Every foreign
   key referencing the tenant row must cascade. Every audit-chain
   tombstone of the old tenant row must be preserved per FRCP 37(e)
   legal-hold compatibility.
4. **Re-emit audit-chain stream names.** Old streams (`oyatie.root`,
   `oyatie.foundry`, etc.) cannot be deleted (per ADR-0028 Merkle
   sealing). New streams must be created; old streams retained for
   the legal retention period; rollup views must union them.
5. **Reissue OIDC service principals.** Every `oyatie.*` service
   principal in Zitadel must be reissued under the new prefix. JWT
   issuer claims change; downstream verifiers must accept both
   issuers during transition.
6. **Re-evaluate every active session token.** Tokens minted under
   `oyatie.*` audiences must be honoured or revoked; transition
   policy decides.
7. **Update every observability dashboard.** Grafana / dashboards
   referencing tenant filter `oyatie` must update; pre-existing
   recording rules must add aliases.
8. **Update every µservice manifest.** ADR-0244's
   `tenant.audience_type` references; ADR-0240's
   `data_residency_allowed` blocks; ADR-0241's `dr_tier` declarations.
9. **Update every ADR that names the slug literally.** The ADR
   ledger itself names the slug in roughly 50 ADRs as of 2026-05-20.
   Each must either be amended or marked as carrying-historical-slug-
   intentionally. Per ADR-0212 buildability doctrine, ADR text is
   immutable post-acceptance for the historical record; amendments
   add forward links.
10. **Update every memory-feedback entry referencing the slug.** Per
    `feedback_oyatie_is_a_tenant_doctrine` and others; memory files
    must be added rather than rewritten.
11. **Update every spec file.** `/specs/platform-architecture.json`,
    `/specs/tenant-model.json`, and dozens more.
12. **Update every documentation surface.** Runbooks, READMEs, the
    installed runtime skill guidance, root `CLAUDE.md`, etc.
13. **Update every test fixture.** Test data referencing
    `oyatie.foundry.ci-agent` as a principal must be updated unless
    intentionally retained for historical regression testing.
14. **Update every external-integration manifest.** OAuth provider
    consent screens, third-party API tenant identifiers, contractual
    references, registry entries (PyPI, crates.io, npm, container
    registry org names).
15. **Update every DNS-bound asset.** `dsar@oyatie.com` becomes
    `dsar@something-else.com`; redirects required; SPF/DKIM/DMARC
    records updated; certificate transparency log entries cannot be
    retracted.
16. **Update every contractual document.** Customer agreements naming
    the platform owner; the platform-owner-as-tenant identifier in
    legal language.

Without indirection, this is a multi-week engineering effort with
high regression risk. With indirection (this ADR), most of items 1-3
and 5-9 collapse into "change one constant + rebuild + re-sign once."
Items 4 (audit-chain stream rename), 10-16 remain non-trivial but
become tractable because the engineering surface is bounded.

### Brand vs slug — Apple Intelligence pattern

Apple Intelligence (introduced at WWDC 2024) provides the canonical
pattern for separating *brand display name* from *underlying tenant
slug*. Apple's internal tenancy primitive is the Apple ID / Apple
account, which has carried a stable identifier scheme for decades.
The *brand surface* — what users see, what marketing emphasises, what
appears on retail packaging — has shifted across "Apple Intelligence,"
"Apple AI," "Personal Intelligence System," "Siri Intelligence,"
"on-device intelligence," and so on, depending on surface, audience,
locale, and announcement context. The brand name is i18n'd:
"Apple Intelligence" in English, "Apple 智能" in Chinese, "Apple Intelligenz"
in German, etc. (Where Apple chose to translate; sometimes brand names
are deliberately kept untranslated.)

The same separation applies here:

- **Slug** (`PLATFORM_OWNER_TENANT_SLUG`): stable, lowercase ASCII,
  reserved-namespace-protected, used internally in Cedar fragments,
  audit-chain stream names, OIDC issuer claims, database tenant_id
  values. Never localised; never marketing-controlled.
- **Brand display name** (`PLATFORM_OWNER_BRAND_DISPLAY_NAME`):
  marketing-controlled, i18n'd via Fluent (per ADR-0206), surface-
  dependent. Can be "Oyatie" in English marketing copy, "오야티"
  in Korean (hypothetical), "Oyatie Platform" in formal contractual
  language, "オヤティ" in Japanese localised UI, etc. Used in user-
  facing UI strings only.

These are two distinct constants with distinct lifecycle rules:

- The slug is changed only via a rebrand event with the full
  migration ceremony (this ADR §D-6).
- The brand display name is changed via the i18n message bundle update
  process (per ADR-0206 Fluent + ICU), with no migration ceremony.

### Why a constant crate rather than a config file

Alternatives considered for the indirection mechanism:

- **Environment variable.** Rejected (see Alternatives §Alt-1).
- **Config file loaded at runtime.** Rejected (see Alt-2).
- **Database row.** Rejected (see Alt-3).
- **Compile-time Rust constant in a shared kernel crate** (chosen).
- **Build-time substitution.** Rejected (see Alt-4).

The compile-time constant approach matches twelve-factor app guidance
for *immutable build artifacts* (per twelve-factor §III config — config
varies between deploys; the platform-owner identity does not vary
between deploys; therefore it is not config). It matches the
hyperscaler invariants from ADR-0128 (no runtime configuration of
platform identity). It matches ADR-0211's in-house Rust-primary stack.
It allows the Rust compiler to inline the constant; downstream code
referencing it has zero runtime overhead. It allows `cargo build` to
fail with a compile error if the constant is ever referenced
incorrectly. It allows `cargo doc` to surface the constant with its
documentation block as a discoverable type.

### Why a separate spec file (`/specs/platform-constants.json`)

The spec file is the language-neutral source of truth. The Rust crate
is one consumer; future TypeScript / Python / Go bindings (if and when
required by surface-specific code) consume the spec file via codegen.
This matches the spec-first pattern established across the portfolio:
specs at `/specs/*.json` are the source of truth; code is generated
or hand-derived from them.

Per ADR-0212 buildability doctrine, the spec file must be
machine-readable and have a JSON-Schema validation.

## Decision

### D-1. Single source of truth: `PLATFORM_OWNER_TENANT_SLUG` constant

A single named constant `PLATFORM_OWNER_TENANT_SLUG` holds the
platform-owner tenant slug. Its value at the time of this ADR is
`"oyatie"` (matching ADR-0242 §D-1). The value is declared in exactly
two places:

1. **`/specs/platform-constants.json`** (language-neutral source of
   truth). Schema:

   ```json
   {
     "$schema": "https://json-schema.org/draft/2020-12/schema",
     "title": "PlatformConstants",
     "version": "1.0.0",
     "constants": {
       "PLATFORM_OWNER_TENANT_SLUG": {
         "value": "oyatie",
         "type": "string",
         "stability": "rebrand-only",
         "lifecycle_authority": "ADR-0284",
         "namespace_authority": "ADR-0242",
         "constraints": {
           "encoding": "ASCII",
           "case": "lowercase",
           "max_length": 16,
           "regex": "^[a-z][a-z0-9-]{1,15}$"
         },
         "rebrand_history": []
       },
       "PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS": {
         "value": ["oyatie", "oya", "oyat", "oyati"],
         "type": "array<string>",
         "stability": "rebrand-only",
         "lifecycle_authority": "ADR-0284",
         "namespace_authority": "ADR-0242"
       },
       "PLATFORM_OWNER_BACKWARD_COMPAT_ALIASES": {
         "value": [],
         "type": "array<string>",
         "stability": "amend-on-rebrand",
         "lifecycle_authority": "ADR-0284"
       },
       "PLATFORM_OWNER_BRAND_DISPLAY_NAME_DEFAULT": {
         "value": "Oyatie",
         "type": "string",
         "stability": "marketing-controlled",
         "lifecycle_authority": "ADR-0284",
         "i18n_authority": "ADR-0206"
       },
       "PLATFORM_OWNER_DSAR_CONTACT_DOMAIN": {
         "value": "oyatie.com",
         "type": "string",
         "stability": "rebrand-or-dns-change",
         "lifecycle_authority": "ADR-0284"
       }
     }
   }
   ```

2. **`oya-shared-platform-constants-kernel` crate** (Rust source of
   truth, generated from the spec). The crate's `src/lib.rs`:

   ```rust
   //! Platform-owner identity constants.
   //!
   //! Authoritative spec: /specs/platform-constants.json
   //! Doctrine authority: ADR-0242 (slug value) + ADR-0284 (indirection).
   //!
   //! CRITICAL: do NOT reference the literal string "oyatie" anywhere
   //! else in the workspace. CI lane
   //! `oya-check-no-hardcoded-platform-owner-slug` enforces this.

   /// The platform-owner tenant slug.
   ///
   /// Stable across deploys; changeable only via the rebrand migration
   /// ceremony documented in ADR-0284 §D-6.
   pub const PLATFORM_OWNER_TENANT_SLUG: &str = "oyatie";

   /// The reserved-namespace root family.
   ///
   /// Used by the reserved-namespace admission gate (per ADR-0242 §D-1)
   /// to prevent typosquatting / impersonation.
   pub const PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS: &[&str] =
       &["oyatie", "oya", "oyat", "oyati"];

   /// Backward-compatibility aliases for prior slug values.
   ///
   /// Populated after a rebrand event. The prior slug remains routable
   /// for N years (per ADR-0284 §D-7). Empty under the initial slug.
   pub const PLATFORM_OWNER_BACKWARD_COMPAT_ALIASES: &[&str] = &[];

   /// Default brand display name (English).
   ///
   /// Surface-localised via Fluent (per ADR-0206). User-facing UI MUST
   /// resolve through the i18n bundle; this constant is the bootstrap
   /// fallback for code paths that load before i18n bundles.
   pub const PLATFORM_OWNER_BRAND_DISPLAY_NAME_DEFAULT: &str = "Oyatie";

   /// DSAR contact domain.
   ///
   /// Used to construct the DSAR contact email (e.g., `dsar@oyatie.com`).
   /// Changes only on a rebrand or DNS-change event.
   pub const PLATFORM_OWNER_DSAR_CONTACT_DOMAIN: &str = "oyatie.com";

   /// Helper: full DSAR contact email.
   pub fn dsar_contact_email() -> String {
       format!("dsar@{}", PLATFORM_OWNER_DSAR_CONTACT_DOMAIN)
   }

   /// Helper: principal-path prefix for the platform-owner tenant.
   ///
   /// Returns the slug followed by a dot, suitable for `format!`
   /// composition in principal-path code (e.g., `{}foundry.ci-agent`).
   pub fn principal_prefix() -> String {
       format!("{}.", PLATFORM_OWNER_TENANT_SLUG)
   }

   /// Helper: returns true if the given slug matches the platform-owner
   /// slug or any reserved-namespace root or any backward-compat alias.
   ///
   /// Used by the reserved-namespace admission gate. Performs NFKC +
   /// lowercase + diacritic-strip per ADR-0242 §D-6 before comparison.
   pub fn is_platform_owner_slug(candidate: &str) -> bool {
       let normalised = unicode_normalize(candidate);
       normalised == PLATFORM_OWNER_TENANT_SLUG
           || PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS
               .iter()
               .any(|r| &normalised == r)
           || PLATFORM_OWNER_BACKWARD_COMPAT_ALIASES
               .iter()
               .any(|a| &normalised == a)
   }

   fn unicode_normalize(s: &str) -> String {
       // NFKC + lowercase + diacritic-strip + confusable-remove per
       // ADR-0242 §D-6. Delegates to oya-shared-unicode-security.
       oya_shared_unicode_security::canonicalise(s)
   }
   ```

The spec is the source; the crate is generated by the
`oya-codegen-platform-constants` tool, which reads
`/specs/platform-constants.json` and emits the crate. Hand-editing the
generated crate is rejected by CI lane `oya-check-platform-constants-codegen-clean`.

### D-2. All Cedar fragments, audit-chain code, tenancy bootstrap, identity provisioning reference the constant — not the literal

Every consumer of the platform-owner slug uses `PLATFORM_OWNER_TENANT_SLUG`
or one of the helper functions. Concretely:

**Rust code** (e.g., `microservices/tenancy/src/reserved_namespace.rs`):

```rust
use oya_shared_platform_constants_kernel as platform_const;

pub fn is_reserved(proposed_id: &str) -> ReservedResult {
    if platform_const::is_platform_owner_slug(proposed_id) {
        return ReservedResult::Reserved {
            root: platform_const::PLATFORM_OWNER_TENANT_SLUG.to_string(),
            normalised: proposed_id.to_string(),
        };
    }
    ReservedResult::Available
}
```

**Cedar fragments** must reference the slug via a templated build
step. Cedar policy text is not Rust; it is loaded as text. To keep
Cedar fragments free of the hardcoded literal, the fragment is
authored as a `.cedar.tera` template with the slug as a substitution
variable, and built at compile time by the `oya-codegen-cedar-fragments`
tool from the constant:

```cedar.tera
// policy-engine/fragments/reserved-tenant-namespace.cedar.tera

forbid (
  principal,
  action == TenancyAction::"RegisterTenant",
  resource is Tenant
)
when {
  resource.id matches /^{{ platform_owner_tenant_slug }}[-_.]/i
  || resource.id == "{{ platform_owner_tenant_slug }}"
  || resource.normalized_id == "{{ platform_owner_tenant_slug }}"
};
```

The build step substitutes `{{ platform_owner_tenant_slug }}` →
`oyatie` (or whatever the constant resolves to at build time),
emitting `reserved-tenant-namespace.cedar` as the build artifact.
Cedar fragments are signed at the build artifact (after substitution),
not at the template, because cryptographic signing must commit to the
exact text loaded into the policy engine.

**Migration SQL** uses a templating system at code-generation time.
The migration `0001_create_self_tenant.sql.tera` becomes
`0001_create_self_tenant.sql` after substitution:

```sql.tera
-- migrations/0001_create_self_tenant.sql.tera
INSERT INTO tenants (tenant_id, audience_type, locked, created_at, created_by)
VALUES (
  '{{ platform_owner_tenant_slug }}',
  'PLATFORM_OWNER',
  TRUE,
  CURRENT_TIMESTAMP,
  'system:bootstrap-migration'
);
```

**Audit-chain stream provisioner** uses the constant directly:

```rust
let streams = vec![
    format!("{}.root", platform_const::PLATFORM_OWNER_TENANT_SLUG),
    format!("{}.foundry", platform_const::PLATFORM_OWNER_TENANT_SLUG),
    format!("{}.security", platform_const::PLATFORM_OWNER_TENANT_SLUG),
    format!("{}.finance", platform_const::PLATFORM_OWNER_TENANT_SLUG),
    format!("{}.platform-ops", platform_const::PLATFORM_OWNER_TENANT_SLUG),
];
```

**Spec files** (other than `/specs/platform-constants.json` itself)
reference the slug via the `$ref` mechanism in JSON-Schema:

```json
{
  "tenant_id": {
    "$ref": "platform-constants.json#/constants/PLATFORM_OWNER_TENANT_SLUG/value"
  }
}
```

At code-generation time, the schema resolver materialises the value;
spec text never carries the literal.

**Markdown documentation** is the unavoidable exception (text files
cannot reference programmatic constants). Markdown that *intentionally*
discusses the current slug value uses the literal `oyatie` and is
exempted from the `oya-check-no-hardcoded-platform-owner-slug` lane by
file extension. ADRs are similarly exempted by directory
(`docs/decisions/`). On rebrand, ADRs are not retroactively rewritten;
they carry historical-slug-intentionally markers in the rebrand
migration ledger (per §D-6).

### D-3. CI lane `oya-check-no-hardcoded-platform-owner-slug`

A new CI lane detects literal string usage outside the constant
declaration. The lane is implemented as a small Rust binary
(`crates/oya-check-no-hardcoded-platform-owner-slug/`) that:

1. Reads the current value of `PLATFORM_OWNER_TENANT_SLUG` from
   `/specs/platform-constants.json`.
2. Reads the `PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS` array.
3. Reads `PLATFORM_OWNER_BACKWARD_COMPAT_ALIASES`.
4. Constructs a regex matching whole-word occurrences of any of those
   values, case-insensitively, allowing common prefix/suffix separators
   (`-`, `_`, `.`, `:`, `/`).
5. Walks the workspace files, excluding:
   - `/specs/platform-constants.json` (the source of truth).
   - `crates/oya-shared-platform-constants-kernel/` (the Rust constant
     declaration crate).
   - `docs/decisions/` (ADRs — historical-slug-intentionally).
   - `**/*.md` (Markdown documentation).
   - `**/*.tera` (templates — substitution variables expected here).
   - `**/CHANGELOG.md` (changelog history).
   - `**/migration-ledger/*` (rebrand migration ledger entries —
     historical).
   - Test fixtures under `**/tests/fixtures/historical/` (intentional
     historical regression data).
6. For every match outside the exclusion set, the lane emits a finding
   with file path, line number, and the matched literal.
7. The lane exits non-zero (BLOCKER) post-bootstrap; advisory until
   bootstrap.

The lane runs on every PR. It also runs in the merge queue projection
check (per ADR-0111 fix-at-any-stage).

Implementation detail: the lane uses `oya git`'s `git grep --line-number`
under the hood for performance on large workspaces, plus a Rust pass
for the more nuanced normalisation checks (NFKC + diacritic-strip
variants).

### D-4. Brand display name separate from slug; display name is i18n'd

Two distinct identifiers govern user-facing rendering vs internal
identity:

- **Slug** (`PLATFORM_OWNER_TENANT_SLUG`, `"oyatie"`): internal
  identity. Cedar fragments, audit-chain streams, OIDC issuer, DB
  tenant_id. Stable; rebrand-only.
- **Brand display name**
  (`PLATFORM_OWNER_BRAND_DISPLAY_NAME_DEFAULT`, `"Oyatie"`): English-
  default user-facing label. Resolved through Fluent (per ADR-0206)
  for locale-specific surfaces.

The Fluent message bundle layout (per ADR-0206):

```fluent
# packs/canonical/i18n/en/platform.ftl
platform-owner-brand-display-name = Oyatie
platform-owner-product-name-prefix = Oyatie
platform-owner-formal-legal-name = Oyatie Platform

# packs/kr/i18n/ko/platform.ftl
platform-owner-brand-display-name = 오야티
platform-owner-product-name-prefix = 오야티
platform-owner-formal-legal-name = 오야티 플랫폼

# packs/eu-de/i18n/de/platform.ftl
platform-owner-brand-display-name = Oyatie
platform-owner-product-name-prefix = Oyatie
platform-owner-formal-legal-name = Oyatie Plattform
```

Per ADR-0064 canonical-base + localization-overlay pattern:

- The canonical base ships
  `packs/canonical/i18n/en/platform.ftl` with the English value.
- Regional packs ship locale overlays
  (`packs/kr/i18n/ko/platform.ftl`, etc.).
- Per-surface overrides (e.g., a marketing landing page that wants a
  different rendering for a campaign) use the Fluent override
  hierarchy.

CI lane `oya-check-platform-owner-brand-vs-slug-separation` enforces:

1. User-facing UI code (any file under `apps/web/*`, `apps/mobile/*`,
   `apps/desktop/*`) MUST NOT reference `PLATFORM_OWNER_TENANT_SLUG`
   directly for rendering. It MUST resolve through the Fluent
   `platform-owner-brand-display-name` message.
2. Audit-chain stream code, Cedar fragments, tenancy bootstrap, and
   identity provisioning MUST NOT reference
   `PLATFORM_OWNER_BRAND_DISPLAY_NAME_DEFAULT` or Fluent messages.
   They MUST use `PLATFORM_OWNER_TENANT_SLUG`.

Violations fall into one of two patterns:

- **Pattern A (brand-in-identity):** rendering the brand display name
  in an audit-chain stream name or Cedar principal path. This is
  rejected because i18n'd text is not stable; the audit-chain stream
  name must be deterministic.
- **Pattern B (slug-in-brand):** using the slug `oyatie` as a user-
  facing rendering target. This is rejected because the slug is
  marketing-uncontrolled; UI text must be localisable.

### D-5. Reserved namespace list also lives in the constant

The reserved-namespace root family from ADR-0242 §D-1 (`oyatie`,
`oya`, `oyat`, `oyati`) is declared as
`PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS` in
`/specs/platform-constants.json`. The reserved-namespace admission
gate (per ADR-0242 §D-6) reads from the constant rather than
hardcoding the array.

The family covers:

- The full slug `oyatie`.
- Three short-prefix aliases (`oya`, `oyat`, `oyati`) — these prevent
  typosquatting where a third party might register `oya` or `oyat`
  to imply affiliation.
- The prefix-with-separator family: `oyatie-*`, `oyatie.*`,
  `oyatie_*`, computed at admission time by the reserved-namespace
  gate (per ADR-0242 §D-6 code).
- IDN-homograph variants per UTS #39, computed at admission time via
  `oya-shared-unicode-security::canonicalise`.

On rebrand (see §D-6), the reserved-namespace root family migrates
together with the slug. Backward-compatibility aliases are added to
the array to prevent the *old* slug from being reclaimed by a third
party after rebrand. Concretely, if `oyatie` rebranded to
`something-else`, the post-rebrand
`PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS` array would be:

```rust
&[
    "something-else", "something", "somethi", "someth", "somet", "some",
    "oyatie", "oya", "oyat", "oyati",   // legacy, retained to block reclamation
]
```

The legacy roots stay reserved indefinitely. Customer tenants can
never register the prior platform-owner slug as their own.

### D-6. Migration procedure if rebrand ever happens

A rebrand from `OLD_SLUG` to `NEW_SLUG` follows the migration ceremony
documented here. The ceremony is invoked exactly once per rebrand
event and is captured in a rebrand-migration ledger entry at
`/migration-ledger/rebrand/<rebrand-id>/`.

**Pre-rebrand gates (must be satisfied before ceremony begins):**

1. ADR amendment to ADR-0284 (this ADR) authorising the rebrand,
   with the new slug, the rebrand-driver justification, the target
   completion date, and the backward-compat alias retention period
   (default 7 years per §D-7).
2. Council-architecture + council-product + council-brand +
   council-legal approval, recorded in the ledger.
3. Trademark clearance on `NEW_SLUG` in all operating jurisdictions
   (per council-legal sign-off).
4. Reserved-namespace audit: confirm no customer tenant has
   registered any variant of `NEW_SLUG` (per the case-fold + NFKC +
   confusable check). If a customer holds a conflicting tenant, the
   rebrand cannot proceed until the conflict is resolved.
5. DR drill: simulate the rebrand against the dr cell; confirm
   rollback works.
6. Multispectrum review v2.4.0 verdict APPROVE on the rebrand
   ChangeSet (per the standard governance workflow).

**Ceremony stages (executed in strict order):**

**Stage R-1: Pause.** All new tenant registrations are paused. All
in-flight Foundry workflows complete or are quiesced. Audit-chain
emissions continue (cannot pause; sealed log requires append-only).

**Stage R-2: Slug update in spec.**
`/specs/platform-constants.json` is amended:

```json
{
  "constants": {
    "PLATFORM_OWNER_TENANT_SLUG": {
      "value": "something-else",
      "rebrand_history": [
        {
          "old_value": "oyatie",
          "rebrand_id": "<rebrand-id>",
          "rebrand_date": "<date>",
          "rebrand_driver": "<trademark|restructure|geo|acquisition|reset|language|regulatory>",
          "amendment_adr": "ADR-XXXX",
          "backward_compat_alias_until": "<date + 7 years>"
        }
      ]
    },
    "PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS": {
      "value": ["something-else", "something", "somethi", "someth",
                "somet", "some", "oyatie", "oya", "oyat", "oyati"]
    },
    "PLATFORM_OWNER_BACKWARD_COMPAT_ALIASES": {
      "value": ["oyatie"]
    },
    "PLATFORM_OWNER_BRAND_DISPLAY_NAME_DEFAULT": {
      "value": "Something Else"
    },
    "PLATFORM_OWNER_DSAR_CONTACT_DOMAIN": {
      "value": "something-else.com"
    }
  }
}
```

**Stage R-3: Constant crate rebuild.** `cargo build -p
oya-shared-platform-constants-kernel` rebuilds the Rust constant
crate. All workspace crates depending on it are rebuilt. Cedar
fragment templates and migration SQL templates are re-rendered. The
build is performed in an isolated rebuild cell.

**Stage R-4: Migration ledger entry.** A signed entry at
`/migration-ledger/rebrand/<rebrand-id>/manifest.json` records:

```json
{
  "rebrand_id": "<rebrand-id>",
  "old_slug": "oyatie",
  "new_slug": "something-else",
  "rebrand_date": "<date>",
  "rebrand_driver": "<reason>",
  "amendment_adr": "ADR-XXXX",
  "ceremony_stages": [...],
  "signed_by_org_root_key": "<signature>",
  "ledger_predecessor_hash": "<hash>",
  "backward_compat_alias_until": "<date + 7 years>"
}
```

**Stage R-5: Re-sign Cedar fragments.** Every Cedar fragment whose
text changed (because it referenced the slug) is re-signed under the
org root key. The signing ceremony emits audit-chain events. Old
Cedar fragments remain signed and retained for the legal hold period.

**Stage R-6: Tenancy table rename.** Migration `<next>_rename_self_tenant.sql`
runs:

```sql
UPDATE tenants SET tenant_id = 'something-else'
WHERE tenant_id = 'oyatie';

INSERT INTO tenant_aliases (alias, current_tenant_id, retained_until)
VALUES ('oyatie', 'something-else', '<date + 7 years>');
```

Foreign keys cascade.

**Stage R-7: Audit-chain stream creation.** New streams
`something-else.root`, `something-else.foundry`, etc., are created
and provisioned with new Ed25519 signing keys (held in OpenBao). The
old streams are NOT deleted; they remain Merkle-sealed and append-
only. A bridge audit event in each old stream marks the rebrand and
references the corresponding new stream. Aggregation views (e.g., the
FinOps cost-center rollup) union the old and new streams.

**Stage R-8: Service-principal reissue.** Every Zitadel service
principal under `oyatie.*` is reissued under `something-else.*` with
overlapping validity. JWT verifiers in every µservice accept tokens
from either issuer during the transition window (90 days default,
configurable).

**Stage R-9: Observability migration.** Grafana dashboards
referencing tenant filter `oyatie` are updated to `something-else`
with the alias clause `or tenant = "oyatie"` for the legacy window.
Recording rules add aliases.

**Stage R-10: External integration update.** OAuth consent screens,
third-party API tenant identifiers, registry org renames, DNS
updates, certificate transparency log entries (cannot be retracted,
new entries added for the new domain).

**Stage R-11: Documentation update sweep.** Runbooks, READMEs, the
root `CLAUDE.md` and installed runtime skill guidance are updated. ADRs are NOT retroactively
rewritten; the historical-slug-intentionally exemption preserves
ADR text.

**Stage R-12: Unpause.** New tenant registrations resume. Foundry
workflows resume. The reserved-namespace admission gate is updated
to reject any registration attempting the old slug (which is now in
the backward-compat alias list and therefore reserved).

**Stage R-13: Verification.** The verification checklist from
`/migration-ledger/rebrand/<rebrand-id>/verification.md` runs.
Required checks:

- `oya gate validate platform-owner-constant-indirection` exits 0.
- `oya gate validate no-hardcoded-platform-owner-slug` exits 0 (the
  new slug; the old slug is in the alias array which is also
  excluded from the check).
- `oya gate validate platform-owner-brand-vs-slug-separation` exits 0.
- `oya gate validate reserved-namespace-protection` exits 0.
- DSAR cascade test against an old-slug-prefixed principal returns
  results from the renamed tenant.
- Audit-chain old-stream readback proves the bridge event is present
  and Merkle-valid.

### D-7. Backward-compat alias: old slug remains routable for N years after rebrand

After a rebrand event, the prior slug remains a valid alias for the
new tenant for **N = 7 years** by default. Rationale:

- **Customer integration contracts.** Customer-side OAuth client
  registrations, webhook URLs, API base paths may include the old
  slug. The 7-year window matches typical enterprise contract
  duration plus renewal cycle.
- **Audit-chain readback compatibility.** Auditors querying the
  audit chain by historical principal path
  (`oyatie.foundry.ci-agent`) must continue to receive results.
- **Legal hold compatibility.** Records under legal hold may
  reference the old slug; the hold typically runs 7 years (FRCP-
  bounded retention).
- **DNS record TTL + redirect grace.** Customer DNS caches for
  `oyatie.com` may take months to expire; the 7-year window swallows
  this comfortably.

The alias works at three layers:

1. **Tenancy substrate.** The `tenant_aliases` table maps old slug
   → new tenant_id. Lookups by tenant_id resolve through this table.
2. **Identity substrate.** OIDC issuer claims accept the old issuer
   URL; tokens minted under the old issuer remain valid until the
   alias expiration.
3. **Cedar policy engine.** Cedar fragments matching `oyatie.*`
   principals are retained alongside new fragments matching
   `something-else.*`; gates fire on whichever matches.

The alias expiration is configurable in the rebrand-migration ledger
entry. Council-architecture + council-legal must approve an
extension beyond the 7-year default; the default cannot be shortened
without an ADR amendment.

The expiration removal procedure runs as a scheduled migration on
the alias expiration date:

- The `tenant_aliases` row is deleted (subject to legal-hold
  preservation overrides).
- The old OIDC issuer is decommissioned in Zitadel.
- Old Cedar fragments are retired (text preserved in the policy-
  engine retirement archive; effective policy is post-rebrand only).
- The old slug remains in `PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS`
  permanently to prevent reclamation by a third party.

### D-8. Brand-of-product separation (Apple Intelligence pattern)

User-facing brand surfaces are decoupled from the platform-owner
tenant slug. Per the Apple Intelligence pattern, individual products
under the platform-owner umbrella may carry their own brand surfaces
that vary by:

- **Locale.** Mail might render as "Mail," "메일," "Correo," "Mailer,"
  depending on locale and marketing decision.
- **Surface.** Marketing landing page vs in-app product header vs
  contractual document may render differently.
- **Audience.** B2B-tenant surface vs B2C-consumer surface may
  carry different product branding.
- **Campaign.** A time-limited campaign may rebrand the product
  surface (e.g., "Mail X.0" for a major release).

These per-product brand surfaces are managed through the Fluent i18n
bundle (per ADR-0206) under per-product message keys:

```fluent
# packs/canonical/i18n/en/product-mail.ftl
product-mail-display-name = Mail
product-mail-full-name = Oyatie Mail
product-mail-tagline = Email that works for you
```

The platform-owner tenant slug never appears in user-facing UI. The
brand-of-product separation enforced by CI lane
`oya-check-platform-owner-brand-vs-slug-separation` (§D-4) ensures:

- Audit-chain stream names use the slug (stable).
- Cedar principal paths use the slug (stable).
- UI labels use the Fluent message (variable).

The decoupling enables:

- Marketing-led product rebrands without platform-rebrand ceremony.
- Per-locale product naming.
- Per-campaign product naming.
- Per-audience product naming.
- A/B testing of product brand surfaces.

It also constrains:

- Platform-owner identity remains stable across product brand changes.
- Audit-chain evidence remains semantically valid across product
  brand changes.
- Customer contracts referencing the platform-owner tenant slug
  remain valid across product brand changes.

## Alternatives considered

### Alt-1. Environment variable (`PLATFORM_OWNER_TENANT_SLUG=oyatie`)

Read the slug from an environment variable at process startup.

**Pros:**

- No rebuild required on rebrand (just restart with new env var).
- Per-environment override possible (e.g., for sandbox / preview
  tenants).
- Twelve-factor app-aligned for the "config" interpretation.

**Cons:**

- **Violates twelve-factor app §III "config" definition.** Twelve-
  factor specifies config as "everything that is likely to vary
  between deploys." Platform-owner identity does not vary between
  deploys; therefore it is not config; therefore it should not be in
  env vars.
- **Runtime variability is a security risk.** If the env var is
  unset or wrong, code that constructs Cedar principal paths,
  audit-chain stream names, or tenancy IDs from the value may
  silently emit garbage. A misconfigured deploy could emit to the
  wrong audit stream, breaking tamper-detection coverage.
- **Compile-time inlining is lost.** The Rust compiler cannot inline
  a runtime-read value; downstream code carries a runtime read
  overhead. Negligible per-call, but the cumulative effect across
  millions of audit-chain emissions per day is measurable.
- **Test fixtures complicate.** Tests must set the env var
  explicitly; integration tests across multiple processes must agree
  on the value.
- **Drift risk.** Different env vars in different cells lead to
  divergent platform-owner identity, the worst kind of split-brain.

**Rejected** because the value is not config; it is identity. Identity
is compile-time-stable.

### Alt-2. Config file loaded at runtime (`config/platform.toml`)

Read the slug from a config file loaded at process startup.

**Pros:**

- No rebuild required on rebrand (just deploy new config file +
  restart).
- Hot-reload possible (with extra machinery).
- Version-controlled config file.

**Cons:**

- **Same fundamental issue as Alt-1.** Identity is not config.
- **Hot-reload introduces split-brain windows.** While processes
  reload, some are on the old value, some on the new. Audit-chain
  emissions during the window are split across streams.
- **Bootstrap dependency cycle.** The tenancy bootstrap migration
  needs the value before any config-loading machinery runs.
  Chicken-and-egg.
- **Compile-time inlining is lost** (same as Alt-1).

**Rejected** for the same reasons as Alt-1.

### Alt-3. Database row (`platform_settings` table)

Store the slug in a `platform_settings` row in the tenancy database.

**Pros:**

- Single source of truth at runtime.
- Easy to query.

**Cons:**

- **Bootstrap dependency cycle (worse than Alt-2).** The tenancy
  database needs the slug to create the bootstrap tenant row; if the
  slug is in a database row, the row must exist before the migration
  that creates the bootstrap tenant runs. Chicken-and-egg with no
  resolution.
- **Cedar fragment build time cannot read from a database.** Cedar
  fragments are loaded at policy-engine startup; the fragment text
  must be substituted at *build* time, not at *load* time, because
  signed fragments commit to text. Database read at build time
  introduces a build-time database dependency.
- **Audit-chain stream provisioner cannot read from a database
  before audit-chain exists.** The audit-chain substrate has its own
  dependencies; reading the slug from a database introduces a
  startup-order constraint.
- **Test fixtures complicate** (worse than Alt-1; database setup
  required for every test).
- **Drift risk** (worse than Alt-1; database writes have history;
  reverting to a known-good slug is harder than reverting an env var).

**Rejected** because the bootstrap cycle is unsolvable and the
build-time substitution case is fatal.

### Alt-4. Build-time substitution from a Makefile variable

A Makefile variable `PLATFORM_OWNER_TENANT_SLUG=oyatie` is passed to
`cargo build --features platform-owner-slug-oyatie` or via
`OYA_PLATFORM_OWNER_TENANT_SLUG=oyatie cargo build`.

**Pros:**

- Compile-time inlining preserved.
- Single point of change in the Makefile.

**Cons:**

- **Cargo features for this purpose is anti-idiomatic.** Cargo
  features are intended for conditional compilation (e.g., enable a
  feature flag). The platform-owner slug is not a feature flag.
- **Environment variable at build time** (the `OYA_*` variant) has
  the same drift risk as Alt-1 (different builds with different env
  vars yield different binaries; not detectable at runtime).
- **No source-of-truth file.** The Makefile variable is one of many
  Makefile variables; not discoverable as a portfolio constant.
- **No JSON schema.** Other languages / tools cannot consume a
  Makefile variable directly.

**Rejected** in favour of spec-file + generated-crate (Alt-5).

### Alt-5. Spec file + generated crate ← **CHOSEN**

The chosen design, fully specified in §Decision.

**Pros:**

- **Single source of truth in spec.** `/specs/platform-constants.json`
  is the language-neutral root.
- **Generated Rust crate.** `oya-shared-platform-constants-kernel`
  provides compile-time inlining.
- **JSON-schema validation.** The spec is type-checked.
- **Discoverable.** A directory listing of `/specs/` surfaces the
  constants file.
- **Cross-language.** TypeScript / Python / Go / Cedar / SQL
  bindings all generate from the same spec.
- **Codegen-clean check.** CI lane prevents hand-editing of
  generated artifacts.
- **Bootstrap-friendly.** Compile-time inlining means no runtime
  dependency on tenancy / config / env.
- **Cedar build-time substitution works.** Templates resolve at
  build time; signed fragments commit to substituted text.
- **Twelve-factor compliant.** Identity is not config; identity is
  source.

**Cons:**

- **Rebuild required on rebrand.** Every workspace crate depending
  on the constant crate rebuilds. Acceptable: rebrand is once-per-
  decade; rebuild is a known, bounded operation.
- **Codegen tooling.** A new tool `oya-codegen-platform-constants`
  must be authored. Bounded effort.

**Accepted** as the chosen design.

### Alt-6. Defer indirection until rebrand becomes imminent

Do nothing now; revisit when a rebrand looks likely.

**Pros:**

- Zero work now.

**Cons:**

- **Drift compounds quadratically.** Every new ADR, every new
  µservice, every new Cedar fragment adds literal `oyatie`
  references that must later be hunted down. Per `feedback_no_silent_regression`,
  this kind of silent accretion is exactly what the portfolio's
  Linus-style discipline forbids.
- **Re-signing ceremony cost grows with literal-count.** Re-signing
  100 Cedar fragments is bounded; re-signing 10,000 is much harder.
- **Bootstrap commits cryptographically.** Once bootstrap lands,
  every Merkle-sealed audit-chain row commits to the slug-as-text.
  Re-sealing is impossible (sealed log is append-only). Indirection
  must precede bootstrap for the audit-chain story to be clean.
- **Trademark conflict surface area grows.** Every additional system
  referencing the literal is a system that must be checked for
  jurisdiction-specific naming when a rebrand-driving trademark
  conflict arises. Earlier indirection = smaller surface.

**Rejected** because the drift loop compounds and the cost-to-fix-
later is unbounded.

## Consequences

### Positive

1. **Rebrand becomes a one-place change in spec + crate rebuild +
   ceremony.** Without this ADR, rebrand is unbounded multi-week
   engineering effort with high regression risk.
2. **Audit-chain remains semantically valid across rebrand.** Old
   streams retained; new streams created; bridge events link them.
   No sealed-log corruption.
3. **Cedar fragments are templatable.** Build-time substitution
   means fragments aren't littered with the literal slug.
4. **Brand display name decoupled.** Marketing can iterate on brand
   surfaces without touching tenant identity. Per-locale and
   per-surface variation works through Fluent.
5. **Reserved-namespace protection lives in one place.** Updates to
   the reserved-namespace family (e.g., adding new prefix variants
   on rebrand) are spec-edits.
6. **Codegen-clean discipline enforced.** Generated Rust crate is
   never hand-edited; CI catches drift.
7. **Backward-compat alias machinery is documented.** When rebrand
   happens, the 7-year alias window prevents customer-integration
   breakage.
8. **Drift loop closed.** Future ADRs cannot reintroduce the
   hardcoded literal; CI lane fires.

### Negative

1. **One-time migration cost.** All current literal `oyatie`
   references in code (not ADRs / Markdown) must be replaced by
   constant references. Bounded; one ChangeSet executes it. PR-
   level work; multispectrum review verifies.
2. **Codegen tool must be authored.** `oya-codegen-platform-constants`
   is new tooling. Bounded effort.
3. **Cedar template format is new.** `.cedar.tera` is a new file
   extension; the build pipeline must handle substitution. Bounded.
4. **Rebuild required on rebrand.** Acceptable cost given rebrand
   frequency.
5. **CI lane `oya-check-no-hardcoded-platform-owner-slug` runs on
   every PR.** Adds ~1-2 seconds to PR validation. Acceptable.

### Operational

1. **New crates:**
   - `crates/oya-shared-platform-constants-kernel/` (the Rust
     constant declaration, generated from spec).
   - `crates/oya-codegen-platform-constants/` (the codegen tool).
   - `crates/oya-check-no-hardcoded-platform-owner-slug/` (the CI
     lane binary).
2. **New spec files:**
   - `/specs/platform-constants.json` (the source-of-truth spec).
   - `/specs/platform-constants-schema.json` (JSON-schema for the
     spec, for validation).
3. **New CI lanes:**
   - `oya-check-platform-owner-constant-indirection` (verifies the
     constant crate exists, exports the expected symbols, and
     `/specs/platform-constants.json` matches).
   - `oya-check-no-hardcoded-platform-owner-slug` (verifies no
     literal references outside the allowed locations).
   - `oya-check-platform-owner-brand-vs-slug-separation` (verifies
     UI code uses Fluent, identity code uses the slug constant).
   - `oya-check-platform-constants-codegen-clean` (verifies the
     generated crate matches what the codegen tool would produce
     from the spec).
4. **Modifications to ADR-0242 artifacts:**
   - `microservices/tenancy/migrations/0001_create_self_tenant.sql`
     → `.tera` template form.
   - `microservices/tenancy/src/reserved_namespace.rs` → uses
     `platform_const::*`.
   - `microservices/identity/src/oyatie_service_principals.rs` →
     renamed to `microservices/identity/src/platform_owner_service_principals.rs`;
     uses `platform_const::*`.
   - `microservices/policy-engine/fragments/*.cedar` → all become
     `*.cedar.tera` templates.
   - `microservices/audit-chain/src/oyatie_stream_provisioner.rs`
     → renamed to `platform_owner_stream_provisioner.rs`; uses
     `platform_const::*`.
   - `microservices/finops-portal/src/oyatie_cost_center.rs` →
     renamed similarly.
5. **Modifications to ADR-0242 itself:**
   - Frontmatter adds `amended_by: ADR-0284`.
   - §D-1 amended-in-place to note that the literal value lives in
     `/specs/platform-constants.json` and the constant crate.
6. **Documentation:**
   - `docs/standards/platform-owner-name-indirection.md` (new
     standard).
   - `docs/runbooks/platform-owner-rebrand-ceremony.md` (new
     runbook).

### Sustainability

- No direct sustainability impact. Indirect: codegen-clean discipline
  prevents accidental rebuilds caused by literal-string drift;
  rebuild count per year decreases marginally.

### Compliance

- **GDPR / KR PIPA.** Backward-compat alias preserves DSAR access to
  historical records by old slug.
- **FRCP 37(e).** Sealed audit-chain rows under the old slug remain
  preserved; rebrand does not destroy them.
- **SOC 2 / ISO 27001 / ISO 22301.** Uniform identity treatment
  preserved across rebrand; auditors see a clean migration ledger.
- **Trademark law.** Reserved-namespace family extends to block
  third-party reclamation of the old slug after rebrand, preserving
  brand-confusion-prevention obligations.

## Implementation surface

The following artifacts are required for this ADR to be considered
implemented:

| Artifact | Status |
|---|---|
| `/specs/platform-constants.json` | NEW — source-of-truth spec |
| `/specs/platform-constants-schema.json` | NEW — JSON-schema for the spec |
| `crates/oya-shared-platform-constants-kernel/Cargo.toml` | NEW |
| `crates/oya-shared-platform-constants-kernel/src/lib.rs` | NEW — generated from spec |
| `crates/oya-codegen-platform-constants/` | NEW — codegen tool |
| `crates/oya-check-no-hardcoded-platform-owner-slug/` | NEW — CI lane binary |
| `crates/oya-check-platform-owner-constant-indirection/` | NEW — CI lane binary |
| `crates/oya-check-platform-owner-brand-vs-slug-separation/` | NEW — CI lane binary |
| `crates/oya-check-platform-constants-codegen-clean/` | NEW — CI lane binary |
| `crates/oya-shared-unicode-security/` | NEW (or reused if exists) — NFKC + diacritic-strip + confusable-remove helper |
| `microservices/tenancy/migrations/0001_create_self_tenant.sql.tera` | NEW — template form |
| `microservices/tenancy/src/reserved_namespace.rs` | MODIFIED — uses constant crate |
| `microservices/identity/src/platform_owner_service_principals.rs` | NEW (renamed from `oyatie_service_principals.rs`) |
| `microservices/policy-engine/fragments/reserved-tenant-namespace.cedar.tera` | NEW — template form |
| `microservices/policy-engine/fragments/platform-owner-foundry-permits.cedar.tera` | NEW (renamed) |
| `microservices/policy-engine/fragments/platform-owner-platform-ops-permits.cedar.tera` | NEW (renamed) |
| `microservices/policy-engine/fragments/platform-owner-security-permits.cedar.tera` | NEW (renamed) |
| `microservices/audit-chain/src/platform_owner_stream_provisioner.rs` | NEW (renamed) |
| `microservices/finops-portal/src/platform_owner_cost_center.rs` | NEW (renamed) |
| `microservices/observability/dashboards/platform-owner-tenant.md` | NEW (renamed) |
| `packs/canonical/i18n/en/platform.ftl` | NEW — brand display name resource |
| `packs/canonical/i18n/en/product-*.ftl` | NEW per product |
| `docs/standards/platform-owner-name-indirection.md` | NEW — standard |
| `docs/runbooks/platform-owner-rebrand-ceremony.md` | NEW — runbook |
| `tools/build/cedar-template-renderer/` | NEW — Cedar template substitution at build time |
| `tools/build/sql-template-renderer/` | NEW — SQL template substitution at code-gen time |
| ADR-0242 frontmatter amendment (`amended_by: ADR-0284`) | MODIFIED |

Bootstrap ordering:

1. Author `/specs/platform-constants.json` and its schema.
2. Author the codegen tool `oya-codegen-platform-constants`.
3. Generate `oya-shared-platform-constants-kernel`.
4. Author the four CI-lane binaries; lanes run in advisory mode.
5. Migrate all existing literal references (the post-ADR-0242
   artifacts) to use the constant crate. Run lanes in advisory mode;
   findings drive the migration.
6. Once findings drop to zero, promote the lanes to BLOCKER.
7. Bootstrap proceeds per ADR-0242 §D-5, now with all literals
   sourced from the constant.

## Verification

- [ ] `/specs/platform-constants.json` exists with the §D-1 schema.
- [ ] `oya-shared-platform-constants-kernel` crate builds and
      exports `PLATFORM_OWNER_TENANT_SLUG`,
      `PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS`,
      `PLATFORM_OWNER_BACKWARD_COMPAT_ALIASES`,
      `PLATFORM_OWNER_BRAND_DISPLAY_NAME_DEFAULT`,
      `PLATFORM_OWNER_DSAR_CONTACT_DOMAIN`.
- [ ] The helper functions `dsar_contact_email`, `principal_prefix`,
      `is_platform_owner_slug` exist and are tested.
- [ ] `cargo doc -p oya-shared-platform-constants-kernel` surfaces
      the constants with their documentation.
- [ ] `oya gate validate platform-owner-constant-indirection` exits
      0 on a clean workspace.
- [ ] `oya gate validate no-hardcoded-platform-owner-slug` exits 0
      on a clean workspace (after the migration).
- [ ] `oya gate validate platform-owner-brand-vs-slug-separation`
      exits 0 (UI code uses Fluent; identity code uses constant).
- [ ] `oya gate validate platform-constants-codegen-clean` exits 0
      (generated crate matches codegen output).
- [ ] `oya gate validate reserved-namespace-protection` exits 0
      (Cedar fragment loaded; references the constant via template).
- [ ] Unit test: `is_platform_owner_slug("oyatie") == true`.
- [ ] Unit test: `is_platform_owner_slug("OYATIE") == true`
      (case-fold).
- [ ] Unit test: `is_platform_owner_slug("оyatie") == true`
      (Cyrillic 'о' confusable; UTS #39).
- [ ] Unit test: `is_platform_owner_slug("oyatie-corp") == false`
      directly (prefix family handled at the admission gate, not
      the helper; the helper only matches exact-root forms).
- [ ] Unit test: `is_platform_owner_slug("acme-corp") == false`.
- [ ] Integration test: Cedar fragment `reserved-tenant-namespace.cedar`
      is built from the template and contains the substituted slug.
- [ ] Integration test: tenancy bootstrap migration is built from
      the template and writes `tenant_id = '<slug>'` correctly.
- [ ] Integration test: audit-chain stream provisioner creates
      streams `<slug>.root`, `<slug>.foundry`, etc.
- [ ] Integration test: hypothetical rebrand simulation
      (Appendix B) runs end-to-end on a sandbox cell and yields a
      consistent post-rebrand state.
- [ ] DSAR cascade test against the old-slug-prefixed principal
      after a sandbox-rebrand returns results from the renamed
      tenant.
- [ ] ADR-0242 frontmatter carries `amended_by: ADR-0284`.

## References

### Industry sources

- **Adam Wiggins, "The Twelve-Factor App" (2011, 12factor.net).**
  §III "Config" — definition of config as values that vary between
  deploys. Platform identity does not vary; therefore not config.
- **Stripe Engineering blog — "How and why we built our internal
  developer platform" (2021).** Documents the separation of
  product brand from underlying tenant identifier in Stripe's
  internal tooling.
- **Stripe Engineering blog — "How Stripe's Document Databases
  Supported 99.999% Uptime with Zero-Downtime Data Migrations"
  (2023).** Pattern for sealed-log migration with backward-compat
  aliases.
- **Apple WWDC 2024 keynote — Apple Intelligence introduction.**
  Brand display name vs underlying account identifier separation.
- **Facebook → Meta rebrand (October 2021).** Mark Zuckerberg's
  Founder's Letter; Meta press release; the underlying Facebook
  product slug was preserved while parent-org slug changed. Per
  Forbes 2021 coverage and Bloomberg 2021 financial-impact reporting,
  the rebrand cost approximately $20M in immediate brand-asset
  refresh; the underlying tenant infrastructure (Facebook user IDs,
  OAuth scopes) was preserved. The pattern: corporate-brand rebrand
  without identity-substrate rebrand.
- **Google → Alphabet restructuring (August 2015).** Larry Page's
  founder letter; the Alphabet umbrella was created; Google
  retained its slug; sub-products gained slug-namespaces under
  Alphabet. The pattern: holding-company restructuring with
  underlying-slug preservation.
- **Twitter → X rebrand (July 2023).** Elon Musk's transition;
  public press reporting (Bloomberg, NYT, Reuters 2023) on the
  technical migration cost. The pattern: forced rebrand without
  pre-existing indirection; the migration cost was substantial.
  Cited here as the cautionary example of the cost of not having
  this ADR in place.
- **AWS `arn:aws:iam::aws:` reserved partition.** AWS uses `aws`
  as the platform-owner identifier in IAM ARNs (e.g.,
  `arn:aws:iam::aws:policy/AdministratorAccess`). AWS has never
  rebranded; the reservation has been stable since AWS launch. The
  pattern: reserved-identifier discipline from inception.
- **Cloudflare R2 product launch (2022).** Cloudflare's R2 storage
  product is a brand surface over the underlying Cloudflare tenant
  primitive; the brand surface (R2) is separate from the underlying
  identity machinery.
- **Mozilla rebrand of Firefox sub-brands.** The Firefox sub-product
  family (Firefox Send, Firefox Lockwise, Firefox Monitor) has
  iterated brand surfaces multiple times; the underlying Mozilla
  Account identifier has remained stable since FxA launch in 2013.
- **GitHub rebrand of Atom Editor to "discontinued" status (2022).**
  Demonstrates that brand-surface end-of-life can happen without
  affecting underlying identity (GitHub accounts remain).
- **Twelve-factor cofounder commentary, Adam Wiggins, on identity
  vs config (2014 GitHub gist).** Reaffirms the separation in
  response to the env-var-for-identity question.
- **Unicode Technical Standard #39 (UTS #39) — Unicode Security
  Mechanisms.** Confusable detection.
- **Unicode Technical Report #36 (UTR #36) — Unicode Security
  Considerations.** IDN homograph attacks.

### Regulatory sources

- **GDPR Article 12 (Modalities for the exercise of the rights of
  the data subject).** DSAR response SLA preserved across rebrand
  via alias machinery.
- **GDPR Article 17 (Right to Erasure).** Erasure requests
  referencing old slug must resolve.
- **KR PIPA Article 36.** Erasure equivalent.
- **FRCP 37(e) — Failure to Preserve Electronically Stored
  Information.** Legal-hold preservation across rebrand.
- **Sedona Conference Working Group 1 — The Sedona Principles (3rd
  ed.).** eDiscovery + legal hold authority.
- **SOC 2 Type II Trust Service Criteria.** Auditor expectation of
  uniform identity treatment.
- **ISO 22301:2019 — Security and resilience — Business continuity
  management systems.** Rebrand falls under BC scope.

### Internal portfolio ADRs

- **ADR-0009 — Cell architecture per-tenant per-region.**
  Cell-level isolation primitive; rebrand preserves cell topology.
- **ADR-0049 — Cross-region replication + residency.** Replication
  semantics preserved across rebrand.
- **ADR-0063 — Doc coverage enforcement.** This ADR ships with full
  doc set per the lean-a5-doc-coverage lane.
- **ADR-0064 — Canonical base + localization overlay.** Brand
  display name uses the canonical-base + locale-overlay pattern.
- **ADR-0105 — Thirteen-layer canonical enum.** Layer rules
  unchanged; the constant crate is layer-shared-kernel.
- **ADR-0128 — Hyperscaler architecture invariants.** No-runtime-
  config-of-platform-identity invariant.
- **ADR-0131 — Per-microservice flat layout.** Layout unchanged.
- **ADR-0145 — Inter-microservice communication reform.** Direct
  gRPC + 3 invariants pattern; principal paths use constant.
- **ADR-0183 — Policy engine separation — Cedar app-authz +
  Kyverno admission.** Cedar fragments are templated; substitution
  at build time.
- **ADR-0206 — i18n substrate Fluent + ICU.** Brand display name
  resolved through Fluent.
- **ADR-0211 — In-house Rust-primary tech stack.** Constant crate
  is Rust.
- **ADR-0212 — Buildability doctrine.** This ADR is itself a
  deliverable artifact; the constant crate is buildable from spec.
- **ADR-0216 — Open integration + migration-out policy.** Backward-
  compat alias machinery preserves integration contracts.
- **ADR-0218 — Tenant granular control surface.** Tenant control
  surface uses the constant.
- **ADR-0240 — Sovereign cloud per regional pack.** Data residency
  rules use the constant.
- **ADR-0242 — `oyatie`-is-a-tenant doctrine.** This ADR amends
  ADR-0242 §D-1 by sourcing the literal value from the constant.
- **ADR-0243 — Cedar as universal gate (keystone #2 companion).**
  Cedar fragment templating supports this ADR.
- **ADR-0244 — Tenant as universal scoping primitive (keystone #3
  companion).** Tenant model references the constant.
- **ADR-0245 — Substrate vs Product layering (keystone #4
  companion).** Constant crate is substrate; product surfaces
  resolve through Fluent.
- **ADR-0246 — Policy-engine substrate promotion (keystone #5
  companion).** Policy engine reads templated Cedar fragments.
- **ADR-0247 — Self-hosting / self-modification doctrine (keystone
  #6 companion).** Self-modification workflows respect the
  constant.
- **ADR-0249 — Multi-category marketplace doctrine.** Marketplace
  surfaces resolve brand display name through Fluent.

### Auto-memory feedback

- `feedback_oyatie_is_a_tenant_doctrine` — informs this ADR; this
  ADR refines ADR-0242 by adding indirection.
- `feedback_bominal_inheritance_precedence` — applies; oyatie
  session decisions override Bominal.
- `feedback_no_silent_regression` — this ADR exists because silent
  drift accretion is exactly what Linus-style discipline forbids.
- `feedback_quality_performance_scalability_bar` — reinforced;
  hyperscaler-grade rebrand readiness.
- `feedback_canonical_base_localization` — applies; brand display
  name follows the canonical-base + overlay pattern.
- `feedback_naming_justification` — applies; the constant name
  `PLATFORM_OWNER_TENANT_SLUG` carries its v4-BNF + 12-layer-enum
  justification in this ADR.
- `feedback_automate_everything` — reinforced; the CI lane
  automates literal-detection.
- `feedback_autonomous_implementation_artifacts` — reinforced;
  enables autonomous rebrand-ceremony execution.

---

## Appendix A: Pattern attribution matrix

Per the audit pattern established in ADR-0242 Appendix A, every
architectural decision in this ADR is attributed to a named pattern
+ source + anti-pattern avoided.

| Decision section | Pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (single source of truth) | "Single Source of Truth" / "DRY at Identity Layer" | The Pragmatic Programmer (Hunt + Thomas, 1999); twelve-factor app §III config | "Identity-Literal Drift" — same logical identity scattered as literals |
| D-1 spec + generated crate | "Spec-First Codegen" | gRPC + Protocol Buffers; OpenAPI codegen; AWS API model codegen | "Source-First Multi-Language Drift" — bindings hand-maintained per language |
| D-2 (Cedar template) | "Build-Time Substitution for Signed Artifacts" | Bazel build-time substitution; Cargo build.rs; rustc include_str! | "Runtime Substitution in Signed Artifact" — re-signing on every load |
| D-2 (markdown exemption) | "Documentation Carries Historical Value" | ADR principle of immutable text post-acceptance; git commit history | "Retroactive Doc Rewrite" — rewriting history to match current state |
| D-3 (CI lane for literal detection) | "Negative Pattern Lint" | Clippy lints; ESLint no-restricted-syntax; SwiftLint forbidden-words | "Style-Only Lint" — lint that doesn't catch semantic violations |
| D-4 (brand vs slug separation) | "Apple Intelligence Brand-Surface Pattern" | Apple WWDC 2024; Apple's separation of brand surface from Apple ID account substrate | "Brand-As-Identity Coupling" — rebrand cascades through identity layer |
| D-4 (i18n via Fluent) | "Fluent Message Bundle Localization" | Mozilla Fluent + ICU MessageFormat; ADR-0206 inheritance | "Hardcoded English UI" — non-localisable user-facing text |
| D-5 (reserved namespace in constant) | "Centralised Reserved-Identifier Family" | AWS partition reservations; ICANN reserved TLD list; IETF reserved-name registries | "Scattered Reserved-List Drift" — multiple files declare different reserved sets |
| D-6 (migration ceremony) | "Signed Migration Ledger" | git commit chain; Certificate Transparency append-only log; rustc stage0 bootstrap audit | "Untraceable Rebrand" — migration leaves no signed trail |
| D-6 (audit-chain bridge events) | "Append-Only Bridge Event" | Kafka tombstone-via-new-event; event-sourcing migration pattern; HBase delete-marker | "Sealed-Log Mutation" — modifying an append-only log retroactively |
| D-7 (7-year backward-compat alias) | "Long Sunset for Identity Aliases" | OAuth 2.0 token deprecation windows; ICANN domain hold periods; AWS API version sunset cadence | "Hard Cutover at Rebrand" — breaking customer integrations on day 1 |
| D-8 (brand-of-product separation) | "Product Brand Variation Over Stable Substrate" | Apple product naming (Pages / Numbers / Keynote share Apple ID); Adobe Creative Cloud product brand variation over Adobe ID | "Product-Brand-Locked Identity" — every product brand-change cascades through identity |

---

## Appendix B: Worked example — hypothetical rebrand `oyatie` → `something-else`

To validate that the migration procedure (§D-6) is genuinely
tractable, here is a worked end-to-end example of a hypothetical
future rebrand.

**Scenario:** In 2031, a German trademark holder named "Oyatie GmbH"
files a trademark infringement claim against the platform. Council-
legal concludes the path of least resistance is rebrand. The chosen
new slug is `something-else` (placeholder; actual rebrand-driver
events would pick a real slug). The chosen new brand display name
default is "Something Else."

**Pre-rebrand state (2031-Q2):**

- `/specs/platform-constants.json` declares
  `PLATFORM_OWNER_TENANT_SLUG = "oyatie"`.
- Tenancy table: 47,000 customer tenants + 1 platform-owner tenant
  with `tenant_id = "oyatie"`.
- Audit-chain streams: `oyatie.root`, `oyatie.foundry`,
  `oyatie.security`, `oyatie.finance`, `oyatie.platform-ops`, plus
  approximately 200 sub-stream rollups under `oyatie.*.*`.
- OIDC service principals: approximately 1,400 service principals
  across `oyatie.*` sub-scopes.
- Cedar fragments: 89 fragments referencing the slug via template
  substitution (zero literal references in non-Markdown text, per
  the CI lane).
- Per `feedback_autonomous_implementation_artifacts`: the
  autonomous-masterplan workflows are actively running against
  `oyatie.foundry.ci-agent` principals.

**Rebrand ceremony execution (2031-Q3):**

**Day 1 — Pre-rebrand gates.**

- ADR amendment to ADR-0284 authoring `ADR-XXXX-rebrand-oyatie-to-something-else`,
  with trademark-conflict justification, target completion date
  2031-Q4, and 7-year alias retention.
- Council-architecture + council-product + council-brand +
  council-legal approve in the rebrand ledger.
- Trademark clearance on `something-else` confirmed in US / EU /
  KR / JP / UK / CA.
- Reserved-namespace audit confirms no customer tenant matches any
  variant of `something-else`.
- DR drill on dr cell simulates the rebrand; rollback verified.
- Multispectrum review v2.4.0 APPROVE on the rebrand ChangeSet.

**Day 2 — Stage R-1 (pause).**

- New tenant registrations paused via Cedar policy update.
- In-flight Foundry workflows are allowed to complete (estimated 4
  hours); new workflows queued.
- Audit-chain emissions continue (cannot pause).

**Day 2-3 — Stage R-2 (slug update in spec).**

- `/specs/platform-constants.json` updated:
  - `PLATFORM_OWNER_TENANT_SLUG.value = "something-else"`.
  - `PLATFORM_OWNER_TENANT_SLUG.rebrand_history` appends the
    rebrand record.
  - `PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS.value =
    ["something-else", "something", "somethi", ..., "oyatie",
    "oya", "oyat", "oyati"]`.
  - `PLATFORM_OWNER_BACKWARD_COMPAT_ALIASES.value = ["oyatie"]`.
  - `PLATFORM_OWNER_BRAND_DISPLAY_NAME_DEFAULT.value = "Something Else"`.
  - `PLATFORM_OWNER_DSAR_CONTACT_DOMAIN.value = "something-else.com"`.

**Day 3-4 — Stage R-3 (constant crate rebuild).**

- `cargo build -p oya-shared-platform-constants-kernel` rebuilds.
- All downstream crates rebuild (approximately 600 crates in the
  workspace).
- Cedar template re-render produces 89 new fragment artifacts.
- SQL migration template re-render produces the renamed migration.
- Total build time: 4 hours in the rebuild cell.

**Day 4 — Stage R-4 (migration ledger entry).**

- `/migration-ledger/rebrand/rebrand-2031-q3-oyatie-to-something-else/manifest.json`
  signed under the org root key (YubiKey HSM cluster).

**Day 5 — Stage R-5 (re-sign Cedar fragments).**

- All 89 fragments re-signed.
- Signing emits audit-chain events under both old streams (`oyatie.security.cedar`)
  and new streams (`something-else.security.cedar`).

**Day 5-6 — Stage R-6 (tenancy table rename).**

- Migration `0142_rename_self_tenant.sql` runs.
- Foreign keys cascade: approximately 12 million rows updated
  across the tenancy + identity + audit-chain + finops databases.

**Day 6-7 — Stage R-7 (audit-chain stream creation).**

- New streams `something-else.root`, etc., created.
- Bridge audit events emitted in each old stream:

  ```json
  {
    "event_type": "PlatformOwnerRebrandBridge",
    "old_stream": "oyatie.root",
    "new_stream": "something-else.root",
    "rebrand_id": "rebrand-2031-q3-oyatie-to-something-else",
    "merkle_root_at_bridge": "<hash>",
    "signed_at": "2031-09-15T14:30:00Z"
  }
  ```

**Day 7-8 — Stage R-8 (service-principal reissue).**

- 1,400 service principals reissued under `something-else.*`.
- JWT verifiers accept tokens from either issuer for 90 days.

**Day 8 — Stage R-9 (observability migration).**

- Grafana dashboards updated.
- Recording rules add aliases.

**Day 8-9 — Stage R-10 (external integration update).**

- OAuth consent screens: 12 third-party providers contacted;
  consent screen text updated.
- DNS: `oyatie.com` → `something-else.com` redirect deployed.
- SPF/DKIM/DMARC records updated.
- Container registry org rename queued.

**Day 9-10 — Stage R-11 (documentation update sweep).**

- Runbooks, READMEs, root `CLAUDE.md` updated.
- ADRs NOT retroactively rewritten; historical-slug-intentionally
  exemption preserves them.

**Day 10 — Stage R-12 (unpause).**

- New tenant registrations resume.
- Foundry workflows resume.
- Reserved-namespace gate now rejects any registration of `oyatie`
  variants (in the alias list).

**Day 10-11 — Stage R-13 (verification).**

- All CI lanes exit 0.
- DSAR cascade test against an `oyatie.foundry.engineer.<id>`
  principal returns results from the renamed `something-else.foundry.engineer.<id>`
  tenant.
- Audit-chain old-stream readback proves bridge event presence
  and Merkle validity.

**Total elapsed time:** 11 days from pre-rebrand gates to
verification complete.

**Customer impact:**

- Customer OAuth clients pointing to `oauth.oyatie.com` continue to
  work via redirect for 7 years.
- Customer webhook URLs referencing `oyatie.com` continue to work
  via redirect for 7 years.
- Customer API integrations referencing the old slug in tenant_id
  fields continue to work via the alias-resolution layer for 7
  years.
- Customer-facing UI shows "Something Else" by Day 10.

**Comparison to a rebrand WITHOUT this ADR's indirection:**

| Concern | With ADR-0284 | Without ADR-0284 |
|---|---|---|
| Elapsed time | 11 days | 6-12 weeks |
| Engineering effort | 1 ChangeSet | 100+ ChangeSets |
| Risk of missed literal | Bounded by CI lane | Unbounded |
| Cedar fragment re-sign | Templated; auto | Manual per fragment |
| Audit-chain bridge | Mechanically clean | Manual reconciliation |
| Customer-integration breakage | None (alias works) | Likely (alias hand-coded) |
| Migration ledger | Signed; clean | Best-effort |
| Regression risk | Low | High |
| Cost (engineering hours) | ~200 hours | ~5000+ hours |

The 25x cost reduction is the justification for this ADR.

**End-state (2031-Q4 + 7 years = 2038-Q4):**

- The `oyatie` alias retires on the 7-year anniversary.
- Old OIDC issuer decommissioned in Zitadel.
- Old Cedar fragments archived; effective policy is post-rebrand only.
- Old slug remains permanently in `PLATFORM_OWNER_RESERVED_NAMESPACE_ROOTS`
  to prevent third-party reclamation.
- Old audit-chain streams remain Merkle-sealed and queryable for
  the legal retention period (potentially indefinitely under
  active legal hold).

---

*End of ADR-0284.*
