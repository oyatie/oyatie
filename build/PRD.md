---
doc_class: Owner-PRD
owner: build
status: Active
date: 2026-08-27
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - build/ADR.md
---

# Build product requirements

<product_boundary>

`build/` is repository meta infrastructure for reproducible toolchains,
pinned host/guest image inputs, and translation of source package declarations
into build-engine inputs. Its first active product slice is deterministic
Cargo/Reindeer-to-Buck declaration reconciliation for tenant #0.

Build is not Pipeline, a CI scheduler, a repository/forge product, Storage,
Compute's fleet agent, a price catalog, or a cloud capability engine. The
source port engine remains frozen and is not the reconciliation implementation.

</product_boundary>

<users>

- Rust maintainers need Cargo manifests and the lockfile to remain the only
  package/dependency declarations they edit.
- Build maintainers need one reviewed fixup mechanism for native sources,
  build scripts, generated files, features, cfgs, and platform differences.
- Pipeline needs a hermetic graph step that checks freshness without learning
  Reindeer internals or creating another policy/census product.
- Reviewers need a receipt that proves which exact inputs, generator, toolchain,
  and bytes produced a candidate `third-party/BUCK`.
- Operators need failures to preserve the last complete graph and identify the
  failed phase without exposing host environment or secrets.

</users>

<landed_scope>

## Current foundation

Cargo manifests and `Cargo.lock` are live. Reindeer configuration and 66 fixup
packages exist, and Buck2 consumes the checked `third-party/BUCK`. The checked
output contains native/build-script behavior beyond raw dependency listing.

Reproduction is not closed: configuration and output headers refer to different
deleted wrappers; clean raw generation encounters unresolved local build-script
fixups; native rules have depended on semantic text rewriting; and there is no
owned atomic publisher, source-bound receipt, or freshness graph step.

`build/port-engine` has fourteen packages but is frozen for its separately
named source-port concern. Build's toolchain/image surfaces are partial and no
qualified image factory has landed.

</landed_scope>

<requirements>

## Declaration inputs

- Read a caller-supplied repository root and explicit paths for the workspace
  manifest, lockfile, Reindeer configuration, fixup root, and output.
- Reject missing, non-regular, symlink-substituted, path-escaped, concurrently
  changed, or internally inconsistent inputs before publication.
- Bind the complete configured platform set and generator pin as inputs rather
  than inferring behavior from the current host.
- Preserve Cargo feature/resolution semantics and Reindeer's crate identity;
  do not invent an owner-local package model.

## Deterministic generation

- Resolve one reviewed Reindeer source and binary identity. Initial
  qualification targets `v2026.08.10.00` at source commit
  `bb681570d2bc47d1446080c12b8681a50a95f628`; a later pin change is a reviewed
  compatibility event.
- Invoke generation with locked, offline inputs, explicit tool paths, an empty
  network surface, and an allowlisted environment independent of the caller's
  machine.
- Express AWS-LC environment/build behavior and per-platform PSM native rules in
  fixups or generator-supported configuration, never an output text patch.
- Run raw generation twice in isolated staging locations and require byte-for-
  byte identity before validating or publishing either result.

## Validation and publication

- Validate the generated file's complete syntax/shape, imports, target
  uniqueness, references, required aliases, fixup effects, and configured
  platform behavior before it can replace the checked output.
- Write a same-directory temporary file, flush it, atomically replace the
  destination, and flush the parent directory. Failures before replacement
  retain the prior destination; observers always see prior or new complete
  bytes, never a partial file.
- Return `Unchanged` when the validated digest already matches and `Replaced`
  only after durable replacement. Never report success for staged bytes.
- Emit a deterministic receipt, separate from generated BUCK bytes, that binds
  repository correlation, every input digest, the generator source and binary,
  Cargo/rustc identities, output digest, validation profile, and publication
  outcome.

## Interfaces and integration

- Keep a pure reconciliation core isolated from process/filesystem concerns.
  Use ports for raw generation and atomic publication, with Reindeer/process and
  filesystem adapters at the edge.
- Expose an internal reconciler facade suitable for a Buck/Pipeline graph
  action. Do not create a public CLI contract.
- Make freshness check-only mode use the same core and generator adapter as
  materialization; drift reports the expected and observed digest and exits
  without modifying the tree.
- Keep Pipeline, Git, GitHub, review, merge queue, CAS, and remote execution
  concepts out of the Build core model.

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
- network operations during reconciliation: **zero**;
- partial or unvalidated publications: **zero**;
- freshness false negatives on an input change: **zero**;
- warm reconciliation p95: **10 seconds or less**;
- receipt/input/output provenance coverage: **100%**.

These are objectives, not current claims. The latency objective is advertised
only after a reproducible benchmark records hardware, cache, workspace scale,
generator pin, and platform set.

</slo_objective>

<acceptance>

## Success

- Two isolated generation passes from identical admitted inputs yield identical
  bytes and one stable receipt identity.
- The validated output builds representative native, proc-macro, platform-
  conditional, optional-alias, and WebAssembly dependency targets with Buck2.
- Check-only mode is clean immediately after materialization and detects any
  manifest, lock, config, fixup, platform, generator, or output drift.
- An unchanged result performs no destination replacement; a changed result
  exposes only the old complete file or the new complete file.

## Failure

- Bare Reindeer output plus an undocumented/manual mutation is required.
- Host environment, network availability, iteration order, temporary path, or
  current platform changes output semantics.
- A failed or interrupted run truncates, partially replaces, or blesses stale
  `third-party/BUCK`.
- The receipt omits an input/tool identity or claims publication before durable
  replacement.
- Reconciliation creates a second package graph, executes CI policy, or edits
  the frozen port engine.

## Named fault campaigns

- Missing/malformed manifest, lock, configuration, fixup, generator, and
  required alias; path traversal, symlink substitution, and input mutation.
- Generator nonzero exit, timeout, signal, oversized stdout/stderr, attempted
  network access, and unallowlisted environment dependence.
- Different first/second output, duplicate/dangling targets, unsupported
  platform branch, and absent native fixup effect.
- Temporary-file create/write/flush failure, destination change race, rename
  failure, parent-directory sync failure, and restart at each boundary.

</acceptance>
