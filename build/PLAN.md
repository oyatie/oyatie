---
doc_class: Owner-PLAN
owner: build
status: Active
date: 2026-08-28
---

# Build remaining work

<baseline>

## What has landed

- Cargo manifests and one root `Cargo.lock` declare the Rust workspace and
  dependency solution.
- Reindeer configuration targets the root workspace and fails unresolved
  build-script fixups.
- Seventy package fixups (66 inherited plus four run-only scanners) and a checked
  `third-party/BUCK` are consumed by the Buck graph.
- Fourteen port-engine packages, partial toolchain/cache definitions, and one
  image recipe exist under Build.
- Root Rust declarations and hosted workflows name 1.98.0, while the Buck
  toolchain and distroless image still name 1.97.1. The local floating nightly
  observed on 2026-08-27 is 1.100.0-nightly at rustc commit `bff8e12ff`.
- Several standards still narrate 1.97.1 as current or require the MSRV to equal
  the production pin; those claims are drift to reconcile, not live pin truth.
- `Cargo.lock`, `deny.toml`, and a nonblocking weekly cargo-deny workflow exist;
  the documented `deps.toml`, owned bump bot, and owned supply-chain audit gate
  do not exist at this base.
- An exact 1.100-nightly all-target workspace check passes but reports three
  future-incompatible transitive packages across the application frontend and
  intelligence/Valkey closures; no owner/disposition automation records them.
- Six `build/dependency-declarations/**` packages now establish the Reindeer
  transaction's core, ports, adapters, and facade structure. Core/adapters have
  no behavior and the facade deliberately refuses service.
- ADR-0719 D-17 adopts one corpus-free first-party Cargo↔BUCK source-
  declaration engine. No Build implementation, qualification, drift repair, or
  protected integration has landed.
- Although ADR-0719 D-35 broadly exempts owner law, the live gate excludes the
  `build` meta-root. Keep these files at or below 300 lines and route the
  discrepancy to Pipeline-owner planning rather than editing admission in this
  Build lane.

No supported regeneration entrypoint, complete fixup/overlay inventory,
deterministic double-run gate, validation kernel, qualified publisher,
provenance identity/receipt, or consumer-neutral freshness contract has landed.
The two documented regeneration paths are absent. The port engine remains
frozen. First-party parser, relation, repair-set, and facade behavior are also
absent.

</baseline>

<sequence>

## Finish the third-party pure reconciliation kernel

Class: behavior, test-driven; the reviewed Reindeer scaffold has landed.

- Amend the execution design inside the six packages for a producer-owned exported typed artifact/API, `GenerationPort`, distinct maintained-parser port, full-field DTO/projection, and Buck consumer profile; add no package/root.
- Start RED tests for admission, full-DTO digest/equality, duplicate/sort-key collision/lossy refusal, two-run byte/graph mismatch, graph→renderer→parser→projection equality, bounds, failures, identity, and attempt receipts. Never use `Rule::PartialEq`.
- Implement pure values/transitions only: no process/filesystem/network/Git/Pipeline/Buck/time in core.
- Use one primary item/file and existing generated/globbed membership; do not edit frozen parents/shared declarations.

Closed paths and target names come from the approved declaration-reconciliation
design and remain limited to core items/tests.

- Success: red-green-refactor/property receipts, panic freedom, and identical repeat artifacts/results/identity.
- Failure: adapter leakage, ambient order/path, unbounded input, or tests after behavior.
- Rollback/fault: package-local revert; arbitrary bytes/Unicode, max+1, digest collision, run mismatch, invalid rules/errors.

## Add Reindeer and qualified-publication adapters

Class: behavior at owned ports; depends on the pure reconciliation kernel.

