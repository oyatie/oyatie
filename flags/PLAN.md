---
doc_class: Owner-PLAN
owner: flags
status: Active
date: 2026-08-26
---

# Flags remaining work

<baseline>

## What has landed

- A dependency-free deterministic evaluation kernel with typed values,
  targeting conditions, ordered first-match rules, sticky percentage bucketing,
  default/off behavior, safe selected-reference errors, and a source trait.
- Nineteen domain tests plus one bundled-server seam smoke test.
- The domain crate root is 292 lines and owns four hand-maintained module-index
  entries, so D-35/D-41 structural preparation precedes its next behavior.
- ADR-0719's first corpus reduction removed thousands of lines of contracts,
  dashboards, policy fragments, runbooks, audit reports, catalogs, and product
  narratives.

No server, listener, Connect/OFREP contract, durable definition authority,
admission bounds, explicit kill state, C0 override proof, snapshot distribution,
SDK, authorization, audit, deployment, live metrics, or SLO evidence has landed.
The remaining server, Cedar, IaC, OpenSLO, root loader, and README surfaces are
migration residue and must not be cited as runtime readiness.

</baseline>

<sequence>

## L1a — Establish Flags owner law

Class: documentation/authority only.

- Add the four canonical owner files and replace the stale README narrative.
- Record current kernel behavior, the narrowed charter, target authority and
  interface model, unqualified SLO objectives, fault behavior, and exact next
  structural/behavioral sequence.
- Do not edit code, manifests, lockfiles, root law, generated artifacts, or
  residual files in this lane.

Success: owner law agrees with ADR-0719 and current code; every runtime/SLO claim
is explicitly absent or future; links, format, path admission, and unchanged
Flags tests pass.

Failure: the docs preserve experiment/server duality, treat corpus YAML as
deployment evidence, or claim mutation/distribution/readiness behavior.

Rollback: revert only the five owner documentation paths; no runtime or format
state changes.

Fault evidence: hostile review traces every landed claim to source/tests and
every target claim to a named future lane.

## L1b — Retire `core/server`; preserve `evaluation-domain`

Class: structural; this is `<next_lane>`.

- Delete `flags/core/server/**`: its empty config, no-op initializer, TODO-only
  REST/gRPC/OFREP/storage/targeting/tenant modules, re-export seam, Cargo/BUCK
  package, and binary that immediately exits.
- Preserve `flags/core/evaluation-domain/**` byte-for-byte except any mechanical
  reference cleanup required by admission; do not move or redesign the kernel.
- Prove a fresh reverse-consumer scan finds no live server consumer. Historical
  docs are provenance and are not updated as active call sites.
- Materialize `Cargo.lock` only if the repository's lock freshness command
  requires removal of the retired package; this shared hub requires serialized
  occupancy and no hand edit.

Changed-path envelope: `flags/core/server/**`, plus mechanically generated
`Cargo.lock` only when required. No owner-law, root manifest, evaluation-domain,
generated-face, or behavioral edit.

Success: workspace metadata, Flags domain tests, path admission, and relevant
Cargo/Buck checks pass without a `flags-server` package/target; evaluation
golden behavior is unchanged.

Failure: any evaluation API/vector changes, a live consumer breaks, a listener
is discovered, or lockfile churn exceeds the removed package closure.

Rollback: restore the scaffold package as one structural revert; it never held
runtime or durable state.

Fault evidence: before/after evaluation vectors plus negative repository search
for server imports, targets, runtime entrypoints, and deployment consumers.

## L1c — Remove law-confirmed legacy dump

Class: structural residue cleanup; depends on L1b.

- Re-run exact consumer and build-loader discovery, then remove the current
  unconsumed residue: `flags/cedar/policies.cedar`, `flags/iac/**`,
  `flags/observability/slos/**`, and the root `flags/BUCK` corpus-only loader.
- The Cedar file governs old CI/cell/Argo/Jenkins behavior rather than Flags.
  The IaC directly declares Helm/Kubernetes/Terraform/OpenBao/WAF/ECH/PQC
  deployments for a nonexistent server. The OpenSLO files name nonexistent
  experiment, autosharding, evaluator, propagation, and kill metrics.
