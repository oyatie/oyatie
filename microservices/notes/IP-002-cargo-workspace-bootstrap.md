---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-notes-foundation
impl_plan_id: IP-002-cargo-workspace-bootstrap
status: pending
execution_unit: ChangeSet
owner: axis-notes
acceptance_lanes: [cargo-check, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

# IP-002: Cargo workspace bootstrap

## Intent

Register all 111 crates in the workspace `Cargo.toml`. Crate templates per BC × layer per ADR-0105 + ADR-0106 + ADR-0131. Skeleton crates compile (empty kernel; empty domain stubs) so subsequent IPs add port traits + impls incrementally.

## Concrete File Targets

- `Cargo.toml` — add `members = [ "microservices/notes/src/oya-notes-*-*" ]` glob.
- `microservices/notes/src/<crate>/Cargo.toml` × 111 (workspace-aware deps; LTS pins).
- `microservices/notes/src/<crate>/src/lib.rs` × 111 (empty `pub mod` skeleton).

## Crate Inventory

Per BC × layer (17 BCs, layers vary):

| BC | Layers | Crates |
|---|---|---|
| note-store | kernel, domain, usecase, api, adapter, adapter-postgres, adapter-redis, adapter-s3, rest, sdk, app | 11 |
| tag-graph | kernel, domain, usecase, api, adapter, adapter-postgres, sdk, app | 8 |
| backlink-graph | kernel, domain, usecase, api, adapter, adapter-postgres, worker, sdk, app | 9 |
| daily-note | kernel, domain, usecase, api, adapter, sdk, app | 7 |
| template-gallery | kernel, domain, usecase, api, adapter, adapter-postgres, sdk, app | 8 |
| web-clipper-bridge | kernel, domain, usecase, api, adapter, rest, sdk | 7 |
| share-link | kernel, domain, usecase, api, adapter, adapter-postgres, rest, sdk, app | 9 |
| embed | kernel, domain, usecase, api, adapter, adapter-s3, sdk, app | 8 |
| checklist | kernel, domain, usecase, api, adapter, worker, sdk, app | 8 |
| version-history | kernel, domain, usecase, api, adapter, adapter-postgres, worker, sdk, app | 9 |
| search-index | kernel, domain, usecase, api, adapter-meilisearch, worker, sdk, app | 8 |
| graph-view-data | kernel, domain, usecase, api, adapter, sdk, app | 7 |
| collab-edit | kernel, domain, usecase, api, adapter, adapter-loro, worker, sdk, app | 9 |
| import-pipeline | kernel, domain, usecase, api, adapter, worker, sdk, app | 8 |
| export-pipeline | kernel, domain, usecase, api, adapter, worker, sdk, app | 8 |
| ai-assist | kernel, domain, usecase, api, adapter, worker, sdk, app | 8 |
| e2e-key-management | kernel, domain, usecase, api, adapter, adapter-mls, sdk, app | 8 |

Total: 111 crates.

## Acceptance Gates

```bash
cargo check --workspace
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice notes
cargo run -p oya-dev-cli -- gate validate version-pinning-conformance
```

## Halt Conditions

- Workspace `Cargo.toml` grows beyond agreed budget — escalate.
- Crate naming fails ADR-0056 BNF — fix.

## Next IP

[`IP-003-note-store-kernel-domain.md`](IP-003-note-store-kernel-domain.md)
