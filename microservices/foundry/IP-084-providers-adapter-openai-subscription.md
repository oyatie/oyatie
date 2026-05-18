---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-009-adapter-openai-subscription
status: pending
execution_unit: ChangeSet
owner: axis-foundry + ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, credential-isolation, layer-correctness]
---

# IP-009: oya-foundry-providers-adapter-openai-subscription

## Intent

ChatGPT Plus subscription channel transport (chatgpt.com). Session-cookie credentials from OpenBao; FRAGILE channel with adapter-quarantine on shape anomaly. Forbidden for PHI workloads (data-residency.md pack-us-healthcare row).

## File Targets

Same shape as IP-007.

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/transport.rs` | create — chatgpt.com subscription client |
| `.../src/cookie_handler.rs` | create — OpenBao-opaque-blob cookie load |
| `.../src/anomaly_detector.rs` | create |

## Constraints

- Forbidden for `data_class == "PHI"` requests (per `policy/provider-router-tenant-scope.cedar` forbid rule).
- Forbidden for pack-us-healthcare (per `policy/data-residency.md`).

## Test Plan

| Test | Verifies |
|---|---|
| `test_request_builder_matches_subscription_channel` | spec |
| `tests/integration/openai_sub_no_credential_leak.rs` | CI-INV-03 |
| `tests/integration/openai_sub_shape_anomaly_quarantines` | T-06 |
| `test_cookie_never_persisted` | CI-INV-06 |
| `test_phi_request_denied` | Cedar forbid + residency |

## Acceptance Gates

Standard + `credential-isolation` + `no-cookie-persistence` sub-lane.

## Next IP

[`IP-010-adapter-gemini-api.md`](IP-010-adapter-gemini-api.md)
