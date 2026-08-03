# PR #1364 Stage-1 Foundation Repair Plan

> **SUPERSEDED — DO NOT EXECUTE.**
>
> This draft incorrectly mixed evidence state with live scheduler, lease,
> queue, persistence, and admission-controller state. The replacement is
> `.omx/plans/pr1364-stage1-foundation-repair-v2.md`.

## Outcome and stop condition

Restack the provisional PR #1364 Stage-1 foundation on the exact promoted/head commit of PR #1363, semantically port the donor chain `9c34f412` + `bbfcf5cfc` + `3b7bfc891`, and close the six HIGH review gaps with a dormant, pure, fail-closed contract/evaluator.

The repaired PR stops at:

- a proposed ADR;
- closed JSON schemas and canonical HOLD fixtures;
- a pure Rust candidate evaluator;
- hermetic Buck tests and Cargo library checks;
- registry/root-pointer traceability;
- `HOLD(Planning)`, `roadmap_planning_authorized=false`,
  `binding_plan_approval_allowed=false`, and
  `implementation_dispatch_allowed=false`.

It does **not** implement or activate an authenticated facts producer, trust-root verifier,
materializer service, lease store, scheduler/worker, admission controller, cloud-ci gate, planning
approval, roadmap, or implementation dispatch.

## Grounded starting point

The donor chain contributes 22 unique paths:

- Stage-1 crate, tests, and fixtures:
  `libs/oya-stage1-closure/{BUCK,Cargo.toml,OWNERS,src/lib.rs,tests/**}`;
- four schemas:
  `specs/stage1-{closure-program,evidence-epoch,protected-facts,admission-envelope}.schema.json`;
- proposed ADR-0622;
- `Cargo.lock`, `specs/{BUCK,masterplan.json,capability-registry.json,root-hub-pointers.json}`,
  `registry/{BUCK,artifact-capabilities-registry.json}`, `docs/ADR-INDEX.md`, and
  `docs/machine-readable/decisions.json`.

Important donor hazard: `bbfcf5cfc` is parented by `31002978`, not `9c34f412`; the two parent trees
are not equal. A literal cherry-pick or final-tree checkout can import unrelated doc-axis,
GraphQL-policy, archive, and registry changes. The restack must port only the Stage-1 path/hunk set.

The donor already enforces exact C01-C15 and L01-L16 populations, HOLD-only source effects,
protected-parent bindings, receipt cardinalities, basic C05-after-C06 ordering, and a non-authoritative
post-merge envelope. The repair strengthens those contracts; it does not replace the foundation.

## Canonical repaired model

### Three planes

The program schema and canonical program instance define three explicit planes:

1. `prepare`: create immutable, content-addressed snapshots and typed work items; preparation never
   satisfies a control.
2. `satisfy`: validate evidence and qualified authority against one snapshot; successful work emits a
   typed satisfaction receipt.
3. `admit`: consume only protected, joined outputs and derive at most a non-authoritative candidate.
   Effective planning admission remains external and unimplemented.

Each node has an exact ID, plane, control/lens owner, allowed states, input artifact types, output
artifact type, invalidation keys, and WIP class. Each edge has an exact type, source, destination,
required source state, and subject/snapshot propagation rule.

Canonical node states:

- `NOT_READY`
- `READY`
- `WIP`
- `SATISFIED`
- `BLOCKED_QUALIFIED_HUMAN_INPUT`
- `FAILED`
- `INVALIDATED`
- `ADMITTED_CANDIDATE` (admit-plane output only)

Canonical edge types:

- `PREPARES`
- `AUTHORIZES_SCOPE`
- `REQUIRES_SATISFIED`
- `JOINS`
- `FREEZES`
- `FORKS_INDEPENDENT_REVIEW`
- `ADMITS`
- `INVALIDATES`

### Required DAG

The canonical graph is closed and ordered:

1. C01, C02, and C03 prepare/satisfy from the open epoch.
2. C04 consumes satisfied C01-C03 and closes the exact decision universe.
3. C06 may prepare with C01-C04, but its qualified-human receipt must authorize an exact scope
   digest and freshness window before C05 collection/satisfaction.
4. C05 consumes that exact C06 authorization and current source snapshot.
5. C07-C09 run as independent qualified-authority shards after the common foundation.
6. C10 and both halves of C11 consume satisfied C04-C09.
7. typed join `E` consumes exactly C04-C11 satisfaction receipts, one subject digest, and one
   snapshot generation. `E` owns no control and emits only `qualified_authority_join/v1`.
