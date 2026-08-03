---
id: ADR-0394
title: "Bespoke-Rust Internal Developer Platform (IDP) central hub (Leptos portal + ops-BFF; supersedes/reconciles ADR-0170 Backstage)"
status: Proposed
planning_impact: true
deciders: founder, council-architecture
date: 2026-05-29
owner: council-architecture
supersedes: [ADR-0170]
superseded_by: []
amends: []
related: [ADR-0001, ADR-0011, ADR-0067, ADR-0090, ADR-0130, ADR-0131, ADR-0132, ADR-0170, ADR-0203, ADR-0209, ADR-0213, ADR-0372, ADR-0393, ADR-0476, ADR-0482, ADR-0509]
related_specs:
  - /specs/http-stack-policy.json
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/platform-architecture.json
milestone: M-IDP-CENTRAL-HUB
depends_on: [ADR-0393]
door: two-way
numbering_note: "decisions.json next_adr is ADR-0392, but ADR-0392 and ADR-0408 are already allocated out-of-band by the Buck2 build/CI reversal lane (branch feat/adr-0392-0408-buck2-reversal-2026-05-29, not yet merged to dev). To avoid a number collision this ADR is deliberately allocated ADR-0394 (sibling of ADR-0393 in this same docs PR). The numbering gaps ADR-0377..ADR-0391 and ADR-0395..ADR-0407 are left open and are NOT claimed by this lane; the ADR index will record ADR-0393/ADR-0394 as non-contiguous allocations alongside the existing documented gaps and the Buck2 ADR-0392/ADR-0408 allocations."
affected_surfaces:
  crates: [oya-application-shell-frontend-prototype, oya-ops-workspace-shell-kernel, oya-ops-workspace-shell-rest, oya-ops-workspace-shell-app, oya-ops-docs-portal-rest]
  microservices: [observability, developer-sdk]
  specs: [/specs/http-stack-policy.json, /specs/hyperscaler-architecture-invariants.json]
---

# ADR-0394: Bespoke-Rust Internal Developer Platform (IDP) central hub (Leptos portal + ops-BFF; supersedes/reconciles ADR-0170)

## Status

Proposed — 2026-05-29. DRAFT for founder review; this overturns an Accepted, load-bearing decision (ADR-0170 Backstage-style portal, depended on by ADR-0203/0209/0213) and must NOT auto-merge. In this ADR, **IDP means Internal Developer Platform** (portal/BFF), not the OIDC identity provider.

## Date

2026-05-29

## Supersedes

ADR-0170 ("Backstage-style internal developer portal"), in full as a **substrate decision**. ADR-0170 authorized Spotify Backstage (TypeScript/Node.js + React, plugin marketplace, MkDocs TechDocs, catalog processor) as the canonical internal-developer-portal substrate at `portal.oya.internal`. That substrate choice is reversed: the IDP is a **bespoke-Rust central hub** (a Leptos portal-shell over a bespoke-Rust ops-BFF). The *problem* ADR-0170 correctly identified — fleet-wide discoverability over ~60 microservices + ~870 crate records, with ownership/SLO/maturity/ADR/runbook aggregation — stands and is solved here without Backstage. Backstage is retained as a **feature reference only** (what surfaces a good IDP has), not as a runtime dependency.

## Superseded-by

—

## Related

