# ADR-0217: Vertical Slice Rollout Order

- **Status:** Accepted
- **Date:** 2026-05-18
- **Owner:** council-architecture
- **Deciders:** council-architecture, council-product, council-governance, axis-platform
- **Lane:** governance / substrate-doctrine
- **Supersedes:** none
- **Superseded by:** none
- **Related:** ADR-0211, ADR-0212, ADR-0215, ADR-0216, ADR-0218, ADR-0219, ADR-0220
- **Source:** `evidence/pr-143-session-decisions-checkpoint-2026-05-18.json#queued_adrs_to_author.ADR-0217`
- **Task:** #E substrate doctrines follow-up

## Context

PR #143 planned a large platform surface across substrate, product, compliance, integration, AI, workflow, ontology, and governance. Planning all of it is necessary so interfaces do not fight each other. Shipping all verticals in parallel would create thin sprawl: many shallow surfaces, few complete customer outcomes, and weak evidence for the hyperscaler-grade claim.

The checkpoint decision is depth before breadth. The platform may plan the full catalog, but production-GA rollout must proceed one vertical at a time. Each vertical should prove that the shared substrate, product shell, compliance pack, integration policy, no-code UX, tenant controls, and intelligence substrate work together under real customer pressure.

## Decision

Plan all microservices, but promote to production-GA one vertical at a time. A vertical is not "done" until it reaches hyperscaler-grade depth for its own workflows, compliance posture, integrations, import/export adapters, tenant controls, SLOs, runbooks, and audit evidence.

Canonical vertical rollout order:

| Order | Vertical | Rationale |
| --- | --- | --- |
| 1 | Enterprise Generic | Largest revenue surface, best cross-product integration test, medium compliance bar. |
| 2 | Healthcare | Highest early compliance pressure; validates HIPAA, Joint Commission, state DPH, Epic/Cerner FHIR connectors, and multi-context platform stress. |
| 3 | Retail / Commerce | Validates B2C marketplace, commerce stack, payments, and consumer surfaces at scale. |
| 4 | SMB Generic | Tests mass-market onboarding and ease-of-adoption claims after enterprise substrate hardens. |
| 5 | Manufacturing | Validates ERP integration, operational technology adjacency, and dense sector data models. |
| 6 | Logistics and Delivery | Validates route optimization, fleet workflows, and consent-graph real-time visibility. |
| 7 | Hospitality | Validates PMS integration, reservations, service operations, and workforce scheduling. |
| 8 | Financial Services | Validates the highest combined regulatory bar: KYC, AML, SOC 2, PCI-DSS, and fintech integrations. |
| 9 | Education | Validates LMS, campus recruiting, student/teacher/parent contexts, and FERPA-like boundaries. |
| 10 | Government | Validates procurement, public-sector controls, and FedRAMP High style trust posture; longest sales cycle, so last. |

### Gate for starting the next vertical

The next vertical may start only after the current vertical has:

- product PRD and phase specs at the ADR-0212 buildability bar;
- importer/exporter list per ADR-0216;
- tenant control requirements per ADR-0218;
- no-code UX surfaces per ADR-0219;
- intelligence scope per ADR-0220 where AI is in scope;
- compliance pack and threat model;
- SLOs, runbooks, CI lanes, audit-chain evidence, and rollback plan.

## In-house roadmap

This is operating doctrine, not a vendor-replaceable runtime. The roadmap lives in repo-native phase specs, product PRDs, and governance lanes. The rollout order can change only through a new ADR that names the vertical moved, the evidence that changed, and the impact on already-authored follow-up PRs.

Phase 1 ships Enterprise Generic depth. Phase 2 uses Healthcare as the stress test for multi-context, consent, FHIR, and compliance. Later phases reuse hardened substrate rather than restarting architecture.

## Alternatives considered

### Alternative 1 - Horizontal rollout across every vertical

**Rejected because** it optimizes demo breadth over production depth. A shallow healthcare, retail, finance, and government surface would create false signals and make every CI gate look green without proving any real workflow.

### Alternative 2 - Start with the easiest SMB vertical

**Rejected because** ease of onboarding is important but does not stress the substrate enough. Enterprise Generic surfaces more cross-product integration earlier while staying below the healthcare/finance regulatory ceiling.

### Alternative 3 - Start with the highest-compliance vertical

**Rejected because** Healthcare or Financial Services first would front-load the hardest regulatory work before the generic enterprise substrate has proven product cohesion. High-compliance verticals need a stable base.

### Alternative 4 - Let sales demand choose the next vertical

**Rejected because** opportunistic sales sequencing creates architecture whiplash. Sales input matters, but the platform order must preserve dependency and evidence discipline.

## Consequences

### Positive

- Each vertical can reach credible production depth before engineering attention splits.
- Shared substrate is hardened by real vertical requirements rather than theoretical completeness.
- Follow-up PRs can be sequenced predictably and reviewed with concrete acceptance gates.
- Customer-facing claims become evidence-backed per vertical.

### Negative

- Later verticals wait longer even if their market demand appears attractive.
- Some shared platform work must be planned before it is used by every vertical, which can feel slower early.
- Enterprise Generic becomes the first proof point and therefore carries high integration pressure.

### Operational

- Roadmap artifacts must tag work to the canonical vertical order.
- New vertical PRs must name the prior vertical's exit evidence or declare themselves pre-GA planning only.
- Reviewers should reject product PRs that claim GA depth without the gate evidence listed above.
- Parallel work is allowed for substrate and shared tooling, but GA claims remain sequential by vertical.

## Named industry sources

- AWS service launches: broad platform vision, but individual services reach GA with service-specific docs, limits, SLOs, and support paths.
- Stripe product sequencing: deep payments primitives precede adjacent financial products.
- Palantir Foundry deployments: high-value vertical depth and ontology modeling precede broad rollout.
- Microsoft Dynamics and M365: enterprise surface depth anchors later sector-specific clouds.
- Shopify: commerce depth first, then adjacent merchant services and ecosystem expansion.

## References

- ADR-0211: In-house tech stack policy ensures substrate choices do not block later verticals.
- ADR-0212: Buildability doctrine defines the artifact bar before a vertical can claim depth.
- ADR-0215: Multi-context platform is required before regulated or multi-role verticals.
- ADR-0216: Open integration and migration-out are required for every vertical product.
- ADR-0218: Tenant control surface is required before enterprise and regulated vertical GA.
- ADR-0219: No-code-first UX is required for professional users in every vertical.
- ADR-0220: Consumer Intelligence must remain separated from internal Foundry while serving product AI.
