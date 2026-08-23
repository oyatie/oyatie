# Spec: ontology-kernel-link-instance-cardinality-enforcement

## Objective

Extend the `ontology-kernel` crate with a tenant-scoped link-instance registry and
cardinality enforcement. The engine already knows the declared `LinkCardinality` of every
registered `LinkTypeDefinition`; this slice enforces that cardinality at the instance level
when callers register concrete directed edges between entity instances.

## Crate boundary

Only `crates/ontology-kernel` is modified. No workspace Cargo.toml changes. No new crate.

## Mod layout (flat-clean-arch per ADR-0509)

All implementation lives directly in `src/lib.rs` (the single-source pattern already in use in
this crate). No new mod files are needed; the surface area is small and cohesive with existing
`OntologyEngine` state.

## New public types

### `LinkInstanceOutcome`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkInstanceOutcome {
    /// The instance was freshly inserted.
    Registered,
    /// The identical (link_type_id, from_entity_id, to_entity_id) tuple already existed;
    /// no state change occurred.
    AlreadyExists,
}
```

### New error variants on `OntologyEngineError`

| Variant | Trigger |
|---------|---------|
| `UnknownLinkType` | `register_link_instance` called with a `LinkTypeId` not registered for the tenant. |
| `CardinalityViolation { cardinality: LinkCardinality }` | A second edge would violate the `LinkCardinality` declared on the `LinkTypeDefinition`. |

## New method

```rust
impl OntologyEngine {
    /// Register a directed link instance from `from_entity_id` to `to_entity_id`
    /// under `link_type_id` for `tenant_id`.
    ///
    /// # Behaviour
    ///
    /// 1. Rejects an unknown `link_type_id` with [`OntologyEngineError::UnknownLinkType`].
    /// 2. Is **idempotent** for the identical `(link_type_id, from_entity_id, to_entity_id)`
    ///    tuple — returns `Ok(LinkInstanceOutcome::AlreadyExists)` without mutation.
    /// 3. Enforces the [`LinkCardinality`] declared on the type:
    ///    - `OneToOne`: rejects a second outbound edge from `from_entity_id` **and** a
    ///      second inbound edge into `to_entity_id`.
    ///    - `OneToMany`: rejects a second inbound edge into `to_entity_id`; fan-out from
    ///      a single `from_entity_id` is permitted.
    ///    - `ManyToMany`: no restriction.
    ///    On violation returns `Err(CardinalityViolation { cardinality })`.
    pub fn register_link_instance(
        &mut self,
        tenant_id: &str,
        link_type_id: &LinkTypeId,
        from_entity_id: &str,
        to_entity_id: &str,
    ) -> Result<LinkInstanceOutcome, OntologyEngineError>;
}
```

## Storage design

Three `BTreeMap` fields added to `OntologyEngine` (all `data_class: INTERNAL_ONLY`):

| Field | Key type | Purpose |
|-------|----------|---------|
| `link_instances` | `(String, String, String, String)` — `(tenant_id, link_type_id_value, from_entity_id, to_entity_id)` | Full 4-tuple for idempotency check |
| `link_outbound` | `(String, String, String)` — `(tenant_id, link_type_id_value, from_entity_id)` | OneToOne outbound fan-out guard |
| `link_inbound` | `(String, String, String)` — `(tenant_id, link_type_id_value, to_entity_id)` | OneToOne + OneToMany inbound fan-in guard |

All three are plain `BTreeMap<K, ()>` (present = edge exists). `BTreeMap` is used throughout this
crate; no new dependency is needed.

## Cardinality enforcement table

| `LinkCardinality` | outbound guard | inbound guard |
|---|---|---|
| `OneToOne` | Yes (at most 1 outbound per from) | Yes (at most 1 inbound per to) |
| `OneToMany` | No | Yes (at most 1 inbound per to) |
| `ManyToMany` | No | No |

Check order: idempotency → outbound → inbound → insert.

## Testing strategy

Hermetic unit tests only. New integration test file:
`crates/ontology-kernel/tests/link_instance_cardinality.rs`

| Test | Scenario |
|------|----------|
| `unknown_link_type_rejected` | `UnknownLinkType` returned for unregistered link type |
| `one_to_one_second_from_rejected` | Same from, different to → `CardinalityViolation(OneToOne)` |
| `one_to_one_second_to_rejected` | Different from, same to → `CardinalityViolation(OneToOne)` |
| `one_to_many_fan_out_allowed` | Same from, two distinct to's → both `Registered` |
| `one_to_many_second_into_rejected` | Same to, different from → `CardinalityViolation(OneToMany)` |
| `many_to_many_all_allowed` | Multiple combinations → all `Registered` |
| `idempotent_reinsert_returns_already_exists` | Same edge twice → `Registered` then `AlreadyExists` |

All existing tests in `lib.rs`, `link_action_invariants.rs`, `schema_evolution.rs`, `types.rs`
remain green.

## Observability / SLO

This crate is a pure kernel (no I/O, no HTTP, no OTel instrumentation surface). SLO authoring
applies to microservice promotion gates, not kernel crates. No SLO file is required for this slice.

## Contracts

No OpenAPI/AsyncAPI/proto3 changes. Link-instance management is an internal engine concern;
the external contract surface is owned by the service layer that wraps this kernel.

## Cloud-native readiness

- Deterministic BTreeMap state: serialisable, snapshotable, replayable.
- No external dependencies introduced.
- No async, no allocator, no I/O.
- Tenant isolation is structural (tenant_id is the leading key segment in every map).
