---
doc_status: published
---

# Team: Axis — SaaS Multi-Tenant Platform

## Mission
This team owns the SaaS multi-tenant platform axis: the workflow engine, Object Graph, plugin substrate and marketplace, Bench (collaborative workspace), and partner surface. It exists to make Oyatie the operating system for tenant businesses — the layer that end users touch every day, shaped per vertical by the vertical teams but authored and governed here. It does **not** own the underlying cloud infrastructure (→ `axis-cloud`), the agent runtime (→ `axis-foundry`), or per-vertical domain logic beyond what is shared across all verticals.

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
  - Products owned: `products/saas-platform/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Object Graph property tier` (co-owner with `platform-eventing-og`, `platform-privacy-dub`) — OG shape changes are cross-axis
  - `Marketplace listing` (co-owner with `axis-foundry`) — plugin signing + sandbox gate
  - `Billing event` (co-owner with `axis-cloud`) — SaaS metering side
  - `Webhook delivery + signing` (consumer of `platform-api-sdk` — SaaS-authored webhooks)
  - `Public REST stability tier` (consumer — SaaS surface slice)
- **Catalog records:** `crates/oya-saas-*`, `crates/oya-platform-forms-*`, `crates/oya-platform-metering-*`, `crates/oya-platform-web-*`
- **Runbooks:** `runbooks/workflow-engine-restart.md`, `runbooks/plugin-sandbox-escape.md`, `runbooks/marketplace-listing-takedown.md`
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
- Agent runtime (→ `axis-foundry`)
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
| `axis-foundry` | Capability invocation API for agent-authored workflows | Wave gate |
| `axis-cloud` | Compute cells for workflow execution, storage for OG | Wave gate |
| `platform-api-sdk` | Webhook delivery, public REST stability tier | Per-release |
| `axis-foundry` | Catalog gate for plugin signing, fitness functions | Per-release |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| All vertical teams | Workflow engine, OG, plugin substrate, metering | Per vertical onboard |
| `axis-search` | OG-indexed content (via consent tier) | Monthly |
| `axis-ads-analytics` | Tenant-consented data classes for ad targeting | Wave gate |
| `gtm-sales-se` | Demo environment, pilot tenant workflows | Monthly |
| `gtm-customer-success` | Tenant health dashboards, workflow analytics | Monthly |

## Success metrics
- **Tenant onboarding + plugin install + marketplace listing all functional:** PRD §4.1 W-SaaS-Preview gate
- **Workflow execution p99 latency:** < 500 ms for synchronous steps
- **Plugin sandbox escape incidents:** 0
- **Marketplace plugin listing review turnaround:** ≤ 5 business days
- **OG schema evolution backward-compatibility violations:** 0 at merge
- **Metering event completeness:** 100% of billable events emitted

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council (`teams/council-architecture/CHARTER.md`) for OG contract changes
- Privacy: privacy council for OG tier → data-class disputes
- Founder: as last resort

## Communication cadence
- Stand-up: daily async
- Weekly: 60-min sync — workflow engine queue, plugin review backlog, marketplace metrics
- Cross-team review: monthly cross-axis contract audit for OG and marketplace contract changes

## Bandwidth + hiring
- Current FTE: TBD
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; OG contract PRs require cross-axis label
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Plugin sandbox escape allows cross-tenant data access | Catastrophic | WASM/container isolation; `ops-security` review on plugin substrate changes |
| Workflow engine single-point-of-failure | High | Cell-routed stateless execution; per-tenant partition |
| OG schema migration corrupts tenant data | High | Backward-compatible migration policy; blue-green migration gate |
| Marketplace fraudulent plugin listed | Medium | Plugin signing + review pipeline; automated static analysis |

## Sources scanned
PRD.md §2, §3.1 (W-SaaS-Preview), DESIGN.md §1 (Axis 1), §10 (OG tier, marketplace, billing event rows), products/saas-platform/PRD.md (draft), DOC-CATALOG.md §2.5.
