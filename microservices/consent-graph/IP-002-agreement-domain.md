# IP-002: agreement-domain — lifecycle invariants + scope/terms model

- Bounded context: agreement
- Layer: domain (per ADR-0105)
- Crate: `oya-consent-graph-agreement-domain`
- Acceptance status: ga
- Authority: ADR-0214 §2, ADR-0056 (domain = pure business rules), ADR-0105 (domain may depend on
  kernel only; never on adapter/usecase).
- Depends on: `oya-consent-graph-agreement-kernel`.

## 1. Goal

Encode the **business rules** that the kernel types alone cannot enforce — rules that need cross-field
reasoning, structural validation, or canonical normalization. Like the kernel layer, this is pure: no
I/O, no async runtime, no clock-as-side-effect (clock arrives via port).

## 2. Scope

In:
- Scope-narrowing semantics (intersection/subset checks for amendments).
- Terms validation (purpose-of-use ∈ catalogue, redaction-config compatibility with mode).
- Sovereignty constraint resolver (which grantee regions are eligible given grantor region + pack
  overlay + cross-border-transfer flag).
- Predicate parser for `EntityScope.predicate` (lex + parse only; evaluation belongs to
  `enforcement-domain`).
- Agreement template materialization (5 starter templates).
- Versioned-amendment computation (delta between two `EntityScope` instances).

Out:
- Actual Cedar policy compilation (→ `enforcement-domain`).
- Persistence (→ `agreement-adapter`).
- Pulsar emission (→ `agreement-adapter`).

## 3. Scope-narrowing semantics

When an agreement is amended (`Active → Drafted (new version)`), the new scope MUST be a **subset**
of the old scope, unless explicit grantee re-acceptance is requested. Why subset-only by default: a
strict-monotone narrowing rule lets grantees rely on cached field lists without re-evaluating; broadening
requires full re-handshake.

```rust
pub fn is_subset(new: &EntityScope, old: &EntityScope) -> Result<bool, ScopeError> {
    if new.entity_type != old.entity_type { return Ok(false); }
    let new_fields = resolve_field_set(&new.field_set, &SchemaRegistry::current(&new.entity_type)?)?;
    let old_fields = resolve_field_set(&old.field_set, &SchemaRegistry::current(&old.entity_type)?)?;
    if !new_fields.is_subset(&old_fields) { return Ok(false); }
    if predicate_strictly_narrows(&new.predicate, &old.predicate)?.is_none() { return Ok(false); }
    Ok(true)
}
```

`predicate_strictly_narrows` performs syntactic AST comparison; full semantic equivalence is undecidable,
so we accept conservatively (false-negative = require re-handshake, never false-positive).

## 4. Terms validation

