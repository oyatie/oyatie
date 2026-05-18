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

## References

- PRD-drive §FR-05; §FR-15; AC-06.