- Land/review a producer-side patch/API against exact pinned Reindeer, binding patch/fork/source/binary digests; propose upstream where useful, but Build owns qualification/rollback. One invocation returns `ReindeerGeneratedArtifactV1` graph and bytes rendered from that graph instance, refusing duplicate keys before `BTreeSet` loss. Implement its adapter with read-only sources, closed env, offline/locked bounds and reaping if isolated; no private introspection, second view invocation, text reconstruction, or bare-stdout contract.
- Implement only qualified lease-or-CAS, directory-relative no-follow, same-directory atomic, sync/recovery publication; unsupported profiles refuse before staging.
- Wire API/declarative-resource/reconciler surfaces for double-run validation/publication/receipts. Local process transports or diagnostic CLIs are retirement-marked, never merge authority.

Closed paths and targets come from the approved design; shared generator,
configuration, and fixup paths remain a separate serialized structural lane.

- Success: fake/real fixtures, unsupported-profile refusal, distinct attempt
  receipts, old-or-new visibility, and honest indeterminate durability.
- Failure: ambient/network/shell influence, semantic output patches, or partial
  visibility.
- Rollback/fault: withdraw unrouted adapters, retaining core and the old graph;
  inject generator/source/environment/profile/lease/CAS/stage/sync/temp/symlink
  failures.

## Replace the broken third-party seam and publish freshness

Class: serialized shared declarations plus consumer-neutral Build handoff;
depends on the Reindeer and publication adapters.

- Convert every live semantic overlay into reviewed fixup/config behavior,
  remove obsolete fixups, set the canonical generated header, and materialize
  `third-party/BUCK` only through the owned reconciler.
- Update `reindeer.toml`, required `third-party/fixups/**`, and the generated
  output in one serialized Build/shared-declaration PR. Never hand-edit the
  generated face.
- Publish and contract-test a consumer-neutral check-only facade with explicit
  inputs, results, failures, and receipts. Stop at the Build boundary; Pipeline
  owner law decides whether and how to schedule or admit it.

The approved provenance/design contract names every shared fixup path. The
generated file is tool-materialized. No Pipeline behavior enters this sequence.

- Success: no old wrappers, one materialize/check engine, and typed receipt-bound
  drift without tree mutation.
- Failure: hidden post-step, Pipeline prescription, manual generated repair, or
  missed input drift.
- Rollback/fault: restore the prior complete tuple; vary each input and inject
  unavailable generator, stale/corrupt output, and incompatible profiles.

## Qualify third-party reproducibility and Buck consumption

Class: verification/promotion; depends on third-party materialization and
freshness.

- From two clean roots, invoke the producer API once/root; compare bytes and every DTO field/digest, refusing duplicate/colliding/lossy graphs even when sort keys compare equal.
- Bind exact inputs, generator source/build/binary, Cargo/rustc, renderer, environment/sandbox, graph schema, grammar/header/import and platform. This Reindeer prerequisite receipt lands before a parser dependency.
- Later exact-pin reviewed `starlark_syntax` behind the distinct parser port; prove graph→renderer bytes→maintained parser→bounded projection→full equality. Producer graph is primary seam, parser projection independent cross-check, Buck2 consumer/configured authority; no Oyatie renderer.
- For every promoted generator/parser/grammar/platform tuple bind Buck2 source/binary/toolchain/cell/config/prelude; build/cquery representative AWS-LC, PSM, alias, proc-macro, generated-source and nine-platform consumers.
- Measure warm-cache SLO; prove check-only clean after generation and red for every admitted input mutation without destination changes.

- Success: byte/full-graph/projection identity, all mappings, Buck consumption, zero network/partial publication, independent approval.
- Failure: host/manual dependence, private/reconstructed graph, divergent/lossy projection, unbound tuple, missed freshness, or unbuildable rules.
- Rollback/fault: reconcile the last qualified tuple; repeat adapter faults plus removed
  platform/alias, native-symbol/build-script corruption, and false negatives.

## Freeze the first-party grammar and parser boundary

Class: owner design and dependency qualification; documentation may proceed
while Reindeer finishes, but no parser/package graph changes may land first.

