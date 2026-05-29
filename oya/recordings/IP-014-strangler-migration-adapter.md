---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-014-strangler-migration-adapter
status: pending
owner: axis-recordings
acceptance_lanes: [strangler-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: Strangler migration adapter (oya-recordings-* → oya-recordings-*)

## Intent

Author `oya-recordings-migration-adapter` crate that shims the
legacy `oya-recordings-domain` symbol surface to the new
`oya-recordings-*` BCs. Preserves Hyrum's-Law surfaces per
`migration-from-connect.md`.

## Concrete crate

`oya-recordings-migration-adapter` (single crate; lives under
`microservices/recordings/src/migration-adapter/`).

## Acceptance Gates

```bash
cargo build -p oya-recordings-migration-adapter
cargo nextest run -p oya-recordings-migration-adapter -- byte_compat
cargo run -p oya-dev-cli -- gate validate strangler-conformance --microservice recordings
```

## Phase mapping

Per ADR-0134 Phase 2 adapter soak — 3-month soak required before Phase 3
canary.

## ChangeSet metadata

```yaml
changeset_id: CS-RECORDINGS-IP-014-strangler-migration-adapter
depends_on_changesets: [CS-RECORDINGS-IP-004-recording-bc]
parallel_safe_with_changesets: []
enables: [CS-RECORDINGS-IP-015-hg-recordings]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Migration adapter exposes legacy `oya-recordings-domain` symbol surface 1:1 | `cargo build -p oya-recordings-migration-adapter` |
| AC-02 | Byte-for-byte serialisation compat with legacy crate | `cargo nextest run -p oya-recordings-migration-adapter -- byte_compat` |
| AC-03 | 3-month adapter soak window scheduled before Phase 3 canary | ADR-0134 schedule (calendar artifact) |
| AC-04 | `oya gate validate strangler-conformance --microservice recordings` exits 0 | governance lane |

## Build Sequence

1. Single crate `oya-recordings-migration-adapter` at `microservices/recordings/src/migration-adapter/`.
2. Reexport surface symbols from new `oya-recordings-*` BCs, mapping legacy structs.
3. Byte-compat fixture corpus from production data captured under legal hold.
4. `cargo nextest run -p oya-recordings-migration-adapter -- byte_compat`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| migration-from-connect.md | Hyrum surfaces enumeration |
| ADR | ADR-0134 (Strangler timeline), ADR-0135 (unbundle) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Legacy callers break on subtle struct field reorder | Byte-compat fixture corpus tests catch reorder |
| Soak window cut short under pressure | ADR-0134 mandates 3-month minimum; admission gate refuses earlier promotion |
| Adapter masks new-side bugs by translating away | Telemetry tags `via_migration_adapter=true` to expose call surface |

## References

- ADR-0134 (Strangler timeline).
- ADR-0135 (unbundle parallel session).
- "Strangler Fig Application" — Martin Fowler (`martinfowler.com/bliki/StranglerFigApplication.html`).
- `microservices/recordings/migration-from-connect.md`.

## Phase mapping

Per ADR-0134 Phase 2 adapter soak — 3-month soak required before Phase 3
canary.

## Next IP

[`IP-015-hg-recordings.md`](IP-015-hg-recordings.md)
