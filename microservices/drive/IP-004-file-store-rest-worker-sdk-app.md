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
cargo run -p oya-dev-cli -- gate validate audit-emission-coverage --microservice drive --bc file-store
```

## References

- `microservices/drive/PRD.md` §"Workflow events produced".
