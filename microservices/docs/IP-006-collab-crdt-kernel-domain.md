---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-006-collab-crdt-kernel-domain
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-crdt-no-silent-loss, oya-governance-crdt-cross-microservice-consistency]
---

# IP-006: collab-crdt kernel + domain + adapter (Loro 1.x integration; cross-µservice consistent per ADR-DOCS-0001 + ADR-WS-0001)

## Intent

Implement the CRDT substrate per ADR-DOCS-0001 (Loro 1.x). Cross-µservice consistent CrdtOp envelope shape with workflow-studio per ADR-WS-0001. Property tests for AC-06 never-silent-loss invariant. Deterministic projection to canonical block tree for AC-02 byte-equality.

## ChangeSet boundary

4 crates: kernel + domain + usecase + adapter (Loro wrapping).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-collab-crdt-kernel/src/{lib,merge_engine,state,op,conflict}.rs` | create |
| `microservices/docs/src/crates/oya-docs-collab-crdt-domain/src/{lib,no_silent_loss,canonicalisation,conflict_surfacing}.rs` | create |
| `microservices/docs/src/crates/oya-docs-collab-crdt-usecase/src/{lib,apply_op,project_to_canonical,resolve_conflict}.rs` | create |
| `microservices/docs/src/crates/oya-docs-collab-crdt-adapter/src/{lib,loro_engine,projection}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-collab-crdt-domain -- never_silent_loss  # AC-06
cargo nextest run -p oya-docs-collab-crdt-domain -- round_trip_byte_equality  # AC-02
cargo run -p oya-dev-cli -- gate validate crdt-no-silent-loss --microservice docs
cargo run -p oya-dev-cli -- gate validate crdt-cross-microservice-consistency
```

## References

- ADR-DOCS-0001 (Loro 1.x; cross-µservice consistent with workflow-studio).
- ADR-WS-0001 (CRDT library selection; primary cross-µservice authority).
- Loro CRDT — `loro.dev`.
