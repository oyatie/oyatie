# Plan: ontology-entity-type-schema-evolution

vertical: ontology  
crate: oya-ontology-kernel  
branch: feat/task-ontology-entity-type-schema-evolution-2026-05-28  

---

## Objective

Extend `OntologyEngine` with a revision-aware schema evolution path for entity types.
Today `register_entity_type` rejects a second registration of any known id with
`DuplicateEntityType`.  This slice adds `evolve_entity_type` that admits a
higher-revision candidate **only** when the change is backward-compatible
(additive-only).

---

## Subtasks

### ST1 — Backward-compatibility checker + new error variants

**What to build**

- Add two new `OntologyEngineError` variants (no changes to existing variants):
  - `NonMonotonicRevision` — candidate revision ≤ stored revision.
  - `IncompatibleSchemaEvolution` — a prior property was removed, or its
    `tier`, `data_class`, or `required` flag was mutated.
- Add a free function `check_schema_compatibility(prior: &EntityTypeDefinition, candidate: &EntityTypeDefinition) -> Result<(), OntologyEngineError>` inside `src/lib.rs` (private, `pub(crate)` not needed — used only in ST2).
  - For every property in `prior.properties` assert an identical-named property
    exists in `candidate.properties` with unchanged `tier`, `data_class`, and
    `required`.
  - New properties in `candidate` that are absent from `prior` are allowed.
  - Revision monotonicity is **not** checked here (checked in the engine method).

**Acceptance**

- `cargo check -p oya-ontology-kernel --all-targets` passes.
- Unit test in `src/lib.rs` (`#[cfg(test)] mod schema_evolution_tests`):
  - Additive property (new property added, prior properties unchanged, revision +1) → `Ok(())`.
  - Tier mutation of an existing property → `Err(IncompatibleSchemaEvolution)`.
  - `data_class` mutation → `Err(IncompatibleSchemaEvolution)`.
  - `required` flip → `Err(IncompatibleSchemaEvolution)`.
  - Property removed from candidate → `Err(IncompatibleSchemaEvolution)`.

---

### ST2 — `OntologyEngine::evolve_entity_type`

**What to build**

- Add `pub fn evolve_entity_type(&mut self, definition: EntityTypeDefinition) -> Result<EntityTypeId, OntologyEngineError>` to the `impl OntologyEngine` block in `src/lib.rs`.
- Behaviour:
  1. Look up the existing tenant-scoped definition by `(tenant_id, id.value)`.
  2. If **not found** → insert as a first registration (identical to
     `register_entity_type`) and return `Ok(id)`.
  3. If found → enforce `definition.revision > stored.revision`; on failure
     return `Err(NonMonotonicRevision)`.
  4. Run `check_schema_compatibility(stored, &definition)`; propagate any error.
  5. Replace the stored definition and return `Ok(id)`.
- `register_entity_type` is **not modified**; it continues to reject duplicates
  with `DuplicateEntityType`.

**Acceptance**

- `cargo nextest run -p oya-ontology-kernel` green.
- Test matrix (in `#[cfg(test)] mod schema_evolution_tests`):
  - (a) First registration via `evolve_entity_type` succeeds and is queryable.
  - (b) Monotonic additive evolution (revision N→N+1, new property added) returns
    the `EntityTypeId` and the stored definition reflects the new revision and
    property set.
  - (c) Equal revision → `Err(NonMonotonicRevision)`.
  - (d) Lower revision → `Err(NonMonotonicRevision)`.
  - (e) Breaking change (tier mutation) with higher revision →
    `Err(IncompatibleSchemaEvolution)`.
  - (f) Existing `register_entity_type` tests remain green (no regressions).

---

## Acceptance summary

| Gate | Command | Expected |
|------|---------|----------|
| Type-check | `cargo check -p oya-ontology-kernel --all-targets` | 0 errors |
| Tests | `cargo nextest run -p oya-ontology-kernel` | all green |
| Regression | existing `backbone_tests` and `tests/` integration tests | all green |

---

## Boundaries

- **No new crates.** All code goes in `crates/oya-ontology-kernel/src/lib.rs`.
- **No changes** to `pillar.rs`, `tests/link_action_invariants.rs`, or
  `tests/types.rs`.
- **No link-action or pillar variants touched.** Only two new
  `OntologyEngineError` variants added.
- **No REST/gRPC surface.** Pure kernel logic only.
