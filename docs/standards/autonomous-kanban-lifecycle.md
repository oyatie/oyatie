---
doc_class: Standard
title: Autonomous Kanban Lifecycle Convention
status: Accepted
date: 2026-07-01
owner_team: council-architecture + axis-foundry
classification: INTERNAL_ONLY
related:
  - ../AGENTS.md
  - ../../AGENTS.md
  - ../../specs/root-hub-pointers.json
  - agentic-dev-team-optimization.md
  - ../checklists/done-definition-checklist.md
  - ../../templates/pull-request-template.md
planned_enforcement_ref: Hermes Kanban steward + review/fix child-card sweep
---

# Standard: Autonomous Kanban Lifecycle Convention

## Purpose

Oyatie autonomous work uses one visible Hermes Kanban lifecycle spine so agents,
reviewers, and stewards can see where work is blocked without reading chat. This
standard defines the card convention for:

Research -> Plan -> Spec -> RED test/repro -> Build/GREEN ->
Simplify/refactor/test expansion -> Security hardening -> Review/fix loop ->
Merge -> Rollout/E2E verification -> Learning/observation harvest.

The convention is deliberately a repo standard and card template, not a new
Kanban engine. Hermes Kanban already owns task rows, parent/child links,
comments, workspaces, dispatcher caps, block kinds, and idempotency keys.
Forward-compatible fields such as `workflow_template_id` and `current_step_key`
remain extension seams until a separate accepted implementation makes them live.

## Card body template

Every card created or touched by the autonomous PR lifecycle directive MUST be
normalized to this section shape before dispatch or closeout. Empty sections are
not allowed: use `N/A — <bounded reason>` only when the skip rules below permit
it and the replacement proof is named.

```markdown
## Research / source intake
- Sources read: <repo docs, code paths, PRs/issues, parent artifacts, board cards>
- Current-state facts: <live facts with timestamp/tool/source>
- Duplicate/no-action check: <searched card ids, PRs/issues, paths, result>

## Plan / scope control
- Owner/profile: <real profile or human/terminal lane>
- Workspace/worktree: <kind, absolute path or project slug, branch>
- Path scope: <allowed paths, forbidden paths, paths touched/expected>
- Conflict class: <read-only|disjoint|shared-root|generated-controller-owned|merge-integrator|human-gate>
- Dependencies: <parent/child cards, PRs/issues, idempotency keys>
- Non-goals: <explicit exclusions>

## Spec / acceptance contract
- User-visible or governance outcome: <one outcome>
- Acceptance criteria: <observable checks>
- Rollback expectation: <procedure or N/A rationale>
- Observability expectation: <golden signal/audit/log evidence or N/A rationale>
- Release-governance note: <release-note impact or N/A rationale>

## RED test evidence
- RED command/repro/transcript: <command or artifact>
- Expected failure and observed failure: <why it fails for the right reason>
- Test-after exception: <N/A unless permitted by this standard>

## GREEN implementation evidence
- Changed paths/PR/worktree/head: <bounded diff evidence>
- GREEN command/results: <tests/checks proving the intended behavior>
- Generated-file policy: <none|controller-materialized-only|source-owned>

## Simplification / refactor / test expansion
- Simplification performed or skipped with reason: <complexity removed/bounded>
- Edge/error/security tests added or N/A rationale: <evidence>
- Regression scope kept green: <commands/results>

## Security hardening
- Review scope: <secrets, input validation, authn/authz, tenant/privacy, path traversal, shell/SQL, generated files as applicable>
- Findings/fixes/deferred items: <cards or explicit N/A>
- Verification: <commands/review verdicts/transcripts>

## Independent review/fix loop
- Review/fix card or PR review handle: <id/url>
- Required lenses: <correctness/spec, RED/test, security/privacy, architecture/productization, ops/rollback/observability, UX/a11y if user-facing, docs/traceability, simplification>
- Verdict/fix loop: <APPROVE or REQUEST_CHANGES -> fix evidence -> rerun evidence>

## PR / CI / merge evidence
- PR/branch/head: <url, branch, commit>
- Required status: <oya-ci-required URL/result>
- Merge readiness: <review threads resolved, no conflict, branch protection satisfied>
- Merge evidence: <merge commit/mergedAt or human-blocked exception>

## Post-merge closeout and learning
- Rollout/E2E evidence or N/A rationale: <browser/user-story, API/CLI replay, or non-runtime proof>
- Rollback and observability confirmation: <evidence>
- Agent-observation harvest: <created/linked cards or duplicate/no-action rationale>
- Final state: <why this is complete, not merely an opened PR>
```

