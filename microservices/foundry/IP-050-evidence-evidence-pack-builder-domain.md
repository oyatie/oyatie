---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-005-evidence-pack-builder-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence
acceptance_lanes: [cargo-clippy, cargo-doc, lean-layer-correctness, property-based-tests]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: Evidence-pack-builder domain crate

## Intent

`oya-foundry-evidence-evidence-pack-builder-domain`: pack-schema construction logic + invariants + framework-profile builders. Pure functions. Layer = `domain`; imports own-BC `kernel` only.

## ChangeSet boundary

Single Rust crate. Pure compute (no I/O).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/Cargo.toml` | create | edition=2024; depends on own kernel only |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/lib.rs` | create | re-export |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/assembly/pack_envelope.rs` | create | `build_pack_envelope(invocation, eval, guardrails, supervisor) -> EvidencePack` |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/assembly/pack_payload_sha.rs` | create | `compute_pack_payload_sha(envelope) -> Sha256` — canonical CBOR encoding (deterministic) |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/assembly/partial_pack.rs` | create | partial-pack assembly with `missing_sources` |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/invariants/single_bind.rs` | create | EPI-02 invariant: pack assembled exactly once per `(invocation_id, attempt_no)` |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/invariants/eval_temporal_correctness.rs` | create | EPI-07 invariant: eval verdict current at invocation_ts |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/invariants/authority_cohesion.rs` | create | EPI-08 invariant: autonomy-tier decision is single-author from foundry-supervisor |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/eu_ai_act.rs` | create | EU AI Act profile field selector |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/hipaa.rs` | create | HIPAA profile |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/gdpr.rs` | create | GDPR profile |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/kr_pipa.rs` | create | KR PIPA profile |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/soc2.rs` | create | SOC 2 profile |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/iso_27001.rs` | create | ISO 27001 profile |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/tests/pack_sha_determinism.rs` | create | property: two assemblies of same inputs produce same sha |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/tests/eval_temporal_correctness.rs` | create | property: verdict applied is current at ts |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/tests/framework_profile_field_completeness.rs` | create | every profile produces all required fields for golden inputs |
| `Cargo.toml` (workspace) | edit | register |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-evidence-pack-builder-domain
cargo nextest run -p oya-foundry-evidence-evidence-pack-builder-domain
cargo clippy -p oya-foundry-evidence-evidence-pack-builder-domain -- -D warnings
cargo doc -p oya-foundry-evidence-evidence-pack-builder-domain --no-deps
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice foundry-evidence
```

## Halt Conditions

- Pack-payload-sha non-determinism detected — block; CBOR encoding must canonicalise.
- Any framework profile fails field-completeness test against golden — block; profile is the regulator-export contract.
- Authority-cohesion test fails (autonomy-tier sourced from anywhere other than foundry-supervisor) — block.

## Next IP

[`IP-006-evidence-pack-builder-usecase-and-adapters.md`](IP-006-evidence-pack-builder-usecase-and-adapters.md)

## References

- `microservices/foundry/policy/evidence-pack-integrity.md` EPI-02, EPI-03, EPI-07, EPI-08.
- ADR-0024 (eval-evidence integration).
