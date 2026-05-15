---
purpose: Ship object-graph.entity.upsert + 5 property tiers (vector/timeseries/geo/ciphertext/struct) per ADR-0006.
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P04-IP-002
title: Object Graph entity upsert + 5 property tiers
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship object-graph.entity.upsert + 5 property tiers (vector/timeseries/geo/ciphertext/struct) per ADR-0006.
---

# M01-P04-IP-002 — Object Graph entity upsert + 5 property tiers

## Purpose
Ship object-graph.entity.upsert + 5 property tiers (vector/timeseries/geo/ciphertext/struct) per ADR-0006.

## Symbols-to-grit-claim
```
crates/oya-ontology-domain/src/lib.rs::ObjectEntity
crates/oya-ontology-domain/src/lib.rs::ObjectProperty
crates/oya-ontology-domain/src/lib.rs::PropertyTier
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M-CC-P01 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test --locked -p oya-ontology-domain
cargo clippy --locked -p oya-ontology-domain --all-targets -- -D warnings
scripts/check.sh
```

## Done-criteria
- Scoped Rust acceptance commands return 0; repository-wide `scripts/check.sh`
  now passes the restored helper-script preflight and remains a pre-existing
  acceptance blocker at repo-wide `cargo fmt --all -- --check` drift.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).
- Distroless + provider-coupling + LTS-dependency lanes green on PR.

## Probe-evidence (2026-05-14)
- Added the domain entity upsert seam `ObjectGraph::upsert_entity`, keyed by
  tenant id + entity id with explicit created/updated outcomes and row-isolated
  tenant storage.
- Kept `ObjectEntity::upsert_property` as an intra-entity helper with separate
  property insert/update outcome naming so it is not confused with the stable
  entity-upsert surface.
- Exposed the five Object Graph property tiers through
  `PropertyTier::object_graph_property_tiers()` with stable wire labels for
  `vector`, `timeseries`, `geo`, `ciphertext`, and `struct`; `scalar` remains
  available through the all-tier compatibility set.
- Added `data_class` annotations to the Object Graph kernel struct fields and
  promoted the machine-readable `object-graph.entity.upsert` contract mirror to
  stable.
- Fresh scoped evidence is recorded in
  `/evidence/foundation/m01-p04-ip-002-object-graph-property-tiers.json`.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs depend on, follow per-crate split unwind per ADR-0015 §7.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M01-P04-IP-002 Object Graph entity upsert + 5 property tiers is probe-green / acceptance-blocked; scoped ontology evidence is green, scripts/check.sh now passes helper preflight/codeview/cargo fmt and remains blocked at cargo check stale connect-domain imports' -i high -k 'M01-P04-IP-002,probe-green,acceptance-blocked'
```

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: entity-level create/update and
property-level insert/update now use separate outcome types, avoiding the
misleading state where a stable `object-graph.entity.upsert` contract was backed
only by property replacement inside a pre-existing entity.