```rust
pub fn validate_terms(terms: &SharingTerms, scope: &EntityScope) -> Result<(), Vec<TermsError>> {
    let mut errors = Vec::new();
    // 4.1: purpose-of-use must be in the catalogue (curated list of 142 verticals × purposes)
    if !PURPOSE_OF_USE_CATALOGUE.contains(&terms.purpose_of_use) {
        errors.push(TermsError::UnknownPurposeOfUse(terms.purpose_of_use.clone()));
    }
    // 4.2: Aggregate mode requires k_anonymity ≥ 2 (per ADR-0214 §2.2)
    if matches!(terms.mode, SharingMode::Aggregate) && terms.k_anonymity.map(|k| k < 2).unwrap_or(true) {
        errors.push(TermsError::AggregateModeRequiresKAnonymity);
    }
    // 4.3: Redaction must reference only fields present in scope
    for redacted in terms.redaction.fields() {
        if !scope.field_set.contains(redacted) {
            errors.push(TermsError::RedactionFieldNotInScope(redacted.clone()));
        }
    }
    // 4.4: DP config implies aggregate mode
    if terms.differential_privacy.is_some() && !matches!(terms.mode, SharingMode::Aggregate) {
        errors.push(TermsError::DpRequiresAggregateMode);
    }
    // 4.5: max_qps sanity bounds (1..=100_000)
    if let Some(q) = terms.max_qps {
        if q == 0 || q > 100_000 { errors.push(TermsError::MaxQpsOutOfRange(q)); }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

The `PURPOSE_OF_USE_CATALOGUE` is curated in
`microservices/consent-graph/specs/purpose-of-use-catalogue.json` (142 entries across 14 verticals); a
grantor proposing a novel purpose-of-use must first land an ADR-SVC-CG-* extending the catalogue.

## 5. Sovereignty constraint resolver

```rust
pub fn resolve_eligible_grantee_regions(
    grantor: TenantId,
    grantor_region: Region,
    grantee: TenantId,
    grantee_known_regions: &[Region],
    cross_border_transfer_permitted: bool,
    pack_overlay: Option<PackId>,
) -> Result<Vec<Region>, SovereigntyError> {
    let base = if cross_border_transfer_permitted {
        grantee_known_regions.to_vec()
    } else {
        grantee_known_regions.iter().filter(|r| r == &&grantor_region).cloned().collect()
    };

    // Pack overlay restricts further (e.g., KR pack forbids EU regions for KR-origin data unless
    // adequacy decision exists).
    let restricted = if let Some(pack) = pack_overlay {
        PackOverlayResidencyRules::for_pack(pack).restrict(&base, grantor_region)?
    } else { base };

    if restricted.is_empty() {
        return Err(SovereigntyError::NoEligibleGranteeRegion {
            grantor_region, grantee, pack_overlay,
        });
    }
    Ok(restricted)
}
```

Pack overlay rules ship in `iac/kustomize/overlays/<pack>/sovereignty-rules.yaml`; the domain reads
them via a `PackOverlayResidencyRules` port (injected, not hardcoded — pack rules evolve faster than
code).

## 6. Predicate parser

The `EntityScope.predicate` is a Cedar-flavored expression authored by the grantor. The domain layer
performs **lex + parse only**; semantic evaluation belongs to `enforcement-domain` where Cedar lives.

```rust
pub fn parse_predicate(src: &str) -> Result<PredicateAst, PredicateParseError> { /* recursive descent */ }
```

Predicate AST shape (subset of Cedar):
- `Var(name)` — `principal`, `resource`, `context`
- `Field(target, name)`
- `In(left, set)`
- `Eq(left, right)`, `Neq(left, right)`
- `And(left, right)`, `Or(left, right)`, `Not(inner)`
- `Lit(JsonValue)`

Why a strict subset: full Cedar expression authoring is reserved for advanced operators; the predicate
sublanguage is what 95% of agreements need + is statically auditable.

## 7. Agreement template materialization

5 starter templates (one per Big-5 vertical):

| Template ID | Vertical | Default mode | Default k | Default expiration |
|-------------|----------|--------------|-----------|--------------------|
| `tmpl-supply-chain-po-visibility` | Supply chain | Projection | n/a | 1y |
| `tmpl-healthcare-eligibility-verification` | Healthcare | AttestedQuery | n/a | 90d |
| `tmpl-banking-account-share` | Banking | AttestedQuery+Projection | n/a | 180d |
| `tmpl-marketplace-cohort-stats` | Marketplace | Aggregate | k=5, ε=1.0 | 1y |
| `tmpl-b2c-order-tracking` | B2C | Projection | n/a | until-revoked-or-90d-inactive |

Templates ship in `specs/agreement-templates/<id>.json`. The domain layer's
`materialize_template(id, grantor, grantee, overrides)` returns a `DataSharingAgreement` in the
`Drafted` state with template defaults applied + grantor/grantee bound + overrides merged.

Templates are *Drafted-only*; advancing to `Offered` is always an explicit action by the grantor's
data-steward. CI emits a warning if a template fingerprint is unchanged at offer-time (per PHASE-01
risk R-6).

## 8. Versioned-amendment delta

```rust
pub struct AmendmentDelta {
    pub old_version: u64,
    pub new_version: u64,
    pub fields_added: Vec<FieldName>,
    pub fields_removed: Vec<FieldName>,
    pub predicate_narrowing: NarrowingKind,    // Strict | NonStrict | Equivalent
    pub terms_diff: TermsDiff,
    pub requires_grantee_re_acceptance: bool,  // true iff broadening detected
}

pub fn compute_delta(old: &DataSharingAgreement, new: &DataSharingAgreement) -> AmendmentDelta { ... }
```

Used by `agreement-usecase::amend` to decide whether re-acceptance is required. Always emitted on the
`oya.consent-graph.agreement-amended` audit event (full delta is part of the seal).

## 9. Tests

- `is_subset_strict_monotone` — adding a field reports non-subset.
- `validate_terms_aggregate_requires_k` — Aggregate + k=None → error.
- `validate_terms_dp_requires_aggregate` — Projection + DP cfg → error.
- `resolve_eligible_grantee_regions_pack_overlay_restricts` — KR pack + EU grantee + no adequacy → empty.
- `predicate_parser_subset` — full Cedar syntax rejected; sublanguage accepted.
- `materialize_template_supply_chain_defaults` — defaults applied correctly.
- `compute_delta_field_added_requires_reacceptance` — adding a field → `requires_grantee_re_acceptance=true`.

100% line coverage required.

## 10. Dependencies

- `oya-consent-graph-agreement-kernel`
- `oya-shared-tenant-id`
- `oya-shared-region`
- `oya-shared-pack-overlay-rules` (port; impl in adapter layer)
- `serde`, `thiserror`
- `nom = "7"` (predicate parser only)

**No** Postgres, **no** Pulsar, **no** Cedar runtime — those are downstream.

## 11. Verification

- `cargo build` + `cargo test` clean.
- `oya-check-layer-bnf-conformance` clean.
- `oya-check-pack-overlay-rules-injected` (custom lint per ADR-0064 canonical-base neutrality) clean.

## 12. Public surface

- `validate_terms`, `validate_scope`, `is_subset`, `resolve_eligible_grantee_regions`,
  `parse_predicate`, `materialize_template`, `compute_delta`.
- Domain errors: `TermsError`, `ScopeError`, `SovereigntyError`, `PredicateParseError`.

Any addition requires a public-API-surface snapshot delta + reviewer-agent approval per
ADR-0064.

## Wave 15-IP-substance counterpart evidence

Preserved as substantive. Counterpart anchors: OneTrust and TrustArc provide consent/preference record semantics; Cookiebot provides consent-state categories; Snowflake and Databricks provide sharing scopes without consent-domain invariants. This domain IP keeps Oyatie-specific scope narrowing, sovereignty resolution, predicate parsing, and template materialization in pure rules so counterpart-style UX and data-share concepts cannot bypass agreement validity.
