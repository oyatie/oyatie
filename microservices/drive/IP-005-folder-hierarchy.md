---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-005-folder-hierarchy
status: pending
execution_unit: ChangeSet
owner: axis-drive
acceptance_lanes: [cargo-build, cargo-nextest]
---

# IP-005: folder-hierarchy (8 crates)

## Intent

Stand up `oya-drive-folder-hierarchy-*` BC: nested folder tree with per-folder permission inheritance + per-file override resolved per ADR-DRIVE-0003 + PRD AC-06 (5-level depth verified).

## Crates

`oya-drive-folder-hierarchy-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,app}` (8 crates).

## Acceptance Gates

```bash
cargo nextest run -p oya-drive-folder-hierarchy-domain -- inheritance_5levels
cargo nextest run -p oya-drive-folder-hierarchy-domain -- per_file_override
```

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-005-folder-hierarchy
depends_on_changesets: [CS-DRIVE-IP-001-iac-bootstrap, CS-DRIVE-IP-002-cargo-workspace, CS-DRIVE-IP-003-file-store-kernel-domain]
parallel_safe_with_changesets: [CS-DRIVE-IP-006-upload, CS-DRIVE-IP-007-download]
enables: [CS-DRIVE-IP-010-permissions, CS-DRIVE-IP-011-search-index]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | 5-level nested folder depth resolves with inherited ACL evaluation ≤ p95 40ms | `cargo nextest run -p oya-drive-folder-hierarchy-domain -- inheritance_5levels` |
| AC-02 | Per-file ACL override at any depth shadows inherited grant deterministically | `cargo nextest run -p oya-drive-folder-hierarchy-domain -- per_file_override` |
| AC-03 | Folder move preserves audit-chain seal across parent reparenting | `cargo nextest run -p oya-drive-folder-hierarchy-domain -- audit_chain_move` |
| AC-04 | `oya gate validate layer-correctness --microservice drive` exits 0 for this BC | ADR-0131 / ADR-0105 |

## Build Sequence

1. `cargo new --lib oya-drive-folder-hierarchy-kernel` — port traits `FolderRepository`, `FolderTreeReader`.
2. `cargo new --lib oya-drive-folder-hierarchy-domain` — `Folder`, `FolderPermission`, `FolderTree` entities.
3. `cargo new --lib oya-drive-folder-hierarchy-usecase` — `CreateFolder`, `MoveFolder`, `RenameFolder`, `DeleteFolder`.
4. `cargo new --lib oya-drive-folder-hierarchy-{api,adapter,adapter-postgres,rest,app}`.
5. `cargo nextest run -p oya-drive-folder-hierarchy-*`.
6. `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice drive`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-03 (folder hierarchy), FR-05 (per-folder permissions) |
| PRD-drive NFR | NFR perf — file-list folder p95 ≤ 150ms |
| PRD-drive AC | AC-06 (folder + permission inheritance) |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Deep-tree pathological queries (>20 levels) — Postgres recursive CTE blow-up | Cap depth at 20 in `domain`; refuse beyond per AC verified test |
| ACL inheritance cache stampede under bulk move | Per-tenant Redis lock + invalidation broadcast |
| Cross-tenant folder reparent attempted | `domain` refuses cross-tenant; covered by `cross_tenant_refused` UI test |

## References

- PRD-drive §FR-03, §FR-05, AC-06.
- AWS S3 prefix semantics (S3 User Guide, "Organizing objects using prefixes").
- Google Drive folder model (Google Workspace Admin Help — "Drive folder structure").
- ADR-0105 (13-value layer enum); ADR-0131 (per-microservice flat layout).
