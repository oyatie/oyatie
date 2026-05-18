---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-010-adapter-gemini-api
status: pending
execution_unit: ChangeSet
owner: axis-foundry + ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, credential-isolation, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: oya-foundry-providers-adapter-gemini-api

## Intent

Gemini API HTTP transport (`generativelanguage.googleapis.com`). Resolves credentials via OpenBao bridge, builds Gemini request, sends via mTLS HTTPS, captures + signs envelope.

## File Targets

Same shape as IP-006 (anthropic-api).

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/transport.rs` | create — Gemini HTTPS client w/ pinned vendor CA |
| `.../src/request_builder.rs` | create — kernel → Gemini `generateContent` JSON |
| `.../src/response_parser.rs` | create — Gemini response → kernel canonical |
| `.../src/envelope.rs` | create |
| `.../src/response_validator.rs` | create |

## Test Plan

| Test | Verifies |
|---|---|
| `test_request_builder_matches_gemini_generate_content_schema` | spec |
| `test_response_parser_canonical_shape` | normalisation |
| `tests/integration/gemini_api_no_credential_leak.rs` | CI-INV-03 |
| `tests/integration/gemini_api_response_shape_anomaly_quarantines` | T-03 |
| `test_pinned_cert_rejects_unknown_ca` | T-01 |
| `test_credential_drops_after_call` | CI-INV-04 |

## Acceptance Gates

Standard + `credential-isolation` lane.

## Next IP

[`IP-011-adapter-gemini-subscription.md`](IP-011-adapter-gemini-subscription.md)
