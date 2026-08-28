---
doc_class: Owner-PRD
owner: build
status: Active
date: 2026-08-28
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - build/ADR.md
---

# Build product requirements

<product_boundary>

`build/` is repository meta infrastructure for reproducible toolchains, pinned
host/guest image inputs, and source-package translation into build-engine
inputs. Its first active slice is Cargo/Reindeer-to-Buck reconciliation.

Build is not Pipeline, a CI scheduler, repository/forge, Storage, Compute's
fleet agent, pricing, or a cloud capability engine. The port engine stays frozen.

ADR-0719 D-17 also adopts one corpus-free first-party Cargo↔BUCK source engine.
Build owns neutral grammar, relation, violations, and repairs; callers own SCM
snapshots, ownership, application, and campaigns. Package/parser choices remain
design- and supply-chain-gated. No first-party behavior has landed.

</product_boundary>

<users>

- Rust maintainers need Cargo manifests and the lockfile to remain the only package declarations they edit.
- Build maintainers need one reviewed fixup path for native, generated, feature, cfg, and platform effects.
- Orchestrators need consumer-neutral freshness without learning Reindeer or creating another authority.
- Reviewers need exact input, generator, toolchain, and byte provenance for `third-party/BUCK`.
- Operators need typed failures that preserve the last graph without leaking host state or secrets.
- Product owners need an independent tested MSRV floor and current qualified production stable.
- Security needs exact reachability without delegating severity, embargo, disclosure, or CNA duties.
- Package owners need stale, missing, duplicate, or unsupported BUCK declarations caught from either side.
- Repair orchestration needs one canonical V1 set with exact owner groups, preconditions, postimages, and identity.

</users>

<landed_scope>

## Current foundation

Cargo manifests and `Cargo.lock` are live; Reindeer configuration, 70 fixup
packages (66 inherited plus four run-only scanners), and checked
`third-party/BUCK` carry native/build-script semantics.

Reproduction is open: headers name deleted wrappers, clean generation hits
unresolved fixups, native rules used text rewrites, and no qualified publisher,
source-bound receipt, or neutral freshness contract exists.

`build/port-engine` has fourteen frozen packages. Toolchain/image surfaces are
partial: root Rust requires 1.98.0 while Buck, the image recipe, and standards
still name 1.97.1. That is drift, not a compatibility promise.

Six `build/dependency-declarations/**` packages establish Reindeer core, ports,
adapters, and facade shape. They remain structural/refusing; neither behavior
path has landed.

</landed_scope>

<requirements>

## Toolchain and dependency evolution

- Keep four distinct identities: declared MSRV, qualified production stable,
  beta candidate, and an exact dated/committed nightly observation. A floating
  channel name is discovery input, never receipt identity.
- Advance production to the latest qualified stable patch through a reversible
  candidate. Never raise `rust-version` merely because rustc, Cargo, a build
  image, or a dependency moves; test the declared MSRV as a separate matrix.
- Inventory rustc, Cargo, rustfmt, Clippy, LLVM, components, targets, source and
  binary digests, CI pins, Buck toolchains, image builders, Reindeer inputs,
  cache namespaces, and every other consumer before proposing a toolchain bump.
- Ingest release notes and compatibility/security advisories into a versioned
  applicability ledger. Build validates complete coverage and may emit a
  nonbinding recommendation; every item carries a consuming-owner-supplied or
  accepted `ADOPT`, `BENCHMARK`, `DEFER`, or `REJECT`, evidence, MSRV effect,
  affected targets, and a re-evaluation trigger. Build never selects another
  owner's semantic adoption.
- For dependency candidates, bind exact package/source/version/checksum,
  publication age, yank/deletion state, maintainer/provenance changes, feature
  graph, normal/build/dev/proc-macro role, native inputs, duplicate versions,
  license/audit status, and transitive affected targets.
- Normalize aliases among supported RustSec, OSV, CVE, and GHSA records while
  preserving issuing-CNA provenance; ingest upstream security, registry
  yank/deletion, and malicious-package notices without double-counting aliases.
  Build consumes security-owned severity/exploitability/embargo decisions and
  must not claim that advisory ingestion makes Oyatie a CNA.
