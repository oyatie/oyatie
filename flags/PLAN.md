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

## L1f.0 — Create unrouted serving, distribution, compatibility, SLO, and deployment faces

Class: serialized structural proto/package/build-graph mutation; depends on
L1e.4.

- Add the one sold contract at exact package path
  `flags/facade/proto/flags/runtime/v1/flags_service.proto`, declaring protobuf
  package `flags.runtime.v1`, plus package-local `BUCK` and `OWNERS`. It contains
  evaluation, separately authorized control, and authenticated distribution
  services under one semantic version; it does not create REST, gRPC, or OFREP
  truth beside Connect.
- Create these exact Rust package roots:

  ```text
  flags/facade/runtime-app
  flags/ports/draft/runtime-authority
  flags/ports/draft/runtime-telemetry
  flags/ports/draft/runtime-deployment
  flags/adapters/draft/snapshot-connect
  flags/adapters/draft/openfeature-ofrep-compat
  flags/adapters/draft/runtime-telemetry-otel
  flags/adapters/draft/runtime-deployment-iac
  ```

  The existing `snapshot-memory` adapter remains the future cell-cache backend.
- Each Rust package receives its Cargo manifest, Buck library/binary and local
  contract-test targets, package-root `build.rs`, stable crate/test roots, and a
  compile-neutral `a_face.rs`; the facade instead has stable `src/main.rs`,
  `src/lib.rs`, and `src/items/a_wiring.rs` whose only result is typed
  `Unrouted`. No listener or ready response exists.
- Every scanner sorts direct Rust entries from declared `src/items`,
  `src/test_items`, and `tests/items` directories into named files under
  `OUT_DIR`; the facade also emits its stable main/library membership. Buck
  stages the identical globs, runs the same scanner, and supplies the same
  generated sources. There is no tracked/manual index, and add/rename/remove
  canaries require no parent edit.
- Reuse the already admitted protobuf, transport, and telemetry dependency
  closure at L1a. Existing workspace globs admit all new packages, so root
  `Cargo.toml` and `third-party/BUCK` stay frozen; materialize only the new
  workspace-package entries in `Cargo.lock`.
- `runtime-telemetry` and `runtime-deployment` are unrouted owner-local faces,
  not copied Observability/IaC schemas. Do not add `flags/iac/runtime.textproto`
  or `flags/observability/slos/runtime.generated.openslo.yaml` until L1f.2/L1f.3
  has a provider-owned consumer and live producer.

Closed write envelope: `Cargo.lock` package entries; exact proto paths
`flags/facade/proto/flags/runtime/v1/flags_service.proto`,
`flags/facade/proto/flags/runtime/v1/BUCK`, and
`flags/facade/proto/flags/runtime/v1/OWNERS`; and the eight exact Rust package
roots named above. Within each of the seven library
port/adapter roots, only `Cargo.toml`, `BUCK`, `build.rs`, `src/lib.rs`,
`src/items/a_face.rs`, `src/test_items/a_face.rs`, `tests/contract.rs`, and
`tests/items/a_face.rs` are writable. Within `flags/facade/runtime-app`, only
`Cargo.toml`, `BUCK`, `build.rs`, `src/main.rs`, `src/lib.rs`,
`src/items/a_wiring.rs`, `src/test_items/a_face.rs`, `tests/contract.rs`, and
`tests/items/a_face.rs` are writable. Core, L1e ports/adapters, root manifest,
generated third-party/SLO files, owner law, IaC/Observability, and behavior are
frozen.

Build closure: locked/offline metadata; proto package/lint and Cargo/Buck targets
for `flags-runtime-app`, the three `flags-runtime-*-draft` ports, the four new
`flags-*-draft` adapters, evaluation, all L1e ports/adapters, and their empty
contract tests. Required review is Flags plus independent API/architecture,
build, Gateway/protocol, IAM/Policy, Observability, and IaC reviewers; the sold
proto additionally requires every discovered consumer owner under D-29.

Success: one canonical proto and eight unrouted package faces compile through
Cargo and Buck with identical membership; all current vectors/authority/durable
contracts remain green; every handwritten file is at most 300 lines; no request
can be served and no deployment, SLO, or readiness claim exists.

