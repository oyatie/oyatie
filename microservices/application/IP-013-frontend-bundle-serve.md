---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-013-frontend-bundle-serve
status: pending
execution_unit: ChangeSet
owner: axis-application
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

# IP-013: frontend-bundle-serve crates

## Intent

Combined IP: frontend-bundle-serve BC's kernel + usecase + adapter +
adapter-cdn + adapter-postgres + worker.

- kernel: ports (BundleStore, CdnPurger, BundleVersionPointerStore) + entities.
- usecase: PromoteBundleVersion, PurgeBundle, ReadActivePointer.
- adapter-cdn: per-pack OCI CDN purge + invalidation client.
- adapter-postgres: bundle_version + bundle_pointer + purge_job tables.
- worker: purge-queue consumer (drains pending purge jobs; emits CdnPurgeRequested events).

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-frontend-bundle-serve-kernel/{Cargo.toml,src/{lib,entities,ports,errors}.rs}` | create |
| `microservices/application/src/crates/oya-application-frontend-bundle-serve-usecase/{Cargo.toml,src/{lib,promote,purge,read_pointer}.rs}` | create |
| `microservices/application/src/crates/oya-application-frontend-bundle-serve-api/{Cargo.toml,src/lib.rs}` | create |
| `microservices/application/src/crates/oya-application-frontend-bundle-serve-adapter/{Cargo.toml,src/lib.rs}` | create — protocol-neutral |
| `microservices/application/src/crates/oya-application-frontend-bundle-serve-adapter-cdn/{Cargo.toml,src/{lib,oci_cdn,cloudflare}.rs}` | create |
| `microservices/application/src/crates/oya-application-frontend-bundle-serve-adapter-postgres/{Cargo.toml,src/lib.rs,migrations/*.sql}` | create |
| `microservices/application/src/crates/oya-application-frontend-bundle-serve-worker/{Cargo.toml,src/{lib,main,purge_consumer}.rs}` | create |
| 7 × catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
// worker
pub async fn run_purge_consumer(deps: WorkerDeps) {
    loop {
        let jobs = deps.queue.dequeue_batch(10, Duration::from_secs(60)).await;
        for job in jobs {
            match deps.cdn.purge(&job.pack, &job.pattern).await {
                Ok(()) => deps.queue.ack(&job).await,
                Err(e) => deps.queue.nack_with_backoff(&job, e).await,
            }
            deps.metrics.purge_job_completed(&job.pack);
        }
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-frontend-bundle-serve-usecase --all-features
cargo nextest run -p oya-application-frontend-bundle-serve-worker --all-features
cargo nextest run -p oya-application-frontend-bundle-serve-adapter-cdn --all-features
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_purge_under_60s_p99` | CDN purge budget |
| `test_pointer_revert_on_rollback` | revert path |
| `test_purge_queue_drains` | worker progress |
| `test_purge_idempotent` | duplicate purge no-op |
| `test_cdn_origin_shield_mtls` | mTLS to origin |

Coverage: 85 % / 75 %.

## Next IP

[`IP-014-leptos-frontend-and-composition.md`](IP-014-leptos-frontend-and-composition.md)
