# Cedar policy discipline (canonical)

Authority: ADR-0064 canonical-base + localization-packs (SWEEP-I Slice 1).
Schema: `specs/policy/cedar-scope-schema.md`.
Canonical imports: `microservices/governance/policy/cedar-canonical-imports.cedar`.

## Naming convention

Every µservice's `microservices/<ms>/policy/` directory authors exactly
these four canonical archetypes:

```
microservices/<ms>/policy/tenant-scope.cedar    # tenant/operator ABAC
microservices/<ms>/policy/ci-scope.cedar        # lane runner + named CI roles
microservices/<ms>/policy/auditor-scope.cedar   # external auditor JIT
microservices/<ms>/policy/public-read.cedar     # PublicAnonymous allow-list
```

Additional scopes (e.g. `providers-ci-scope.cedar` for sub-bounded-contexts)
are permitted only with an ADR documenting the extension. The four canonical
archetypes remain mandatory.

## Default-deny rule

Cedar's default decision is `forbid`. Therefore:

1. **Every permitted action must have an explicit `permit` rule**.
2. **Missing permit ⇒ DENY**. Do not rely on absence; declare intent.
3. **Explicit deny wins**. `forbid` overrides `permit` for the same triple.

## Defence-in-depth FORBID pattern (mandatory)

Every per-µservice file MUST embody the four canonical FORBID purposes,
even when also enforced upstream (e.g. by the API gateway or by service
mesh authz). Defence-in-depth means each layer refuses independently.

1. **F-CROSS-TENANT** — refuse cross-tenant resource access regardless of permits
2. **F-CROSS-PACK** — refuse cross-residency-pack data egress
3. **F-EXPIRED-TOKEN** — refuse expired/rotated credentials
4. **F-LEAST-PRIVILEGE** — exhaustive allow-list when principal is anonymous

Concrete wording may vary; the structural validator checks that each
purpose is detectable.

## Header comment requirement

Every per-µservice file's first 10 lines MUST include:

- `// Cedar policy fragment: <filename>`
- `// Scope: <microservice> µservice — <one-line purpose>`
- `// Imports: microservices/governance/policy/cedar-canonical-imports.cedar`
- `// Related: <links to threat-model.md, dpia.md, data-residency.md as relevant>`

## Body section comment requirement

Every per-µservice file MUST be sectioned with these comment dividers:

```cedar
// =============================================================================
// PERMITS
// =============================================================================

// P1 — <intent>
permit (...) when { ... };

// =============================================================================
// FORBIDS — defence-in-depth
// =============================================================================

// F1 — <intent>
forbid (...) when { ... };

// =============================================================================
// SCHEMA HINTS
// =============================================================================
//
// (µservice-specific entities + actions not in canonical imports)

// =============================================================================
// FRAMEWORK MAPPING
// =============================================================================
//   P1 → <framework section>
//   F1 → <framework section>
```

## What stays per-µservice (post-SWEEP-I)

- The µservice-specific permit rules expressing authorization logic
- The µservice-specific forbid rules implementing defence-in-depth
- Schema hints for µservice-specific entities + actions
- Framework mapping linking each P/F to standards

## What is removed (post-SWEEP-I)

- Generic Cedar 3.x version preamble (canonical-declared)
- Generic entity-type sketches (canonical-imported via header comment)
- Generic context.now / context.allow_listed_github_ranges declarations
- Generic decision-order narrative

## Validation

`retired CLI cedar-structural-validator` validates every per-µservice cedar:

1. Filename matches canonical archetype
2. Header cites `cedar-canonical-imports.cedar`
3. PERMITS block exists with ≥1 P-rule
4. FORBIDS block exists with ≥1 F-rule
5. Each canonical F-purpose detectable
6. SCHEMA HINTS block closes the file
7. FRAMEWORK MAPPING block follows SCHEMA HINTS

## References

- ADR-0064 canonical-base + localization-packs
- ADR-0117 data-residency
- ADR-0131 per-microservice flat layout
- `specs/policy/cedar-scope-schema.md`
- `microservices/governance/policy/cedar-canonical-imports.cedar`
