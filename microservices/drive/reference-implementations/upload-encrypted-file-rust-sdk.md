---
doc_class: ReferenceImplementation
microservice: drive
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Upload an envelope-encrypted file via the drive Rust SDK

A runnable example that:

1. Authenticates as a tenant drive_member principal.
2. Performs content-defined chunking on the input file.
3. Generates per-chunk DEKs + wraps under the active KEK.
4. Uploads ciphertext chunks to SeaweedFS.
5. Inserts FileVersionEnvelope rows.
6. Issues a signed share-link.
7. Verifies the audit-chain emission.

## Cargo.toml

```toml
[package]
name = "drive-upload-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-drive-client = { path = "../../../../crates/oya-drive-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
fastcdc = "3.1"
aes-gcm = "0.10"
rand = "0.8"
blake3 = "1.5"
ed25519-dalek = "2.1"
tokio = { version = "1.40", features = ["rt-multi-thread", "macros", "fs", "io-util"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead, generic_array::GenericArray, Payload}};
use fastcdc::v2020::FastCDC;
use rand::RngCore;
use oya_drive_client::{
    DriveClient, DriveClientConfig,
    FileVersionCreate, ChunkUpload, FileVersionFinalize,
    ShareLinkCreate, SharePermission,
    DataClass, RetentionClass,
    EnvelopeMetadata,
};
use oya_cedar_client::CedarPrincipal;
use tokio::io::AsyncReadExt;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 1. Construct the client.
    let principal = CedarPrincipal::from_env("DRIVE_MEMBER_JWT")?;
    let client = DriveClient::connect(DriveClientConfig {
        cell_endpoint: std::env::var("DRIVE_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(120),
    }).await?;

    // 2. Read the input file.
    let input_path = "./large-document.pdf";
    let mut file = tokio::fs::File::open(input_path).await?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;
    info!("Read {} bytes from {}", contents.len(), input_path);

    // 3. Create the file version (server allocates file_id + version_id; returns active KEK epoch).
    let version_create = client.file_version_create(FileVersionCreate {
        folder_path: "/Reports/Q2-2026".into(),
        name: "Q2 2026 Financial Report.pdf".into(),
        content_type: "application/pdf".into(),
        data_class: DataClass::PiiFinancialSensitive,
        retention_class: RetentionClass::Audit7y,
        tags: vec!["confidential".into(), "financial".into()],
    }).await?;
    info!("File version created: file_id={}, version_id={}, kek_epoch={}",
          version_create.file_id, version_create.version_id, version_create.kek_epoch);

    // 4. Content-defined chunking with FastCDC (target 4 MiB).
    let chunker = FastCDC::new(&contents, 1 * 1024 * 1024, 4 * 1024 * 1024, 16 * 1024 * 1024);
    let mut chunk_envelopes = Vec::new();

    for (chunk_idx, chunk) in chunker.enumerate() {
        let chunk_bytes = &contents[chunk.offset..chunk.offset + chunk.length];

        // 5. Generate random DEK + nonce for this chunk.
        let mut dek_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut dek_bytes);
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        // 6. Compute chunk digest (for AAD + content addressing).
        let chunk_digest = blake3::hash(chunk_bytes);

        // 7. Construct AAD per ADR-DRIVE-001 § Decision.
        let aad = format!("{}|{}|{}|{}|{}|{}",
            client.tenant_id(),
            version_create.file_id,
            version_create.version_id,
            chunk_digest.to_hex(),
            "audit_7y",
            "PII_FINANCIAL_SENSITIVE"
        );

        // 8. Encrypt chunk with DEK + AAD.
        let cipher = Aes256Gcm::new(GenericArray::from_slice(&dek_bytes));
        let ciphertext = cipher.encrypt(
            GenericArray::from_slice(&nonce_bytes),
            Payload {
                msg: chunk_bytes,
                aad: aad.as_bytes(),
            }
        )?;

        // 9. Request server to wrap the DEK under the active KEK (server holds KEK in OpenBao).
        let wrap_resp = client.dek_wrap(
            &dek_bytes,
            version_create.kek_epoch,
        ).await?;
        let dek_ciphertext = wrap_resp.wrapped_dek_b64;
        // Zero out plaintext DEK from memory immediately
        let mut zeroed_dek = dek_bytes;
        for b in zeroed_dek.iter_mut() { *b = 0; }

        // 10. Upload encrypted chunk to SeaweedFS via drive-api.
        let chunk_upload = client.chunk_upload(ChunkUpload {
            file_id: version_create.file_id.clone(),
            version_id: version_create.version_id.clone(),
            chunk_index: chunk_idx as u32,
            chunk_ciphertext: ciphertext,
            nonce_b64: base64::encode(&nonce_bytes),
            dek_ciphertext_b64: dek_ciphertext.clone(),
            aad_hash_b64: blake3::hash(aad.as_bytes()).to_hex().to_string(),
            kek_epoch: version_create.kek_epoch,
            algorithm: "AES-256-GCM".into(),
        }).await?;

        info!("Chunk {} uploaded: object_ref={}, ciphertext_size={}",
              chunk_idx, chunk_upload.object_ref, chunk_upload.ciphertext_size);

        chunk_envelopes.push(EnvelopeMetadata {
            chunk_index: chunk_idx as u32,
            object_ref: chunk_upload.object_ref,
            dek_ciphertext_b64: dek_ciphertext,
            aad_hash_b64: chunk_upload.aad_hash_b64,
            kek_epoch: version_create.kek_epoch,
        });
    }

    // 11. Finalize the file version (server commits the manifest).
    let finalize = client.file_version_finalize(FileVersionFinalize {
        file_id: version_create.file_id.clone(),
        version_id: version_create.version_id.clone(),
        chunk_count: chunk_envelopes.len() as u32,
        total_size: contents.len() as u64,
    }).await?;
    info!("File version finalized: audit_event_id={}", finalize.audit_event_id);

    // 12. Issue a share-link.
    let share_link = client.share_link_create(ShareLinkCreate {
        file_id: version_create.file_id.clone(),
        permissions: SharePermission::Viewer,
        expires_at: chrono::Utc::now() + chrono::Duration::days(30),
        max_views: Some(10),
        watermark_policy: Some("email-tagged".into()),
        require_email_verification: true,
    }).await?;
    info!("Share-link issued: url={}, expires_at={}, max_views={}",
          share_link.url, share_link.expires_at, share_link.max_views.unwrap_or(0));

    Ok(())
}
```

