---
id: ADR-0025
status: Proposed
doc_status: published
---

# ADR-0025: Foundry as the engineering platform — repoctl, catalog, gates, fitness functions, supply chain, customer-facing builder surfaces all under one axis

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0020 (provider adapter), ADR-0021 (capability registry), ADR-0022 (autonomy ceiling), ADR-0023 (sandbox), ADR-0024 (eval harness — same gate substrate)

---

## Context

The 2026-05-09 reframing folded the standalone "Foundry engineering platform" axis into Foundry. The thesis: every engineering surface that gates how we build (repoctl, catalog, claim-ceiling validator, foundation-bypass ledger, plane-gated CI lanes, scorecards, fitness functions, ADR templates, branch-protection-as-code, signed commits, supply-chain attestation, plugin substrate trust gates, plugin marketplace authoring, customer-facing builder surfaces) is the same substrate that gates how the agent runtime executes capabilities — and the same agent runtime that authors workflows is the agent runtime that authors PRs against the same fitness functions. Splitting these into separate axes fractures the cohesion: it produces two policy stores, two scorecard surfaces, two reviewer pools, two on-call rotations.

The forces are: (a) recursion — the same agent runtime that authors customer workflows must author engineering PRs under the same gates; (b) authority unification — one fitness-function suite gates both an agent step and a human PR; (c) customer-facing parity — the workflow studio, plugin authoring, and regional-pack authoring surfaces are exactly the customer-side projection of the internal builder surfaces; (d) per-capability metering — engineering work is just another capability invocation that happens to author code rather than execute a workflow.

---

## Decision

We consolidate the engineering platform surfaces into the foundry. The axis owns: `repoctl`, the catalog, the claim-ceiling validator, the foundation-bypass ledger, plane-gated CI lanes, scorecards, fitness functions, ADR templates, branch-protection-as-code, signed commits, supply-chain (Trivy / Cosign / SBOM), plugin substrate trust gates, plugin marketplace authoring, and the customer-facing builder surfaces (workflow studio, plugin authoring, regional-pack authoring). Recursion is structural: the same runtime authors workflows and PRs; the same fitness function gates both.

### Crate layout

Engineering platform crates sit alongside the agent runtime under the Foundry namespace:

```
crates/oya-foundry-*                 — agent runtime (capability, step, run, evidence, provider, autonomy, sandbox, eval)
crates/oya-intelligence-catalog-{kernel,app,api} — catalog record + projection
crates/oya-governance-gate-{kernel,domain,app} — gate primitive + rule evaluation
crates/oya-intelligence-bypass-{kernel,app}      — foundation-bypass ledger + autonomy break-glass ledger
crates/oya-governance-lane-{kernel,app}        — CI lane primitive + per-PR routing
crates/oya-governance-*                — one crate per fitness function
crates/oya-governance-scorecard-{kernel,app}   — quality scorecard rollup
crates/oya-governance-supply-app               — Cosign + Trivy + SBOM
crates/oya-intelligence-plugin-substrate-app     — plugin sandbox + signing
crates/oya-intelligence-marketplace-app          — plugin + capability marketplace authoring
```

The `oya-foundry-*` namespace is governed by the same team as `oya-foundry-*`; cross-crate review within Foundry does not require a cross-microservice label.

### Catalog as ground truth

```rust
// crates/oya-intelligence-catalog-kernel/src/lib.rs
pub struct CatalogRecord {
    pub crate_id: CrateId,
    pub plane: Plane,                      // control | data | analytics
    pub lane_class: LaneClass,             // rust | typescript | database | security | cross-microservice
    pub claims: ClaimSet {
        pub api_stability: StabilityTier,  // preview | stable | GA
        pub security_review: SecurityReviewState,
        pub supply_chain: SupplyChainAttestation,
    },
    pub bypasses: Vec<BypassEntry>,        // structurally bounded; each has owner + expiry + rationale
    pub owners: Vec<TeamId>,
}
```

The catalog is the projection that the registry (ADR-0021), the autonomy gate (ADR-0022), the sandbox (ADR-0023), and the eval harness (ADR-0024) all consume.

### Claim-ceiling validator

Every new crate's claims (API stability, security review, supply chain) are validated against what the foundation has actually shipped. The validator ratchets WARN→BLOCK per wave: each wave promotes ≥ 1 WARN class to BLOCK so the gate stays live. The PRD target is ≥ 1 block per 100 PRs.

### Foundation-bypass ledger

Bypasses are real (the system would freeze without them) but bounded: every entry carries an owner, an expiry, and a rationale; the ledger is published; an automated monitor alerts at 80% of the expiry window and forces removal at 100%. Autonomy break-glass entries (ADR-0022) live in the same ledger under a distinct class.

### Plane-gated CI lanes

Every PR is routed to its plane lane (`rust-control`, `rust-data`, `rust-analytics`, `typescript-control`, `database-data`, `security-cross-microservice`, ...) and only the relevant fitness functions execute. A PR that touches multiple planes runs the union.

### Fitness functions

Per-domain fitness functions live as crates under `oya-governance-*`. Examples:

