---
id: ADR-0021
status: Proposed
doc_status: published
---

# ADR-0021: Foundry capability registry and MCP gateway — `Capability` schema, MCP-compatible discovery, per-tenant endpoint

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0020 (provider adapter model), ADR-0022 (autonomy ceiling enforcement), ADR-0024 (eval harness — gates publish), ADR-0025 (Foundry as engineering platform)

---

## Context

Capabilities are the unit of work in the Foundry runtime: each capability is a typed contract (input schema, output schema, autonomy requirement, data classes touched, audit-chain emission topic, regulatory packs consumed, cost profile, sunset policy) that an agent can invoke. Without a single registry, capability authors would scatter contracts across crates and consumers would have no agent-discoverable entry point; every cross-microservice surface would need a bespoke client. Without an industry-standard discovery surface, we cannot integrate with the agent ecosystem (Claude Desktop, Cursor, Continue, Cline, OpenAI Apps SDK) without per-client adapters. Without a per-tenant endpoint, we cannot enforce per-tenant routing, autonomy ceilings, or evidence emission boundaries at the protocol layer.

We need a capability schema rich enough to gate autonomy, route providers, attribute cost, and emit audit-chain evidence — and a discovery surface that any MCP (Model Context Protocol) client can consume so agents outside our runtime can invoke our capabilities under the same trust envelope.

---

## Decision

We define the canonical `Capability` record in `oya-intelligence-capability-kernel` and serve it via an MCP-compatible gateway that exposes a per-tenant endpoint. The catalog YAML in `registry/catalog/` is the source of truth; the kernel projects it into typed records at runtime.

### Capability primitive (`oya-intelligence-capability-kernel`)

```rust
// crates/oya-intelligence-capability-kernel/src/lib.rs
pub struct Capability {
    pub id: CapabilityId,                            // e.g. "workflow.preview-vertical"
    pub namespace: Namespace,                        // owning axis ("foundry", "saas", "vertical-healthcare", ...)
    pub input_schema: JsonSchema,
    pub output_schema: JsonSchema,
    pub description: CapabilityDescription,          // dual-audience: agent-readable + human-readable
    pub autonomy_tier_required: AutonomyTier,        // T1..T4; gated at runtime per ADR-0022
    pub data_classes_touched: DataClassSet,          // exhaustive; any class not declared is forbidden
    pub evidence_emission_topic: AuditTopic,         // ADR-0025 audit-chain destination
    pub regulatory_packs_consumed: RegulatoryPackSet,// e.g. {"kr-pipa", "hipaa"}; gated per tenant pack
    pub cost_profile: CostProfile {
        pub per_invocation_usd_ceiling: Decimal,
        pub per_tenant_monthly_usd_ceiling: Decimal,
        pub provider_preference: Vec<ProviderId>,    // resolved via ADR-0020 router
    },
    pub sunset_policy: SunsetPolicy {
        pub announce_at: Option<DateTime>,
        pub eol_at: Option<DateTime>,
        pub migration_target: Option<CapabilityId>,
    },
    pub side_effects: SideEffectDeclaration,         // reads/writes tenant data, external, events
}
```

### MCP gateway (`oya-intelligence-mcp-gateway`)

