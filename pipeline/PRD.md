---
doc_class: Owner-PRD
owner: pipeline
status: Active
date: 2026-08-28
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md#ADR-0719-D17a
  - pipeline/ADR.md
---

# Pipeline declaration-integrity requirements

<boundary>

## Product boundary

Pipeline turns immutable repository facts into a protected invocation of the
Build-owned declaration contract and, later, reviewable repair proposals. Build
alone owns Cargo/BUCK source semantics, grammar profiles, violations, and
`DeclarationRepairSetV1` construction. Pipeline never becomes a parser,
configured-graph oracle, Cargo/Buck executor, or second CI product.

The current protected layout job is a foundation only. It does not yet invoke
the Build contract, apply repairs, qualify a profile, or enforce a declaration
verdict.

</boundary>

<requirements>

## Repository facts

- Acquire immutable base and complete-HEAD snapshots through an SCM-neutral
  capability; Git is the required current adapter.
- Preserve a bounded, lossless delta for add, modify, delete, rename, copy, and
  entry-kind change. Delta and base facts attribute a trigger or repair shard;
  Build receives complete requested HEAD facts for correctness.
- Bind every semantic and proposed-write path to caller-resolved expected owner
  or absence under one protected owner-authority identity and revision. Missing,
  stale, conflicting, or ambiguous facts refuse.
- Treat candidate bytes, paths, SCM responses, Build output, and metadata as
  untrusted data. Only protected source and a qualified contract identity may
  direct admission.

## Protected admission

- Invoke one versioned Build contract from protected source in the existing
  repository-layout admission path and pass its one result to existing
  `presubmit`.
- Preserve Build's typed violations, refusal, order, provenance, and repair
  values without interpreting their declaration semantics.
- Do not add a workflow, required context, candidate executable, candidate
  compile/test proof, Cargo/Buck invocation, graph claim, corpus, baseline,
  count threshold, or exception inventory.

## ChangeSet application

- Wrap one repair shard in one canonical ChangeSet with complete semantic
  reads/writes, expected digest-or-absence preconditions, deterministic complete
  postimages, postconditions and digests, source/profile provenance, and all
  bound owner facts. A shard is routing metadata, not authority.
- Before future publication, re-read every declared semantic input and owner
  fact from one current immutable snapshot. Any mismatch, invalid postimage, or
  ambiguous owner result refuses without partial publication.
- Permit a disjoint successor only when every declared precondition still
  matches; whole-head inequality alone is neither success nor conflict.
- Publish a future repair only as an isolated protected PR with ordinary review;
  never write directly to `dev` or infer approval from a receipt or CI result.

## Qualification and activation

- Pipeline orchestrates qualification evidence, owner-grouped repair proposals,
  a one-owner canary, clean complete-head replay, then activation. These are
  future stages, not present behavior.
- The separately protected, out-of-presubmit qualification harness owns any
  native differential query. Pipeline consumes its qualification result; the
  required path and Build engine do not execute native tools.
- First activation requires adversarial qualification and deterministic repair
  of all detected current drift, with no grandfathered baseline or count/path
  exception. Profile identity changes require requalification before activation.

</requirements>

<success_and_failure>

## Success and failure

Success after activation means each relevant change receives one protected,
complete-head Build result in existing `presubmit`; all repair preconditions
are complete; and a truly disjoint successor can apply without stale mutation.

Failure includes accepted missing/ambiguous facts, a candidate-selected or
unqualified result, delta-only correctness, a semantic/owner precondition gap,
partial repair publication, an activated known drift, or a second execution,
context, graph, corpus, or inventory plane. Unknown errors refuse rather than
report success.

</success_and_failure>

<non_goals>

## Non-goals

- Define Cargo/BUCK parsing, grammar, labels, dependency categories, or
  configured-graph behavior.
- Build an owned SCM, general repository graph, campaign API, repair CLI, or
  candidate execution environment.
- Claim any implementation, qualification, repair, activation, or enforcement
  has landed merely because this owner law is active.

</non_goals>
