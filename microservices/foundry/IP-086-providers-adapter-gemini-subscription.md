---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-011-adapter-gemini-subscription
status: pending
execution_unit: ChangeSet
owner: axis-foundry + ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, credential-isolation, layer-correctness]
---

# IP-011: oya-foundry-providers-adapter-gemini-subscription

## Intent

Gemini Advanced subscription channel transport (gemini.google.com). Session-cookie credentials from OpenBao; FRAGILE channel with adapter-quarantine on shape anomaly. Forbidden for PHI workloads.

## File Targets

Same shape as IP-007 + IP-009.

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/transport.rs` | create — gemini.google.com subscription client |
| `.../src/cookie_handler.rs` | create — OpenBao-opaque-blob cookie load |
| `.../src/anomaly_detector.rs` | create |

## Constraints

- Forbidden for `data_class == "PHI"`.
- Forbidden for pack-us-healthcare.

## Test Plan

| Test | Verifies |
|---|---|
| `test_request_builder_matches_subscription_channel` | spec |
| `tests/integration/gemini_sub_no_credential_leak.rs` | CI-INV-03 |
| `tests/integration/gemini_sub_shape_anomaly_quarantines` | T-06 |
| `test_cookie_never_persisted` | CI-INV-06 |
| `test_phi_request_denied` | Cedar forbid + residency |

## Acceptance Gates

Standard + `credential-isolation` + `no-cookie-persistence`.

## Next IP

[`IP-012-adapter-in-house.md`](IP-012-adapter-in-house.md)
