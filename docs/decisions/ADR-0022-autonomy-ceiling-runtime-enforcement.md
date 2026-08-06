---
id: ADR-0022
status: Proposed
doc_status: published
---

# ADR-0022: Autonomy ceiling — runtime enforcement via Cedar policy at every capability invocation

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0020 (provider adapter), ADR-0021 (capability registry), ADR-0023 (sandbox; complements process-level isolation), ADR-0024 (eval harness; adversarial cases test the ceiling), ADR-0025 (engineering-platform; same gates apply to agent-authored PRs)

---

## Context

A capability declares the autonomy tier it requires (T1 = recommend-only, T2 = supervised execution, T3 = scheduled autonomous, T4 = continuous autonomous). The actual autonomy granted to an invocation depends on the tenant's configuration, the capability's minimum, the regional/vertical pack overrides, and the data subject's class. Without a runtime gate, a misconfigured tenant could grant T4 to a capability whose data flows would breach a healthcare or fintech pack; an agent invoking on behalf of a tenant could escalate above the tenant's permissions; a single-policy-failure could produce a catastrophic, irreversible action with no audit trail.

We need an enforcement point that is **outside** the capability author's control, **inside** the runtime hot path, **deterministic** about which factors lower the ceiling, and **auditable** for every decision (allow / deny / break-glass). Cedar (the policy engine adopted at the platform layer) is the policy substrate; the autonomy ceiling is a Cedar policy bundle plus a runtime gate that evaluates it on every capability invocation.

---

## Decision

We enforce the autonomy ceiling at `oya-intelligence-policy-app` on **every** capability invocation. The effective ceiling is the minimum of four sources; agents inherit (and cannot exceed) tenant permissions; healthcare and fintech tenant classes force T1/T2 maxima for regulated capabilities; agentic ad-buying defaults to recommend-only.

### Effective-ceiling resolution

```rust
// crates/oya-intelligence-policy-kernel/src/ceiling.rs
pub struct AutonomyCeilingInputs {
    pub tenant_configured: AutonomyTier,
    pub capability_min_required: AutonomyTier,
    pub vertical_pack_cap: AutonomyTier,
    pub subject_class_cap: AutonomyTier, // e.g. minor / vulnerable
}

impl AutonomyCeilingInputs {
    pub fn effective(&self) -> AutonomyTier {
        AutonomyTier::min_of(&[
            self.tenant_configured,
            self.capability_min_required,
            self.vertical_pack_cap,
            self.subject_class_cap,
        ])
    }
}
```

### Runtime gate (`oya-intelligence-policy-app`)

```rust
// crates/oya-intelligence-policy-app/src/gate.rs
pub fn enforce(
    cedar_engine: &CedarEngine,
    invocation: &InvocationRequest,
    capability: &Capability,
    tenant: &Tenant,
    invoker: &InvokerPrincipal,    // human, agent, or agent-on-behalf-of-agent
) -> CeilingDecision {
    let inputs = AutonomyCeilingInputs::collect(capability, tenant, invocation.subject_class());
    let effective = inputs.effective();

    // Cedar policy bundle includes:
    //  - tenant.autonomy.policy
    //  - capability.autonomy.policy (from registry projection)
    //  - vertical-pack.autonomy.policy (e.g. hipaa-pack.cedar)
    //  - global.autonomy.policy (founder-set anti-patterns)
    let cedar_decision = cedar_engine.is_authorized(
        principal: invoker.into(),
        action: invocation.action(),
        resource: capability.into(),
        context: ctx_with_effective_tier(effective),
    );

    let decision = match (cedar_decision, effective) {
        (Allow, tier) if tier >= invocation.requested_tier() => CeilingDecision::Allow(effective),
        (Allow, _) => CeilingDecision::Lowered(effective),    // ran but at the lower tier
        (Deny, _) => CeilingDecision::Deny(reason),
    };

    // Always emit decision evidence (allow + lowered + deny + break-glass)
    audit_chain.emit(EVT_AUTONOMY_DECISION { invocation, inputs, effective, decision });

    decision
}
```

### Break-glass workflow

Break-glass raises the effective ceiling for a bounded window when the standing policy would block work that the operator believes is justified.

- **M-of-N approval.** Standard break-glass requires 2-of-3 (any two designated approvers). **Catastrophic-class** break-glass (T4 on a regulated capability touching tier-1 data classes, or any capability flagged `catastrophic_blast_radius`) requires **3-of-5**.
- **Window-bounded.** Each break-glass carries an explicit expiry; the runtime auto-revokes at the expiry without action.
- **Audit-chain anchored.** Approval, the override, every invocation under the override, and the auto-revoke are all evidence-emitted.
- **Foundation-bypass ledger entry.** Every break-glass appears in the same ledger as foundation-bypass entries (per ADR-0025); the ledger is published; expiry SLA is enforced by the same monitor.

### Tenant-class overrides