- Do not replace them with prose, hand-authored deployment YAML, or hand-authored
  OpenSLO. Later IaC and SLO sources land only with a consuming reconciler/
  controller and real producer in the same behavioral wave.
- If fresh evidence finds a non-corpus live consumer, stop that path and return
  a blocker card; loader-only reachability is not product consumption.

Changed-path envelope: `flags/BUCK`, `flags/cedar/**`, `flags/iac/**`, and
`flags/observability/slos/**` only.

Success: repository/build searches prove no deleted artifact is consumed by a
runtime, policy evaluator, reconciler, SLO controller, or deployment path;
workspace/path tests remain green; only owner law, ownership, README, and the
evaluation kernel remain under Flags.

Failure: a live producer/consumer loses required input, a valid Flags-specific
policy is deleted without replacement, or new fiction replaces old residue.

Rollback: restore only a proven consumed artifact and its exact loader; do not
restore the dump as a bundle.

Fault evidence: consumer scans plus negative fixtures showing a stale reference
or a prohibited hand-authored Helm/OpenSLO replacement fails admission/review.

## L1d.0 — Stabilize the evaluation crate before L1d

Class: structural file-budget and D-41 preparation; depends on L1c.

- Add `flags/core/evaluation-domain/build.rs`. Its owned scanner reads only
  direct `.rs` entries from `src/items/` and `src/test_items/`, sorts paths, and
  writes `lib.generated.rs` and `tests.generated.rs` only under `OUT_DIR`.
- Mechanically move `bucket.rs`, `engine.rs`, `model.rs`, and `port.rs` into
  `src/items/{a_bucket,b_engine,c_model,d_source_port}.rs`; preserve the public
  `bucket`, `engine`, `model`, and `port` module paths with item-local wrappers.
  Move root re-exports to `src/items/z_exports.rs` and the `src/lib.rs` tests to
  `src/test_items/a_golden_vectors.rs`.
- Reduce `src/lib.rs` to its stable crate prelude plus generated source/test
  `include!` lines. No tracked generated membership or hand-maintained per-item
  `mod` list remains, and later behavior does not edit this root.
- Update `BUCK` to build/run the same script, stage the same two globbed input
  directories, and pass the generated `OUT_DIR` to library and unit-test targets.
  Cargo auto-detects `build.rs`; `Cargo.toml` is not changed.

Closed write envelope: `flags/core/evaluation-domain/build.rs`, `BUCK`,
`src/lib.rs`, the four current `src/{bucket,engine,model,port}.rs` paths,
`src/items/**`, and `src/test_items/**`. `Cargo.toml`, root/lock/generated files,
other Flags paths, and behavior are forbidden.

Build closure: `cargo test --locked --offline -p flags-evaluation-domain` and
`buck2 test //flags/core/evaluation-domain:flags-evaluation-domain-unittest`.
Required review is the Flags owner plus an independent build/architecture
reviewer; neither hosted green checks nor the author supplies APPROVE.

Success: all 19 evaluations/source vectors, public modules, root re-exports,
type identities, reasons, errors, and FNV assignments match before/after; Cargo
and Buck generate the same ordered membership; every handwritten file is at most
300 lines.

Failure: a public path or vector changes, one build graph omits an item, an
index is tracked/manual, or structure is mixed with admission behavior.

Rollback: revert the scanner and file moves together; no data or semantic format
exists.

Fault evidence: a public-path compile fixture, before/after 19-vector receipt,
and add/rename/remove item canary prove both build graphs regenerate without a
crate-root edit; a manual/tracked inventory fixture is rejected.

## L1d — Admit bounded definitions before evaluation

Class: narrow behavioral core; depends on L1d.0.

- Add `ValidatedFlag`, `ValidatedEvaluationContext`, and stable validation
  failures in the unique item `src/items/e_admission.rs`; add boundary,
  deterministic-arbitrary-input, and golden evidence only as
  `src/test_items/{b_admission_boundaries,c_admission_arbitrary,d_admission_golden}.rs`.
