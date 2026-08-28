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

The adopted migration-provider boundary also assigns reusable repository
analysis and deterministic transformation machinery to Build. Consuming product
owners retain semantic intent, postconditions, and acceptance; Pipeline retains
campaign and protected-review orchestration. Exact schemas and interfaces are
not adopted or landed by this declaration-integrity slice.

</product_boundary>

<users>

- Rust maintainers need Cargo manifests and the lockfile to remain the only
  package/dependency declarations they edit.
- Build maintainers need one reviewed fixup mechanism for native sources,
  build scripts, generated files, features, cfgs, and platform differences.
- Downstream orchestrators need a consumer-neutral freshness result without
  learning Reindeer internals or creating another declaration authority.
- Reviewers need a receipt that proves which exact inputs, generator, toolchain,
  and bytes produced a candidate `third-party/BUCK`.
- Operators need failures to preserve the last complete graph and identify the
  failed phase without exposing host environment or secrets.
- Product owners need the declared MSRV to remain a tested compatibility floor
  while production builds independently consume current qualified stable Rust.
- Security owners need exact toolchain/dependency reachability and remediation
  candidates without delegating severity, embargo, disclosure, or CNA duties to
  Build.

</users>

<landed_scope>

## Current foundation

Cargo manifests and `Cargo.lock` are live. Reindeer configuration and 66 fixup
packages exist, and Buck2 consumes the checked `third-party/BUCK`. The checked
output contains native/build-script behavior beyond raw dependency listing.

Reproduction is not closed: configuration and output headers refer to different
deleted wrappers; clean raw generation encounters unresolved local build-script
fixups; native rules have depended on semantic text rewriting; and there is no
owned qualified publisher, source-bound provenance identity/receipt, or
consumer-neutral freshness contract.

`build/port-engine` has fourteen packages but is frozen for its separately
named source-port concern. Build's toolchain/image surfaces are partial and no
qualified image factory has landed. Root Rust declarations require 1.98.0,
while `build/toolchains/BUCK` and the distroless image recipe still name 1.97.1;
several standards also narrate 1.97.1 as current. That split is current drift,
not a qualified compatibility promise.

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

## Deterministic generation

- Qualify one reviewed Reindeer source and binary identity. L1c ratifies
  `v2026.08.10.00` at source commit
  `bb681570d2bc47d1446080c12b8681a50a95f628` as the sole implementation
  candidate, not a qualified binary. Promotion remains blocked on the eleven
  measured inherited and four planned scanner fixup decisions, plus clean
  generation and consumer evidence.
- Invoke generation with locked, offline inputs, explicit tool paths, an empty
  network surface, an explicit read-only Cargo source snapshot, and an
  allowlisted environment independent of the caller's machine.
- Express AWS-LC environment/build behavior and per-platform PSM native rules in
  fixups or generator-supported configuration, never an output text patch.
- Run raw generation twice in isolated staging locations and require byte-for-
  byte identity before validating or publishing either result.

## Validation and publication

- Validate the generated file's complete syntax/shape, imports, target
  uniqueness, references, required aliases, fixup effects, and configured
  platform behavior before it can replace the checked output.
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

- Keep a pure reconciliation core isolated from process/filesystem concerns.
  Use ports for raw generation and atomic publication, with Reindeer/process and
  filesystem adapters at the edge.
- Expose consumer-neutral internal reconcile and check-only facades. Build does
  not decide whether or how Pipeline wires either facade into its graph.
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
- Build analysis/refusal receipt after an ingested critical affected-component
  fact: **p95 <= 1 hour**.

These are objectives, not current claims. The latency objective is advertised
only after a reproducible benchmark records hardware, cache, workspace scale,
generator pin, and platform set.

</slo_objective>

<acceptance>

## Success

- Two isolated generation passes from identical admitted inputs yield identical
  bytes and one stable generation identity; publication-attempt receipts may
  differ only with their bound destination preimage, profile, or outcome.
- The validated output builds representative native, proc-macro, platform-
  conditional, optional-alias, and WebAssembly dependency targets with Buck2.
- Check-only mode is clean immediately after materialization and detects any
  manifest, lock, config, fixup, platform, generator, or output drift.
- On a qualified publication profile, an unchanged result performs no
  destination replacement; a changed result exposes only the old complete file
  or the new complete file.

## Failure

- Bare Reindeer output plus an undocumented/manual mutation is required.
- Host environment, network availability, iteration order, temporary path, or
  current platform changes output semantics.
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

## Named fault campaigns

- Missing/malformed manifest, lock, configuration, fixup, generator, and
  required alias; path traversal, symlink substitution, and input mutation.
- Generator nonzero exit, timeout, signal, oversized stdout/stderr, attempted
  network access, and unallowlisted environment dependence.
- Different first/second output, duplicate/dangling targets, unsupported
  platform branch, and absent native fixup effect.
- Unsupported publication profile, lease acquisition/loss or CAS conflict,
  temporary-file create/write/flush failure, rename failure, parent-directory
  sync failure, and restart at each boundary.
- Stable, beta, or nightly feed rollback; same-version/different-commit nightly;
  compiler miscompilation; Cargo CVE; malicious or yanked transitive package;
  dependency owner change; stale advisory mirror; alias conflict; no fixed
  version; MSRV-incompatible resolution; and one missed pin surface.

</acceptance>
