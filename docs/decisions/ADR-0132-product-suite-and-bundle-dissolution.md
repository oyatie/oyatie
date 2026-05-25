---
id: ADR-0132
status: Accepted
planning_impact: true
deciders: council-architecture, council-engineering, axis-foundry, axis-workspace, ops-sre-reliability
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: []
related: [ADR-0056, ADR-0105, ADR-0110, ADR-0115, ADR-0119, ADR-0122, ADR-0127, ADR-0139, ADR-0131, ADR-0135]
related_specs: [/specs/per-microservice-flat-layout.json]
session_context:
  authored: 2026-05-17
  parallel_session_caveat: "Parallel session merges before this ADR. ADR numbers 0125–0129 are claimed by parallel work (OP-11 audit, Connect super-app, hyperscaler patterns, hyperscaler invariants, plan-schema). This ADR was renumbered 0126→0132 on 2026-05-17 to clear the collision. Connect-specific µservice topology is delegated to ADR-0135 (originally drafted as ADR-0126 in the oyatie 2026-05-17 session, renumbered 2026-05-18 to avoid collision with dev's ADR-0126 Employment classification); this ADR carries only the universal forward-policy."
bominal_source: "override — Bominal's bundled product groupings (suites, dual-context wrappers, vertical suites) are explicitly overridden by oyatie's flat-catalog doctrine per feedback_flat_product_catalog.md"
purpose: Forward-policy — no new suite, bundle, or vertical-grouping µservices anywhere in oyatie. Every new µservice ships flat as a single-concern µservice consistent with AWS / Google / Microsoft / Stripe per-service practice and with ADR-0131 (per-microservice flat layout).
---

# ADR-0132: No-suite forward-policy (universal flat catalog)

## Status

Accepted — 2026-05-17.

## Context

User directive 2026-05-17: *"we won't worry about suites yet. just have them as clean architecture flattened microservices"* and *"so no suites anywhere in our ecosystem. just microservices for now."*

Existing suite wrappers in the repo (Connect Suite, Workspace Productivity Suite, Enterprise Suite, Foundry, Workflow, Cloud) are **out of scope** for this ADR — they remain as authored until their owning teams refactor them on their own cadence. This ADR establishes only the forward-policy preventing new suite formations.

Connect's specific dissolution into seven flat sub-products (mail / messenger / calendar / social / shorts / network / anonymous) is owned by ADR-0135 (`connect-super-app-expansion`; originally drafted as ADR-0126 in the oyatie 2026-05-17 session, renumbered 2026-05-18 to avoid collision with dev's ADR-0126 Employment classification); this ADR cross-references that decision but does not re-author it.

Industry precedent: AWS / Google / Microsoft / Oracle / Stripe ship per-surface microservices, not per-suite bundles. "Suite" is a brand-layer concept (e.g., "Google Workspace" is a marketing brand for Gmail + Drive + Calendar + Docs + …), never an architecture-layer concept; each surface ships, scales, deploys, and SLOs independently.

## Decision

Effective immediately and going forward, every new µservice in oyatie ships as a flat single-concern µservice under `microservices/<ms>/` per ADR-0131. The following patterns are **prohibited**:

- Creating a new `microservices/<bundle>/` folder that contains more than one user-facing concern (e.g., `microservices/connect/`, `microservices/workspace/`, `microservices/healthcare/`, `microservices/fintech/`).
- Authoring a new `/specs/microservices/<bundle>-suite.json` collection-spec wrapper that binds multiple flat µservices (legacy `/specs/products/<bundle>/suite.json` path retired 2026-05-18 per the specs/products → specs/microservices flatten).
- Authoring a new PRD or phase-spec that declares its scope as "X Suite" or "X Platform" or "X Bundle" covering more than one concern.
- Authoring a new µservice whose name is an industry / vertical / sector / domain term (`healthcare`, `fintech`, `grc`, `ats`, `procurement`, `medical`, `financial-services`, etc.) when that µservice would contain more than one concern.

The following patterns are **mandatory**:

