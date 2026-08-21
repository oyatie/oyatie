---
doc_status: published
id: ADR-0705
title: "Live product protocols, APIs, and communications plane"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-06
door: two-way
owner: council-architecture
supersedes: [ADR-0001, ADR-0013, ADR-0019, ADR-0090, ADR-0132, ADR-0166, ADR-0176, ADR-0249, ADR-0258, ADR-0314, ADR-0354, ADR-0516, ADR-0548]
superseded_by: []
amends: []
amended_by: [ADR-0719]
depends_on: []
related: []
milestone: W0
deliverables:
  - id: ADR-0705-D1
    description: "Live apex source-of-truth for topic product_protocol: Live product protocols, APIs, and communications plane."
    exit_criteria: "docs/decisions/ADR-0705-product-protocol-live-apex.md is Accepted with planning_impact true; member ADRs listed in supersedes are archived under docs/adr-archive/."
    verified_by: "oya-ci-required"
---
# ADR-0705: Live product protocols, APIs, and communications plane

## Status

**Accepted** — live consolidated source-of-truth entry for topic `product_protocol` (E5 2026-08-06).

## Context

Oyatie ADR corpus cleanup: agents must not treat every historical Accepted file as equal live law.
This apex consolidates **13** Accepted ADRs in the `product_protocol` topic. Member files are
**Superseded** by this apex and then archived; full text remains in git history.

Live resolution: prefer this apex; follow `supersedes` for provenance.

## Decision

1. **This ADR is the live reading entry** for topic `product_protocol` under the end-state ADR policy.
2. **Member ADRs listed in `supersedes`** are historical; normative gist is preserved below.
3. **Contradictions** among members are resolved by later higher-number members and by
   ADR-0515 / ADR-0363 / ADR-0562 / ADR-0615 / ADR-0635 / ADR-0637–0639 when applicable.
4. **Activation-sensitive** items (warm CAS, RE workers) remain fail-closed until explicit go-gate.

## Preserved member gists

