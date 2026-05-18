---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-007-capability-invocation-recorder-stack
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence
acceptance_lanes: [cargo-clippy, cargo-doc, lean-layer-correctness, integration-tests, load-drill-record-invocation]
---

# IP-007: Capability-invocation-recorder REST stack

## Intent

`oya-foundry-evidence-capability-invocation-recorder-{domain,usecase,api,adapter,rest,sdk}`: synchronous REST receipt + WAL + dead-letter for record_invocation. p99 ≤ 500 ms.

## ChangeSet boundary

6 Rust crates. REST surface bound to OpenAPI contract.

## Concrete File Targets

| Crate | Layer | Notes |
|---|---|---|
| `oya-foundry-evidence-capability-invocation-recorder-domain` | domain | envelope validation rules |
| `oya-foundry-evidence-capability-invocation-recorder-usecase` | usecase | RecordInvocationUsecase: idempotency check + WAL append + pack-builder enqueue |
| `oya-foundry-evidence-capability-invocation-recorder-api` | api | re-exports |
| `oya-foundry-evidence-capability-invocation-recorder-adapter` | adapter | WAL adapter (Postgres + dead-letter cooperation) |
| `oya-foundry-evidence-capability-invocation-recorder-rest` | rest | axum router; OpenAPI conformance |
| `oya-foundry-evidence-capability-invocation-recorder-sdk` | sdk | Rust client; per `sdk-plan.md` |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-capability-invocation-recorder-domain
cargo check -p oya-foundry-evidence-capability-invocation-recorder-usecase
cargo check -p oya-foundry-evidence-capability-invocation-recorder-api
cargo check -p oya-foundry-evidence-capability-invocation-recorder-adapter
cargo check -p oya-foundry-evidence-capability-invocation-recorder-rest
cargo check -p oya-foundry-evidence-capability-invocation-recorder-sdk
cargo nextest run -p oya-foundry-evidence-capability-invocation-recorder-usecase --test record_happy_path
cargo nextest run -p oya-foundry-evidence-capability-invocation-recorder-rest --test openapi_conformance
cargo run -p oya-dev-cli -- gate validate cedar-tenant-scope --microservice foundry-evidence
oya gate validate load-drill-record-invocation --microservice foundry-evidence
# load-drill verifies p99 ≤ 500 ms sustained at peak target.
```

## Halt Conditions

- p99 record_invocation drill exceeds 500 ms — block; performance regression.
- Idempotency dedup misses a duplicate within 24 h — block (FR-01 contract).
- REST surface diverges from `contracts/openapi/foundry-evidence.yaml` — block.

## Next IP

[`IP-008-eval-evidence-aggregator.md`](IP-008-eval-evidence-aggregator.md)

## References

- `contracts/openapi/foundry-evidence.yaml`.
- ADR-0133 (load-drill claim assertion).
