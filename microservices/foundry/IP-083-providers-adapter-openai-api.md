---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-008-adapter-openai-api
status: pending
execution_unit: ChangeSet
owner: axis-foundry + ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, credential-isolation, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: oya-foundry-providers-adapter-openai-api

## Intent

OpenAI API HTTP transport (`api.openai.com/v1/chat/completions` + responses API). Resolves credentials via OpenBao bridge, builds OpenAI request, sends via mTLS HTTPS, captures + signs envelope.

## File Targets

Same shape as IP-006 (anthropic-api), with OpenAI-specific request/response shape conversion.

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/transport.rs` | create — `api.openai.com` HTTPS client w/ pinned vendor CA |
| `.../src/request_builder.rs` | create — kernel → OpenAI chat-completions JSON |
| `.../src/response_parser.rs` | create — OpenAI response → kernel canonical |
| `.../src/envelope.rs` | create — BLAKE3 + Ed25519 |
| `.../src/response_validator.rs` | create — shape conformance |

## Test Plan

| Test | Verifies |
|---|---|
| `test_request_builder_matches_openai_chat_completions_schema` | spec |
| `test_response_parser_canonical_shape` | normalisation |
| `tests/integration/openai_api_no_credential_leak.rs` | CI-INV-03 |
| `tests/integration/openai_api_response_shape_anomaly_quarantines` | T-03 |
| `test_pinned_cert_rejects_unknown_ca` | T-01 |
| `test_credential_drops_after_call` | CI-INV-04 |
| `test_pack_us_healthcare_baa_required` | residency invariant (denies if no BAA flag) |

## Acceptance Gates

Standard + `credential-isolation` lane.

## Next IP

[`IP-009-adapter-openai-subscription.md`](IP-009-adapter-openai-subscription.md)
