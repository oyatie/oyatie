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

## References

- PRD-drive §FR-03, §FR-05, AC-06.
