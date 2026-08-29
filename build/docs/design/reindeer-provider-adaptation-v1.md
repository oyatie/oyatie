---
doc_class: Owner-Design
owner: build
status: Proposed
date: 2026-08-28
base: a355428b265db665a18c29e4fc0a35872fbd0053
revision: 1
---

# Reindeer provider adaptation v1

This design closes the producer gap in the declaration-reconciliation plan. It
does not qualify Reindeer, add a new external package or version, change
generated BUCK, add the maintained BUCK parser, or create a general patch
platform. Authority remains `build/{ADR,PRD,SPEC,PLAN}.md`.

## Decision

Reindeer remains a replaceable Cargo-to-Buck provider. Build adapts an exact
upstream source snapshot with a deterministic Rust recipe, builds the adapted
source, and consumes one typed artifact from each whole-workspace invocation.
A textual patch may be rendered for review, but it is derived evidence and is
never an authored input or checked-in source of truth.

The recipe belongs to the existing
`build/dependency-declarations/adapters/generation-reindeer` package. Its
upstream Rust syntax and edit roles are provider-specific and do not inflate
the pure reconcile core into a general patcher. No seventh package, fork
repository, vendored Reindeer tree, shell wrapper, or user CLI is introduced.

## Batch contract

`ProviderSourceSnapshotV1` binds the upstream repository, tag, revision, source
tree digest, bounded touched-file bytes and digests, Rust syntax-provider
identity, and recipe identity. `ProviderSourceRecipeV1` consumes that one
snapshot and emits one `ProviderSourceAdaptationV1` containing:

- complete preimages and postimages for every touched file;
- a sorted, non-overlapping semantic edit set;
- the complete admitted `Rule` variant and field inventory;
- the generated artifact API and transport postimages;
- one digest over the complete adapted batch; and
- one deterministic adaptation receipt over every preceding field.

The source-tree identity remains caller-supplied provenance at this pure
adapter boundary. The later hermetic build transaction independently verifies
the complete staged source tree and records its digest before compilation; an
adaptation receipt alone is not source-tree proof.

The adapter parses each touched Rust file once per candidate. It does not start
a process, parser, RPC, or query for a rule, package, field, or edge. Applying
one recipe is linear in admitted source bytes plus emitted bytes, with explicit
byte, file, edit, variant, field, and diagnostic bounds.

After one adapted binary is built, core requests exactly two whole-workspace
generations: run A in one clean root and run B in another. Each invocation
returns one producer-framed `ReindeerGeneratedArtifactV1` containing a concrete
closed `ReindeerRuleGraphV1`, BUCK bytes rendered from that graph instance, and
an invocation receipt. Reusing an observation or receipt across A and B
refuses. The maintained parser later parses each artifact once, and Buck2
qualification batches representative consumers by promoted profile.

## Source and schema adaptation

The Rust syntax adapter must use one exact, reviewed maintained parser behind
the adapter boundary. The recipe identifies items by parsed role and signature,
not line numbers or free-form search. It verifies the expected `Rule` enum,
every referenced rule-struct field and type, sort-key construction,
`do_buckify` collection point, renderer call, and CLI dispatch seam before
emitting edits.

The generated graph schema is exhaustive over the admitted upstream inventory.
Every variant and emitted semantic attribute/value is encoded with a named,
closed representation; private generation-only fields remain in the complete
source-schema proof rather than being mistaken for BUCK semantics. Unsupported
forms refuse. There is no catch-all value, debug-string encoding, duplicated
hand-maintained field map, or fallback to rendered text. Duplicate or colliding
rule sort keys refuse before any set can discard them.

Generated Rust must look like maintained Rust: semantic identifiers, small
modules, exhaustive matches, explicit errors, ordinary control flow, and short
contract rustdoc. The in-process recipe emits deterministic, pinned
`prettyplease`-canonical intermediate bytes. A `rustfmt`-clean promotion claim
waits for the later hermetic build transaction to bind the exact rustfmt
binary, toolchain, configuration, host profile, whole batch, and final receipt;
ambient-PATH or hand formatting cannot rewrite recipe evidence.

Clippy qualification is a structured pristine-versus-adapted differential.
Existing upstream diagnostics remain visible but do not authorize a new
diagnostic identity or multiplicity in a touched file, and generated files
admit none. Comment blobs, comment-controlled behavior, opaque metaprogramming,
and generated boilerplate that obscures the contract fail qualification.

## Update and retirement loop

The weekly Reindeer candidate is an immutable source snapshot. The recipe runs
in check-only mode first. An unchanged supported shape yields the same semantic
adaptation; a changed variant, field, signature, dependency, or output produces
a typed refusal and a bounded review item. Automation never fuzzily reapplies a
stale edit.

The generic artifact export should be proposed upstream. Oyatie does not depend
on upstream acceptance: until a native qualified API exists, the generated
adaptation remains reversible and bound to its source and recipe identities.
When upstream supplies an equivalent API, differential qualification must show
equal graph, bytes, and consumer evidence before the recipe is retired.

## Acceptance and faults

Success requires one source batch, one adapted tree, one binary, two distinct
whole-graph invocations, byte/full-graph equality, bounded resource use, and no
authored patch artifact. Reordering source discovery or recipe facts must not
change postimages or receipt identity.

Tests must inject a missing or renamed item, new `Rule` variant, added/changed
field, sort collision, overlapping edit, stale preimage, parser disagreement,
unformatted or warning-producing output, repeated receipt, split-output mode,
limit plus one, and upstream source rollback. Every case refuses without a
partial tree or a claimed generation proof.

## Delivery slices

1. Land this owner design and remove the caller-fabricated proof surface.
2. Review and exact-pin the Rust source-recipe providers, then TDD the bounded
   whole-batch Reindeer recipe and check-only weekly candidate path.
3. Build the adapted source and prove distinct A/B producer invocation receipts.
4. Add the maintained BUCK parser projection and per-profile Buck2 consumer
   qualification; neither is part of the source patcher.

Current Buck qualification is correctly blocked before compilation: the
generated `third-party/BUCK` contains `prettyplease-0.2` and `proc-macro2-1`
with private visibility, while its Syn, proc-macro2, and quote versions lag
`Cargo.lock`. The source recipe records this as declaration-reconciliation
input. It does not broaden visibility or hand-edit generated third-party rules.
