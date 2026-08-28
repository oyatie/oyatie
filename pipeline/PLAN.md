---
doc_class: Owner-PLAN
owner: pipeline
status: Active
date: 2026-08-28
---

# Pipeline remaining work

<baseline>

## What has landed

- `pipeline/core/admission` contains pure path-delta, layout, owner-occupant,
  occupancy, and fan-in behavior.
- `pipeline/ports/draft/repository` and
  `pipeline/adapters/draft/repository-git` provide the current read-only Git
  seam for merge base, raw NUL-delimited name-status, immutable blobs, tree
  entries, and kinds. Admission reduces name-status to occupied and live
  layout-candidate path sets; no versioned SCM-neutral, status-rich lossless-
  delta contract has landed.
- `pipeline/facade/path-layout-app` evaluates candidate repository objects.
- The existing layout workflow builds that application from
  ruleset-selected protected source and contributes to the one `presubmit`.

No first-party declaration call, SCM-neutral snapshot contract, canonical
ChangeSet write/CAS behavior, qualification state, repair campaign, or enforced
declaration verdict has landed. Build owns the not-yet-landed semantic engine
and `DeclarationRepairSet`; this plan does not implement or redefine it.

ADR-0719 D-18's one graph/queue/schedule engine, shared internal/sold semantics,
Compute worker boundary, promotion graph, and hosted-adapter retirement remain
Pipeline's broader destination. This declaration sequence neither implements
nor forks that programme; its repository facts and ChangeSet contracts are
reusable inputs to it.

</baseline>

<sequence>

## Adopt Pipeline owner law

Class: documentation/authority only; depends on the ADR-0719 source-declaration
amendment merging unchanged.

- Add `pipeline/{ADR,PRD,SPEC,PLAN}.md` as one four-file D-36 unit.
- Record the current trust seam, Pipeline/Build split, SCM-neutral repository
  facts, ChangeSet application, qualification, repair, success/failure/SLO, and
  fault contracts without claiming implementation.
- Keep code, manifests, lockfiles, workflows, Build paths, and other owners
  read-only.

Closed write envelope: these four files only.

- Success: all four files agree, every load-bearing MUST has five fields, and
  hostile review cannot derive declaration semantics or landed behavior from
  Pipeline law.
- Failure: root-law amendment, Build prescription beyond the accepted
  interface, code/workflow change, duplicate graph authority, or implementation
  claim.
- Rollback/fault: revert only these files; missing one file, stale central-law
  base, or a cross-owner claim fails review.

## Establish immutable repository facts

Class: test-driven Pipeline core/port/adapter behavior after owner-law merge.

- Introduce versioned SCM-neutral snapshot identity, entry, path-delta, owner-
  fact, compare, and publication capabilities behind Pipeline-owned ports.
- Bind owner facts to one protected owner-authority identity/revision and model
  expected owner or absence per path; missing or ambiguous mappings refuse.
- Adapt the current Git implementation without leaking command/SHA/checkout
  concepts into core; preserve the existing read API during a bounded migration
  and retire it only after all consumers move.
- Add explicit bounds, stable refusal categories, immutable-object reads,
  environment/hook/config isolation, and owner ambiguity refusal.

The implementation plan names exact leaf crates/files and keeps one writer per
leaf. Any new package, root workspace/lock edit, agreed port, or facade contract
is a separate structural D-29 lane before behavior.

- Success: identical immutable repository inputs yield identical normalized
  facts; Git fault/encoding/kind/owner fixtures refuse without panic or mutable
  checkout dependence.
- Failure: Git types in core/Build, working-tree reads, lossy paths, default
  owner, ambient process behavior, or combined structural/behavior mutation.
- Rollback/fault: retain the current protected layout path while the unrouted
  port is withdrawn; inject every repository read/compare and owner boundary.

## Integrate protected declaration admission

Class: Pipeline facade behavior after the Build check-only contract and
immutable repository facts are independently reviewed and qualified for
integration.