- `oya-governance-tenant-shape` — every tenant-touching surface conforms to the tenancy contract.
- `oya-governance-audit-emission` — every regulated capability emits to the audit chain.
- `oya-governance-data-use-boundary` — every data-class touch passes the consent gate.
- `oya-governance-eventing-topic` — every emitted event conforms to its topic schema.
- `oya-governance-flat-crates` — flat path shape plus kernel/domain/app/adapter/api/worker/runtime layering.
- `oya-governance-doc-catalog` — every public surface has a docs page.
- `oya-governance-product-prd` — every product has a PRD with the required sections.
- `oya-governance-horizontal-scale` — every stateful surface has a documented horizontal-scale story.
- `oya-governance-contract-orphan` — every cross-microservice contract has both an owner and a consumer.
- `oya-governance-license` — license-tier gate (AGPL/GPL hard-fail in product code; SSPL/BUSL ADR review).
- `oya-governance-supply-chain` — Cosign + Trivy + SBOM coverage.

### Customer-facing builder surfaces

The same engineering platform surfaces project outward as customer products:

- **Workflow Studio** — agent-authored workflows surface (the runtime is Foundry; the UX shell is the SaaS axis but the agent authoring capability is Foundry).
- **Plugin authoring + signing + marketplace publishing** — same trust substrate as internal supply chain.
- **Regional-pack authoring** — same gate substrate as internal regulatory packs.
- **Capability authoring SDK** — same kernel as internal capability authoring.

### Recursive property

The same agent runtime that authors customer workflows authors PRs against the same fitness functions. The same fitness function that gates a PR evaluates an agent step. There is no separate "agent gate" and "human gate" — there is one gate substrate.

### CI lanes

- `foundry-platform-cohesion` — asserts no `oya-foundry-*` crate violates the architecture-boundary fitness function.
- `foundry-platform-recursion` — asserts that an agent-authored PR passes through the identical lane set as a human-authored PR (synthetic test).
- `foundry-platform-claim-ratchet` — asserts the wave promoted ≥ 1 WARN→BLOCK.
- `foundry-platform-bypass-expiry` — asserts no ledger entry is past 100% of its expiry without action.

---

## Consequences

### Positive
- One team, one substrate, one policy store, one scorecard, one reviewer pool — cohesion at the structural level.
- Recursion is real: the runtime that automates engineering operates under the same gates as the engineers it augments.
- Customer-facing builder surfaces inherit the internal trust substrate without a parallel build.
- The capability runtime and the engineering platform share the catalog; a change to one is visible to the other.
- One on-call rotation owns both — no coordination tax during incidents.

### Negative
- The Foundry team's surface area is large; we may need to split into sub-teams (runtime / platform / customer-builder) at scale while keeping the substrate unified.
- Recursion makes failure modes self-amplifying: a fitness-function bug breaks both human and agent PRs simultaneously.
- Customer-facing builder surfaces are commercial products; they create a tension between internal-tooling tempo and external-product polish.

### Operational
- Runbook: `runbooks/foundry-platform-incident.md` — joint runtime + platform incident response.
- Runbook: `runbooks/foundry-fitness-rollback.md` — how to roll a fitness function back without freezing every PR.
- Runbook: `runbooks/foundry-bypass-expiry-monitor.md` — the automation that retires bypasses on schedule.
- On-call: Foundry on-call covers both runtime and platform; rotation depth must reflect the broader surface.
- Per-wave review: claim-ceiling ratchet, bypass-ledger audit, fitness-function coverage map.

---

## Alternatives considered

1. **Keep Foundry engineering platform as a separate axis.** Pros: clean per-microservice bounded contexts. Cons: two policy stores, two scorecards, two reviewer pools; the recursion thesis fragments. Rejected — cohesion is the entire point.
2. **Push engineering platform into a `platform-*` cross-cutting team.** Pros: signals it is shared substrate. Cons: dilutes ownership; the platform team would not own the runtime that consumes the platform, so the recursion is broken. Rejected.
3. **Adopt an external developer-platform product (Backstage / Port / OpsLevel / Cortex / Humanitec) as system of record.** Pros: less to build. Cons: external SoR for our most cohesion-critical authority surface; cannot enforce our autonomy-ceiling shape; cannot author capabilities; cannot run our fitness functions natively. Rejected per the build-vs-buy posture.
4. **Customer-facing builder surfaces in a separate "ISV product" axis.** Pros: clarity for commercial customers. Cons: produces a parallel trust substrate that drifts from the internal one. Rejected — we sell the same substrate we use.

---

## Open questions

1. When the Foundry team splits into sub-teams (runtime / platform / customer-builder), how do we keep the catalog and fitness-function authority unified? *Owner: `foundry` + `council-architecture`.*
2. The customer-facing capability authoring SDK ships in Rust + TypeScript; do we author one and codegen the other, or maintain both natively? *Owner: `foundry` + `platform-api-sdk`.*
3. Bypass-ledger expiry monitor — at what alert noise level does the monitor lose attention? Do we need a graduated escalation? *Owner: `foundry` + `ops-sre-reliability`.*
4. Recursion failure mode: what is the playbook when a fitness-function bug blocks both human and agent PRs and the fix itself requires a PR? Foundation-bypass is the answer; what is the bypass policy for this exact case? *Owner: `foundry`.*

---

## References

- Internal: ADR-0021 (registry shares the catalog), ADR-0022 (autonomy gate uses the same Cedar bundle that the platform fitness functions reference), ADR-0024 (eval is the same harness substrate).
- Anti-references: external developer platforms as SoR are explicitly out per the build-vs-buy posture.
- Foundry capability publishing checklist: `docs/checklists/foundry-capability-publishing.md`.
