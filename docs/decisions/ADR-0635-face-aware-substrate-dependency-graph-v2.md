---
id: ADR-0635
title: "Face-aware substrate dependency graph v2: five typed graphs and derived failure closure"
status: Accepted
planning_impact: true
deciders: founder
owner: council-architecture
date: 2026-08-01
door: one-way
supersedes: []
superseded_by: []
depends_on: [ADR-0245, ADR-0280, ADR-0562, ADR-0615]
amends: [ADR-0245, ADR-0280, ADR-0562, ADR-0615]
related: [ADR-0176, ADR-0515]
related_specs:
  - /specs/substrate-dependency-dag.json
  - /specs/substrate-dependency-dag.schema.json
  - /ci/facade/dependency-graph-acyclicity/substrate-dependency-dag-policy.json
milestone: W0-C
---

# ADR-0635: Face-aware substrate dependency graph v2

## Status

**Accepted — 2026-08-01.** The founder authorized autonomous execution of the capability-first
reorganization and required ambiguity below five percent before implementation. This amendment
closes ADR-0280 §D-13.H item 1 without changing capability membership, module membership, or
layer-rank baselines.

## Context

ADR-0245 established dependency direction, ADR-0280 made the steady-state substrate dependency
DAG canonical, ADR-0562 moved ownership to capability-first roots, and ADR-0615 resolved the
capability boundary rulings. ADR-0280 §D-13 then identified the remaining category error: a
capability can have plane-specific faces whose genesis, provisioning, request, publication, and
failure relationships point in different directions. The v1 artifact flattened those relationships
into `nodes`, `edges`, and one `bootstrap_order`, while explicitly deferring a face-aware schema.

The flat artifact was correct only for steady-state requests. Extending its single edge set would
make legitimate reverse directions appear cyclic or would weaken the acyclicity invariant until it
stopped protecting runtime requests.

## Decision

### D1 — `dependency_units` are `runtime_face`-qualified

`specs/substrate-dependency-dag.json` v2 declares exactly 19 unique, closed `dependency_units`. A
unit id combines a canonical capability and its founder-locked `runtime_face`, for example
`cell.envelope`, `cell.lifecycle.cp`, `iam.local-verifier`, or `policy.authoring.cp`. Every unit
declares a capability from the closed 24-capability registry, its `runtime_face`, and one of the
internal planes `B0`, `C0`, `C1`, `C2`, `G`, or `R`.

The validator compares the document to the exact founder-authoritative closed set of 19
`(id, capability, runtime_face, plane)` tuples. Shape-valid substitutions, renamed faces, and
consistently rewired replacement units are contract drift and fail closed rather than silently
creating a nineteenth alternative topology.

A unit is a topology endpoint, not a new capability and not a repository module. The closed
24-capability membership from ADR-0562/0615 is unchanged. E0 integrity and genesis roots are
declared separately as `external_anchors`; they are valid topology endpoints but are not falsely
classified as capabilities or counted among the 19 dependency units.

### D2 — exactly five graph kinds

The contract contains exactly these graph kinds, in canonical order:

1. `genesis`
2. `new_cell_provisioning`
3. `steady_state_request`
4. `control_data_publication`
5. `failure_brownout_propagation`

Every edge repeats its `graph_kind`; a mismatch with its containing graph is cross-kind
contamination and fails closed. Every endpoint must name a declared dependency unit.

### D3 — only steady-state requests are acyclic

Only `steady_state_request` is Tarjan-acyclic and dependency-first Kahn-topo-sortable. Its edges
mean `from` synchronously depends on `to`. It retains the v1 edge metadata, forbidden-edge
assertions, and valid hand-authored bootstrap order after mapping the ten legacy nodes to their
runtime faces.

The other four graphs have distinct semantics. A reverse direction or cycle outside graph 3 does
not violate the request-DAG invariant. This is deliberate: rejecting those directions would
recreate the flattening error this amendment removes.

### D4 — failure propagation is derived, with explicit max-min composition

`failure_brownout_propagation` is the exact reverse transitive closure of
`steady_state_request`. A request edge `A -> B` therefore produces failure direction `B -> A`.
The failure graph is not independent authoring authority.

For multiple-hop paths the composition rule is:

- **Within one path:** take the minimum severity across its edges. The weakest propagation edge
  bounds that path; a BROWNOUT dependency cannot become FULL merely because an upstream edge is
  FULL.
- **Across multiple paths:** take the maximum path severity. The strongest path by which a failed
  unit affects an impacted unit wins.
- Severity order is `INDEPENDENT < BROWNOUT < DEGRADED < FULL`.

This max-min rule removes the ambiguity in ADR-0280 §D-5's prose while preserving its intent: weak
links attenuate a path, but an alternative stronger path still governs impact.

### D5 — closed schema and fail-closed validator

`specs/substrate-dependency-dag.schema.json` is a Draft 2020-12 closed schema. The Rust gate does
not claim to be a general-purpose JSON Schema interpreter: it faithfully enforces every invariant
used by this schema and binds the parsed schema authority to its reviewed deterministic parsed-JSON
serialization SHA-256.
Therefore a replacement schema cannot be ignored merely because it retains the Draft marker;
executable mutations of `prefixItems`, `items: false`, `required`, `additionalProperties`, `const`,
ranges, and types all fail closed, including a rejecting `{"not": {}}` replacement.

The existing `dependency-graph-acyclicity` Buck targets keep their names and reject missing/extra
graph kinds, 18/20-unit drift, any deviation from the exact 19 tuples, duplicate or unknown units,
unknown capabilities or endpoints, missing `runtime_face`, cross-kind contamination, graph-3
self-loops and cycles, request weights outside `(0, 1]`, malformed request or failure metadata, and
any missing, extra, forward, or incorrectly composed failure closure edge. The gate declares the
graph, schema, and capability registry as Buck resources. Executable fixture-driven tests prove
those RED classes and prove reverse directions outside graph 3 plus the exact closure remain GREEN.

### D6 — mandatory follow-ups, no new baselines

This slice does **not** migrate module-membership consumers or layer-rank projections. Both are
recorded in the contract as mandatory follow-ups:

- `W0-C-MODULE-MEMBERSHIP`
- `W0-C-LAYER-RANKS`

Neither follow-up may mint a new frozen baseline. Existing debt must be preserved only through the
current merge-base/controller-derived mechanisms until the consumer migrates.

## Consequences

- Control-plane publication and provisioning directions can be expressed without weakening the
  runtime request DAG.
- Failure impact is deterministic and mechanically reviewable rather than a hand-maintained second
  truth.
- Existing v1 consumers that read top-level `nodes` or `edges` must migrate to the graph-v2 contract;
  those migrations are follow-up work, not silently bundled here.
- Capability membership and repository placement remain unchanged.

## Alternatives considered

- **Keep v1 and encode faces in prose.** Rejected: ADR-0280 already identified the missing machine
  contract, and prose cannot fail closed on cross-kind contamination or closure drift.
- **One graph with an edge-type field.** Rejected: applying acyclicity to the union rejects valid
  reverse directions; not applying it to the union permits request cycles.
- **Require every graph to be acyclic.** Rejected: only synchronous steady-state request dependency
  direction carries that invariant.
- **Compose impact by maximum edge severity along a path.** Rejected: it lets a FULL upstream edge
  amplify through a BROWNOUT boundary. Max-min composition respects attenuation and still selects
  the strongest alternative path.
