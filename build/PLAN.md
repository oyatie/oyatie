---
doc_class: Owner-PLAN
owner: build
status: Active
date: 2026-08-27
---

# Build remaining work

<baseline>

## What has landed

- Cargo manifests and one root `Cargo.lock` declare the Rust workspace and
  dependency solution.
- Reindeer configuration targets the root workspace and fails unresolved
  build-script fixups.
- Sixty-six package fixups and a checked `third-party/BUCK` are consumed by the
  Buck graph.
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
- Although ADR-0719 D-35 broadly exempts owner law, the live gate excludes the
  `build` meta-root. Keep these files at or below 300 lines and route the
  discrepancy to Pipeline-owner planning rather than editing admission in this
  Build lane.

No supported regeneration entrypoint, complete fixup/overlay inventory,
deterministic double-run gate, validation kernel, qualified publisher,
provenance identity/receipt, or consumer-neutral freshness contract has landed.
The two documented regeneration paths are absent. The port engine remains
frozen. Build's reusable migration-machinery provider boundary is adopted; its
exact semantic contracts are not.

</baseline>

<sequence>

## L1a — Establish Build owner law

Class: documentation/authority only; this is the active lane.

- Add `build/{ADR,PRD,SPEC,PLAN}.md` from exact `origin/dev`.
- Record inherited Build boundaries, current declaration truth, the
  deterministic reconciliation destination, explicit success/failure/SLO and
  fault criteria, the adopted migration-provider and toolchain/dependency
  lifecycle boundaries, and their nonbinding contract details.
- Keep all code, manifests, lockfiles, Reindeer configuration, fixups,
  generated files, port-engine files, and other owners read-only.

Closed write envelope: the four owner-law files only.

- Success: four consistent files, five-field strict law, visibly nonbinding
  schema proposals, and green owner-law/layout checks.
- Failure: claims of landed behavior/adopted schemas, Pipeline prescription,
  shared declaration edits, or a port-engine change.
- Rollback/fault: revert only these docs; hostile review must fail to derive
  implementation authority from proposals.

## L1b — Inventory declaration provenance and drift

Class: read-heavy Build analysis with a bounded owner-local deliverable; depends
on L1a review/merge.

- Enumerate every input and semantic effect from Cargo manifests, lockfile,
  `reindeer.toml`, all fixups, the checked BUCK file, and historical overlays.
- Classify each rule or mutation as raw Reindeer output, declarative fixup,
  obsolete residue, or undocumented post-generation change.
- Re-run clean generation with the proposed pin in an isolated scratch area and
  enumerate every missing/stale fixup, required alias, native branch, generated
  source, and representative consumer.
- Produce a concise owner-local design amendment or PR description; do not add
  an evidence dump or mutate declarations/generated output.

Write envelope: owner docs only when evidence changes law; scratch stays outside
Git and implementation waits for L1c. Re-measure the observed eleven missing
build-script fixups, stale `syn`, AWS-LC environment, and PSM OS overlay.

- Success: every semantic difference has a source and mapped consumer target.
- Failure: unexplained rules, generated-byte edits, or historical authority.
- Rollback/fault: no published state; scratch perturbations of a fixup, alias,
  and platform must be detected.

## L1c — Ratify structure and the reconciliation contract

Class: structural design and shared-path escalation; depends on L1b.

- Write an independently reviewed implementation plan naming every new package,
  source/test file, Buck target, Cargo member implication, and exact verification
  command before code changes.
- Prefer `build/dependency-declarations/{core,ports,adapters,facade}`; verify the
  live layout grammar, workspace globs, package naming, and parent-index rules.
  Route any admission need to Pipeline owner planning; it is outside this
  Build implementation path.
- Ratify the Reindeer pin and freeze public request/result/error schemas, v1
  bounds, exact Rust triples and Buck configuration mappings, read-only Cargo
  source snapshot, environment/sandbox/validator/publisher profiles, stable
  generation identity, publication-attempt receipt, and fake adapter seams.
- Assign one serialized structural worker for shared `Cargo.toml`, `Cargo.lock`,
  `reindeer.toml`, `third-party/BUCK`, fixup, or workspace/Buck declarations.

Nothing becomes executable before plan approval. Build stops at its neutral
facade/receipts and one writer owns shared declarations.

- Success: file-exact, budgeted plan; no port-engine edge; independent Build,
  architecture, and build-system review.
- Failure: guessed placement, multiple writers, Pipeline edits, or proposed
  semantic dependencies.
- Rollback/fault: remove unconsumed scaffolding; fixtures reject port-engine,
  root dumps, manual parent indexes, and cross-owner drafts.

## L1d — Implement the pure reconciliation kernel

Class: behavior, test-driven; depends on L1c structural merge.

- Start with failing tests for request admission, digest/canonical ordering,
  two-run mismatch, generated-graph validation, bounds, stable failures,
  generation identity, and publication-attempt receipt construction.
- Implement only pure values and state transitions. No process, filesystem,
  network, Git, Pipeline, Buck execution, or wall-clock access enters core.
- Use one primary item per file and generated/globbed membership established by
  the structural lane; do not edit frozen parents or shared declarations.

Closed write envelope and target names come from the approved L1c plan and are
limited to new core items/tests.

- Success: red-green-refactor/property receipts, panic freedom, and identical
  results/identity for repeat requests.
