---
doc_class: ImplementationPlan
impl_plan_id: IP-009-sealing-adapter-postgres-s3
status: pending
owner: axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, layer-correctness]
---

# IP-009: sealing-adapter-postgres + sealing-adapter-s3

## Intent

Two backend-qualified adapters per ADR-0105 Amendment 3:
- `oya-audit-chain-sealing-adapter-postgres`: implements `IndexWriter`; INSERT SealRecord; never UPDATE/DELETE.
- `oya-audit-chain-sealing-adapter-s3`: implements `ObjectStoreWriter`; writes raw event blobs + Merkle-tree blobs to WORM-Object-Lock-Compliance bucket.

## Crate Naming

- `oya-audit-chain-sealing-adapter-postgres`
- `oya-audit-chain-sealing-adapter-s3`

## Concrete File Targets

| Path | Action |
|---|---|
| `.../src/crates/oya-audit-chain-sealing-adapter-postgres/Cargo.toml` | create — dep `sqlx` |
| `.../src/crates/oya-audit-chain-sealing-adapter-postgres/src/lib.rs` | create |
| `.../src/crates/oya-audit-chain-sealing-adapter-s3/Cargo.toml` | create — dep `aws-sdk-s3` |
| `.../src/crates/oya-audit-chain-sealing-adapter-s3/src/lib.rs` | create |
| Per-crate migrations + tests | create |

## Code Shape (postgres)

```rust
#[async_trait]
impl IndexWriter for PostgresIndexWriter {
    async fn write_seal_record(&self, record: &SealRecord) -> Result<(), KernelError> {
        // INSERT-only; role `audit_sealer` lacks UPDATE/DELETE per policy/seal-integrity.md T-T-01
        sqlx::query!(
            "INSERT INTO seal_record (pack, tenant_partition, period_id, root_hash, signature,
             signer_public_key_fp, prior_root_hash, event_count, signed_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            ...
        ).execute(&self.pool).await?;
        Ok(())
    }
}
```

## Code Shape (s3)

```rust
#[async_trait]
impl ObjectStoreWriter for S3WormWriter {
    async fn write_tree(&self, pack: &PackId, period_id: &PeriodId, tree: &MerkleTree) -> Result<(), KernelError> {
        let key = format!("{}/{}/tree.bin", pack, period_id);
        let serialized = tree.canonical_serialize();
        let sha = sha256(&serialized);

        // PUT with Object Lock retention mode COMPLIANCE
        self.client.put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(serialized.into())
            .object_lock_mode(ObjectLockMode::Compliance)
            .object_lock_retain_until_date(retention_date_for(pack))
            .server_side_encryption(ServerSideEncryption::AwsKms)
            .ssekms_key_id(&self.kms_key)
            .send().await?;

        // Local-verify: read back; recompute SHA; compare
        let readback = self.client.get_object().bucket(&self.bucket).key(&key).send().await?;
        let readback_sha = sha256(&readback.body.collect().await?);
        if readback_sha != sha {
            return Err(KernelError::S3ReadbackMismatch);
        }
        Ok(())
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-audit-chain-sealing-adapter-postgres --features integration-postgres
cargo nextest run -p oya-audit-chain-sealing-adapter-s3 --features integration-s3
cargo run -p oya-dev-cli -- gate validate audit-chain-postgres-role-conformance
```

## References

- Bominal ADR-0028 §"Storage backends".
- `microservices/audit-chain/policy/seal-integrity.md` §"SI-04" + §"FM-SI-03".
- AWS S3 Object Lock docs (OCI Object Storage WORM equivalent).