Failure: handler/cache/stream/telemetry behavior lands, a listener starts, a
draft contract leaks to another owner, root members or third-party closure
change, Cargo/Buck membership differs, or generated/manual indexes appear.

Rollback: remove the proto, eight empty faces, and only their lock entries as one
structural revert; no network, deployment, SLO, or durable format state exists.

Fault evidence: proto-package and inverse-consumer checks, add/rename/remove
scanner canaries in both graphs, manual/tracked-index rejection, compile-neutral
unrouted-process proof, before/after L1e closure, and forbidden behavior/IR
fixtures.

## L1f.1 — Implement serving behavior in frozen unique files

Class: content-only behavioral serving; depends on L1f.0.

- Add request-bound default-deny authority and its contract evidence only at
  `flags/ports/draft/runtime-authority/src/{items,test_items}/b_contract.rs`.
  Missing, forged, expired, wrong-audience, or cross-tenant authority fails
  before evaluation or mutation.
- Add bounded telemetry and deployment-intent values only at
  `flags/ports/draft/runtime-telemetry/src/items/b_contract.rs`,
  `flags/ports/draft/runtime-telemetry/src/test_items/b_cardinality.rs`, and
  `flags/ports/draft/runtime-deployment/src/{items,test_items}/b_contract.rs`.
  These are local semantic values, not Observability/IaC engine copies.
- Add the Connect evaluation, control, distribution, and fail-closed readiness
  handlers only as facade items
  `src/items/{b_evaluation,c_control,d_distribution,e_readiness}.rs`; add local
  evidence only as
  `src/test_items/{b_default_deny,c_contract,d_readiness}.rs` and
  `tests/items/b_end_to_end.rs` under `flags/facade/runtime-app`.
- Implement signed stream/resync only in
  `flags/adapters/draft/snapshot-connect/src/items/b_stream.rs`, with evidence
  in `src/test_items/b_stream.rs` and `tests/items/b_faults.rs`; implement the
  atomic cell cache only in existing frozen `snapshot-memory` items
  `src/items/c_cell_cache.rs` and `src/test_items/d_cell_cache.rs`.
- Implement OpenFeature and OFREP mappings only in
  `openfeature-ofrep-compat/src/items/{b_openfeature,c_ofrep}.rs` with
  `src/test_items/b_conformance.rs`. They derive values, reasons, errors, and
  generations from the canonical contract and own no metadata or authority.
- Implement bounded telemetry emission and deployment IR translation only in
  `runtime-telemetry-otel/src/{items,test_items}/b_emitter.rs` and
  `runtime-deployment-iac/src/{items,test_items}/b_intent.rs`. Neither writes a
  generated SLO/deployment file or calls another owner's core.

Closed write envelope: only these 26 unique paths:

```text
flags/ports/draft/runtime-authority/src/items/b_contract.rs
flags/ports/draft/runtime-authority/src/test_items/b_contract.rs
flags/ports/draft/runtime-telemetry/src/items/b_contract.rs
flags/ports/draft/runtime-telemetry/src/test_items/b_cardinality.rs
flags/ports/draft/runtime-deployment/src/items/b_contract.rs
flags/ports/draft/runtime-deployment/src/test_items/b_contract.rs
flags/facade/runtime-app/src/items/b_evaluation.rs
flags/facade/runtime-app/src/items/c_control.rs
flags/facade/runtime-app/src/items/d_distribution.rs
flags/facade/runtime-app/src/items/e_readiness.rs
flags/facade/runtime-app/src/test_items/b_default_deny.rs
flags/facade/runtime-app/src/test_items/c_contract.rs
flags/facade/runtime-app/src/test_items/d_readiness.rs
flags/facade/runtime-app/tests/items/b_end_to_end.rs
flags/adapters/draft/snapshot-connect/src/items/b_stream.rs
flags/adapters/draft/snapshot-connect/src/test_items/b_stream.rs
flags/adapters/draft/snapshot-connect/tests/items/b_faults.rs
flags/adapters/draft/snapshot-memory/src/items/c_cell_cache.rs
flags/adapters/draft/snapshot-memory/src/test_items/d_cell_cache.rs
flags/adapters/draft/openfeature-ofrep-compat/src/items/b_openfeature.rs
flags/adapters/draft/openfeature-ofrep-compat/src/items/c_ofrep.rs
flags/adapters/draft/openfeature-ofrep-compat/src/test_items/b_conformance.rs
flags/adapters/draft/runtime-telemetry-otel/src/items/b_emitter.rs
flags/adapters/draft/runtime-telemetry-otel/src/test_items/b_emitter.rs
flags/adapters/draft/runtime-deployment-iac/src/items/b_intent.rs
flags/adapters/draft/runtime-deployment-iac/src/test_items/b_intent.rs
```

