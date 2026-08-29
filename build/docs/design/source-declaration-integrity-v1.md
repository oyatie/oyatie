---
doc_class: Owner-Design
owner: build
status: Proposed
date: 2026-08-28
base: a355428b265db665a18c29e4fc0a35872fbd0053
revision: 1
---

# Source declaration integrity v1

This design supplies the closed package, semantic-role, Rust-edition, and MSRV
contract referenced by `build/{ADR,PRD,SPEC,PLAN}.md`. It does not declare the
recorded repository inventory conformant, qualify Buck2, or authorize Build to
execute a migration campaign.

## Boundary

Build owns reusable, deterministic analysis and repair generation. A consuming
owner supplies semantic intent, ownership facts, postconditions, acceptance,
and the disposition of ambiguous inventory. Pipeline alone may adopt campaign
execution, canary waves, protected review, halt, repair, rollback, and merge
orchestration.

The relation governs repository source, Cargo and Buck declarations, build
relationships, structural conformance, and generated code transformations. It
does not own production database or schema migrations, customer-data changes,
traffic shifting, service deployment, or runtime cell evacuation. Those systems
may supply receipts without merging their failure domains into this fabric.

Production query, conformance, and repair capabilities are versioned APIs,
declarative resources, and reconcilers. A local CLI is diagnostic scaffolding,
retirement-marked, and never campaign or merge authority.

## Package relation

One retained, buildable first-party Cargo package maps to exactly one Buck
package rooted in the same directory. Every Cargo target and buildable source
maps to exactly one compatible Buck target in that package. A parent `BUCK`
cannot own a child Cargo package. A nested `BUCK` cannot split a crate's source
tree; it is admitted only for a genuinely independent build unit with disjoint
sources and its own identity.

The relation never manufactures a Cargo or Buck package merely to satisfy
itself. Each unmatched unit requires an owner-supplied disposition:

- retain as an independently compiled unit;
- absorb, move, or remove as pre-reorganization inventory; or
- exclude as a proved non-buildable fixture.

Only the retained case produces a colocated `BUCK` repair. A non-Cargo Buck
package must have one closed admitted purpose: toolchain, rule library,
schema/proto, asset, or aggregate. Unknown purpose refuses instead of becoming
a baseline, exception, or micro-package.

For each immutable snapshot, the relation emits the sorted union of discovered
Cargo and Buck package roots. Every root appears exactly once with its evidence
and owner disposition; a missing, duplicate, or contradictory entry makes the
result incomplete and prevents repair. The union and its digest are ephemeral
query output, not a tracked census or count baseline. A newly added, removed,
or moved root therefore cannot disappear behind an old inventory.

## Semantic role proof

Path shape is discovery evidence, not semantic proof. A retained role joins:

- Cargo target kind;
- Buck2 rule kind;
- the actual entrypoint;
- dependency direction and fanout;
- visibility and `within_view`;
- code ownership; and
- serving or deployment relationships where relevant.

Workspace membership, a canonical-looking directory, creation history, or a
vacant `src/main.rs` proves neither a facade process nor any other production
role. Structural conformance does not imply production readiness. Missing or
contradictory role facts yield a typed ambiguity and no package repair.

When reflection, configuration strings, routes, plugins, generated code, or
dynamic dispatch can hide relevant edges, the caller must supply supported
proved or observed facts, explicitly exclude that surface, or accept refusal.
Language and build references alone cannot justify a safety claim on an
incomplete surface.

## Rust policy facts

Canonical first-party Cargo manifests inherit edition 2024 from the workspace.
Canonical first-party Buck Rust targets inherit edition 2024 from the qualified
toolchain default. Per-package copies and overrides are drift; dependency
editions remain upstream facts.

Declared `rust-version` is the package's MSRV contract. The production stable
toolchain, nightly observation toolchain, Cargo version, Buck2 Rust toolchain,
and MSRV are separate typed facts even when two values happen to match. An
update candidate may advance production tooling without silently changing MSRV;
an intentional MSRV change requires its own compatibility evidence and owner
acceptance.

The dependency-evolution intake binds toolchain/channel/version/commit/target,
Cargo and standard-library identities, release-feature inventory, advisories,
CVE/CNA records, lock/declaration drift, and rollback identity. Stable features
are evaluated for owned tooling adoption; nightly observations are evidence,
never a production or MSRV dependency before stabilization and explicit
adoption.

## Recorded census

At exact `origin/dev` `a355428b265db665a18c29e4fc0a35872fbd0053`:

- 32 workspace Cargo package roots lack a colocated `BUCK`;
- 31 tracked Buck package roots lack a colocated `Cargo.toml`;
- three workspace packages resolve to Rust 2021; and
- 39 workspace packages lack a resolved `rust-version`.

These counts are reproducible disposition inputs, not an allowlist, a retained
package list, or an instruction to add 32 build files. Some roots entered during
broad topology work; that provenance does not prove they survived the
reorganization semantically. The owner must classify every item against current
role facts before mutation.

## Analysis and repair contract

The pure relation consumes one immutable complete-HEAD snapshot, one versioned
grammar/profile, and bounded caller facts. It parses each admitted file once and
evaluates the normalized graph in batches. It never spawns a parser, query, or
process per package, target, source, or edge.

Normalized Cargo facts retain package root, target kind, source, entrypoint,
dependency kind and condition, feature/optional/path semantics, effective
edition and origin, and declared MSRV. Normalized Buck facts retain package
root, rule kind, sources, direct edges, visibility, `within_view`, cells and
loads, and qualified toolchain edition. Owner facts carry disposition,
ownership, serving/deployment evidence, exclusions, and provenance.

The engine emits sorted violations and one canonical non-mutating repair set.
Repairs are owner-grouped, preconditioned, bounded, byte-deterministic, and
closure-complete. Ambiguous role, overlapping source ownership, parent/nested
package capture, unmatched target kind, unknown syntax, incomplete facts, or an
unproved hidden edge refuses without a partial repair.

Manual migrations may become adversarial gold-corpus fixtures. They remain
fixtures and do not count as the automated campaign required for activation.
Pre-platform migrations may use ordinary pull requests, but cannot introduce a
second parser, one-off patcher, or competing declaration engine.

## Qualification

Protected qualification independently compares exact locked/offline Cargo facts
with native Buck2 consumers. It batches `buck2 audit file-package`, `uquery`, and
configured toolchain/build evidence by promoted profile. Command success is not
proof when structured output contains a per-path error.

Fixtures cover missing, parent-only, duplicate, and nested-source `BUCK` files;
orphan targets and sources; unjustified micro-packages; pre-reorganization
inventory; vacant entrypoints; structural-but-not-serving units; proved
fixtures; unknown non-Cargo packages; Rust 2021; explicit edition overrides;
absent MSRV; hidden-edge incompleteness; and every bound plus one.

Success is a complete zero-drift relation plus owner-accepted repairs and native
consumer evidence. Failure is any ambiguity, incomplete surface, hand edit,
per-node work amplification, unqualified profile, structural readiness claim,
or baseline/allowlist. Rollback withdraws the unpromoted repair set and restores
the last qualified declaration/profile tuple; no partial package wave is
published by Build.
