---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-009-cms-collection
status: pending
execution_unit: ChangeSet
owner: axis-sites
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: cms-collection BC

## Intent

Author the `cms-collection` BC per ADR-SITES-0005 (hybrid portable-text + relational). Implements `CollectionType`, `Entry`, `FieldDefinition`, `Relationship`. Schema versioning (forward-compatible migrations + LEAN refuse on breaking). Per-tenant scoping.

## ChangeSet boundary

8 crates: `oya-sites-cms-collection-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,app}`. AC-04 covered.

## Acceptance Gates

```bash
cargo nextest run -p oya-sites-cms-collection-domain -- schema_version_monotonic
cargo nextest run -p oya-sites-cms-collection-domain -- field_definition_validation
cargo nextest run -p oya-sites-cms-collection-adapter-postgres -- query_1000
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice sites
```

## Test Plan

- Unit: schema-version monotonicity.
- Unit: field-definition validation (type narrowing, required-flag).
- Unit: relationship resolution (intra-collection + inter-collection).
- Integration: 1000-entry collection query p95 ≤ 150ms.
- Integration: cross-tenant collection-reference refused (Cedar).

## References

- ADR-SITES-0005 (CMS-collection hybrid model).
- ADR-0105, ADR-0131.
- Sanity portable-text spec — `sanity.io/docs/presenting-block-text`.
- Strapi field-definition model — `strapi.io/docs`.