- **ADR-1** (ADR-0001-cohesion-thesis-one-product-flat-catalog): We adopt the **cohesion thesis** as the foundational invariant of the Oyatie codebase, product, and roadmap: > Oyatie is one cohesive product across a flat catalog of shared microservices, joined at exactly six shared substrates: **single tenancy**, **single identity**, **single audit chain**, **single capability registry**, **single agent runtime*
- **ADR-13** (ADR-0013-product-license-policy): We adopt a **three-tier license policy** for product code with an explicit dev-only carve-out, a CI lane that hard-fails forbidden licenses, and a per-release SBOM generation requirement. ### Tier 1 — Allowed in product code (no review required) | License family | SPDX identifiers | |---|---| | Apache 2.0 | `Apache-2.0`, `Apache-2.0 WITH LLVM-excep
- **ADR-19** (ADR-0019-doc-catalog-and-update-protocol): We adopt a **doc catalog as protocol** with a structured per-doc record, a five-stage lifecycle, an agent-authoring policy with explicit roles, and a machine-readable mirror at `machine-readable/catalog.json`. ### Per-doc catalog record Every consolidated doc has a row in `docs/DOC-CATALOG.md` and a mirror in `docs/machine-readable/catalog.json`: `
- **ADR-90** (ADR-0090-hyper-canonical-http-backbone): Hyper 1.x is the canonical HTTP backbone for every µservice in the workspace. Direct deps for the runtime composition layer: | Crate | Version pin | Purpose | | ---------------- | ----------- | -------------------------------------- | | `hyper` | `1` | bare HTTP I/O protocol (h1/h2) | | `hyper-util` | `0.1` | tokio + auto-builder connection helper 
- **ADR-132** (ADR-0132-product-platform-and-bundle-dissolution): Effective immediately and going forward, every new µservice in oyatie ships as a flat single-concern µservice under `microservices/<ms>/` per ADR-0131. The following patterns are **prohibited**: - Creating a new `microservices/<bundle>/` folder that contains more than one user-facing concern (e.g., `microservices/connector/`, `microservices/workspa
- **ADR-166** (ADR-0166 — Schema Registry (Apicurio Registry; Confluent-compat API; AsyncAPI 3.): Oyatie adopts **Apicurio Registry 3.x** as the canonical schema registry. Properties: ### Schema kinds The registry holds: - **AsyncAPI 3.x** — event-driven contracts (eventing backbone per ADR-0005). One AsyncAPI doc per event-emitting µservice; per-event-class subject. - **Protobuf 3 (proto3)** — gRPC service definitions (per ADR-0145). One `.pro
- **ADR-176** (ADR-0176-brownout-degradation-signal-api): ### D-1. Normative response header Every public HTTPS REST response emits the header below. Public webhooks, AsyncAPI events, SSE, and WebSocket messages carry the same value as protocol-appropriate metadata; internal gRPC responses carry it as response metadata without creating a public RPC contract: ``` oya-degradation-class: nominal|degraded|bro
- **ADR-249** (ADR-0249-multi-category-marketplace-doctrine): ### D-1. Eight marketplace substrate microservices (NEW, built day-one) The eight substrates are NEW first-class microservices under `microservices/` per ADR-0131 flat layout. Each is single-concern per ADR-0132. Each serves ALL marketplace categories (and the existing plugin-app-store after refactor). #### D-1.1 `microservices/catalog/` — typed pr
- **ADR-258** (ADR-0258-api-versioning-model): We adopt twelve interlocking decisions (D-1 through D-12) that together constitute the canonical oyatie API versioning model. Each decision is independently enforceable; the bundle composes into a coherent system. ### D-1 — External (public) APIs use Stripe-style request-time pinning via `X-Oyatie-API-Version` header Every public API surface (Works
- **ADR-314** (ADR-0314-marketplace-as-universal-deal-settlement): > **Disposition light-edit (2026-08-06):** Context re-triage Accept: Marketplace deal-settlement substrate
- **ADR-354** (ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc): ### §B-1 HTTP/3 Fallback Chain **Decision:** Every oyatie endpoint that accepts external traffic MUST implement the following protocol fallback chain in the listed priority order. Inter-cell and internal endpoints are declared through §B-8 and remain gRPC over HTTP/2 with SPIFFE mTLS until a real endpoint-specific pull justifies a transport-runtime
- **ADR-516** (Agentic Delivery Fabric — the owned, cloud-native, productized unified delivery ): Adopt the **Agentic Delivery Fabric** as the apex product north-star: an owned, cloud-native, infinite-scale, productized platform that lets anyone automatically create and maintain hyperscaler-grade, well-architected, well-documented, well-maintained projects, with AI agents as the primary producers and quality built-in from project genesis and co
- **ADR-548** (Pipeline as product: neutral ratchet engine + policy packs on the paved road): ### D1 — The pipeline itself is a product: a neutral ratchet engine + policy packs The product is a NEUTRAL ratchet engine plus policy packs layered on it. The engine hardcodes nothing repo-specific; **all repo facts are policy-as-data**. The kernel contract is already latent in the gate fleet: `collect(root, policy) -> observed rows` + `evaluate(p

## Consequences

- Agent default read path: `docs/decisions/ADR-0xxx` apex files + this topic.
- Citations to member numbers remain valid via `docs/decisions/_disposition/adr-redirect.v1.json`.
- Further body merge refinements may land as amendments to this apex only.

### ADR-516 residual

**Agentic Delivery Fabric — the owned, cloud-native, productized unified delivery platform (apex vision + 5-component topo** — Adopt the **Agentic Delivery Fabric** as the apex product north-star: an owned, cloud-native, infinite-scale, productized platform that lets anyone automatically create and maintain hyperscaler-grade, well-architected, well-documented, well-maintained projects, with AI agents as the primary producers and quality built-in from project genesis and continuously auto-remediated. The fabric unifies SCM

### ADR-176 residual

**ADR-0176-brownout-degradation-signal-api** — ### D-1. Normative response header Every public HTTPS REST response emits the header below. Public webhooks, AsyncAPI events, SSE, and WebSocket messages carry the same value as protocol-appropriate metadata; internal gRPC responses carry it as response metadata without creating a public RPC contract: ``` oya-degradation-class: nominal|degraded|brownout|outage ``` Class semantics: | Class | Meanin

### ADR-90 residual

**ADR-0090-hyper-canonical-http-backbone** — Hyper 1.x is the canonical HTTP backbone for every µservice in the workspace. Direct deps for the runtime composition layer: | Crate | Version pin | Purpose | | ---------------- | ----------- | -------------------------------------- | | `hyper` | `1` | bare HTTP I/O protocol (h1/h2) | | `hyper-util` | `0.1` | tokio + auto-builder connection helper | | `tokio` | `1` | async runtime (rt-multi-thread

### ADR-13 residual

**ADR-0013-product-license-policy** — We adopt a **three-tier license policy** for product code with an explicit dev-only carve-out, a CI lane that hard-fails forbidden licenses, and a per-release SBOM generation requirement. ### Tier 1 — Allowed in product code (no review required) | License family | SPDX identifiers | |---|---| | Apache 2.0 | `Apache-2.0`, `Apache-2.0 WITH LLVM-exception` | | MIT | `MIT`, `MIT-0` | | BSD permissive

### ADR-249 residual

**ADR-0249-multi-category-marketplace-doctrine** — ### D-1. Eight marketplace substrate microservices (NEW, built day-one) The eight substrates are NEW first-class microservices under `microservices/` per ADR-0131 flat layout. Each is single-concern per ADR-0132. Each serves ALL marketplace categories (and the existing plugin-app-store after refactor). #### D-1.1 `microservices/catalog/` — typed product/listing entities **Concern:** Universal `Lis

### ADR-548 residual

**Pipeline as product: neutral ratchet engine + policy packs on the paved road** — ### D1 — The pipeline itself is a product: a neutral ratchet engine + policy packs The product is a NEUTRAL ratchet engine plus policy packs layered on it. The engine hardcodes nothing repo-specific; **all repo facts are policy-as-data**. The kernel contract is already latent in the gate fleet: `collect(root, policy) -> observed rows` + `evaluate(policy, observed) -> findings -> verdict`, with rat

### ADR-258 residual

**ADR-0258-api-versioning-model** — We adopt twelve interlocking decisions (D-1 through D-12) that together constitute the canonical oyatie API versioning model. Each decision is independently enforceable; the bundle composes into a coherent system. ### D-1 — External (public) APIs use Stripe-style request-time pinning via `X-Oyatie-API-Version` header Every public API surface (Workspace, Cloud, Intelligence, Verticals, Connect, Search,

### ADR-166 residual

**ADR-0166 — Schema Registry (Apicurio Registry; Confluent-compat API; AsyncAPI 3.x + proto3 + OpenAPI 3.1; backward-compa** — Oyatie adopts **Apicurio Registry 3.x** as the canonical schema registry. Properties: ### Schema kinds The registry holds: - **AsyncAPI 3.x** — event-driven contracts (eventing backbone per ADR-0005). One AsyncAPI doc per event-emitting µservice; per-event-class subject. - **Protobuf 3 (proto3)** — gRPC service definitions (per ADR-0145). One `.proto` file per gRPC service. - **OpenAPI 3.1** — RES

### ADR-314 residual

**ADR-0314-marketplace-as-universal-deal-settlement** — > **Disposition light-edit (2026-08-06):** Context re-triage Accept: Marketplace deal-settlement substrate

### ADR-354 residual

**ADR-0354-amendment-http3-fallback-strict-tls-ech-pqc** — ### §B-1 HTTP/3 Fallback Chain **Decision:** Every oyatie endpoint that accepts external traffic MUST implement the following protocol fallback chain in the listed priority order. Inter-cell and internal endpoints are declared through §B-8 and remain gRPC over HTTP/2 with SPIFFE mTLS until a real endpoint-specific pull justifies a transport-runtime adapter. ``` Priority 1 (default): HTTP/3 over QU

### ADR-19 residual

**ADR-0019-doc-catalog-and-update-protocol** — We adopt a **doc catalog as protocol** with a structured per-doc record, a five-stage lifecycle, an agent-authoring policy with explicit roles, and a machine-readable mirror at `machine-readable/catalog.json`. ### Per-doc catalog record Every consolidated doc has a row in `docs/DOC-CATALOG.md` and a mirror in `docs/machine-readable/catalog.json`: ```yaml doc_id: doc.privacy_program title: PRIVACY-

### ADR-132 residual

**ADR-0132-product-platform-and-bundle-dissolution** — Effective immediately and going forward, every new µservice in oyatie ships as a flat single-concern µservice under `microservices/<ms>/` per ADR-0131. The following patterns are **prohibited**: - Creating a new `microservices/<bundle>/` folder that contains more than one user-facing concern (e.g., `microservices/connector/`, `microservices/workspace/`, `microservices/healthcare/`, `microservices/

### ADR-1 residual

**ADR-0001-cohesion-thesis-one-product-flat-catalog** — We adopt the **cohesion thesis** as the foundational invariant of the Oyatie codebase, product, and roadmap: > Oyatie is one cohesive product across a flat catalog of shared microservices, joined at exactly six shared substrates: **single tenancy**, **single identity**, **single audit chain**, **single capability registry**, **single agent runtime**, and **single autonomy ceiling**. No microservic