All `Cargo.toml`, `BUCK`, `build.rs`, stable crate/test parents, `a_face.rs`,
facade `a_wiring.rs`, proto, root/lock, L1e files, owner law, `flags/iac/**`,
`flags/observability/**`, and every external owner path are frozen. Any missing
dependency or parent edit stops this lane and dispatches a separate structural
repair.

Build closure: Cargo/Buck library, facade, unit, contract, and integration
targets for all L1f.0 packages; the L1e evaluation/authority/durability closure;
native/proto/OpenFeature/OFREP contract vectors; and dependency checks proving
no direct IAM/Policy/Gateway/Observability/IaC core edge. Required review is
Flags plus independent security, protocol, distribution, performance,
Observability, and IaC reviewers.

Success: authorized requests preserve canonical results and generations;
ordinary evaluation reads one bounded local snapshot with no remote hop;
duplicate/reordered/corrupt streams never expose partial state; compatibility
vectors match; telemetry labels are bounded; the process remains `Unrouted` and
readiness false pending L1f.2.

Failure: caller fields grant authority, control/evaluation share an auth bypass,
evaluation performs I/O, stream work is unbounded, cache publication is partial,
OFREP owns metadata, raw targeting/proof material becomes telemetry, deployment
is applied, or any frozen structure/parent/build path changes.

Rollback: remove only the new unique items; retain the empty L1f.0 faces and
L1e authority/data formats. No routed endpoint or external desired state exists.

Fault evidence: forged/replayed/cross-tenant IAM/PDP/C0, maximum+one bodies and
queue/in-flight bytes, duplicate/drop/reorder/truncate/corrupt stream, resync
storm, atomic-swap interruption, restart on old generation, compatibility
default/error parity, telemetry-cardinality overflow, and unavailable deployment
sink with no readiness or side effect.

## L1f.2 — Obtain D-29 provider integration and route the runtime

Class: escalated provider integration; depends on L1f.1 and is not dispatchable
at L1a because Observability and IaC have not supplied exact sold/generated
consumer paths.

- Keep IAM, Policy, Gateway, Observability, and IaC trees read-only. Their owners
  must first publish and review the exact sold auth/transport, SLO-IR consumer,
  and IaC-reconciler paths. Flags MUST NOT depend on their core crates, guess
  provider paths, or widen this lane into another owner.
- Once those contracts exist, the dispatcher records their exact path/build
  closures and opens the required D-29 provider PRs. Any Cargo/BUCK/lock change
  is a separate structural L1f.2.0; it cannot share a PR with wiring behavior.
- Only after those prerequisites merge may a Flags behavior PR activate
  `flags/facade/runtime-app/src/items/a_wiring.rs`, emit the exact owner-local
  `flags/iac/runtime.textproto` consumed by the IaC reconciler, and feed the
  existing runtime-telemetry port to the provider-owned SLO controller. The
  generated OpenSLO output remains absent until L1f.3.

Closed Flags write envelope after provider ratification:
`flags/facade/runtime-app/src/items/a_wiring.rs` and
`flags/iac/runtime.textproto` only. Provider path sets are supplied by their
owners and remain separate D-29 PRs; all manifests, lockfiles, proto, handlers,
adapters, generated SLO, and other Flags paths are frozen.

