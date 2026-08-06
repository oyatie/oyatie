---
id: ADR-0119
status: Superseded
deciders: council-architecture, council-foundry-vcs
date: 2026-05-16
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
amended_by:
  - ADR-0131-per-microservice-flat-layout.md (partial product-owned-spec colocation only; the flat cross-cutting specs root remains binding)
related:
  - ADR-0115-registry-consolidation-flat-singular.md
  - ADR-0116-retire-external-agent-coordination-tooling.md
  - ADR-0121-onprem-k8s-stack-kubeadm-containerd-istio-envoy.md
purpose: Retire the former nested spec scope directory by hoisting its machine-readable specifications to the flat `specs/` root, preserving history through git moves and aligning spec topology with the Foundry pipeline contribution substrate.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0119: Specs flat-root topology

## Context

ADR-0115 established the current topology cleanup pattern: use a flat canonical root, keep the semantic basename, and remove redundant `cross-cutting/` scope directories when the scope is already the omitted default. The same sprawl existed under the former nested spec scope directory: the repository had no sibling scoped spec roots, so every lookup paid a redundant path segment without adding routing information.

ADR-0116 also changed the contribution substrate: the Foundry pipeline (M01-P18) is the canonical VCS substrate, and external coordination tools are retired. This cleanup therefore uses plain `git mv` in an isolated worktree branch and enters the Foundry path through a PR against `dev`; no retired coordination primitive participates in the topology decision.

## Decision

`specs/` is the canonical flat root for machine-readable specifications. The former nested spec scope directory is retired. All prior children of that retired directory are hoisted to `specs/`, while the typed lifecycle-config family remains grouped at `specs/lifecycle-configs/`.

All live references to the retired nested path are rewritten to `specs/`. Historical prose may still use "cross-cutting" as a milestone or concern name, but it is no longer a spec path segment.

## Naming justification