```rust
// crates/oya-intelligence-policy-domain/src/tenant_class.rs
pub fn class_override(class: TenantClass, capability: &Capability) -> Option<AutonomyTier> {
    use TenantClass::*;
    match (class, capability.regulatory_packs_consumed.dominant()) {
        (Healthcare, RegulatoryPack::Hipaa | RegulatoryPack::KrPipaHealth) => Some(AutonomyTier::T2),
        (Fintech, RegulatoryPack::Pci | RegulatoryPack::KrFsc | RegulatoryPack::JpFsa) => Some(AutonomyTier::T2),
        (PublicSector, _) if capability.touches_class(DataClass::CitizenIdentifier) => Some(AutonomyTier::T1),
        _ => None,
    }
}
```

### Agentic ad-buying default

Capabilities namespaced under `ads.*` whose action is `bid` or `budget.adjust` default to T1 (recommend-only) unless the tenant explicitly grants T2/T3 via Cedar policy and a co-signed approval. T4 is structurally unavailable for any `ads.*` capability without a founder-grade policy override.

### CI lanes

- `foundry-autonomy-policy-coherence` — every capability YAML must declare a tier; the declared tier must be consistent with declared data classes and regulatory packs.
- `foundry-autonomy-runtime-gate` — integration test asserts the gate is on the hot path of every capability invocation; absence of an `EVT-AUTONOMY-DECISION` for an invocation fails the lane.
- `foundry-autonomy-break-glass` — synthetic break-glass test asserts M-of-N enforcement, expiry, and auto-revoke.
- `foundry-autonomy-tenant-class-override` — asserts healthcare/fintech tenants are forced to T1/T2 max on regulated capabilities even if the tenant config requests higher.
- `foundry-autonomy-agent-cannot-exceed-tenant` — synthetic test where an agent invokes with a stronger principal than its tenant grants; must deny.

---

## Consequences

### Positive
- Every capability invocation passes through one gate; no axis can route around it.
- The effective ceiling is deterministic — given the same inputs, the gate produces the same decision and the same audit emission.
- Break-glass is real (not a trapdoor) but bounded, audited, and self-expiring.
- Tenant-class overrides are structural — a misconfigured healthcare tenant cannot accidentally consent to T4 on a PHI capability.
- Cedar gives us the same policy substrate as the platform tenancy and identity layers; we do not maintain a parallel engine.

### Negative
- Cedar evaluation is on the hot path of every invocation; we must keep evaluation latency under the capability's invocation SLO budget.
- Policy bundles must be versioned and rolled out atomically; a partial-rollout split-brain produces inconsistent decisions across replicas.
- Break-glass M-of-N requires a coordinator that does not itself become a break-glass attack surface.

### Operational
- Runbook: `runbooks/foundry-autonomy-break-glass.md` — how to initiate, gather approvals, monitor the window, and verify auto-revoke.
- Runbook: `runbooks/foundry-autonomy-policy-rollback.md` — how to roll a policy bundle back without ground-stopping invocations.
- On-call: a spike in `EVT-AUTONOMY-DECISION` deny rate indicates either a misconfigured policy push or an attack; the alert is high-priority.
- Quarterly: red-team exercise against the ceiling (also an ADR-0024 eval cohort).

---

## Alternatives considered

1. **Trust the capability author to enforce its own ceiling.** Pros: simpler runtime. Cons: defense-in-depth fails at the first author bug; no uniform audit emission; no cross-capability policy. Rejected — this is exactly the failure mode the gate exists to prevent.
2. **OPA / Rego instead of Cedar.** Pros: large community. Cons: platform tenancy is already on Cedar; running two policy engines fragments authority. Rejected — single policy engine across the platform.
3. **Static policy at deploy time (no runtime evaluation).** Pros: no hot-path cost. Cons: cannot react to tenant pack changes, subject-class context, or break-glass within a deploy cycle. Rejected — autonomy is contextual, not static.
4. **Tier ceiling as a soft recommendation only.** Pros: max flexibility. Cons: a soft ceiling is not a ceiling; this is the failure mode we are eliminating. Rejected.

---

## Open questions

1. What is the latency budget for the Cedar evaluation, and how do we cache without breaking decision freshness on policy push? *Owner: `foundry` + `ops-sre-reliability`.*
2. Is the break-glass approver pool tenant-scoped, region-scoped, or global? Different scopes have different attack surfaces. *Owner: `foundry` + `ops-security`.*
3. How does the ceiling interact with cross-capability invocations (a T2 capability invoking a T3 sub-capability)? Inheritance, intersection, or independent re-evaluation? *Owner: `foundry`.*
4. Should the ceiling decision be replayable against a past tenant snapshot for forensic analysis? *Owner: `foundry` + `ops-compliance`.*

---

## References

- Internal: ADR-0021 (capability registry — declares the required tier), ADR-0023 (sandbox — process-level isolation complements policy enforcement), ADR-0024 (eval harness — adversarial autonomy-bypass cases are mandatory), ADR-0025 (audit chain — all decisions emit there).
- Cedar: [Cedar policy language](https://www.cedarpolicy.com).
- Compliance binding: KR PIPA, HIPAA, PCI DSS, KR FSC, JP FSA pack policies.
- Flat-crates binding: autonomy-ceiling enforcement lives in `crates/oya-intelligence-policy-kernel` and capability invocation surfaces consume it through flat `oya-foundry-*` crates. The retired `services/agent/daemon` path is historical only and must not be recreated.
