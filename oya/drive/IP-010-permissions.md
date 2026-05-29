---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-010-permissions
status: pending
execution_unit: ChangeSet
owner: axis-drive + ops-security
acceptance_lanes: [cargo-build, cargo-nextest, oya-check-cedar-policy-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: permissions BC — per-folder + per-file ACL + inheritance + override + ownership transfer

## Intent

Stand up `oya-drive-permissions-*` BC. 4-level access (read/comment/edit/manage) + per-folder inheritance + per-file override + ownership transfer ceremony. Cedar policy authority.

## Crates

`oya-drive-permissions-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,app}` (8 crates).

## Acceptance Gates

```bash
cargo nextest run -p oya-drive-permissions-domain -- inheritance_5levels
cargo nextest run -p oya-drive-permissions-domain -- per_file_override
cargo nextest run -p oya-drive-permissions-domain -- ownership_transfer_ceremony
cargo run -p oya-dev-cli -- gate validate cedar-policy-coverage --microservice drive --bc permissions
```

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-010-permissions
depends_on_changesets: [CS-DRIVE-IP-005-folder-hierarchy, CS-DRIVE-IP-003-file-store-kernel-domain]
parallel_safe_with_changesets: [CS-DRIVE-IP-011-search-index, CS-DRIVE-IP-012-preview]
enables: [CS-DRIVE-IP-009-share-link]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | 5-level inheritance computes correct effective ACL | `cargo nextest run -p oya-drive-permissions-domain -- inheritance_5levels` |
| AC-02 | Per-file override shadows inherited entry deterministically | `cargo nextest run -p oya-drive-permissions-domain -- per_file_override` |
| AC-03 | Ownership transfer is atomic + audit-chain sealed; never partial | `cargo nextest run -p oya-drive-permissions-domain -- ownership_transfer_ceremony` |
| AC-04 | `oya gate validate cedar-policy-coverage --microservice drive --bc permissions` exits 0 | ADR-0140 (retired per ADR-0145) |

## Build Sequence

1. Kernel: `PermissionRepository`, `CedarPermissionPolicy`, `OwnershipTransfer` ports.
2. Domain: `Acl`, `Grant`, `Role` (read/comment/edit/manage), `Inheritance`.
3. Usecase: `GrantPermission`, `RevokePermission`, `TransferOwnership`.
4. Postgres adapter (RLS-bound).
5. Cedar policy authoring + lint.
6. `cargo nextest run -p oya-drive-permissions-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-05 (per-folder + per-file permissions), FR-15 (ownership transfer) |
| PRD-drive AC | AC-06 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Cedar policy regression silently broadens access | `cedar-policy-coverage` lane refuses unguarded action surface |
| Ownership transfer partial under failure | Saga with compensating txn; audit-chain rollback record |
| Inheritance cache stale post-grant | Per-folder cache invalidation broadcast; max-staleness 1s |

## References

- PRD-drive §FR-05; §FR-15; AC-06.
- Cedar Policy Language reference (Amazon Cedar docs — `docs.cedarpolicy.com`).
- Google Drive permission model (Workspace Help — "Share files, folders & drives").
- Dropbox file permissions reference (Dropbox Business Help).
- ADR-0140 (Cedar pack overlays).
