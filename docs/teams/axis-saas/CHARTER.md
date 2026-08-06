---
doc_status: published
---

# Team: Axis — SaaS Multi-Tenant Platform

## Mission
This team owns the SaaS multi-tenant platform axis: the workflow engine, Object Graph, plugin substrate and marketplace, Bench (collaborative workspace), and partner surface. It exists to make Oyatie the operating system for tenant businesses — the layer that end users touch every day, shaped per vertical by the vertical teams but authored and governed here. It does **not** own the underlying cloud infrastructure (→ `axis-cloud`), model/provider execution (→ `cloud-intelligence`), tenant intelligence servicing (→ `oya-intelligence`), or per-vertical domain logic beyond what is shared across all verticals.

## Owned axes / surfaces / contracts
- **Axis(es):** SaaS multi-tenant (Axis 1)
- **Surfaces:**
  - `oya-saas-workflow-sdk-kernel` / `oya-saas-workflow-sdk-app` — workflow definition, step types, saga orchestration
  - `oya-saas-workflow-engine-*` — execution runtime (ADR-0035)
  - `oya-saas-plugin-kernel` / `oya-saas-plugin-app` — plugin signing, sandbox, lifecycle
  - `oya-saas-marketplace-kernel` — marketplace listing, ISV onboarding, revenue share
  - `oya-platform-forms-kernel` / `oya-platform-forms-app` — form builder (split from `forms` crate)
  - `oya-platform-metering-kernel` — billing event emission for SaaS surfaces
  - `oya-platform-web-kernel` / `oya-platform-web-app` — SaaS web layer
  - Bench: collaborative workspace surface (uses OG + workflow + identity)
  - Connect: partner integration surface
  - Product PRD: `docs/products/saas-platform/PRD.md` (planning-closed contract authored; live readiness requires changeset gate evidence)
- **Cross-axis contracts (DESIGN §10):**
  - `Object Graph property tier` (co-owner with `platform-eventing-og`, `platform-privacy-dub`) — OG shape changes are cross-axis
  - `Marketplace listing` (co-owner with `cloud-intelligence`, `oya-intelligence`, and central governance) — plugin signing + sandbox gate
  - `Billing event` (co-owner with `axis-cloud`) — SaaS metering side
  - `Webhook delivery + signing` (consumer of `platform-api-sdk` — SaaS-authored webhooks)
  - `Public REST stability tier` (consumer — SaaS surface slice)
- **Catalog records:** current registry-store package records for `oya-saas-workflow-kernel`, `oya-saas-workflow-domain`, `oya-saas-workflow-app`, `oya-saas-plugin-app`, `oya-saas-plugin-runtime`, `oya-saas-plugin-marketplace`, `oya-saas-plugin-marketplace-kernel`, `oya-platform-forms-*`, `oya-platform-metering-*`, and `oya-platform-web-*`.
- **Runbooks:** `docs/runbooks/saas/workflow-engine-deadlock.md`, `docs/runbooks/saas/plugin-runtime-sandbox-escape.md`, `docs/runbooks/saas/marketplace-listing-takedown.md`
- **ADRs:** ADR-0035 (workflow engine), ADR-0006..0112 (OG — co-author with `platform-eventing-og`)

## In-scope work
- Workflow engine: step types, conditional branching, loops, human-in-the-loop steps, agent-authored workflows
- Workflow Studio: the visual authoring UI surface for tenant builders
- Object Graph: node/edge/property authoring, schema evolution (seam contract owned with `platform-eventing-og`)
- Plugin substrate: signing key verification, WASM/container sandbox, lifecycle (install, update, revoke)
- Plugin marketplace: ISV listing, review pipeline, revenue share, tenant installation
- Bench: collaborative workspace (real-time co-editing, presence, notifications)
- Connect: partner API surface for external integrators (uses `platform-api-sdk` for delivery)
- SaaS metering: usage events (per-seat, per-workflow-run, per-plugin-invocation) → `oya-platform-metering-kernel`
- Tenant onboarding UX (control-plane — onboarding logic is `platform-tenancy-identity`)
- Per-vertical workflow template library (templates authored here; domain logic in vertical teams)

