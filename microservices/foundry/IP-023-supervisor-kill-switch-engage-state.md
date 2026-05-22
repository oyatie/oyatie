---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-008-kill-switch-engage-state
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, oya-check-kill-switch-2-person-rule]
depends_on: [IP-002, IP-003, IP-007]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-008: kill-switch state model + engage/disengage primitives

## Intent

Kernel + domain + usecase + adapter for kill-switch-circuit-breaker BC; Redis-backed state cache; CRD-authoritative source-of-truth; Ed25519 signature verification at engage; 2-person rule enforcement at type level for fleet-wide.

## Concrete File Targets

Crates at `microservices/foundry/src/crates/oya-foundry-supervisor-kill-switch-circuit-breaker-{kernel,domain,usecase,api,adapter}/`. rest + worker + sdk + app scheduled-for-distinct-tracked-work to IP-009 + IP-011 + IP-013 + IP-014.

## Key code

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait KillSwitchStateStore: Send + Sync + Sealed {
    async fn engage(&self, scope: KillSwitchScope, target: &str, reason: EngageReason, signatures: &[Ed25519Signature]) -> Result<(), KernelError>;
    async fn disengage(&self, scope: KillSwitchScope, target: &str, signatures: &[Ed25519Signature]) -> Result<(), KernelError>;
    async fn query(&self, scope: KillSwitchScope, target: &str) -> Result<Option<KillSwitch>, KernelError>;
}

// usecase/src/engage.rs
pub async fn engage_kill_switch(
    scope: KillSwitchScope,
    target: TargetId,
    reason: EngageReason,
    signatures: SignatureBundle,
    store: &dyn KillSwitchStateStore,
    publisher: &dyn SupervisionEventPublisher,
) -> Result<EngagedAt, KernelError> {
    // 1. Verify signature count: fleet=2, else 1
    // 2. Verify Ed25519 signatures against authorising principal pubkey
    // 3. Write Valkey state
    // 4. Write CRD (Operator picks up + propagates)
    // 5. Publish KillSwitchEngaged event Ed25519-signed
    // 6. Audit-chain seal
}
```

Two-person rule type-system enforcement:

```rust
// Type system prevents single-signature fleet engage.
pub struct FleetWide;
pub struct ScopeLocal;
pub fn engage_fleet_wide(
    reason: EngageReason,
    signature_1: Ed25519Signature,
    signature_2: Ed25519Signature,  // two args; compiler enforces both
) -> ...

pub fn engage_scope_local<S: ScopeKind>(
    scope: S,
    target: TargetId,
    reason: EngageReason,
    signature: Ed25519Signature,
) -> ...
```

## Acceptance Gates

```bash
cargo nextest run -p oya-foundry-supervisor-kill-switch-circuit-breaker-usecase --test two_person_rule_enforced
cargo run -p oya-dev-cli -- gate validate kill-switch-2-person-rule --microservice foundry-supervisor
```

## Halt Conditions

- Single-signature fleet-wide engage compiles.
- Engage skips signature verification.

## Next IP

[`IP-009-kill-switch-propagation.md`](IP-009-kill-switch-propagation.md)

## References

- PRD FR-04.
- ADR-0133 §"Hyperscaler safety-claim parity".
- `policy/tenant-scope.cedar` PERMIT 2 + FORBID fleet-scope.
- `runbooks/kill-switch-engage.md`.

## Wave 15 counterpart anchor

- Counterparts: Palantir AIP Operator, Azure AI Foundry deployments, and GitHub merge-queue controls.
- Gap closure: this IP closes fleet control, kill-switch propagation, and deployability evidence with tenant-scoped policy enforcement.
- Evidence source: `microservices/foundry/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/foundry/bc-sources/` when present.
