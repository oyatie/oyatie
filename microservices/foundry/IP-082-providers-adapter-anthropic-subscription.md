---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-007-adapter-anthropic-subscription
status: pending
execution_unit: ChangeSet
owner: axis-foundry + ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, credential-isolation, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: oya-foundry-providers-adapter-anthropic-subscription

## Intent

Anthropic Claude Pro/Max subscription channel transport. Uses session-cookie credentials resolved from OpenBao (stored as opaque blob per CI-INV-06). FRAGILE channel — extra response-shape conformance + adapter-quarantine on anomaly.

## ChangeSet boundary

New crate `microservices/foundry/src/crates/oya-foundry-providers-adapter-anthropic-subscription/`. Implements `ProviderInvoker`.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/transport.rs` | create — claude.ai subscription channel client |
| `.../src/cookie_handler.rs` | create — load cookie from OpenBao opaque blob; never persist locally |
| `.../src/anomaly_detector.rs` | create — quarantine adapter on shape change (T-06) |

## Constraints (per `policy/credential-isolation.md` CI-INV-06)

- Subscription cookies are stored in OpenBao as opaque blobs.
- Adapter NEVER writes cookies to local disk, never logs them, never includes them in error messages.
- `oya-check-no-cookie-persistence` sub-lane sweep: BLOCKER on any local cookie persistence pattern.

## Code Shape

```rust
pub struct AnthropicSubscriptionAdapter<C>
where C: CredentialResolver<Credential = ResolvedCredential> {
    pub client: reqwest::Client,
    pub credential_resolver: C,
    pub signing_key: ed25519_dalek::SigningKey,
    pub event_emitter: EventEmitter,
    pub anomaly_detector: AnomalyDetector,
}

#[async_trait]
impl<C> ProviderInvoker for AnthropicSubscriptionAdapter<C> { /* ... */ }
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_request_builder_matches_subscription_channel` | spec |
| `tests/integration/anthropic_sub_no_credential_leak.rs` | CI-INV-03 |
| `tests/integration/anthropic_sub_shape_anomaly_quarantines` | T-06 |
| `test_cookie_never_persisted` | CI-INV-06 (filesystem + redis sweep) |
| `test_credential_drops_after_call` | CI-INV-04 |

## Acceptance Gates

Standard + `credential-isolation` lane + `no-cookie-persistence` sub-lane.

## Next IP

[`IP-008-adapter-openai-api.md`](IP-008-adapter-openai-api.md)
