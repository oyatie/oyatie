---
doc_class: ReferenceImplementation
microservice: audit-chain
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Emit + verify an audit event via the audit-chain Rust SDK

A runnable example that:

1. Emits an audit event via the `oya-audit-emission-adapter` from a downstream µservice's perspective.
2. Queries the chain to retrieve the event.
3. Generates a Merkle proof for the event.
4. Externally verifies the proof using only `openssl` + `sha256sum` (no oyatie tooling).

## Cargo.toml

```toml
[package]
name = "audit-chain-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-audit-emission-adapter = { path = "../../../../crates/oya-audit-emission-adapter" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
ed25519-dalek = "2.1"
```

## src/main.rs

```rust
use anyhow::{Context, Result};
use chrono::Utc;
use oya_audit_chain_client::{AuditChainClient, AuditChainClientConfig, VerifyOptions};
use oya_audit_emission_adapter::{AuditEvent, AuditEmitter};
use oya_cedar_client::CedarPrincipal;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct WorkflowStepCompletedPayload {
    workflow_id: String,
    step_id: String,
    step_name: String,
    duration_ms: u64,
    output_summary: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Construct the emitter (this is what a downstream µservice like workflow-engine uses).
    let principal = CedarPrincipal::from_env("WORKFLOW_ENGINE_AUDIT_PRINCIPAL_JWT")?;
    let emitter = AuditEmitter::connect(
        std::env::var("AUDIT_CHAIN_INGEST_ENDPOINT")?,
        principal.clone(),
    )
    .await?;

    // 2. Emit a workflow-step-completed event.
    let payload = WorkflowStepCompletedPayload {
        workflow_id: "wf-001".into(),
        step_id: "step-12".into(),
        step_name: "tax_calculation".into(),
        duration_ms: 184,
        output_summary: "tax_amount_minor_units=12450".into(),
    };
    let event = AuditEvent::builder()
        .event_class("workflow.step.completed")
        .tenant_ids(vec!["acme-corp".to_string()])
        .principal_id("u-workflow-engine-runner-42")
        .payload_json(&payload)?
        .build()?;
    let receipt = emitter.emit(event).await?;
    println!("Emitted event_id={}, chain_seq={}", receipt.event_id, receipt.chain_seq);

    // 3. The emitter returns a receipt with the event_id. Wait briefly for the next seal interval,
    //    then query the chain to confirm the event is sealed.
    tokio::time::sleep(std::time::Duration::from_secs(6)).await;

    let chain_client = AuditChainClient::connect(AuditChainClientConfig {
        cell_endpoint: std::env::var("AUDIT_CHAIN_QUERY_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal,
        request_timeout: std::time::Duration::from_secs(30),
    })
    .await?;

    let event_row = chain_client
        .get_event_by_id(&receipt.event_id)
        .await?
        .context("event not found in chain — seal interval not elapsed?")?;

    println!(
        "Retrieved event: class={}, sealed_in_batch={}, signed_with_key={}",
        event_row.event_class,
        event_row.sealed_in_batch_id,
        event_row.signing_key_id
    );

    // 4. Generate a Merkle proof for the event.
    let proof = chain_client.generate_proof(&receipt.event_id).await?;
    let proof_json = serde_json::to_string_pretty(&proof)?;
    std::fs::write("./proof.json", &proof_json)?;
    println!("Wrote ./proof.json");

    // 5. Fetch the signing-key public component (this is the public material an external verifier needs).
    let signing_key_pub = chain_client
        .get_signing_key_public(&event_row.signing_key_id)
        .await?;
    std::fs::write("./signing_key.pub", signing_key_pub.ed25519_public_pem.as_bytes())?;
    println!("Wrote ./signing_key.pub");

    // 6. Verify the chain itself (returns OK if continuity + signature + Merkle root all check).
    let verify = chain_client
        .verify_chain(VerifyOptions {
            tenant_id: "acme-corp".into(),
            since: Utc::now() - chrono::Duration::minutes(10),
            until: Utc::now(),
            event_class_filter: Some(vec!["workflow.step.completed".into()]),
        })
        .await?;
    println!(
        "Chain verification: {} events, {} batches, signature_gaps={}, prev_hash_breaks={}",
        verify.event_count, verify.batch_count, verify.signature_gaps, verify.prev_hash_breaks
    );

    Ok(())
}
```

