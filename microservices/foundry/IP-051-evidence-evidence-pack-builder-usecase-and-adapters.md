---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-006-evidence-pack-builder-usecase-and-adapters
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence
acceptance_lanes: [cargo-clippy, cargo-doc, lean-layer-correctness, lean-cross-microservice-import-forbidden, integration-tests]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: Evidence-pack-builder usecase + adapters + worker + app

## Intent

`oya-foundry-evidence-evidence-pack-builder-{usecase,api,adapter,adapter-postgres,adapter-s3,adapter-audit-chain-bridge,worker,app}`: the orchestration that pulls signals, runs domain assembly, persists to Postgres + stages to substrate WORM through the bridge, emits to audit-chain. Per ADR-0131 substrate split, the audit-chain-bridge adapter is the ONLY crate that imports `oya-audit-chain-emission-sdk`.

## ChangeSet boundary

8 Rust crates (one per layer per the catalog plan). Cross-BC imports forbidden except own-BC api re-exports.

## Concrete File Targets (high-level; full file list inside each crate skeleton)

| Crate | Layer | Purpose |
|---|---|---|
| `oya-foundry-evidence-evidence-pack-builder-usecase` | usecase | `BuildPackUsecase` orchestrates signal collection + domain assembly + emit |
| `oya-foundry-evidence-evidence-pack-builder-api` | api | re-exports of usecase + entity types for adapters + REST |
| `oya-foundry-evidence-evidence-pack-builder-adapter` | adapter | generic glue + retry policy |
| `oya-foundry-evidence-evidence-pack-builder-adapter-postgres` | adapter | implements `EvidenceIndexWriterPort` against Postgres; INSERT-only role |
| `oya-foundry-evidence-evidence-pack-builder-adapter-s3` | adapter | implements `DeadLetterStorePort` against S3 (foundry-evidence-owned dead-letter bucket, NOT the substrate WORM) |
| `oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge` | adapter | implements `AuditChainBridgePort` against `oya-audit-chain-emission-sdk` |
| `oya-foundry-evidence-evidence-pack-builder-worker` | worker | leader-elected pack-assembly daemon; consumes Workflow events |
| `oya-foundry-evidence-evidence-pack-builder-app` | app | binary entrypoint |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-evidence-pack-builder-usecase
cargo check -p oya-foundry-evidence-evidence-pack-builder-api
cargo check -p oya-foundry-evidence-evidence-pack-builder-adapter
cargo check -p oya-foundry-evidence-evidence-pack-builder-adapter-postgres
cargo check -p oya-foundry-evidence-evidence-pack-builder-adapter-s3
cargo check -p oya-foundry-evidence-evidence-pack-builder-adapter-audit-chain-bridge
cargo check -p oya-foundry-evidence-evidence-pack-builder-worker
cargo check -p oya-foundry-evidence-evidence-pack-builder-app
cargo nextest run -p oya-foundry-evidence-evidence-pack-builder-usecase --test pack_assembly_happy_path
cargo nextest run -p oya-foundry-evidence-evidence-pack-builder-worker --test late_substrate_dead_letter
cargo run -p oya-dev-cli -- gate validate cross-microservice-import-forbidden --microservice foundry-evidence
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice foundry-evidence
```

## Halt Conditions

- Any non-bridge adapter imports `oya-audit-chain-*` — block.
- adapter-postgres uses non-INSERT-only role for default write path — block (EPI-04).
- Worker writes to the substrate WORM bucket directly — block (must go through the bridge → substrate emit).

## Next IP

[`IP-007-capability-invocation-recorder-stack.md`](IP-007-capability-invocation-recorder-stack.md)

## References

- ADR-0105 + ADR-0131.
- `policy/evidence-pack-integrity.md`.
