---
id: ADR-0007
status: Superseded
superseded_by: [ADR-702]
doc_status: published
---

# ADR-0007: Cedar policy engine for RBAC/ABAC + persona-tier autonomy ceiling (T1–T4) with per-capability runtime enforcement

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `tenancy-identity` (Cedar surface) + `foundry` (autonomy ceiling) + `council-privacy`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0008, ADR-0011

---

## Context

Cohesion (ADR-0001) requires a single authorization surface across all microservices. Without a unified policy DSL, every axis ships its own AuthZ logic — and the prior decade of multi-product engineering shows that drift between them produces both privacy regressions (over-grant) and operational regressions (under-grant breaks workflows). Cedar (AWS-authored, Apache-2 licensed, open-source policy DSL with formally-verifiable evaluator) gives the right shape: declarative, attribute-aware, suitable for both per-tenant scope (tenant-author policies) and global scope (council-author policies).

The agent runtime adds a second pressure. Foundry capabilities range from "summarize this doc" (low blast radius) to "provision 10k cloud VMs" (massive blast radius) to "execute this drug-prescription workflow" (regulator-bound). A single boolean allow/deny is insufficient; what's needed is a tier-graded autonomy scale where higher tiers require either per-step human approval or council-ratified uplift. The persona-tier framing (T1–T4) lets every capability declare its required ceiling, every tenant declare its accepted ceiling, and the runtime hard-fail any invocation that would exceed either.

---

## Decision

We adopt **Cedar** as the sole authorization policy engine for RBAC/ABAC across all axes, **persona tiers T1–T4** as the autonomy-ceiling scale, and **per-capability runtime enforcement** that consults both Cedar and the autonomy ceiling on every invocation.

### Cedar surface

- Engine: Cedar (Apache-2.0; in-house Rust binding under `crates/oya-policy-cedar-*`).
- Per-tenant scope: tenant admins author tenant-local policies via the Workflow Studio + admin CLI.
- Global scope: `council-privacy` and `council-architecture` author baseline policies (e.g. PHI hard-deny, defense-vertical exclusions per ADR-0008 §2.2.3).
- Cedar entity types are generated from `oya-tenancy-kernel` + `oya-identity-kernel` so policy authoring stays type-safe end-to-end.

```cedar
// Example: PHI hard-deny for ad targeting (ADR-0008 §2.2.1)
forbid (
    principal in ResourceClass::"AdTargetingService",
    action  in [Action::"target_ad", Action::"score_audience"],
    resource is Record
)
when {
    resource.data_class == "PHI" ||
    resource.data_class == "PCI" ||
    resource.data_class == "SENSITIVE_PIPA_ART23"
};

// Example: per-tenant capability autonomy uplift
permit (
    principal == Tenant::"acme-kr",
    action  == Action::"invoke_capability",
    resource == Capability::"cloud.iam.role.publish"
)
when {
    context.requested_autonomy_tier <= principal.autonomy_tier &&
    capability.autonomy_tier_required <= principal.autonomy_tier
};
```

### Persona tiers (T1–T4) — the autonomy-ceiling scale

| Tier | Reads as | Examples |
|---|---|---|
| **T1 — view-only** | Agent reads, suggests; never mutates | "Search for similar past tickets and surface them" |
| **T2 — advisory** | Agent drafts; human edits + approves | "Draft this PR / draft this ad campaign / draft this prescription" |
| **T3 — execute-with-approval** | Agent executes a step bounded by per-step human approval | "Provision N VMs (approve N first)"; "Buy ad inventory up to $X (per-bid approval)" |
| **T4 — auto-execute** | Agent executes within declared bounds without per-step approval | "Re-balance cell capacity within ±10% nightly"; "Auto-rotate idle credentials" |

Default: **T2** for all newly registered capabilities. Tenants explicitly uplift per capability per session; uplift requires per-tenant council ratification for T4. Safety-critical actuation (medical robotics, drone control, defense-tagged surfaces) is **T4-disabled by default** with founder + legal carve-out for any exception (DESIGN §3.0.2).

### Per-capability `autonomy_tier_required` field

Every capability declared under `registry/capability-templates/<id>.yaml`:

```yaml
id: cloud.iam.role.publish
namespace: oya.cloud.iam
autonomy_tier_required: T3
data_classes_touched: [INTERNAL_ONLY]
regulatory_packs_consumed: [oya-pack-kr.K-ISMS-P]
evidence_emission_topic: oya.cloud.iam.role-published.v1
plane: control
```

The runtime gate (in `crates/oya-intelligence-runtime-policy-*`) refuses to invoke if `tenant.autonomy_tier < capability.autonomy_tier_required`.

### Runtime enforcement gate

```rust
// crates/oya-intelligence-runtime-policy
pub async fn evaluate_invocation(
    principal: &Principal,
    tenant: &Tenant,
    capability: &Capability,
    context: &InvocationContext,
) -> Result<AuthorizedInvocation, AuthorizationError> {
    // 1. Cedar evaluation — RBAC + ABAC
    let cedar_decision = cedar_engine.evaluate(principal, action_for(capability), capability_resource(capability), context)?;
    if !cedar_decision.allow { return Err(AuthorizationError::CedarDeny(cedar_decision.diagnostics)); }

    // 2. Autonomy-ceiling check
    if tenant.autonomy_tier < capability.autonomy_tier_required {
        return Err(AuthorizationError::AutonomyCeiling { tenant: tenant.autonomy_tier, required: capability.autonomy_tier_required });
    }

    // 3. Per-class data-use boundary (ADR-0008)
    for class in capability.data_classes_touched {
        ensure_class_permission(tenant, principal, class, capability)?;
    }

    // 4. Per-step approval gate (T3 only)
    if capability.autonomy_tier_required == AutonomyTier::T3 && !context.has_step_approval() {
        return Err(AuthorizationError::StepApprovalRequired);
    }

    // 5. Audit emission (ADR-0003)
    audit_chain.emit(AuditEvent::CapabilityAuthorized { principal, tenant, capability, decision: &cedar_decision }).await?;

    Ok(AuthorizedInvocation { /* ... */ })
}
```