## Expected output (against a demo_trial tenant_class cell with no other emission load)

```
Emitted event_id=01HZX9K3M2P4QR7S8T9V0W1X2Y, chain_seq=4218
Retrieved event: class=workflow.step.completed, sealed_in_batch=01HZX9K3M3..., signed_with_key=audit-chain-demo_trial-drill-syd-1-2026-05-20
Wrote ./proof.json
Wrote ./signing_key.pub
Chain verification: 1 events, 1 batches, signature_gaps=0, prev_hash_breaks=0
```

## Standalone Merkle proof verification (NO oyatie tooling)

The `proof.json` shape:

```json
{
  "event_id": "01HZX9K3M2P4QR7S8T9V0W1X2Y",
  "event_payload_sha256": "sha256:7c4a2b8e1f...",
  "merkle_leaf_index": 1247,
  "merkle_path": [
    {"position": "right", "hash": "sha256:abc123..."},
    {"position": "left",  "hash": "sha256:def456..."},
    {"position": "right", "hash": "sha256:111aaa..."},
    {"position": "left",  "hash": "sha256:222bbb..."}
  ],
  "batch_root": "sha256:fedcba987654...",
  "batch_id": "01HZX9K3M3...",
  "batch_seal_timestamp": "2026-05-20T14:32:23.412Z",
  "signature_ed25519": "ed25519:9f8e7d6c5b4a39281706...",
  "signing_key_id": "audit-chain-demo_trial-drill-syd-1-2026-05-20",
  "signing_algorithm": "Ed25519",
  "hash_algorithm": "SHA-256"
}
```

A standalone Bash verifier (the same one shipped in regulator-export bundles):

```bash
#!/usr/bin/env bash
set -euo pipefail
PROOF=${1:-proof.json}
PUBKEY=${2:-signing_key.pub}
EVENT_PAYLOAD=${3:-event_payload.bin}

# 1. Recompute the leaf hash from the event payload.
LEAF=$(sha256sum "$EVENT_PAYLOAD" | awk '{print $1}')
EXPECTED_LEAF=$(jq -r .event_payload_sha256 "$PROOF" | sed 's/sha256://')
[ "$LEAF" = "$EXPECTED_LEAF" ] || { echo "FAIL: leaf hash mismatch"; exit 1; }

# 2. Walk the Merkle path, recomputing the root.
ROOT=$LEAF
for hop in $(jq -c '.merkle_path[]' "$PROOF"); do
    POS=$(jq -r .position <<< "$hop")
    SIB=$(jq -r .hash <<< "$hop" | sed 's/sha256://')
    if [ "$POS" = "left" ]; then
        ROOT=$(printf '%s%s' "$SIB" "$ROOT" | xxd -r -p | sha256sum | awk '{print $1}')
    else
        ROOT=$(printf '%s%s' "$ROOT" "$SIB" | xxd -r -p | sha256sum | awk '{print $1}')
    fi
done

# 3. Compare against the claimed batch root.
EXPECTED_ROOT=$(jq -r .batch_root "$PROOF" | sed 's/sha256://')
[ "$ROOT" = "$EXPECTED_ROOT" ] || { echo "FAIL: Merkle root mismatch"; exit 1; }

# 4. Verify the Ed25519 signature on the batch root.
SIG=$(jq -r .signature_ed25519 "$PROOF" | sed 's/ed25519://')
echo -n "$EXPECTED_ROOT" | xxd -r -p > /tmp/root.bin
echo -n "$SIG" | xxd -r -p > /tmp/sig.bin
openssl pkeyutl -verify -pubin -inkey "$PUBKEY" -rawin -in /tmp/root.bin -sigfile /tmp/sig.bin
echo "PASS: proof verified end-to-end."
```