8. C12 alone consumes admitted `E`, materializes the canonical successor bundle, verifies canonical
   ordering, and freezes its digest.
9. C13/L01-L16 and C14 fork in parallel **only after C12 is frozen**. They consume the same C12
   bundle digest and may not consume each other's outputs.
10. C15 joins all sixteen C13 lens receipts plus C14, then performs the context-free exit.
11. The admit plane may structurally validate a candidate only after C15. The source candidate can
    never produce effective `PASS(Planning)` or dispatch.

### WIP, invalidation, failure, and lease semantics

- At most one active attempt exists per `(epoch_id, node_id, snapshot_digest)`.
- C13 permits one WIP shard per L01-L16 and C14 permits one concurrent shard after C12; the admit
  plane is serial.
- Every WIP record carries a lease binding with `lease_id`, `holder_principal_id`, `node_id`,
  `epoch_id`, `snapshot_digest`, `fencing_token`, `acquired_at_unix`, and `expires_at_unix`.
- The pure evaluator validates lease shape, uniqueness, fencing monotonicity within the submitted
  snapshot, and expiry relative to protected evaluation time. It never grants, renews, or persists a
  lease.
- A change to subject, program, parser, policy, producer, evaluator, schema, authority scope, source
  cutoff, or upstream output digest invalidates the node and all descendants. An invalidated or
  expired WIP attempt cannot satisfy or admit.
- Failures enter a typed queue record with node/attempt/snapshot identity and one exact class:
  `TRANSIENT_RETRYABLE`, `PERMANENT_EVIDENCE_FAILURE`, `QUALIFIED_HUMAN_BLOCKED`, or
  `SNAPSHOT_INVALIDATED`. Only the qualified-human class may support the existing terminal blocked
  state, and only for C06-C11.

## Six HIGH-gap closures

### H1 — C04 closed decision universe

Add a discriminated C04 evidence payload to protected facts. It binds:

- protected repository/base commit/tree and parser/policy versions;
- the canonical inclusion predicate and decision-status vocabulary;
- an ordered member manifest binding and member count;
- an ordered exclusion manifest binding with typed exclusion reasons;
- a universe digest derived from both manifests and the predicate;
- a completeness result and evaluator identity.

Reject partial populations, unknown statuses/reasons, duplicate members, overlap between included and
excluded members, count/digest mismatches, or C04 receipts without the exact closed-universe payload.
No live decision population may be inferred from a fixture or prose list.

### H2 — C05/C06 causal scope and freshness

C06 emits a qualified `scope_authorization/v1` with `scope_digest`, jurisdiction, allowed source
classes, `issued_at_unix`, `source_cutoff_unix`, and `expires_at_unix`. C05 emits
`comparator_snapshot/v1` with the same scope digest, the causal C06 receipt digest, collection start
and completion times, source versions/digests, negative evidence, uncertainty, and an Oyatie exit
gate.

The evaluator rejects C05 when:

- its collection started before C06 issuance;
- its scope or causal receipt digest differs;
- a source observation exceeds the C06 cutoff;
- evaluation occurs at/after expiry;
- a source version/digest or negative-evidence/uncertainty field is absent;
- a prepared inventory is mislabeled as satisfied evidence.

### H3 — C06-C11 typed authority and separation of duties

Replace free-form qualified-authority matching with discriminated payloads for:

- C06 `qualified-legal-jcr`;
- C07 `qualified-affected-party`;
- C08 `qualified-operations`;
- C09 `qualified-custody`;
- C10 `authorized-veto`;
- C11 `machine-pilot-evidence` and `qualified-pilot-authorization`.

Each payload binds principal identity, qualification class, authority source object, jurisdiction and
scope digests, validity/revocation, subject/snapshot, and the protected receipt digest.

Encode the minimum incompatible-role matrix:

- candidate author cannot be a C06-C11 qualified issuer;
- evidence producer/materializer/evaluator cannot be a C06-C11 qualified issuer;
- C06 scope authorizer cannot be the C05 collector;
- C07 affected-party representative cannot be the C08 operations owner;
- C09 custodian cannot be the materializer/producer;
- C10 veto owner cannot be the C11 qualified pilot;
- C11 machine principal and qualified-human principal must differ.

Reject boolean/self-asserted independence when the protected principal bindings do not prove the
required inequality.

### H4 — C12 canonical bundle and freeze order

