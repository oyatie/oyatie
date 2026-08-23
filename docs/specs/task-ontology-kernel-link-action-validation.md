# Spec: task-ontology-kernel-link-action-validation

## Objective

Extend `ontology-kernel` with stronger structural validation for
`LinkTypeDefinition` and `ActionTypeDefinition` at `OntologyEngine` registration
time. The three concrete goals are:

1. **Endpoint-reference validation** — reject any link or action type whose
   endpoint `EntityTypeId` was not previously registered for the same tenant,
   with a dedicated `OntologyEngineError::UnknownEntityTypeEndpoint` variant.
2. **Pillar-consistency enforcement** — reject link types that bind an
   org-pillar endpoint to a person-pillar endpoint
   (Bominal-ADR-0132 org/person isolation), with a dedicated
   `OntologyEngineError::CrossPillarLink` variant.
3. **Documentation** — crate-level rustdoc and lane-namespaced docs describing
   every new invariant and error variant.

All changes are pure-Rust, deterministic, and in-memory. There is no external
service dependency, no REST/gRPC/event surface, and no new crate.

## Vertical and crate scope

- **Vertical**: ontology
- **Sole crate**: `crates/ontology-kernel`
- **Branch**: `feat/task-ontology-kernel-link-action-validation-2026-05-28`
- **No root `Cargo.toml` edit**: this task extends an existing crate only.

## External contract surface

`ontology-kernel` is a pure in-memory kernel library. It exposes no HTTP
endpoints, no gRPC service definitions, and no event topics. There is therefore
no OpenAPI 3.2.0, AsyncAPI 3.1.0, or proto3 contract surface for this task.

The public Rust API surface that is extended:

- `OntologyEngineError` enum: two new variants added (`UnknownEntityTypeEndpoint`,
  `CrossPillarLink`).
- `EntityTypeDefinition`: gains an optional `pillar: Option<OntologyPillar>`
  field and a `with_pillar(pillar: OntologyPillar) -> Self` builder method.
  The existing `EntityTypeDefinition::new` signature is unchanged.
- `OntologyEngine::register_link_type`: enforces endpoint-reference and
  pillar-consistency invariants.
- `OntologyEngine::register_action_type`: enforces endpoint-reference
  invariant.

## Mod layout (flat clean-arch inside `src/`)

```
crates/ontology-kernel/src/
  lib.rs          — module doc, re-exports, OntologyEngine, all type definitions
  pillar.rs       — OntologyPillar, UnknownPillarLabel (unchanged public contract)
```

No new source files are added. All new logic is added directly to `lib.rs` (the
single-mod pattern used by this crate). The `pillar.rs` public surface
(`wire_label`, `from_wire_label`, `all`) is not modified.

## New error variants

### `OntologyEngineError::UnknownEntityTypeEndpoint`

**Trigger**: `register_link_type` is called with a `LinkTypeDefinition` whose
`from_entity_type` or `to_entity_type` has not been registered for the same
`tenant_id`. Also triggered by `register_action_type` when `entity_type` has
not been registered for the same `tenant_id`.

**Replaces at callsite**: the existing `UnknownEntityType` check in
`register_link_type` is updated to return `UnknownEntityTypeEndpoint` instead,
providing a semantically distinct reason. `UnknownEntityType` is retained for
its existing use in `authorize_action_invocation` (unknown action type lookup).

**Data class**: `INTERNAL_ONLY` (no tenant-specific data in the error variant
itself; the error is a static enum arm).

### `OntologyEngineError::CrossPillarLink`

**Trigger**: `register_link_type` is called with a `LinkTypeDefinition` where
both `from_entity_type` and `to_entity_type` carry an `OntologyPillar`
annotation and those pillars differ (e.g., one is `Org`, the other is
`Person`). Per Bominal-ADR-0132, org-scoped and person-scoped objects must not
be bound by a direct link type.

**No trigger** (accepted): either endpoint has `pillar: None` (pillar-agnostic
entity types are neutral and do not participate in the cross-pillar check).
Both endpoints share the same pillar.

**Data class**: `INTERNAL_ONLY`.

## `EntityTypeDefinition` pillar extension