Build closure: the entire L1f.1 Cargo/Buck closure plus the exact provider sold-
facade/reconciler targets supplied by their owners and one rendered-but-not-
applied IaC consumer check. Required review is Flags, every provider/consumer
owner, architecture/API, security, Observability, and IaC; no single owner or
hosted check supplies APPROVE.

Success: one default-deny runtime is routed through sold provider boundaries,
the IaC reconciler consumes exactly one typed desired-state IR, telemetry reaches
the provider controller, and readiness still refuses until L1f.3 promotion.

Failure: a direct external-core edge, guessed/unreviewed contract, mixed graph
and behavior diff, hand-authored deployment/SLO output, unrouted telemetry,
provider outage reported healthy, or external owner edit in the Flags PR.

Rollback: remove routing and withdraw the un-applied desired-state IR; retain
the unrouted L1f.0 faces and L1f.1 tested behavior. Once applied, rollback uses
the IaC reconciler and a supported higher desired-state generation, never a
manual cluster edit.

Fault evidence: provider denial/outage, mTLS/audience/tenant substitution,
render rejection, stale desired-state generation, telemetry-controller outage,
and routing restart all keep readiness false and create no bypass or partial
deployment.

## L1f.3 — Qualify deployment, readiness, and SLOs with measured evidence

Class: content/evidence promotion; depends on L1f.2.

- Add only unique promotion tests
  `flags/facade/runtime-app/tests/items/{c_capacity,d_partition,e_restart,f_security}.rs`
  for the declared 70% load envelope, asymmetric partitions, restart/recovery,
  tenant isolation, and prioritized kill propagation.
- Exercise the IaC-rendered cell runtime and live metrics. The Observability
  controller may then materialize exactly
  `flags/observability/slos/runtime.generated.openslo.yaml` from consumed IR;
  authors never hand-edit it. Evidence remains in protected checks and provider
  readback, not a repository evidence dump.
- Advertise readiness and the PRD objectives only after the selected local
  snapshot, sold auth path, routed facade, telemetry producer, SLO consumer, and
  supported desired-state generation all pass together.

Closed write envelope: the four unique promotion-test paths above and the one
controller-materialized generated OpenSLO output only. Runtime source, proto,
manifests, scanners, roots, lockfiles, IaC IR, provider trees, and owner law are
frozen.

Build closure: full L1f.2 graph, generated-artifact freshness/idempotence,
IaC render/readback, Observability SLO consumption, maximum-definition load,
partition/recovery/security campaigns, and compatibility conformance. Required
review is Flags plus independent SRE/performance, security, Observability, IaC,
Gateway, IAM/Policy, and durability reviewers.

Success: cell-local evaluation meets the unqualified PRD objectives at the
declared envelope, ordinary evaluation has no remote hop, generations remain
observable/monotonic, and only produced/consumed capabilities report ready.

Failure: REST/gRPC dual truth returns, stale/corrupt state is hidden, a metric or
deployment is unconsumed, an SLO is hand-authored, load causes unbounded work,
or a provider/fault campaign leaves readiness green.

Rollback: route new traffic to the prior admitted runtime generation through
the reconciler, keep durable definition history readable, and withdraw SLO
qualification until the full producer/consumer/fault gate is green again.

Fault evidence: asymmetric partition; snapshot loss/reorder/corruption; process,
cell, and authority restart; overload; forged auth/C0; telemetry/SLO consumer
outage; IaC rollback; and prioritized kill propagation during ordinary updates.

</sequence>

<parallelism>

The Flags L1 chain is sequential because each slice deletes or stabilizes the
next slice's build and semantic surface. L1f.0 serializes on sold proto and
`Cargo.lock`; L1f.1 releases those hubs and owns only unique files; L1f.2 waits
for provider-owned D-29 paths without occupying them; L1f.3 alone may materialize
the generated SLO output. It may run beside lanes whose changed paths and
practical Cargo/Buck closures exclude the active exact Flags paths, provider
closures, root workspace/admission files, and generated hubs. Owner-law files
retain one writer; cloud CI observation and independent review do not occupy
this worker.

</parallelism>
