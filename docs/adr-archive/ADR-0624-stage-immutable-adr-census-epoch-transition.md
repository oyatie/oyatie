---
id: ADR-0624
title: "Stage the immutable ADR census epoch transition"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-07-24
door: one-way
owner: cloud-ci-platform
supersedes: []
superseded_by: [ADR-700]
amends: [ADR-0515]
amended_by: []
depends_on: [ADR-0515, ADR-0613, ADR-0619]
related: [ADR-0525, ADR-0552, ADR-0595, ADR-0597, ADR-0623]
related_specs: [/registry/adr-census-epoch/control-plane.json, /registry/adr-census-epoch/OWNERS, /specs/adr-census-epoch-control-plane.schema.json, /specs/adr-census-epoch-receipt.schema.json]
milestone: W0
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0624: Stage the immutable ADR census epoch transition

## Frontmatter

| Field | Value |
|---|---|
| **Id** | ADR-0624 |
| **Title** | Stage the immutable ADR census epoch transition |
| **Status** | Accepted |
| **Date** | 2026-07-24 |
| **Supersedes** | - |
| **Superseded-by** | - |
| **Owner** | `cloud-ci-platform` |
| **Related** | ADR-0525, ADR-0552, ADR-0595, ADR-0597, ADR-0623 |
| **Bominal source** | no Bominal equivalent |

## Status

**Accepted — 2026-07-24 (founder-ratified; door: one-way).**

This acceptance governs only the immutable ADR-census epoch transition inside
ADR-0515's protected `oya-ci-required` mechanism. It does not accept, supersede,
or operationalize Proposed ADR-0623. ADR-0623 remains nonbinding context only.

`planning_impact: false` is binding. The transition and either epoch's receipt
remain under `BLOCKED/HOLD`. This ADR does not satisfy Stage-1 control C12, lift
`HOLD(Planning)`, approve a plan, authorize a roadmap, dispatch implementation,
or establish legal, JCR, affected-party, operations, custody, veto, pilot,
council, dissent, or context-free-exit authority.

## Context

The protected cloud-CI gate currently requires a fixed historical P2 ADR-census
receipt. P2 binds an exact historical corpus and parser source set. A current
parser change must not make that historical receipt execute current parser code
while claiming the old parser's identity. Metadata compatibility is not
execution identity: exact P2 replay must run the exact historical implementation
and dependencies stored in Git history.

Keeping the old source or receipt in a readable `archive`, `_archive`,
`historical`, or similar directory would leave stale bytes in ordinary
tree-based agent discovery. Git history is the only archive for the retired P2
implementation and old named receipt. The current tree may retain only neutral
predecessor digests and the protected mechanism needed to execute P2 while P2
remains active.

The successor also cannot bind its core identity to its own candidate commit or
the repository's full tree. A squash merge rewrites the commit while preserving
relevant content, and a full-tree binding changes when the receipt, pointer, or
unrelated file changes. Both shapes create self-reference or false churn.
Likewise, a candidate-editable descriptor must not select its own parser,
selector, executor, or predecessor policy.

## Decision

### 1. Use a four-step protected merge train

The epoch transition proceeds in exactly this order. Every step is a separate
protected pull request against `dev`, with an SSH-signed commit, independent
review, resolved threads, and a green `oya-ci-required` context on the exact
candidate head.

1. **Bootstrap the generic epoch producer and gate.** Protect the generic
   producer, validator, receipt schema, pointer schema, and the P3 policy in
   cloud-CI. Keep `active_epoch` at `P2`. Produce P2 by executing the exact
   historical P2 implementation and dependency graph from Git history. Register
   P3 as dormant; no candidate can select or execute it yet.
2. **Protect the parser and authority-envelope change.** Admit the parser and
   envelope in a later PR while historical P2 replay remains the active census
   gate. This prevents a parser PR from judging itself and ensures P2 remains
   reproducible during parser promotion.
3. **Activate P3 with a pointer-only PR.** After the parser change is promoted
   and its protected admission evidence is complete, a later PR may change only
   the source pointer from `P2` to `P3`. It must not modify the producer, gate,
   parser, selector, receipt schema, predecessor policy, or qualification rules.
   Protected-parent code validates the exact pointer-only delta and derives the
   P3 receipt.