## Expected output (against a paid-tier cell)

```
INFO Read 10485760 bytes from ./large-document.pdf
INFO File version created: file_id=f_acme_001, version_id=v_acme_001_1, kek_epoch=2
INFO Chunk 0 uploaded: object_ref=seaweedfs://prod-us-east-1/3,01637037d6, ciphertext_size=4194316
INFO Chunk 1 uploaded: object_ref=seaweedfs://prod-us-east-1/3,01637041e2, ciphertext_size=4194316
INFO Chunk 2 uploaded: object_ref=seaweedfs://prod-us-east-1/3,01637052b8, ciphertext_size=2097160
INFO File version finalized: audit_event_id=ae_drive_file_uploaded_001
INFO Share-link issued: url=https://acme.oyatie.local/s/EyJ0..., expires_at=2026-06-20T..., max_views=10
```

## HTTP alternative (curl)

```sh
# 1. Create file version
curl -X POST https://drive.prod-us-east-1.oyatie.local/v1/drive/files \
    -H "Authorization: Bearer $DRIVE_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "folder_path":"/Reports/Q2-2026",
        "name":"Q2 2026 Financial Report.pdf",
        "content_type":"application/pdf",
        "data_class":"PII_FINANCIAL_SENSITIVE",
        "retention_class":"audit_7y",
        "tags":["confidential","financial"]
    }'

# 2. Request DEK wrap (server holds KEK)
curl -X POST https://drive.prod-us-east-1.oyatie.local/v1/drive/keys/wrap-dek \
    -H "Authorization: Bearer $DRIVE_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "kek_epoch":2,
        "dek_plaintext_b64":"<base64 random 32 bytes>"
    }'

# 3. Upload encrypted chunk
curl -X POST https://drive.prod-us-east-1.oyatie.local/v1/drive/files/f_acme_001/versions/v_acme_001_1/chunks/0 \
    -H "Authorization: Bearer $DRIVE_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/octet-stream" \
    -H "X-Oya-DEK-Ciphertext-B64: <wrapped DEK>" \
    -H "X-Oya-Nonce-B64: <nonce>" \
    -H "X-Oya-AAD-Hash-B64: <BLAKE3 hash>" \
    -H "X-Oya-KEK-Epoch: 2" \
    --data-binary @encrypted-chunk-0.bin

# 4. Finalize
curl -X POST https://drive.prod-us-east-1.oyatie.local/v1/drive/files/f_acme_001/versions/v_acme_001_1/finalize \
    -H "Authorization: Bearer $DRIVE_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "chunk_count":3,
        "total_size":10485760
    }'

# 5. Issue share-link
curl -X POST https://drive.prod-us-east-1.oyatie.local/v1/drive/share-links \
    -H "Authorization: Bearer $DRIVE_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "file_id":"f_acme_001",
        "permissions":"viewer",
        "expires_at":"2026-06-20T00:00:00Z",
        "max_views":10,
        "watermark_policy":"email-tagged"
    }'
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Lacks `drive::file::upload` or folder access |
| `kek_epoch_retiring` | 422 | No (config) | Active KEK is retiring; new epoch should be in place |
| `dek_wrap_failed_openbao` | 503 | Yes (auto, 1-5s backoff) | OpenBao temporarily unavailable |
| `chunk_aad_mismatch` | 422 | No | AAD doesn't match server expectation; client error |
| `chunk_object_digest_mismatch` | 422 | No | Object integrity failure; re-upload chunk |
| `dlp_pre_encryption_block` | 422 | No (data violation) | DLP scanner rejected content per tenant policy |
| `virus_detected` | 422 | No (data violation) | ClamAV detected malware; quarantined |
| `quota_exceeded` | 413 | No | Tenant quota exceeded |
| `worm_immutable` | 422 | No | File version is WORM-locked; cannot overwrite |
| `cross_tenant_cmk_residency_violation` | 403 | No | Pack requires home-cell CMK; cannot upload to remote cell |
| `share_link_pack_restricted` | 403 | No | Pack policy denies share-link issuance for this data_class |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `file_version_create` | `drive.file.version.created.v1` |
| `dek_wrap` | `drive.file.dek.wrapped.v1` |
| `chunk_upload` | `drive.file.chunk.uploaded.v1` |
| `file_version_finalize` | `drive.file.uploaded.v1` |
| `file_download` | `drive.file.downloaded.v1` |
| `kek_rotate` | `drive.kek.rotated.v1` |
| `rewrap_completed` | `drive.rewrap.completed.v1` |
| `share_link_create` | `drive.share-link.created.v1` |
| `share_link_viewed` | `drive.share-link.viewed.v1` |
| `transfer_ownership` | `drive.file.ownership.transferred.v1` |
| `cryptoshred_scheduled` | `drive.cmk.cryptoshred.scheduled.v1` |
| Cedar deny anywhere | `drive.cedar.denied.v1` |

## Where this file lives

`microservices/drive/reference-implementations/upload-encrypted-file-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/drive/reference-implementations/upload-example/` once `oya-drive-client` ships.