### Boundary

- Applies to: every capability invocation (Foundry runtime), every cross-microservice API call that bears regulated authority, every cloud control-plane mutation, every workflow step that touches Ontology (ADR-0006).
- Does not apply to: per-cell ephemeral synchronous calls strictly inside a single capability invocation (which is itself authorized at the boundary).

---

## Consequences

### Positive

- One Cedar surface = mechanically true cohesion at the AuthZ substrate.
- Persona-tier ceiling lets every regulated workflow declare its autonomy requirement explicitly; agents cannot escape via misconfigured per-microservice logic.
- Cedar's formally-verified evaluator and explicit-deny-by-default semantics give regulator-defensible authorization decisions.
- Per-capability `autonomy_tier_required` is a catalog field; auditors can ask "what runs at T4 in this tenant?" and the answer is a SQL query, not a code crawl.

### Negative

- Cedar policy authoring has a learning curve; mitigated by Cedar tooling + per-microservice policy templates + Workflow Studio policy authoring UI.
- Per-invocation Cedar evaluation adds 0.5–2 ms; the cohesion + audit guarantee justifies the cost.
- T3 per-step approval is operationally expensive for high-frequency capabilities; mitigation is per-tenant T4 uplift with bounded delta + audit emission.

### Operational

- On-call: `EVT-AUTONOMY-CEILING-DENY` per-tenant rate alert; sustained denial → tenant-onboarding runbook trigger.
- Runbooks: `runbooks/autonomy-tier-uplift.md`, `runbooks/break-glass-with-evidence.md`, `runbooks/cedar-policy-rollback.md`.
- CI: `oya-governance-cedar-coverage` (every regulated capability has an explicit policy), `oya-governance-autonomy-ceiling-coverage` (every capability declares the field).
- Eval: per-capability eval set (ADR-0019 cadence) includes adversarial autonomy-escape attempts; failing eval blocks promotion.

---

## Alternatives considered

### Alternative A — OPA / Rego

- **Pros:** large community, rich tooling.
- **Cons:** Rego type system weaker than Cedar's; AWS-spec backing on Cedar means evaluator semantics are formally verified.
- **Rejected because:** Cedar's verification + AWS-spec stability + Apache-2 license.

### Alternative B — Per-axis policy DSLs

- **Pros:** axis-specific ergonomics.
- **Cons:** drift catastrophe; ADR-0001 forbids substrate forking.
- **Rejected because:** cohesion.

### Alternative C — Hard-coded role check (no DSL)

- **Pros:** simplest.
- **Cons:** every policy change is a code change; non-engineers cannot author policy; regulator cannot audit declaratively.
- **Rejected because:** scale + auditability.

### Alternative D — Two-tier autonomy (advisory vs auto)

- **Pros:** simpler.
- **Cons:** loses the per-step-approval middle ground that maps to the dominant agent-as-copilot UX.
- **Rejected because:** T3 is the most-used tier in practice.

---

## Open questions

1. **Q1.** Per-tenant Cedar policy size cap — what's the budget before evaluation latency degrades? Default: 10k policy lines per tenant; soft warn at 7k. → owner: `tenancy-identity`.
2. **Q2.** Cedar policy hot-reload cadence — eventual consistency vs strong-consistency on policy publish? Default: strong-consistent within a region (≤ 1 s); eventual cross-region. → ADR-0010 (regional packs).
3. **Q3.** Is `T0` needed (read-prohibited stand-down mode for break-glass aftermath)? Default: NO; `T1` + per-capability deny achieves the same. → owner: `foundry`.
4. **Q4.** Per-capability eval-set ownership — Foundry team or per-microservice team? Default: per-microservice authors; Foundry runs the harness. → ADR-0011.
5. **Q5.** Agent-on-behalf-of-user — does the agent inherit the user's autonomy ceiling, or the tenant's? Default: minimum of both (most-restrictive). → ADR-0008.

---

## References

- `docs/DESIGN.md` §3 (Foundry: capability registry, autonomy ceiling, evidence emission), §10 (cross-microservice contract `Autonomy ceiling policy`)
- `docs/PRIVACY-PROGRAM.md` §2.2.8 (agent-runtime specifics under autonomy ceiling)
- `docs/COMPLIANCE-MATRIX.md` §3.7 (EU AI Act Art 14 human oversight)
- `docs/GLOSSARY.md` §8 ("Persona tier (T1..T4)", "Autonomy ceiling")
- ADR-0001 (cohesion), ADR-0002 (identity kernel + Cedar evaluation site), ADR-0003 (audit emission per decision), ADR-0008 (data-class permissions composition), ADR-0011 (catalog `autonomy_tier_required` field)
- Cedar policy language: https://www.cedarpolicy.com/