- Enforce every exact v1 count, individual byte, aggregate byte, reference,
  rollout, operator, finite-float, and safe-fallback rule in `SPEC.md` before
  evaluation. Preserve current precedence and FNV-1a assignments.
- Explicit kill state, C0 overrides, mutation authority, distribution, and
  facade remain later work; do not simulate them with context attributes.

Closed write envelope: the four unique item/test paths named above only.
`src/lib.rs`, existing items/tests, `build.rs`, `BUCK`, `Cargo.toml`, root/lock,
and every other Flags path are frozen.

Build closure is the two L1d.0 Cargo/Buck targets. Required review is the Flags
owner plus an independent evaluation-security/performance reviewer.

Success: all 19 landed vectors remain unchanged; every valid maximum admits;
every maximum+one and non-finite float fails before evaluation; maximum admitted
evaluation is deterministic, panic-free, and within the declared work bound.

Failure: a malformed reference remains subject-dependent, a valid subject re-
buckets, unbounded hashing/cloning survives, or parent/build files are edited.

Rollback: remove only the four new items before a facade publishes the validated
API; the L1d.0 stable structure and landed kernel remain.

Fault evidence: empty/duplicate/unknown identifiers, bad operand shapes,
overflow/over-100% rollout, every individual and aggregate max/max+one,
targeting/string/object limits, NaN and infinities, Unicode boundaries,
randomized rule order, and repeated/cross-build golden replay.

## L1e.0 — Promote authority contracts into owner-local port faces

Class: structural port/graph mutation; depends on L1d.

- Move the synchronous source seam and its source test from
  `core/evaluation-domain/src/items/d_source_port.rs` into new
  `ports/draft/definition-source`; remove only its compatibility re-export from
  `src/items/z_exports.rs`. The combined closure retains all 19 behaviors.
- Create owner-local draft packages `ports/draft/mutation-authority`,
  `ports/draft/snapshot-distribution`, and `ports/draft/audit-outbox` for the
  SPEC contract values/traits only. Add no store, network, authorization,
  snapshot publication, or mutation behavior.
- Each new port package gets its own sorted `build.rs` scanner, stable
  `src/lib.rs` `OUT_DIR` includes, `src/items/a_contract.rs`, contract-test item,
  Cargo manifest, and equivalent Buck `buildscript_run` graph. Root workspace
  membership stays untouched because the accepted glob already owns these faces.
- These remain `draft` because no external consumer exists. Discovery of a
  consumer stops the lane and triggers a separately dispatched D-29 promotion;
  this PR does not self-widen into another owner.

Closed write envelope: exact core paths
`flags/core/evaluation-domain/src/items/{d_source_port,z_exports}.rs` and
`flags/core/evaluation-domain/src/test_items/a_golden_vectors.rs`; the four exact
`flags/ports/draft/{definition-source,mutation-authority,snapshot-distribution,audit-outbox}/**`
trees; and materialized `Cargo.lock` only if new workspace-package entries
require it. Root `Cargo.toml`, adapters, behavior, and generated third-party
files are forbidden.

Build closure: the evaluation target plus Cargo/Buck library and contract-test
targets for the four new `flags-*-draft` packages. Required review is the Flags
owner plus an independent structural/architecture reviewer; any discovered
consumer owner is a blocker and belongs to the separate D-29 review envelope.

Success: core contains no I/O trait, each contract has one draft owner and no
implementation, all 19 behaviors remain in the combined closure, workspace lock
materialization is minimal, and Cargo/Buck membership matches.

Failure: behavior lands, a port imports an adapter/provider type, the old/new
contracts coexist, root membership is edited, or another owner consumes draft.

Rollback: restore the source seam/test to their exact L1d.0 items and remove the
four draft packages plus only their lock entries.

Fault evidence: inverse dependency scans, compile-fail foreign/draft-consumer
fixtures, contract type checks, all 19 vectors, and Cargo/Buck membership parity.

## L1e.1 — Admit the durable-reference dependency

Class: serialized shared dependency/graph prerequisite; depends on L1e.0 and runs
only while the workspace has no approved SQLite binding, which is true at L1a.