ADR-0001 (one-product cohesion — the Internal Developer Platform dogfoods the catalog), ADR-0011 (cross-microservice contract registry — API catalog source), ADR-0067 (SurfaceCatalog / VisibilityTier — the shell's surface model), ADR-0090 (hyper canonical HTTP backbone + hyper/axum split — the BFF/SSR host backbone), ADR-0130 (agentic SLO-gated promotion — SLO authoring mandatory before any new Internal Developer Platform µservice promotes past dev), ADR-0131 (per-microservice flat layout), ADR-0132 (no-grouping / single-concern µservices), ADR-0170 (the superseded Backstage substrate), ADR-0203 (docs three-tier — retargeted, see below), ADR-0209 (compliance evidence automation — retargeted), ADR-0213 (Ecosystem-as-a-Service developer-sdk portal — retargeted), ADR-0372 (the prior SolidJS frontend, superseded by ADR-0393), ADR-0393 (Leptos canonical app-shell — the portal-shell stack this hub uses), ADR-0509 (hyperscaler single-crate-per-service decomposition — the BFF/module crate layout).

## Owner

council-architecture (with founder as deciding authority — this is a doctrine reversal of an Accepted, load-bearing ADR).

## Context

### Terminology and identity boundary

ADR-0394 uses **IDP** only in the Internal Developer Platform sense: a portal shell plus ops-BFF that aggregates catalog, docs, CI/CD, observability, agent-fleet, and admin surfaces. It does not select or own the OIDC identity-provider endpoint.

Human identity is governed by ADR-0476 and ADR-0482: Keycloak is the Phase-1 bridge, `oya-identity` is the founder-accepted bespoke Rust target, and cutover is gated on OIDC/OAuth2/WebAuthn/tenant-IdP-federation/MFA feature parity plus integration tests (`docs/decisions/ADR-0476-oya-identity-bespoke-human-identity.md:29-37`, `:66-78`; `docs/decisions/ADR-0482-bespoke-substrate-roadmap.md:52-60`). ADR-0187/Zitadel is superseded historical authority and must not be read as the live default for this portal.

### What ADR-0170 decided, and why it must be reconciled

ADR-0170 (Accepted 2026-05-18) chose Spotify Backstage as the canonical internal developer portal, explicitly rejecting a custom-built portal as "NIH … ~6 engineer-months for a worse version of Backstage." It documented two exceptions it had to carve: a Node.js runtime in the portal cluster (against ADR-0120 Rust-first) and tenancy-OAuth coupling. ADR-0170 shipped Helm-chart skeletons at `microservices/observability/iac/helm/backstage/` and is depended on by:

- **ADR-0203** (documentation engine three-tier) — Tier 2 = "the internal developer portal (ADR-0170 Backstage)" for federated per-service docs.
- **ADR-0209** (compliance evidence automation) — lists "Backstage developer portal (ADR-0170) — read-only auditor view" as one of the existing primitives.
- **ADR-0213** (Ecosystem-as-a-Service) — the `developer-sdk` portal "lives under" the Backstage substrate; `microservices/developer-sdk/decisions/ADR-SDK-0007` records "dev-portal-as-backstage-extension."

So ADR-0170 is load-bearing and cannot simply be deleted; it must be superseded with explicit retargeting of its dependents.

### Why bespoke-Rust now (the doctrine that moved)

Since ADR-0170 (2026-05-18) the governing doctrine has hardened in ways that make a Node/React/Docker Backstage runtime non-viable:

- **Container/runtime doctrine** forbids the Docker Inc. toolchain (containerd + BuildKit/nerdctl canonical); Backstage's distribution and the `ADR-0120` "documented Node.js exception" are no longer acceptable carve-outs under the hyperscaler-lens (active-upstream + self-hostable + no-managed-dep + hyperscaler-internal-equivalent).
- **ADR-0372 → ADR-0393** make **Leptos (Rust/WASM)** the canonical frontend; React/Node are off the table for any operator surface.
- **Bespoke-over-OSS doctrine** requires challenging every OSS pick against a bespoke-Rust alternative; the IDP surfaces are exactly the kind of catalog/aggregation/console work Oyatie is already building in Rust (`crates/oya-ops-workspace-shell-*`, `oya-ops-docs-portal-rest`, `oya-dev-cli` catalog readers, the slo-engine, Cedar authz, the ci-webhook-gateway adapter pattern).
- **Dogfooding doctrine**: Oyatie runs as a tenant of its own cloud; the IDP is the first internal product that exercises its own catalog, identity, SLO, CI, and agent-fleet seams end-to-end. A third-party portal cannot dogfood the substrate.

The NIH objection ADR-0170 raised is re-weighed and rejected: the catalog/SLO/CI/identity primitives already exist in Rust, so the bespoke hub is a projection-and-aggregation layer over existing seams, not a from-scratch Backstage clone — and it is the only option consistent with the Leptos + container + bespoke-Rust doctrine.

### The new wrinkle Backstage cannot serve: agentic development

ADR-0170's design predates the agentic-development substrate. The IDP central hub treats **AI agents as first-class API consumers**: agent-fleet management is a first-class module (active agents/missions, provider/seat-pool health, dispatch SSE, replay timelines), and every IDP surface is reachable through a stable machine-consumable BFF contract. Backstage's React-plugin model is human-first and cannot natively serve the agent-fleet console or expose the `.omc` agent/workflow state (which today has no API at all).

## Decision

Oyatie's Internal Developer Platform (IDP) is a **bespoke-Rust central operator hub** — a single-pane-of-glass portal — built as:

### 1. Leptos portal-shell (per ADR-0393)
A Leptos (Rust/WASM, SSR+hydration) full-stack portal-shell — the production-promoted `crates/oya-application-shell-frontend-prototype`. It mounts every IDP domain as a Cedar-gated **Surface** via the existing `SurfaceCatalog` model (`crates/oya-ops-workspace-shell-kernel`: `Surface{id, canonical_route, VisibilityTier (6-tier), SurfaceState (Live/ReservedComingSoon/Retired), owning_bc_id, cedar_fragments, openapi_contract}`, 14 slots per ADR-0067). SSR data contract = `render_envelope`. Cedar `VisibilityTier` gates which surfaces render per principal.

### 2. Ops-BFF / aggregation backend (greenfield)
A bespoke-Rust aggregation tier (the **stable BFF contract**) that fans out to each domain API, normalizes, caches, and serves the Leptos shell. It is the **only** component that holds upstream credentials; the WASM shell never talks to GitHub/Jenkins/ArgoCD/Mimir directly. **http-stack**: axum is the sanctioned control-plane choice for the CRUD/extractor-heavy BFF (must register an `axum` justification in `specs/http-stack-policy.json#justified_crates.axum`, same discipline as `oya-identity-workload-rest`); bare-hyper for latency-critical proxy/streaming paths (SSE/WS panel feeds); `leptos_axum` hosts server-fns on the same router. Flat single-crate-per-service per ADR-0509/0131 (kernel/usecase/adapter/rest/app split; REST crates framework-free with route consts 1:1 to OpenAPI 3.2.0).

### 3. Eighteen modules
The hub is organized as 18 modules, each a Cedar-gated surface fed by the BFF: **catalog, scaffolder (golden-path), scm, cicd, observability, agent-fleet-management, task-board, finops, feature-flags, secrets, incidents/on-call, scorecards, status-page, audit/rbac, provisioning/control-plane, docs-portal**, plus the **portal-shell** and **ops-BFF** themselves. Agent-fleet-management is a first-class module (it gives the `.omc` agent/workflow state its first API seam).

### 4. Stable BFF contract survives the SCM/CICD cutovers
SCM and CICD are integrated through the BFF's domain-client adapters (cloned from the canonical async-client pattern at `microservices/ci-webhook-gateway/crates/…-jenkins-adapter`/`-github-adapter`), so the shell depends on `/bff/api/v1/scm` and `/bff/api/v1/cicd` — **not** on GitHub or Jenkins directly. This lets the GitHub→bespoke-SCM and Jenkins→Argo cutovers happen behind the BFF contract without touching the shell. (Note: the reference Jenkins adapter is `reqwest::blocking`; all BFF adapters MUST be async so they do not starve the `leptos_axum` runtime.)

### 5. Authz seam
Every Internal Developer Platform action delegates human authentication and token authority to the current identity substrate from ADR-0476/ADR-0482 (Keycloak bridge during Phase 1, `oya-identity` after feature parity). The portal/BFF routes authorization through identity `/authorize` + `/authorize-with-token` + `/tokens/validate`; RBAC fragments via `oya-policy-cedar-api`; privileged actions (rollback, secret reveal, surface flip) require step-up. This seam does not revive ADR-0187/Zitadel as the OIDC IdP default.

### 6. Backstage = feature reference only; charts quarantined/retired
Backstage is retained as a **feature reference** (the surface taxonomy a mature IDP needs) but is **not** a runtime dependency. Node/React/Docker are forbidden per ADR-0393 (Leptos) + the container doctrine. The following are **quarantined and retired** (frozen, removed from any deploy/promotion path; physical removal is an implementation follow-up):
- `microservices/observability/iac/helm/backstage/` (Chart.yaml + values.yaml)
- `microservices/developer-sdk/iac/helm/backstage/` (Chart.yaml + values.yaml)
- `microservices/developer-sdk/decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md` is **superseded** by this ADR (the developer-sdk portal becomes a surface/module of the bespoke hub, not a Backstage extension).
- Backstage-coupled IPs (`microservices/developer-sdk/implementation-plans/IP-001`, `IP-008`; `microservices/docs/IP-DOCS-005-backstage-techdocs-renderer.md`) are retargeted to the bespoke docs-portal module (implementation follow-up).

### 7. Design principles
The hub is **cloud/k8s-optimized** (ArgoCD/kubers-native provisioning, live SLO/rollout panels), **agentic-development-optimized** (agents are first-class API consumers; agent-fleet-management is first-class; `.omc` state gets an API), and **pipeline-optimized** (CI/gate/rollout aggregation is a core surface, not a plugin). ADR-0130 SLO authoring is mandatory before any new Internal Developer Platform µservice promotes past `dev`.

## Retargeting of load-bearing dependents

Because ADR-0170 is referenced by Accepted/Proposed ADRs, each reference is retargeted from "Backstage" to "the ADR-0394 bespoke-Rust IDP central hub":

- **ADR-0203** (docs three-tier) — Tier 2 "the internal developer portal (ADR-0170 Backstage)" → "the internal developer portal (ADR-0394 bespoke-Rust IDP; docs-portal module replaces Backstage TechDocs; Tier-1 mdbook in-repo unchanged)." Tier 2's *role* (federated per-service docs alongside ownership/SLO/ADR index) is unchanged; only the substrate moves.
- **ADR-0209** (compliance evidence automation) — "Backstage developer portal (ADR-0170) — read-only auditor view" → "the ADR-0394 IDP audit/rbac module — read-only auditor view." The auditor-view *primitive* is unchanged; it is now served by the bespoke audit/rbac surface over `audit-chain` + `oya-policy-cedar-api`.
- **ADR-0213** (Ecosystem-as-a-Service) — the `developer-sdk` portal "lives under" Backstage → "is a surface/module of the ADR-0394 IDP." `ADR-SDK-0007` (dev-portal-as-backstage-extension) is superseded; the developer-sdk portal is a bespoke surface, not a Backstage plugin.

(The edits to ADR-0203/0209/0213 prose and to ADR-SDK-0007 status are an implementation follow-up code PR; this ADR records the retargeting decision and authorizes it. ADR-0170's own bidirectional supersession marker IS applied in this PR.)

## Rejected alternatives

- **Keep Backstage (status quo of ADR-0170).** Rejected: Node/React/Docker runtime violates the Leptos (ADR-0393) + container + hyperscaler-lens doctrine; cannot dogfood the Rust substrate; cannot natively serve agents-as-first-class-consumers or the `.omc` agent-fleet state.
- **Thin Rust shell over a Backstage backend.** Rejected: still carries the Node/React backend and its container/license surface; the BFF would just proxy Backstage's catalog processor, which is the exact component bespoke catalog projection replaces.
- **Buy a commercial IDP (Port, OpsLevel, Cortex).** Rejected: managed-service / SaaS dependency fails the hyperscaler-lens (self-hostable, no managed-service dep) and the in-house doctrine (ADR-0209/0211 lineage); also React/Node-based.
- **Per-product mini-portals, no fleet-wide hub.** Rejected for the same reason ADR-0170 rejected it: cross-product/cross-service discovery (the dominant use case) fragments.

## Consequences

### Positive
- One bespoke-Rust IDP, end-to-end Rust (Leptos shell → axum/hyper BFF → Rust adapters), dogfooding Oyatie's own catalog/identity/SLO/CI/agent-fleet seams; consistent with Leptos + container + bespoke-over-OSS doctrine.
- Agents are first-class consumers; the `.omc` agent/workflow state gets its first API; agent-fleet management is a core module.
- The stable BFF contract decouples the shell from GitHub/Jenkins, so the SCM and CI cutovers happen behind the contract.
- Eliminates the two ADR-0170 carve-out exceptions (Node.js runtime, documented Rust-first exception) — the IDP is Rust-first with no exception.

### Negative / cost
- Build cost: the catalog-projection, scaffolder, scorecard, and per-domain adapters are net-new Rust crates (greenfield ops-BFF + ~15 adapter/module crates). Re-weighed against ADR-0170's "~6 engineer-months" NIH objection: most of the projected surfaces are aggregation over already-real Rust seams, and the doctrine leaves no compliant alternative.
- The Backstage Helm charts + IP work + ADR-SDK-0007 are written off (quarantined). Sunk cost; the charts were skeletons, not a deployed portal.
- Loses Backstage's 40+ community plugins; each is re-implemented only as needed as a bespoke module. Accepted — the plugin breadth was never the point for an internal-only fleet.

### Operational
- ADR-0130: every new Internal Developer Platform µservice (ops-bff, catalog projection, scorecard, scaffolder, omc-state, …) authors `slos/*.openslo.yaml` before promoting past `dev`.
- The ops-BFF holds all upstream credentials; the WASM shell holds none. Secrets surface is read-only metadata/rotation/lease-TTL only (Cedar + step-up gated), NEVER values.
- Reconciliation pre-reqs to settle before the corresponding modules bind (tracked, not decided here): OIDC issuer for Internal Developer Platform login follows ADR-0476/ADR-0482 (Keycloak bridge → `oya-identity` after feature parity; ADR-0187/Zitadel is historical only); canonical FinOps surface (finops-portal + opencost now, oya-cost/meter/billing trio when ADR-0478/0479/0480 land); canonical catalog schema (BFF projects BOTH `registry/catalog/*.yaml` and per-µservice `ServiceCatalog`); the intelligence "foundry"→non-foundry rename before the agent-fleet console binds to those identifiers.

## Verification

- ADR-0170 carries the bidirectional supersession markers (`superseded_by: [ADR-0394]`, status `Superseded`) — see the companion edit in this PR.
- The Backstage Helm charts (observability + developer-sdk) and ADR-SDK-0007 are quarantined/superseded; ADR-0203/0209/0213 references are retargeted (prose/physical edits = implementation follow-up).
- The ops-BFF + portal-shell crates register `axum` justifications in `specs/http-stack-policy.json` and author SLOs per ADR-0130 (implementation follow-up).
- The bespoke catalog-projection serves a single read API reconciling both catalog schemas (implementation follow-up).

## References

- ADR-0170 — the superseded Backstage substrate decision (this PR adds the `superseded_by` marker).
- ADR-0203 / ADR-0209 / ADR-0213 — load-bearing dependents retargeted off Backstage onto the bespoke IDP.
- `microservices/developer-sdk/decisions/ADR-SDK-0007-…` — superseded (dev-portal becomes a bespoke surface, not a Backstage extension).
- ADR-0393 — Leptos canonical app-shell (the portal-shell stack).
- ADR-0067 — SurfaceCatalog / VisibilityTier surface model.
- ADR-0090 — hyper canonical HTTP backbone + hyper/axum split; ADR-0509 — flat single-crate-per-service decomposition.
- `specs/http-stack-policy.json` — axum sanctioned-with-justification (BFF + SSR host).
- `microservices/observability/iac/helm/backstage/`, `microservices/developer-sdk/iac/helm/backstage/` — Backstage Helm charts quarantined by this ADR.
- `crates/oya-ops-workspace-shell-*`, `crates/oya-ops-docs-portal-rest`, `crates/oya-application-shell-frontend-prototype` — existing Rust seams the hub builds on.
- `microservices/ci-webhook-gateway/crates/…-jenkins-adapter` / `-github-adapter` — the canonical async domain-client adapter pattern the BFF adapters clone.
- `.omc/idp-central-hub-campaign.json` — the full IDP central-hub campaign design + founder decisions.
