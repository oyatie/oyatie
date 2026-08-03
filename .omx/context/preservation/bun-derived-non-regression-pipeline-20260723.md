# Bun-derived non-regression pipeline packet

Status: `NON_AUTHORITY`

Planning state: `HOLD(Planning)`

Comparator status: inactive process-preparation only. This packet is not C05
comparator evidence, a legal/JCR disposition, a roadmap, implementation
authority, or a dispatch instruction.

Source reviewed:

- Bun, "Bun's Rust rewrite", 2026-07-08:
  https://bun.com/blog/bun-in-rust

## Transfer rule

Bun is a process comparator, not a behavioral target. Kubernetes, Talos, and
other upstream systems are compatibility and experience comparators, not
destination ceilings.

Every behavior must be judged against three independent oracles:

1. current Oyatie behavior;
2. pinned upstream behavior;
3. intended Oyatie contract.

Every behavior must receive exactly one classification before implementation
fanout:

- `OYATIE_PROTECTED_ADVANTAGE`: candidate must match or improve Oyatie.
- `REQUIRED_UPSTREAM_COMPATIBILITY`: candidate must match declared upstream
  behavior within the pinned version and normalization scope.
- `DELIBERATE_SAFE_DIVERGENCE`: candidate needs rationale, consumer-impact
  evidence, migration/rollback, and accepted authority.
- `UPSTREAM_DEFECT_OR_NON_GOAL`: candidate must not import the behavior and
  needs an explicit refusal or corrected-behavior regression.

An unclassified behavior blocks its shard. Matching upstream is never sufficient
when Oyatie is already better.

## Throughput topology

Use the following funnel:

`serial authority snapshot -> parallel preparation -> serial contract lock ->
bounded ownership shards -> immutable validation/failure queues -> serial
protected-PR admission -> post-merge proof`

Parallelize:

- read-only inventories and behavior classification proposals;
- non-overlapping implementation shards after contract lock;
- independent diff-only adversarial reviews;
- immutable failure-queue items;
- platform/test shards within explicit resource budgets.

Serialize:

- authority and behavior-classification decisions;
- shared API/wire/storage and security contracts;
- generated faces and their materializers;
- repository structural changes;
- global diagnostics snapshots;
- final integration, protected PR admission, and promotion.

Each code-producing shard uses separate roles:

1. writer/implementer;
2. independent reviewer one;
3. independent reviewer two where risk warrants;
4. applier/fixer.

The writer does not approve its own result. Reviewers see the exact diff and
candidate identity and assume the change is wrong until evidence proves
otherwise.

## Worktree isolation and consolidation

Bun's useful worktree lesson is contention control, not a fixed worktree or
agent count. Shared checkout Git operations, repeated global builds, and
unbounded worker fanout created collisions and resource saturation. Oyatie
therefore uses the following non-authoritative execution protocol:

1. Freeze the protected predecessor commit/tree and the candidate's canonical
   write-area set.
2. Give each code-producing lane one isolated mirror/worktree/branch. Two live
   writers may not own the same canonical write area or affected-target closure.
3. Capture expensive global diagnostics once into an immutable, version-bound
   snapshot. Partition the resulting failures by root cause and ownership
   boundary; workers do not independently rerun global diagnostics in a loop.
4. Prohibit shared-checkout recovery operations inside worker lanes. In
   particular, a worker may not use stash/pop/reset to coordinate with another
   lane or mutate another worktree's refs.
5. Review the exact candidate commit/tree from a separate context. Review
   evidence is stale as soon as that candidate or its protected predecessor
   changes.
6. Use one distinct applier for each integration generation. The applier
   consolidates only reviewed commits or patches, in dependency order, into a
   fresh integration worktree and reruns the smallest validation that proves
   the combined claim.
7. Serialize protected-branch admission, merge, and post-merge proof. A merged
   predecessor forces every dependent candidate to restack, revalidate, and
   receive a fresh exact-head review before admission.