4. **Remove P2 replay after promoted P3 proof.** Only after the promoted
   pointer-only activation has a valid P3 receipt and post-merge
   `oya-ci-required` proof may a later PR remove P2 replay code, P2-only
   constants, the retired P2 operation, and obsolete compatibility tests. The
   old implementation and named receipt remain available only through Git
   history.

Failure at any step stops the train. A later step cannot be combined with,
backfilled into, or used to waive an earlier step.

### 2. Keep the mutable control plane pointer-only

`registry/adr-census-epoch/control-plane.json` selects only the active protected
epoch. Its mutable decision field is `active_epoch`. Fixed schema identity,
canonical control name, and the generated receipt path may accompany that
pointer, but the control plane must not carry candidate-editable parser,
selector, executor, toolchain, receipt-policy, or predecessor descriptors.

The protected producer and gate own the generic P3 policy:

- direct-child `docs/decisions/ADR-*.md` selection;
- the canonical ADR parser API, version, and diagnostic policy;
- the Buck2 target, configuration, source inputs, and toolchain inputs;
- the P3 receipt schema and `BLOCKED/HOLD` claim ceiling;
- the exact P2 predecessor digest tuple; and
- the rule that P3 remains dormant until the pointer-only activation.

The producer derives identities from protected source bytes and declared Buck2
inputs. It never trusts a candidate-supplied descriptor for those identities.
This separation prevents descriptor laundering.

### 3. Make the P3 receipt core content-addressed and squash-stable

The canonical P3 receipt core binds:

- receipt schema and epoch;
- selector identity and deterministic ordered corpus-content digest;
- parser source-set digest;
- producer and gate source-set digest;
- declared Buck2 action, configuration, and toolchain-input digest;
- the neutral P2 predecessor digest tuple; and
- the `BLOCKED/HOLD` claim ceiling.

The neutral P2 predecessor tuple is:

| Field | Digest |
|---|---|
| `outer_sha256` | `c3c4195f440fbf7825101dcf303fea9d8aec9d2ce7a77bd3ec25d8411dfdf528` |
| `canonical_digest` | `7a8eb3848e3b5d1dd148595b5210f2a059fac582db9e5607cf54be2f502b24d8` |
| `aggregate_fold` | `2aeb7459f61b6f216b4eee75164bcfb85e405bbe8ca74cf180e5492b09c99507` |

The core excludes the candidate commit, promoted commit, full repository tree,
worktree state, branch name, event payload, and generated face bytes as identity
inputs. Protected admission and post-merge evidence may record exact commits and
runs outside the receipt core, but those observations do not change or
self-authenticate the content-addressed core. Relevant source bytes that are
identical before and after squash therefore produce the same core identity.

### 4. Preserve the history-only and generated-face boundaries

The P2 implementation is resolved from exact immutable Git objects and executed
in an isolated historical worktree or equivalent exact-tree boundary. It never
falls back to current parser code. The implementation, its source tree, and the
old named receipt are not copied into a live archive directory.

The active epoch face is
`ci/facade/artifact-inventory-registry/adr-census-epoch-receipt.generated.json`.
It is ignored, controller-materialized, and never hand-edited, force-added, or
used as source authority. The canonical Buck2 producer materializes it, and the
protected gate validates it. Missing history, missing source objects, source
digest drift, empty output, nondeterminism, pointer-policy mismatch, or receipt
drift fails closed.

### 5. Roll back in dependency-safe order

Before P3 activation, revert the latest train PR normally and leave P2 active.
If the pointer-only activation fails, revert only the pointer from `P3` to `P2`;
the protected P2 replay remains available because step 4 has not run.

After P2 cleanup, rollback cannot point directly to removed code. First admit a
protected PR that restores the exact P2 replay mechanism from Git history and
proves its fixed receipt. Then admit a separate pointer-only `P3` to `P2`
reversion. Never repair an epoch failure by editing the generated receipt,
loosening its schema, accepting current-code replay under historical identity,
or restoring a readable archive directory.