- Freeze `build/docs/design/source-declaration-integrity-v1.md`: exact packages/targets/ports/adapters/profile, admitted/refused Cargo/BUCK, relation, `DeclarationRepairSetV1`, bounds, fixtures, and root writer.
- Review exact parser source/version/license/maintenance/advisories/audits/transitives/API/pin-update-rollback. No hand parser/interpreter qualifies.
- Success: independent Build/build-system/supply-chain approval plus RED/property/fuzz/differential acceptance before package mutation.
- Failure/fault: vague grammar, unreviewed dependency, set equality, or unproved-harmless unknown influence.

## Implement the complete-HEAD declaration relation

Class: test-driven pure Build behavior; starts after Reindeer qualification and
the source-declaration design, with one serialized package/root writer.

- Begin RED normalized-fact tests for both triggers, complete HEAD, unique declared-identity edge resolution, legitimate subsets, modeled Cargo kinds, and refusal.
- Keep parsing, ownership/SCM/configuration/process/network/mutation/Buck2 outside pure relation.
- Success: identical complete facts yield byte-identical sorted violations;
  arbitrary order, duplicate identities, limit+1, and partial extraction refuse.
- Rollback/fault: revert the isolated package behavior; retain the qualified
  third-party path and replay malformed/unknown relation fixtures.

## Add maintained parsers and the repair-set facade

Class: Build adapters and non-mutating composition; depends on the pure relation.

- Adapt reviewed syntax libraries into complete normalized facts or typed refusal; never execute source.
- Emit one canonical `DeclarationRepairSetV1`, including zero actions/groups, with engine/snapshot/profile/owner provenance, complete reads, and `semantic_writes` as sole action authority. Each proposed path maps bijectively to one concrete-owner `Replacement`, whose complete present/absent postimage and path-bound digest are sole postimage authority. Bind typed postconditions, group-output digests, whole-set digest/identity, exact induced canonical non-empty groups, exact-once membership/disjoint writes; refuse every root-law absent-owner/group/precondition/digest violation. API/declarative reconciler never applies/regroups.
- Property/fuzz tests cover syntax bytes, admitted/refused forms, ordering,
  preimage races, incomplete read sets, output bounds, and forbidden effects.
- Success: repeated immutable inputs produce byte-identical violations/repairs;
  failure includes parser fallback, mutation, SCM/Buck/process/network access,
  Reindeer scope crossing, or any incomplete precondition/postimage.

## Differentially qualify, repair drift, and hand off

Class: out-of-required-path qualification and owner-grouped remediation; depends
on the parser/facade and leaves protected integration to Pipeline.

- Compare with exact `cargo metadata --offline --locked --no-deps --format-version 1` and non-building `buck2 uquery`; only protected out-of-presubmit qualification invokes them.
- Prove every profile form, both triggers, full-HEAD scope, target subsets,
  modeled Cargo semantics, unknown-influence proof/refusal, deterministic
  repairs, precondition mismatch, no effects, and profile requalification.
- Scan current `dev`; hand one neutral V1 set to a separately adopted consumer. Build prescribes no apply/regroup behavior. Repair all findings without allowlists, then rerun.
- Publish a versioned check-only Build facade. Pipeline owner law separately
  selects protected source, trusted layout integration, and the one presubmit.
- Success: no legacy violation, differential disagreement, false green/failure,
  or unqualified profile drift remains. Any one blocks activation.
- Rollback/fault: keep enforcement off, preserve the qualified profile, and replay
  stale/missing/duplicate labels, unproved harmless constructs, and effect traps.

## Reconcile toolchain/dependency lifecycle truth

Class: read-heavy owner design; may overlap disjoint declaration implementation.

- Inventory declared MSRV, stable/beta/nightly identities, every compiler/Cargo/
  component/target/LLVM pin surface, dependency-update policy, advisory source,
  updater/enforcement claim, and future-incompatibility inverse closure.