- Apply an owned publication-age/quarantine policy and explicit emergency-fix
  exception. Do not depend on Cargo's unstable minimum-age or breaking-update
  experiments; upstream may change or remove them.
- Keep beta/nightly as read-only differential shadows over the full workspace,
  representative Buck targets, unsafe/FFI, WASM, proc-macro/build-script, and
  platform matrices. Capture Cargo future-incompatibility reports and bind
  every warning to its inverse dependency/target closure, owner, disposition,
  and remediation horizon. Nightly-only behavior cannot become a production or
  MSRV dependency before stabilization and explicit adoption.
- Produce consumer-neutral candidate changes and qualification receipts. The
  consuming product owner accepts semantic behavior and Pipeline alone owns
  campaign, protected-review, retry, and merge orchestration.

## Declaration inputs

- Read a caller-supplied repository root and explicit paths for the workspace
  manifest, lockfile, Reindeer configuration, fixup root, and output.
- Reject missing, non-regular, symlink-substituted, path-escaped, concurrently
  changed, or internally inconsistent inputs before publication.
- Bind the complete configured platform set and generator pin as inputs rather
  than inferring behavior from the current host.
- Bind an explicit read-only Cargo registry/vendor source snapshot, Cargo and
  rustc binary digests, the effective allowlisted environment, sandbox policy,
  validation profile, and publication capability profile.
- Preserve Cargo feature/resolution semantics and Reindeer's crate identity;
  do not invent an owner-local package model.

## First-party source declarations

- Treat either Cargo or BUCK change as a complete-HEAD first-party graph
  trigger. Deltas attribute findings and repairs, never correctness.
- Parse a closed, versioned unconfigured grammar through maintained Cargo/Starlark
  syntax dependencies behind parser ports and bind all profile identities. Refuse
  every unadmitted form: inability to prove an unknown construct cannot influence
  target identity or dependencies is itself refusal; never classify it harmless.
- Require each first-party BUCK edge to resolve to a unique declared identity in
  the admitted unconfigured source IR and be permitted by Cargo. Coverage applies
  only to participating target/dependency kinds; legitimate subsets pass. Map
  normal, build, dev, optional, target-specific, and path dependency semantics;
  incomplete, unknown, unmapped, malformed, or ambiguous extraction refuses.
- Accept immutable snapshot bytes, changed-path attribution, and ownership facts
  from the caller. Do not invoke Git, resolve owners, execute Buck2, compile a
  candidate, interpret Starlark, mutate files, access a network, or spawn a
  process.
- Emit sorted violations and exactly one canonical `DeclarationRepairSetV1` per evaluation, including zero actions/groups. Bind engine/snapshot/profile/caller owner-authority/ownership-fact provenance; complete reads and `semantic_writes`; exact proposed-path projection; and digest-or-absence plus owner-or-absence on every bound path.
  Owner absence is valid only for non-write reads. `semantic_writes` is sole action authority: one concrete-owner `Replacement` per proposed path and no others; each `Replacement` alone binds its path's complete present/absent postimage and canonical digest. Bind typed postconditions, exact group-output digests, and a whole-set digest/identity over every other canonical field.
  Groups are exactly non-empty groups induced by replacement owners, canonically ordered, with each replacement/path once and disjoint writes; zero actions have zero groups. Refuse absent-owner writes, incomplete facts, precondition mismatch, or empty/extraneous/missing/duplicate/ambiguous/wrong-owner/cross-owner/incomplete/overlapping groups. Snapshot identity is provenance, not a global lock; a disjoint successor requires every semantic and owner precondition to match.
- Exclude generated `third-party/BUCK`/`third-party//`; Reindeer stays their sole path and qualifies before a new parser or package-graph change.

## Deterministic generation

- Qualify reviewed Reindeer `v2026.08.10.00` source `bb681570d2bc47d1446080c12b8681a50a95f628` and binary identity. It is the sole candidate, not qualified; revalidation of eleven inherited and four landed run-only scanner decisions plus clean generation/consumer evidence still block promotion.
- Because that source is binary-only and its graph private, qualification also requires a reviewed producer-side patch/API bound by patch/fork/source/binary digests. Generator code returns one `ReindeerGeneratedArtifactV1` containing its typed graph and bytes rendered from that same graph instance; upstreaming is optional, while Build owns qualification/rollback.
- Bind exact manifest/lock/config/fixup/source/platform inputs; generator source/build toolchain/target/flags/binary, renderer, closed environment, and sandbox. Run locked/offline in clean roots with explicit tools, read-only sources, empty network, and no ambient host state.
- Express AWS-LC and per-platform PSM native behavior in fixups or generator configuration, never a text patch.
- Require two clean runs with identical bytes/full canonical producer DTOs. Refuse duplicates/sort-key collisions before `BTreeSet` loss; equality/digest covers every DTO field, never `Rule::PartialEq`, private introspection, a second invocation, caller-authored expected graph, or text reconstruction.