- Select one policy-admitted Rust SQLite binding for the self-contained durable
  reference adapter; generate the lock and Buck third-party face with the
  repository-owned materializer. Add no Flags implementation in this slice.

Closed write envelope: root `Cargo.toml`, `Cargo.lock`, and generated
`third-party/BUCK` only. It serializes against all shared-hub lanes and may not
touch `flags/**`.

Build closure: locked/offline metadata, dependency/license/source policy,
idempotent third-party regeneration, and the generated Buck target. Required
review is supply-chain plus architecture and the Flags owner as the named
consumer. Failure is an unapproved license/source, runtime download, hand edit,
non-idempotent generation, or unrelated lock churn. Rollback removes the one
dependency and its generated closure before any adapter format exists.

## L1e.2 — Create frozen adapter package faces

Class: structural adapter/graph mutation; depends on L1e.1.

- Create exact draft packages `adapters/draft/definition-sqlite` and
  `adapters/draft/snapshot-memory` with Cargo manifests, Buck library/test
  targets, package-root `build.rs` scanners, stable `src/lib.rs` generated
  includes, and compile-neutral `src/items/a_face.rs` plus local test/integration
  item roots. The scanners sort declared `src/items`, `src/test_items`, and
  `tests/items` membership into `OUT_DIR`; Buck stages the same globs and executes
  the same scripts as Cargo.
- Wire the admitted SQLite dependency and the four L1e.0 port packages into the
  new package graphs. Materialize only workspace-package lock entries required
  by these faces. Do not add schemas, stores, snapshot publication, transaction
  behavior, conformance claims, or runtime/readiness behavior.

Closed write envelope: the two exact
`flags/adapters/draft/{definition-sqlite,snapshot-memory}/**` trees and
materialized `Cargo.lock` package entries only. Root `Cargo.toml`,
`third-party/BUCK`, core, ports, facade, owner law, and all behavior are frozen.

Build closure: locked/offline metadata, evaluation and all four port targets,
plus Cargo/Buck library and empty-face tests for the two new adapters. Required
review is the Flags owner plus independent structural/build and Data durability
reviewers.

Success: both draft faces compile through Cargo and Buck with identical sorted
membership and stable parents, and no runtime or durability behavior exists.

Failure: a store/schema/snapshot algorithm lands, an adapter claims readiness,
Cargo/Buck membership differs, a manual/tracked index appears, or root/port/core
paths change.

Rollback: remove the two package faces and only their lock entries; no format or
runtime state exists.

Fault evidence: add/rename/remove item canaries in both graphs, inverse-
dependency checks, forbidden scaffold-behavior fixtures, and idempotent locked
metadata prove the faces are structural only.

## L1e.3 — Implement durable definition and snapshot behavior

Class: behavioral adapters; depends on L1e.2.

- Implement definition source, mutation, idempotency, and pre-ack audit/outbox
  in `definition-sqlite/src/items/b_store.rs`, with versioned
  `migrations/0001_definition.sql`, contract evidence in
  `src/test_items/b_contract.rs`, and real-file recovery evidence in
  `tests/items/b_recovery.rs`.
- Implement immutable process-local generation publication in
  `snapshot-memory/src/items/b_snapshot.rs` with conformance evidence in
  `src/test_items/b_contract.rs`. It is an atomicity reference, never durable
  authority.

Closed write envelope: the six unique behavior/test/migration paths named above
only. Both packages' manifests, BUCK files, `build.rs`, stable `src/lib.rs` and
test roots, face items, lockfile, ports, core, root, facade, and owner law are
frozen.

Build closure: evaluation, all four port targets, and the two frozen adapter
Cargo/Buck library/contract/integration targets. Required review is the Flags
owner, an independent durability/Data reviewer, and an audit/security reviewer.

Success: same-key/same-digest replay returns one monotonic committed generation;
changed digest conflicts; pre-commit interruption exposes nothing; post-commit
response loss survives hard close/reopen; snapshot readers see one prior or next
generation and never a partial value.

Failure: SQLite page-cache success is called durable, definition/idempotency/
audit state diverges, two stores become authoritative, snapshot mutation is
in-place, or a frozen structure/build path changes.