## Required card fields

Every lifecycle card body MUST include these fields before it is dispatchable:

```yaml
source_context: <root card ids, upstream audit/research artifacts, PR/issues/docs>
stage: <Research|Plan|Spec|RED test/repro|Build/GREEN|Simplify/refactor/test expansion|Security hardening|Review/fix|Merge|Rollout/E2E|Learning>
owner_profile: <real Hermes profile or documented terminal/human lane>
workspace_kind: <scratch|dir|worktree>
workspace_path_or_project: <absolute path, project slug, or N/A for read-only>
worktree_branch: <branch name or N/A>
allowed_paths: [<repo-relative prefixes or semantic symbols>]
forbidden_paths: [<generated faces, shared roots, secrets, unrelated paths>]
paths_touched_or_expected: [<repo-relative paths or N/A for read-only>]
path_conflict_class: <read-only|disjoint|shared-root|generated-controller-owned|merge-integrator|human-gate>
objective: <one outcome the card owns>
non_goals: [<explicit exclusions and adjacent work not owned here>]
acceptance_criteria: [<observable exit checks>]
evidence_required: [<commands, artifacts, PR/check URLs, browser/API transcripts>]
rollback_expectation: <rollback path, N/A rationale, or human-only dependency>
observability_expectation: <golden signal, audit/log evidence, or N/A rationale>
release_governance_note: <release-note impact, no-release-impact rationale, or human-only blocker>
review_lenses: [<required lenses or N/A rationale>]
dependencies_and_links: <parent/child card ids or intended deterministic idempotency keys>
duplicate_check: <searched ids/titles/paths/PRs and result>
skip_rationale: <stage skips, if any, with source evidence>
test_order_policy: <RED-before-GREEN|test-after permitted because ...>
human_blocked_exceptions: <credentials/approval/access/leader gate, or N/A>
bounded_release_gate: <exact release/merge batch and hold card, or N/A>
idempotency_key: <stable key for retryable automation, or N/A>
verification_path: <exact command/check/user-story used by this card>
```

Required fields MAY be presented as prose instead of YAML, but the same facts
MUST be present and unambiguous.

## Stage exits and evidence

| Stage | Exit evidence |
|---|---|
| Research | Source files/artifacts read, current-state facts, risks, duplicate/no-action candidates, and no product mutation unless explicitly assigned. |
| Plan | Bounded sequence, owners, dependencies, conflict classes, downstream review/merge/rollout/learning cards, and non-goals. |
| Spec | Acceptance criteria, user-visible outcome, constraints, rollback/observability expectations, and exact verification path. |
| RED test/repro | Failing test, repro script, browser/API transcript, or explicit N/A rationale for docs/research/process-only work. |
| Build/GREEN | Changed paths/PR/worktree, implementation evidence, GREEN checks, and confirmation the diff stays within allowed paths. |
| Simplify/refactor/test expansion | Complexity deleted or bounded, no drive-by refactors, edge/error/security tests expanded when applicable, and evidence that tests still target the intended behavior. |
| Security hardening | Proportional checks for secrets, input validation, authn/authz, tenant/privacy boundaries, path traversal, shell/SQL injection, generated-file discipline, and deferred findings routed to cards. |
| Review/fix loop | Independent verdicts, findings, fixes, rerun evidence, and final APPROVE or documented human-only blocker. |
| Merge | Protected PR against `dev`, review threads resolved, no merge conflict, and `oya-ci-required` green. |
| Rollout/E2E verification | Real shipped surface evidence: browser/user-story, API/CLI replay, rollout status, rollback note, and observability check, or N/A rationale for non-runtime work. |
| Learning/observation harvest | Agent observations deduped; follow-up/maturity/feature/fix cards created or linked; duplicate/no-action rationale recorded. |

