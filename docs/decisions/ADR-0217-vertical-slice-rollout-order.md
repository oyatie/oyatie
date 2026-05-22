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
| 1 | Enterprise Generic + SMB Generic | First deliverable. Must exit in full production depth, not MVP/preview/reduced scope. Required scope includes core, messenger, mail, community, infra, Ops Dashboard / Control Center, Foundry, Workflow, Ontology, canonical base, and Korea localization pack. Enterprise proves complex integration pressure; SMB proves mass-market onboarding and ease-of-adoption against the same substrate before sector-specific fan-out. |
| 2 | Healthcare | Highest early compliance pressure; validates HIPAA, Joint Commission, state DPH, Epic/Cerner FHIR connectors, and multi-context platform stress after the generic product substrate is proven. |
| 3 | Retail / Commerce | Validates B2C marketplace, commerce stack, payments, and consumer surfaces at scale. |
| 4 | Manufacturing | Validates ERP integration, operational technology adjacency, and dense sector data models. |
| 5 | Logistics and Delivery | Validates route optimization, fleet workflows, and consent-graph real-time visibility. |
| 6 | Hospitality | Validates PMS integration, reservations, service operations, and workforce scheduling. |
| 7 | Financial Services | Validates the highest combined regulatory bar: KYC, AML, SOC 2, PCI-DSS, and fintech integrations. |
| 8 | Education | Validates LMS, campus recruiting, student/teacher/parent contexts, and FERPA-like boundaries. |
| 9 | Government | Validates procurement, public-sector controls, and FedRAMP High style trust posture; longest sales cycle, so last. |

### Gate for starting the next vertical

The next vertical may start only after the current vertical has:

- product PRD and phase specs at the ADR-0212 buildability bar;
- importer/exporter list per ADR-0216;
- tenant control requirements per ADR-0218;
- no-code UX surfaces per ADR-0219;
- intelligence scope per ADR-0220 where AI is in scope;
- flat microservice boundaries, clean architecture layer boundaries, API-first contracts, hyperscaler pattern mappings, and independent horizontal scaling strategy before implementation handlers land;
- compliance pack and threat model;
- SLOs, runbooks, CI lanes, audit-chain evidence, and rollback plan.
- a production-grade exit record proving there were no deferrals, scope reductions, placeholders, stubs, or thin scaffolds inside the vertical's accepted scope.
- canonical base plus Korea localization pack readiness evidence for the first deliverable before sector-specific vertical fan-out begins.
- reproducible cloud-native Kubernetes deployment evidence through one-command or one-click setup across Talos, Ubuntu LTS, Debian, Fedora Server, Oracle Linux, RHEL-compatible distributions, CentOS Stream, Rocky Linux, AlmaLinux, SUSE Linux Enterprise, and macOS Apple Silicon; product code must not assume a host distribution or desktop OS.
- one-time script or one-click setup evidence proving any supported machine that meets declared prerequisites can securely join the production cluster as a secured, hardened, policy-compliant member with observability and audit enrollment, or fail closed with an actionable evidence report.
- remote config-driven secure cluster join evidence for Talos-class nodes, with signed policy-validated config, externalized secrets, identity proof, and auditable join evidence.
- distroless or scratch production image posture by default, with fuller base images allowed only through explicit exception evidence, SBOM, image-size and vulnerability budgets, and a removal/optimization follow-up.
- Ops Dashboard / Control Center evidence at full vertical depth, covering incident response, deployment control, cluster/node health, tenant lifecycle, tenant isolation posture, policy/compliance decisions, audit trails, SLO/error-budget state, evidence packs, bootstrap/recovery workflows, localization/escalation runbooks, and safe operator actions with approval and rollback.
- evidence-backed claim controls proving no empty promises, false green signals, or silent regressions in behavior, performance, policy, schema, API, tenant isolation, observability, or auditability.
- development-pipeline evolution controls proving phase-appropriate agent skills, CI gates, review loops, and regression baselines ratchet with the masterplan and the product.
- automation-first development-cycle controls proving automatable work is automated and any manual exception has evidence, owner, expiry trigger, and automation follow-up.

## In-house roadmap

This is operating doctrine, not a vendor-replaceable runtime. The roadmap lives in repo-native phase specs, product PRDs, and governance lanes. The rollout order can change only through a new ADR that names the vertical moved, the evidence that changed, and the impact on already-authored follow-up PRs.