Preparation worktrees may coexist even when their future PRs overlap, but
overlapping candidates are not independently mergeable. The current
#1361-#1364 train is a concrete example: preparation and read-only review
overlap, while predecessor restack, exact-head approval, required-check
admission, merge, and post-merge verification remain ordered.

Worktrees are disposable execution isolation, not archival custody. Retired
authority bytes must not be moved into a readable worktree directory. Durable
preservation uses signed Git object history, recovery-tested bundles, and
protected PR/admission records; current-tree archive directories do not become
authority by location.

### Restricted-history custody rule

Engineering recovery and restricted historical custody are different
requirements:

- Signed Git objects, recovery bundles, protected branches, PRs, CI artifacts,
  ignored paths, hidden paths, and repo-local `archive/` directories are valid
  only for restart-safe engineering recovery. They remain readable to an
  ordinary agent with repository access.
- Removing retired authority from the current tree while retaining ordinary Git
  history prevents accidental current-authority discovery, but it does not make
  the historical bytes inaccessible and must not be described as such.
- A requirement that future ordinary agents cannot retrieve retained historical
  content needs a separate qualified-custodian boundary outside the repository,
  every repository ref, normal forge artifact storage, and every credential
  available to agents. The custodian must own encryption keys, access policy,
  retention, retrieval logging, and revocation.
- The repository may retain only non-content metadata needed to prove
  disposition: an opaque record identifier, cryptographic digest, lifecycle
  classification, custodial-system identifier, and an explicit non-claim about
  past readability. It must not retain excerpts, retrieval URLs, keys, or access
  instructions.
- If already-published Git history must become inaccessible, a qualified
  founder/repository-administrator/custodian disposition must authorize and
  independently verify the destructive rewrite or purge across reachable refs,
  forks, caches, and artifacts. No process can truthfully make retroactive
  confidentiality claims about prior reads or copies.

A low-privilege agent access-denial receipt and a separate logged
custodian-only retrieval receipt are both required before a
`future_agent_inaccessible` claim. Neither an agent self-check nor a readable
Git bundle can supply those receipts.

Measure the consolidation system with:

- worktree collision and repark rate;
- stale-review rate after candidate/base movement;
- patch-application conflict rate;
- reused versus rerun validation work;
- writer-to-review and review-to-apply queue age;
- integration dwell time;
- escaped-regression and rework rate;
- verified candidates per protected admission slot.

Increase worktree fanout only while marginal verified throughput improves and
review, applier, validation, disk, and admission queues remain within their
measured limits.

## Validation ladder

Validation grows in cost:

1. syntax, formatting, schema, and static policy;
2. compile/check;
3. smallest behavior smoke;
4. focused regression and differential corpus;
5. subsystem shards;
6. cross-component integration, failure, upgrade, and rollback;
7. full protected multi-platform CI;
8. post-merge runtime, observability, rollback, and user-story proof.

Compilation is not behavioral proof. Stubs, skipped/deleted tests, silent
fallbacks, paragraph-length workarounds, and unclassified divergences fail
closed.

## Backpressure

Active shard count must not exceed the smallest available capacity among:

- independent reviewers;
- appliers/fixers;
- local validation slots;
- CPU, memory, IOPS, disk, sockets, and process limits;
- protected CI and admission capacity.

Pause new fanout when any of these grows beyond its frozen threshold:

- review age;
- validation queue age/depth;
- stale diagnostic rate;
- resource saturation;
- worktree collision or repark rate;
- admission queue age;
- rework or escaped-regression rate.

Optimize verified candidates per unit time, not agent count, lines changed, or
commit count.

## Kubernetes then Talos

During `HOLD(Planning)`, only inventories, behavior-ledger/corpus design,
immutable queue rehearsals, non-integrating pilots, and process validation are
allowed.

Kubernetes is the process proving ground. Talos may inventory and classify in
parallel, but Talos implementation may not inherit the process until Kubernetes
proves it through bounded pilots.

Talos may later inherit only:

- behavior-ledger schema;
- evidence and differential-fixture formats;
- writer/reviewer/applier protocol;
- immutable failure-queue and diagnostic-freshness rules;
- WIP/backpressure policy;
- non-regression, safe-divergence, and upstream-defect-refusal gates.

Talos may not inherit Kubernetes component boundaries, API semantics outside
the hosting seam, shard sizes, storage/controller assumptions, Go translations,
test receipts, or unvalidated workarounds.

## Stage-1 concurrency revision

Stage-1 must distinguish three planes:

- `prepare`: safe, non-authoritative work may fan out against a named snapshot;
- `satisfy`: evidence may advance only across typed semantic and authority edges;
- `admit`: protected facts, exact-head review, materialization, CI, and promotion
  remain single-filed against the current protected parent.

Preparation may produce only `prepared_unbound`, never `satisfied`, planning
authority, or dispatch authority.

The minimal satisfaction DAG is:

```text
C01 || C02 || C03
  -> JA(C01-C03)
  -> C04

C04 -> C06 -> C05
C04 -> C07
C04 -> C08
C04 -> C09

JB(C04-C09)
  -> C10 || C11
  -> E(C04-C11 satisfy-plane join)
  -> C12 canonical successor-bundle candidate
  -> L01 || ... || L16 || C14 fresh dissent
  -> JD(C13+C14)
  -> C15 oracle || blind reader
  -> C15 qualified planning authority
  -> external admission-envelope validation
```

C05 protocol and inactive pointer preparation may run early, but C05
collection, expansion, analysis, citation, or satisfaction may not begin before
the exact C06 legal/JCR authorization is current, scope-bound, and authenticated.

C13 and C14 are parallel after C12 because fresh dissent must not consume prior
council context. If either path changes the frozen successor, the subject
generation changes and all C13/C14 receipts become stale.

C10 and C11 are siblings in the current proposed contract. Any externally
effectful pilot operation must wait for the veto disposition; if C11
satisfaction itself must wait, that edge must be added explicitly and tested.

The dormant source contract may encode only pure evidence-causality kinds:

- `semantic`;
- `authority`;
- `subject_freeze`;
- `evidence_join`;
- `external_admission`.

Every candidate evidence record must bind its exact program, epoch, subject,
snapshot, object identity, and predecessor where applicable. The source
validator may reject cross-object, stale, missing, duplicate, or non-canonical
candidate records, but it cannot create trusted time, authority, durable
immutability, or protected admission.

Live task states, attempts, queues, retries, WIP, leases, fencing, persistence,
writer ownership, materialization, and admission compare-and-swap belong to an
external scheduler/repository-control plane. They are not implemented or
claimed by the dormant Stage-1 source contract.

Program, parser, producer, evaluator, policy, schema, evidence, authority, or
subject changes require a new externally attested epoch or successor generation.
Protected-base movement invalidates protected facts, exact-head review, and the
external admission attempt.

Maximum useful semantic width is 17 after C12: sixteen independent lens lanes
plus one fresh-dissent lane. Mutable integration still has one applier and one
protected-admission slot.

For a unit-weight structural comparison, the graph contains 33 atomic
evidence/admission items. Treating the four joins as zero-duration barriers,
the critical path is 10 units and the idealized upper-bound speedup over full
serialization is 3.30x. Treating joins as unit work yields 37 nodes, a 14-unit
span, and a 2.64x idealized bound. These are topology measurements only; actual
elapsed time must be measured by control, authority, review, and admission
latency before setting WIP.

## Mandatory outcome gates

- protected Oyatie behavior regressions: `0`;
- unresolved required-compatibility deltas: `0`;
- undocumented/unapproved divergences: `0`;
- accidentally imported upstream defects: `0`;
- skipped or deleted tests used to obtain green: `0`;
- hand-edited generated faces: `0`.

`PASS_CANDIDATE` remains non-authoritative. No roadmap planning, binding
approval, monorepo migration, Kubernetes/Talos rewrite, or implementation wave
may dispatch until every controlling Stage-1 gate independently authorizes
planning exit.