## Validation and publication

- After Reindeer prerequisite qualification, separately review/exact-pin maintained Meta `starlark_syntax` source, crate version/checksum, and bounded profile behind a distinct Build port. `ReindeerRuleGraphV1` is the primary semantic seam; its parser projection is the independent cross-check; Buck2 is consumer/configured authority. Semantic round trip is graph→Reindeer renderer→BUCK bytes→maintained parser→bounded projection→full equality; unknown/lossy/extra/ambiguous forms refuse. This docs lane adds no dependency, behavior, or qualification.
- Final promotion binds generator/parser/renderer/graph-schema/grammar/platform and Buck2 source/binary/toolchain/cell/config/prelude identities, with representative cquery/build evidence for every tuple. Buck2 is configured authority; parser equality is not configured-graph proof.
- Publish only through a declared filesystem capability profile that has been
  qualified for directory-relative no-follow operations, same-directory atomic
  replacement, and durability sync. Hold an exclusive destination lease or use
  a genuine compare-and-swap primitive; refuse unsupported profiles before
  staging bytes.
- On qualified profiles, return `Unchanged` when the validated digest and mode
  already match and `Replaced` only after replacement and directory sync.
  Never report success for staged bytes or claim durable publication after an
  indeterminate sync result.
- Emit a stable generation identity, separate from generated BUCK bytes, that
  binds every semantic input, tool, environment/sandbox/validation profile, and
  output digest. Emit a separate publication-attempt receipt binding that
  generation identity, destination preimage, publisher profile, and actual
  success, typed failure, or indeterminate replacement/durability outcome.

## Interfaces and integration

- Keep pure core isolated from effects. `GenerationPort` carries one producer artifact; a distinct parser port independently projects bytes. Both fit existing six packages; publication remains separate; add no package/root.
- Expose versioned APIs, declarative resources, and reconciler status for neutral reconcile/check-only behavior. CLIs are retirement-marked local diagnostics; Build does not prescribe Pipeline wiring.
- Make freshness check-only mode use the same core and generator adapter as
  materialization; drift reports the expected and observed digest and exits
  without modifying the tree.
- Keep Pipeline, Git, GitHub, review, merge queue, content-addressable storage,
  and remote execution concepts out of the Build core model. Filesystem
  compare-and-swap remains a publication-port capability, not a core concern.

## Operability and supply chain

- Bound input bytes, generated bytes, subprocess output, runtime, and diagnostic
  length; kill and reap a timed-out generator.
- Record no secrets or unrestricted environment values. Diagnostics identify
  semantic phase, path class, and typed cause with bounded safe context.
- Make the pinned generator source, release asset or reproducible build, and
  binary digest independently verifiable. Network acquisition is a separate
  reviewed pin/update lane, never part of reconciliation.

</requirements>

<slo_objective>

## Initial qualification objective

At the recorded workspace package/target scale and declared warm-cache profile:

- byte reproducibility across two clean runs: **100%**;
- generator-graph/parser-projection equality for qualified tuples: **100%**;
- representative Buck2 consumer evidence for every promoted tuple: **100%**;
- network operations during reconciliation: **zero**;
- partial or unvalidated publications: **zero**;
- freshness false negatives on an input change: **zero**;
- warm reconciliation p95: **10 seconds or less**;
- receipt/input/output provenance coverage: **100%**.
- owner-supplied release-note and supported advisory disposition coverage:
  **100%**;
- unexplained toolchain/dependency pin drift: **zero**;
- declared-MSRV regressions from a production toolchain/dependency bump:
  **zero**;