Phase 1 ships Enterprise Generic + SMB Generic at full production depth. The first deliverable includes core, messenger, mail, community, infra, Ops Dashboard / Control Center, Foundry, Workflow, Ontology, canonical base, and Korea localization pack. Flat microservices, clean architecture, API-first architecture, independent horizontal scaling, and hyperscaler patterns are entry criteria for implementation packets, not cleanup passes after handlers exist. Every deployable must have reproducible cloud-native Kubernetes setup evidence for major enterprise Linux families and macOS Apple Silicon before exit; setup must be one-command or one-click, backed by OpenTofu/IaC, GitOps, multi-arch OCI images, SBOM/provenance, conformance, local Kubernetes/OCI parity, one-time secure bootstrap, remote config-driven Talos cluster join, production hardening, and restore evidence. Production images default to distroless or scratch where technically viable. Empty promises, false green signals, and silent regressions are release blockers, not review comments. The development pipeline must evolve with the project: phase-appropriate agent skills, CI gates, review loops, performance baselines, and regression detectors ratchet as the masterplan changes. Automatable development-cycle work must be automated; manual exceptions require evidence, owner, expiry trigger, and automation follow-up. It is explicitly not an MVP, preview, or reduced-scope launch; exit means the generic product is ready to support honest industry-leading and hyperscaler-grade claims for that scope. Phase 2 uses Healthcare as the stress test for multi-context, consent, FHIR, and compliance. Later phases reuse hardened substrate rather than restarting architecture.

## Alternatives considered

### Alternative 1 - Horizontal rollout across every vertical

**Rejected because** it optimizes demo breadth over production depth. A shallow healthcare, retail, finance, and government surface would create false signals and make every CI gate look green without proving any real workflow.

### Alternative 2 - Start with SMB as a later standalone vertical

**Rejected because** SMB onboarding is not optional polish; it is part of the first generic-product proof. Shipping Enterprise Generic first and postponing SMB would let the plan defer ease-of-adoption, self-service onboarding, and low-touch operations. SMB is therefore bundled with Enterprise Generic in the first deliverable while sharing the same core, messenger, mail, community, infra, Ops Dashboard / Control Center, Foundry, Workflow, Ontology, canonical base, and Korea localization pack substrate.

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
- Enterprise Generic + SMB Generic becomes the first proof point and therefore carries both high integration pressure and high ease-of-adoption pressure.

### Operational

- Roadmap artifacts must tag work to the canonical vertical order.
- New vertical PRs must name the prior vertical's exit evidence or declare themselves pre-GA planning only.
- Reviewers should reject product PRs that claim GA depth without the gate evidence listed above.
- Parallel work is allowed for substrate and shared tooling, but GA claims remain sequential by vertical.
- The first deliverable's planning and implementation packets must explicitly name core, messenger, mail, community, infra, Ops Dashboard / Control Center, Foundry, Workflow, Ontology, canonical base, and Korea localization pack as required full-depth production scope.
- The first deliverable's implementation packets must prove flat microservice boundaries, clean architecture inward dependency direction, API-first contract existence before handlers, independent horizontal scaling and backpressure design, and mapping to the hyperscaler architecture invariant catalog.
- The first deliverable's gates must reject empty coverage, aspirational done states, and silent regressions through baseline-diff evidence and non-empty claim evidence.
- The first deliverable's gates must reject host-distro lock-in, mutable-host snowflakes, missing macOS Apple Silicon parity, missing multi-arch OCI evidence, and setup paths that require undocumented manual steps instead of one-command or one-click bootstrap.
- The first deliverable's gates must reject setup scripts that partially mutate hosts after failed prerequisites, cluster joins without signed config and externalized secrets, missing hardening evidence, and production images that use fuller base layers without exception evidence.
- The first deliverable's pipeline must prove phase-appropriate use of agent skills and ratchet CI, review, performance, and regression gates when implementation evidence reveals a new class of risk.
- The first deliverable's pipeline must automate every repeatable development-cycle control, including contract validation, boundary checks, impacted-test mapping, CI-fix context bundles, evidence-pack generation, and blocker audits; manual exceptions expire into automation ratchet work.

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
