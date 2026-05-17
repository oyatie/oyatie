---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-004-mailbox-store-s3-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, port-location, layer-correctness, oya-governance-encryption-tenant-dek]
---

# IP-004: oya-mail-mailbox-store-adapter-s3

## Intent

Implement `MimeBlobStore` against S3-compatible object storage (OCI Object Storage primary; AWS S3 fallback) with SSE-KMS envelope encryption under tenant DEK per Bominal ADR-0111. Content-addressable (CAS) by SHA-256; per-tenant prefix; object-lock for HIPAA + KR-FSS packs.

## ChangeSet boundary

One Rust crate at `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-s3/`. Bucket-policy IaC at `microservices/mail/iac/s3-blob/`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-s3/Cargo.toml` | create | `aws-sdk-s3` (OCI S3 compat) + ring/rustls + kernel dep |
| `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-s3/src/lib.rs` | create | exports |
| `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-s3/src/blob_store.rs` | create | `S3MimeBlobStore` impl |
| `microservices/mail/src/crates/oya-mail-mailbox-store-adapter-s3/src/envelope.rs` | create | tenant-DEK envelope wrap/unwrap |
| `microservices/mail/iac/s3-blob/bucket-policy-baseline.json` | create | per-tenant prefix policy template |
| `microservices/mail/iac/s3-blob/bucket-policy-hipaa.json` | create | object-lock + cross-region replication blocked |
| `microservices/mail/iac/s3-blob/bucket-policy-kr-fss.json` | create | KR-resident KMS key required; 5y retention floor |
| `microservices/mail/catalog/oya-mail-mailbox-store-adapter-s3.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-mail-mailbox-store-adapter-s3
JUSTIFICATION:
- microservice = mail
- bc-tokens = mailbox-store
- layer = adapter (backend-qualified)
- backend = s3
- exemptions claimed: none
```

## Code Shape

```rust
// src/blob_store.rs
pub struct S3MimeBlobStore {
    client: aws_sdk_s3::Client,
    bucket: String,
    kms: TenantKmsClient,
}

#[async_trait]
impl MimeBlobStore for S3MimeBlobStore {
    async fn put(&self, blob: MimeBlob, tenant: Option<TenantId>) -> Result<BlobRef, BlobError> {
        let dek = self.kms.fetch_dek(tenant.as_ref()).await?;
        let nonce = generate_random_nonce();
        let ciphertext = aes_gcm_256_encrypt(&dek, &nonce, blob.bytes())?;
        let sha = sha256(&ciphertext);
        let key = format!("tenants/{}/blobs/{:x}/{:x}",
                          tenant.as_deref().unwrap_or("personal"), &sha[..2], &sha[2..]);
        self.client.put_object()
            .bucket(&self.bucket).key(&key).body(ciphertext.into())
            .ssekms_key_id(self.kms.kek_arn_for(tenant.as_ref()).await?)
            .server_side_encryption(aws_sdk_s3::types::ServerSideEncryption::AwsKms)
            .send().await?;
        Ok(BlobRef { key, sha, dek_id: dek.id })
    }
    async fn get(&self, r: BlobRef, tenant: Option<TenantId>) -> Result<MimeBlob, BlobError> {
        let dek = self.kms.fetch_dek_by_id(&r.dek_id, tenant.as_ref()).await?;
        let obj = self.client.get_object().bucket(&self.bucket).key(&r.key).send().await?;
        let plaintext = aes_gcm_256_decrypt(&dek, /* nonce in metadata */, obj.body_bytes())?;
        Ok(MimeBlob::new(plaintext))
    }
    // ... delete (subject to object-lock + retention)
}
```

## Acceptance Gates

```bash
cargo check -p oya-mail-mailbox-store-adapter-s3
cargo clippy -p oya-mail-mailbox-store-adapter-s3 -- -D warnings
cargo nextest run -p oya-mail-mailbox-store-adapter-s3 --features integration-test
cargo run -p oya-dev-cli -- gate validate encryption-tenant-dek --microservice mail
```

## Test Plan

- Unit: envelope encrypt/decrypt round-trip; tampering (modify ciphertext) decrypts to error.
- Integration: minio-container + KMS-mock; per-tenant scoped fetch verifies cross-tenant deny.
- Object-lock test: pack-us-healthcare overlay applied; DELETE refused within retention.

## Halt Conditions

- DEK fetched without tenant scope — refactor; KMS policy must refuse.
- Object-lock not applied where pack requires — fail lane.

## Next IP

[`IP-005-dual-context-isolation.md`](IP-005-dual-context-isolation.md)

## References

- ADR-0111 (Bominal: envelope encryption)
- ADR-0117 (residency)
- AWS S3 Object Lock — `docs.aws.amazon.com/AmazonS3/latest/userguide/object-lock.html`
- OCI Object Storage — `docs.oracle.com/iaas/Content/Object/home.htm`
- NIST SP 800-57 (key management)
- HIPAA §164.312(a)(2)(iv) (encryption + decryption)
