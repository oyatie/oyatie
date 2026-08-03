# PR #1364 Stage-1 foundation repair plan v2

Status: execution-ready after the donor-only restack is frozen.

Pinned predecessor:

- PR #1363 provisional head:
  `1026a65b707ce57693d9b830de33ee0ce228f16b`
- tree:
  `d20c21163c6ec1ac2f6e1b581031e4431b54f401`

## Outcome

Port the Stage-1 donor onto the pinned predecessor, close the six independently
reviewed contract gaps, and encode the maximum safe evidence concurrency as a
pure partial order.

The result remains:

- ADR-0622 `Proposed`;
- a dormant pure Rust evaluator and closed schemas;
- `HOLD(Planning)`;
- non-authoritative;
- unable to approve planning or dispatch implementation.

This PR does not implement or claim:

- a scheduler, worker, queue, retry engine, WIP controller, lease, or fencing;
- a producer, trust-root verifier, identity authority, revocation service, or
  durable store;
- a materializer service, admission controller, new CI context, planning
  approval, roadmap, or implementation dispatch.

## Category boundaries

- Evidence state: content-addressed candidate records, typed receipt bindings,
  structural joins, and pure validation. This PR may implement it.
- Scheduler state: live ownership, attempts, leases, retries, queues, and
  backpressure. Deferred.
- Authority state: authentic qualification, revocation, jurisdiction, trusted
  time, and independence. Supplied only by a later external attestation path.
- Repository-admission state: protected PR, required context, merge, and
  post-merge facts. Retained only as a candidate envelope interface.
- Product implementation: producer, verifier, controller, scheduler, or
  materializer. Deferred.

Structural equality or inequality never creates qualified-human authority.
Source fixtures never prove external facts.

## Donor-only restack

Port exactly:

1. `9c34f412da7df7922bea9e3dd352fbe0ed55ab6e` — thirteen
   foundation paths.
2. Stage-1-only hunks from
   `bbfcf5cfcabedaa985275615af42a3d39e7e42ba` — nine binding
   paths.
3. `3b7bfc89163ea638ce99e41a3242695a2bbca008` — four harness
   corrections on already counted paths.

The final baseline scope is twenty-two unique Stage-1 paths. Preserve newer
#1363 ADR/index/registry context. Do not import unrelated donor-parent archive,
doc-axis, or GraphQL changes. Do not edit a generated face by hand.

## Pure evidence partial order

Preparation may occur early outside this satisfaction graph, but it creates no
satisfied receipt or authority.

```text
C01 || C02 || C03
  -> J-A
  -> C04

C04 -> C06 -> C05
C04 -> C07
C04 -> C08
C04 -> C09

J-B(C04-C09)
  -> C10 || C11
  -> E(C04-C11 satisfy-plane evidence join)
  -> C12 canonical successor-bundle candidate binding
  -> L01 || ... || L16 || C14
  -> J-D(C13+C14)
  -> C15 oracle || blind reader
  -> C15 qualified-planning-authority receipt binding
  -> external admission-envelope validation only
```

Rules:

- C05 inactive pointer/protocol preparation is not C05 evidence.
- C05 collection, analysis, citation, and satisfaction require the exact C06
  scope authorization binding.
- C10 and C11 remain siblings because no accepted/current contract proves an
  ordering between them. An external effectful pilot must remain blocked until
  veto authority permits it.
- E is a satisfy-plane join candidate, never repository admission.
- C12 validates the canonical members and digest of a supplied successor-bundle
  candidate. It cannot itself make the bytes durable or immutable.
- C13's sixteen lenses and C14 fresh dissent are a seventeen-way antichain
  over the same C12 digest and may not consume one another's output.
- Any mutation of the bound successor requires a new subject/epoch and makes
  prior C13/C14/C15 candidate receipts stale.
- C15 consumes the closed C13/C14 join. Its source fixture cannot establish
  context-free execution; that fact remains externally attested.

## Six required closures

### H1 — C04 closed universe

Bind a closed decision-population candidate:

- authority roots and protected cutoff commit/tree;
- inclusion predicate and exact lifecycle/status vocabulary;
- ordered included and excluded manifests;
- typed exclusion reasons;
- counts, object bindings, universe digest, and reconciliation result.

Reject receipt-only satisfaction, missing/unclassified candidates, duplicates,
included/excluded overlap, unknown states/reasons, and count/digest mismatch.

### H2 — C05/C06 causal scope and freshness

C06 candidate binding:

- exact comparator protocol/scope digest;
- jurisdiction and allowed actions/source classes;
- external authority-attestation binding;
- trusted issued/cutoff/expiry time bindings;
- revocation/conflict binding.

C05 candidate binding:

- same scope and causal C06 receipt digest;
- source publisher/version/digest and observation times;
- collection start/end;
- allowed-use disposition reference;
- supported/contradicted fact, uncertainty, negative evidence, and Oyatie gate.

The pure evaluator checks structural causality against supplied protected time
facts but still returns HOLD until the external verifier/trust root exists.

### H3 — C06-C11 typed authority and role separation

Use closed discriminators for every C06-C11 role. Bind principal, authority
source, qualification class, jurisdiction/scope, validity/revocation/conflict,
subject, and receipt digest.

Enforce supplied principal-ID inequalities:

