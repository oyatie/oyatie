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

## L1d — Admit bounded definitions before evaluation

Class: narrow behavioral core; depends on L1c.

- Add the `ValidatedFlag` and `ValidatedEvaluationContext` admission boundary
  and exact v1 limits from `SPEC.md` inside `core/evaluation-domain`.
- Validate all identifiers, uniqueness, references, operator shapes, rollout
  totals/overflow, and maximum dimensions before evaluation. Preserve the
  current valid-input precedence and FNV-1a golden assignments.
- Return stable typed validation failures; no malformed definition enters the
  hot path or produces an optimistic evaluation.
- Add unit, boundary, property-style, fuzz-target or deterministic arbitrary-
  input, mutation, and cross-toolchain golden-vector evidence without adding
  network, storage, clock, policy, pack, or runtime dependencies.
- Explicit kill state, C0 overrides, mutation authority, distribution, and
  facade remain later work; do not simulate them with context attributes.

Changed-path envelope: `flags/core/evaluation-domain/**` and its exact Cargo/
BUCK test declarations only. New third-party dependencies or root/lock changes
require a separately reviewed serialized hub slice; prefer owned deterministic
test generation for this first behavior.

Success: every currently valid vector is unchanged; invalid and maximum+one
definitions/contexts fail before evaluation; admitted evaluation is total,
bounded, deterministic, and panic-free under the declared corpus.

Failure: a malformed reference is subject-dependent, current valid subjects
re-bucket, validation adds I/O/nondeterminism, or an oversized tenant input
causes unbounded work/allocation.

Rollback: revert the new validated API before any facade publishes it; retain
the prior kernel and golden vectors.

Fault evidence: empty/duplicate/unknown identifiers, bad operand shape,
overflow/over-100% rollout, every limit boundary and maximum+one, arbitrary
bytes/Unicode, randomized rule order, and repeated/cross-build golden replay.

## L1e — Build authority ports and a durable reference adapter

Class: behavioral authority; depends on L1d.

- Move source/mutation/distribution/audit abstractions to proper `ports/` faces
  and keep core on owned semantic values.
- Implement one durable definition/idempotency/audit-outbox adapter and one
  immutable in-memory snapshot adapter; publish a parameterized conformance
  suite and recovery/upgrade contract.
- Add explicit kill-state and verified C0 override values only with their
  authorization/provenance contract.

Success: mutation replay commits one monotonic generation and atomic snapshot
readers see the prior or verified next generation; core remains I/O-free.

Failure: port traits remain trapped in core/adapter, dual stores become
authoritative, or caller attributes forge policy/override state.

Fault evidence: transaction interruption/reopen/replay, corrupt state,
duplicate/out-of-order generation, forged proof, and kill/update race.

## L1f — Publish Connect facade and cell-local distribution

Class: serving capability; depends on L1e.

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
