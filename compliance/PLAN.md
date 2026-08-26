---
doc_class: Owner-PLAN
owner: compliance
status: Active
date: 2026-08-26
---

# Compliance remaining work

<baseline>

## L3a evidence

The current owner is not feature-ready.

- Seven workspace libraries exist. Six are outside the D-14/D-19 charter:
  `core/dlp`, `core/dsr`, `core/ediscovery`, `core/retention-dsr`,
  `core/trust-portal`, and `ports/dsr-usecase`.
- `core/retention` is the only retained behavioral seed. It is a deterministic
  compatibility oracle for policies, legal holds, record references, and purge
  decisions; it is not a projection engine or durable retention authority.
- Nine handwritten Rust files total 9,401 lines, and every file exceeds 300
  lines. Structural preparation therefore precedes new behavior.
- The seven packages pass 61 tests at base
  `2ed6af0b7ce0f48d071561c07a7489c0501c30f2` with the locked workspace.
- Buck does not close: root `compliance/BUCK` loads deleted
  `//governance/corpus/extract:yaml_facts.bzl`, and package targets name deleted
  `//libs/data-boundary-kernel:data-boundary-kernel` labels.
- No Compliance Rust code loads root `packs/`, the Cedar text, deployment
  manifests, or the handwritten OpenSLO files. Those files are not runtime,
  deployment, or SLO evidence.
- No external Rust, Cargo, or Buck consumer was found for any of the six burn
  package identities. Internal reverse edges are trust-portal to DSR,
  DSR-usecase to DSR, and eDiscovery/retention-DSR to retention.

The lawful dependency chain is `L3a > L3b > L3c > L3d-S > L3d-P > L3d-E`
with `L3d-F` able to follow the frozen L3d-S contract in parallel with the
engine's later unique-file work. Structural and behavioral changes do not
share a pull request.

</baseline>

<sequence>

## L3a — Owner law and truthful inventory

Class: documentation/decision; this pull request.

Changed-path envelope:

```text
compliance/ADR.md
compliance/PRD.md
compliance/SPEC.md
compliance/PLAN.md
compliance/README.md
```

No Rust, Cargo, Buck, lockfile, root-law, manifest, generated, pack, or runtime
path changes in L3a.

Success: the five owner surfaces agree on current maturity, D-14/D-19 scope,
the exact structural/burn/behavior order, success and failure, target SLOs,
and fault campaigns; path-layout admission accepts the exact diff.

Failure: target prose is presented as landed behavior, an off-charter package
is silently retained/re-homed, a current scaffold is called executable, or a
future lane lacks a bounded path/build closure.

Rollback: revert the owner-law documents. No code, route, data, or format
changes exist in this stage.

Fault evidence: reviewer checks the claims against the exact base tree, missing
pack consumers, package reverse closure, failing Buck parse, and locked Cargo
tests rather than accepting narration.

## L3b — Retention scanner and file-budget preparation

Class: structural; behavior, public identity, errors, validation order, and
parser results stay byte-for-byte/variant-for-variant compatible.

Exact changed-path envelope:

```text
compliance/core/retention/BUCK
compliance/core/retention/build.rs
compliance/core/retention/src/lib.rs
compliance/core/retention/src/items/a_types.rs
compliance/core/retention/src/items/b_policy_and_hold.rs
compliance/core/retention/src/items/c_decision.rs
compliance/core/retention/src/items/d_validation.rs
compliance/core/retention/src/test_items/a_policy.rs
compliance/core/retention/src/test_items/b_hold.rs
compliance/core/retention/src/test_items/c_decision.rs
```

The owned, standard-library-only `build.rs` discovers and lexically sorts
`src/items/*.rs` and `src/test_items/*.rs`, refuses invalid/non-Rust entries,
and writes generated membership only under stable `OUT_DIR` include roots.
Tracked/manual module inventories are forbidden. Buck supplies the same source
sets through `buildscript_run`/`genrule`; Cargo/Buck canaries prove equal
membership and order. The scanner becomes the D-41 pattern for new Compliance
engines.

Logical build closure:

```text
compliance-retention
  <- compliance-ediscovery
  <- compliance-retention-dsr
```

Run locked tests for all three packages plus Cargo/Buck target/build/test
parity and the candidate path-layout application. Root `Cargo.toml` already
discovers packages through the D-8 workspace globs. `Cargo.toml` and
`Cargo.lock` do not change; a dependency-free build script is auto-detected.

Success: every touched handwritten Rust file is at or below 300 lines, the two
build graphs include the same sorted members, the existing three-package
closure produces unchanged public behavior, and injected scanner violations
fail closed.

Failure: a manual inventory appears, Cargo and Buck compile different members,
an item is silently skipped, public type/error/parser identity changes, or a
lockfile/package manifest changes.

Rollback: revert only the split and scanner. The pre-L3b single-file oracle
remains format-compatible and no data migration exists.

