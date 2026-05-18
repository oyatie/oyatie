---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-008-sync
status: pending
execution_unit: ChangeSet
owner: axis-drive
acceptance_lanes: [cargo-build, cargo-nextest, oya-check-cdc-parameters-pinned]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: sync BC — FastCDC + LBFS delta-sync + deterministic conflict tie-break

## Intent

Stand up `oya-drive-sync-*` BC per ADR-DRIVE-0002. FastCDC implementation + chunk-manifest exchange + delta-set computation + deterministic conflict tie-break.

## Crates

`oya-drive-sync-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` (10 crates).

## Acceptance Gates

```bash
cargo nextest run -p oya-drive-sync-domain -- fastcdc_parameters_pinned
cargo nextest run -p oya-drive-sync-domain -- delta_minimum_bytes
cargo nextest run -p oya-drive-sync-domain -- tie_break_determinism
cargo nextest run -p oya-drive-sync-domain -- fastcdc_adversarial
```

## ChangeSet metadata

```yaml
changeset_id: CS-DRIVE-IP-008-sync
depends_on_changesets: [CS-DRIVE-IP-003-file-store-kernel-domain, CS-DRIVE-IP-006-upload]
parallel_safe_with_changesets: [CS-DRIVE-IP-007-download, CS-DRIVE-IP-005-folder-hierarchy]
enables: [CS-DRIVE-IP-011-search-index]
acceptance_status: ga
```

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | FastCDC parameters pinned (min 4KiB, avg 8KiB, max 64KiB) and stable across releases | `cargo nextest run -p oya-drive-sync-domain -- fastcdc_parameters_pinned` |
| AC-02 | 100-file delta exchange p95 ≤ 30s on 1MB/s link | `cargo nextest run -p oya-drive-sync-domain -- delta_minimum_bytes` |
| AC-03 | Concurrent edits resolve deterministically (lexicographic on `(version, client_id)`) | `cargo nextest run -p oya-drive-sync-domain -- tie_break_determinism` |
| AC-04 | Adversarial CDC inputs (all-zero, repeating, random-large) do not OOM | `cargo nextest run -p oya-drive-sync-domain -- fastcdc_adversarial` |

## Build Sequence

1. Kernel: `ChunkManifestExchange`, `DeltaComputer`, `ConflictResolver` ports.
2. Domain: FastCDC implementation (Xia 2016); LBFS-style rolling-hash diff per Muthitacharoen 2001.
3. Usecase: `ComputeManifest`, `ExchangeDelta`, `ResolveConflict`.
4. Postgres adapter for sync-state checkpoints.
5. Worker that drains pending delta requests.
6. `cargo nextest run -p oya-drive-sync-*`.

## Traceability

| Sibling artifact | Reference |
|---|---|
| PRD-drive FR | FR-06 (delta-sync), FR-22 (conflict tie-break) |
| PRD-drive NFR | NFR perf — sync delta 100 files p95 ≤ 30s |
| PRD-drive AC | AC-04 |
| ADR | ADR-DRIVE-0002 |

## Risk + Mitigation

| Risk | Mitigation |
|---|---|
| Pathological CDC inputs (e.g., all zeros) producing massive chunk count | `domain` refuses files > 5TB; chunk-count cap with diagnostic |
| Conflict tie-break non-determinism across clock skew | Tie-break by `(version, client_id_lex)`; never wall clock |
| Manifest exchange replay across tenants | Manifest bound to `(tenant_id, file_id, version)` signed Ed25519 |

## References

- ADR-DRIVE-0002.
- PRD-drive §FR-06; FR-22; AC-04.
- Muthitacharoen, A. et al. "A Low-bandwidth Network File System" (SOSP 2001 — LBFS).
- Xia, W. et al. "FastCDC" (USENIX ATC 2016).
- rsync algorithm — Tridgell, A. "Efficient Algorithms for Sorting and Synchronization" (PhD thesis, ANU 1999).