| New symbol / path | One-line justification |
|---|---|
| `specs/` | Flat plural root because it houses many machine-readable specifications and matches ADR-0115's flat-root pattern without the redundant implicit scope token. |
| `specs/lifecycle-configs/` | Lifecycle configs stay grouped because the child files are a typed family consumed together by lifecycle fitness lanes, while the group now hangs directly under `specs/`. |
| `docs/decisions/ADR-0119-specs-flat-root-topology.md` | ADR filename uses the next free ADR number after the archive-orphan retirement ADR plus kebab summary `specs-flat-root-topology` to name the topology decision, not the mechanical move. |
| `specs/active-machine-readable-artifact-contract.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/artifact-profile-defaults.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/ci-fix-loop-context-bundle.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/codeview-read-surface.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/crate-naming-audit.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/decision-principles.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/decision-rights.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/evidence-taxonomy.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/final-report-schema.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/forbidden-operations.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/gitops-vcs-replacement.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/governance-amendment.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/hyperscaler-gates.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/iterative-fix-loop.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/knowledge-graph-schema.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/lifecycle-configs/adr-status-lifecycle.json` | Preserves the lifecycle config basename and removes only the redundant `cross-cutting` scope segment; the `lifecycle-configs/` parent supplies the family context. |
| `specs/lifecycle-configs/api-stability-tier-lifecycle.json` | Preserves the lifecycle config basename and removes only the redundant `cross-cutting` scope segment; the `lifecycle-configs/` parent supplies the family context. |
| `specs/lifecycle-configs/capability-status-lifecycle.json` | Preserves the lifecycle config basename and removes only the redundant `cross-cutting` scope segment; the `lifecycle-configs/` parent supplies the family context. |
| `specs/lifecycle-configs/crate-status-lifecycle.json` | Preserves the lifecycle config basename and removes only the redundant `cross-cutting` scope segment; the `lifecycle-configs/` parent supplies the family context. |
| `specs/lifecycle-configs/dependency-status-lifecycle.json` | Preserves the lifecycle config basename and removes only the redundant `cross-cutting` scope segment; the `lifecycle-configs/` parent supplies the family context. |
| `specs/lifecycle-configs/doc-status-lifecycle.json` | Preserves the lifecycle config basename and removes only the redundant `cross-cutting` scope segment; the `lifecycle-configs/` parent supplies the family context. |
| `specs/lifecycle-configs/feature-flag-status-lifecycle.json` | Preserves the lifecycle config basename and removes only the redundant `cross-cutting` scope segment; the `lifecycle-configs/` parent supplies the family context. |
| `specs/lifecycle-configs/migration-status-lifecycle.json` | Preserves the lifecycle config basename and removes only the redundant `cross-cutting` scope segment; the `lifecycle-configs/` parent supplies the family context. |
| `specs/lifecycle-configs/plan-status-lifecycle.json` | Preserves the lifecycle config basename and removes only the redundant `cross-cutting` scope segment; the `lifecycle-configs/` parent supplies the family context. |
| `specs/markdown-retirement-policy.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/master-plan-sequencing.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/masterplan.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/merge-queue-parked-pr.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/multispectrum-review.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/oyatie-doctrine.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/plan-schema.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/root-hub-pointers.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/stop-conditions.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |
| `specs/test-standard.json` | Preserves the existing spec basename and hoists it under the canonical flat `specs/` root; cross-cutting is the omitted default scope. |

## Migration

- Move every file from the former nested spec scope directory to `specs/` with `git mv` so history is preserved.
- Move the former nested lifecycle-configs path to `specs/lifecycle-configs/` as the only retained grouping because those files are one schema-governed family.
- Remove the empty retired nested spec scope directory.
- Rewrite retired nested spec path references to the flat `specs/` form across tracked text files.

## Consequences

### Positive

- Agents have one canonical spec root and one fewer path token to remember.
- The specs root now mirrors ADR-0115's registry consolidation: flat canonical root, semantic direct children, no redundant scope directory.
- Foundry pipeline admission sees a simpler topology for future spec edits and path-based checks.

### Negative

- This is a broad reference rewrite, so reviewers should focus on mechanical path correctness rather than semantic content changes.
- Historical evidence files that contained live path strings are rewritten to the new path, which keeps current lookups green but reduces literal preservation of old command examples.

### Operational

- Future cross-cutting machine-readable specs are added at `specs/<concept>.json` unless they belong to an existing typed family such as `specs/lifecycle-configs/`.
- PRs for this topology continue through the Foundry pipeline (M01-P18): isolated worktree branch, PR against `dev`, admission gate, review, CI, and merge queue.
- Direct local validation uses `~/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo`; no external coordination shim is part of the workflow.

## Rejected alternatives

| Alternative | Why rejected |
|---|---|
| Keep the retired nested spec scope directory | Retains a redundant default-scope token and diverges from the ADR-0115 flat-root cleanup pattern. |
| Create `spec/cross-cutting/` singular root | Introduces a new root and keeps the redundant scope token; existing BNF and path conventions already use `specs/`. |
| Split by concern under multiple scoped directories | No current sibling scopes justify the taxonomy; adding them now would increase sprawl instead of removing it. |
| Move lifecycle configs to flat files at `specs/<name>.json` | Loses the typed-family grouping used by lifecycle automation lanes; the family directory is meaningful while `cross-cutting/` was not. |

## Verification

- A directory-existence check confirms the retired nested spec scope directory is gone.
- A path grep for the retired nested spec scope returns no hits after rewrite.
- JSON and Rust validation are recorded in the PR for the implementing change.

## References

- ADR-0115 — registry consolidation: flat singular `registry/`.
- ADR-0116 — retire external agent-coordination tooling in favour of the Foundry pipeline.
- ADR-0110/0111/0112/0113 — Foundry pipeline substrate for changeset state, projected merge state, webhook invocation, and end-to-end VCS orchestration.
- M01-P18 — Foundry pipeline canonical VCS substrate.
