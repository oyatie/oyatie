# Task Plan: ontology-kernel-link-action-validation

## Identity

- **Vertical**: ontology
- **Task slug**: ontology-kernel-link-action-validation
- **Crate**: `oya-ontology-kernel` (sole crate this task may touch)
- **Branch**: `feat/task-ontology-kernel-link-action-validation-2026-05-28`
- **Base**: `origin/dev`
- **Lane docs**: `docs/specs/task-ontology-kernel-link-action-validation.md`, `tasks/ontology-kernel-link-action-validation-plan.md`

## Objective

Extend the typed-entity substrate in `oya-ontology-kernel` with stronger
`LinkTypeDefinition`/`ActionTypeDefinition` modeling:

1. Enforce endpoint-reference validity at `OntologyEngine` registration time —
   reject any link/action type whose endpoint `EntityTypeId` is not previously
   registered for the same tenant.
2. Enforce `LinkCardinality` endpoint-arity invariants and org/person pillar
   isolation (Bominal-ADR-0132) — reject link types that cross the org/person
   pillar boundary.
3. Document all new invariants and error variants in crate-level rustdoc and
   lane-namespaced docs.

No new crate. No root `Cargo.toml` edit. Pure-Rust, deterministic, in-memory.
All new logic lives as mods inside `crates/oya-ontology-kernel/src/`.

## Subtasks

### [st1] Endpoint-reference validation

**What**: Add endpoint-reference validation to `OntologyEngine.register_link_type`
and `register_action_type`. A `LinkTypeDefinition`/`ActionTypeDefinition` that
references an `EntityTypeId` not previously registered for the same tenant is
rejected with a new `OntologyEngineError` variant.

**New error variant**: `UnknownEntityTypeEndpoint`

**Note**: `register_link_type` already rejects unknown entity-type endpoints via
`UnknownEntityType`. This subtask introduces `UnknownEntityTypeEndpoint` as the
dedicated, semantically distinct variant that makes the rejection reason
unambiguous in callers and test assertions. The existing `UnknownEntityType`
variant (used by `authorize_action_invocation`) is preserved unchanged.

**Acceptance**:
- `cargo check -p oya-ontology-kernel --all-targets` is clean.
- `cargo nextest run -p oya-ontology-kernel` passes.
- New unit tests prove:
  - A link type with a dangling `from_entity_type` `EntityTypeId` is rejected
    with `UnknownEntityTypeEndpoint`.
  - A link type with a dangling `to_entity_type` `EntityTypeId` is rejected
    with `UnknownEntityTypeEndpoint`.
  - An action type with a dangling `entity_type` `EntityTypeId` is rejected
    with `UnknownEntityTypeEndpoint`.
  - A fully-registered link type and action type still register successfully.

### [st2] LinkCardinality arity and pillar-consistency invariants

**What**: Enforce two invariants at `register_link_type`:

1. **Pillar-consistency**: if both endpoint `EntityTypeDefinition`s carry an
   `OntologyPillar`, they must be the same pillar. A cross-pillar link type
   (one org-pillar endpoint + one person-pillar endpoint) is rejected.
2. **Cardinality arity consistency**: `LinkCardinality::OneToOne` requires both
   endpoints to be defined; `OneToMany` requires a source and target;
   `ManyToMany` allows any arity. (Since endpoint presence is already enforced
   by st1, this is a definitional consistency check for future per-cardinality
   rules, expressed as an explicit invariant in code and tests.)

**New error variant**: `CrossPillarLink`

**Pillar annotation**: `EntityTypeDefinition` gains an optional
`pillar: Option<OntologyPillar>` field (defaults to `None` = pillar-agnostic,
backward-compatible). `EntityTypeDefinition::new` is extended with a `pillar`
parameter or a `with_pillar` builder method — choose the builder to keep the
existing `new` signature unchanged and avoid breaking existing call sites.

**Acceptance**:
- `cargo nextest run -p oya-ontology-kernel` passes.
- Unit tests cover:
  - (a) A cross-pillar link type (org → person) rejected with `CrossPillarLink`.
  - (b) A same-pillar link type (org → org) accepted.
  - (c) A pillar-agnostic link type (no pillar on either endpoint) accepted.
  - (d) Each `LinkCardinality` variant exercised in a valid registration.
- `OntologyPillar::wire_label` / `from_wire_label` public contract unchanged.

### [st3] Rustdoc and lane docs

**What**: Document the new invariants and error variants.

- Crate-level rustdoc in `src/lib.rs`: expand the module doc to describe
  the new validation invariants, each new error variant, and its trigger
  condition.
- `docs/specs/task-ontology-kernel-link-action-validation.md`: objective,
  vertical, contracts (no REST/gRPC surface — pure in-memory kernel; note
  absence of external contract surface), mod layout, testing strategy,
  boundaries, each new `OntologyEngineError` variant with trigger condition.
- `tasks/ontology-kernel-link-action-validation-plan.md`: this file.
- Preserve all existing `data_class` annotations on every touched field.

**Acceptance**:
- `cargo check -p oya-ontology-kernel --all-targets` clean; no new clippy
  denials under workspace lints.
- Rustdoc compiles (no broken intra-doc links).
- Both lane docs exist and describe each new `OntologyEngineError` variant and
  its trigger condition.

## Execution order

```
st1 → st2 → st3
```

Each subtask is verified with `cargo check -p oya-ontology-kernel --all-targets`
and `cargo nextest run -p oya-ontology-kernel` before proceeding to the next.

## Constraints

- NEVER edit root `Cargo.toml`.
- NEVER touch another crate.
- All new logic as mods inside `crates/oya-ontology-kernel/src/`.
- No new crate members, no new workspace members.
- Match existing naming, error-handling, and doc-comment patterns.
- All `data_class` annotations preserved on every touched struct field.

## Verification commands

```sh
cargo check -p oya-ontology-kernel --all-targets
cargo nextest run -p oya-ontology-kernel
```
