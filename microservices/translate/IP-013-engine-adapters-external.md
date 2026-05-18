---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-translate-platform
impl_plan_id: IP-013-engine-adapters-external
status: pending
execution_unit: ChangeSet
owner: axis-translate + ops-security (2-person rule per CI-INV-09)
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, layer-correctness, credential-isolation]
---

# IP-013: Engine adapters — external vendors (Anthropic / OpenAI / Google / DeepL)

## Intent

External vendor adapter crates that route through `foundry-providers` µservice (per ADR-0126 + foundry-providers PRD — no direct vendor calls from translate). Each adapter implements `TranslateInvoker` over a specific vendor's translation surface.

## ChangeSet boundary

Four new Rust crates:

- `oya-translate-adapter-anthropic` — Claude messages API for MT (system-prompt → translate; long-context premium tier)
- `oya-translate-adapter-openai` — GPT messages API for MT (alternate LLM-class)
- `oya-translate-adapter-google-translate` — Cloud Translation API v3 + AutoML
- `oya-translate-adapter-deepl` — DeepL Pro API

Each crate is structurally identical:

| Path | Action |
|---|---|
| `Cargo.toml` | create — depends on kernel + `foundry-providers-router-sdk` |
| `src/lib.rs` | create |
| `src/transport.rs` | create — wraps foundry-providers `ProviderInvoker` |
| `src/request_builder.rs` | create — translation-request → vendor request body |
| `src/response_parser.rs` | create — vendor response → kernel canonical shape |
| `src/glossary.rs` | create — per-vendor glossary parameter mapping (where supported) |
| `src/envelope.rs` | create — BLAKE3 + Ed25519 |
| `src/response_validator.rs` | create — shape conformance (T-03) |

## Per-Vendor Notes

### Anthropic (LLM-class)

- Used for content classes requiring frontier capability (long-context, contextual nuance, legal/medical).
- Glossary passed as system-prompt addendum.
- ZDR negotiation required for pack-us-healthcare + pack-kr (per `policy/data-residency.md`).

### OpenAI

- Alternate LLM-class.
- Glossary via system prompt.
- Region: requires SCC for pack-eu.

### Google Cloud Translation API + AutoML

- Native NMT for short segments (high volume).
- Native `glossaryConfig` parameter — no prompt-side workaround needed.
- AutoML custom model id passed when tenant has a fine-tuned model.
- Region: per Google Cloud region (per pack).

### DeepL Pro

- Premium quality on EU pairs.
- Native `glossary_id` parameter.
- DE-EU resident; cross-border permitted only with tenant PIPA Art. 28 consent for pack-kr.

## Credential Isolation

All adapters resolve credentials via `foundry-providers`' OpenBao bridge (per foundry-providers credential-isolation.md). translate µservice has no direct credential-bytes path; CI lane `oya-translate-credential-isolation` runs a regex sweep against the test fixture set and asserts zero occurrences.

## Test Plan

Per adapter:

| Test | Verifies |
|---|---|
| `test_request_builder_matches_vendor_schema` | spec |
| `test_response_parser_canonical_shape` | normalization |
| `test_envelope_sign_verify_roundtrip` | crypto |
| `tests/integration/<vendor>_no_credential_leak.rs` | regex sweep |
| `tests/integration/<vendor>_response_shape_anomaly_quarantines` | T-03 |
| `tests/integration/<vendor>_glossary_honored` | per-vendor glossary path |
| `tests/integration/<vendor>_pinned_cert_rejects_unknown_ca` | mTLS + pinned CA |
| `tests/integration/<vendor>_residency_constraint_honored.rs` | only configured region called |

## Halt Conditions

- Any adapter makes a direct HTTP call bypassing foundry-providers.
- Any credential byte appears in adapter src or logs (regex sweep > 0).
- Vendor response shape silently accepted as canonical (T-03).
- Per-vendor pinned CA bypassed.

## Next IP

[`IP-014-router-rest-worker-app.md`](IP-014-router-rest-worker-app.md)