- Extend the existing protected path-layout application to gather complete HEAD
  declaration entries and owner facts and invoke the one Build engine.
- Validate the versioned result without interpreting declaration semantics;
  render sorted violations and preserve all Build refusals.
- Add protected-policy, complete-head, trigger, timeout/crash/malformed-output,
  limit, and one-fan-in tests.

The consumed Build facade is an external contract: its adoption requires
Pipeline, Build, and architecture review under D-29 rather than a drive-by path
dependency.

Prefer no workflow edit: the trusted layout application already executes in
the required seam. If protected scheduling changes are actually necessary,
route `.github/workflows/presubmit.yml` as its own escalated repo-root D-29 lane;
do not smuggle it into facade behavior.

- Success: either declaration side triggers one protected complete-head verdict
  inside existing layout admission, and candidate source cannot select or skip
  the engine.
- Failure: candidate-compiled checker, Pipeline parser, delta-only proof,
  required Buck/Cargo invocation, second context/job, compatibility target, or
  false-green engine failure.
- Rollback/fault: withdraw the unrouted call before activation; tests prove
  engine absence, panic, timeout, malformed identity, excessive output, and an
  unqualified profile remain nonblocking qualification failures before first
  activation and become fail-closed after activation.

## Implement canonical ChangeSet application

Class: test-driven Pipeline kernel and SCM adapter behavior after immutable
repository facts; independent of protected admission files when exact leaf
envelopes are disjoint.

- Freeze the canonical ChangeSet value, identity encoding, full semantic
  read/write expectations, complete postimages, protected owner-authority
  identity/revision, sorted per-path owner-or-absence preconditions, routing-only
  owner shard, and typed outcomes.
- Wrap Build repair sets without semantic loss; re-read and compare every
  declared semantic fact, owner-authority identity/revision, and bound per-path
  owner fact against one current immutable snapshot.
- Construct and publish one successor commit/branch or none; surface
  indeterminate adapter outcomes honestly and make retries idempotent.
- Keep PR creation/review/merge in orchestration adapters. No direct `dev`
  writes or automatic APPROVE enter the kernel.

- Success: property/model tests prove semantic conflicts refuse, unrelated
  commits remain applicable, retries converge, and no fault exposes partial
  success.
- Failure: whole-head locking, text-hunk authority, destination-only compare,
  owner-shard-as-authority, missing per-path owner CAS, owner crossing, partial
  working-tree mutation, or Git-only ChangeSet core.
- Rollback/fault: leave repair output non-applying; inject every read, owner,
  compare, tree, commit, push, and PR transition including ambiguous completion.

## Qualify one exact engine profile for repair

Class: protected out-of-presubmit qualification after the engine, repository
facts, protected exchange, and ChangeSet contracts pass their own tests.

- Run adversarial fixtures for every admitted/refused source form, modeled
  relation, owner shape, bound, side-effect, and deterministic replay contract.
- Differentially compare protected facts with
  `cargo metadata --offline --locked` and non-building Buck queries on isolated
  snapshots. Keep those invocations out of the engine and required admission.
- Measure the PRD safety, coverage, determinism, atomicity, availability,
  latency, and linear-work targets for the exact protected identity.
- Record `RepairQualified` only when independent review accepts the engine,
  repair, trust, determinism, resource, and differential evidence needed to
  open application-ready repair PRs; any bound identity change invalidates it.

- Success: the exact profile reaches `RepairQualified` with zero unresolved
  semantic mismatch, side effect, nondeterminism, unbounded path, or repair-
  application fault gap; declaration admission remains nonblocking.
- Failure: compile proof substituted for source semantics, required-path tool
  invocation, hand-authored waiver, current-count threshold, or latency target
  weakened after measurement.
- Rollback/fault: because no profile has activated in this initial sequence,
  return only the candidate to `Unqualified`; perturb every identity and inject
  false green, false failure, timeout, process/network/write attempt, and
  differential disagreement.

## Repair one owner canary