- Failure: adapter leakage, ambient ordering/paths, unbounded input, or tests
  written after behavior.
- Rollback/fault: one package-local revert; exercise arbitrary bytes/Unicode,
  maximum+one, digest collision, run mismatch, and invalid rules/errors.

## L1e — Add Reindeer and qualified-publication adapters

Class: behavior at owned ports; depends on L1d.

- Implement the process adapter for the ratified Reindeer, explicit read-only
  Cargo source snapshot and environment, offline/locked/stdout generation,
  time/output bounds, and process reaping.
- Implement only qualified filesystem capability profiles, using an exclusive
  destination lease or genuine compare-and-swap, directory-relative no-follow
  operations, same-directory atomic replacement, file/parent sync, and restart
  cleanup. Refuse unsupported profiles before staging.
- Wire an internal facade that generates twice, validates, publishes, and emits
  the stable generation identity and publication-attempt receipt. It is not a
  user-facing CLI.

Closed write envelope and build targets come from L1c; shared generator/config/
fixup paths remain a separate serialized L1e structural sublane.

- Success: fake/real fixtures, unsupported-profile refusal, distinct attempt
  receipts, old-or-new visibility, and honest indeterminate durability.
- Failure: ambient/network/shell influence, semantic output patches, or partial
  visibility.
- Rollback/fault: withdraw unrouted adapters, retaining core and the old graph;
  inject generator/source/environment/profile/lease/CAS/stage/sync/temp/symlink
  failures.

## L1f — Replace the broken seam and publish a freshness contract

Class: serialized shared declarations plus consumer-neutral Build handoff;
depends on L1e.

- Convert every live semantic overlay into reviewed fixup/config behavior,
  remove obsolete fixups, set the canonical generated header, and materialize
  `third-party/BUCK` only through the owned reconciler.
- Update `reindeer.toml`, required `third-party/fixups/**`, and the generated
  output in one serialized Build/shared-declaration PR. Never hand-edit the
  generated face.
- Publish and contract-test a consumer-neutral check-only facade with explicit
  inputs, results, failures, and receipts. Stop at the Build boundary; Pipeline
  owner law decides whether and how to schedule or admit it.

Shared write envelope is fixed by L1b/L1c and names every fixup. The generated
file is tool-materialized. No Pipeline path or behavior enters this sequence.

- Success: no old wrappers, one materialize/check engine, and typed receipt-bound
  drift without tree mutation.
- Failure: hidden post-step, Pipeline prescription, manual generated repair, or
  missed input drift.
- Rollback/fault: restore the prior complete tuple; vary each input and inject
  unavailable generator, stale/corrupt output, and incompatible profiles.

## L1g — Qualify reproducibility and Buck consumption

Class: verification/promotion; depends on L1f.

- From two clean isolated roots, run the same pinned generator and reconciler;
  compare bytes and generation identity.
- Build and cquery representative AWS-LC, PSM, optional-alias, proc-macro,
  generated-source, Windows, Linux GNU/musl, macOS, and WebAssembly targets.
- Record the warm-cache performance profile and qualify SLO language only if
  the measured envelope meets `PRD.md`.
- Prove check-only is clean after generation and red for each admitted input
  mutation without changing the destination.

- Success: byte identity, all mappings, representative Buck consumption, zero
  network/partial publication, and protected independently approved admission.
- Failure: host/manual dependence, divergent identity, missed freshness,
  unbuildable rules, or unmeasured latency claims.
- Rollback/fault: reconcile the last qualified tuple; repeat L1e plus removed
  platform/alias, native-symbol/build-script corruption, and false negatives.

## L2a — Reconcile toolchain/dependency lifecycle truth

Class: read-heavy owner design after L1a; may overlap disjoint L1b scratch work.

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

## L2b — Implement pure intake, graph-impact, and disposition validation

Class: test-driven Build behavior after reviewed L2a and L1 contracts.

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

## L2c — Add mirrored-source adapters and candidate rendering

Class: Build adapters and deterministic transformation; depends on L2b.

- Materialize signed/digested release, registry, RustSec/OSV/CVE/upstream,
  audit, and toolchain snapshots in a refreshed mirror; qualification is locked
  and offline.
- Render complete toolchain or dependency manifest/lock/fixup/generated-BUCK
  ChangeSets through L1; preserve MSRV absent separate acceptance and emit
  rollback, SBOM/provenance, release-item, and graph-impact receipts.
- Success: candidates are closure-complete, reproducible, reviewable,
  reversible, and use no competing updater.
- Failure/faults: discovery network leakage, missed pins, Cargo/Buck divergence,
  escaped scripts, mixed semantic/MSRV changes, truncated/stale mirrors,
  deletion, native/feature drift, lock races, or partial publication.

## L2d — Qualify stable and run preview shadows

Class: verification and consumer-neutral handoff; depends on L2c.

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

L1a–L1g are sequential gates because each freezes the next declaration and
publication contract. L2a read-only analysis may overlap L1b, but L2 behavior
waits for the L1 declaration contract and serializes every shared toolchain,
manifest, lock, Reindeer, fixup, generated BUCK, workflow, Buck, and image pin.
Read-only Pipeline reconnaissance may proceed, but no Build PR prescribes
Pipeline behavior. Port-engine remains read-only throughout. Other owner work
may proceed only when its changed paths and practical Cargo/Buck closures do
not overlap the active lane.

</parallelism>