- candidate authors differ from C06-C11 issuers;
- producer/evaluator/materializer principals differ from qualified issuers;
- C06 authorizer differs from C05 collector;
- C07 affected-party differs from C08 operations;
- C09 custody differs from producer/materializer;
- C10 veto differs from C11 qualified pilot;
- C11 machine and human principals differ.

These inequalities do not prove qualification. Missing external authentication
keeps the candidate held.

### H4 — E join and C12 canonical bundle

E binds exactly the satisfied C04-C11 candidate receipt digests for one subject,
program, epoch, and snapshot. Missing, duplicate, stale, blocked, cross-subject,
or cross-snapshot inputs reject the join.

C12 binds one canonically ordered successor-bundle candidate containing:

- program/source epoch;
- protected-facts candidate;
- E join;
- C01-C11 receipts;
- parser, policy, producer, evaluator, schema, and trust-root references;
- predecessor/transition reference.

Cross-check source, protected-facts, and admission references to the same object
binding. Actual durable freeze remains externally attested and unimplemented.

### H5 — C13/C14/C15 independence and reviewed input

Protected candidate facts bind principal sets for authors, subject owners,
producer, evaluator, materializer, and qualified issuers.

- C13 requires sixteen unique lens principals outside protected principal sets.
- C14 differs from every C13 and protected principal.
- C13 and C14 bind only the same C12 candidate digest.
- C15 oracle and blind reader differ from each other and all prior principals.
- C15 binds an immutable allowed-input-manifest candidate and the same C12/J-D
  digests.
- `conversation_context_used=false` remains an externally attested input, not a
  source-proven fact.

### H6 — typed external admission-envelope interface

Replace generic bindings with closed candidate records for:

- current PR head/tree and independent exact-head review;
- protected parent commit/tree;
- required `oya-ci-required` context, source App, run, and conclusion;
- promoted commit/tree relationship;
- resolved review threads and branch protection;
- rollout, rollback, observability, browser/user-story, release, and
  observation-harvest outcomes.

The pure validator always reports the intentional external-controller/trust
HOLD. An envelope-shaped source candidate cannot produce `PASS_CANDIDATE`,
`PASS(Planning)`, or dispatch.

## Regression-first commit sequence

### A — donor baseline

- Freeze exact 22-path restack.
- Run formatting, JSON, scope, signature, HOLD, and generated-face checks.
- Attempt the two declared Buck targets; record sandbox inability without a
  PASS claim.

### B — RED contract lock

One test writer adds exact failing mutations for:

- pure partial-order topology and C13/C14 parallel fork after C12;
- H1 closed universe;
- H2 pre-C06, stale, revoked, expired, wrong-scope, and incomplete C05;
- H3 wrong discriminators and every forbidden principal overlap;
- H4 invalid E joins, mismatched C12 objects, missing/order/digest errors, and
  pre-C12 reviews;
- H5 author/reviewer collisions, differing reviewed inputs, and unbound
  context attestations;
- H6 generic/counterfeit/stale-head admission records.

No generic `is_red()` assertion is sufficient for these closures; assert exact
findings or a deliberately narrow finding family.

### C — GREEN pure contract

One Rust evaluator writer owns `src/lib.rs`. Schema writers may work in parallel
only after the discriminator/property table is frozen; the evaluator writer
integrates them.

Implement only:

- closed schema shapes;
- digest/object equality and canonical ordering;
- the partial-order and joins;
- supplied timestamp/scope causality;
- supplied principal-ID inequality;
- unconditional external-authority/controller HOLD.

Do not add attempts, WIP, leases, fencing, queues, retries, persistence, or a
ready-task scheduler.

### D — traceability and final review

One governance writer reconciles ADR-0622, masterplan, root pointers, indexes,
registries, Buck exports, and Cargo.lock. ADR remains Proposed and the purpose
text remains dormant/non-authoritative.

Then run independent reviews for:

1. donor-parent leakage and #1363 non-regression;
2. schema/evaluator parity;
3. C04 and C05/C06 causality;
4. C06-C11 authority/role boundaries;
5. E/C12/C13/C14/C15 ordering and independence;
6. admission honesty and HOLD/no-dispatch;
7. Rust correctness and simplification.

## Ownership and concurrency

One writer at a time for:

- `libs/oya-stage1-closure/src/lib.rs`;
- the main contract test file and shared fixtures;
- masterplan/registries/indexes;
- final integration.

Safe parallel work after the field table freezes:

- one writer per independent JSON schema;
- read-only adversarial reviews by H1-H6 concern;
- read-only build/static verification;
- docs consistency review separate from the docs writer.

## Verification and stop rule

Required local checks:

- exact head/tree/base and clean status;
- all commits authorized and signed;
- direct rustfmt on changed Rust;
- `git diff --check`;
- parse every changed JSON;
- exact 22-path baseline plus reviewed repair paths;
- no changed `*.generated.json`;
- HOLD/no planning/no dispatch flags;
- targeted Buck contract/unit targets when the environment permits.

Protected admission additionally requires exact-head independent approval,
resolved threads, conflict-free branch, protected `oya-ci-required`, branch
protection, squash merge, and post-merge completion evidence.

Stop at a reviewed, recovery-tested local candidate if transport or protected
CI is unavailable. Never infer a planning PASS from source-only fixtures or
from unavailable external authority.