Run:

```sh
chmod +x verify-proof.sh
./verify-proof.sh proof.json signing_key.pub event_payload.bin
# Output: PASS: proof verified end-to-end.
```

This script depends ONLY on `sha256sum`, `openssl ≥ 3.0`, `jq`, `xxd`. No Rust, no oyatie tooling, no network.

## HTTP alternative (direct against the audit-chain query gateway)

Until the SDK lands fully, the same flow can be issued via the HTTP gateway:

```sh
# Emit
curl -X POST https://audit-chain-pack-kr.oyatie.com/api/v1/emit \
    -H "Authorization: Bearer $AUDIT_EMIT_JWT" \
    -H "X-Scope-OrgID: tenant:acmecorp00000001" \
    -H "Idempotency-Key: 01HZX9K3M2P4QR7S8T9V0W1X2Y" \
    -H "Content-Type: application/json" \
    -d '{
        "event_class": "workflow.step.completed",
        "tenant_ids": ["acme-corp"],
        "principal_id": "u-workflow-engine-runner-42",
        "payload": {"workflow_id": "wf-001", "step_id": "step-12", "duration_ms": 184}
    }'
# Returns: {"event_id":"01HZX9K3M2P4QR7S8T9V0W1X2Y","period_id":"2026-05-20T14:32:17Z","pack":"pack-kr","sealed":false}

# Query
curl -X POST https://audit-chain-pack-kr.oyatie.com/api/v1/query \
    -H "Authorization: Bearer $AUDIT_QUERY_JWT" \
    -H "X-Scope-OrgID: tenant:acmecorp00000001" \
    -H "Content-Type: application/json" \
    -d '{"event_ids":["01HZX9K3M2P4QR7S8T9V0W1X2Y"],"limit":1}'

# Prove
curl -G https://audit-chain-pack-kr.oyatie.com/api/v1/events/01HZX9K3M2P4QR7S8T9V0W1X2Y/proof \
    -H "Authorization: Bearer $AUDIT_QUERY_JWT" \
    -H "X-Scope-OrgID: tenant:acmecorp00000001" \
    -o proof.json

# Get signing-key public
curl -G https://audit-chain-pack-kr.oyatie.com/api/v1/keys/pack-kr/epoch-2026-05-20 \
    -H "Authorization: Bearer $AUDIT_QUERY_JWT" \
    -H "X-Scope-OrgID: tenant:acmecorp00000001" \
    -o signing_key.pub
```

## Error handling — what to retry

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | The principal lacks the permission. Fix at IAM, not at runtime. |
| `seal_pending` | Yes (auto, 5 s backoff) | The event is in the chain head but not yet sealed; wait 5 s and re-query. |
| `cell_unavailable` | Yes (with circuit-breaker) | The cell is down; SDK fails after 3 retries; circuit-breaker opens for 30 s. |
| `signing_key_rotated` | No | The signing key referenced in the proof was rotated. Re-fetch the proof — the chain replays with the new key on the rotation event. |
| `chain_fork_detected` | No | Critical incident; emit `audit_chain.chain_fork_detected` event and page on-call. Should be impossible in normal operation. |
| `payload_too_large` | No | The event payload > 64 KiB; spill to SeaweedFS-S3 with hash-only in the leaf, then re-emit. |

## Where this file lives in the µservice

`microservices/audit-chain/reference-implementations/emit-and-verify-rust-sdk.md` (this file).

The runnable Cargo project lands at `microservices/audit-chain/reference-implementations/emit-verify-example/` once IP-014 ships the cross-µservice emission adapter to production. Until then, this file is the contract; CI's `audit-chain-reference-impl-compiles` lane runs a stubbed compile against the kernel crates.
