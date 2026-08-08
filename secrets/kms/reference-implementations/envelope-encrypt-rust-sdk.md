# Reference implementation — Envelope-encrypt + rotate + cryptoshred via `oya-cloud-kms-sdk`

Runnable Rust program that mints a CMK, envelope-encrypts a customer PII row, rotates the KEK, decrypts after rotation,
and walks a cryptoshred receipt verification.

## `Cargo.toml`

```toml
[package]
name = "kms-envelope-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-cloud-kms-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::Result;
use oya_cloud_kms_sdk::{
    Aad, CmkAlias, CmkCreateRequest, CmkRotationCadence, CryptoshredReason, KmsClient, KmsConfig,
    KmsError, Tenant,
};
use oya_trace::TraceContext;
use std::time::Duration;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let cfg = KmsConfig::builder()
        .endpoint("https://loopback.cloud-kms.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .service_account_credentials_path("/etc/oya/kms/sa-creds.json")
        .request_timeout(Duration::from_secs(3))
        .dek_issuance_deadline(Duration::from_millis(50))
        .build()?;

    let client = KmsClient::connect(cfg).await?;
    info!("connected to cloud-kms");

    // 1. Mint a CMK
    let cmk = client
        .cmk_create(
            CmkCreateRequest::builder()
                .alias(CmkAlias::parse("acme-customer-pii-demo")?)
                .algorithm("AES-256-GCM")
                .rotation_cadence(CmkRotationCadence::Days(30))
                .grace_window_days(90)
                .policy_tag("pii-data")
                .build()?,
            trace.child(),
        )
        .await?;
    info!(
        cmk_id = %cmk.id(),
        kek_version = cmk.current_kek_version(),
        next_rotation = %cmk.next_rotation_at(),
        "cmk minted"
    );

    // 2. Envelope-encrypt a payload
    let payload = serde_json::json!({
        "customer_id": "cust-42",
        "email": "jane@example.com",
        "ssn": "123-45-6789"
    });
    let payload_bytes = serde_json::to_vec(&payload)?;

    let aad = Aad::from(
        "tenant_id=oyatie.b2b.smb.acme-software;table=customers;row=cust-42".as_bytes(),
    );

    let ciphertext = client
        .encrypt(cmk.alias(), &payload_bytes, &aad, trace.child())
        .await?;
    info!(
        cmk_id = %ciphertext.cmk_id(),
        kek_version = ciphertext.kek_version(),
        length_bytes = ciphertext.bytes().len(),
        "encrypted"
    );

    // 3. Decrypt + assert round-trip
    let plaintext = client.decrypt(&ciphertext, &aad, trace.child()).await?;
    assert_eq!(plaintext, payload_bytes, "round-trip mismatch");
    info!("round-trip OK");

    // 4. Bad AAD must fail
    let bad_aad = Aad::from(
        "tenant_id=oyatie.b2b.smb.acme-software;table=customers;row=cust-99".as_bytes(),
    );
    match client.decrypt(&ciphertext, &bad_aad, trace.child()).await {
        Err(e) if matches!(e, KmsError::AeadAuthFailure { .. }) => {
            info!("expected AeadAuthFailure on AAD mismatch");
        }
        other => panic!("expected AeadAuthFailure; got {:?}", other),
    }

    // 5. Rotate the KEK
    let rotated = client
        .cmk_rotate(cmk.alias(), "demo: rotate", trace.child())
        .await?;
    info!(
        previous_kek_version = rotated.previous_kek_version(),
        new_kek_version = rotated.new_kek_version(),
        previous_destroy_at = %rotated.previous_kek_destroy_at(),
        "rotated"
    );

    // 6. Decrypt old ciphertext under the new KEK
    let plaintext_after = client.decrypt(&ciphertext, &aad, trace.child()).await?;
    assert_eq!(plaintext_after, payload_bytes, "post-rotation round-trip mismatch");
    info!("decrypt-after-rotation OK (KEK v{} decrypt-only)", rotated.previous_kek_version());

    // 7. Cryptoshred
    let shred = client
        .cryptoshred(
            cmk.alias(),
            CryptoshredReason::new("demo cleanup"),
            /* confirm_irreversible= */ true,
            trace.child(),
        )
        .await?;
    info!(
        cmk_id = %shred.cmk_id(),
        propagation_deadline = %shred.propagation_deadline(),
        hsm_destroy_attestations = shred.hsm_attestations().len(),
        "cryptoshredded"
    );

    // 8. Decrypt must now fail
    match client.decrypt(&ciphertext, &aad, trace.child()).await {
        Err(KmsError::CmkCryptoshredded { cmk_id, .. }) => {
            info!(%cmk_id, "decrypt refused (cmk cryptoshredded) as expected");
        }
        other => panic!("expected CmkCryptoshredded; got {:?}", other),
    }

    // 9. Pull the cryptoshred receipt for compliance evidence
    let receipt = client
        .extract_cryptoshred_receipt(shred.audit_chain_event_id(), trace.child())
        .await?;
    info!(
        chain_root = %receipt.chain_root(),
        hsm_quote_kind = %receipt.hsm_quote_kind(),
        signed_by = %receipt.signing_key_id(),
        "cryptoshred receipt extracted"
    );

    Ok(())
}
```

## Run it

```bash
cargo run --release
```

Expected output (trimmed):
```
INFO  connected to cloud-kms
INFO  cmk minted cmk_id=cmk-… kek_version=1 next_rotation=2026-06-19T…
INFO  encrypted cmk_id=cmk-… kek_version=1 length_bytes=153
INFO  round-trip OK
INFO  expected AeadAuthFailure on AAD mismatch
INFO  rotated previous_kek_version=1 new_kek_version=2 previous_destroy_at=2026-08-18T…
INFO  decrypt-after-rotation OK (KEK v1 decrypt-only)
INFO  cryptoshredded cmk_id=cmk-… propagation_deadline=2026-05-20T13:34:… hsm_destroy_attestations=1
INFO  decrypt refused (cmk cryptoshredded) as expected
INFO  cryptoshred receipt extracted chain_root=blake3-256:… hsm_quote_kind=tdx-attestation-v1 signed_by=hsm-paid-key-08
```

## SDK correctness guarantees

1. `encrypt(...)` is **strict on AAD**: omitting AAD or passing an empty `Aad` returns `KmsError::AadRequired`.
2. `decrypt(...)` validates AAD before returning plaintext — never partial-trust.
3. `cmk_rotate(...)` is atomic — either the new KEK is fully provisioned and the old is decrypt-only, or no change.
4. `cryptoshred(...)` requires `confirm_irreversible=true` and refuses without an explicit reason; UI/CLI integration must
   capture both.
5. `extract_cryptoshred_receipt(...)` returns a self-contained receipt — HSM measured-boot quote + zeroization confirmation +
   BLAKE3 chain root — sufficient for GDPR/CCPA evidence.

## Tests

```bash
cargo test --features hermetic
```

The `hermetic` feature uses `oya_cloud_kms_sdk::testkit::Hermetic` with SoftHSM 2.6.1 as the in-process HSM; tests finish in
≤ 45 s and do not require a real HSM.

## Error budget

`KmsError::HsmClusterDegraded` indicates ≥ 2 HSMs in the cluster are unreachable. Do not retry — the SDK already retried across
peer HSMs. File a `cloud_kms.slo.hsm_cluster_degraded` event so the on-call rotation can engage.
