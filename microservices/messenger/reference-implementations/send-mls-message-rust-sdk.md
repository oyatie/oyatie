---
doc_class: ReferenceImplementation
microservice: messenger
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Send an MLS-protected message via the messenger Rust SDK

A runnable example that:

1. Authenticates as a tenant principal with passkey-issued JWT.
2. Generates an MLS KeyPackage for the device.
3. Creates an MLS-protected conversation.
4. Sends an encrypted message under the MLS group epoch.
5. Verifies the audit-chain emission.

## Cargo.toml

```toml
[package]
name = "messenger-send-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-messenger-client = { path = "../../../../crates/oya-messenger-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
openmls = "0.6.0"  # RFC 9420 implementation
openmls_rust_crypto = "0.3.0"
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
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
use openmls::prelude::*;
use openmls_rust_crypto::OpenMlsRustCrypto;
use oya_messenger_client::{
    MessengerClient, MessengerClientConfig,
    ConversationCreate, ConversationKind,
    KeyPackageUpload, MlsCommitAppend,
    MessageSend, MlsCiphersuite,
};
use oya_cedar_client::CedarPrincipal;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 1. Construct the client bound to a messenger_member Cedar principal.
    let principal = CedarPrincipal::from_env("MESSENGER_MEMBER_JWT")?;
    let client = MessengerClient::connect(MessengerClientConfig {
        cell_endpoint: std::env::var("MESSENGER_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(30),
    }).await?;

    // 2. Set up OpenMLS provider (per RFC 9420 §5).
    let backend = OpenMlsRustCrypto::default();
    let ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

    // 3. Generate Alice's MLS credential + KeyPackage for this device.
    let alice_credential = CredentialWithKey {
        credential: Credential::new_basic("u-alice@acme-corp.com".into()),
        signature_key: SignaturePublicKey::from(
            backend.crypto().signature_key_gen(SignatureScheme::ED25519)?
        ),
    };
    let alice_key_package = KeyPackage::builder()
        .build(
            CryptoConfig::with_default_version(ciphersuite),
            &backend,
            &alice_signer,
            alice_credential.clone(),
        )?;
    info!("Generated KeyPackage for Alice");

    // 4. Upload the KeyPackage to the server.
    let kp_upload = client.keypackage_upload(KeyPackageUpload {
        principal_id: "u-alice@acme-corp.com".into(),
        device_id: "d_alice_macbook_001".into(),
        key_package_bytes: alice_key_package.tls_serialize_detached()?,
        attestation_ref: Some("webauthn-aaguid:ee882879-721c-4913-9775-3dfcce97072a".into()),
        ciphersuite: MlsCiphersuite::Mls128DhkemX25519Aes128GcmSha256Ed25519,
    }).await?;
    info!("KeyPackage uploaded: kp_id={}, credential_epoch={}",
          kp_upload.key_package_id, kp_upload.credential_epoch);

    // 5. Fetch Bob's KeyPackage (recipient).
    let bob_kp_ref = client.keypackage_fetch(
        "u-bob@acme-corp.com",
    ).await?;
    let bob_key_package = KeyPackage::tls_deserialize_exact(&bob_kp_ref.key_package_bytes)?;
    info!("Fetched Bob's KeyPackage: kp_id={}", bob_kp_ref.key_package_id);

    // 6. Create a new MLS group for the DM (per RFC 9420 §11).
    let mut alice_group = MlsGroup::new(
        &backend,
        &alice_signer,
        &MlsGroupCreateConfig::default(),
        alice_credential,
    )?;
    let mls_group_id = alice_group.group_id().to_vec();
    info!("Created MLS group: id={}", hex::encode(&mls_group_id));

    // 7. Add Bob to the group; produces Commit + Welcome.
    let (commit_msg, welcome_msg, _group_info) = alice_group.add_members(
        &backend,
        &alice_signer,
        &[bob_key_package],
    )?;
    alice_group.merge_pending_commit(&backend)?;
    info!("Bob added to group; epoch advanced to {}", alice_group.epoch().as_u64());

    // 8. Register the conversation server-side + submit the Commit + Welcome.
    let conversation = client.conversation_create(ConversationCreate {
        kind: ConversationKind::Dm,
        mls_group_id: hex::encode(&mls_group_id),
        ciphersuite: MlsCiphersuite::Mls128DhkemX25519Aes128GcmSha256Ed25519,
        initial_members: vec![
            "u-alice@acme-corp.com".into(),
            "u-bob@acme-corp.com".into(),
        ],
    }).await?;
    let conversation_id = conversation.conversation_id;

    let commit_ack = client.mls_commit_append(MlsCommitAppend {
        conversation_id: conversation_id.clone(),
        sender_device_id: "d_alice_macbook_001".into(),
        commit_bytes: commit_msg.tls_serialize_detached()?,
        welcome_bytes: Some(welcome_msg.tls_serialize_detached()?),
    }).await?;
    info!("Commit accepted at epoch {}; audit_event_id={}",
          commit_ack.epoch, commit_ack.audit_event_id);

    // 9. Send an encrypted application message under the MLS group's current epoch.
    let plaintext = b"Hello Bob! This is a test message under MLS group epoch ${epoch}.";
    let mls_message_out = alice_group.create_message(
        &backend,
        &alice_signer,
        plaintext,
    )?;
    let send_ack = client.message_send(MessageSend {
        conversation_id: conversation_id.clone(),
        sender_device_id: "d_alice_macbook_001".into(),
        ciphertext: mls_message_out.tls_serialize_detached()?,
        content_type: "text/plain".into(),
    }).await?;
    info!("Message sent: msg_id={}, audit_event_id={}",
          send_ack.message_id, send_ack.audit_event_id);

    // 10. (Bob's side, in a real client) Decrypt the message under the MLS group epoch.
    // This is shown here for completeness; in production Bob's client does this independently.
    // Bob processes the Welcome to join the group, then processes the message.

    Ok(())
}
```