The gateway implements the [Model Context Protocol](https://modelcontextprotocol.io) so any compliant client can discover and invoke capabilities:

- **Per-tool instructions** — each capability's `description.agent_readable` becomes the MCP tool description; `description.human_readable` ships in the docs portal.
- **Server-level prompts** — higher-level workflows are surfaced as MCP prompts: `workflow.preview-vertical`, `regional-pack-authoring`, `adr-promotion`, `foundation-bypass-renewal`, `capability-publish`. These are agent-recognizable starting points for multi-step orchestrations.
- **Per-tenant endpoint** — every tenant gets a dedicated MCP endpoint at `mcp.foundry.<region>.oyatie.<tld>/tenants/<tenant-id>`; the endpoint authenticates the client to the tenant binding and refuses cross-tenant reach.
- **OAuth 2.0 device flow** — clients (Claude Desktop / Cursor / Continue / Cline / OpenAI Apps SDK / oya CLI) authenticate via device flow against the platform identity service; the issued token carries tenant + autonomy + capability scopes.
- **Audit-chain emission** — every MCP invocation emits to the `oya.foundry.capability.invoked` topic with the request envelope, the routing decision, the autonomy verdict, and the response summary.
- **Discovery contract** — `GET /capabilities` returns the per-tenant filtered set (capabilities the tenant is licensed to invoke under its current autonomy tier and pack binding).

### Persona-split CLI surface

The `oya` CLI is the agent-discoverable interface to the registry; it splits by persona per the SPEC §6 layout:

- `oya dev capability list|describe|invoke` — developer persona
- `oya admin capability publish|deprecate|sunset` — admin persona
- `oya agent capability invoke --tenant=<id> --autonomy=<tier>` — agent persona (also exposed as MCP tool)
- `oya catalog capability validate|promote` — catalog persona

Each subcommand is mirrored as an MCP tool in the gateway, so the same surface is reachable from a human terminal, a programmatic call, and an agentic discover-then-invoke loop.

### Catalog flow

1. Author writes a `registry/catalog/oya-intelligence-capability-<id>.yaml` per the capability-record template.
2. CI lane `foundry-capability-schema` validates against `oya-intelligence-capability-kernel`'s JSON schema.
3. CI lane `foundry-capability-eval-coverage` confirms a golden eval set is checked in (ADR-0024 dependency).
4. CI lane `foundry-capability-autonomy-coherence` validates that the declared autonomy tier is consistent with the declared data classes (e.g. PHI access cannot be T4-default).
5. On merge, `oya admin capability publish` projects the YAML into the registry and emits `EVT-CAPABILITY-AUTHORED` to the audit chain.
6. The MCP gateway hot-reloads the registry on the publish event; new tenants discovering the endpoint see the new tool within seconds.

### CI lanes

- `foundry-capability-schema` — kernel-driven validation of every `registry/catalog/oya-intelligence-capability-*.yaml`.
- `foundry-capability-eval-coverage` — refuses publish without an eval set (delegates to ADR-0024).
- `foundry-capability-autonomy-coherence` — refuses incoherent autonomy/data-class pairings.
- `foundry-mcp-gateway-contract` — asserts MCP wire compatibility against the upstream MCP test set.
- `foundry-capability-tenant-isolation` — synthetic cross-tenant reach test; must always refuse.

---

## Consequences

### Positive
- One source of truth for capability shape; every consumer reads from the same projection.
- MCP compatibility means any compliant agent client can integrate without bespoke work; we ride the agent-tooling ecosystem instead of rebuilding it.
- Per-tenant endpoint forces tenant isolation at the protocol layer — no cross-tenant data leak via misconfigured client.
- Per-tool description and server-level prompts give us a curated surface for agent discoverability without leaking implementation details.
- Catalog YAML is human-reviewable and diff-able; capability changes are first-class PRs.

### Negative
- MCP is a young protocol; spec churn may force breaking changes in the gateway.
- Per-tenant endpoints multiply DNS + TLS + routing surface; capacity planning is non-trivial at scale.
- The MCP server-level prompt surface is opinionated; we must curate which workflows we surface (over-surfacing dilutes discoverability, under-surfacing hides value).

### Operational
- Runbook: `runbooks/foundry-capability-publish.md` — author flow + lane gates + post-publish smoke.
- Runbook: `runbooks/foundry-mcp-gateway-incident.md` — per-tenant endpoint outage triage; how to fail capabilities open vs. closed.
- On-call: MCP gateway is the public face of every capability; it gets the highest-tier paging.
- Per-tenant onboarding: provisioning includes generating the MCP endpoint URL, OAuth client credentials, and a discovery primer for the tenant's preferred agent client.

---

## Alternatives considered

1. **Bespoke RPC instead of MCP.** Pros: no spec churn risk; full control. Cons: every agent client needs a custom adapter; we lose the upstream ecosystem; we re-invent discovery semantics. Rejected — the cost of building agent-client adapters dwarfs the spec churn cost.
2. **Single global endpoint (no per-tenant endpoint).** Pros: simpler operations. Cons: tenant isolation becomes an application-layer responsibility; one routing bug becomes a cross-tenant leak. Rejected — the isolation must be structural.
3. **Separate registries per axis.** Pros: each axis owns its own surface. Cons: agents can't discover across axes without a federation layer; cohesion fractures; cost ceilings cannot be enforced globally. Rejected — capability registry is a Foundry-owned cross-microservice contract.
4. **OpenAPI as the discovery surface (instead of MCP).** Pros: mature; widely tooled. Cons: not agent-native; doesn't carry tool descriptions in an agent-consumable shape; doesn't carry per-prompt server-level workflows. Rejected for agent-facing discovery (we still publish OpenAPI for programmatic clients).

---

## Open questions

1. How do we version the MCP endpoint when the protocol evolves? Per-tenant endpoint pin to a protocol version, or roll all tenants forward together? *Owner: `foundry`; target the next pack.*
2. What is the rate-limit shape on the per-tenant MCP endpoint? Per-capability, per-tenant, per-IP? *Owner: `foundry` + `ops-sre-reliability`.*
3. Should agent-authored capabilities (capabilities composed by another capability) be marked structurally distinct in the registry to prevent infinite-loop discovery? *Owner: `foundry`.*
4. How are per-region MCP gateway endpoints reconciled with tenant residency — does a strict-KR tenant ever expose a non-KR endpoint? *Owner: `foundry` + `platform-privacy-dub`.*

---

## References

- Internal: ADR-0020 (provider routing the registry triggers), ADR-0022 (autonomy gate every invocation passes through), ADR-0024 (eval gate at publish), ADR-0025 (audit-chain emission topic ownership).
- External: [Model Context Protocol specification](https://modelcontextprotocol.io); MCP reference clients (Claude Desktop, Cursor, Continue, Cline, OpenAI Apps SDK).
- Catalog template: `docs/templates/capability-record-template.yaml`.
- Capability publishing checklist: `docs/checklists/foundry-capability-publishing.md`.
