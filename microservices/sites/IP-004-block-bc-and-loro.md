---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-sites-foundation
impl_plan_id: IP-004-block-bc-and-loro
status: pending
execution_unit: ChangeSet
owner: axis-sites
acceptance_lanes: [cargo-build, cargo-nextest, oya-governance-layer-correctness, oya-governance-crdt-tenant-scope]
---

# IP-004: block BC + Loro CRDT adapter

## Intent

Author the `block` bounded-context's full crate stack including the Loro CRDT adapter aligned with docs + sheets + slides + workflow-studio per ADR-SITES-0001 + ADR-WS-0001. Implements `Block`, `BlockKind` enum, `PortableTextNode`. Loro op-log signing + per-tenant session-token validation. Portable-text round-trip from/to HTML via Pandoc adapter.

## ChangeSet boundary

6 crates: `oya-sites-block-{kernel,domain,usecase,api,adapter,adapter-loro,app}` (note `adapter-loro` is the backend-qualified variant per ADR-0105 Amendment 3). AC-10 covered.

## Acceptance Gates

```bash
cargo build -p oya-sites-block-kernel ..  -p oya-sites-block-adapter-loro
cargo nextest run -p oya-sites-block-adapter-loro -- crdt_converge
cargo nextest run -p oya-sites-block-adapter-loro -- crdt_spoof_refuse
cargo nextest run -p oya-sites-block-adapter-loro -- crdt_tenant_scope
cargo run -p oya-dev-cli -- gate validate crdt-tenant-scope --microservice sites
```

## Test Plan

- Unit: portable-text serialisation round-trip.
- Unit: BlockKind enum exhaustive coverage.
- Integration: Loro CRDT deterministic convergence on concurrent ops.
- Integration: cross-tenant CRDT op refused (Invariant 5 of editor-isolation.md).
- Integration: SVG sanitisation refuses embedded `<script>`.

## References

- ADR-0105, ADR-0131, ADR-0140.
- ADR-SITES-0001 (Loro CRDT 1.x).
- ADR-WS-0001 (Loro alignment across docs + sheets + slides + workflow-studio).
- PRD §"Bounded Contexts" + AC-10.
- Loro CRDT documentation — `loro.dev/docs`.
