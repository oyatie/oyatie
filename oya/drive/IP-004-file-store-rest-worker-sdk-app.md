---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-004-file-store-rest-worker-sdk-app
status: pending
execution_unit: ChangeSet
owner: axis-drive
acceptance_lanes: [cargo-build, cargo-nextest, oya-check-audit-emission-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: file-store rest + worker + sdk + app

## Intent

Compose the file-store REST handler, background workers (retention sweep, version pruner, WORM integrity scan), SDK client, and composition-root binary. Wire to Cedar policy gate + audit-chain emission.

## Concrete File Targets

| Path | Action |
|---|---|
| `oya-drive-file-store-rest/...` | created — HTTP handler; OpenAPI alignment; Cedar policy gate |
| `oya-drive-file-store-worker/...` | created — retention sweep + version pruner + WORM integrity scan workers |
| `oya-drive-file-store-sdk/...` | created — Rust SDK client |
| `oya-drive-file-store-app/...` | created — composition-root binary |

## Acceptance Gates

```bash
cargo build -p oya-drive-file-store-{rest,worker,sdk,app}
cargo nextest run --test e2e_file_lifecycle
buck2 build //:quality-lane-registry-authority-check # lane=audit-emission-coverage --microservice drive --bc file-store
```

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-004-file-store-rest-worker-sdk-app
depends_on_changesets: [CS-DRIVE-IP-003-file-store-kernel-domain]
parallel_safe_with_changesets: [CS-DRIVE-IP-005-folder-hierarchy]
enables: [CS-DRIVE-IP-006-upload, CS-DRIVE-IP-007-download, CS-DRIVE-IP-009-share-link]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | REST handler maps every domain usecase 1:1 (zero handler-only logic) | `cargo nextest run -p oya-drive-file-store-rest -- handler_purity` |
| AC-02 | Retention sweeper purges expired versions; WORM-protected objects skipped | `cargo nextest run -p oya-drive-file-store-worker -- retention_sweep` |
| AC-03 | SDK round-trips full file lifecycle (upload, list, download, delete, restore) | `cargo nextest run --test e2e_file_lifecycle` |
| AC-04 | Audit-chain emission covers every state transition | `buck2 build //:quality-lane-registry-authority-check # lane=audit-emission-coverage --microservice drive --bc file-store` |

## Build Sequence

1. `oya-drive-file-store-rest` (axum + tower; Cedar policy gate).
2. `oya-drive-file-store-worker` — retention sweep + version pruner + WORM integrity scan.
3. `oya-drive-file-store-sdk` — Rust client.
4. `oya-drive-file-store-app` — composition-root binary.
5. `cargo build -p oya-drive-file-store-{rest,worker,sdk,app}`.
6. `cargo nextest run --test e2e_file_lifecycle`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-01..FR-23 surface routed through REST |
| PRD-drive NFR | NFR availability — read 99.95%, write 99.9% |
| PRD-drive | §"Workflow events produced" |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| REST handler drifts from usecase contract | `handler_purity` UI test pins 1:1 mapping |
| Worker double-execution under restart | Idempotency key on retention-sweep batches |
| SDK + REST version skew | SDK pinned to OpenAPI spec hash; CI lane enforces |

## References

- `microservices/drive/PRD.md` §"Workflow events produced".
- OpenAPI 3.1 specification (`spec.openapis.org/oas/v3.1.0`).
- axum + tower documentation (`docs.rs/axum`).
- Twelve-Factor App methodology — composition-root binary pattern.