- One concern, one µservice, one `microservices/<ms>/` folder, one PRD.
- Cross-µservice composition flows through Workflow events and Ontology reads/writes, per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`.
- Categorization (e.g., `DomainTag = {Agentic, Dev, Business, Healthcare, SupplyChain, Delivery}` inside Workflow Studio) stays as metadata, NOT as a directory split.

Existing suite wrappers (Connect Suite, Workspace Productivity Suite, Enterprise Suite, Foundry, Workflow, Cloud) are not touched by this ADR. Their refactor — if any — is owned by their respective teams under separate ADRs.

## Rejected alternatives

- **Dissolve all existing suites in this ADR** — rejected per user directive 2026-05-17 ("we won't worry about suites yet"); out of scope. Refactor of existing suite µservices is scheduled-for-distinct-tracked-work to per-team decisions.
- **Retire suite as a brand-layer concept too** — rejected. Marketing / GTM may still use "Workspace" or "Connect" as a brand name; this ADR governs architecture, not brand.
- **Allow new bundle µservices when the bundle has a single SLO target** — rejected. A single SLO across multiple user-facing concerns means one concern's regression breaks the others' SLO; that is the failure mode this ADR prevents.

## Consequences

### Positive

- New µservices land in a uniform shape; no bundle/suite drift.
- Per-concern SLO + release pointer (per ADR-0139) is the universal pattern, not the exception.
- **Independent scaling.** Each concern scales on its own dimension without dragging unrelated concerns along: mail scales on mailbox-count + inbound-message-rate; calendar scales on event-write-rate; messenger scales on persistent-connection-count + message-rate; slo-engine scales on µservice-count × evaluator-cadence; ehr (future) scales on patient-count + FHIR-write-rate; payments (future) scales on transaction-rate + fraud-eval-rate. A bundled "Connect" or "Workspace" µservice would force these to scale together — wasting capacity on under-utilised surfaces while under-provisioning the bottleneck. Clean-architecture + flat-catalog enables horizontal-scaling efficiency at hyperscaler-grade. Per-pod HPA, per-µservice Mimir cardinality budgets, per-µservice cost budgets all flow from this.
- Future migrations of existing suites have a clear target shape (this ADR's flat-µservice mandate); their per-team refactor ADRs reference this one as the destination.
- Forward-policy is enforceable as a CI lane.

### Negative

- The existing suite µservices remain inconsistent with the forward-policy. Reviewers reading old suite PRDs may be confused about which doctrine applies; the resolution is "ADR-0132 forward-policy applies to new µservices; old suites retain their original doctrine until refactored."

### Migration cost

This ADR adds zero migration cost: existing suite µservices are explicitly out of scope (per 2026-05-17 user directive "we won't worry about suites yet"). The cost is exclusively forward-prevention (one new BLOCKER CI lane). Per-µservice refactor costs, if and when scheduled by their owning team, follow the per-µservice migration tooling at `/specs/microservice-migration-tooling.json` and the cost classes in ADR-0131 §"Migration cost quantification".

### Operational

- **New CI lane: `oya-governance-no-new-suite-bundles`** (BLOCKER on `dev`). Refuses:
  - New `microservices/<name>/` folder whose name matches a banned bundle/industry/vertical pattern AND which contains more than one user-facing concern.
  - New `/specs/microservices/<name>-suite.json` files (legacy `/specs/products/<name>/suite.json` path retired 2026-05-18 per the specs/products → specs/microservices flatten).
  - New PRDs whose `microservice` frontmatter value matches a banned-pattern term and whose scope covers >1 concern.
  Pattern detail in `/specs/per-microservice-flat-layout.json` §`validator_rules`.
- Existing suite files (`/specs/microservices/connect-suite.json`, `/specs/microservices/enterprise-suite.json`, ADR-0029, etc. — relocated 2026-05-18 from `/specs/microservices/{connect,enterprise}/suite.json` per the specs/products → specs/microservices flatten) are explicitly exempted by allowlist in the lane until their owning team refactors them.

## Clean Architecture Impact

This ADR is structural / policy; layer assignments, dependency direction, and port-location rules are unchanged. ADR-0131 (per-microservice flat layout) remains the authority for folder shape; this ADR is its sibling that forbids new suite-shaped µservices.

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | none |
| `per-microservice-layout` (per ADR-0131) | Reinforced | this ADR's `no-new-suite-bundles` lane is a sibling enforcement |
| `no-new-suite-bundles` (NEW) | New BLOCKER on dev | refuses suite-shaped new µservices |

## Verification

- `cargo run -p oya-dev-cli -- gate validate no-new-suite-bundles` — exit 0; no banned bundle names introduced by new µservices.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout` — exit 0 (ADR-0131 sibling lane).
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0.

## References

- ADR-0056: BNF v4.1 naming.
- ADR-0105: 13-layer enum.
- ADR-0110: ChangeSet state machine.
- ADR-0135 (originally drafted as ADR-0126 in the oyatie 2026-05-17 session; renumbered 2026-05-18 to avoid collision with dev's ADR-0126 Employment classification): Connect-specific dissolution — full social-network super-app expansion; this ADR cross-references but does not re-author.
- ADR-0127 (parallel session 2026-05-17): Portfolio hyperscaler pattern enforcement.
- ADR-0139: Agentic SLO-gated promotion (per-microservice release pointers depend on this ADR's flat-µservice mandate).
- ADR-0131: Per-microservice flat layout (the layout authority; this ADR is its enforcement sibling for suite-prevention).
- `feedback_flat_product_catalog.md`: "Everything is shared; flat product catalog."
- `feedback_glossary_shared_not_platform.md`: retirement of "platform" (= suite by another name) terminology.
- Industry: AWS / Google / Microsoft / Oracle / Stripe per-service shipping precedent.
