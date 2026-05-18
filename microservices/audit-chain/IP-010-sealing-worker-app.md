---
doc_class: ImplementationPlan
impl_plan_id: IP-010-sealing-worker-app
status: pending
owner: axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, layer-correctness, shardability, statelessness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: sealing-usecase + sealing-api + sealing-adapter + sealing-worker + sealing-app

## Intent

Tie sealing-kernel + sealing-domain + sealing-adapter-{hsm,postgres,s3} together via the usecase orchestrator + leader-elected worker daemon + composition root.

## Crates introduced (5)

- `oya-audit-chain-sealing-usecase`
- `oya-audit-chain-sealing-api`
- `oya-audit-chain-sealing-adapter` (protocol-neutral; in-process Merkle build coordination + RootPublisher impl)
- `oya-audit-chain-sealing-worker` (long-lived daemon; leader-elected per `(pack, tenant_partition)` shard)
- `oya-audit-chain-sealing-app` (composition root binary)

## Code Shape (worker)

```rust
// sealing-worker/src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    let config = SealingConfig::from_env()?;
    let pack = config.pack;
    let shard = config.shard;

    // HA leader-election via Kubernetes Lease
    let leader_lease = kube_leader_election::start(
        &format!("sealing-worker-{pack}-{shard}-leader"),
        Duration::from_secs(30),
    ).await?;

    leader_lease.run_when_leader(|| async {
        let signer = HsmSigner::new(&config.hsm_partition).await?;
        let publisher = MultiChannelPublisher::new(&config.s3, &config.mimir, &config.git_publisher);
        let usecase = SealingUseCase::new(signer, publisher, postgres_index, s3_writer);

        loop {
            let period_id = current_period(&pack);
            let envelopes = wal_reader.read_for_period(&pack, &shard, &period_id).await?;
            usecase.seal_period(&pack, &shard, &period_id, &envelopes).await?;
            tokio::time::sleep(period_duration_for(&pack)).await;
        }
    }).await
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-audit-chain-sealing-usecase
cargo nextest run -p oya-audit-chain-sealing-worker --features integration
cargo run -p oya-dev-cli -- gate validate shardability --crate oya-audit-chain-sealing-worker
cargo run -p oya-dev-cli -- gate validate statelessness --crate oya-audit-chain-sealing-rest
```

## End-to-end drill

```bash
# seal-latency drill
cargo nextest run -p oya-audit-chain-sealing-worker --test seal_latency_drill
# Expectation: 100 events emitted; seal complete with valid Merkle root + Ed25519 signature within ≤ 1s p99
```

## References

- Bominal ADR-0028 §"Sealing process".
- `microservices/audit-chain/policy/seal-integrity.md`.
- Kubernetes lease docs.