Fault evidence: add temporary negative fixtures for duplicate/non-Rust names,
unexpected directories, missing member propagation, and Cargo/Buck membership
drift; differential fixtures prove the old and split oracle agree.

## L3c — Exact off-charter burn and stale-artifact removal

Class: structural deletion and package-lock hop. This is the sole `Cargo.lock`
writer among concurrent Compliance lanes.

Delete the complete six package cones:

```text
compliance/core/dlp/**
compliance/core/dsr/**
compliance/core/ediscovery/**
compliance/core/retention-dsr/**
compliance/core/trust-portal/**
compliance/ports/dsr-usecase/**
```

At the L3a base those cones contain exactly 20 tracked files. Also delete the
unconsumed/stale owner artifacts:

```text
compliance/BUCK
compliance/cedar/policies.cedar
compliance/iac/**
compliance/observability/slos/**
```

At the same base `iac/**` contains 12 files and `observability/slos/**`
contains 13 handwritten OpenSLO files. Preserve `compliance/core/retention/**`,
the owner-law files, `OWNERS`, and root `packs/**`.

Root `Cargo.toml` does not change. Regenerate `Cargo.lock` through Cargo and
verify the delta removes exactly the six local package blocks and their now-
unreachable edges, with no unrelated third-party version churn. There are no
known external package consumers; a fresh reverse-dependency search is a hard
precondition because the tree may advance after L3a.

Logical build closure after deletion is `compliance-retention` plus repository
admission. Buck must parse `//compliance/...` after removing the stale root
loader; L3b must already have corrected the retained package target.

Success: the six identities and every enumerated stale artifact are absent;
retention behavior remains green; workspace metadata and Buck expose only the
retained package; history remains in git rather than being copied into new
owners.

Failure: any removed identity still resolves, a current consumer is broken,
the retention oracle changes, root packs are deleted, or the lockfile includes
unrelated package/version movement.

Rollback: revert the deletion and exact lockfile delta. No replacement route,
data format, or product authority is introduced.

Fault evidence: inject references to each burned package identity and stale
Buck loader and prove admission/build fails; prove the retained oracle's
differential corpus is unchanged before and after burn.

## L3d-S — Freeze the unrouted CaS seed

Class: structural contract; depends on L3c and uses the D-41 scanner from L3b.

Exact changed-path envelope:

```text
compliance/ports/draft/pack-source/Cargo.toml
compliance/ports/draft/pack-source/BUCK
compliance/ports/draft/pack-source/src/lib.rs
compliance/ports/draft/cas/Cargo.toml
compliance/ports/draft/cas/BUCK
compliance/ports/draft/cas/src/lib.rs
compliance/core/evidence-domain/Cargo.toml
compliance/core/evidence-domain/BUCK
compliance/core/evidence-domain/build.rs
compliance/core/evidence-domain/src/lib.rs
compliance/core/evidence-domain/src/items/a_contract.rs
compliance/core/evidence-domain/src/test_items/a_contract.rs
compliance/facade/cas-app/Cargo.toml
compliance/facade/cas-app/BUCK
compliance/facade/cas-app/build.rs
compliance/facade/cas-app/src/lib.rs
compliance/facade/cas-app/src/items/a_projection_api.rs
compliance/facade/cas-app/src/test_items/a_projection_api.rs
Cargo.lock
```

The two owner-local draft ports freeze bounded pack-source and CaS semantic
contracts. `evidence-domain` owns immutable catalog/binding/projection/
manifest identities and state transitions. `cas-app` exposes an unrouted
application contract only. Both multi-file packages install deterministic
compile-time scanners in their first commit. There is no network listener,
filesystem pack fetch, durable store claim, target-owner adapter, or external
consumer in this slice.

Root workspace globs discover the packages, so root `Cargo.toml` does not
change. This lane owns `Cargo.lock`; it serializes after L3c and before any
other workspace-package-identity lane. Cargo and Buck each close the four new
packages plus retained `compliance-retention`, and membership canaries prove
the same scanner source sets.

Success: one versioned contract represents catalog, binding, projection, and
manifest identities; package/source bounds and stable typed failures are
frozen; no route or production claim exists.

Failure: the contract authorizes product requests, reads root packs directly,
owns target workflows, invents a second classification value, gains a foreign
consumer, or Cargo/Buck membership differs.

Rollback: remove the unrouted packages and their exact lockfile blocks. No
published endpoint or durable state exists.

Fault evidence: contract tests reject invalid bounds, tenant/id/generation
mixes, unknown enum values, and changed idempotency fingerprints; scanner
negative fixtures run in both graphs.

## L3d-P — Pack admission and version fencing

Class: behavioral engine; depends on frozen L3d-S identities.

Unique-file envelope:

```text
compliance/core/evidence-domain/src/items/b_pack_admission.rs
compliance/core/evidence-domain/src/test_items/b_pack_admission.rs
```

Implement bounded canonical decoding and verify digest/signature provenance,
namespace/instrument id, schema, plane, dimensions, validity, signer
generation, and compare-and-swap catalog generation. Lower versions and equal
versions with conflicting content fail before candidate or catalog mutation.
No manifest, build file, root pack, or lockfile changes.

