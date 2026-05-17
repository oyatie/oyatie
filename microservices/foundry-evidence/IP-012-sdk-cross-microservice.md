---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-012-sdk-cross-microservice
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence + axis-developer-experience
acceptance_lanes: [cargo-doc, sdk-codegen-reproducible, sdk-no-silent-regression]
---

# IP-012: Cross-microservice SDK + reference integration

## Intent

Ship `oya-foundry-evidence-sdk` as the canonical client; wire reference integrations in foundry-runtime + foundry-guardrails + foundry-supervisor + foundry-eval. Per `sdk-plan.md`.

## ChangeSet boundary

1 SDK crate + 4 reference-integration patches.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-foundry-evidence-sdk/src/lib.rs` | create | top-level re-exports |
| `crates/oya-foundry-evidence-sdk/src/recorder.rs` | create | `CapabilityInvocationRecorderClient` |
| `crates/oya-foundry-evidence-sdk/src/query.rs` | create | `EvidenceQueryClient` (streaming) |
| `crates/oya-foundry-evidence-sdk/src/regulator_export.rs` | create | `RegulatorExportClient` |
| `crates/oya-foundry-evidence-sdk/src/spiffe.rs` | create | SPIFFE auto-load |
| `crates/oya-foundry-evidence-sdk/src/retry.rs` | create | retry policy per `sdk-plan.md` §"Retry semantics" |
| `crates/oya-foundry-evidence-sdk/src/observability.rs` | create | OpenTelemetry span + metric instrumentation |
| `crates/oya-foundry-evidence-sdk/tests/sdk_contract.rs` | create | drives against test-tenant fixture |
| `crates/oya-foundry-runtime-*` (existing) | edit | replace ad-hoc evidence emission with SDK call (reference integration) |
| `crates/oya-foundry-guardrails-*` (existing) | edit | replace ad-hoc emission with SDK call |
| `crates/oya-foundry-supervisor-*` (existing) | edit | replace ad-hoc emission with SDK call |
| `crates/oya-foundry-eval-*` (existing) | edit | publish eval-verdict events that aggregator consumes (Workflow-bus path; SDK used for direct verdict-at-invocation lookups during eval re-runs) |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-sdk
cargo doc -p oya-foundry-evidence-sdk --no-deps
cargo nextest run -p oya-foundry-evidence-sdk --test sdk_contract
cargo nextest run -p oya-foundry-runtime-* --test evidence_emission_via_sdk
oya gate validate sdk-codegen-reproducible --microservice foundry-evidence
oya gate validate sdk-no-silent-regression --microservice foundry-evidence
```

## Halt Conditions

- SDK public symbol removal without ADR + sunset — block.
- Reference integration still uses ad-hoc emission path — block.

## Next IP

[`IP-013-regulator-export-framework-profiles.md`](IP-013-regulator-export-framework-profiles.md)

## References

- `sdk-plan.md`.
- ADR-0131 (per-microservice layout; SDK layer).
