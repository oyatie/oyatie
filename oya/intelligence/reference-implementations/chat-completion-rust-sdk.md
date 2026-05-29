---
doc_class: ReferenceImplementation
microservice: intelligence
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — RAG-augmented chat completion via the intelligence Rust SDK

A runnable example that:

1. Authenticates as a tenant ai_admin principal.
2. Issues a RAG-augmented chat completion.
3. Inspects the retrieved chunks + the response with citations.
4. Verifies the watermark.
5. Verifies the audit-chain emission.

## Cargo.toml

```toml
[package]
name = "intelligence-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-intelligence-client = { path = "../../../../crates/oya-intelligence-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

## src/main.rs

```rust
use anyhow::Result;
use oya_intelligence_client::{
    IntelligenceClient, IntelligenceClientConfig,
    RequestCreate, TaskClass, RAGOptions, ResponseConstraints,
};
use oya_audit_chain_client::AuditChainClient;
use oya_cedar_client::CedarPrincipal;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Construct the client bound to an ai_admin Cedar principal.
    let principal = CedarPrincipal::from_env("INTELLIGENCE_AI_ADMIN_JWT")?;
    let client = IntelligenceClient::connect(IntelligenceClientConfig {
        cell_endpoint: std::env::var("INTELLIGENCE_CELL_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal: principal.clone(),
        request_timeout: std::time::Duration::from_secs(60),
        max_retries: 2,
    }).await?;

    // 2. Issue a RAG-augmented chat completion.
    let request = RequestCreate::builder()
        .task(TaskClass::Chat)
        .prompt("How do I configure WPA3 + WPA2 mixed mode on the ACME RT-7000?")
        .rag(RAGOptions {
            knowledge_base_id: "acme-product-docs".into(),
            top_k: 5,
            min_similarity: 0.65,
            citation_format: "inline_with_brackets".into(),
        })
        .response_constraints(ResponseConstraints {
            max_tokens: 1500,
            temperature: 0.7,
            citations_required: true,
        })
        .build()?;

    let response = client.request_create(&request).await?;

    println!("Request ID: {}", response.request_id);
    println!("Provider used: {}", response.provider_used);
    println!(
        "Latency: {} ms (RAG retrieval: {} ms; completion: {} ms)",
        response.total_latency_ms, response.rag_retrieval_ms, response.completion_latency_ms
    );
    println!("High-risk class: {:?}", response.high_risk_class);
    println!("Cedar decision: {}", response.cedar_decision);
    println!();
    println!("Retrieved chunks:");
    for (i, chunk) in response.rag_retrieved_chunks.iter().enumerate() {
        println!(
            "  [Source {}] {} (similarity={:.3})",
            i + 1, chunk.source_doc, chunk.similarity
        );
        println!("    Snippet: {}", chunk.snippet.chars().take(120).collect::<String>());
    }
    println!();
    println!("Response:");
    println!("{}", response.response);
    println!();

    // 3. Verify the watermark.
    let detect = client.watermark_detect(&response.response).await?;
    println!(
        "Watermark detected: confidence={:.4}, watermark_id={:?}",
        detect.confidence, detect.watermark_id
    );

    // 4. Verify the audit-chain emission.
    let audit_client = AuditChainClient::connect_for_tenant("acme-corp", principal).await?;
    let events = audit_client.query()
        .event_class_prefix("intelligence.")
        .since(chrono::Utc::now() - chrono::Duration::minutes(5))
        .request_id_filter(&response.request_id)
        .execute()
        .await?;

    println!("Audit events for this request:");
    for event in &events {
        println!("  {} — {}", event.event_class, event.event_id);
    }

    let verify = audit_client.verify_chain_since(
        chrono::Utc::now() - chrono::Duration::minutes(5),
        chrono::Utc::now()
    ).await?;
    println!(
        "Audit chain verification: {} events, batches: {}, signature_gaps: {}",
        verify.event_count, verify.batch_count, verify.signature_gaps
    );

    Ok(())
}
```

## Expected output (against a paid tenant_class cell with an ingested KB)

```
Request ID: req_01HZX9K3M2P4QR7S8T9V0W1X2Y
Provider used: anthropic-claude-3-5-sonnet-20251022
Latency: 2354 ms (RAG retrieval: 42 ms; completion: 2304 ms)
High-risk class: None
Cedar decision: allow