Success: valid fixtures admit deterministically and every malformed, unsigned,
digest-mismatched, unsupported, unknown, expired, stale, or conflicting fixture
has a stable typed refusal with no visible mutation.

Failure: fallback parsing admits the current Markdown/YAML scaffold, a stale
version changes state, parser work is unbounded, or Policy evaluation appears.

Rollback: remove the unique files from the scanner-discovered source sets; the
L3d-S contract stays unrouted.

Fault evidence: fuzz/property and fixed corpora cover truncation, duplicate and
trailing fields, depth/item/byte limits, invalid text, signature/digest
corruption, unknown dimensions, and concurrent stale admission.

## L3d-E — Binding, projection, and evidence oracle

Class: behavioral engine; depends on L3d-P.

Unique-file envelope:

```text
compliance/core/evidence-domain/src/items/c_binding.rs
compliance/core/evidence-domain/src/items/d_projection.rs
compliance/core/evidence-domain/src/items/e_evidence_manifest.rs
compliance/core/evidence-domain/src/test_items/c_binding.rs
compliance/core/evidence-domain/src/test_items/d_projection.rs
compliance/core/evidence-domain/src/test_items/e_evidence_manifest.rs
```

Implement deterministic in-memory compare-and-swap oracles for bindings,
target projections, generation-bound acknowledgements, Audit-reference
coverage, manifests, and export admission. The oracle remains unrouted and
cannot claim acknowledged durability. Production Audit/Data/Storage/Policy,
records, Cell-time, IAM, Secrets, and persistence adapters require separate
D-29 provider-owner/architecture envelopes.

Success: replay is byte-deterministic; duplicate receipts converge; stale,
skipped, reordered, incomplete, and foreign-tenant input remains visible and
cannot complete a manifest or target generation.

Failure: Compliance executes retention, fabricates Audit evidence, reports a
gap complete, applies a projection to the wrong tenant/generation, or claims
durability from memory.

Rollback: remove only the unique behavioral files. No route, external owner,
or data-format migration exists.

Fault evidence: kill-point state-model tests, loss/duplication/reorder of
receipts, concurrent bind/revoke, missing Audit ranges, cross-tenant ids,
replay, corrupt snapshots once persistence exists, and noisy-tenant bounds.

## L3d-F — CaS application facade oracle

Class: behavioral facade; may start after L3d-S while L3d-P/E continue because
its source files are disjoint, but it cannot integrate or route before those
engine contracts stabilize.

Unique-file envelope:

```text
compliance/facade/cas-app/src/items/b_authz.rs
compliance/facade/cas-app/src/items/c_handlers.rs
compliance/facade/cas-app/src/test_items/b_authz.rs
compliance/facade/cas-app/src/test_items/c_handlers.rs
```

Implement default-deny application handlers against fake ports: authenticate,
verify Policy provenance, bind tenant/idempotency/admission context, and map
typed engine results. Do not add protobuf generation, Connect routing, gateway
registration, or production adapters here.

Success: tenant zero and ordinary tenants share one contract; forged/expired
Policy evidence, Audit outage, changed fingerprints, and tenant mismatch fail
before disclosure or mutation.

Failure: a private route, caller-asserted authorization, permit/forbid response,
unbounded queue, or external owner import lands.

Rollback: remove the scanner-discovered handler files; the engine contract is
unchanged and no endpoint was published.

Fault evidence: default-deny contract tests cover missing/forged/expired/wrong-
audience decisions, cross-tenant pagination/ids, overload, cancellation, retry,
and Audit unavailability.

</sequence>

<coordination>

## Lane and ownership rules

- L3b is the next executable slice. It may run concurrently with Storage and
  app structural lanes whose changed paths and Cargo closures are disjoint.
- L3c and L3d-S each write root `Cargo.lock`; they serialize with every other
  package-add/delete lane in the monorepo even when source paths are disjoint.
- L3d-P and L3d-F can run in parallel after L3d-S because scanners discover
  unique files and neither writes manifests or the lockfile. L3d-E follows the
  admission contract; integration waits for both engine and facade evidence.
- Any change to root `packs/`, Policy, Audit, Data, Storage, Cell, IAM, Secrets,
  Observability, gateway routing, protobuf roots, or a foreign Cargo/Buck
  consumer is outside this plan's owner-only envelope and needs a separately
  dispatched D-29 path/reviewer envelope.
- Moving the defining `data-classification` Rust values into Compliance is not
  implied. That provider migration needs Data and all direct/reverse consumers
  in a separate decision and cannot run as an L3 cleanup.

## Next dispatch

Dispatch L3b as a behavior-preserving structural worker with the exact ten-path
envelope above. Require before/after public-surface and error/parser fixtures,
Cargo/Buck scanner parity, the three-package reverse closure, candidate path
admission, a signed commit, independent review, and no `Cargo.lock` delta.

</coordination>