Review/fix cards MUST name the required lenses. Use the smallest set that fits
the change class, but code and product changes normally include correctness/spec,
RED/test coverage, security/privacy, architecture/productization, operations
(observability/rollback/release), UX/accessibility when user-facing, docs/traceability,
and over-engineering/simplification.

## RED/GREEN and test-after policy

Bugfix and feature implementation cards MUST prove RED before GREEN: the card
names the failing test/repro/transcript, shows it fails for the intended reason,
then records the implementation and rerun evidence that makes the same check
green. A worker MUST NOT replace RED with broad post-hoc tests simply because the
code was easy to change.

Test-after is permitted only for these bounded cases, and the card MUST say which
case applies:

- read-only research, inventory, triage, status-mapping, or duplicate/no-action
  work where there is no product/repo behavior to make fail;
- documentation-only convention/template work, where replacement proof is
  read-back, link/grep coverage, schema/markdown validation where available, and
  downstream Kanban comments that point to the authoritative template;
- external/human-only gates such as credentials, branch protection settings,
  regulator/leader approval, or production access, where the card records the
  blocker and any safe preparatory evidence;
- legacy or unsafe-to-automate seams where an immediate RED test would require a
  broader platform change; the card MUST create/link follow-up work for the test
  seam unless an existing card covers it.

Opened PR is never a completion condition. A lifecycle card that mutates the repo
is complete only after review approval, `oya-ci-required` green, conflict-free
merge readiness, merge evidence, and post-merge closeout evidence exist, unless
the card is explicitly blocked on a named human-only exception.

## Linking rules

Hermes Kanban links are `parent_id -> child_id`; the child waits until every
parent is done or archived. Use that direction consistently:

1. For a lifecycle chain, link each predecessor as a parent of the next stage.
   Example: Research gates Plan; Plan gates Spec; RED gates Build/GREEN;
   Build/GREEN gates Simplify/refactor/test expansion; Simplify gates Security
   hardening; Security hardening gates Review/fix; Review/fix gates Merge;
   Merge gates Rollout/E2E; Rollout/E2E gates Learning.
2. For a high-level root/closeout card, make all required stage cards parents of
   the root/closeout card so it only wakes when the spine is complete.
3. For retryable automation, create stage cards with deterministic idempotency
   keys derived from `<root-card>:<stage>:<scope>`. Do not create duplicate stage
   cards when an active card already covers the same source context and path set.
4. Shared-root, generated-face, release/governance, and merge-integrator cards
   MUST serialize conflicting downstream cards. Do not rely on prose to prevent
   two workers from editing the same root.
5. A card that discovers follow-up work MUST create or link a follow-up card with
   source context, classification, affected card/PR/artifact, acceptance criteria,
   verification path, suggested owner/profile, and dependencies/conflict notes.

## Implementation-to-review closeout rule

An implementation worker MUST NOT park its own finished work as
`blocked/review_required`, `review-required`, or vague `needs_input` merely
because review is next. Before closeout, it MUST create or link a dedicated
Review/fix lifecycle card that owns the review-required state.

The Review/fix card MUST include:

- source implementation card and PR/worktree/changed paths;
- required review lenses and reviewer owner/profile;
- acceptance criteria for APPROVE vs REQUEST CHANGES;
- exact verification commands or browser/API replay;
- path-conflict class and generated-face policy;
- fix-loop rule: REQUEST CHANGES reopens/creates a fix card, reruns the failed
  evidence, and returns to Review/fix until APPROVE or a true blocker exists.

If review-card creation or linking fails, the implementation worker blocks on
that concrete capability failure. It MUST NOT collapse the failure into a
passive review-required block on the implementation card.

## Bounded active work rules

- At most one mutating stage in the same lifecycle spine SHOULD be `running` per
  owner/profile. Downstream stages SHOULD be `todo`, `scheduled`, or parent-gated
  until their predecessor evidence exists.
- Read-only Research/Plan/Review lanes MAY run in parallel only when their path
  sets and source contexts are disjoint or explicitly read-only.