## Out-of-scope (anti-scope)
- Cloud infrastructure hosting SaaS (→ `axis-cloud`)
- Model/provider execution and tenant intelligence servicing (→ `cloud-intelligence` / `oya-intelligence`)
- Per-vertical FHIR/EDI/ISA-95 domain logic (→ per-vertical teams; SaaS provides the workflow substrate)
- Public API gateway infrastructure (→ `platform-api-sdk`)
- Audit chain infrastructure (→ `platform-audit-evidence` — SaaS emits but doesn't own the chain)
- Search engine (→ `axis-search`)
- Advertising (→ `axis-ads-analytics`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-tenancy-identity` | Tenant/identity kernel, RBAC enforcement | Per-release |
| `platform-eventing-og` | OG property-tier schema, outbox relay | Per-release |
| `platform-privacy-dub` | Data Use Boundary check on OG tier changes | Per OG schema change |
| `cloud-intelligence` / `oya-intelligence` | Capability invocation API for agent-authored workflows | Wave gate |
| `axis-cloud` | Compute cells for workflow execution, storage for OG | Wave gate |
| `platform-api-sdk` | Webhook delivery, public REST stability tier | Per-release |
| central governance | Catalog gate for plugin signing and contract fitness functions | Per-release |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| All vertical teams | Workflow engine, OG, plugin substrate, metering | Per vertical onboard |
| `axis-search` | OG-indexed content (via consent tier) | Monthly |
| `axis-ads-analytics` | Tenant-consented data classes for ad targeting | Wave gate |
| `gtm-sales-se` | Controlled evaluation tenants, sales-engineering validation workflows, and evidence-backed pilot workflow packs | Monthly |
| `gtm-customer-success` | Tenant health dashboards, workflow analytics | Monthly |

## Success metrics
- **Tenant onboarding + plugin install + marketplace listing all functional:** M03-P04/M03-P08 changeset evidence plus branch-protected `oya-ci-required` gate; readiness remains `target_non_claim` until that evidence is green.
- **Workflow execution p99 latency:** < 500 ms for synchronous steps
- **Plugin sandbox escape incidents:** 0
- **Marketplace plugin listing review turnaround:** ≤ 5 business days
- **OG schema evolution backward-compatibility violations:** 0 at merge
- **Metering event completeness:** 100% of billable events emitted

## Escalation path
- Internal: tech lead → team manager
- Cross-team: founder-governed architecture review for OG contract changes
- Privacy: `platform-privacy-dub` and central governance for OG tier → data-class disputes
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 60-min sync — workflow engine queue, plugin review backlog, marketplace metrics
- Cross-team review: monthly cross-axis contract audit for OG and marketplace contract changes

## Bandwidth + hiring
Capacity is tracked outside this repository in the staffing system and is not a product-readiness signal. SaaS readiness is gated by M03-P04/M03-P08 functional, security, SLO, runbook, and `oya-ci-required` evidence rather than staffing-count assertions.

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; OG contract PRs require cross-axis label
- PR shape: 5-section H2 template
- Readiness authority: branch-protected `oya-ci-required`; workstation diagnostics may help authors but are never merge, production, or hyperscaler authority.
- ADR proposal cadence: monthly batch

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Plugin sandbox escape allows cross-tenant data access | Catastrophic | WASM/container isolation; `ops-security` review on plugin substrate changes |
| Workflow engine single-point-of-failure | High | Cell-routed stateless execution; per-tenant partition |
| OG schema migration corrupts tenant data | High | Backward-compatible migration policy; blue-green migration gate |
| Marketplace fraudulent plugin listed | Medium | Plugin signing + review pipeline; automated static analysis |

## Sources scanned
docs/products/README.md SaaS Platform entry, docs/products/saas-platform/PRD.md, specs/masterplan.json M03-P04/M03-P08 references, docs/adr-archive/ADR-0035-workflow-engine-state-machine-and-dag-hybrid.md ADR-0036, ADR-0249, ADR-0314, registry/stores/* product inputs, specs/root-hub-pointers.json, the HANDOFF.md thin redirect, and DOC-CATALOG.md §2.5 (legacy projection pending PHASE-5 machine-catalog promotion).
