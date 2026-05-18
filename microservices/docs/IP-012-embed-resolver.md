---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-012-embed-resolver
status: pending
execution_unit: ChangeSet
owner: axis-docs
acceptance_lanes: [cargo-check, cargo-clippy, cargo-nextest, oya-governance-embed-resolver-acl-passthrough]
---

# IP-012: embed-resolver BC (8 crates)

## Intent

Implement cross-µservice embed resolution to workflow-studio (canvases) + sheets (cells) + slides (decks). Cross-µservice mTLS; source-side ACL passthrough; cycle detection bounded at depth 3; stale-fallback. TTL ≤ 5 min with jitter; single-flight coalescing.

## ChangeSet boundary

8 crates: kernel + domain + usecase + api + adapter + rest + worker + app.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/docs/src/crates/oya-docs-embed-resolver-{kernel,domain,usecase,api,adapter,rest,worker,app}/src/lib.rs` | create |
| `microservices/docs/src/crates/oya-docs-embed-resolver-domain/src/{cycle_detection,acl_passthrough,stale_fallback}.rs` | create |
| `microservices/docs/src/crates/oya-docs-embed-resolver-adapter/src/{lib,workflow_studio_client,sheets_client,slides_client}.rs` | create |
| `microservices/docs/src/crates/oya-docs-embed-resolver-worker/src/{lib,refresh_worker,grant_revocation_listener}.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-docs-embed-resolver-domain -- cycle_detection
cargo nextest run -p oya-docs-embed-resolver-domain -- stale_fallback  # AC-15
cargo nextest run -p oya-docs-embed-resolver-domain -- acl_passthrough_source_side
cargo run -p oya-dev-cli -- gate validate embed-resolver-acl-passthrough --microservice docs
```

## References

- ADR-DOCS-0004 (per-block ACL; embed-resolver source-side passthrough).
- `policy/data-residency.md` Invariant DR-04 (cross-pack embed snapshot-only).
