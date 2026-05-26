---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-006-autonomy-ceiling-gate-kernel-and-cedar-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, cedar-fragment-coverage, cedar-default-deny-enforced]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-foundry-guardrails-autonomy-ceiling-gate-kernel + adapter-cedar

## Intent

Two crates: `-kernel` (port traits + `AutonomyLevelClaim`, `EffectiveCeiling`, `TierViolation` entities; effective-ceiling computation per ADR-0022) + `-adapter-cedar` (Cedar v4 client + policy-bundle loader; in-process Cedar engine for sidecar pattern). The adapter is backend-qualified per ADR-0105 Amendment 3 (`*-adapter-cedar`) since Cedar v4 is the sanctioned backend per ADR-0140 (retired per ADR-0145).

## ChangeSet boundary

Two crates introduced in one IP because they are tightly coupled (kernel port resolves through Cedar adapter; alternate backends not sanctioned by ADR-0140).

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-guardrails-autonomy-ceiling-gate-kernel/Cargo.toml` | create |
| `.../-kernel/src/{lib.rs,entities.rs,ports.rs,errors.rs,ceiling.rs}` | create |
| `src/crates/oya-foundry-guardrails-autonomy-ceiling-gate-adapter-cedar/Cargo.toml` | create |
| `.../-adapter-cedar/src/{lib.rs,engine.rs,bundle_loader.rs}` | create |
| `Cargo.toml` (workspace) | update — both members |
| `catalog/{oya-foundry-guardrails-autonomy-ceiling-gate-kernel,oya-foundry-guardrails-autonomy-ceiling-gate-adapter-cedar}.yaml` | create |

## Crate Naming

```
NAME: oya-foundry-guardrails-autonomy-ceiling-gate-kernel
JUSTIFICATION: microservice=foundry-guardrails; bc=autonomy-ceiling-gate; layer=kernel

NAME: oya-foundry-guardrails-autonomy-ceiling-gate-adapter-cedar
JUSTIFICATION: microservice=foundry-guardrails; bc=autonomy-ceiling-gate; layer=adapter; backend=cedar per ADR-0105 §"Amendment 3" + ADR-0140 (Cedar canonical)
```

## Code Shape

```rust
// kernel/src/ceiling.rs (per ADR-0022 §"Effective-ceiling resolution")
pub struct AutonomyLevelInputs {
    pub tenant_configured: AutonomyLevel,
    pub capability_min_required: AutonomyLevel,
    pub vertical_pack_cap: AutonomyLevel,
    pub subject_class_cap: AutonomyLevel,
}

impl AutonomyLevelInputs {
    pub fn effective(&self) -> AutonomyLevel {
        AutonomyLevel::min_of(&[
            self.tenant_configured,
            self.capability_min_required,
            self.vertical_pack_cap,
            self.subject_class_cap,
        ])
    }
}

#[async_trait]
pub trait AutonomyLevelGate: Send + Sync + Sealed {
    async fn enforce(&self, claim: &AutonomyLevelClaim) -> Result<EffectiveCeiling, TierViolation>;
}

#[async_trait]
pub trait CedarEngineHandle: Send + Sync + Sealed {
    async fn evaluate(&self, request: &CedarRequest) -> Result<CedarDecision, KernelError>;
    fn bundle_sha(&self) -> Result<String, KernelError>;
}
```

```rust
// adapter-cedar/src/engine.rs
use cedar_policy::{Authorizer, Context, Entities, PolicySet, Request};

pub struct CedarEngine {
    authorizer: Authorizer,
    bundle: PolicySet,
    entities: Entities,
    bundle_sha: String,
}

#[async_trait]
impl CedarEngineHandle for CedarEngine {
    async fn evaluate(&self, request: &CedarRequest) -> Result<CedarDecision, KernelError> {
        let cedar_req: Request = request.into_cedar()?;
        let response = self.authorizer.is_authorized(
            &cedar_req,
            &self.bundle,
            &self.entities,
        );
        Ok(response.into())
    }
    fn bundle_sha(&self) -> Result<String, KernelError> {
        Ok(self.bundle_sha.clone())
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-autonomy-ceiling-gate-kernel --all-features
cargo check -p oya-foundry-guardrails-autonomy-ceiling-gate-adapter-cedar --all-features
cargo nextest run -p oya-foundry-guardrails-autonomy-ceiling-gate-kernel --all-features
cargo nextest run -p oya-foundry-guardrails-autonomy-ceiling-gate-adapter-cedar --all-features
cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice foundry-guardrails
cargo run -p oya-dev-cli -- gate validate cedar-default-deny-enforced
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_effective_ceiling_min_of_four_inputs` | math correctness per ADR-0022 |
| `test_tier_excess_refused` | claim above ceiling → TierViolation |
| `test_cedar_default_deny` | empty bundle → deny |
| `test_cedar_per_tenant_overlay_composition` | overlay permits stack with base deny |
| `integration_cedar_v4_round_trip` | real Cedar v4 PolicySet evaluation |

## Halt Conditions

- Any path bypasses effective-ceiling computation — refactor.
- Cedar bundle compiles without default-deny — refuse merge.

## Next IP

[`IP-007-content-safety-rule-engine-kernel-and-postgres-adapter.md`](IP-007-content-safety-rule-engine-kernel-and-postgres-adapter.md)

## References

- ADR-0022 §"Effective-ceiling resolution".
- ADR-0140 Cedar substrate.
- `policy/tenant-scope.cedar`, `policy/ci-scope.cedar`, `policy/auditor-scope.cedar`.
- Cedar v4 docs — `docs.cedarpolicy.com`.

## Wave 15 counterpart anchor

- Counterparts: AWS Bedrock Guardrails, OpenAI Moderation, Anthropic safety tooling, and NVIDIA NeMo Guardrails.
- Gap closure: this IP closes inline prompt, output, autonomy, jailbreak, and false-positive-budget enforcement before tenant-visible release.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