- Route stale MSRV/pin claims in `code-style-rust.md`, `dependency-policy.md`,
  `lts-versions-verified.md`, and `observability-slo.md` to their owner; complete
  the immutable 1.96–1.100 ledger, keeping 1.99/1.100 provisional.
- Build records only consumer-neutral port needs and routes Security and
  Pipeline contract decisions to those owners.
- Success: every pin/item has provenance, owner, disposition, acceptance, and
  drift state; failure includes floating identity, silent MSRV movement, absent
  machinery narrated as live, or cross-owner claims.
- Faults: same-version/different-commit, advisory withdrawal/modification,
  duplicate aliases, and an omitted release item must fail completeness.

## Implement pure intake, graph-impact, and disposition validation

Class: test-driven Build behavior after reviewed lifecycle and declaration contracts.

- Begin with failing tests for channel identity, MSRV independence, release-item
  and owner-disposition completeness, advisory alias/withdrawal handling,
  dependency role/feature closure, publication quarantine, nonbinding
  recommendations, and deterministic candidate grouping.
- Keep network, SCM, Pipeline, Security policy, clocks, registries, and process
  execution behind ports; consume immutable facts and refuse gaps/conflicts.
- Success: identical bounded facts deterministically yield affected closure,
  candidates, nonbinding recommendations, and refusals, and validate
  owner-supplied dispositions without panic.
- Failure/faults: prose scraping, guessed aliases, `cargo update`, severity
  assignment, Build-selected product adoption, unowned items, feed rollback,
  source substitution, malicious scripts, owner transfer, yanks, MSRV conflict,
  cycles, or missing targets.

## Add mirrored-source adapters and candidate rendering

Class: Build adapters and deterministic transformation; depends on pure intake.

- Materialize signed/digested release, registry, RustSec/OSV/CVE/upstream,
  audit, and toolchain snapshots in a refreshed mirror; qualification is locked
  and offline.
- Render complete toolchain or dependency manifest/lock/fixup/generated-BUCK
  ChangeSets through declaration reconciliation; preserve MSRV and emit
  rollback, SBOM/provenance, release-item, and graph-impact receipts.
- Success: candidates are closure-complete, reproducible, reviewable,
  reversible, and use no competing updater.
- Failure/faults: discovery network leakage, missed pins, Cargo/Buck divergence,
  escaped scripts, mixed semantic/MSRV changes, truncated/stale mirrors,
  deletion, native/feature drift, lock races, or partial publication.

## Qualify stable and run preview shadows

Class: verification and consumer-neutral handoff; depends on candidate rendering.

- Canary the exact stable candidate, then test declared MSRV and stable as
  separate Cargo/Buck/platform/symbol/FFI/WASM/macro/performance/product matrices.
- Run exact beta/nightly identities as non-mutating compiler/diagnostic/format/
  dependency/cache/runtime differentials; emit candidates and receipts only.
  Pipeline owns rollout, review, retry, and merge order.
- Success: stable qualifies or has a typed blocker, MSRV stays green, every item
  is disposed, shadows reproduce, and rollback restores the qualified tuple.
- Failure/faults: nightly authority, algebraic floats on exact/deterministic
  surfaces or without a consuming-owner-accepted error budget, ignored ABI/
  symbol or rebuild impact, Build-run campaigns, miscompilation, nightly syntax,
  solver/Polonius/demangler/WASM/GPU/cache/rollback failures.

</sequence>

<parallelism>

Third-party kernel, adapters, conversion, and qualification are sequential.
First-party design may overlap as read-only work, but parser dependency and
package-graph changes wait for Reindeer qualification; relation, adapters,
differential qualification, drift repair, and handoff then follow in order.
Toolchain analysis may overlap disjoint paths, while one writer serializes every
root manifest/lock, Reindeer/fixup/generated BUCK, toolchain, Buck, and image
pin. Pipeline reconnaissance stays read-only; Build does not prescribe Pipeline
behavior. Port-engine remains frozen. Other work proceeds only with disjoint
changed paths and practical Cargo/Buck closures.

</parallelism>