- pinned-nightly shadow cadence: **at least daily**;
- Build analysis/refusal receipt after a critical affected-component fact: **p95 <= 1 hour**.
- complete-HEAD evaluation from either declaration-side trigger: **100%**;
- false greens and false failures for qualified first-party forms: **zero**;
- repeated violation/repair-set bytes for identical immutable inputs: **100%**;
- repair semantic read/write precondition and postimage coverage: **100%**;
- engine file, process, and network side effects: **zero**.

These are objectives, not current claims. Advertise latency only after a
reproducible benchmark binds hardware, cache, scale, generator, and platforms.

</slo_objective>

<acceptance>

## Success

- Two isolated generation passes from identical admitted inputs yield identical
  bytes and direct generator graph, the independent parser round-trips and
  projects those bytes exactly, and one stable generation identity results;
  publication receipts differ only with bound preimage, profile, or outcome.
- The validated output builds representative native, proc-macro, platform-
  conditional, optional-alias, and WebAssembly dependency targets with Buck2.
- Check-only mode is clean immediately after materialization and detects any
  manifest, lock, config, fixup, platform, generator, or output drift.
- On a qualified publication profile, an unchanged result performs no
  destination replacement; a changed result exposes only the old complete file
  or the new complete file.
- Either declaration-side trigger yields the same complete-HEAD relation;
  legitimate target subsets pass and stale/duplicate/unresolved identities fail.
- Identical first-party inputs and ownership facts yield byte-identical sorted
  violations and canonical owner groups; a disjoint change remains applicable only when
  every declared semantic precondition still matches.

## Failure

- Bare Reindeer output plus an undocumented/manual mutation is required.
- Host environment, network availability, iteration order, temporary path, or
  current platform changes output semantics.
- Generator graph, rendered bytes, maintained-parser projection, round trip, or
  Buck2 consumer evidence disagrees, is lossy, unbound, or self-derived.
- A failed or interrupted run truncates, partially replaces, or blesses stale
  `third-party/BUCK`.
- Generation identity or publication receipt omits a bound input/tool/profile,
  or the receipt claims publication before durable replacement.
- Reconciliation creates a second package graph, executes CI policy, or edits
  the frozen port engine.
- A production bump silently changes MSRV, leaves a pin surface split, consumes
  a floating nightly, omits a release/advisory item, or treats an alias as a
  second vulnerability.
- Build selects a consuming product's semantic adoption instead of validating
  its owner-supplied disposition and evidence.
- First-party evaluation is delta-only, equates package sets, or crosses Reindeer;
  classifying an unrecognized construct harmless without proving it cannot influence
  target identity or dependencies also fails, as do candidate parser/SCM/Buck/effects.
- A repair omits a semantic read/write dependency or complete postimage,
  accepts a digest/absence mismatch, changes under input reordering, or relies
  on a baseline, count, path census, or legacy-violation allowlist.

## Named fault campaigns

- Missing/malformed manifest, lock, configuration, fixup, generator, and
  required alias; path traversal, symlink substitution, and input mutation.
- Generator nonzero exit, timeout, signal, oversized artifact/diagnostic,
  attempted network access, and unallowlisted environment dependence.
- Two-root byte/graph mismatch; unknown parser node; lossy projection; round-trip
  mismatch; same-digest/different-content; parser/profile drift; and Buck2
  consumer failure for each generator/parser/grammar/platform tuple.
- Different first/second output, duplicate/dangling targets, unsupported
  platform branch, and absent native fixup effect.
- Unsupported publication profile, lease acquisition/loss or CAS conflict,
  temporary-file create/write/flush failure, rename failure, parent-directory
  sync failure, and restart at each boundary.
- Stable, beta, or nightly feed rollback; same-version/different-commit nightly;
  compiler miscompilation; Cargo CVE; malicious or yanked transitive package;
  dependency owner change; stale advisory mirror; alias conflict; no fixed
  version; MSRV-incompatible resolution; and one missed pin surface.
- Cargo-only and BUCK-only changes; legitimate subsets; every modeled dependency
  kind; unknown load/macro/mutation/select/control flow; unproved harmlessness;
  duplicate, stale, missing, malformed, and ambiguous identities.
- Parser/profile/prelude/rule identity drift; read/write preimage races;
  ownership reordering; attempted file/process/network effects; and differential
  disagreement with protected Cargo metadata or non-building Buck queries.

</acceptance>
