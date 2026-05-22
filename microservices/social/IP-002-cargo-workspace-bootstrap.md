---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-002-cargo-workspace-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-per-microservice-layout]
---

# IP-002: Social Cargo workspace bootstrap

## A. Problem
Social has many bounded contexts and adapters; without a workspace scaffold, later IPs will invent incompatible crate locations and layer boundaries.

## B. Approach
Create the per-microservice flat Cargo workspace and only the crate paths already named by the PRD, manifest, catalog, or this IP set. Keep product integration through contracts, Workflow, and Ontology rather than direct product-crate dependencies.

## C. Deliverables
| Artifact | Role |
|---|---|
| `manifest.json` | Machine-readable crate and contract source. |
| `catalog/oya-social-*.yaml` | Existing crate inventory anchors. |
| `src/crates/oya-social-app/` | Planned composition root named by catalog. |
| `src/crates/oya-social-{user-profile,follow-graph,post-composition,feed-timeline,content-moderation,search}-*/` | Planned crate families already named by PRD/catalog/IPs. |

## D. Ordered implementation steps
1. Create workspace manifests with one package per named catalog crate.
2. Add minimal lib/bin targets with no business behavior.
3. Configure shared lint, test, and feature conventions already used by this repo.
4. Add compile-only dependency direction tests.
5. Register workspace members without adding unnamed crates.
6. Run cargo check over the social workspace.
7. Run per-microservice layout and layer-correctness gates.

## E. Acceptance
- `cargo check --workspace` scoped to social packages passes.
- `cargo nextest run --workspace` scoped to social packages passes for bootstrap tests.
- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice social` passes.
- `cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice social` passes.
- Manifest and catalog crate names match workspace members.

## F. Evidence
- Crate source: `manifest.json`, `catalog/`.
- PRD bounded contexts: `PRD.md`.
- Contracts: `contracts/openapi/social.yaml`, `contracts/asyncapi/social-events.yaml`, `contracts/proto/social.proto`.

## G. Counterpart comparison
X, Instagram, TikTok, and Snapchat are monolithic from a buyer's view; Mastodon and Bluesky expose more modular protocol surfaces. Oyatie's workspace must support modular, auditable service boundaries while preserving a first-party social product experience.

## H. Foundation delivery expansion
- Deliverable detail: workspace members map one-to-one with manifest and catalog crate names.
- Deliverable detail: crate families stay flat under the social service rather than a shared monolith.
- Deliverable detail: bootstrap targets include lib/bin shells only where the catalog requires them.
- Deliverable detail: dependency direction tests encode kernel, domain, usecase, adapter, rest, worker, and app boundaries.
- Deliverable detail: contracts, Workflow, and Ontology remain integration surfaces across products.
- Deliverable detail: CI uses social-scoped cargo package filters so unrelated workspace churn is not required.
- Deliverable detail: empty behavior stubs include TODO references to exact IP ids.
- Deliverable detail: Slack app-directory and community integrations are pressure for clean contract boundaries.

## I. Acceptance expansion
- Acceptance detail: manifest/catalog names must match package names exactly.
- Acceptance detail: cargo check must run on social packages without pulling unrelated product crates.
- Acceptance detail: layer tests must fail on adapter imports from kernel/domain crates.
- Acceptance detail: workspace registration must not introduce unnamed helper crates.
- Acceptance detail: contract paths must remain reachable from the social manifest.
- Acceptance detail: generated package list must include profile, graph, post, feed, moderation, search, and app crates.
- Acceptance detail: branch promotion must include crate list evidence.
- Acceptance detail: Slack, GitHub, and Linear-style integration pressure must be handled through contracts and work items, not direct imports.

## J. Evidence expansion
- Evidence detail: capture social-scoped `cargo check` output.
- Evidence detail: capture per-microservice-layout gate output.
- Evidence detail: capture lean-a1 or layer-correctness output for the social workspace.
- Evidence detail: cite `manifest.json` and `catalog/` as the crate inventory sources.
- Evidence detail: cite `contracts/openapi/social.yaml`, `asyncapi/social-events.yaml`, and `proto/social.proto`.
- Evidence detail: cite `PRD.md` bounded contexts for package scope.
- Evidence detail: cite Slack as integration-pressure evidence that validates modular service boundaries.
