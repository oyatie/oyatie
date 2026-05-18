---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-012-adapter-in-house
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, credential-isolation, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: oya-foundry-providers-adapter-in-house

## Intent

Adapter for oyatie-trained in-house models served via vLLM/TGI co-located endpoint per ADR-0026. Mesh-internal mTLS (no public egress). Implements ProviderInvoker port with canonical kernel shape so the router treats in-house identically.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/transport.rs` | create — mesh-internal mTLS to in-house vLLM/TGI fleet |
| `.../src/request_builder.rs` | create — kernel → OpenAI-compatible vLLM JSON |
| `.../src/response_parser.rs` | create — vLLM response → kernel canonical |
| `.../src/envelope.rs` | create — BLAKE3 + Ed25519 (same as hosted adapters) |
| `.../src/endpoint_discovery.rs` | create — service-mesh discovery of in-house pool |
| `.../src/canary_cohort.rs` | create — per-tenant cohort weighting (1/10/50/100%) |

## Credentials

In-house adapter does NOT use vendor API keys; mesh mTLS establishes pod-to-pod identity. The CredentialResolver port returns a sentinel `ResolvedCredential::InHouse` value carrying only the SPIFFE peer-identity binding (no secret bytes).

## Test Plan

| Test | Verifies |
|---|---|
| `test_request_builder_matches_vllm_openai_compat_schema` | spec |
| `test_response_parser_canonical_shape` | normalisation |
| `tests/integration/in_house_no_external_egress.rs` | no external network connection during call |
| `tests/integration/in_house_canary_cohort_ramp` | cohort weighting |
| `tests/integration/in_house_burn_rate_demote` | T-04 (quality regression demote) |
| `test_envelope_signed_identically_to_hosted` | parity |

## Acceptance Gates

Standard + `credential-isolation` (sentinel-shaped only).

## Next IP

[`IP-013-adapter-openbao.md`](IP-013-adapter-openbao.md)
