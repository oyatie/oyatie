---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-009-evidence-emitter-adapter-rest-worker
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, openapi-rest-route-parity]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: oya-governance-evidence-emitter-{usecase,adapter,rest,worker,sdk,app}

## Intent

Complete `evidence-emitter` BC: usecase orchestrator + adapter (Postgres CRUD + S3 evidence write + OpenBao Ed25519 sign + audit-chain seal client) + REST replay surface + long-lived seal worker + SDK + app.

## ChangeSet boundary

6 crates.

## Concrete File Targets

| Crate | Files |
|---|---|
| `-usecase` | `src/emit_orchestrator.rs`, `src/seal_orchestrator.rs`, `src/replay_orchestrator.rs` |
| `-adapter` | `src/postgres_finding_persistence.rs`, `src/s3_evidence_writer.rs`, `src/openbao_signer.rs`, `src/audit_chain_client.rs` |
| `-rest` | `src/handlers/{findings,evidence,evidence_export}.rs` per OpenAPI |
| `-worker` | `src/main.rs` (seal-reconciler per F-03 mitigation) |
| `-sdk` | `src/client.rs` |
| `-app` | `src/main.rs` (composition root) |

## Code Shape

```rust
// usecase/src/emit_orchestrator.rs
pub async fn emit_finding(
    raw_finding: RawFinding,
    persistence: &dyn FindingPersistence,
    sealer: &dyn AuditChainSealer,
) -> Result<Finding, UsecaseError> {
    let canonical = canonicalize(&raw_finding.into_json());
    let hash = sha256(&canonical);
    let signed = sealer.sign(&canonical).await?;
    let finding = Finding { signature: signed.signature, finding_hash: hash, ..raw_finding.into() };
    persistence.insert(&finding).await?;     // outbox: durable before seal
    sealer.enqueue_seal(&finding).await?;    // async seal via audit-chain µservice
    Ok(finding)
}
```

```rust
// adapter/src/audit_chain_client.rs
pub struct AuditChainClient { /* gRPC client to audit-chain µservice */ }

#[async_trait::async_trait]
impl AuditChainSealer for AuditChainClient {
    async fn sign(&self, payload: &[u8]) -> Result<SignedPayload, KernelError> {
        // Ed25519 sign via OpenBao-issued key
        todo!()
    }
    async fn enqueue_seal(&self, f: &Finding) -> Result<(), KernelError> { todo!() }
}
```

```rust
// worker/src/main.rs (seal-reconciler)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let persistence = PostgresFindingPersistence::connect().await?;
    let sealer = AuditChainClient::new().await?;

    // Per F-03: every 60s, scan for unsealed Findings older than 5min
    loop {
        let unsealed = persistence.list_unsealed_older_than(Duration::seconds(300)).await?;
        for f in unsealed {
            sealer.enqueue_seal(&f).await?;
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-governance-evidence-emitter-{usecase,adapter,rest,worker,sdk,app}
cargo nextest run --workspace
cargo run -p oya-dev-cli -- gate validate openapi-rest-route-parity --microservice governance
```

## Test Plan

| Test | Verifies |
|---|---|
| `usecase::test_emit_outbox_durability` | Finding persists before seal |
| `usecase::test_replay_signature_verify` | replay returns valid sig |
| `adapter::test_s3_evidence_writer_object_lock` | object-lock applied |
| `adapter::test_openbao_signer_rotation` | rotated-out key refused |
| `worker::test_reconciler_drains_after_audit_chain_recovery` | F-03 recovery |

## Halt Conditions

- Seal latency > 1s p99 → tune audit-chain client; possibly batch.
- Signing with rotated-out key → halt; verify OpenBao key freshness.

## Next IP

[`IP-010-aggregation-indexer-full-stack.md`](IP-010-aggregation-indexer-full-stack.md)

## References

- IP-008 (kernel + domain).
- `microservices/governance/failure-modes.md` F-03.
- `microservices/governance/runbooks/evidence-replay.md`.
- `microservices/audit-chain/PRD.md` (upstream).
