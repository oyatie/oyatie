---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-006-adapter-anthropic-api
status: pending
execution_unit: ChangeSet
owner: axis-foundry + ops-security (2-person rule per CI-INV-09)
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, credential-isolation, layer-correctness]
---

# IP-006: oya-foundry-providers-adapter-anthropic-api

## Intent

Anthropic Claude API HTTP transport. Resolves credentials via OpenBao bridge, builds `messages` API request, sends via mTLS HTTPS, captures response, computes BLAKE3 hashes, signs Ed25519 envelope, emits `ProviderInvoked` event.

## ChangeSet boundary

New crate `microservices/foundry/src/crates/oya-foundry-providers-adapter-anthropic-api/`. Implements `ProviderInvoker` port from kernel.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create — `reqwest`, `blake3`, `ed25519-dalek`, `zeroize`, `tokio`, kernel + adapter-openbao deps |
| `.../src/lib.rs` | create |
| `.../src/transport.rs` | create — HTTPS client w/ pinned vendor CA |
| `.../src/request_builder.rs` | create — kernel → Anthropic `messages` JSON |
| `.../src/response_parser.rs` | create — Anthropic response → kernel canonical shape |
| `.../src/envelope.rs` | create — BLAKE3 + Ed25519 |
| `.../src/response_validator.rs` | create — shape conformance check (T-03) |

## Code Shape

```rust
pub struct AnthropicApiAdapter<C>
where C: CredentialResolver<Credential = ResolvedCredential> {
    pub client: reqwest::Client,    // pinned vendor CA + mTLS-from-pod identity
    pub credential_resolver: C,
    pub signing_key: ed25519_dalek::SigningKey,
    pub event_emitter: EventEmitter,
}

#[async_trait]
impl<C> ProviderInvoker for AnthropicApiAdapter<C>
where C: CredentialResolver<Credential = ResolvedCredential> + Send + Sync {
    type Request = ProviderInvokeRequest;
    type Response = ProviderInvokeResponse;
    async fn invoke(&self, req: ProviderInvokeRequest) -> Result<ProviderInvokeResponse, RouterError> {
        // CI-INV-04: just-in-time credential resolution; lifetime ≤ one HTTP call
        let credential_ref = req.tenant_provider_config.credential_ref_for(Vendor::Anthropic);
        let credential = self.credential_resolver.resolve(&credential_ref, &caller_ctx(&req)).await?;

        let body = request_builder::build_anthropic_body(&req)?;
        let request_hash = blake3_hex(&body);

        // credential.with_credential is a closure-based accessor (CI-INV-02)
        let response = credential.with_credential(|cred_bytes| async {
            self.client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", cred_bytes)
                .header("anthropic-version", "2023-06-01")
                .body(body.clone())
                .send().await
        }).await?;
        // credential is dropped here; zeroize fires.

        let response_bytes = response.bytes().await?;
        let response_hash = blake3_hex(&response_bytes);
        let parsed = response_parser::parse(&response_bytes)?;
        response_validator::check_shape(&parsed)?;     // T-03

        let envelope_sig = envelope::sign_envelope(
            &self.signing_key, &request_hash, &response_hash, &metadata(&req),
        );
        let evidence_ref = self.event_emitter.emit_provider_invoked(/*...*/).await?;

        Ok(ProviderInvokeResponse {
            request_hash, response_hash, envelope_signature: envelope_sig,
            evidence_ref, response: parsed, ..
        })
    }
}
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_request_builder_matches_anthropic_messages_schema` | spec |
| `test_response_parser_canonical_shape` | normalisation |
| `test_envelope_sign_verify_roundtrip` | crypto |
| `tests/integration/anthropic_api_no_credential_leak.rs` | CI-INV-03 (zero credential bytes in any log/span/event) |
| `tests/integration/anthropic_api_response_shape_anomaly_quarantines` | T-03 |
| `test_pinned_cert_rejects_unknown_ca` | T-01 + T-03 |
| `test_credential_drops_after_call` | CI-INV-04 (zeroize) |

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate credential-isolation --crate oya-foundry-providers-adapter-anthropic-api
```

## Next IP

[`IP-007-adapter-anthropic-subscription.md`](IP-007-adapter-anthropic-subscription.md)
