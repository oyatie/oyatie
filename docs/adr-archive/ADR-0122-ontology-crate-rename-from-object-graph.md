---
id: ADR-0122
status: Superseded
deciders: council-architecture, ontology-team
date: 2026-05-16
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0056, Bominal-ADR-0106, Bominal-ADR-0107, Bominal-ADR-0133]
related_specs: [/specs/microservices/ontology.json, /specs/knowledge-graph-schema.json]
purpose: Rename oya-platform-object-graph-kernel crate (and any siblings) to oya-ontology-* per the glossary rename Object Graph → Ontology (matching Palantir terminology); aligns crate names with /specs/microservices/ontology.json + Bominal-ADR-0106; eliminates the stale "object-graph" token from the canonical naming surface.
---

# ADR-0122: Ontology crate rename — retire "object-graph" naming

## Status

Accepted — 2026-05-16.

## Context

The glossary directive `feedback_glossary_ontology_not_object_graph` (captured 2026-05-14) renamed the architectural concept "Object Graph" to "Ontology", matching the Palantir-Foundry term that the engine-enforced typed-entity layer is patterned on (per Bominal-ADR-0106).

The crate `oya-platform-object-graph-kernel` was scaffolded before that rename. It is now drift: a stale token in canonical naming surface that contradicts the doctrine + `/specs/microservices/ontology.json` (PRD authored 2026-05-16, this same PR sequence).

The multispectrum-review wave that landed as PR #20 (2026-05-16) also surfaced this drift via the A1 naming-adherence facet, but the fix was filed as successor-IP. Per `/specs/agent-durable-goal.json#operating_principles/OP-11` (no_stubs_no_defer_100_production_quality, accepted 2026-05-16), the rename is now executed in-PR rather than scheduled-for-distinct-tracked-work.

## Decision

Rename:

| Current crate name | New crate name | Reason |
|---|---|---|
| `oya-platform-object-graph-kernel` | `oya-ontology-kernel` | Match Bominal-ADR-0106 Ontology naming + `feedback_glossary_ontology_not_object_graph` |
| (already correct: `oya-ontology-api`, `oya-ontology-domain`) | n/a | sanity-check — these already use `ontology` |

Plus the planned-but-not-yet-scaffolded crates per `/specs/microservices/ontology.json#identity.crate_refs_planned`:
- `oya-ontology-usecase` (12-layer enum requires usecase between domain + adapter per ADR-0056)
- `oya-ontology-adapter` (per ADR-0056 inward-flow)

This ADR ratifies the naming. The kernel rename is performed via `git mv` in this same PR sequence (separate commit). The planned-but-not-scaffolded crates are downstream IPs sequenced in `specs/masterplan.json` (filed via the milestone audit step).

## Rejected alternatives

- **Keep "object-graph" naming**: rejected — contradicts the ratified glossary rename + Bominal-ADR-0106 + `/specs/microservices/ontology.json`. Would force every consumer to maintain two mental models.
- **Rename to "knowledge-graph-kernel"**: rejected — knowledge-graph is the runtime layer (semantic/kinetic/dynamic 3-layer split per `/specs/knowledge-graph-schema.json`); ontology is the typed-entity schema layer. Ontology kernel hosts type/instance/action types; knowledge-graph hosts runtime state. Per user pick 2026-05-16 the PRD merges them into ONE product but the crate boundaries reflect the architecture: ontology = schema authority; knowledge-graph = runtime state via OTel + Kafka.
- **Defer the rename to a successor-IP PR**: rejected per `/specs/agent-durable-goal.json#operating_principles/OP-11` — drift between code and doctrine is in-scope and must be fixed in-PR.

## Consequences

### Positive
- Crate names match canonical doctrine + Palantir-aligned Ontology terminology.
- `/specs/microservices/ontology.json` becomes self-consistent (no "but the crate is still called object-graph" caveat).
- Future agents searching for ontology code find it under the expected token.

### Negative / migration cost
- Inbound `use oya_platform_object_graph_kernel::*` statements need rewriting wherever they exist (sweep performed in the same PR commit).
- `registry/catalog/oya-platform-object-graph-kernel.yaml` renamed accordingly.
- `Cargo.toml` workspace.members rewrite + path adjustments.

### Operational
- One commit performs the rename + sweep + Cargo.lock regen + cargo check verification per `/specs/agent-durable-goal.json#verification_before_completion`.

## Naming justification

Per `feedback_naming_justification`: `oya-ontology-kernel` parses cleanly against P14 BNF (kind=ontology bound to v4 BNF slot-2 cross-cutting backbone) + the 12-layer enum (kernel layer). One-line proof: `oya-<context=ontology>-<role=kernel>` is the canonical shape per ADR-0056. The retired token `object-graph` is preserved in `crate-naming-audit.json#retired_package_notes` for traceability per the PR #20 pattern.

## Sunset / Reversal

Terminal rename; no future sunset clause.

Reversal procedure (if Ontology terminology proves wrong, which is unlikely given Palantir industry alignment):
1. `git revert <merge-sha-of-this-ADR-PR>` — restores the prior `oya-platform-object-graph-kernel` crate name everywhere.
2. Re-amend `/specs/microservices/ontology.json` to use the legacy token.
3. Update `feedback_glossary_ontology_not_object_graph` memory entry to mark the rename reversed.

data_loss_class: none. Pure rename. `git mv` preserves history.

## Verification plan

- `cargo check --workspace --locked` exit 0 after the rename + sweep commit.
- `cargo nextest run --workspace --locked --no-fail-fast` 0 failures.
- `grep -rln "oya-platform-object-graph\|oya_platform_object_graph" --include='*.rs' --include='*.toml' --include='*.yaml' --include='*.json'` returns zero results (excluding evidence/* + this ADR's own historical record + crate-naming-audit retired_package_notes).
- `/specs/microservices/ontology.json#identity.crate_refs_current` updated to reference the renamed crate.
- `registry/catalog/oya-platform-object-graph-kernel.yaml` renamed to `oya-ontology-kernel.yaml`.
