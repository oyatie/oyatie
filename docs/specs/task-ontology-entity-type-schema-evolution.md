# Spec: ontology-entity-type-schema-evolution

| Field | Value |
|-------|-------|
| Vertical | ontology |
| Crate | `ontology-kernel` |
| Stage | SPEC |
| ADR baseline | Bominal-ADR-0132 (pillar isolation), ADR-0083 (Tier 3 test exemption) |
| Branch | `feat/task-ontology-entity-type-schema-evolution-2026-05-28` |

---

## Objective

Add revision-aware schema evolution to `OntologyEngine`. Today
`register_entity_type` rejects any re-registration of a known `(tenant_id, id)`
pair with `DuplicateEntityType`, leaving no forward path for schema change.
This slice introduces `evolve_entity_type`, a separate method that admits a
higher-revision candidate for an existing entity type **only** when the change is
backward-compatible (additive-only), enforces strictly-monotonic revision
increments, and rejects breaking changes or non-monotonic edits with new typed
errors. First registration of an unseen id continues to work identically to
`register_entity_type`.

---

## Scope and boundaries

- **In scope:** `crates/ontology-kernel/src/lib.rs` only.
- **Out of scope:** REST/gRPC adapters, Postgres persistence, link/action type
  evolution, pillar annotation evolution, cross-tenant schema propagation.
- **Not modified:** `src/pillar.rs`, `tests/link_action_invariants.rs`,
  `tests/types.rs`, root `Cargo.toml`, any other crate.
- **No new crates** (ADR-0509 / flat-crate doctrine).

---

## Data model (unchanged structs, reproduced for reference)

```rust
pub struct EntityTypePropertyDefinition {
    pub name: String,
    pub tier: PropertyTier,
    pub data_class: PrivacyDataClass,
    pub required: bool,
}

pub struct EntityTypeDefinition {
    pub tenant_id: String,
    pub id: EntityTypeId,
    pub display_name: Classified<String>,
    pub properties: Vec<EntityTypePropertyDefinition>,
    pub revision: u32,            // <-- already present; engine ignores it today
    pub pillar: Option<OntologyPillar>,
}
```

`revision` is already on `EntityTypeDefinition` but is ignored by
`register_entity_type`. This slice makes it semantically meaningful inside
`evolve_entity_type`.

---

## New error variants

Added to the existing `OntologyEngineError` enum (no existing variants changed):

| Variant | Trigger |
|---------|---------|
| `NonMonotonicRevision` | `candidate.revision <= stored.revision` |
| `IncompatibleSchemaEvolution` | A property in the prior definition is absent from the candidate, or its `tier`, `data_class`, or `required` flag differs |

---

## Module layout (flat clean-arch mod pattern)

All new code lands in `src/lib.rs` alongside existing engine logic. No new
source files or modules are introduced. The compatibility checker is a
crate-private free function:

```
src/
  lib.rs          ← check_schema_compatibility() + evolve_entity_type() added here
  pillar.rs       ← unchanged
```

---

## API contract

### `OntologyEngine::evolve_entity_type`

```rust
pub fn evolve_entity_type(
    &mut self,
    definition: EntityTypeDefinition,
) -> Result<EntityTypeId, OntologyEngineError>
```

**Preconditions** (same as `register_entity_type`):
- `definition.tenant_id` must satisfy `validate_ontology_tenant` (prefix `ten_`,
  non-empty suffix); otherwise `InvalidTenantId`.
- `definition.display_name` must be non-empty; otherwise `EmptyDisplayName`.
- `definition.properties` must be non-empty; otherwise `EmptyProperties`.
- Each property name must be non-empty; otherwise `EmptyPropertyName`.

**Evolution semantics** (new, applied only when the id already exists):

1. Revision monotonicity: `definition.revision > stored.revision`; on failure →
   `NonMonotonicRevision`.
2. Backward compatibility: every property in the stored definition must exist in
   `definition.properties` with identical `tier`, `data_class`, and `required`;
   otherwise → `IncompatibleSchemaEvolution`.
3. On success: stored definition is **replaced** with `definition`; `Ok(id)`
   returned.

**First-registration path** (id not yet registered for tenant):
- Identical to `register_entity_type` path; `definition` is inserted and
  `Ok(id)` is returned. No `DuplicateEntityType` error is possible via this
  method.

**`register_entity_type` is unchanged**: continues to reject duplicate ids with
`DuplicateEntityType`.

---

## Proto3 contract (informational — no new .proto files in this slice)

The revision field already maps to the existing `revision` uint32 on any future
`EntityTypeDefinition` proto message. This slice is pure kernel; the proto
surface is not updated here.

---

## OpenAPI 3.2.0 contract (informational — no REST changes in this slice)

A future REST adapter would expose:

```yaml
# PUT /tenants/{tenantId}/entity-types/{entityTypeId}
# Returns 200 on accepted evolution, 409 on NonMonotonicRevision or
# IncompatibleSchemaEvolution, 422 on input validation errors.
```

This slice does not touch any REST adapter.

---

## Testing strategy

All tests reside in `src/lib.rs` under `#[cfg(test)] mod schema_evolution_tests`.

### ST1 unit tests (compatibility checker)

| Test | Input | Expected |
|------|-------|----------|
| `additive_property_is_accepted` | prior has `{name}`, candidate has `{name, email}`, revision +1 | `Ok(())` |
| `tier_mutation_rejected` | prior `name: Scalar`, candidate `name: Vector` | `Err(IncompatibleSchemaEvolution)` |
| `data_class_mutation_rejected` | prior `name: InternalOnly`, candidate `name: PiiIdentifying` | `Err(IncompatibleSchemaEvolution)` |
| `required_flip_rejected` | prior `required: true`, candidate `required: false` | `Err(IncompatibleSchemaEvolution)` |
| `property_removal_rejected` | candidate drops a prior property | `Err(IncompatibleSchemaEvolution)` |

### ST2 engine tests

| Test | Scenario | Expected |
|------|----------|----------|
| `first_registration_via_evolve` | unseen id | `Ok(id)`, queryable |
| `monotonic_additive_evolution_accepted` | revision 1→2, new property added | `Ok(id)`, stored revision=2 |
| `equal_revision_rejected` | revision 1→1 | `Err(NonMonotonicRevision)` |
| `lower_revision_rejected` | revision 2→1 | `Err(NonMonotonicRevision)` |
| `breaking_change_higher_revision_rejected` | revision 1→2, tier mutated | `Err(IncompatibleSchemaEvolution)` |

### Regression

All existing `backbone_tests` and integration tests in `tests/` must remain
green.

---

## Verification gates

```
cargo check -p ontology-kernel --all-targets   # 0 errors
cargo nextest run -p ontology-kernel           # all green
```