Define one `stage1-successor-bundle/v1` manifest containing exact, canonically ordered bindings for:

- program and source epoch;
- protected facts and snapshot generation;
- C01-C11 receipts;
- typed E join;
- parser, policy, producer, evaluator, schema, and trust-root bindings;
- predecessor epoch and transition receipt.

C12 satisfaction is the successful canonicalization and freeze of this manifest. Reject a C12 receipt
before E admission, any missing/duplicate/out-of-order member, digest mismatch, mutable/path-only
reference, or any C13/C14/C15 receipt bound to a pre-freeze subject.

### H5 — author, subject, and context-free independence

Protected facts add exact principal sets for candidate authors, subject owners/sponsors, producer,
materializer, evaluator, and authority issuers. C13, C14, and C15 receipts bind their inputs and
principal IDs to those protected sets.

Enforce:

- sixteen distinct C13 reviewers, each different from candidate authors, subject owners, producer,
  materializer, evaluator, and C06-C11 issuers;
- C14 reviewer different from all C13 reviewers and all protected author/subject/production roles;
- C13 and C14 consume only the frozen C12 bundle, not one another's output or conversation context;
- C15 oracle and blind reader differ from each other and every prior author/reviewer/producer role;
- C15 consumes only the C12 bundle plus the closed C13/C14 join and has
  `conversation_context_used=false`.

The existing `independent_from_author` and `fresh_context` booleans may remain as explanatory source
fields, but they are insufficient for satisfaction without protected principal-set proof.

### H6 — typed E join and admission

Give E a first-class, closed join output instead of an empty A-G group. It must contain the exact
C04-C11 input receipt digests, subject/snapshot digest, join policy digest, protected evaluator
binding, and `READY_TO_FREEZE` outcome. No missing, duplicate, stale, blocked, failed, or invalidated
input can join.

The admission envelope binds the C15 candidate, C12 bundle, E join, promoted commit/tree, protected
base, required context/source App, independent review, branch protection, and post-merge completion
packet. The pure validator must continue returning the intentional non-authoritative HOLD finding;
no source-authored envelope can self-admit.

## Regression-first implementation sequence

### Commit 0 — clean restack and semantic donor port

Owner: integration one-writer.

1. Create an isolated PR #1364 worktree/branch from the exact PR #1363 head after #1363 is ready for
   stacking.
2. Port the 13-path `9c34f412` foundation.
3. Port only the nine Stage-1 binding paths/hunks described above from `bbfcf5cfc`; do not import its
   divergent parent-tree changes.
4. Port the four-path harness correction from `3b7bfc891`.
5. Resolve against #1363 by preserving #1363's trusted materialization, generated-face, workflow,
   action-item-accounting, and retirement decisions.
6. Verify the resulting delta is Stage-1-only and contains no hand-edited `*.generated.json`.

Acceptance:

- the branch ancestry is exactly PR #1363 head plus the semantic Stage-1 port;
- all 22 intended files are present/bound as needed;
- no unrelated donor-parent file is introduced or reverted;
- donor baseline tests pass before repair REDs are added.

### Commit 1 — RED: freeze the repaired program/DAG contract

Owner: test one-writer for `tests/stage1_closure.rs` and canonical fixtures.

Add exact failing regressions for:

- the three planes, canonical node states, edge types, and graph topology;
- C13/L01-L16 plus C14 parallel fork only after C12;
- typed E input/output cardinality;
- C12-before-review order;
- snapshot invalidation propagation;
- one-attempt WIP uniqueness, serial admit, C13/C14 parallel allowance;
- lease identity/fencing/expiry;
- typed failure queue and qualified-human-only terminal blocking.

RED checkpoint:

- run the hermetic contract target;
- retain the exact expected missing-field/semantic findings in the review packet;
- confirm failures are caused only by the new contract assertions.

### Commit 2 — GREEN: program schema, canonical instance, and pure DAG evaluator

Owners:

- one schema writer for `stage1-closure-program.schema.json`;
- one canonical-instance writer for the `masterplan.json` Stage-1 object and `program.json`;
- one evaluator writer for `src/lib.rs`.

Implement the closed program/DAG shape and semantic graph checks. Keep the source candidate effects
false and do not add a dispatcher/executor.

Checkpoint:

- program schema/fixture/evaluator parity passes;
- graph mutations (missing/extra/retyped edge, wrong state, cycle, early C13/C14, direct admission)
  fail deterministically;
