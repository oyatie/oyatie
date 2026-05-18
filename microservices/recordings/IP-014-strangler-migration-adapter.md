---
doc_class: ImplementationPlan
milestone: M02-foundation
phase: P01-recordings-foundation
impl_plan_id: IP-014-strangler-migration-adapter
status: pending
owner: axis-recordings
acceptance_lanes: [strangler-conformance]
---

# IP-014: Strangler migration adapter (oya-connect-recordings-* → oya-recordings-*)

## Intent

Author `oya-connect-recordings-migration-adapter` crate that shims the
legacy `oya-connect-recordings-domain` symbol surface to the new
`oya-recordings-*` BCs. Preserves Hyrum's-Law surfaces per
`migration-from-connect.md`.

## Concrete crate

`oya-connect-recordings-migration-adapter` (single crate; lives under
`microservices/recordings/src/migration-adapter/`).

## Acceptance Gates

```bash
cargo build -p oya-connect-recordings-migration-adapter
cargo nextest run -p oya-connect-recordings-migration-adapter -- byte_compat
cargo run -p oya-dev-cli -- gate validate strangler-conformance --microservice recordings
```

## Phase mapping

Per ADR-0134 Phase 2 adapter soak — 3-month soak required before Phase 3
canary.

## Next IP

[`IP-015-hg-recordings.md`](IP-015-hg-recordings.md)