```rust
pub struct EntityTypeDefinition {
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub id: EntityTypeId,                              // data_class: INTERNAL_ONLY
    pub display_name: Classified<String>,              // data_class: INTERNAL_ONLY
    pub properties: Vec<EntityTypePropertyDefinition>, // data_class: INTERNAL_ONLY
    pub revision: u32,                                 // data_class: INTERNAL_ONLY
    pub pillar: Option<OntologyPillar>,                // data_class: INTERNAL_ONLY
}
```

`EntityTypeDefinition::new` is unchanged (sets `pillar: None`). A new
`with_pillar(self, pillar: OntologyPillar) -> Self` builder method allows
callers to annotate the pillar without breaking existing call sites.

## Invariant logic in `register_link_type`

Evaluation order at `OntologyEngine::register_link_type`:

1. Verify `from_entity_type` is registered for tenant → else `UnknownEntityTypeEndpoint`.
2. Verify `to_entity_type` is registered for tenant → else `UnknownEntityTypeEndpoint`.
3. If both endpoints have `pillar: Some(_)` and those pillars differ → `CrossPillarLink`.
4. Check for duplicate link type id → else `DuplicateLinkType`.
5. Insert and return `Ok(id)`.

## Testing strategy

All tests are in-crate unit tests under `#[cfg(test)]` in `src/lib.rs`, using
the existing `backbone_tests` module pattern. No new test files.

### Subtask st1 tests

- `link_type_with_dangling_from_endpoint_rejected`: registers only the `to`
  entity type; asserts `UnknownEntityTypeEndpoint`.
- `link_type_with_dangling_to_endpoint_rejected`: registers only the `from`
  entity type; asserts `UnknownEntityTypeEndpoint`.
- `action_type_with_dangling_entity_type_rejected`: registers no entity type;
  calls `register_action_type`; asserts `UnknownEntityTypeEndpoint`.
- `valid_link_and_action_type_registers_after_endpoints_present`: registers
  both endpoints; asserts `Ok`.

### Subtask st2 tests

- `cross_pillar_link_org_to_person_rejected`: org-pillar `from`, person-pillar
  `to`; asserts `CrossPillarLink`.
- `cross_pillar_link_person_to_org_rejected`: person-pillar `from`, org-pillar
  `to`; asserts `CrossPillarLink`.
- `same_pillar_link_org_to_org_accepted`: both org-pillar; asserts `Ok`.
- `same_pillar_link_person_to_person_accepted`: both person-pillar; asserts `Ok`.
- `pillar_agnostic_link_accepted`: both endpoints have `pillar: None`; asserts
  `Ok`.
- `one_pillar_agnostic_endpoint_accepted`: one endpoint has `pillar: Some(Org)`,
  the other has `pillar: None`; asserts `Ok`.
- Each `LinkCardinality` variant exercised in at least one successful
  registration test.

### Regression guard

All pre-existing backbone tests (`ontology_engine_registers_entity_types_and_rejects_conflicts`,
`ontology_engine_type_checks_links_before_registration`,
`ontology_engine_gates_action_invocation_by_policy_and_autonomy`) must continue
to pass unchanged.

## Boundaries

- **In scope**: `crates/ontology-kernel/src/lib.rs`, `tasks/ontology-kernel-link-action-validation-plan.md`, `docs/specs/task-ontology-kernel-link-action-validation.md`.
- **Out of scope**: `crates/ontology-kernel/src/pillar.rs` (public contract frozen), all other crates, root `Cargo.toml`, `Cargo.lock`.
- **No runtime dependency changes**: no new entries in `[dependencies]`.
- **No SLO file**: `ontology-kernel` is a kernel library, not a deployable microservice; OpenSLO is not applicable.

## Acceptance summary

| Gate | Command | Expected |
|------|---------|----------|
| Compile clean | `cargo check -p ontology-kernel --all-targets` | Zero errors, zero new clippy denials |
| Tests pass | `cargo nextest run -p ontology-kernel` | All tests green |
| Rustdoc | `cargo doc -p ontology-kernel --no-deps` | Compiles, no broken intra-doc links |
| Lane docs exist | `ls tasks/ontology-kernel-link-action-validation-plan.md docs/specs/task-ontology-kernel-link-action-validation.md` | Both files present |