Forward repair follows the same dependency order: producer and gate, parser,
pointer activation, then predecessor cleanup.

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | BNF v4.1 name | Layer |
|---|---|---|---|
| `docs/decisions/ADR-0624-stage-immutable-adr-census-epoch-transition.md` | create Accepted decision | — | — |
| `registry/adr-census-epoch/control-plane.json` | create pointer-only source control | — | — |
| `registry/adr-census-epoch/OWNERS` | create narrow ownership marker for the census epoch source control | — | — |
| `specs/adr-census-epoch-control-plane.schema.json` | create fail-closed pointer schema | — | — |
| `specs/adr-census-epoch-receipt.schema.json` | create fail-closed receipt schema | — | — |
| `ci/facade/scm-facts-snapshot/src/lib.rs` | add generic P2/P3 producer and exact historical replay | existing `scm-facts-snapshot` | `facade` |
| `ci/facade/scm-facts-snapshot/src/retirement.rs` | add `CanonicalIgnoredGeneratedWriter` and canonical ignored-output writer API used by epoch receipt materialization | existing `scm-facts-snapshot` | `facade` |
| `ci/facade/scm-facts-snapshot/src/bin/adr-census-epoch-receipt-gate.rs` | replace fixed-name P2 gate with epoch gate | existing `scm-facts-snapshot` | `facade` |
| `ci/facade/scm-facts-snapshot/BUCK` | bind epoch producer, gate, and declared inputs | existing `scm-facts-snapshot` | `facade` |
| `ci/facade/scm-facts-snapshot/tests/snapshot_integration.rs` | add train, replay, activation, and rollback regressions | existing `scm-facts-snapshot` | `facade` |
| `ci/facade/generated-artifact-freshness/src/lib.rs` | materialize and freshness-check the active epoch face | existing `generated-artifact-freshness` | `facade` |
| `ci/facade/baseline-ratchet/tests/gate_registration.rs` | prove protected Buck2 gate registration | existing `baseline-ratchet` | `facade` |
| `.github/workflows/oya-ci-required.yml` | relabel the existing matrix entry; no new required context | — | — |
| `.gitignore` | replace the retired named face with the epoch face | — | — |
| `registry/generated-artifact-control-plane.json` | register the controller-materialized epoch face | — | — |
| `registry/artifact-capabilities-registry.json` | register the source control and schemas | — | — |
| `registry/BUCK` and `specs/BUCK` | export declared controller inputs | — | — |
| `specs/root-hub-pointers.json` | register direct discovery pointers | — | — |
| `docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md` | amend the existing single protected-context decision for the epoch gate relationship | — | — |
| `docs/ADR-INDEX.md` | producer-generated projection update | — | — |
| `docs/machine-readable/decisions.json` | producer-generated projection update | — | — |
| `docs/CHANGELOG.md` | record the Tier-1 decision lifecycle event | — | — |

This decision adds no product crate, service, runtime endpoint, CLI authority,
roadmap item, or reorg destination. Existing `cloud/*`, `oya/*`, `cloud-*`, and
`oya-*` names remain reorg source inventory rather than presumed final
boundaries.

### Integration via Workflow and Ontology

Not applicable. The census epoch is a protected CI evidence mechanism. It emits
no product Workflow event and reads or writes no Ontology object. The only
operational integration is the existing ADR-0515 `oya-ci-required` gate.

### Positive

- Historical P2 identity is truthful because historical code executes it.
- Parser work can proceed without weakening or rewriting the protected P2 gate.
- Pointer-only activation prevents a successor from selecting its own rules.
- Content addressing survives squash merges and unrelated tree changes.
- Git-history-only retirement removes stale bytes from ordinary agent reads.

### Negative

- The transition requires four protected PRs rather than one combined change.
- Full Git history is required while P2 is active and for any later P2 restore.
- P3 cannot activate immediately after its producer or parser is authored.
- The receipt proves only census mechanics and cannot advance Stage-1 closure.

### Operational