Rollback: stop selecting the draft SQLite adapter while retaining its admitted
schema reader; remove the memory behavior freely, but never downgrade an opened
database across a declared format barrier.

Fault evidence: interruption after begin and each write, before/after commit,
lost response plus replay, full/busy/corrupt database, N/N+1 migrate/reopen,
concurrent readers, duplicate/out-of-order generation, and atomic-swap failure.

## L1e.4 — Add kill-state and verified C0 authority

Class: behavioral authority; depends on L1e.3.

- Add explicit kill state and unforgeable verified C0 override inputs, preserving
  precedence `kill > disabled > verified override > rules > rollout > default`.
  Ordinary targeting attributes never construct or substitute authority.
- Extend only the already bounded admission/authority items, the mutation and
  distribution contract items, the two adapter implementation items, SQLite
  `migrations/0002_kill_override.sql`, and paired unique test items. Parent
  `lib.rs`, scanners, BUCK/Cargo manifests, root/lock, and other ports stay frozen.

Closed write envelope: exact paths
`flags/core/evaluation-domain/src/items/{e_admission,f_authority}.rs`,
`flags/core/evaluation-domain/src/test_items/e_authority.rs`,
`flags/ports/draft/{mutation-authority,snapshot-distribution}/src/items/a_contract.rs`,
`flags/adapters/draft/definition-sqlite/src/items/b_store.rs`,
`flags/adapters/draft/definition-sqlite/src/test_items/c_kill_override.rs`,
`flags/adapters/draft/definition-sqlite/tests/items/c_kill_recovery.rs`,
`flags/adapters/draft/definition-sqlite/migrations/0002_kill_override.sql`,
`flags/adapters/draft/snapshot-memory/src/items/b_snapshot.rs`, and
`flags/adapters/draft/snapshot-memory/src/test_items/c_authority.rs` only.

Build closure is the full L1e.3 closure plus authority/partition/recovery tests.
Required review is the Flags owner plus Policy/IAM, security, audit, and
durability reviewers; their sold facades are read-only and any required external
edit becomes a separate D-29 lane.

Success: only request-bound verified authority changes kill/override state; kill
survives restart and always wins; generation, idempotency, audit, and snapshot
publication remain atomic and monotonic.

Failure: caller attributes forge authority, stale proof applies, restart clears
kill state, priority skips audit/order, or external owner paths are edited.

Rollback: publish a higher authorized generation through the same control path;
never delete or locally decrement durable kill history.

Fault evidence: forged/expired/wrong-audience/cross-tenant C0 and PDP proofs,
kill/update races, response loss/replay, restart while engaged, duplicate/reordered
snapshots, authority outage, and audit failure before acknowledgement.

## L1f — Publish Connect facade and cell-local distribution

Class: serving capability; depends on L1e.4.

- Author one protobuf/Connect contract, default-deny control/evaluation facade,
  signed snapshot stream/resync, atomic cell-local cache, and compatibility
  adapter conformance.
- Add live telemetry and owner SLO IR only with their consuming Observability
  path; derive deployment through the IaC reconciler rather than restoring YAML.
- Qualify readiness and PRD SLO objectives through load, partition, corruption,
  restart, and kill-switch propagation campaigns.

Success: ordinary evaluation has no remote hop, generations are observable and
monotonic, and only measured/consumed capabilities report ready.

Failure: REST/gRPC dual truth returns, OFREP owns metadata, stale/corrupt state
is hidden, or deployment/OpenSLO text precedes runtime evidence.

Fault evidence: asymmetric partitions, snapshot loss/reorder/corruption,
process/cell restart, authority outage, overload, forged auth/C0, and prioritized
kill propagation under concurrent ordinary updates.

</sequence>

<parallelism>

The Flags L1 chain is sequential because each slice deletes or stabilizes the
next slice's build and semantic surface. It may run beside lanes whose changed
paths and practical Cargo/Buck closures exclude `flags/**`, `Cargo.lock`, root
workspace/admission files, and generated hubs. Owner-law files retain one
writer; cloud CI observation and independent review do not occupy this worker.

</parallelism>