- existing exact C01-C15 and L01-L16 population tests remain green.

### Commit 3 — RED: freeze typed evidence and authority semantics

Owner: test one-writer.

Add table-driven negative regressions for all six HIGH gaps:

- C04 partial/open universe and inclusion/exclusion overlap;
- stale or causally impossible C05/C06 evidence;
- every wrong authority discriminator and every forbidden role overlap;
- missing/duplicate/out-of-order C12 bundle members and early freeze;
- author/subject/producer/reviewer/oracle identity collisions;
- E joins with stale, failed, blocked, invalidated, cross-subject, or cross-snapshot inputs;
- source/self-authored admission and PR-head-as-promoted-commit attempts.

Each mutation must assert the exact finding set or an intentionally scoped finding family; avoid
generic `is_red()` coverage for these HIGH gaps.

### Commit 4 — GREEN: protected facts, epoch, admission schemas and evaluator

Owners after the discriminator/property names are frozen:

- protected-facts schema writer: typed C04-C12 payloads, principal sets, E join, canonical bundle;
- epoch schema writer: node attempts, WIP, invalidation, failure queue, materializer lease references;
- admission schema writer: typed E/C12/C15 and promoted-commit bindings;
- evaluator one-writer: all cross-artifact semantics in `src/lib.rs`;
- fixture one-writer: `hold-epoch.json` and `admission-envelope.json`.

The evaluator must validate:

- closed discriminated unions and exact cardinalities;
- causal timestamps using protected integer Unix times;
- exact subject/snapshot/digest joins;
- incompatible-role matrix;
- C12 canonical order and freeze boundary;
- protected principal-set independence;
- lease/failure/invalidation semantics;
- non-authoritative admission HOLD.

Checkpoint:

- all new RED tests turn green;
- every legacy donor regression remains green;
- schemas and Rust accept/reject the same canonical and mutated shapes;
- no new dependency is added.

### Commit 5 — docs and traceability reconciliation

Owner: governance one-writer.

Update ADR-0622 to describe the three-plane DAG, typed evidence payloads, E join, C12 freeze order,
independence proof, failure/WIP/lease contract, and dormant boundary. Reconcile:

- `specs/masterplan.json`;
- `docs/ADR-INDEX.md`;
- `docs/machine-readable/decisions.json`;
- `specs/root-hub-pointers.json`;
- `specs/capability-registry.json`;
- `registry/artifact-capabilities-registry.json`;
- `specs/BUCK`, `registry/BUCK`, and `Cargo.lock` only where the Stage-1 bindings require it.

Keep ADR-0622 `Proposed`. Registry purpose text must say pure/dormant/non-authoritative and must not
claim CI, producer, materializer, admission, planning, or dispatch activation.

Checkpoint:

- code, schema, fixture, masterplan, ADR, index, registry, and root-pointer populations agree;
- no Accepted authority is amended by implication;
- generated-face policy sees no hand-authored generated artifact.

### Commit 6 — independent repair review and final evidence

No code changes unless a verified finding requires a narrow fix.

Review in this order:

1. semantic donor/restack review: no parent-tree leakage or #1363 regression;
2. schema/evaluator parity review: all closed unions, cardinalities, and exact fields match;
3. causal/authority review: timestamps, scope digests, typed roles, SoD, and qualified-human blocker
   boundaries;
4. DAG/concurrency review: E join, C12 freeze, C13/C14 fork, invalidation, WIP, failure queue, lease;
5. honesty review: HOLD and all three authorization flags remain false; dormant components are named;
6. final diff and generated-artifact review.

Stop at a green provisional PR head with HOLD/no-dispatch evidence. Do not resolve the planning hold,
accept ADR-0622, activate a controller, or dispatch work in this PR.

## File ownership and parallelism

### Must be one-writer

- `libs/oya-stage1-closure/src/lib.rs`: canonical constants and all cross-artifact semantics.
- `libs/oya-stage1-closure/tests/stage1_closure.rs`: shared helpers and exact finding expectations.
- `specs/masterplan.json` and `tests/fixtures/program.json`: one canonical DAG instance.
- `tests/fixtures/hold-epoch.json` and `tests/fixtures/admission-envelope.json`: shared fixture identities.
- ADR/index/decision-registry/root-pointer/capability-registry reconciliation.
- `Cargo.lock`, `specs/BUCK`, `registry/BUCK`, and artifact-capabilities registry.
- final restack/integration commit and conflict resolution.