- `oya-ci-required` remains the single protected merge authority.
- The existing Buck2 gate slot changes mechanism but does not create a second
  required context or a parallel CI.
- The generated face must be reproducible byte-for-byte through the canonical
  materializer.
- Tests must separate pure pointer/schema/receipt unit tests from real-Git
  historical replay and protected-train integration tests.
- Every train step records exact candidate and promoted evidence outside the
  squash-stable receipt core.

### Protected-train acceptance contract

For each protected train candidate, success is all of the following: the exact
candidate head has one green `oya-ci-required` context; active P2 materializes
the fixed historical receipt by running the exact historical implementation;
the receipt validates byte-for-byte; and P3 remains dormant until a later,
protected-parent-validated pointer-only activation. The train's admission SLO
is **100% of candidate steps meeting those four conditions before merge**. This
is a per-candidate correctness objective, not a latency or availability claim.

The gate fails closed, and the train step is unsuccessful, for a missing
historical object, historical-source or dependency digest drift, nonzero or
timed-out historical materializer, receipt tampering or drift, empty or
nondeterministic output, an invalid or candidate-broadened control plane, a P3
parser-source mismatch, or a P3 activation that changes anything beyond the
allowed pointer. No failure permits a fallback to current code under the P2
identity, a hand-edited generated face, a second required context, or a lift of
`HOLD(Planning)`.

The named failure-injection test is
`snapshot_integration::dormant_p3_identity_is_bounded_to_selected_inputs`:
it commits a parser-source mismatch into a real-Git P3 fixture and expects
`dormant_p3_epoch_fingerprint` to reject it with `parser source set is invalid`
before any identity is claimed. Its expected outcome is fail-closed rejection;
the candidate receives no valid P3 receipt and cannot advance the train.

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | no new crate or dependency direction |
| `cross-product-refusal` (LEAN-A2) | Not affected | no product boundary |
| `port-location` | Not affected | no port trait |
| `layer-correctness` | Not affected | existing CI facade only |
| `composition-root-only` | Not affected | no product composition root |
| `sdk-kernel-only` | Not affected | no SDK surface |

No port trait is introduced.

## Alternatives considered

**Alternative 1 — Execute current code under historical P2 metadata**

- Pros: avoids historical source compilation.
- Cons: the executor no longer matches the claimed parser and producer identity.
- Reason rejected: metadata compatibility cannot substitute for exact historical
  execution.

**Alternative 2 — Keep P2 source in a readable archive directory**

- Pros: makes replay code easy to find in a clean checkout.
- Cons: ordinary agents can read stale implementation bytes as current context.
- Reason rejected: Git history is the archive; the active tree retains only
  neutral predecessor digests and the protected replay boundary.

**Alternative 3 — Activate P3 in the parser or producer PR**

- Pros: shortens the merge train.
- Cons: lets a candidate change the mechanism that judges the same candidate.
- Reason rejected: protected-parent interpretation requires later pointer-only
  activation.

**Alternative 4 — Put P3 descriptors in the mutable control plane**

- Pros: makes policy changes data-only.
- Cons: a candidate can repoint parser, selector, executor, or predecessor
  identity and launder its receipt.
- Reason rejected: policy belongs to the protected producer and gate; mutable
  state selects only an already-protected epoch.

**Alternative 5 — Bind the core to the candidate commit or full tree**

- Pros: produces an apparently exact repository identity.
- Cons: squash merge rewrites the commit, and the full tree includes unrelated
  and potentially self-referential changes.
- Reason rejected: the core binds relevant content and declared action inputs;
  admission records commits separately.

## References

- ADR-0515: single protected `oya-ci-required` authority; amended only for the
  ADR-census epoch transition mechanism.
- ADR-0525: Buck2 hermetic execution and SCM-facts boundary.
- ADR-0552, ADR-0595, and ADR-0597: stable/volatile facts and de-committed
  generated-face context.
- ADR-0613: accepted de-commit boundary for controller-materialized faces.
- ADR-0619: accepted Git-history-only retirement boundary.
- ADR-0623: Proposed, nonbinding Stage-1 context only; not authority for this
  decision.