Class: owner-sharded repair campaign after qualification; the paused IAM stale-
label closure is the first canary only if its owner revalidates the live drift.

- Re-evaluate current `dev`, emit one owner repair shard, wrap/apply it through
  ChangeSet, and open one isolated protected PR.
- Require owner review, green existing `presubmit`, resolved threads, merge,
  and a clean complete-head replay before expanding.
- Do not hand-edit labels, bake observed paths/counts into policy, or treat the
  canary's owner/path set as a permanent registry.

Write envelope: exactly the canary owner files declared by the repair set; no
Pipeline/Build/root file shares the repair PR.

- Success: the canary repairs only declared drift, survives a disjoint commit,
  refuses an injected intervening change that mismatches either declared
  semantic set, merges through normal governance, and replays clean.
- Failure: manual repair, cross-owner change, missing postimage/precondition,
  bypassed review, or clean result that depends on a baseline.
- Rollback/fault: revert the canary PR through normal review; inject disjoint and
  overlapping commits, owner movement, push/PR ambiguity, and post-merge replay
  failure.

## Fan out owner repair campaigns

Class: N owner-local PRs after the canary passes end to end.

- Re-evaluate one current snapshot and dispatch one ChangeSet/PR per owner
  shard with disjoint write sets, independent reviewers, and ordinary path
  occupancy.
- Re-evaluate after each merge wave so semantic reads and owners remain current;
  regenerate conflicted shards rather than rebasing postimages manually.
- Keep a nonblocking monitor for PR/check transitions while disjoint owner lanes
  continue; observation never grants merge authority.

- Success: every detected violation is repaired, every owner shard has a
  protected merged record, and complete current `dev` evaluates clean.
- Failure: overlapping owner writes, fleet-wide PR, merge-queue waiting that
  stalls disjoint work, manual conflict repair, or unresolved legacy allowlist.
- Rollback/fault: stop only affected shards; regenerate from current immutable
  facts after overlap, owner change, failed review, or merge conflict.

## Activate and operate protected enforcement

Class: Pipeline activation after qualified evidence and zero current drift.

- After the repair-clean replay, finish admission qualification against every
  PRD SLO and bind that exact identity into existing trusted layout admission;
  prove the one `presubmit` fan-in, alerting, receipts, and rollback transition.
- Exercise relevant Cargo-only and BUCK-only changes, unsupported shapes,
  engine/profile loss, resource saturation, and identity invalidation on the
  protected path.
- Preserve a monotonic protected `ever_enforced` fact. Keep the last-enforced
  immutable profile authoritative while a new identity shadows, and replace it
  only through an atomic admission-qualified transition. If the active profile
  is invalid or unavailable, enter `EnforcementBlocked` and refuse relevant
  changes while remediation proceeds in separate Build, Pipeline, or owner
  lanes.

- Success: every relevant candidate gets one complete-head protected verdict;
  the qualified SLOs hold; no known current violation, false green, secondary
  context, compile plane, or census exists.
- Failure: skipped/untrusted/unqualified verdict, admission outage reported as
  green, unresolved drift, SLO breach without refusal, post-activation fallback
  to nonblocking shadow behavior, non-atomic replacement, or a candidate-
  controlled downgrade.
- Rollback/fault: atomically restore a still-qualified immutable prior profile;
  when none is valid, remain `EnforcementBlocked` rather than disabling
  enforcement. Inject profile loss, limits, timeouts, corrupted facts, false
  verdicts, replacement failure, and fan-in failure.

</sequence>

<parallelism>

Repository-fact behavior and ChangeSet behavior may overlap only after their
exact Pipeline leaf crates and shared-file writers are disjoint. Protected
facade integration waits for the Build interface and repository facts.
Qualification waits for both protected admission and ChangeSet evidence. One
owner canary completes before repair fan-out; later owner shards may run in N
parallel worktrees because their write sets are disjoint. Root workflow,
workspace membership, Cargo lock, agreed ports, and each shared facade manifest
remain serialized D-29 structural lanes.

</parallelism>
