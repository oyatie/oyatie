# Plan: ontology-kernel-link-instance-cardinality-enforcement

## Objective

Extend `OntologyEngine` with a tenant-scoped link-instance registry and cardinality enforcement
against the registered `LinkTypeDefinition`. All logic is pure in-memory, deterministic, no I/O.

## Subtasks (ordered)

### st1 — New error variants
Add `UnknownLinkType` and `CardinalityViolation { cardinality: LinkCardinality }` to `OntologyEngineError`.

**Acceptance**: Both variants compile and can be matched in user code.

### st2 — Link-instance registry storage
Add `link_instances: BTreeMap<LinkInstanceKey, ()>` to `OntologyEngine` where
`LinkInstanceKey = (tenant_id, link_type_id, from_entity_id, to_entity_id)` (all `String`).
Add outbound index `link_outbound: BTreeMap<(String, String, String), ()>` keyed by
`(tenant_id, link_type_id, from_entity_id)` and inbound index
`link_inbound: BTreeMap<(String, String, String), ()>` keyed by
`(tenant_id, link_type_id, to_entity_id)`.

**Acceptance**: `OntologyEngine::default()` compiles with the new fields.

### st3 — `register_link_instance` implementation
Signature:
```rust
pub fn register_link_instance(
    &mut self,
    tenant_id: &str,
    link_type_id: &LinkTypeId,
    from_entity_id: &str,
    to_entity_id: &str,
) -> Result<LinkInstanceOutcome, OntologyEngineError>
```

Logic:
1. Look up `LinkTypeDefinition` for `(tenant_id, link_type_id)`.
   - Not found → `Err(OntologyEngineError::UnknownLinkType)`.
2. Idempotency: if `(tenant_id, link_type_id, from_entity_id, to_entity_id)` is already in the registry → `Ok(LinkInstanceOutcome::AlreadyExists)`.
3. Cardinality enforcement:
   - `OneToOne`: if outbound key `(tenant_id, link_type_id, from_entity_id)` exists → `Err(CardinalityViolation { cardinality: OneToOne })`.
     If inbound key `(tenant_id, link_type_id, to_entity_id)` exists → `Err(CardinalityViolation { cardinality: OneToOne })`.
   - `OneToMany`: if inbound key `(tenant_id, link_type_id, to_entity_id)` exists → `Err(CardinalityViolation { cardinality: OneToMany })`.
   - `ManyToMany`: no enforcement.
4. Insert into all three indices. Return `Ok(LinkInstanceOutcome::Registered)`.

**Acceptance**: All unit tests green.

### st4 — Unit tests (RED → GREEN)
Written in `crates/oya-ontology-kernel/tests/link_instance_cardinality.rs`:

- `unknown_link_type_rejected` — register_link_instance with unregistered link_type_id.
- `one_to_one_second_from_rejected` — same from, different to → CardinalityViolation.
- `one_to_one_second_to_rejected` — different from, same to → CardinalityViolation.
- `one_to_many_fan_out_allowed` — same from, two different to's → both Ok.
- `one_to_many_second_into_rejected` — same to, different from → CardinalityViolation.
- `many_to_many_all_allowed` — multiple from→to combinations all succeed.
- `idempotent_reinsert_returns_already_exists` — inserting the same (type,from,to) twice → first Registered, second AlreadyExists.

## Edge cases considered

- Tenant isolation: link type lookup is scoped by tenant_id; different tenants share no state.
- Idempotency is checked before cardinality; the exact same edge is never double-counted.
- `from_entity_id` and `to_entity_id` are raw strings (no `ent_` prefix enforcement here — the link-type schema already validates entity types at registration time; instance IDs are opaque references).
- `OneToOne` checks BOTH directions before inserting, so the first violation encountered is returned (outbound first, then inbound).

## Acceptance summary

All seven new tests pass. All existing tests in `lib.rs`, `link_action_invariants.rs`,
`schema_evolution.rs`, and `types.rs` remain green. `cargo nextest run -p oya-ontology-kernel` exits 0.
