---
doc_class: Specification
template_id: TPL-SPEC-POLICY
canonical_authority: ADR-0064 (canonical-base + localization-packs)
overlay_authority: ADR-0131 (per-microservice flat layout)
created_at: 2026-05-18
status: active
governs:
  - "microservices/<ms>/policy/tenant-scope.cedar"
  - "microservices/<ms>/policy/ci-scope.cedar"
  - "microservices/<ms>/policy/auditor-scope.cedar"
  - "microservices/<ms>/policy/public-read.cedar"
---

# Cedar policy scope schema — canonical envelope (SWEEP-I Slice 1)

This document declares the four canonical Cedar policy SHAPES every µservice
authors. Each per-µservice `*.cedar` file inherits this canonical envelope and
preserves only the µservice-specific permit/forbid RULES.

Cedar version: **3.x** (Amazon Verified Permissions stable).
Evaluation point: API gateway (Envoy ext_authz) on every authorized read/write.
Default: **forbid** (Cedar default). Permits then forbids; explicit-deny wins.

## Canonical scope archetypes (the SHAPE)

Every µservice MUST author exactly these four policy files. Additional
scopes are permitted only with an ADR justifying the extension.

| Archetype | Filename | Principal lattice | Cardinality of permits |
|---|---|---|---|
| tenant-scope | `tenant-scope.cedar` | TenantOperator + InternalEngineer | µservice-specific |
| ci-scope | `ci-scope.cedar` | LaneRunner + named roles | µservice-specific |
| auditor-scope | `auditor-scope.cedar` | ExternalAuditor | µservice-specific |
| public-read | `public-read.cedar` | PublicAnonymous (allow-list) | µservice-specific |

## Canonical entity/action/context signature

Every µservice's policy fragments import the canonical entity declarations
from `microservices/governance/policy/cedar-canonical-imports.cedar`, then
declare only the µservice-specific entities + actions on top.

### Canonical entities (always imported)

- `TenantOperator { tenant_id, subject, owned_microservices, residency_pack }`
- `InternalEngineer { subject, residency_pack }`
- `ExternalAuditor { subject, audit_window_id, audit_window_start,
    audit_window_end, token_age_seconds, scope_microservices, scope_packs,
    baa_acknowledged }`
- `LaneRunner { runner_id, registered_lanes }`
- `PublicAnonymous` (singleton)
- `Tenant { tenant_id, residency_pack }`
- `Cell { cell_id, tenant_id, residency_pack }`
- `Capability { tier, eu_ai_act_risk_class }`
- `Pack { pack_name }`
- `AuditChainSeal { sealed_at, merkle_root, scope_microservice }`

### Canonical context (always available)

- `context.now` — current timestamp (Long)
- `context.allow_listed_github_ranges` — github webhook source CIDR set
- `context.registered_lanes` — registry-driven set of valid lane ids
- `context.tenant_id` — header-extracted tenant
- `context.residency_pack` — header-extracted pack

## Canonical defence-in-depth FORBID patterns (mandatory)

Every per-µservice policy MUST include the four canonical FORBIDS by
*purpose* (the µservice may use different concrete wording, but the
intent must be present and detectable by structural validator):

1. **F-CROSS-TENANT** — refuse cross-tenant data access regardless of permits
2. **F-CROSS-PACK** — refuse cross-residency-pack data egress
3. **F-EXPIRED-TOKEN** — refuse expired/rotated credentials
4. **F-LEAST-PRIVILEGE** — exhaustive allow-list when principal is anonymous

## Naming convention

- Permits: `// P<N> — <one-line intent>`
- Forbids: `// F<N> — <one-line intent>`
- Permits are positive-form (allow when conditions hold)
- Forbids are defence-in-depth (refuse-all unless explicit exception)
- Decision order: permits, then forbids (cedar default is forbid, so missing
  permit ⇒ deny; explicit forbid overrides explicit permit)

## Framework mapping requirement

Every per-µservice cedar file MUST close with a `// FRAMEWORK MAPPING` block
mapping each P/F to the relevant standard (GDPR / KR PIPA / HIPAA / ISO 27001
/ SOC 2 / SLSA / EU AI Act / OWASP API Top 10).

## Schema-hints requirement

Every per-µservice cedar file MUST close with a `// SCHEMA HINTS` block
declaring µservice-specific entities + actions NOT covered by canonical
imports.

## Per-µservice file checklist (what stays per-µservice)

- Permit rules expressing the µservice's authorization logic
- Forbid rules implementing defence-in-depth for µservice-specific resources
- Schema hints for µservice-specific entities + actions
- Framework mapping linking each rule to standards

## Per-µservice file checklist (what is removed by SWEEP-I)

- Cedar 3.x version header (canonical-declared)
- Generic entity-type sketches (canonical-imported via header comment)
- Generic context.now / context.allow_listed_github_ranges declarations
- Generic decision-order narrative (canonical-declared)

## Validation

The `oya gate cedar-structural-validator` validates every per-µservice
`*.cedar` against:
1. Filename matches one of the four canonical archetypes
2. Header comment cites `cedar-canonical-imports.cedar`
3. PERMITS block exists with at least one P-rule
4. FORBIDS block exists with at least one F-rule
5. SCHEMA HINTS block closes the file
6. FRAMEWORK MAPPING block follows SCHEMA HINTS

## References

- ADR-0064 canonical-base + localization-packs
- ADR-0117 data-residency
- ADR-0131 per-microservice flat layout
- `docs/standards/cedar-policy-discipline.md`
- `microservices/governance/policy/cedar-canonical-imports.cedar`