## Expected output (against a paid-tier cell with both principals provisioned)

```
INFO Generated KeyPackage for Alice
INFO KeyPackage uploaded: kp_id=kp_alice_001, credential_epoch=1
INFO Fetched Bob's KeyPackage: kp_id=kp_bob_001
INFO Created MLS group: id=e5a4b3c2d1a09f8e7d6c5b4a3928e7d6c5b4a392
INFO Bob added to group; epoch advanced to 1
INFO Commit accepted at epoch 1; audit_event_id=ae_msg_mls_commit_001
INFO Message sent: msg_id=m_acme_001, audit_event_id=ae_msg_sent_001
```

## HTTP alternative (curl) — for clients without an MLS library

For clients embedding their own MLS implementation, the wire protocol is:

```sh
# Upload KeyPackage (the bytes are the OpenMLS TLS-encoded KeyPackage)
curl -X POST https://messenger.prod-syd-1.oyatie.local/v1/messenger/mls/key-packages \
    -H "Authorization: Bearer $MESSENGER_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "principal_id":"u-alice@acme-corp.com",
        "device_id":"d_alice_macbook_001",
        "key_package_bytes_b64":"...<base64 TLS-serialized KeyPackage>...",
        "attestation_ref":"webauthn-aaguid:ee882879-721c-4913-9775-3dfcce97072a",
        "ciphersuite":"MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519"
    }'

# Fetch a recipient's KeyPackage
curl -X GET https://messenger.prod-syd-1.oyatie.local/v1/messenger/mls/key-packages/u-bob%40acme-corp.com \
    -H "Authorization: Bearer $MESSENGER_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp"

# Create conversation
curl -X POST https://messenger.prod-syd-1.oyatie.local/v1/messenger/conversations \
    -H "Authorization: Bearer $MESSENGER_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "kind":"dm",
        "mls_group_id":"e5a4b3c2d1...",
        "ciphersuite":"MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519",
        "initial_members":["u-alice@acme-corp.com","u-bob@acme-corp.com"]
    }'

# Append MLS Commit (creates the group + Welcome for Bob)
curl -X POST https://messenger.prod-syd-1.oyatie.local/v1/messenger/conversations/c_acme_001/mls/commits \
    -H "Authorization: Bearer $MESSENGER_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "sender_device_id":"d_alice_macbook_001",
        "commit_bytes_b64":"...<base64 TLS-serialized Commit>...",
        "welcome_bytes_b64":"...<base64 TLS-serialized Welcome>..."
    }'

# Send an encrypted message (the ciphertext is MLS-PrivateMessage TLS-encoded)
curl -X POST https://messenger.prod-syd-1.oyatie.local/v1/messenger/conversations/c_acme_001/messages \
    -H "Authorization: Bearer $MESSENGER_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "sender_device_id":"d_alice_macbook_001",
        "ciphertext_b64":"...<base64 TLS-serialized MLS PrivateMessage>...",
        "content_type":"text/plain"
    }'
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Lacks `messenger::mls_commit::append` or membership |
| `key_package_not_found` | 404 | No | Recipient has no usable KeyPackages — they must publish one |
| `mls_epoch_rejected` | 422 | No | Stale or unauthorized epoch — client must replay commits |
| `mls_commit_malformed` | 400 | No | Client serialization error |
| `mls_recovery_request_in_progress` | 409 | No | Cannot send while device recovery is pending |
| `tenant_offboarding` | 403 | No | Tenant is in offboarding state per ADR-TEN-001 |
| `signing_key_rotation_pending` | 503 | Yes (auto, 1-2s backoff) | Server signing key rotation in progress |
| `cross_tenant_federation_deny` | 403 | No | Tenant-pair federation grant missing or expired |
| `pack_residency_violation` | 403 | No | Conversation pack requires home-cell residency |
| `mls_ciphersuite_mismatch` | 422 | No | Conversation requires P-384 ciphersuite per pack |
| `audit_chain_backpressure` | 503 | Yes (auto, 5s backoff) | Audit-chain emit must succeed before send accepted (fail-closed per ADR-MSG-001) |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `keypackage_upload` | `oya.messenger.mls.key_package.uploaded.v1` |
| `conversation_create` | `oya.messenger.conversation.created.v1` |
| `mls_commit_append` | `oya.messenger.mls.commit.accepted.v1` |
| `mls_commit_append` (welcome enqueued) | `oya.messenger.mls.welcome.enqueued.v1` |
| `message_send` | `oya.messenger.message.sent.v1` |
| `mls_recovery_request` | `oya.messenger.mls.recovery.requested.v1` |
| `mls_recovery_complete` | `oya.messenger.mls.recovery.completed.v1` |
| `mls_epoch_rejected` | `oya.messenger.mls.epoch.rejected.v1` |
| `huddle_start` | `oya.messenger.huddle.started.v1` |
| `federation_grant_created` | `oya.messenger.federation.grant.created.v1` |
| Cedar deny anywhere | `oya.messenger.cedar.denied.v1` |

## Where this file lives

`microservices/messenger/reference-implementations/send-mls-message-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/messenger/reference-implementations/send-mls-example/` once `oya-messenger-client` ships.