### Safe parallel shards after the canonical field/discriminator table is frozen

- the four JSON schemas, one writer per schema;
- independent review lanes for C04, C05/C06, C06-C11 SoD, C12, independence, and E/admission;
- build verification (Buck target, Cargo fmt/check/clippy) in read-only worktrees;
- docs consistency review separate from the docs writer.

Do not parallel-edit `src/lib.rs`, the main contract test file, masterplan, or shared fixtures. If
parallel test authors are used, they should return test cases/expected findings to the test
one-writer instead of editing the shared file.

## Verification ladder

Run from the clean PR #1364 worktree:

1. Targeted format:
   `cargo fmt --all -- --check`
2. Cargo library compile/test (contract integration tests are intentionally Buck-owned):
   `cargo test -p stage1-closure --lib`
3. Hermetic contract:
   `buck2 test //libs/oya-stage1-closure:stage1-closure-contract`
4. Crate unit target:
   `buck2 test //libs/oya-stage1-closure:stage1-closure-unittest`
5. Package target:
   `buck2 test //libs/oya-stage1-closure/...`
6. Lint:
   `cargo clippy -p stage1-closure --lib -- -D warnings`
7. JSON parse:
   `jq empty` on all four schemas, all three fixtures, masterplan, capability registry, artifact
   registry, and machine-readable decisions.
8. Diff hygiene:
   `git diff --check <pr1363-head>...HEAD`
9. Scope audit:
   `git diff --name-status <pr1363-head>...HEAD` and compare against the 22-path intended set.
10. Generated-face audit:
    confirm no `*.generated.json` is added/modified and run the repository's current generated-artifact
    policy target discovered on the #1363 base.
11. Governance integration:
    run the current #1363-base ADR/index/capability/root-pointer validators that own the touched
    registries; do not use retired CLI output as merge authority.
12. Protected PR evidence:
    independent approval, all threads resolved, no conflicts, and the single `oya-ci-required`
    context green.

If a broad repository check fails outside the 22-path delta, classify it against the #1363 base and
do not absorb unrelated repair into #1364.

## Acceptance criteria

- The branch is correctly stacked on #1363 with no semantic donor-parent leakage.
- All six HIGH gaps have exact RED mutations and GREEN evaluator/schema parity.
- The canonical three-plane DAG has typed nodes, edges, states, E join, C12 freeze, C13/C14 parallel
  fork, C15 join, invalidation, WIP, failure queue, and lease bindings.
- C04 proves a closed universe.
- C05 is causally and freshly authorized by exact C06 scope.
- C06-C11 use typed authority and enforce the explicit incompatible-role matrix.
- C12 freezes one canonical bundle only after typed E admission.
- C13/C14/C15 independence is proven from protected principal sets, not booleans alone.
- Admission remains external, post-merge, structurally validated, and non-authoritative in this PR.
- ADR-0622 remains Proposed and all authorization/dispatch flags remain false.
- Targeted Buck/Cargo/JSON/diff/governance verification is fresh and green, or any external
  environment gap is stated without a PASS claim.

## Risks and controls

| Risk | Control |
| --- | --- |
| Literal donor cherry-pick imports unrelated divergent-parent work | Semantic path/hunk port and 22-path scope audit |
| Schema and Rust accept different shapes | Table-driven parity tests for every discriminator/cardinality |
| Time freshness trusts the candidate clock | Compare only protected integer times bound by external facts; source epoch cannot supply authority |
| Boolean independence self-attests | Protected principal sets plus explicit inequality checks |
| Generic receipts conceal control semantics | Closed discriminated payloads for C04-C12 and typed E/C12/C15 bindings |
| Concurrency model implies a running scheduler | Contract-only WIP/lease/failure shapes; no executor/service/catalog claim |
| C13/C14 parallelism races mutable evidence | Both consume only frozen C12 digest; mutations invalidate descendants |
| PR accidentally claims planning exit | Proposed ADR, intentional HOLD finding, three false flags, honesty review |
| Broad base failures expand scope | Compare with #1363 and record unrelated failures; no opportunistic repairs |

## Handoff guidance

Execution should use a clean isolated worktree and preserve the RED-before-GREEN evidence at each
checkpoint. The executor should treat the field/discriminator table and DAG in this plan as the
shared contract freeze. Any need for a live producer, persistent queue, lease backend, external
identity source, admission controller, Accepted ADR transition, or roadmap dispatch is a new scoped
follow-up, not permission to enlarge PR #1364.