Retrieved chunks:
  [Source 1] rt-7000-security-guide.md (similarity=0.892)
    Snippet: WPA3 + WPA2 mixed mode allows older clients to connect via WPA2 PSK while WPA3 clients use SAE...
  [Source 2] rt-7000-wifi-config.md (similarity=0.876)
    Snippet: Access the router admin panel at 192.168.1.1 with default credentials admin/admin (change on first...
  [Source 3] rt-7000-troubleshooting.md (similarity=0.812)
    Snippet: If mixed mode causes connection issues, verify that all clients support WPA3 SAE encryption...
  [Source 4] wifi-security-best-practices.md (similarity=0.784)
    Snippet: Mixed-mode security has known dictionary-attack risk; recommend WPA3-only when all clients support...
  [Source 5] rt-7000-firmware-changelog.md (similarity=0.762)
    Snippet: Firmware v3.4 (2026-04-12) introduced full WPA3-Personal Only mode...

Response:
To configure WPA3 + WPA2 mixed mode on the ACME RT-7000:

1. Access the admin panel at 192.168.1.1 [Source 2].
2. Navigate to Wi-Fi → Security Settings.
3. Set Encryption Mode to 'WPA3-Personal + WPA2-Personal Transition Mode' [Source 1].
4. Save and reboot the router.

The router will then accept WPA3-capable clients (using SAE) and legacy WPA2 clients (using PSK) on the same SSID.

Security implication: WPA2 PSK is vulnerable to offline dictionary attacks if the passphrase is weak [Source 4]. Mixed mode means an attacker can target the WPA2 PSK independently of the WPA3 SAE. For environments where all clients support WPA3, use 'WPA3-Personal Only' mode for forward secrecy [Source 1, Source 5].

Watermark detected: confidence=0.9982, watermark_id=Some("wm_01HZX9K3...")
Audit events for this request:
  intelligence.request.submitted — evt_01HZX9K3...
  intelligence.prompt_fence.applied — evt_01HZX9K4...
  intelligence.rag.retrieved — evt_01HZX9K5...
  intelligence.high_risk_task.classified — evt_01HZX9K6...
  intelligence.cedar.evaluated — evt_01HZX9K7...
  intelligence.response.emitted — evt_01HZX9K8...
  intelligence.watermark.applied — evt_01HZX9K9...
Audit chain verification: 7 events, batches: 1, signature_gaps: 0
```

## HTTP alternative (curl)

```sh
# Submit RAG-augmented request
curl -X POST https://intelligence.prod-syd-1.oyatie.local/v1/requests \
    -H "Authorization: Bearer $INTELLIGENCE_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "task": "chat",
        "prompt": "How do I configure WPA3 + WPA2 mixed mode on the ACME RT-7000?",
        "rag": {
            "knowledge_base_id": "acme-product-docs",
            "top_k": 5,
            "min_similarity": 0.65,
            "citation_format": "inline_with_brackets"
        },
        "response_constraints": {
            "max_tokens": 1500,
            "temperature": 0.7,
            "citations_required": true
        }
    }'

# Watermark detect
curl -X POST https://intelligence.prod-syd-1.oyatie.local/v1/watermarks/detect \
    -H "Authorization: Bearer $INTELLIGENCE_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d "{\"text\": $(jq -Rs . < ./response.txt)}"
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Principal lacks `intelligence::request::create` |
| `high_risk_task_refused` | 403 | No | Task is Annex III high-risk; tenant must file Article 43 attestation |
| `prompt_fence_blocked` | 403 | No | Prompt matched a blocked phrase or rule |
| `pii_redaction_required` | 422 | No | Prompt contains PII that must be redacted client-side |
| `provider_outage_fallback_in_progress` | 503 | Yes (auto) | Primary provider down; routing to fallback; transparent retry |
| `provider_rate_limit` | 429 | Yes (auto, backoff) | Provider rate-limited; SDK exponential backoff |
| `quota_exceeded` | 429 | Yes (auto, backoff) | Tenant burnt per-tier daily quota; backoff |
| `kb_not_found` | 404 | No | Referenced knowledge base doesn't exist |
| `rag_no_chunks_above_similarity` | 422 | No | No chunks above the min_similarity threshold; lower threshold or expand corpus |
| `model_oom` | 503 | Yes (auto, fallback) | Self-hosted model OOM; fall back to passthrough |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `request_create` (entry) | `intelligence.request.submitted` |
| `prompt_fence` (applied) | `intelligence.prompt_fence.applied` |
| `prompt_fence` (blocked) | `intelligence.prompt_fence.blocked` |
| `pii_redaction` (applied) | `intelligence.pii.redacted` |
| `rag_retrieval` | `intelligence.rag.retrieved` |
| `high_risk_classifier` | `intelligence.high_risk_task.classified` or `intelligence.high_risk_task.refused` |
| `cedar_evaluation` | `intelligence.cedar.evaluated` |
| `provider_routing` (selected) | `intelligence.psp.routed` |
| `response` (success) | `intelligence.response.emitted` |
| `watermark` (applied) | `intelligence.watermark.applied` |

## Where this file lives

`microservices/intelligence/reference-implementations/chat-completion-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/intelligence/reference-implementations/chat-completion-example/` once `oya-intelligence-client` is published.