- `path_conflict_class=shared-root`, `generated-controller-owned`, or
  `merge-integrator` requires a serialized integrator/owner card before worker
  fan-out.
- Generated `*.generated.json` faces are controller-owned. Lifecycle cards MAY
  update source manifests/generators or request materialization, but MUST NOT
  hand-edit generated faces.
- Dispatcher dry-runs and steward sweeps MUST stay bounded by max item count,
  path/conflict filter, or explicit source-card set.
- Do not implement a one-cron-per-stage design. A scheduled steward MAY run a
  bounded closed-loop audit that dedupes, comments, creates/links concrete gaps,
  and escalates true human blockers; it MUST NOT become ten independent stage
  crons that race or spam the board.
- Do not mass-dispatch lifecycle work without card-body path/conflict metadata.
  Broad release or merge waves MUST pass through a bounded release gate that
  names the exact candidate set, shared roots, dependency edges, dry-run evidence,
  and non-selected held cards. If that gate cannot be represented safely with
  native Kanban links/statuses, block the gate as a capability/human exception.

## When a stage MAY be skipped

A skip is allowed only when the card records `skip_rationale` with source
evidence and replacement proof:

- Research MAY be skipped when current, cited research already exists and the
  card links it.
- Plan MAY be skipped only when an accepted plan/seed already gives owners,
  dependencies, conflicts, non-goals, and verification.
- Spec MAY be skipped for pure mechanical cleanup only if the existing card body
  already contains acceptance criteria and constraints.
- RED test/repro MAY be skipped for read-only research, documentation-only
  convention work, or impossible-to-automate external gates; record the smallest
  replacement proof such as read-back, link validation, or transcript evidence.
- Build MAY be skipped when no product/repo mutation is required.
- Simplify/refactor MAY be skipped when the diff is already smallest-actionable
  and no complexity is introduced.
- Review/fix MUST NOT be skipped for repo mutations that require PR or merge
  readiness. It MAY be N/A only for read-only research/ops artifacts with no repo
  diff, and the card MUST say why.
- Merge MAY be skipped when no PR/repo mutation exists.
- Rollout/E2E MAY be skipped when there is no runtime/user surface; record the
  substitute evidence and rollback/observability N/A rationale.
- Learning/observation harvest MUST NOT be skipped. If there is no follow-up,
  record duplicate/no-action rationale.

## Duplicate and no-action handling

Before creating a new lifecycle/follow-up card, the worker or steward MUST search
active cards by affected card id, PR, artifact path, changed path, title, and
idempotency key. If an existing card covers the work, comment/link that card
instead of creating a duplicate.

When no new card is created, record a no-action note on the source card with:
source context, classification, existing covering card ids or reason no work is
safe, verification performed, and the next recheck trigger if any. Valid
no-action classes are `already_covered`, `already_merged_verified`,
`intentional_human_gate`, `unsafe_without_approval`, `obsolete`, and
`not_actionable_after_verification`.

## Completion packet

A lifecycle root is complete only when every non-skipped stage has evidence and
every skipped stage has a rationale. The final packet MUST include changed paths,
tests/checks run, security-hardening evidence or N/A rationale, PR/CI/merge state
if any, rollout/E2E or N/A evidence, rollback and observability notes, review/fix
verdicts, release-governance/release-note impact, and observation-harvest outcome
(created/linked card ids or duplicate/no-action rationale).

## Sources scanned

- `AGENTS.md`; `docs/AGENTS.md`; `docs/standards/agentic-dev-team-optimization.md`.
- `docs/decisions/ADR-0363-retire-agentic-vcs-platform-to-intelligence-on-github-substrate.md`.
- `docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md`.
- `templates/pull-request-template.md`; `templates/checklists/done-definition-checklist.md`.
- Parent Kanban artifacts for `t_16000cb6` and `t_63c61ae1`.
- Autonomous lifecycle directive graph: `t_99a1fc0b`, `t_136f526e`, `t_91368d25`,
  `t_1190c8f8`, `t_1babf8cc`, and `t_7d095645`; inventory report
  `/tmp/oyatie_inventory_t_136f526e/inventory.md`.
